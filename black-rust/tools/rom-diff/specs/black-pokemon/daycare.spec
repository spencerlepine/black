# fndiff spec (F13): the day care, `sodateya.c` (overlay 21 of the field set, 0x021C31D4-0x021C38F4), plus the day care
# save block getter FUN_0200BDB0 (main ARM9). Run: tools/check-fn.sh black-pokemon daycare --per-function
#
# Not here: the save block 50 accessors 0x0200BC10-0x0200BDA0 (S5b, black-storage `daycare`; force stubs here), the egg
# generator 0x021C38F4-0x021C4570 (P7, area `egg`; its entry 021C38F4 is a force stub here), the field-skill commands
# 0x021C457C-0x021C46F7 (black-script scrcmd_misc21b) and the GMEVENT 0x021C46F8-0x021C47AE (black-field `ov63_event`),
# which F1 counts into the same row.
#
# Every block lists the stubs of its callees and of the callees of its in-area descendants (closure stubs), so the
# area can be ported in any order. Out-of-area callees are `force` stubs: the save block accessors 0200BC10 (set egg) /
# 0200BC20 (clear egg) / 0200BC2C (slot Pokemon) / 0200BC4C (store) / 0200BC88 (free) / 0200BCA0 (egg flag) / 0200BCA8
# (occupied) / 0200BCB8 (count) / 0200BCD8 (EXP gained) / 0200BCEC (swap) / 0200BD48 (add EXP), the Pokemon accessors
# 02017E1C (get) / 02017E40 (put), the party 0201AB28 / 0201A9C8 / 0201A98C / 0201AA18 / 0201AA14, the PP restore 0201B9F0
# (reached through the in-area veneer 021C3248), the form setter 02017988, the growth / level routines 02018584 /
# 0201850C / 020185C4, the learnset reader 0201ADD0, the move setters 020181FC / 020180FC (reached through the thunks
# 020181F4 / 020180F4), the personal field 02019C90, the RNG 02005728, the field / game-data getters 02188C80 / 020120F4 /
# 0201296C / 02012934 / 0200CA1C / 02012944 / 02012F98 / 02012984, the event-work var address 020143CC, the Pokedex
# 0200CED4 / 0200CDE0, 02007C34, 0216556C, 0216AAE4, the Pokemon constructor 020173F0, Heap_AllocDebug 02030734,
# Heap_Free 020307B0, the save block getter 020071CC and the egg generator 021C38F4.
#
# Layouts. The day care object (Heap_AllocDebug 0x10) is `arg buf 0x10`: +0 u16 heap, +4 field map, +8 save block
# (a handle the save stubs log), +0xC u16 egg-roll counter. Save block answers: count 0..2, occupied / egg words 1, 0 or
# any, EXP gained near 0 or near 0xFFFFFFFF (saturation). Pokemon handles are few and fixed (0x02300000 / 0x02300100) so
# both Pokemon can be the same one. A function that stores through the event-work var pointer (020143CC) gets a trailing
# `arg buf 0x10` it never reads as a pointer (the stub answers `ptr:argK:0:1`).
#
# The learnset reader 0201ADD0 fills the 0x68-byte stack buffer with (move, level) halfword pairs that the caller walks
# until a (0xFFFF, 0xFFFF) pair; the stub fills words from a list that is 5/12 terminators (a buffer without one has
# probability (7/12)^26 < 1e-6 per input).
area daycare
set field

# FUN_0200BDB0(save data): the day care block (tail call to 020071CC(save data, 0x32)).
fn 0200bdb0 ret=u32 inputs=50
  nowork
  arg u32 any
  stub 020071cc force ret=u32 args=v,v

# FUN_021C31D4(heap, field map, save): allocate and fill the 0x10-byte object. arg3 (r3, overwritten with the
# __FILE__ pointer) is the arena the allocator stub hands out.
fn 021c31d4 ret=ofs:arg3 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 02030734 force ret=ptr:arg3:0:1 args=v,v,v,v,v

# FUN_021C31FC(obj): Heap_Free (tail call).
fn 021c31fc ret=none inputs=50
  nowork
  arg u32 any
  stub 020307b0 force ret=none args=v

# FUN_021C3204(obj, party, index): deposit party member `index` into slot `count` (if fewer than two are occupied): PP
# restore, Shaymin back to form 0, store, remove from the party, 021C38DC(field map, party).
fn 021c3204 ret=none inputs=600
  nowork
  arg buf 0x10
  arg u32 oneof 0x02400000 0x02400040
  arg u32 alt range 0 6 || any
  stub 0200bcb8 force ret=pick:0/1/2/2 args=v
  stub 0200bc4c force ret=none args=v,v,v
  stub 0201ab28 force ret=u32 args=v,v
  stub 0201b9f0 force ret=none args=v
  stub 02017e1c force ret=pick:0x1ec/0x1ec/5/0/0x1eb/0x101ec args=v,v,v
  stub 02017988 force ret=none args=v,v
  stub 0201a9c8 force ret=none args=v,v
  stub 02188c80 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012f98 force ret=u32 args=v
  stub 02007c34 force ret=none args=v,v

# FUN_021C3248(mon): veneer `ldr r3, =0x0201B9F1; bx r3` (PP restore).
fn 021c3248 ret=none inputs=50
  nowork
  arg u32 oneof 0x02300000 0x02300100
  stub 0201b9f0 force ret=none args=v

# FUN_021C3250(mon): species 0x1EC (Shaymin) -> form 0.
fn 021c3250 ret=none inputs=200
  nowork
  arg u32 oneof 0x02300000 0x02300100
  stub 02017e1c force ret=pick+u32:0x1ec/0x1ec/0x1eb/0x1ed/0/0x101ec/0x11ec args=v,v,v
  stub 02017988 force ret=none args=v,v

# FUN_021C3270(obj, slot, party): withdraw an occupied slot: day care EXP and moves, Pokedex, add to the party, free the
# slot, move slot 1 down.
fn 021c3270 ret=none inputs=800
  nowork
  arg buf 0x10
  arg u32 alt range 0 2 || any
  arg u32 oneof 0x02400000 0x02400040
  stub 0200bca8 force ret=pick+u32:0/1/1/2 args=v,v
  stub 0200bc2c force ret=u32 args=v,v
  stub 0200bcd8 force ret=pick+u32:0/1/0x100/0xFFFFFF00/0xFFFFFFFF args=v,v
  stub 0200bc88 force ret=none args=v,v
  stub 0200bcec force ret=none args=v,v,v
  stub 02017e1c force seq ret=pick+u32:0/1/2/5/10/20/0x21/100 args=v,v,v
  stub 02018584 force ret=pick+u32:0/100/1000/0xFFFFFFFF args=v,v,v
  stub 02017e40 force ret=none args=v,v,v
  stub 020185c4 force ret=none args=v
  stub 0201add0 force ret=none args=v,v,outw:0x68:0xFFFFFFFF/0xFFFFFFFF/0xFFFFFFFF/0xFFFFFFFF/0xFFFFFFFF/0x00050001/0x000A0002/0x00140021/0x00010000/0x0005FFFF/0xFFFF0005/0x00640064
  stub 020181fc force ret=none args=v,v,v
  stub 020180fc force ret=none args=v,v
  stub 02188c80 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0200ca1c force ret=u32 args=v
  stub 0200ced4 force ret=pick:0/0/1 args=v,v
  stub 0200cde0 force ret=none args=v,v
  stub 0201a98c force ret=none args=v,v

# FUN_021C32BC(obj, mon): register the species in the Pokedex if 0200CED4 says it is not.
fn 021c32bc ret=none inputs=300
  nowork
  arg buf 0x10
  arg u32 oneof 0x02300000 0x02300100
  stub 02188c80 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0200ca1c force ret=u32 args=v
  stub 02017e1c force ret=pick+u32:1/5/0x1ec args=v,v,v
  stub 0200ced4 force ret=pick:0/0/1 args=v,v
  stub 0200cde0 force ret=none args=v,v

# FUN_021C32F4(obj, party): take the egg: a new Pokemon, filled by the egg generator, added to the party; the egg flag and
# var 0x4139 cleared; the seasonal forms refreshed. arg2 (r2, overwritten) is the var arena.
fn 021c32f4 ret=none inputs=400
  nowork
  arg buf 0x10
  arg u32 oneof 0x02400000 0x02400040
  arg buf 0x10
  stub 0200bca0 force ret=pick+u32:0/1/1/2 args=v
  stub 0200bc20 force ret=none args=v
  stub 0200bc2c force ret=u32 args=v,v
  stub 020173f0 force ret=u32 args=v,v,v,v,v
  stub 0216556c force ret=none
  stub 02188c80 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0201296c force ret=u32 args=v
  stub 020143cc force ret=ptr:arg2:0:1 args=v,v
  stub 02012944 force ret=u32 args=v
  stub 021c38f4 force ret=none args=v,v,v,v,v,v
  stub 0201a98c force ret=none args=v,v
  stub 020307b0 force ret=none args=v
  stub 02012984 force ret=u8 args=v
  stub 0216aae4 force ret=none args=v,v,v

# FUN_021C335C(obj): refuse the egg: egg flag and var 0x4139 cleared (if the flag is 1). arg1 is the var arena.
fn 021c335c ret=none inputs=200
  nowork
  arg buf 0x10
  arg buf 0x10
  stub 0200bca0 force ret=pick+u32:0/1/1/2 args=v
  stub 0200bc20 force ret=none args=v
  stub 02188c80 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0201296c force ret=u32 args=v
  stub 020143cc force ret=ptr:arg1:0:1 args=v,v

# FUN_021C3378(obj): the day care step (from the field step hook): one EXP point for both slots, then the egg roll; an
# egg sets the flag and (party not full) var 0x4139 = 1. arg1 (r1, overwritten with 1) is the var arena.
fn 021c3378 ret=u32 inputs=1500
  nowork
  arg buf 0x10
  arg buf 0x10
  fix arg0 0xC u16 alt range 0xE0 0x110 || any
  stub 0200bd48 force ret=none args=v,v
  stub 0200bcb8 force ret=pick:0/1/2/2 args=v
  stub 0200bc2c force ret=u32 args=v,v
  stub 0200bc10 force ret=none args=v
  stub 02005728 force ret=pick+u32:0/19/20/49/50/69/70 args=v
  stub 02017e1c force ret=pick:0/1/2/0x84/0x84/5/7 args=v,v,v
  stub 02019c90 force ret=pick:1/2/3/15/1/2 args=v,v,v
  stub 02188c80 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0201296c force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/5/6 args=v
  stub 0201aa14 force ret=pick:6/6/5 args=v
  stub 020143cc force ret=ptr:arg1:0:1 args=v,v

# FUN_021C33A4(field map): var 0x4139 = 1 if the party is not full. arg1 is the var arena.
fn 021c33a4 ret=none inputs=300
  nowork
  arg u32 any
  arg buf 0x10
  stub 02188c80 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0201296c force ret=u32 args=v
  stub 02012934 force ret=u32 args=v
  stub 0201aa18 force ret=pick:0/1/5/6/7 args=v
  stub 0201aa14 force ret=pick:6/6/5 args=v
  stub 020143cc force ret=ptr:arg1:0:1 args=v,v

# FUN_021C33E0(field map): var 0x4139 = 0. arg1 is the var arena.
fn 021c33e0 ret=none inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  stub 02188c80 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 0201296c force ret=u32 args=v
  stub 020143cc force ret=ptr:arg1:0:1 args=v,v

# FUN_021C3400(obj): the egg flag (tail call to 0200BCA0).
fn 021c3400 ret=u32 inputs=100
  nowork
  arg buf 0x10
  stub 0200bca0 force ret=pick+u32:0/1/1/2 args=v

# FUN_021C340C(obj): number of occupied slots (u8).
fn 021c340c ret=u8 inputs=100
  nowork
  arg buf 0x10
  stub 0200bcb8 force ret=pick+u32:0/1/2/2 args=v

# FUN_021C341C(obj, slot): the level the slot's Pokemon would have now (1 if the slot is not occupied).
fn 021c341c ret=u32 inputs=500
  nowork
  arg buf 0x10
  arg u32 alt range 0 2 || any
  stub 0200bca8 force ret=pick+u32:0/1/1/2 args=v,v
  stub 0200bc2c force ret=u32 args=v,v
  stub 0200bcd8 force ret=pick+u32:0/1/0x100/0xFFFFFF00/0xFFFFFFFF args=v,v
  stub 02017e1c force ret=pick+u32:0/1/5/100/0xFFFFFF00 args=v,v,v
  stub 0201850c force ret=u32 args=v,v,v

# FUN_021C3490(obj, slot): levels gained = 021C341C - stored level.
fn 021c3490 ret=u32 inputs=500
  nowork
  arg buf 0x10
  arg u32 alt range 0 2 || any
  stub 0200bca8 force ret=pick+u32:0/1/1/2 args=v,v
  stub 0200bc2c force ret=u32 args=v,v
  stub 0200bcd8 force ret=pick+u32:0/1/0x100/0xFFFFFF00/0xFFFFFFFF args=v,v
  stub 02017e1c force seq ret=pick+u32:0/1/5/100 args=v,v,v
  stub 0201850c force ret=pick+u32:1/5/6/100 args=v,v,v

# FUN_021C34B4(obj, slot): the fee, (levels gained + 1) * 100.
fn 021c34b4 ret=u32 inputs=300
  nowork
  arg buf 0x10
  arg u32 alt range 0 2 || any
  stub 0200bca8 force ret=pick+u32:0/1/1/2 args=v,v
  stub 0200bc2c force ret=u32 args=v,v
  stub 0200bcd8 force ret=pick+u32:0/1/0x100/0xFFFFFF00/0xFFFFFFFF args=v,v
  stub 02017e1c force seq ret=pick+u32:0/1/5/100 args=v,v,v
  stub 0201850c force ret=pick+u32:1/5/6/100 args=v,v,v

# FUN_021C34C4(obj): the compatibility of the two slots (0 unless both are occupied).
fn 021c34c4 ret=u8 inputs=600
  nowork
  arg buf 0x10
  stub 0200bcb8 force ret=pick:0/1/2/2 args=v
  stub 0200bc2c force ret=u32 args=v,v
  stub 02017e1c force ret=pick:0/1/2/0x84/0x84/5/7 args=v,v,v
  stub 02019c90 force ret=pick:1/2/3/15/1/2 args=v,v,v

# FUN_021C34F8(obj, slot): the slot's Pokemon (0200BCA8 called and ignored, then 0200BC2C).
fn 021c34f8 ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg u32 alt range 0 2 || any
  stub 0200bca8 force ret=pick+u32:0/1/1/2 args=v,v
  stub 0200bc2c force ret=u32 args=v,v

# FUN_021C3510(a, b): a + b saturating at 0xFFFFFFFF.
fn 021c3510 ret=u32 inputs=400
  nowork
  arg u32 alt range 0 0x100 || range 0xFFFFFF00 0xFFFFFFFF || any
  arg u32 alt range 0 0x100 || range 0xFFFFFF00 0xFFFFFFFF || any

# FUN_021C3530(mon, gained): day care EXP: exp = min(exp + gained (saturating), EXP of level 100), recalc, then learn
# every level-up move with old level < move level <= new level.
fn 021c3530 ret=none inputs=1000
  nowork
  arg u32 oneof 0x02300000 0x02300100
  arg u32 alt range 0 0x100 || range 0xFFFFFF00 0xFFFFFFFF || any
  stub 02017e1c force seq ret=pick+u32:0/1/2/5/10/20/0x21/100 args=v,v,v
  stub 02018584 force ret=pick+u32:0/100/1000/0xFFFFFFFF args=v,v,v
  stub 02017e40 force ret=none args=v,v,v
  stub 020185c4 force ret=none args=v
  stub 0201add0 force ret=none args=v,v,outw:0x68:0xFFFFFFFF/0xFFFFFFFF/0xFFFFFFFF/0xFFFFFFFF/0xFFFFFFFF/0x00050001/0x000A0002/0x00140021/0x00010000/0x0005FFFF/0xFFFF0005/0x00640064
  stub 020181fc force ret=none args=v,v,v
  stub 020180fc force ret=none args=v,v

# FUN_021C360C(mon, move): learn a move unless it is 0 or known: the first empty slot, else push out the first.
fn 021c360c ret=none inputs=500
  nowork
  arg u32 oneof 0x02300000 0x02300100
  arg u32 alt oneof 0 1 2 0x21 0x10021 || any
  stub 02017e1c force seq ret=pick+u32:0/1/2/0x21/0x10021 args=v,v,v
  stub 020181fc force ret=none args=v,v,v
  stub 020180fc force ret=none args=v,v

# FUN_021C3670(mon, move): 1 if one of the four move ids equals `move`.
fn 021c3670 ret=u32 inputs=400
  nowork
  arg u32 oneof 0x02300000 0x02300100
  arg u32 alt oneof 0 1 2 0x21 || any
  stub 02017e1c force seq ret=pick+u32:0/1/2/0x21 args=v,v,v

# FUN_021C36B0(save): move an occupied slot 1 down into an empty slot 0.
fn 021c36b0 ret=none inputs=300
  nowork
  arg u32 any
  stub 0200bca8 force ret=pick+u32:0/1/1/2 args=v,v
  stub 0200bcec force ret=none args=v,v,v

# FUN_021C36EC(a, b): compatibility: 0 for an Undiscovered egg group (15), two Dittos, equal or genderless genders, no
# shared egg group; else the species / OT score of 021C37F8 (a Ditto pairs with anything).
fn 021c36ec ret=u32 inputs=1000
  nowork
  arg u32 oneof 0x02300000 0x02300100 0x02300200
  arg u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c force ret=pick:0/1/2/0x84/0x84/5/7 args=v,v,v
  stub 02019c90 force ret=pick:1/2/3/15/1/2 args=v,v,v

# FUN_021C37F8(a, b): 50 same species / same OT, 70 same species / other OT, 20 other species / same OT, 50 else.
fn 021c37f8 ret=u32 inputs=400
  nowork
  arg u32 oneof 0x02300000 0x02300100 0x02300200
  arg u32 oneof 0x02300000 0x02300100 0x02300200
  stub 02017e1c force ret=pick:0/1/0x84/5 args=v,v,v

# FUN_021C385C(obj): every 16th step (u16 counter += 0x10, past 0xFF reset), with both slots occupied: 1 if
# rand(1600) < compatibility.
fn 021c385c ret=u32 inputs=1000
  nowork
  arg buf 0x10
  fix arg0 0xC u16 alt range 0xE0 0x110 || any
  stub 0200bcb8 force ret=pick:0/1/2/2 args=v
  stub 0200bc2c force ret=u32 args=v,v
  stub 02005728 force ret=pick+u32:0/19/20/49/50/69/70 args=v
  stub 02017e1c force ret=pick:0/1/2/0x84/0x84/5/7 args=v,v,v
  stub 02019c90 force ret=pick:1/2/3/15/1/2 args=v,v,v

# FUN_021C3898(obj, egg): the egg generator 021C38F4(heap, profile, 0xEA62, parent 0, parent 1, egg).
fn 021c3898 ret=none inputs=200
  nowork
  arg buf 0x10
  arg u32 any
  stub 0200bc2c force ret=u32 args=v,v
  stub 02188c80 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012944 force ret=u32 args=v
  stub 021c38f4 force ret=none args=v,v,v,v,v,v

# FUN_021C38DC(field map, party): 02007C34(02012F98(game data), party).
fn 021c38dc ret=none inputs=100
  nowork
  arg u32 any
  arg u32 oneof 0x02400000 0x02400040
  stub 02188c80 force ret=u32 args=v
  stub 020120f4 force ret=u32 args=v
  stub 02012f98 force ret=u32 args=v
  stub 02007c34 force ret=none args=v,v
