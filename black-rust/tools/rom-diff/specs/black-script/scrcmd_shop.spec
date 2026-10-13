# fndiff spec: overlay-21 script command file scrcmd_shop.c (F8-spec4), Thumb code 0x021B2FD8..0x021B4BEC (F1-SPLIT row 21.46 minus
# its tail: 0x021B4BEC..0x021B4D60 (commands 314 / 351..354 / 358 and the callback of cmd 335) is scrcmd_proc_fld.c, see
# scrcmd_proc_fld.spec).  __FILE__ "scrcmd_shop.c" (0x021DE210) is passed by 0x021B2FD8 (cmd 329).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_shop --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  329 021b2fd8
# 13 functions specced (1 command id): the shop logic (the command, its wait callback and camera step, shop init / mode
# flags, the item lists from the ROM tables, currency, the purchase check and the payment, the count stepper).
#  * cmd 329: kind = value (0xF7..0xFB dynamic lists (02186670), 0xFC / other = mart list b + 6 (W[3]), 0xFD / 0xFE = special
#    lists 1 / 0 (BP shop: W2[0x163] = 1), 0xFF = mart by badge count 0x021D946C[0200C258(..)]); W in the SW + 0x3C slot; the
#    wait callback 0x021B3158 moves the camera (0218Fxxx camera calls), runs the shop UI 0x021B35A4 until it returns nonzero,
#    restores the camera and frees W.
#  * Money = 0200C274 / 0200C29C (W2+0xC), BP = 0200DABC / 0200DA98 (W2+0x18); record counters 0x15 / 0x16 / 0x22 (02008DF0 /
#    02008DAC on W2+0x14).  Item price from 02020FE4(item, 0, heap) through 0201102C (UNVERIFIED role).
# Not specced (36, render / UI, D3: they stay behind hooks; their behaviour is the U-job's spec):
#   021b331c message strings for the shop kind (02045EC0)
#   021b3470 money / BP window draw
#   021b34f4 list input (pad, SE, list menu cursor)
#   021b35a4 shop main loop (UI state machine; calls the specced pay / buy_check)
#   021b385c list row text
#   021b3b04 list menu free thunk
#   021b3b14 message data / word set / strbuf setup
#   021b3b84 message data free
#   021b3bd8 BG / palette setup
#   021b3c58 BG release
#   021b3c78 3D / OBJ resource load
#   021b3cfc OBJ resource load
#   021b3d60 OBJ release
#   021b3dec OBJ actor setup
#   021b3f00 sprite setup
#   021b3f3c sprite release
#   021b3fa0 windows setup
#   021b40a8 windows release
#   021b40d4 money window
#   021b41ac count window
#   021b4264 item info window
#   021b42d0 price window
#   021b43ec list menu setup
#   021b44cc list cursor draw
#   021b4510 list row callback
#   021b45a0 list row callback (bag count)
#   021b468c list scroll arrows
#   021b46fc list position get
#   021b4714 message print
#   021b4804 count prompt print
#   021b4878 count window clear
#   021b4894 count window draw
#   021b4918 count / price print
#   021b4984 count / price print (BP)
#   021b4aa4 count input (pad, SE)
#   021b4ba4 BG screen fill
# Fragments (not functions): 0x021B4376 (a `bl` inside 0x021B42D0 reached by a data word in main).
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
area scrcmd_shop
set field

# cmd 329 (shop): v = value (kind), var (unused); W = Heap_AllocDebug(4, 0x214, 1, "scrcmd_shop.c", 0xFC) into the SW + 0x3C
# slot (02158FB4); W+0x10 = 021904AC(02188C54(fm)), W+0x14 = 02012F08(GD), W+0xC = 0x15 (u16), W+0x18 = 0201292C(GD), W+0x20 =
# 020071CC(SAVE, 0x3A), W+0x1C = 02012F2C(GD), W+0x24 = 020127C4(GD); v in 0xF7..0xFB / 0xFD..0xFF -> W[2] = v, else W[2] =
# W[3] = (u8)v; 020113D0(vm, 0x021B3159) (wait); v != 0xFE / 0xFD -> 021655D4(); 1.
fn 021b2fd8 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02012ebc force ret=u32 args=v
  stub 020127c4 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 02012f08 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 021904ac force ret=u32 args=v
  stub 0201292c force ret=u32 args=v
  stub 020071cc force ret=u32 args=v,v
  stub 02159b10 force ret=pick:0/5/0xF7/0xFB/0xFC/0xFD/0xFE/0xFF/0x100 args=v,v seq
  stub 02012f2c force ret=u32 args=v
  stub 020113d0 force ret=none args=v,v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158fb4 force ret=ptr:arg1:0x440:1 args=v
  stub 02188c54 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 021655d4 force ret=none args=
  stub 0215a5dc force ret=u32 args=v

# shop_camera(W, fm, E): W[1] = 0218F1B4(E); 0218F1AC(E, 0); 0218F848(E); d = 021A3050(02188CB4(fm)) (player dir); 0218ECE4(E,
# &a); 0218ECB4(E, &t); 0218EC54(E); t += the VecFx32 0x021D9708[d]; 0218F918(E, &{a, t, .., +0x30..0x40 0, +0x44 1}, 0xA).
fn 021b30c8 ret=none inputs=200
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 any
  stub 0218ece4 force ret=none args=v,out:12
  stub 0218f848 force ret=none args=v
  stub 0218f1ac force ret=none args=v,v
  stub 021a3050 force ret=pick:0/1/2/3 args=v
  stub 0218f1b4 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0218ecb4 force ret=none args=v,out:12
  stub 0218ec54 force ret=none args=v
  stub 0218f918 force ret=none args=v,in:0x48,v

# wait callback of cmd 329 (vm, ctx): W = *02158FB4(SW); E = 02188C58(fm); state W[0]: 0 0218F0A8(E); 1 0218F0E8(E) == 0; 2
# 021b30c8(W, fm, E); 3 0218FE50(E) == 0 -> W[8] = 0, 02193060(0); 4 021B35A4(GAMESYS, W + 8, W[2], W[3]) (the shop UI) != 0;
# 5 0218F9C4(E, &rec (the 24 bytes of 0x021D9738), 8), W[1] = 0; 6 0218FE50(E) == 0 -> 0218EC60(E), 0218F1AC(E, W[4]),
# 0218F05C(E), 0218F878(E), 02193060(1); each passes to the next state; 7 Heap_Free(W), 1; else 0.
fn 021b3158 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x440 u32 addrof arg1 0x480
  fix arg1 0x480 u8 alt range 0 8 || any
  stub 0218ec60 force ret=none args=v
  stub 02193060 force ret=u32 args=v
  stub 0218f9c4 force ret=none args=v,in:0x18,v
  stub 021b35a4 ret=pick:0/1 args=v,v,v,v
  stub 021b30c8 ret=none args=v,v,v
  stub 0218f0e8 force ret=pick:0/1 args=v
  stub 0218f0a8 force ret=none args=v
  stub 0218f878 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 0218f1ac force ret=none args=v,v
  stub 0218f05c force ret=none args=v
  stub 020307b0 force ret=none args=v
  stub 0218fe50 force ret=pick:0/1 args=v
  stub 02158fb4 force ret=ptr:arg1:0x440:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02188c58 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  # closure: 021b30c8
  stub 0218ece4 force ret=none args=v,out:12
  stub 0218f848 force ret=none args=v
  stub 021a3050 force ret=pick:0/1/2/3 args=v
  stub 0218f1b4 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0218ecb4 force ret=none args=v,out:12
  stub 0218ec54 force ret=none args=v
  stub 0218f918 force ret=none args=v,in:0x48,v

# shop_init(GAMESYS, W2, kind, b): W2[0x162] = 0; t = 021B3B14(W2, kind) (text resources; returns kind); t 0xF7..0xFB ->
# 021b3a64(W2, t); 0xFD -> W2[0x163] = W2[0x162] = 1, 021b39c4(W2, 1); 0xFE -> 021b39c4(W2, 0), W2[0x163] = 1; 0xFF ->
# 021b38e0(W2, t, b, 0200C258(02012F08(GD))) (badges); else 021b38e0(W2, t, b, 0); then the window / sprite setup
# 021B331C(W2, W2[0x163]), 021B3BD8, 021B3FA0, 021B3F00, 021B40D4, 021B43EC (render).
fn 021b3258 ret=none inputs=200
  nowork
  arg u32 any
  arg buf 0x400
  arg u32 any
  arg u32 range 0 19
  stub 021b38e0 ret=none args=v,v,v,v
  stub 021b3fa0 ret=none args=v
  stub 021b3f00 ret=none args=v
  stub 02012944 force ret=u32 args=v
  stub 021b39c4 ret=none args=v,v
  stub 021b3a64 ret=none args=v,v
  stub 02012f08 force ret=u32 args=v
  stub 021b43ec ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 021b3b14 ret=pick:0xF7/0xF9/0xFB/0xFC/0xFD/0xFE/0xFF/5 args=v,v
  stub 021b40d4 ret=none args=v
  stub 021b3bd8 ret=none args=v
  stub 0200c258 force ret=pick:0/1/4/8 args=v
  stub 021b331c ret=none args=v,v
  # closure: 021b38e0 / 021b39c4 / 021b3a64
  stub 02020fe4 force ret=pick:4/0/1 args=v,v,v
  stub 02045c04 force ret=none args=v
  stub 02021288 force ret=pick:0/1 args=v
  stub 0201f878 force ret=u32 args=v,v,v,v,v
  stub 0201102c force ret=u32 args=v
  stub 021b385c ret=none args=v,v,v,v,v
  stub 021b3834 ret=none args=v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 0201f81c force ret=u32 args=v,v
  stub 02186670 force ret=none args=v,p,outm:2:0x1F/0,v,v

# currency(W2): W2[0x163] ? 0200DABC(W2[0x18]) (BP) : 0200C274(W2[0xC]) (money).
fn 021b3350 ret=u32 inputs=200
  nowork
  arg buf 0x400
  fix arg0 0x163 u8 alt const 0 || any
  stub 0200dabc force ret=pick:0/1/0x50/0x100/0x10000 args=v
  stub 0200c274 force ret=pick:0/1/0x50/0x100/0x10000 args=v

# buy_check(W2): have = 02007FB8(bag, item, heap); 02007E3C(bag, item, 1, heap) == 0 -> msg 0xA (021B4714), state {0xA, 4};
# money = 021b3350(W2) < price -> msg W2[0x163] ? 0x10 : 3, {0xA, 4}; money < 2 price -> 021B4804(W2, W2[0x162]), {0xA, 5};
# W2[0x162] -> 021B4804, {0xA, 6}; else msg 4, {0xA, 2}, max = min(money / price, 99), cut to 0200823C(item) - have.
# F8e-spec2: the price ties the money answers (1 / 0x50 / 0x100 / 0x10000: money == price; 0x28 / 0x80 / 0x8000: money == 2 price)
# and puts money / price at 98..100 for money 0x10000 (price 655..662); 021B4804 gets W2[0x162] in r1 at both calls.
fn 021b3370 ret=none inputs=1000
  nowork
  arg buf 0x400
  fix arg0 0x163 u8 alt const 0 || any
  fix arg0 0x162 u8 alt const 0 || any
  fix arg0 0x200 u32 alt oneof 1 0x50 0x100 0x10000 || oneof 0x28 0x80 0x8000 || range 655 662 || range 1 0x100 || range 1 0xFFFFFF
  stub 021b4804 ret=none args=v,v
  stub 021b3350 ret=pick:0/1/0x50/0x100/0x10000 args=v
  stub 021b4714 ret=none args=v,v,v,v
  stub 02007fb8 force ret=pick:0/5/0x3E7 args=v,v,v
  stub 02007e3c force ret=pick:0/1 args=v,v,v,v
  stub 0200823c force ret=pick:5/0x63/0x3E7 args=v
  # closure: 021b3350
  stub 0200dabc force ret=pick:0/1/0x50/0x100/0x10000 args=v
  stub 0200c274 force ret=pick:0/1/0x50/0x100/0x10000 args=v

# pay(W2, price): W2[0x163] == 0 -> 0200C29C(W2[0xC], price), 02008DAC(W2[0x14], 0x16, price); else 0200DA98(W2[0x18],
# (u16)price), 02008DAC(W2[0x14], 0x22, price); 02008DF0(W2[0x14], 0x15) (records).
fn 021b3568 ret=none inputs=200
  nowork
  arg buf 0x400
  arg u32 any
  fix arg0 0x163 u8 alt const 0 || any
  stub 02008df0 force ret=u32 args=v,v
  stub 02008dac force ret=none args=v,v,v
  stub 0200c29c force ret=none args=v,v
  stub 0200da98 force ret=none args=v,v

# list_flag(list, n): r = 02021288(list[0]); 02021288(list[8 i]) for i < n (results unused); r.
fn 021b3834 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg u32 range 0 0x1F
  stub 02021288 force ret=pick:0/1 args=v

# mart_list(W2, kind, b, badges): D = 02045B38(0, 2, 0x36, heap) (msgdata); idx = kind == 0xFF ? byte 0x021D946C[badges] : b + 6;
# L = ptr table 0x021DE1A8[idx]; 02021288(L[0]) -> W2[0x162] = 1; n = byte 0x021D95DE[idx]; W2[0x180] = 0201F81C(n + 1, heap);
# for i < n: 021B385C(W2, i, W2[0x162], L[i], D) (row text), W2+0x20+8i = L[i], W2+0x24+8i = 0201102C(02020FE4(L[i], 0,
# heap)) (price); W2[0x160] = n; 021b3834(W2 + 0x20, n); 0201F878(W2[0x180] + 8 n, W2[0x188], 0xB, -2, heap); 02045C04(D).
fn 021b38e0 ret=none inputs=200
  nowork
  arg buf 0x400
  arg u32 alt const 0xFF || any
  arg u32 range 0 19
  arg u32 range 0 8
  stub 02020fe4 force ret=pick:4/0/1 args=v,v,v
  stub 02045c04 force ret=none args=v
  stub 02021288 force ret=pick:0/1 args=v
  stub 0201f878 force ret=u32 args=v,v,v,v,v
  stub 0201102c force ret=u32 args=v
  stub 021b385c ret=none args=v,v,v,v,v
  stub 021b3834 ret=none args=v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 0201f81c force ret=u32 args=v,v

# special_list(W2, k): D = msgdata; E = ptr table 0x021DE164[k] ({u16 item, u32 price} records); n = byte 0x021D9430[k]; W2[0x180]
# = 0201F81C(n + 1, heap); rows i < n: 021B385C(W2, i, W2[0x162], E[i].item, D), W2+0x20+8i = item, W2+0x24+8i = E[i].price;
# W2[0x160] = n; 0201F878(..); 02045C04(D).  (k is 0 or 1 from 021b3258.)
fn 021b39c4 ret=none inputs=200
  nowork
  arg buf 0x400
  arg u32 range 0 1
  stub 02045c04 force ret=none args=v
  stub 0201f81c force ret=u32 args=v,v
  stub 021b385c ret=none args=v,v,v,v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 0201f878 force ret=u32 args=v,v,v,v,v

# dyn_list(W2, kind): D = msgdata; 02186670(W2[0x1C], W2 + 0x20, &W2[0x160], kind - 0xF7, heap) (fills the list); W2[0x180] =
# 0201F81C(W2[0x160] + 1, heap); rows i < W2[0x160]: 021B385C(W2, i, W2[0x162], W2+0x20+8i, D); 0201F878(..); 02045C04(D).
fn 021b3a64 ret=none inputs=200
  nowork
  arg buf 0x400
  arg u32 range 0xF7 0xFB
  stub 02045c04 force ret=none args=v
  stub 0201f81c force ret=u32 args=v,v
  stub 021b385c ret=none args=v,v,v,v,v
  stub 02186670 force ret=none args=v,p,outm:2:0x1F/0,v,v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 0201f878 force ret=u32 args=v,v,v,v,v

# (list callback) 0.
fn 021b4710 ret=u32 inputs=200
  nowork

# count_step(s16 *n, max, key): key -1 / 1 -> n -= 1 (0 -> max) / n += 1 (> max -> 1); key -10 / 10 -> n -= 10 (min 1) / n += 10
# (max max); returns 2 (down) / 1 (up) when n changed, else 0; other keys 0.
fn 021b49ec ret=u32 inputs=200
  nowork
  arg buf 0x10
  arg u32 alt const 1 || const 99 || range 1 99
  arg u32 oneof 0xFFFFFFF6 0xFFFFFFFF 1 10 0 2 0xFFFFFFF5
  fix arg0 0 u16 alt const 1 || const 99 || range 0 12 || range 90 100
