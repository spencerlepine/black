# fndiff spec: script commands in overlay-21 / overlay-20 files that are not scrcmd files (F8-spec5): overlay 20 WFBC (Black City /
# White Forest) commands 450..457 at the end of field_wfbc_data_local.c (0x02186BE4..0x021871F8, + the file's counter accessors
# 0x021867A8..0x021869D8 they call), the field-skill commands 197..200 after sodateya.c (0x021C457C..0x021C46F8: no __FILE__ of their own),
# fld_wipe_3dobj.c 415..417 / 576, pleasure_boat.c 421..424 / 426 / 428 (+ the boat-work helpers) and fld_trade.c 443.  Only the command
# handlers and their in-file closure: the rest of those files is F6 / F10 / F13 / U2 work (force stubs).
# Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-script scrcmd_misc21b --per-function
#
# Command ids of this area (id  handler; table 0x0217062C, index = id):
#  197 021c459c  198 021c4628  199 021c4678  200 021c46a4  415 021c812c  416 021c8170  417 021c81b4  421 021c881c
#  422 021c8848  423 021c8868  424 021c88ac  426 021c88ec  428 021c8960  443 021cc544  450 02186be4  451 02186c44
#  452 02186d48  453 02186cc4  454 02186eb8  455 02186e94  456 02187098  457 02187000  576 021c81f8
# 39 functions specced (23 command ids).
#  * WFBC (F1 scope: X1 "deferred, needs a scope decision"; local Black City / White Forest logic, specced so the decision can go either
#    way): R = 02158F58(SW) (R+4 = field map), E = 0218DC70(02188C98(fm)) (field_wfbc.c object), the talked-to OBJ at the player's front
#    tile (021A3460 / 0216DDC0 / 02186090); 450 / 451 / 453 / 457 act on its record, 452 queries 12 fields, 454 scans the party for a
#    species with a given met date, 455 adds 10 to counter b (cap 10000), 456 starts a talk event (0x021870C4; null when the X1
#    delivery_irc object 02158344 is absent) whose GMEVENT 0x021871F8 is a message / yes-no / fade dialog (UI flow, left out).
#  * 197 / 198: field-skill starts (021C1024 / 021C1228 of field_player_grid_event.c) with the wait callbacks 0x021C457D / 0x021C4609;
#    199: 021C9550 (fldeff_iaigiri.c); 200: events 021D2418 / 021D2440.
#  * 415..417 / 576: wipe 3D-object events (021C1CB0 / 021C1D68 / 021C30B8 of fld3d_ci.c on 02189A00(fm); 576 maps a key through the
#    17 pairs at 0x021DAAE0).  Render: the events themselves (U2) are force stubs.
#  * Pleasure boat: B = *02012ED0(GD) (0x1B8 bytes, 021C8608 allocates it and inits overlay 36 0x021F36C0); time W+4 in units of 0x546,
#    W[8] rounds, 15 records W + 0x24 + 0x1C i, bytes W+0x1B0..0x1B3, flag W+0x1B4.
#  * 443: the name-input event 0216C204 (namein_setup.c) of an in-game trade.  Roles UNVERIFIED (named by address).
# Not specced (6, render / UI flow, D3):
#   021871f8 GMEVENT of cmd 456 (WFBC): 18-state message / yes-no / fade dialog (UI flow, D3; also polls 02158344 delivery_irc, X1)
#   0218741a fragment (the `pop` tail of 0x021871F8)
#   0218741c message + window step of 0x021871F8 (UI)
#   0218748c message + window step of 0x021871F8 (UI)
#   02187508 yes / no menu step of 0x021871F8 (UI)
#   021875b8 window close step of 0x021871F8 (UI)
# Fragments (not functions): 0x0218741A (the `pop` tail of 0x021871F8).
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
area scrcmd_misc21b
set field

# wfbc_rec_hi4(rec): high nibble of byte 0.
fn 021867a8 ret=u8 inputs=200
  nowork
  arg buf 0x10

# wfbc_rec_near(rec): byte 2 <= 0x14.
fn 021867b4 ret=bool inputs=200
  nowork
  arg buf 0x10
  fix arg0 2 u8 alt near 0x14 2 || any

# wfbc_count_a(c): u16 c[0].
fn 021869ac ret=u16 inputs=200
  nowork
  arg buf 0x10

# wfbc_count_a_inc(c): c[0] += 1 (u16, wrapping).
fn 021869b0 ret=none inputs=200
  nowork
  arg buf 0x10

# wfbc_count_b(c): u16 c[1].
fn 021869b8 ret=u16 inputs=200
  nowork
  arg buf 0x10

# wfbc_count_b_add10(c): c[1] < 10000 -> c[1] += 10.
fn 021869bc ret=none inputs=200
  nowork
  arg buf 0x10
  fix arg0 2 u16 alt near 10000 12 || any

# wfbc_count_b (second getter, same body as 021869B8): u16 c[1].
fn 021869d0 ret=u16 inputs=200
  nowork
  arg buf 0x10

# wfbc_count_a (second getter): u16 c[0].
fn 021869d8 ret=u16 inputs=200
  nowork
  arg buf 0x10

# cmd 450: R; E; id = OBJ id at the player's front tile (see header); 02193B40(E, id); 0.
fn 02186be4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x300 u32 addrof arg1 0x400
  stub 021a3460 force ret=none args=v,outm:4:0xFF/0xFF/0/0,out:2,outm:2:0xFF/0xFF
  stub 0216ddc0 force ret=nullable:3:u32 args=v,v,v,v
  stub 02193b40 force ret=none args=v,v
  stub 0218dc70 force ret=u32 args=v
  stub 02186090 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:1 args=v
  stub 02188c7c force ret=u32 args=v
  stub 02188c98 force ret=u32 args=v

# cmd 451: R; E; C = 020127F0(GAMEDATA) (WFBC counters); id at the front tile; 02193BE4(E, id) != 0 -> 02193BC4(E, id),
# 021869B0(C) (count a += 1); 0.
fn 02186c44 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x300 u32 addrof arg1 0x400
  stub 021a3460 force ret=none args=v,outm:4:0xFF/0xFF/0/0,out:2,outm:2:0xFF/0xFF
  stub 0216ddc0 force ret=nullable:3:u32 args=v,v,v,v
  stub 0215a5e4 force ret=u32 args=v
  stub 02193be4 force ret=pick:0/1 args=v,v
  stub 02193bc4 force ret=none args=v,v
  stub 0218dc70 force ret=u32 args=v
  stub 020127f0 force ret=ptr:arg1:0x500:1 args=v
  stub 02186090 force ret=u32 args=v
  stub 021869b0 ret=none args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:1 args=v
  stub 02188c7c force ret=u32 args=v
  stub 02188c98 force ret=u32 args=v

# cmd 453: R; E; L = 02158F74(SW) (word set); k = value; h = heap (0215A5CC); id at the front tile; 02193B68(E, L, id, k, h); 0.
fn 02186cc4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x300 u32 addrof arg1 0x400
  stub 021a3460 force ret=none args=v,outm:4:0xFF/0xFF/0/0,out:2,outm:2:0xFF/0xFF
  stub 0216ddc0 force ret=nullable:3:u32 args=v,v,v,v
  stub 02193b68 force ret=none args=v,v,v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0218dc70 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02186090 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:1 args=v
  stub 02188c7c force ret=u32 args=v
  stub 02188c98 force ret=u32 args=v

# cmd 452: SW, D = 020127C4(GAMEDATA); k = value, var; k == 3 -> *var = 02186278(D, 0); else R; E; C = 020127F0(GAMEDATA);
# front-tile OBJ m (null -> id / records unset); m: id = 02186090(m), dir = 0216D588(m), 02193904(E); Q = 0219390C(E, id);
# Q: a = 02193C00(), b = 02193C04(Q); k 0: *var = a[0x18]; 1: 021867B4(b); 2: 021867A8(b); 3: (above); 4: 021869AC(C);
# 5: 021869B8(C); 6: 021938F4(E) != 0; 7: 021869D8(C); 8: 021869D0(C); 9: id; 10: 02193BE4(E, id); 11: dir; else nothing; 0.
# Spec: the OBJ and the record are always found (0216DDC0 / 0219390C never null here): without them the original reads the
# unset stack words a / b (0 under Z19) and dereferences them (address 0 / 2 / 0x18) in cases 0..2.
fn 02186d48 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x300 u32 addrof arg1 0x400
  stub 02193c00 force ret=ptr:arg1:0x540:1 args=
  stub 02193904 force ret=none args=v
  stub 02193c04 force ret=ptr:arg1:0x5E0:1 args=v
  stub 0216d588 force ret=u16 args=v
  stub 0219390c force ret=ptr:arg1:0x5C0:1 args=v,v
  stub 02159b10 force ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v,v seq
  stub 02186090 force ret=u32 args=v
  stub 02188c98 force ret=u32 args=v
  stub 021867a8 ret=none args=v
  stub 021869ac ret=none args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021867b4 ret=none args=v
  stub 021869b8 ret=none args=v
  stub 0216ddc0 force ret=u32 args=v,v,v,v
  stub 020127c4 force ret=u32 args=v
  stub 021869d0 ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:1 args=v
  stub 021869d8 ret=none args=v
  stub 021a3460 force ret=none args=v,outm:4:0xFF/0xFF/0/0,out:2,outm:2:0xFF/0xFF
  stub 0215a5e4 force ret=u32 args=v
  stub 02193be4 force ret=u32 args=v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0218dc70 force ret=u32 args=v
  stub 020127f0 force ret=ptr:arg1:0x500:1 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021938f4 force ret=pick:0/1/2 args=v
  stub 02186278 force ret=u32 args=v,v
  stub 02188c7c force ret=u32 args=v

# cmd 455: 021869BC(020127F0(02188C80(R+4) ... GAMEDATA)) (count b += 10, capped at 10000); 0.
fn 02186e94 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x300 u32 addrof arg1 0x400
  stub 02188c80 force ret=u32 args=v
  stub 020127f0 force ret=ptr:arg1:0x500:1 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:1 args=v
  stub 021869bc ret=none args=v

# cmd 454: vars A, B, C (u16): *A = *B = 0, *C = 0xFFFF; party = 02012934(GAMEDATA); n = 0201AA18(party); want = 021869D0(
# 020127F0(GAMEDATA)) (species); date = 0203F4E4 (RTC date, 3 words); for i < n: mon i: species (5) == want, egg (0x4C) == 0,
# 0x96 == 0x33, 0x8F + 0x91 + 0x90 == 0 -> *A = 1, *C = i if unset; and met date (0x92 / 0x93 / 0x94) == today -> *B = 1, *C = i;
# 0.  (PokeParam 02017E1C(mon, id, 0).)
fn 02186eb8 ret=u32 inputs=2000
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x502 u16 alt range 0 1 || any
  stub 020127c4 force ret=u32 args=v
  stub 0215a5e4 force ret=u32 args=v
  stub 0203f4e4 force ret=none args=outw:12:0/0/0/1
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201ab28 force ret=u32 args=v,v
  stub 020127f0 force ret=ptr:arg1:0x500:1 args=v
  stub 021869d0 ret=pick:0/1/2 args=v
  stub 02012934 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/6 args=v
  stub 02017e1c force ret=pick:0/0/0/0/1/0x33 args=v,v,v

# cmd 457: R; E; L; var; k = value; id at the front tile; *var = (k - 1) + 4 * id + (0219391C(E) == 0 ? 1 : 9) (u16); 0.
fn 02187000 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x300 u32 addrof arg1 0x400
  stub 021a3460 force ret=none args=v,outm:4:0xFF/0xFF/0/0,out:2,outm:2:0xFF/0xFF
  stub 0216ddc0 force ret=nullable:3:u32 args=v,v,v,v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0218dc70 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 02186090 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 02158f74 force ret=u32 args=v
  stub 0219391c force ret=pick:0/1 args=v
  stub 02158f58 force ret=ptr:arg1:0x300:1 args=v
  stub 02188c7c force ret=u32 args=v
  stub 02188c98 force ret=u32 args=v

# cmd 456: ev = 021870C4(GAMESYS, fm); 0215897C(SW, ev) (child event; its GMEVENT 0x021871F9 is UI flow); 1.
fn 02187098 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021870c4 ret=u32 args=v,v
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  # closure: 021870c4
  stub 02188c84 force ret=u32 args=v
  stub 0216d88c force ret=none args=v,out:12
  stub 0219390c force ret=u32 args=v,v
  stub 02186090 force ret=u32 args=v
  stub 02188c98 force ret=u32 args=v
  stub 020457b0 force ret=nullable:4:u32 args=v,v
  stub 02158434 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 02012138 force ret=u32 args=v
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 0216ddc0 force ret=u32 args=v,v,v,v
  stub 02158344 force ret=nullable:2:ptr:arg2:0:1 args=v
  stub 020127d0 force ret=u32 args=v,v
  stub 02188c54 force ret=u32 args=v
  stub 021a3460 force ret=none args=v,outm:4:0xFF/0xFF/0/0,out:2,outm:2:0xFF/0xFF
  stub 0201ec64 force ret=u32 args=v
  stub 020124e8 force ret=ptr:arg1:0x300:1 args=v
  stub 0218dc70 force ret=u32 args=v
  stub 020127f0 force ret=ptr:arg1:0x500:1 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02188c7c force ret=u32 args=v

# wfbc_talk_event(GAMESYS, fm): h = 02188C84(fm), X = 02188C54(fm); 02158344(02012138(GAMESYS)) (X1 delivery_irc object) null -> 0;
# else ev = 020122C0(GAMESYS, 0, 0x021871F9, 0x58), W = 020124E8(ev) = {GAMESYS, fm, (h & 0x7FFF) | 0x8000 at +8, X at +0xC, strbufs
# 020457B0(0x100, h) at +0x20 / +0x24, msg 02045B38(0, 3, 0xEA + 0x58?, h) at +0x18, word set 0201EC64(h) at +0x1C, +0x40
# 02158434(), +0x44 / +0x48 020127D0(GD, 1 / 0), front-tile OBJ: +0x28 E, +0x2C 0219390C(E, id), +0x30 mmdl, +0x34 position
# (0216D88C), +0x4C id, +0x56 = 1}; ev.  Arena: arg 2.
fn 021870c4 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x400
  stub 02188c84 force ret=u32 args=v
  stub 0216d88c force ret=none args=v,out:12
  stub 0219390c force ret=u32 args=v,v
  stub 02186090 force ret=u32 args=v
  stub 02188c98 force ret=u32 args=v
  stub 020457b0 force ret=u32 args=v,v
  stub 02158434 force ret=u32 args=
  stub 02188cb4 force ret=u32 args=v
  stub 02045b38 force ret=u32 args=v,v,v,v
  stub 02012138 force ret=u32 args=v
  stub 020122c0 force ret=nullable:4:u32 args=v,v,v,v
  stub 0216ddc0 force ret=u32 args=v,v,v,v
  stub 02158344 force ret=nullable:2:u32 args=v
  stub 020127d0 force ret=u32 args=v,v
  stub 02188c54 force ret=u32 args=v
  stub 021a3460 force ret=none args=v,outm:4:0xFF/0xFF/0/0,out:2,outm:2:0xFF/0xFF
  stub 0201ec64 force ret=u32 args=v
  stub 020124e8 force ret=ptr:arg2:0:1 args=v
  stub 0218dc70 force ret=u32 args=v
  stub 020127f0 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02188c7c force ret=u32 args=v

# wait of cmd 197 (vm, ctx): x = 0215A64C(ctx); 021C1094(x) == 1 -> 021C10A0(x) (free), 1; else 0.
fn 021c457c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c10a0 force ret=none args=v
  stub 021c1094 force ret=pick:0/1/2 args=v
  stub 0215a64c force ret=u32 args=v

# cmd 197: fm = 0201210C(GAMESYS); P = 02188CB4(fm); dir = 0216D5F8(021A3074(P)); B = 02188C98(fm); h; 021A3364(P, dir, &pos)
# (front position, 12 bytes); a = 021AB0D4(B, &pos); x = 021C1024(P, dir, a, h) (start); 0215A644(ctx, x); wait 0x021C457D; 1.
fn 021c459c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021a3364 force ret=none args=v,v,out:12
  stub 021c1024 force ret=u32 args=v,v,v,v
  stub 0215a644 force ret=none args=v,v
  stub 0201210c force ret=u32 args=v
  stub 0215a5cc force ret=u16 args=v
  stub 020113d0 force ret=none args=v,v
  stub 021a3074 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 021ab0d4 force ret=u32 args=v,in:12
  stub 0216d5f8 force ret=u16 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 02188c98 force ret=u32 args=v

# wait of cmd 198 (vm, ctx): x = 0215A64C(ctx); 021C12C4(x) == 1 -> 021C12D0(x), 1; else 0.
fn 021c4608 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021c12d0 force ret=none args=v
  stub 021c12c4 force ret=pick:0/1/2 args=v
  stub 0215a64c force ret=u32 args=v

# cmd 198: h; P = 02188CB4(0201210C(GAMESYS)); m = 021A3050(P); v = value; x = 021C1228(P, m, v, h); 0215A644(ctx, x); wait
# 0x021C4609; 1.
fn 021c4628 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0215a644 force ret=none args=v,v
  stub 021c1228 force ret=u32 args=v,v,v,v
  stub 0215a5cc force ret=u16 args=v
  stub 0201210c force ret=u32 args=v
  stub 021a3050 force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 020113d0 force ret=none args=v,v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 199: fm; 021C9550(021A3074(02188CB4(fm)), 02188CD0(fm)) (fldeff_iaigiri.c); 0.
fn 021c4678 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0201210c force ret=u32 args=v
  stub 02188cd0 force ret=u32 args=v
  stub 021c9550 force ret=none args=v,v
  stub 021a3074 force ret=u32 args=v
  stub 02188cb4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 200: v = value; ev = v 0 -> 021D2418(GAMESYS, fm); 1 -> 021D2440(GAMESYS, fm, 0); else 021D2440(GAMESYS, fm, 1); child
# event; 1.
fn 021c46a4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 021d2440 force ret=u32 args=v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0/1/2/3 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 021d2418 force ret=u32 args=v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 415: W; k = u16 operand (02011330); ev = 021C1CB0(GAMESYS, W, k & 0xFF); ev -> child event; 1.
fn 021c812c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02189a00 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02011330 force ret=u16 args=v seq
  stub 021c1cb0 force ret=nullable:2:u32 args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 416: W; k = value; ev = 021C1D68(GAMESYS, W, k & 0xFF); ev -> child event; 1.
fn 021c8170 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02189a00 force ret=u32 args=v
  stub 021c1d68 force ret=nullable:2:u32 args=v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 417: W; k = value; ev = 021C30B8(GAMESYS, W, k); ev -> child event; 1.
fn 021c81b4 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02189a00 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=u16 args=v,v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 021c30b8 force ret=nullable:2:u32 args=v,v,v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 576: W; k = value; the first of the 17 pairs {u16 key, u16 kind} at 0x021DAAE0 with key == k -> ev = 021C1CB0(GAMESYS, W,
# kind & 0xFF), ev -> child event; no match -> nothing; 1.
fn 021c81f8 ret=u32 inputs=400
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02189a00 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02159b10 force ret=pick:0x268/0x269/0x26E/0x1D2/0x1B5/0x1BA/0x1BF/0x1C2/0x1D1/0x1D7/0x26D/0x272/0x242/0x26F/0x270/0x271/0x243/0x77/0/0x267/0xFFFF args=v,v seq
  stub 021c1cb0 force ret=nullable:2:u32 args=v,v,v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# boat_alloc(): W = Heap_AllocDebug(5, 0x1B8, 1, "pleasure_boat.c", 0x31); W[0] = 0, (u16)W[4] = 0, W[8] = 0, bytes 0x1B0..0x1B3 = 0,
# W[0x1B4] = 0; 02034AC4(0x24) (overlay load); 021F36C0(W) (overlay 36 init); 02034A5C(0x24) (unload); W.  Arena: arg 0.
fn 021c8608 ret=u32 inputs=200
  nowork
  arg buf 0x400
  stub 021f36c0 force ret=none args=v
  stub 02034ac4 force ret=none args=v
  stub 02030734 force ret=seqptr:arg0:0x100:0x1C0 args=v,v,v,v,v
  stub 02034a5c force ret=none args=v

# boat_time_add(W, n, f): t = s16 W+4; r = t mod 0x546 (_s32_div); r + 30 * n >= 0x546 -> f != 0: W+4 = (W[8] + 1) * 0x546 - 1
# (W[8] unchanged); f == 0: W[8] += 1, W+4 = W[8] * 0x546; else W+4 = t + (s16)(30 * n).
fn 021c86e8 ret=none inputs=400
  nowork
  arg buf 0x40
  arg u32 alt oneof 1 2 3 || range 0 50 || any
  arg u32 range 0 1
  fix arg0 4 u16 alt oneof 0x528 0x50A 0x4EC || range 0 0x546 || range 0x500 0x2000 || any
  fix arg0 8 u32 alt range 0 8 || any

# boat_query(W, k): 0: byte W+0x1B0; 1..3: bytes W+0x1B1..0x1B3; 4: u16 W+6; 5: count of the 15 records
# (0x1C bytes from W+0x24) whose word 0 is nonzero; else 0.
fn 021c8738 ret=u32 inputs=200
  nowork
  arg buf 0x200
  arg u32 alt range 0 6 || any
  fix arg0 0x24 u32 alt const 0 || any rep=15,0x1C

# boat_set(W, i, v): W + 0x24 + 0x1C * i = v (u32).
fn 021c87d4 ret=none inputs=200
  nowork
  arg buf 0x200
  arg u32 range 0 14
  arg u32 any

# boat_flag(W): W[0x1B4] = 1 (u32).
fn 021c8810 ret=none inputs=200
  nowork
  arg buf 0x200

# cmd 421: G = 02012ED0(GD) (via 02188C80 / 020120F4); *G == 0 -> *G = 021C8608() (boat_alloc); 0.
fn 021c881c ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 alt const 0 || any
  stub 02188c80 force ret=u32 args=v
  stub 021c8608 ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02012ed0 force ret=ptr:arg1:0x500:1 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  # closure: 021c8608
  stub 021f36c0 force ret=none args=v,v,v
  stub 02034ac4 force ret=none args=v
  stub 02030734 force ret=seqptr:arg1:0x300:0x40 args=v,v,v,v,v
  stub 02034a5c force ret=none args=v

# cmd 422: 021666A4(02012ED0(GD)) (frees the boat work: symbol_map.c helper); 0.
fn 021c8848 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 02188c80 force ret=u32 args=v
  stub 021666a4 force ret=none args=v
  stub 0201210c force ret=u32 args=v
  stub 02012ed0 force ret=ptr:arg1:0x500:1 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 423: B; k = u16 operand; var; *var = boat_query(*B, k); 0.
fn 021c8868 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 addrof arg1 0x200
  stub 02188c80 force ret=u32 args=v
  stub 02159ae8 force ret=ptr:arg1:0x600:0x80 args=v,v seq
  stub 0201210c force ret=u32 args=v
  stub 02012ed0 force ret=ptr:arg1:0x500:1 args=v
  stub 02011330 force ret=pick:0/1/2/3/4/5/6 args=v seq
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 021c8738 ret=u32 args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 424: B; n, f = u16 operands; boat_time_add(*B, n, f); 0.
fn 021c88ac ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 addrof arg1 0x200
  fix arg1 0x204 u16 alt range 0 0x546 || any
  stub 02188c80 force ret=u32 args=v
  stub 021c86e8 ret=none args=v,v,v
  stub 0201210c force ret=u32 args=v
  stub 02012ed0 force ret=ptr:arg1:0x500:1 args=v
  stub 02011330 force ret=pick:0/1/2/30/60 args=v seq
  stub 020120f4 force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 426: B; i = value; k = u16 operand; (u16 operand ignored); v = value; k == 8 -> boat_set(*B, i, v == 1); other k (the jump
# table's 0..7 and k > 8) -> nothing; 0.
fn 021c88ec ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 addrof arg1 0x200
  stub 02012ed0 force ret=ptr:arg1:0x500:1 args=v
  stub 02159b10 force ret=pick:0/1/2/14 args=v,v seq
  stub 02011330 force ret=pick:0/1/4/8/9 args=v seq
  stub 0215a5f4 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 021c87d4 ret=none args=v,v
  stub 0215a5dc force ret=u32 args=v

# cmd 428: boat_flag(*02012ED0(GD)); 0.
fn 021c8960 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  fix arg1 0x500 u32 addrof arg1 0x200
  stub 02188c80 force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02012ed0 force ret=ptr:arg1:0x500:1 args=v
  stub 021c8810 ret=none args=v
  stub 020120f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v

# cmd 443: ev = 0216C204(GAMESYS, fm) (namein_setup.c: name-input event for the traded mon?); child event; 1.
fn 021cc544 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 0x800
  fix arg0 0x14 u32 addrof arg0 0x500
  fix arg0 0x1C u32 addrof arg0 0x40
  fix arg0 0xE u8 range 0 2
  fix arg1 0x20 u32 addrof arg1 0x100
  stub 0216c204 force ret=u32 args=v,v
  stub 0201210c force ret=u32 args=v
  stub 0215a5f4 force ret=u32 args=v
  stub 0215a5dc force ret=u32 args=v
  stub 0215897c force ret=none args=v,v
