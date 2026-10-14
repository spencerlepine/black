# fndiff spec: overlay-21 script command file scrcmd_c03center.c (F8-spec4), Thumb code 0x021B5A18..0x021B66B8 (F1-SPLIT row
# 21.50).  __FILE__ "scrcmd_c03center.c" (0x021DE25C) is passed by 0x021B5A18 (cmd 307).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_c03center --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  256 021b627c  261 021b631c  262 021b63d4  263 021b5b4c  265 021b636c  285 021b5b8c  286 021b5bfc  287 021b5c80
#  288 021b65d8  307 021b5a18  446 021b654c  447 021b647c  462 021b5eb8  463 021b5df8  484 021b60f4  485 021b6164
#  486 021b6124  487 021b61e8  566 021b62c8  567 021b5ce8  568 021b6590  578 021b6628
# 28 functions specced (22 command ids).  Despite the name (C03 = Castelia City) nothing here touches the comm / Wi-Fi /
# C-Gear state: no X1 networking command, all specced as logic.
#  * 307: turn tween (TCB 0x021B5B0D on the u16 BSS word 0x021EE724, rounded to a multiple of 0x4000).
#  * 285 / 286 / 287 / 288 / 578: party / record queries into a save object (02012F08 / 02012FD8: 0200C11C, 0200C2B0, 0200C2D0,
#    0200E7BC, names 0200E740 / 0200E7AC); 567: EV sum / max / single EV; 256: any party mon with field 0x97 (Pokerus?).
#  * 462 / 463: counters / thresholds of 0200CA1C(GD) (0200CF08 / 0200D0F8 / 0200CF54 with the tables 0x021D992C / 9904 / 98DC).
#  * Move tutors 484 / 485 / 486 / 487: 021b5f24 = the tutor move by kind (0x1B2 Draco Meteor; 0x152 / 0x133 / 0x134 ultimate
#    moves; 0x208 / 0x207 / 0x206 pledges, by 0201891C type checks), 021b5fb8 = can learn, 021b6048 = party scan (mask, level
#    0xFF from 021b5fb4), 487 starts 0215A0E8; 566 move from 0202130C(0x151) (Relearner?).
#  * Trades 446 / 447 (fld_trade.c 021CBFB0 / 021CC060 / 021CC068 / 021CC500); events 261 021C5AAC, 263 0215A07C, 265 0215A2E4,
#    262 02166C4C (first egg-like party mon); 568: a random item of the bag pocket 2 (02007FA0 / 020081DC / 020212B0).
# Not specced (0):
#   none
# Fragments (not functions): 0x021B6100, 0x021B6294, 0x021B63BC (mid-function return points that Ghidra / data words made
# entries), 0x021B6158, 0x021B636E (decoded data words).
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
area scrcmd_c03center
set field

# cmd 307 (turn tween): fm; T = 02188D30(fm) (TCB system), H = 02188C84(fm) (heap), E = 02188C58(fm), X = 02188C7C(fm); n =
# value (frames); W = Heap_AllocDebug(H, 0x10, 1, "scrcmd_c03center.c", 0x5D); g = 0x021EE724 (u16): *g = *0218ED50(E);
# 0216CDB8(X, g); a = 0218FE70(E)+0x26 (u16); W = {*g, delta, 0, n} with delta = (a rounded to the nearest multiple of 0x4000)
# - *g, wrapped into -0x8000..0x7FFF; 02030DA8(T, 0x021B5B0D, W, 0) (TCB add); 0.
# F8e-spec2: the target angle a (arg1 + 0x566) is drawn at the rounding ties (a = 0x2000 mod 0x4000) and next to them.
fn 021b5a18 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x566 u16 alt oneof 0x2000 0x6000 0xA000 0xE000 || oneof 0x1FFF 0x2001 0x5FFF 0x6001 0xDFFF 0xE001 || any
  global 021ee724 4
  stub 0215a5e4 force ret=u32 args=v
  stub 02188c84 force ret=u32 args=v
  stub 0216cdb8 force ret=none args=v,v
  stub 02030da8 force ret=none args=v,v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02188d30 force ret=u32 args=v
  stub 02188c7c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0218ed50 force ret=ptr:arg1:0x500:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 0218fe70 force ret=ptr:arg1:0x540:1 args=v
  stub 02188c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# TCB of cmd 307 (tcb, W): W[2] += 1; *g = (u16)W[0] + _s32_div(W[2] * W[1], W[3]) (u16, g = 0x021EE724); W[2] >= W[3] ->
# Heap_Free(W), 02030EAC(tcb) (TCB delete).
# V-0028: n = 0 is drawn too (_s32_div_f returns the dividend for a zero divisor; i >= 0 ends the task at once).
fn 021b5b0c ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x40
  global 021ee724 4
  fix arg1 8 u32 alt range 0 8 || any
  fix arg1 0xC u32 alt const 0 || range 1 8 || range 9 0xFFFFFFFF
  stub 020307b0 force ret=none args=v
  stub 02030eac force ret=u32 args=v

# cmd 263: a, b = vars, (value ignored); ev = 0215A07C(GAMESYS, a, b); child event; 1.
fn 021b5b4c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a07c force ret=u32 args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 285: idx; 0215B4AC(ctx, idx, &mon) == 1 -> 0200C11C(02012F08(GAMEDATA), get(mon, 5), get(0x6F), get(0x4C), get(0x6E))
# (PokeParam 02017E1C(mon, id, 0)); 0.
fn 021b5b8c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02012f08 force ret=u32 args=v
  stub 0215b4ac force ret=pick:0/1 args=v,v,out:4
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0200c11c force ret=none args=v,v,v,v,v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# cmd 286: party = 02012934(GAMEDATA); v; arr[6] = 0; for i < 0201AA18(party): mon = 0201AB28(party, i), get(mon, 0x4C) == 0
# (not an egg) -> arr[k++] = get(mon, 5) (species); 0200C2B0(02012F08(GAMEDATA), v, arr); 0.
fn 021b5bfc ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02012f08 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0200c2b0 force ret=none args=v,v,in:12
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# cmd 287: L = 02158F74(SW); v, var; 0200C2D0(02012F08(GAMEDATA), v, arr) (fills u16 arr[6]); every nonzero arr[i] ->
# 0201EE60(L, k++, arr[i]) (word set); *var = k; 0.
fn 021b5c80 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201ee60 force ret=none args=v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f08 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0200c2d0 force ret=none args=v,v,outm:12:1/0/0xFF/0xFF
  stub 02158f74 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v

# cmd 567: idx, sel, var; 0215B4AC(ctx, idx, &mon) == 0 -> *var = 0; else e[0..6] = get(mon, 0x46..0x4B) (EVs), sum (u16), max;
# sel 0 -> *var = sum, 1 -> max, 2..7 -> e[sel - 2], else nothing; 0.
fn 021b5ce8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=pick:0/1/2/3/4/5/6/7/8 args=v,v seq
  stub 0215b4ac force ret=pick:0/1 args=v,v,out:4
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq

# table_lookup(u16 pairs[][2], v): first pair with key == 0 or v <= key (signed) -> its value.
fn 021b5dd0 ret=u16 inputs=200
  nowork
  arg buf 0x40
  arg u16 any
  fix arg0 0 u16 alt range 0 0x20 || any rep=14,4
  fix arg0 0x38 u32 const 0

# cmd 463: k = value, A, B = vars; h = 0215A5CC(ctx); R = 0200CA1C(GAMEDATA); 0200CA38(R); *A = 5, *B = 0; k 1: n = 0200CF08(R,
# h), *A = 0200D298(R) == 1 ? 0x32 : 021b5dd0(0x021D992C, n); k 2: n = 0200D0F8(R, h), *A = 0200D2C4(R, h) == 1 ? 0xD :
# lookup(0x021D9904, n); k 3: n = 0200CF54(R, h), *A = 0200D2B0(R, h) == 1 ? 0x1E : lookup(0x021D98DC, n); *B = n; 0.
fn 021b5df8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200d0f8 force ret=u32 args=v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 0200d2c4 force ret=pick:0/1 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0200cf08 force ret=u32 args=v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0200d298 force ret=pick:0/1 args=v
  stub 02159b10 force ret=pick:0/1/2/3/4 args=v,v seq
  stub 0200d2b0 force ret=pick:0/1 args=v,v
  stub 021b5dd0 ret=u32 args=v,v
  stub 0200cf54 force ret=u32 args=v,v
  stub 0200ca38 force ret=u16 args=v
  stub 0200ca1c force ret=u32 args=v

# cmd 462: R = 0200CA1C(GAMEDATA); var; k = value: 1 -> *var = 0200D298(R); 3 -> 0200D2B0(R, 02188C84(fm)); 2 ->
# 0200D2C4(R, 02188C84(fm)); else nothing; 0.
fn 021b5eb8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02188c84 force ret=u32 args=v
  stub 0200d2c4 force ret=u32 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/3/4 args=v,v seq
  stub 0200d2b0 force ret=u32 args=v,v
  stub 0200d298 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0200ca1c force ret=u32 args=v

# tutor_move(mon, k): k 0 -> 0x1B2; k 1: 0201891C(mon, 3 / 4 / 5) -> 0x152 / 0x133 / 0x134; k 2: (mon, 0 / 1 / 2) -> 0x208 /
# 0x207 / 0x206; otherwise 0x133 (the ultimate / pledge moves by type).
fn 021b5f24 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 range 0 3
  stub 0201891c force ret=pick:0/0/1 args=v,v

# tutor_max_level(k): 0xFF.
fn 021b5fb4 ret=u8 inputs=200
  nowork
  arg u32 any

# tutor_can_learn(mon, k): k 0: 0201891C(mon, 6); k 1: any of (mon, 3 / 4 / 5); k 2: any of (mon, 0 / 1 / 2); else 0.
fn 021b5fb8 ret=bool inputs=200
  nowork
  arg u32 any
  arg u32 range 0 3
  stub 0201891c force ret=pick:0/0/1 args=v,v

# tutor_party_check(vm, ctx, k, u8 *mask, level_check (stack)): party; n = 0201AA18(party); *mask = 0; for i < n: mon not an
# egg (get 0x4C != 1) and 021b5fb8(mon, k) -> any = 1; !level_check or get(mon, 9) >= 021b5fb4(k) -> ok = 1, *mask |= 1 << i;
# returns any == 0 ? 1 : ok == 0 ? 2 : 0.
fn 021b6048 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  arg u32 range 0 3
  arg buf 0x10
  arg u32 alt const 0 || any
  stub 0215a5e4 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02012934 force ret=u32 args=v
  stub 021b5fb4 ret=u32 args=v
  stub 021b5fb8 ret=u32 args=v,v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v
  # closure: 021b5fb4 / 021b5fb8
  stub 0201891c force ret=pick:0/0/1 args=v,v

# cmd 484: k, var; *var = 021b6048(vm, ctx, k, &mask, 1); 0.
fn 021b60f4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 021b6048 ret=u32 args=v,v,v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  # closure: 021b6048
  stub 0215a5e4 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02012934 force ret=u32 args=v
  stub 021b5fb4 ret=u32 args=v
  stub 021b5fb8 ret=u32 args=v,v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v
  stub 0201891c force ret=pick:0/0/1 args=v,v

# cmd 486: k, idx, var; *var = 0215B4AC(ctx, idx, &mon) == 1 ? 021b5f24(mon, k) : 0; 0.
fn 021b6124 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 0215b4ac force ret=pick:0/1 args=v,v,out:4
  stub 021b5f24 ret=u32 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  # closure: 021b5f24
  stub 0201891c force ret=pick:0/0/1 args=v,v

# cmd 485: k, idx, var; *var = 1; 0215B4AC(ctx, idx, &mon) == 0 -> 0; !021b5fb8(mon, k) -> 1; 0215B500(mon, 021b5f24(mon, k))
# == 1 (knows it) -> 3; get(mon, 9) < 021b5fb4(k) -> 2; else 0; 0.
fn 021b6164 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215b500 force ret=pick:0/1 args=v,v
  stub 021b5f24 ret=u32 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215b4ac force ret=pick:0/1 args=v,v,out:4
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 021b5fb4 ret=pick:0/1/0xFF args=v
  stub 021b5fb8 ret=pick:0/1 args=v,v
  stub 02017e1c force ret=pick:0/1/2/0xFF args=v,v,v
  # closure: 021b5f24 / 021b5fb4 / 021b5fb8
  stub 0201891c force ret=pick:0/0/1 args=v,v

# cmd 487: k, A, B; 021b6048(vm, ctx, k, &mask, 0) != 0 -> *A = 0; else ev = 0215A0E8(GAMESYS, A, B, mask, move) with move =
# k 0: 0x1B2, 1: 0xFFFE, 2: 0xFFFD, else 0xA5 (stack); child event; 1.  (k > 2 reaches 0xA5 only with the 021b6048 stub:
# the real check answers 1 for those k.)
fn 021b61e8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021b6048 ret=pick:0/1/2 args=v,v,v,ptr,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215a0e8 force ret=u32 args=v,v,v,v,v
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  # closure: 021b6048
  stub 0215a5e4 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02012934 force ret=u32 args=v
  stub 021b5fb4 ret=u32 args=v
  stub 021b5fb8 ret=u32 args=v,v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v
  stub 0201891c force ret=pick:0/0/1 args=v,v

# cmd 256: var = 0; party; for i < 0201AA18(party): get(mon_i, 0x97) != 0 -> *var = 1, stop; 0.
fn 021b627c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201ab28 force ret=u32 args=v,v
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 02017e1c force ret=pick:0/1/2/0x1D/0x20/0x249/0x24A args=v,v,v

# cmd 566: var, idx; X = 0202130C(0x151); 0215B4AC(ctx, idx, &mon) and 020188AC(mon, X) -> *var = 02018790(mon), else 0xFFFF; 0.
fn 021b62c8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0215b4ac force ret=pick:0/1 args=v,v,out:4
  stub 0202130c force ret=u32 args=v
  stub 020188ac force ret=pick:0/1 args=v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02018790 force ret=u32 args=v

# cmd 261: var, a, b; ev = 021C5AAC(GAMESYS, var, a, b); child event; 1.
fn 021b631c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 021c5aac force ret=u32 args=v,v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 265: A, B = vars, c, d = values; ev = 0215A2E4(GAMESYS, field map, c, A, B, d == 0, d); child event; 1.
fn 021b636c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a2e4 force ret=u32 args=v,v,v,v,v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/0x20 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 262: var; party; for i < 0201AA18(party): mon; get(mon, 0x4C) != 0 and get(3) == 0 and get(9) == 0 -> ev =
# 02166C4C(GAMESYS, fm, mon), child event, *var = i, 1; none -> 0.  (02188C84 / 02012944 results unused.)
fn 021b63d4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188c84 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201ab28 force ret=u32 args=v,v
  stub 0201210c force ret=u32 args=v
  stub 02166c4c force ret=u32 args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v
  stub 02017e1c force ret=pick:0/0/1 args=v,v,v

# cmd 447 (trade check): var, i, j; mon = 0201AB28(party, j); T = 021CBFB0(02188C84(fm), i) (fld_trade.c); *var = 0;
# get(mon, 0x4C) == 1 -> 1; else get(mon, 5) != 021CC060(T) -> 2; else 021CC068(T) != 2 and get(mon, 0x6E) != 021CC068(T) ->
# 3; 021CC040(T); 0.
fn 021b647c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021cc060 force ret=pick:0/1/2/3 args=v
  stub 021cc040 force ret=none args=v
  stub 02188c84 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201ab28 force ret=u32 args=v,v
  stub 021cc068 force ret=pick:0/1/2 args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 021cbfb0 force ret=u32 args=v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02017e1c force ret=pick:0/1/2/3 args=v,v,v

# cmd 446: a, b; ev = 021CC500(GAMESYS, (u8)a, (u8)b) (trade event); child event; 1.
fn 021b654c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021cc500 force ret=u32 args=v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 568: var; S = 0201292C(GAMEDATA); n = 02007FA0(S, 2); r = 020056EC(); *var = n == 0 ? 0 : 020212B0(*020081DC(S, 2,
# (u16)r)); 0.
fn 021b6590 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02007fa0 force ret=pick:0/1/2 args=v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 020056ec force ret=lt:0 args=v
  stub 0201292c force ret=u32 args=v
  stub 020212b0 force ret=u32 args=v
  stub 020081dc force ret=ptr:arg1:0x500:0x10 args=v,v,v

# cmd 288: Z = 02012FD8(GAMEDATA); i, k; 0200E7BC(Z, k, 0201AB28(party, i)); 0.
fn 021b65d8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201ab28 force ret=u32 args=v,v
  stub 0200e7bc force ret=none args=v,v,v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 02012fd8 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 578: Z = 02012FD8(GAMEDATA); i, k; B = 02158F7C(SW) (strbuf), L = 02158F74(SW) (word set); 0200E740(Z, i) ->
# 02045924(B, 0200E7AC(Z, i)) (copy the name) else 02045924(B, &{02045A98()}) (a one-character string); 0201EE0C(L, k, B, 0, 1,
# 2); 0.
fn 021b6628 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0200e740 force ret=pick:0/1 args=v,v
  stub 02045924 force ret=none args=v,in:2
  stub 02158f7c force ret=u32 args=v
  stub 0200e7ac force ret=ptr:arg1:0x500:0x10 args=v,v
  stub 0201ee0c force ret=none args=v,v,v,v,v,v
  stub 02045a98 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02012fd8 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
