# fndiff spec: box_savedata.c (S2) = main ARM9 0x02007454..0x02007AF4 (33 real functions).
# Grammar: svflow_specs/README.md (generic mode). Run (the area name `box` is a Rust keyword, so the
# port lives in box_data.rs):
#   FNDIFF_SRC=crates/black-storage/src/box_data.rs FNDIFF_TESTS=crates/black-storage/src/box_data/tests.rs \
#   FNDIFF_FILTER=box_data::tests::fndiff_match_rom tools/check-fn.sh black-storage box --per-function
#
# Range (confirmed): the rodata names are save_control.c 0x020A705C, box_savedata.c 0x020A706C,
# perapvoice.c 0x020A707C in that order. FUN_02007AA4 allocs 4 bytes under "box_savedata.c" (line 0x3B9) and
# FUN_02007B20 allocs under "perapvoice.c" (line 0x57), so box_savedata.c is the code between the last
# save_control.c function (FUN_02007428, flash clear) and perapvoice.c (FUN_02007B00 onward: size 0x3EC,
# init, alloc). T1's start 0x020071A8 is save_control.c (FUN_020071CC = SaveData_GetBlock thunk, FUN_020073xx ...),
# not box data. Not functions: FUN_020075A4 (2 bytes, literal-pool padding), sub_02007828 (tail of 0x0200781C
# that Ghidra split), FUN_020078E2 (middle of FUN_020078C4). Missing from the Ghidra export (found in rizin):
# 02007610 020076A4 020076B4 02007774 020077B8 0200780C 0200781C 02007898.
#
# BOX LAYOUT. Two kinds of save blocks, fetched by FUN_020071CC(savedata_obj, block_id) (thunk to FUN_020319A4):
#  * block 0 = box META, 0x3E0 bytes (size getter 0200745C): +0 u32 current tray (0..23), +4 + i*0x28 name of
#    tray i (20 UTF-16 chars, 24 trays), +0x3C4 + i u8 wallpaper of tray i (init i % 16), +0x3DC u8 flags (bit
#    masks, FUN_02007A68 sets, FUN_02007A80 tests), +0x3DD u8 number of unlocked trays (init 8, +8 each
#    FUN_020076B4, max 24 = 0x18).
#  * blocks 1..24 = box TRAYS, 0xFF0 bytes each (size getter 02007AEC): 30 slots of 0x88 bytes (BoxPokemon,
#    P1 layout). A slot is empty when species (FUN_02017E88 id 5) is 0; the counting loops test id 0xA9 != 0.
# The `box manager` object (arg 0 of nearly every function) is a pointer to a struct whose first word is the
# savedata object (FUN_02007AA4 allocates 4 bytes and stores it). Tray index 0xFFFFFFFF means "current tray".
# Modelling: arg 0 is a plain u32 (the manager pointer, only logged); FUN_020074CC (get meta) and FUN_02007AF4
# (get tray) are stubbed in every caller (ret=ptr into the meta/tray buffer that follows the real arguments)
# and verified by their own entries via a stub of FUN_020071CC. Per-mon calls: FUN_02017E88 (PokeParaGet,
# black-pokemon P1) is stubbed ret=pick:0/1; FUN_020181A0 (post-copy fixup) and FUN_02017550 (clear one slot)
# are stubs; assert FUN_02034714 is noreturn. Name strings go through FUN_02045924 / FUN_02045A38 (stubs).
# Ghidra's arities are wrong in places (FUN_02007610 reads a 5th stack argument); this follows the disassembly.
area box
set field

# FUN_02007454(meta): tail call of the meta initialiser FUN_02007464 (via literal pool 0x02007465).
fn 02007454 ret=none
  nowork
  arg u32 any
  stub 02007464 force ret=none args=v

# FUN_0200745C(): 0x3E0 = size of the meta block.
fn 0200745c ret=u32
  nowork

# FUN_02007464(meta): wallpaper[i] = i % 16, flags = 0; msg = FUN_02045B38(0, 2, 9, 1); if msg: name[i] =
# msg string i+6 (0x14 chars) for 24 trays, then FUN_02045C04(msg); current tray = 0; tray count = 8.
fn 02007464 ret=none
  nowork
  arg buf 0x3E0
  stub 02045b38 ret=pick:0/0x1234 args=v,v,v,v
  stub 02045f5c ret=none args=v,v,out:0x28,v
  stub 02045c04 ret=none args=v

# FUN_020074CC(mgr): FUN_020071CC(*mgr, 0) = the meta block (stub: a pointer into arg 1).
fn 020074cc ret=ofs:arg1
  nowork
  arg buf 0x10
  arg buf 0x400
  fix arg0 0 u32 addrof arg0 8
  stub 020071cc ret=ptr:arg1:0:1 args=v,v

# FUN_02007AF4(mgr, i): FUN_020071CC(*mgr, i + 1) = tray i (stub: a pointer into arg 2).
fn 02007af4 ret=ofs:arg2
  nowork
  arg buf 0x10
  arg u32 alt range 0 23 || any
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 8
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# FUN_020074D8(mgr, mon): try FUN_0200752C on trays cur, cur+1, .. (wrapping at the unlocked count); if none
# has room and count != 24: count += 8 and put into the first new tray (index old count). Returns 1 / 0.
fn 020074d8 ret=bool
  nowork
  arg u32 any
  arg buf 0x90
  arg buf 0x3E0
  fix arg2 0 u32 range 0 23
  fix arg2 0x3dd u8 oneof 8 16 24
  stub 020074cc force ret=ptr:arg2:0:1 args=v
  stub 0200752c force ret=pick:0/0/0/0/0/0/0/0/0/1 args=v,v,v seq

# FUN_0200752C(mgr, tray, mon): tray -1 = current; tray >= count: assert. Copies the 0x88-byte mon into the
# first slot of the tray whose species (get id 5) is 0, then FUN_020181A0(slot); returns 1, or 0 if full.
fn 0200752c ret=bool
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg buf 0x90
  arg buf 0x3E0
  arg buf 0x1000
  fix arg3 0 u32 alt range 0 23 || any
  fix arg3 0x3dd u8 oneof 8 16 24
  stub 020074cc force ret=ptr:arg3:0:1 args=v
  stub 02007af4 force ret=ptr:arg4:0:1 args=v,v
  stub 02017e88 ret=pick:0/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1 args=v,v,v
  stub 020181a0 ret=none args=v
  stub 02034714 noreturn ret=none args=v,v,v

# FUN_020075B0(mgr, tray, slot, mon): tray -1 = current; tray < count and slot < 30 else assert. Copies the mon
# into the slot, FUN_020181A0(slot); returns 1.
fn 020075b0 ret=bool
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg u32 alt range 0 29 || range 0 29 || range 0 29 || range 30 33
  arg buf 0x90
  arg buf 0x3E0
  arg buf 0x1000
  fix arg4 0 u32 alt range 0 23 || any
  fix arg4 0x3dd u8 oneof 8 16 24
  stub 020074cc force ret=ptr:arg4:0:1 args=v
  stub 02007af4 force ret=ptr:arg5:0:1 args=v,v
  stub 020181a0 ret=none args=v
  stub 02034714 noreturn ret=none args=v,v,v

# FUN_02007610(mgr, trayA, slotA, trayB, slotB): swap slot A of tray A with slot B of tray B (0x88-byte
# temporary on the stack); no checks. slotB is the first stack word. Both trays resolve to the one tray buffer.
fn 02007610 ret=none
  nowork
  arg u32 any
  arg u32 range 0 23
  arg u32 range 0 29
  arg u32 range 0 23
  arg u32 range 0 29
  arg buf 0x1000
  stub 02007af4 force ret=ptr:arg5:0:1 args=v,v

# FUN_02007664(mgr, tray, slot): if slot < 30 and tray (-1 = current) < 24: FUN_02017550(&tray[slot]) clears it.
fn 02007664 ret=none
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg u32 alt range 0 29 || range 0 29 || range 0 29 || range 30 33
  arg buf 0x3E0
  arg buf 0x1000
  fix arg3 0 u32 alt range 0 23 || any
  stub 020074cc force ret=ptr:arg3:0:1 args=v
  stub 02007af4 force ret=ptr:arg4:0:1 args=v,v
  stub 02017550 ret=none args=v

# FUN_02007698(mgr): current tray.
fn 02007698 ret=u32
  nowork
  arg u32 any
  arg buf 0x3E0
  stub 020074cc force ret=ptr:arg1:0:1 args=v

# FUN_020076A4(mgr): number of unlocked trays (meta +0x3DD).
fn 020076a4 ret=u8
  nowork
  arg u32 any
  arg buf 0x3E0
  fix arg1 0x3dd u8 alt oneof 8 16 24 || any
  stub 020074cc force ret=ptr:arg1:0:1 args=v

# FUN_020076B4(mgr): if the count != 24 it grows by 8; returns the count.
fn 020076b4 ret=u8
  nowork
  arg u32 any
  arg buf 0x3E0
  fix arg1 0x3dd u8 alt oneof 8 16 24 || any
  stub 020074cc force ret=ptr:arg1:0:1 args=v

# FUN_020076D0(mgr, u32 *tray, int *slot): first slot at or after (*tray, *slot) (tray -1 = current; wrapping
# over the unlocked trays) whose get(0xA9) is 0; writes both and returns 1. None: if count == 24 return 0,
# else *tray = old count, *slot = 0, count += 8, return 1.
fn 020076d0 ret=bool
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  arg buf 0x3E0
  arg buf 0x1000
  fix arg3 0 u32 range 0 7
  fix arg3 0x3dd u8 oneof 8 8 8 16 24
  fix arg1 0 u32 alt range 0 7 || const 0xFFFFFFFF
  fix arg2 0 u32 alt range 0 29 || range 28 31
  stub 020074cc force ret=ptr:arg3:0:1 args=v
  stub 02007af4 force ret=ptr:arg4:0:1 args=v,v
  stub 02017e88 ret=pick:0/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1/1 args=v,v,v seq

# FUN_02007774(mgr): number of slots over all 24 trays with get(0xA9) == 0.
fn 02007774 ret=u32
  nowork
  arg u32 any
  arg buf 0x1000
  stub 02007af4 force ret=ptr:arg1:0:1 args=v,v
  stub 02017e88 ret=pick:0/1 args=v,v,v

# FUN_020077B8(mgr, tray): same for one tray (-1 = current; tray >= 24: assert).
fn 020077b8 ret=u32
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg buf 0x3E0
  arg buf 0x1000
  stub 020074cc force ret=ptr:arg2:0:1 args=v
  stub 02007af4 force ret=ptr:arg3:0:1 args=v,v
  stub 02017e88 ret=pick:0/1 args=v,v,v
  stub 02034714 noreturn ret=none args=v,v,v

# FUN_0200780C(mgr, tray): if tray < 24: current tray = tray.
fn 0200780c ret=none
  nowork
  arg u32 any
  arg u32 alt range 0 23 || range 24 40 || any
  arg buf 0x3E0
  stub 020074cc force ret=ptr:arg2:0:1 args=v

# FUN_0200781C(mgr, tray): wallpaper of the tray (meta +0x3C4 + tray), 0 if tray >= 24 (no "-1 = current").
fn 0200781c ret=u8
  nowork
  arg u32 any
  arg u32 alt range 0 23 || range 24 40 || any
  arg buf 0x3E0
  stub 020074cc force ret=ptr:arg2:0:1 args=v

# FUN_02007838(mgr, tray, wallpaper): tray -1 = current; if tray < 24: meta[0x3C4 + tray] = wallpaper.
fn 02007838 ret=none
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg u8 any
  arg buf 0x3E0
  fix arg3 0 u32 alt range 0 23 || any
  stub 020074cc force ret=ptr:arg3:0:1 args=v

# FUN_0200785C(mgr, tray, strbuf): tray -1 = current; < 24: FUN_02045924(strbuf, &name[tray]) else assert.
fn 0200785c ret=none
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg u32 any
  arg buf 0x3E0
  fix arg3 0 u32 alt range 0 23 || any
  stub 020074cc force ret=ptr:arg3:0:1 args=v
  stub 02045924 ret=none args=v,in:0x28
  stub 02034714 noreturn ret=none args=v,v,v

# FUN_02007898(mgr, tray, strbuf): tray -1 = current; if tray < 24: FUN_02045A38(strbuf, &name[tray], 0x14).
fn 02007898 ret=none
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg u32 any
  arg buf 0x3E0
  fix arg3 0 u32 alt range 0 23 || any
  stub 020074cc force ret=ptr:arg3:0:1 args=v
  stub 02045a38 ret=none args=v,out:0x28,v

# FUN_020078C4(mgr, tray): number of slots of the tray (-1 = current; >= 24: assert) with get(0xA9) != 0.
fn 020078c4 ret=u32
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg buf 0x3E0
  arg buf 0x1000
  fix arg2 0 u32 alt range 0 23 || any
  stub 020074cc force ret=ptr:arg2:0:1 args=v
  stub 02007af4 force ret=ptr:arg3:0:1 args=v,v
  stub 02017e88 ret=pick:0/1 args=v,v,v
  stub 02034714 noreturn ret=none args=v,v,v

# FUN_0200791C(mgr, tray): slots with get(0xA9) != 0 and get(0x4C) == 0 (not an egg).
fn 0200791c ret=u32
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg buf 0x3E0
  arg buf 0x1000
  fix arg2 0 u32 alt range 0 23 || any
  stub 020074cc force ret=ptr:arg2:0:1 args=v
  stub 02007af4 force ret=ptr:arg3:0:1 args=v,v
  stub 02017e88 ret=pick:0/1 args=v,v,v
  stub 02034714 noreturn ret=none args=v,v,v

# FUN_02007984(mgr): sum of FUN_020078C4 over trays 0..23.
fn 02007984 ret=u32
  nowork
  arg u32 any
  stub 020078c4 force ret=mod:31 args=v,v

# FUN_020079A0(mgr): sum of FUN_0200791C over trays 0..23.
fn 020079a0 ret=u32
  nowork
  arg u32 any
  stub 0200791c force ret=mod:31 args=v,v

# FUN_020079BC(mgr, tray, slot, id, buf): return PokeParaGet(&tray[slot], id, buf); tray > 23 (but not -1) or slot > 29: assert.
# (V-0022 audit: tail bl 02017E88 + pop; the ov21 / ov170 callers compare the value)
fn 020079bc ret=u32
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg u32 alt range 0 29 || range 0 29 || range 0 29 || range 30 33
  arg u32 any
  arg u32 any
  arg buf 0x3E0
  arg buf 0x1000
  fix arg5 0 u32 alt range 0 23 || any
  stub 020074cc force ret=ptr:arg5:0:1 args=v
  stub 02007af4 force ret=ptr:arg6:0:1 args=v,v
  stub 02017e88 ret=u32 args=v,v,v
  stub 02034714 noreturn ret=none args=v,v,v

# FUN_02007A18(mgr, tray, slot): &tray[slot] (same asserts as FUN_020079BC); returned as an offset into the tray.
fn 02007a18 ret=ofs:arg4
  nowork
  arg u32 any
  arg u32 alt range 0 7 || range 0 23 || range 0 23 || const 0xFFFFFFFF || range 24 40
  arg u32 alt range 0 29 || range 0 29 || range 0 29 || range 30 33
  arg buf 0x3E0
  arg buf 0x1000
  fix arg3 0 u32 alt range 0 23 || any
  stub 020074cc force ret=ptr:arg3:0:1 args=v
  stub 02007af4 force ret=ptr:arg4:0:1 args=v,v
  stub 02034714 noreturn ret=none args=v,v,v

# FUN_02007A68(mgr, mask): meta flags (+0x3DC) |= mask.
fn 02007a68 ret=none
  nowork
  arg u32 any
  arg u8 any
  arg buf 0x3E0
  stub 020074cc force ret=ptr:arg2:0:1 args=v

# FUN_02007A80(mgr, mask): (meta flags & mask) != 0.
fn 02007a80 ret=bool
  nowork
  arg u32 any
  arg u8 any
  arg buf 0x3E0
  stub 020074cc force ret=ptr:arg2:0:1 args=v

# FUN_02007AA4(heap, savedata): mgr = alloc(heap, 4, 0, "box_savedata.c", 0x3B9); *mgr = savedata; returns mgr.
fn 02007aa4 ret=ofs:arg2
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 02030734 ret=ptr:arg2:0:1 args=v,v,v,v,v

# FUN_02007AC8(mgr): Heap_Free(mgr)  (tail call of FUN_020307B0).
fn 02007ac8 ret=none
  nowork
  arg u32 any
  stub 020307b0 ret=none args=v

# FUN_02007AD0(tray): FUN_02017550 on each of the 30 slots (tray + i * 0x88).
fn 02007ad0 ret=none
  nowork
  arg buf 0x1000
  stub 02017550 ret=none args=v

# FUN_02007AEC(): 0xFF0 = size of a tray block (30 * 0x88).
fn 02007aec ret=u32
  nowork
