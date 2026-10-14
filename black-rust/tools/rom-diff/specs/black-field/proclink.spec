# fndiff spec (F14): `event_field_proclink.c` (F1 row 10.22, overlay 10 of the field set, 0x02160A94..0x02161D96).
# Run: tools/check-fn.sh black-field proclink --per-function
#
# The file holds the callbacks of the field PROCLINK table (ROM 0x021712CC, 12 entries of 0x18 bytes: {overlay, proc
# data, init(pl, mode, kind, prev), return(pl, param), special, free}; the table is read by the proclink event proc in
# 0x0216075C..0x02160A92, which F1 put in row 10.21) and a few field GMEVENTs used around the app launches: the
# "pause the OBJs" event 02161824 (11 callers: script commands), a GMEVENT wrapper 021616A4 / 021616F0, the acmd-list
# event 021618A0 / 02161850 (+ helpers 02161938..02161984), the jump-down event 02161AC0 / 021619B8, the fade + sub-proc
# event 02161B90 / 02161BF8 (+ parameter builders 02161C84 / 02161CDC / 02161D18) and 02161D64 (display toggles).
#
# PROCLINK object `pl` (as the callbacks read it; arg0, `arg buf 0x100`): +0x08 return kind, +0x0C next menu
# action (written), +0x18 -> the proclink event work S (planted at arg0 + 0x80: S+0 GAMESYS, S+4 FIELDMAP, S+0x0C
# state, S+0x28 result, S+0x2C slot, S+0x30..0x3C params), +0x4C..0x5F copied into the party-list parameter, +0x60 u8
# party slot, +0x62 u16 item, +0x64 word, +0x68 sub-state, +0x6C mode flag. `prev` / `param` = the parameter block of
# the app that ran before / just returned (`arg buf 0x80`).
#
# Conventions (as F5b / F13): all blocks `nowork`; objects are `arg buf` arenas; Heap_AllocDebug 02030734 is a `seqptr`
# bump allocator and GMEVENT works (020124E8) are `ptr:` answers into a trailing `arg buf` arena the function never
# reads as a register; MI_CpuFill8 02082BCC runs as real code (ALLOW). Closure stubs (a block lists the stubs of its
# callees and of all its in-area descendants; checked with every function treated as ported: no fault, stray write or
# unstubbed callee). Every out-of-area stub is `force`. Jump-table indices are drawn over the table and past it.
# Paths that read an unset callee-saved register are kept out: 021618E0 / 02161B1C pick an acmd list by the OBJ direction
# 0..3 (another direction reads r4 as left by the caller), so `0216D5F8` answers `pick:0/1/2/3` there.
# The veneer 0216171C (`ldr r3, =020307B0; bx r3`, the table's free callback = Heap_Free) is not specced.
area proclink
set field

# FUN_02160A94(pl, mode, kind, prev): init of entry 0 (overlay 0x5B, the party list): a 0x98-byte parameter block.
fn 02160a94 ret=u32 inputs=800
  nowork
  arg buf 0x100
  arg u32 alt oneof 5 0xFFFFFFFF 0 1 || any
  arg u32 alt oneof 0 2 6 9 10 || const 2 || any
  arg buf 0x80
  arg buf 0x200
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg0 0x68 u32 alt oneof 0 2 6 || any
  fix arg3 0x34 u32 alt range 0 19 || oneof 3 12 18 || any
  fix arg3 0x2C u32 alt const 2 || any
  fix arg3 0x12 u8 alt range 0 2 || any
  fix arg3 0x0D u8 alt const 2 || any
  stub 020120f4 force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg4:0:0xA0 args=v,v,v,v,v
  stub 02012934 force ret=u32 args=v
  stub 0201292c force ret=u32 args=v
  stub 02008fec force ret=u32 args=v
  stub 0200ca1c force ret=u32 args=v
  stub 020127a4 force ret=ptr:arg4:0x180:8 args=v
  stub 02012984 force ret=u32 args=v
  stub 02034ac4 force ret=none args=v
  stub 021c3e00 force ret=pick+u32:0/1/2 args=v
  stub 02034a5c force ret=none args=v

# FUN_02160C88(pl, param): return of entry 0: the slot back into S+0x2C / pl+0x60, then the next action by the
# parameter's end mode (+0x40, jump table of 22).
fn 02160c88 ret=u32 inputs=600
  nowork
  arg buf 0x100
  arg buf 0x80
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg0 0x6C u32 alt const 1 || any
  fix arg0 0x8C u32 alt const 2 || any
  fix arg1 0x3C u32 alt const 8 || any
  fix arg1 0x34 u32 alt const 6 || any
  fix arg1 0x40 u32 alt range 0 22 || any

# FUN_02160D6C(pl, page): init of entry 1 (overlay 0xD4): a 0x10-byte block {gd, 0200CA1C(gd), 02012944(gd), u16 page
# (-1 -> 0)}; 02029DBC(02012A30(gd), 02012134(gs)) before.
fn 02160d6c ret=u32 inputs=100
  nowork
  arg buf 0x100
  arg u32 alt oneof 0xFFFFFFFF 0 3 || any
  arg buf 0x100
  fix arg0 0x18 u32 addrof arg0 0x80
  stub 02030734 force ret=seqptr:arg2:0:0x40 args=v,v,v,v,v
  stub 020120f4 force ret=u32 args=v
  stub 0200ca1c force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 02012a30 force ret=u32 args=v
  stub 02012134 force ret=u32 args=v
  stub 02029dbc force ret=none args=v,v

# FUN_02160DDC(pl, param): return of entry 1: 02029DD8(02012A30(param.gd), 02012134(gs)); param+0xE != 0.
fn 02160ddc ret=u32 inputs=100
  nowork
  arg buf 0x100
  arg buf 0x40
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg1 0xE u16 alt const 0 || any
  stub 02012a30 force ret=u32 args=v
  stub 02012134 force ret=u32 args=v
  stub 02029dd8 force ret=none args=v,v

# FUN_02160E08(pl, mode, kind, prev): init of entry 6 (overlay 0x83): a 0x28-byte block; kind 6 with mode 1 / 2 looks
# for the first party Pokemon that qualifies (02017E1C fields 0xA9 / 0x4C, 0201897C).
fn 02160e08 ret=u32 inputs=800
  nowork
  arg buf 0x100
  arg u32 alt oneof 0xFFFFFFFF 0 1 2 || any
  arg u32 alt oneof 0 6 || any
  arg buf 0x80
  arg buf 0x200
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg3 0x40 u32 alt range 3 6 || any
  stub 02030734 force ret=seqptr:arg4:0:0x40 args=v,v,v,v,v
  stub 020120f4 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v
  stub 0200ca1c force ret=u32 args=v
  stub 0200ca38 force ret=u16 args=v
  stub 0201aa18 force ret=pick:0/1/2/3/6 args=v
  stub 0200873c force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02017e1c force ret=pick+u32:0/1 args=v,v,v
  stub 0201897c force ret=pick+u32:0/1 args=v

# FUN_02160F88(pl, param): return of entry 6.
fn 02160f88 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x40
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg0 0x8C u32 alt const 6 || any
  fix arg1 0x12 u8 alt range 0 3 || any

# FUN_02160FB4(gs): 1 / 3 / 0 by two zone predicates (02013CC8 / 02013CE0) of 02012AC8(020120F8(gs)).
fn 02160fb4 ret=u32 inputs=200
  nowork
  arg u32 any
  stub 020120f8 force ret=u32 args=v
  stub 02012ac8 force ret=u32 args=v
  stub 02013cc8 force ret=pick+u32:0/1/2 args=v
  stub 02013ce0 force ret=pick+u32:0/1/2 args=v

# FUN_02160FE0(pl, item, kind, prev): init of entry 2 (overlay 0x46, the bag): 0202E474(gd, &pl+0x34, mode, 4) with the
# mode of 02160FB4 (kind 0 after an app with end mode 3: mode 2); kind 2 with an item preselects it (02008298).
fn 02160fe0 ret=u32 inputs=400
  nowork
  arg buf 0x100
  arg u32 alt oneof 0xFFFFFFFF 0 0x10003 || any
  arg u32 alt range 0 9 || any
  arg buf 0x80
  arg buf 0x100
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg3 0x40 u32 alt const 3 || any
  stub 020120f4 force ret=u32 args=v
  stub 02160fb4 ret=u32 args=v
  stub 0202e474 force ret=ptr:arg4:0:8 args=v,v,v,v
  stub 02008298 force ret=none args=v,v
  stub 020120f8 force ret=u32 args=v
  stub 02012ac8 force ret=u32 args=v
  stub 02013cc8 force ret=pick+u32:0/1/2 args=v
  stub 02013ce0 force ret=pick+u32:0/1/2 args=v

# FUN_02161070(pl, param): return of entry 2: next action by the bag's end mode (+0x34, jump table of 21).
fn 02161070 ret=u32 inputs=600
  nowork
  arg buf 0x100
  arg buf 0x80
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg1 0x34 u32 alt range 0 21 || any
  fix arg1 0x2C u32 alt const 2 || any
  stub 020120f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02017e1c force ret=pick+u32:0/1 args=v,v,v
  stub 0202136c force ret=pick+u32:0/1/2 args=v

# FUN_0216117C(pl, x): init of entry 3 (overlay 0x6F): 0216F7BC(gd, 4, flag) (flag 0 in zones of 02013CC8 / 02013CE0),
# its +0x14 = x unless x is -1.
fn 0216117c ret=u32 inputs=300
  nowork
  arg buf 0x100
  arg u32 alt oneof 0xFFFFFFFF 0 || any
  arg buf 0x100
  fix arg0 0x18 u32 addrof arg0 0x80
  stub 020120f4 force ret=u32 args=v
  stub 020127a4 force ret=ptr:arg2:0x80:8 args=v
  stub 02012a30 force ret=u32 args=v
  stub 02012134 force ret=u32 args=v
  stub 02029dbc force ret=none args=v,v
  stub 02013cc8 force ret=pick+u32:0/1/2 args=v
  stub 02013ce0 force ret=pick+u32:0/1/2 args=v
  stub 0216f7bc force ret=ptr:arg2:0:8 args=v,v,v

# FUN_021611E0(pl, param): return of entry 3: 02029DD8(02012A30(gd), 02012134(gs)); param+0x10 == 1.
fn 021611e0 ret=u32 inputs=100
  nowork
  arg buf 0x100
  arg buf 0x40
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg1 0x10 u32 alt range 0 2 || any
  stub 020120f4 force ret=u32 args=v
  stub 02012a30 force ret=u32 args=v
  stub 02012134 force ret=u32 args=v
  stub 02029dd8 force ret=none args=v,v

# FUN_02161214(pl): init of entry 7 (overlay 0x48): a 0x18-byte block {S+0xC != 0, -, u16 02012AC8(zone), s16
# 020128B8(gd)+4, ..., gs}.
fn 02161214 ret=u32 inputs=100
  nowork
  arg buf 0x100
  arg buf 0x100
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg0 0x8C u32 alt const 0 || any
  stub 02030734 force ret=seqptr:arg1:0:0x40 args=v,v,v,v,v
  stub 020120f4 force ret=u32 args=v
  stub 020127a4 force ret=u32 args=v
  stub 02012ac8 force ret=u32 args=v
  stub 020128b8 force ret=ptr:arg1:0x80:8 args=v

# FUN_02161280(pl, param): return of entry 7, by S+0xC (7 / 2 / 0) and the result param+4.
fn 02161280 ret=u32 inputs=400
  nowork
  arg buf 0x100
  arg buf 0x40
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg0 0x8C u32 alt oneof 7 2 0 || any
  fix arg1 4 u32 alt range 0 3 || any

# FUN_02161308(pl): init of entry 5 (overlay 0x44): 0200873C(02012EBC(gd)); an 8-byte block {-, gs}.
fn 02161308 ret=u32 inputs=60
  nowork
  arg buf 0x100
  arg buf 0x100
  fix arg0 0x18 u32 addrof arg0 0x80
  stub 020120f4 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v
  stub 0200873c force ret=u32 args=v
  stub 02030734 force ret=seqptr:arg1:0:0x40 args=v,v,v,v,v

# FUN_02161340(pl, param): return of entry 5: param+0 != 0.
fn 02161340 ret=u32 inputs=60
  nowork
  arg buf 0x100
  arg buf 0x40
  fix arg1 0 u32 alt const 0 || any

# FUN_02161350(pl): the special callback of entry 4: run the GMEVENT 021616A4(gs, fm, u16 pl+0, 02188C54(fm), &pl+0x10)
# as a sub-event of pl+0x14 (02012374).
fn 02161350 ret=none inputs=100
  nowork
  arg buf 0x100
  arg buf 0x100
  fix arg0 0x18 u32 addrof arg0 0x80
  stub 02188c54 force ret=u32 args=v
  stub 021616a4 ret=u32 args=v,v,v,v,v
  stub 02012374 force ret=none args=v,v
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg1:0:1 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# FUN_02161378(pl): init of entry 8 (overlay 0x80): an 8-byte block {gd, -}.
fn 02161378 ret=u32 inputs=60
  nowork
  arg buf 0x100
  arg buf 0x100
  fix arg0 0x18 u32 addrof arg0 0x80
  stub 02030734 force ret=seqptr:arg1:0:0x40 args=v,v,v,v,v
  stub 020120f4 force ret=u32 args=v

# FUN_021613A4(pl, param): return of entry 8.
fn 021613a4 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x40
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg0 0x8C u32 alt const 8 || any
  fix arg1 4 u32 alt const 0 || any

# FUN_021613CC(pl, mode, kind, prev): init of entry 9 (no table overlay; loads overlay 0x88 itself): the item-use
# screens 021C3CC0 / 021C3D44 / 021C3DA4.
fn 021613cc ret=u32 inputs=400
  nowork
  arg buf 0x100
  arg u32 any
  arg u32 alt const 0 || any
  arg buf 0x80
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg3 0x40 u32 alt const 6 || any
  fix arg3 0x2C u32 alt const 2 || any
  stub 020120f4 force ret=u32 args=v
  stub 02034ac4 force ret=none args=v
  stub 02012934 force ret=u32 args=v
  stub 0202138c force ret=u32 args=v
  stub 021c3cc0 force ret=u32 args=v,v,v,v,v
  stub 0201ab28 force ret=u32 args=v,v
  stub 021c3d44 force ret=u32 args=v,v,v
  stub 021c3da4 force ret=u32 args=v,v,v

# FUN_02161484(pl, param): return of entry 9: hand the result to the party slot (021C3E00 / 021C3E04), next action,
# unload overlay 0x88.
fn 02161484 ret=u32 inputs=400
  nowork
  arg buf 0x100
  arg u32 any
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg0 8 u32 alt oneof 0 2 || any
  fix arg0 0x68 u32 alt oneof 0 2 || any
  stub 020120f4 force ret=u32 args=v
  stub 021c3e00 force ret=pick+u32:0/1/2 args=v
  stub 02012934 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 021c3e04 force ret=none args=v,v
  stub 02034a5c force ret=none args=v

# FUN_02161534(param): free callback of entry 9 (021C3E24 inside overlay 0x88).
fn 02161534 ret=none inputs=30
  nowork
  arg u32 any
  stub 02034ac4 force ret=none args=v
  stub 021c3e24 force ret=none args=v
  stub 02034a5c force ret=none args=v

# FUN_02161554(pl, mode, kind, prev): init of entry 10 (overlay 0xC3): kind 0 only (else 0): 0201B2B0(party, mon, ...)
# and a 0x14-byte block.
fn 02161554 ret=u32 inputs=300
  nowork
  arg buf 0x100
  arg u32 any
  arg u32 alt const 0 || any
  arg buf 0x80
  arg buf 0x100
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg3 0x40 u32 alt const 8 || any
  stub 020120f4 force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0201ab28 force ret=u32 args=v,v
  stub 02012984 force ret=u32 args=v
  stub 020127a4 force ret=ptr:arg4:0x80:8 args=v
  stub 0201b2b0 force ret=u32 args=v,v,v,v,v,out:4,v
  stub 02030734 force ret=seqptr:arg4:0:0x40 args=v,v,v,v,v

# FUN_02161610(pl, param): return of entry 10, by pl+0x68 (3 / 4).
fn 02161610 ret=u32 inputs=100
  nowork
  arg buf 0x100
  arg buf 0x40
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg0 0x68 u32 alt range 2 5 || any
  stub 020120f4 force ret=u32 args=v

# FUN_0216163C(pl): init of entry 11 (overlay 0xBB): a 0x10-byte block {-, gd, ..}.
fn 0216163c ret=u32 inputs=60
  nowork
  arg buf 0x100
  arg buf 0x100
  fix arg0 0x18 u32 addrof arg0 0x80
  stub 02030734 force ret=seqptr:arg1:0:0x40 args=v,v,v,v,v
  stub 020120f4 force ret=u32 args=v

# FUN_02161668(pl, param): return of entry 11, by S+0xC (11 or not) and param+0xC.
fn 02161668 ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg buf 0x40
  fix arg0 0x18 u32 addrof arg0 0x80
  fix arg0 0x8C u32 alt const 11 || any
  fix arg1 0xC u32 alt range 0 2 || any

# FUN_021616A4(gs, fm, id, x, &result): a GMEVENT (proc 021616F0, work 0x1C {02012EBC(gd), gs, fm, x, u16 id, &result}).
fn 021616a4 ret=u32 inputs=60
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x100
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg5:0:1 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012ebc force ret=u32 args=v

# FUN_021616F0(event, seq, work): the proc: 02169AB8(work) 0 -> *result = 1, done; 1 -> *result = 0, done; else wait.
fn 021616f0 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg2 0x14 u32 addrof arg2 0x30
  stub 02169ab8 force ret=pick+u32:0/1/2 args=v

# FUN_02161724(event, seq, work): pause the field OBJs (seq 0: 0216DE54 / 0216D9A0, remember which OBJs were paused in
# work+0xC), seq 1 waits for them (0216D454 / 0216DA5C) and releases (0216DAF8), seq 2 ends.
fn 02161724 ret=u32 inputs=600
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 3 || any
  fix arg2 0xC u8 alt range 0 7 || any
  stub 02188c7c force ret=u32 args=v
  stub 0216dee0 force ret=u32 args=v
  stub 0216de54 force ret=u32 args=v,v
  stub 0216d9a0 force ret=none args=v
  stub 0216d454 force ret=pick+u32:0/1/2 args=v
  stub 0216db04 force ret=none args=v
  stub 0216da5c force ret=pick+u32:0/1/2 args=v
  stub 0216daf8 force ret=none args=v

# FUN_02161824(gs, fm): the pause event (proc 02161724, work 0x10 {gs, fm, gd, flags}).
fn 02161824 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg2:0:1 args=v
  stub 020120f4 force ret=u32 args=v

# FUN_02161850(event, seq, work): acmd-list event proc: seq 0 starts the list work+0x10 on the OBJ 02161938(sys, id)
# (none: done), seq 1 waits until every list ended (02161984).
fn 02161850 ret=u32 inputs=300
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x80
  fix arg1 0 u32 alt range 0 2 || any
  fix arg2 0xC u16 alt oneof 0xF1 0xF2 || any
  fix arg2 0x14 u32 alt const 0 || any rep=8,4
  stub 02188c7c force ret=u32 args=v
  stub 02161938 ret=nullable:3:u32 args=v,v
  stub 02197d3c force ret=u32 args=v,v
  stub 02161964 ret=none args=v,v
  stub 02161984 ret=pick:0/1 args=v
  stub 0216de54 force ret=nullable:3:u32 args=v,v
  stub 0216de94 force ret=nullable:3:u32 args=v,v
  stub 02197da0 force ret=pick+u32:0/1/2 args=v
  stub 02197dac force ret=none args=v

# FUN_021618A0(gs, fm, id, list): the acmd-list event (proc 02161850, work 0x34 {gs, fm, gd, u16 id, list, slots[8]}).
fn 021618a0 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x80
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg4:0:1 args=v
  stub 02161950 ret=none args=v
  stub 020120f4 force ret=u32 args=v

# FUN_021618E0(gs, fm): the acmd-list event for the player's OBJ (id 0xFF) with the ROM list of its direction
# (0x02171404 / 0x021713FC / 0x021713F4 / 0x021713EC).
fn 021618e0 ret=u32 inputs=60
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x80
  stub 02188cb4 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v
  stub 021618a0 ret=u32 args=v,v,v,v
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg2:0:1 args=v
  stub 020120f4 force ret=u32 args=v

# FUN_02161938(sys, id): id 0xF2 -> 0216DE54(sys, 0x30); id 0xF1 -> sys itself; else 0216DE94(sys, id).
fn 02161938 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 alt oneof 0xF1 0xF2 || any
  stub 0216de54 force ret=u32 args=v,v
  stub 0216de94 force ret=u32 args=v,v

# FUN_02161950(work): clear the eight list slots (work+0x14).
fn 02161950 ret=none inputs=20
  nowork
  arg buf 0x40

# FUN_02161964(work, list): put the list in the first free slot.
fn 02161964 ret=none inputs=100
  nowork
  arg buf 0x40
  arg u32 any
  fix arg0 0x14 u32 alt const 0 || any || any || any rep=8,4

# FUN_02161984(work): free the slots whose list ended (02197DA0 / 02197DAC); 1 while one still runs.
fn 02161984 ret=u32 inputs=200
  nowork
  arg buf 0x40
  fix arg0 0x14 u32 alt const 0 || any rep=8,4
  stub 02197da0 force ret=pick+u32:0/1/2 args=v
  stub 02197dac force ret=none args=v

# FUN_021619B8(event, seq, work): jump-down event proc: seq 0 moves the OBJ along the ROM arc 0x0217140C (index
# work+8 += 2 per frame) towards the target (FX_Div by 15); seq 1 drops it to the target and ends.
fn 021619b8 ret=u32 inputs=400
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  fix arg2 8 u16 alt range 0 16 || oneof 14 16
  fix arg2 0x10 u32 alt range 0 0x20000 || any
  fix arg2 0x1C u32 alt range 0 0x20000 || any
  stub 0207c768 force ret=u32 args=v,v
  stub 0216d89c force ret=none args=v,in:12
  stub 0216d88c force ret=none args=v,outm:12:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0x03/0x00/0xFF/0xFF/0xFF/0xFF
  stub 0216d5f8 force ret=u32 args=v
  stub 0216e014 force ret=none args=v,in:12,v
  stub 020061e4 force ret=none args=v
  stub 021ac528 force ret=none args=v,v

# FUN_02161AC0(gs, obj, &from, &to): the jump-down event (proc 021619B8, work 0x24 {02188CD0(0201210C(gs)), obj, -,
# from, to - from}); sound 0x55E. arg4 (a stack word, never read) = arena.
fn 02161ac0 ret=u32 inputs=50
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg4:0:1 args=v
  stub 0201210c force ret=u32 args=v
  stub 02188cd0 force ret=u32 args=v
  stub 0207ca00 force ret=none args=in:12,in:12,out:12
  stub 020061e4 force ret=none args=v

# FUN_02161B1C(fm): start the ROM acmd list of the player OBJ's direction (0216D4B4 first); returns the list handle.
fn 02161b1c ret=u32 inputs=60
  nowork
  arg u32 any
  stub 02188cb4 force ret=u32 args=v
  stub 021a3074 force ret=u32 args=v
  stub 0216d5f8 force ret=pick:0/1/2/3 args=v
  stub 0216d4b4 force ret=none args=v
  stub 02197d3c force ret=u32 args=v,v

# FUN_02161B74(list): 1 (and free it) once the acmd list ended.
fn 02161b74 ret=u32 inputs=60
  nowork
  arg u32 any
  stub 02197da0 force ret=pick+u32:0/1/2 args=v
  stub 02197dac force ret=none args=v

# FUN_02161B90(gs, parent, a, b, flag): fade + sub-proc event (proc 02161BF8, work 0x24 {gs, 02012138(gs), param
# (0x18, 02161CDC when flag else 02161D18)}); a == 3 also calls 0216570C. arg5 = arena.
fn 02161b90 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt range 0 4 || any
  arg u32 any
  arg u32 alt const 0 || any
  arg buf 0x80
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg5:0:1 args=v
  stub 02012138 force ret=u32 args=v
  stub 02161cdc ret=none args=v,v,v,v
  stub 02161d18 ret=none args=v,v,v,v
  stub 0216570c force ret=none
  stub 0203f4cc force ret=u32 args=out:8
  stub 0202e888 force ret=u32 args=v
  stub 02161c84 ret=u32 args=v,v,v,v,v,v,v,v
  stub 020120f4 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 02008550 force ret=u32 args=v
  stub 02169100 force ret=u32 args=v
  stub 02169110 force ret=u32 args=v
  stub 02012984 force ret=u32 args=v

# FUN_02161BF8(event, seq, work): seq 0 saves / sets 02012144 / 0201213C and starts the fade task (02026454 / 02026400),
# seq 1 waits for it, seq 2 calls the sub-proc 020120B4(gs, 0xCB, 0x021BFB3C, &param), seq 3 waits (020120F0) and
# restores.
fn 02161bf8 ret=u32 inputs=300
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 4 || any
  stub 02012144 force ret=u32 args=v
  stub 0201213c force ret=none args=v,v
  stub 02026454 force ret=pick+u32:0/1 args=v
  stub 02026400 force ret=none args=v
  stub 020120b4 force ret=u32 args=v,v,v,v
  stub 020120f0 force ret=pick+u32:0/1 args=v

# FUN_02161C84(param, gs, x, b, w, c, d, e): fill the 0x18-byte sub-proc parameter; bit 7 of +0xF from
# 02008550(02012944(gd)).
fn 02161c84 ret=u32 inputs=100
  nowork
  arg buf 0x20
  arg u32 any
  arg u32 any
  arg u8 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 020120f4 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 02008550 force ret=u32 args=v

# FUN_02161CDC(param, gs, x, b): 02161C84(param, gs, x, b, 0, two bytes of 0203F4CC, 0202E888(..)).
fn 02161cdc ret=u32 inputs=100
  nowork
  arg buf 0x20
  arg u32 any
  arg u8 any
  arg u8 any
  stub 0203f4cc force ret=u32 args=out:8
  stub 0202e888 force ret=u32 args=v
  stub 02161c84 ret=u32 args=v,v,v,v,v,v,v,v
  stub 020120f4 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 02008550 force ret=u32 args=v

# FUN_02161D18(param, gs, x, b): 02161C84(param, gs, x, b, 0, 02169100(gd), 02169110(gd), 02012984(gd)).
fn 02161d18 ret=u32 inputs=100
  nowork
  arg buf 0x20
  arg u32 any
  arg u8 any
  arg u8 any
  stub 020120f4 force ret=u32 args=v
  stub 02169100 force ret=u32 args=v
  stub 02169110 force ret=u32 args=v
  stub 02012984 force ret=u32 args=v
  stub 02161c84 ret=u32 args=v,v,v,v,v,v,v,v
  stub 02012944 force ret=u32 args=v
  stub 02008550 force ret=u32 args=v

# FUN_02161D64(rec): +2 == 1: 02013A80(0, 1), 02013A80(1, 1); +2 == 0: 02013A80(0, 0), 02013A80(1, 0).
fn 02161d64 ret=none inputs=60
  nowork
  arg buf 0x10
  fix arg0 2 u16 alt range 0 2 || any
  stub 02013a80 force ret=none args=v,v
