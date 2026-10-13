# svflow spec: client3 (btl_client.c part 3, 0x021CF5B0-0x021D10B8: mon-selection checks and party/target helpers; B13-spec1).
# Harness-job owned; porting workers may not edit. Grammar: svflow_specs/README.md. Area split: doc comment of client/mod.rs.
#
# Same object as client1.spec (`arg buf 0x1C0`, `nowork`). Extra fields used here: +0x034 (u32, passed to 021efce4), +0x0D8 / +0x0DC result
# pointer and size, +0x10C 6 signed bytes (0xFF = empty slot; 021cfba4/fbbc/fbe0), +0x118 party/mon container (u32, `0x46 << 2`),
# +0x11C u8 (side id for 021d7f2c), +0x124 result words (4 bytes each, 021d0b6c).
# Mon queries (021d59b4 BattleMon_Get(mon, field), 021d5814, 021d62c4 flag test, 021d631c, 021d5b68, 021b9c00 party slot) are stubs; the
# BattleMon accessors are verified by `battle_mon`.
# Not specced in this batch: 021cf5b0 (the big pick routine, 0x26C bytes), 021cf81c 021cf8d4 021cf918 021cf9b4 021cfadc (they call
# 021cf5b0), 021cfbf0 021cfd50 021cfe08.. 021d0770 (step functions / tables of stub calls), 021d0c78 (reads the adapter block through
# 021eec20, an adapter smoke port: `force` stub and an `outp` buffer needed).
area client3

# can_use: 0 when the target mon's flag 0x13 is set (021d62c4) or the party list answers 021efce4(c.view_party, 7); else (BattleMon_Get(mon,
# 0x11) != 0x67)
fn 021cf89c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  stub 021efce4 ret=pick:0/1 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v

# (BattleMon_Get(mon, 0x11) != 0x17); r0 is unused
fn 021cfac4 ret=u32
  nowork
  arg u32 any
  arg u32 any
  stub 021d59b4 ret=pick:0x17/0x17/3 args=v,v

# 021d5814(mon, 8) != 0; r0 is unused
fn 021cfb8c ret=u32
  nowork
  arg u32 any
  arg u32 any
  stub 021d5814 ret=pick:0/1 args=v,v

# c[0x10C + i] = 0xFF for i in 0..6
fn 021cfba4 ret=none
  nowork
  arg buf 0x1C0

# is r1 one of the six signed bytes c[0x10C..0x112]
fn 021cfbbc ret=u32
  nowork
  arg buf 0x1C0
  arg s32 alt range 0 5 || oneof 0xFFFFFFFF 0x80
  fix arg0 0x10C u8 alt range 0 5 || const 0xFF rep=6,1

# c[0x10C + i] = v when i < 6
fn 021cfbe0 ret=none
  nowork
  arg buf 0x1C0
  arg u32 alt range 0 7 || any
  arg u32 any

# the mon has flag 0x14 (021d62c4) and its counter (021d631c + 1) equals 021e8afc(021d6314(mon, 0x14)); r0 is unused
fn 021cfdcc ret=u32
  nowork
  arg u32 any
  arg u32 any
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021e8afc ret=pick:1/2/3 args=v
  stub 021d631c ret=pick:0/1/2 args=v,v

# list the party slots that 021d5b7c accepts, starting at 021d7f2c(battle type, c[0x11C]); out[k++] = slot (out may be null); returns k
fn 021d0980 ret=u32
  nowork
  region out 8
  arg buf 0x1C0
  arg u32 alt const 0 || addrof region:out
  stub 021b9b94 force ret=pick:0/1/2/3/6 args=v
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b7c ret=pick:0/1 args=v

# result words: for i in 0..r1[6]: 021d8394(&c[0x124 + 4i], r1[3 + i], r1[i]); then result pointer = &c[0x124], size = 4 * count
fn 021d0b6c ret=none
  nowork
  arg buf 0x1C0
  arg buf 0x40
  fix arg1 6 u8 range 0 8
  stub 021d8394 ret=none args=p,v,v

# battle type == 3 and the mon in slot 0 of the party container passes 021d5b68
fn 021d102c ret=bool
  nowork
  arg buf 0x1C0
  stub 021b8588 force ret=pick:0/3 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v

# two checks (sides 2 and 3: 021b9ab0 -> party slot -> 021d5b68): the first failing one is reported to r1 through 021d83fc(r1, side) and
# returns 4; both passing returns 0
fn 021d1058 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x40
  stub 021b9ab0 force ret=u8 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d83fc ret=none args=p,v

# ---------------------------------------------------------------------------------------------------------------------------------- B13-spec2
# B13-spec2 specs every function the B13-spec1 header above lists as "not specced in this batch" (list in client/mod.rs).
# The rest of the range: move-selection checks (021cf5b0 "this move cannot be chosen" with its message, 021cf81c usable-move mask, 021cf8d4 /
# 021cf918 / 021cf9b4 escape and trapping checks), the trainer-AI switch heuristics (021cfbf0 and its tests 021cfd50 .. 021d06c8; they draw
# from a 64-bit LCG kept in the client object: state c+0xF4/0xF8, multiplier c+0xFC/0x100, increment c+0x104/0x108, stepped through the
# real 64-bit multiply 0209c054, which runs as code on both sides (ALLOW)), the AI action step 021d0770 (request 3 mode 1, calls the overlay-96
# AI 02187fb0 / 021880bc / 02188134 / 02188140: stubs), the switch-in steps 021d0cc4 .. 021d0e7c (requests 7, 5, 6, 4) and their helpers.
# ROM tables read as data: 0x021EFFC2 (4 x {u8 type, pad, u16 abilities[4]}, 021d0100).
# BattleMon accessors 021d5440-021d7f04 and the waza-data accessors 0201c314 / 0201c1c8 / 0201c1d8 / 0201beb8 (black-pokemon `waza`, which
# need its global cache) are stubs. Veneers 021d0b3c, 021d0b4c, 021d0b5c (Thumb `bx pc` to ARM thunks to 0x0689xxxx) are stubbed at the
# Thumb entry. 021d0d38 / 021d0d44 / 021d0e74 are table-entry thunks (`ldr r3, =fn; [movs r2, #k;] bx r3`).

# "move cannot be chosen" (move r2 for mon r1; r3 = message buffer or 0, filled with the reason): 0xA5 (struggle) -> 0. Encore-like flag 0x1B
# (021cf89c usable, 021d62c4(mon, 0x1B), m = 021e8b80(021d6314(mon, 0x1B)), 021d5750(mon, m), m != move) -> msg 0x63; flag 0x17 (m !=
# move) -> msg 0x64; flag 0xB and !0201c314(move) -> msg 0x23B; flag 0xC and 021d67b8(mon) == move -> msg 0x244; flag 0xD and
# 021d62d8(mon, 0xD) == move -> msg 0x253; flag 0xF and 0201beb8(move, 0xC) -> msg 0x37A; 021efce4(c+0x34, 3) and 021efc1c(c+0x34,
# pokecon, mon, move) -> msg 0x24D; 021efce4(c+0x34, 2) and 0201beb8(move, 9) -> msg 0x43E (each: 021eaf18(buf, 1|2, id), 021eaf50(buf,
# 021d5b40 / 021d5440(mon)), 021eaf50(buf, value)); returns 1 when blocked
fn 021cf5b0 ret=u32 inputs=600
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 alt range 1 4 || const 0xA5 || any
  arg u32 alt const 0 || any
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d62c4 ret=pick:0/0/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021e8b80 ret=pick:1/2/3/4 args=v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021eaf18 ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021eaf50 ret=none args=p,v
  stub 021d5440 ret=u32 args=v
  stub 0201c314 ret=pick:0/1 args=v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021efce4 ret=pick:0/1 args=v,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v

# usable-move mask: out[j] = 021d5584(mon, j) and !021cf5b0(c, mon, 021d54c0(mon, j), 0) for j < 021d5448(mon), out[j] = 0 up to 4;
# returns the first usable j (4 = none)
fn 021cf81c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x10
  stub 021d5448 ret=pick:0/1/2/3/4 args=v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d54c0 ret=u16 args=v,v
  stub 021cf5b0 ret=pick:0/0/1 args=p,v,v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 0201c314 ret=pick:0/1 args=v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d5440 ret=u32 args=v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v
  stub 021d5b40 ret=u32 args=v
  stub 021d62c4 ret=pick:0/0/1 args=v,v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021e8b80 ret=pick:1/2/3/4 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# can the mon r1 escape: usable (021cf89c) and holding item 0x127 -> 4; r = 021cf9b4(c, mon, outb, outh); 4 -> *outb = 0x1F, *outh = 0;
# returns r
fn 021cf8d4 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d5b40 ret=pick:0x127/0/5 args=v
  stub 021cf9b4 ret=pick:0/3/4/4 args=p,v,p,p
  stub 021b8d38 force ret=pick:0/1/2/3/4 args=v,v,v,outm:4:0xFF
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021cfac4 ret=pick:0/1 args=p,v
  stub 021cfadc ret=pick:0/1 args=p,v
  stub 021cfb8c ret=pick:0/1 args=p,v
  stub 021d5440 ret=u32 args=v
  stub 021d5814 ret=pick:0/0/1 args=v,v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# can the client run: for each of the c[0x11C] party mons (021b9c00(c+0x118, i)) alive (021d5b68 == 0): usable and item 0xE4 -> 4, ability
# (021d59b4(mon, 0x11)) 0x32 -> 4; then the first 021cf9b4(c, mon, outb, outh) != 4 is returned; else 4
fn 021cf918 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x10
  fix arg0 0x11C u8 alt range 0 4 || any
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d5b40 ret=pick:0xE4/0/5 args=v
  stub 021d59b4 ret=pick:0x32/0/1/2 args=v,v
  stub 021cf9b4 ret=pick:0/3/4/4/4 args=p,v,p,p
  stub 021b8d38 force ret=pick:0/1/2/3/4 args=v,v,v,outm:4:0xFF
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021cfac4 ret=pick:0/1 args=p,v
  stub 021cfadc ret=pick:0/1 args=p,v
  stub 021cfb8c ret=pick:0/1 args=p,v
  stub 021d5440 ret=u32 args=v
  stub 021d5814 ret=pick:0/0/1 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# trapping check for mon r1: the opponents n = 021b8d38(server, pokecon, 021b8d8c(server, pokecon, 021d5440(mon)) | 0x100, list) (each
# m = 021b9934(pokecon, list[i])): ability 0x17 and 021cfac4 / 0x47 and 021cfadc / 0x2A and 021cfb8c -> *outb = 021d5440(m), *outh = the
# ability, return 0; then the mon's flags 0x16 / 8 / 0x15 (021d62c4) -> *outb = 021d5440(mon), *outh = 0, return 3; else 4
fn 021cf9b4 ret=u32 inputs=400
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  stub 021d5440 ret=u32 args=v
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b8d38 force ret=pick:0/1/2/3/4 args=v,v,v,outm:4:0xFF
  stub 021b9934 force ret=u32 args=v,v
  stub 021d59b4 ret=pick:0x17/0x47/0x2A/5 args=v,v
  stub 021cfac4 ret=pick:0/1 args=p,v
  stub 021cfadc ret=pick:0/1 args=p,v
  stub 021cfb8c ret=pick:0/1 args=p,v
  stub 021d62c4 ret=pick:0/0/0/1 args=v,v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d5814 ret=pick:0/0/1 args=v,v
  stub 021d5b40 ret=pick:0x116/0x21D/5 args=v
  stub 021efce4 ret=pick:0/0/1 args=v,v

# arena-trap test for mon r1: u = 021cf89c(c, mon); 021efce4(c+0x34, 2) -> 1; flags 0x1F / 0x15 -> 1; u and item 0x116 -> 1; ability 0x1A
# (levitate) -> 0; 021d5814(mon, 2) -> 0; flags 0x1E / 0x20 -> 0; u and item 0x21D -> 0; else 1
fn 021cfadc ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021efce4 ret=pick:0/0/1 args=v,v
  stub 021d62c4 ret=pick:0/0/0/1 args=v,v
  stub 021d5b40 ret=pick:0x116/0x21D/5 args=v
  stub 021d59b4 ret=pick:0x1A/5 args=v,v
  stub 021d5814 ret=pick:0/0/1 args=v,v

# trainer AI "switch out?" for mon r1 (r2 = slot index for 021cfbe0, r3 = out byte): 021cf8d4(c, mon, &b, &h) != 4 -> 0; 021b9b98(party) <=
# c[0x11C] -> 0; target = 021cfd50(c, 021b8d8c(server, pokecon, 021d5440(mon))) (none -> 0), 021d6c24(target) -> target =
# 021b937c(server, pokecon, target); the first test that fires among 021cfdcc, 021cfe08, 021cfe98, 021cffac, 021d0100, 021d0248,
# 021d0304 (some pick the slot, else 6): slot 6 -> the best of the 021d0980 list sorted by 021d09e0 that is not taken (021cfbbc); slot !=
# 6 -> 021cfbe0(c, r2, slot), *out = slot, return 1; else 0
fn 021cfbf0 ret=u32 inputs=600
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 range 0 7
  arg buf 0x10
  fix arg0 0x11C u8 alt range 0 3 || any
  fix arg0 0x10C u8 alt range 0 5 || const 0xFF rep=6,1
  stub 021cf8d4 ret=pick:4/4/4/0 args=p,v,p,p
  stub 021b9b98 force ret=pick:0/2/3/4/6 args=v
  stub 021d5440 ret=u32 args=v
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021cfd50 ret=nullable:4:u32 args=p,v
  stub 021d6c24 ret=pick:0/1 args=v
  stub 021b937c force ret=u32 args=v,v,v
  stub 021cfdcc ret=pick:0/0/0/1 args=p,v
  stub 021cfe08 ret=pick:0/0/0/1 args=p,v,v
  stub 021cfe98 ret=pick:0/0/0/1 args=p,v,v
  stub 021cffac ret=pick:0/0/0/1 args=p,v,v
  stub 021d0100 ret=pick:0/0/0/1 args=p,v,v,outm:1:7
  stub 021d0248 ret=pick:0/0/0/1 args=p,v,outm:1:7
  stub 021d0304 ret=pick:0/0/0/1 args=p,v,v,outm:1:7
  stub 021d0980 ret=pick:0/1/2/3 args=p,outm:6:7
  stub 021d09e0 ret=none args=p,p,v,v
  stub 021cfbe0 ret=none args=p,v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 0201c1c8 ret=u32 args=v
  stub 0201c1d8 ret=pick:0/5/40/80/120 args=v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 021b8588 force ret=pick:0/0/1 args=v
  stub 021b8d38 force ret=pick:0/1/2/3/4 args=v,v,v,outm:4:0xFF
  stub 021b9934 force ret=u32 args=v,v
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021cf5b0 ret=pick:0/0/1 args=p,v,v,v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021cf9b4 ret=pick:0/3/4/4 args=p,v,p,p
  stub 021cfac4 ret=pick:0/1 args=p,v
  stub 021cfadc ret=pick:0/1 args=p,v
  stub 021cfb8c ret=pick:0/1 args=p,v
  stub 021d04e4 ret=pick:0/1 args=p,v,outm:1:7
  stub 021d055c ret=pick:0/1 args=p,v,v
  stub 021d05c4 ret=pick:0/1 args=p,v,v,outm:1:7
  stub 021d0644 ret=pick:0/0/1 args=p,v,v,v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021d57fc ret=u32 args=v
  stub 021d5814 ret=pick:0/0/1 args=v,v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v
  stub 021d5b40 ret=pick:0x127/0/5 args=v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d5b7c ret=pick:0/1 args=v
  stub 021d5c44 ret=pick:0/0x31FFF/0x32000/0x40000 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d631c ret=pick:0/1/2 args=v,v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021d6940 ret=pick:0/1/1/1 args=v,v,v,outm:8:0xFF/0xFF/0xFF/0xFF/0xF/0/0xFF/0xFF
  stub 021d799c ret=pick:0/1/2/3/4/5 args=v,v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021e8afc ret=pick:1/2/3 args=v
  stub 021e8b80 ret=pick:1/2/3/4 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# random opponent of side r1: n = 021b8d38(server, pokecon, side | 0x100, list); LCG step; 021b9934(pokecon, list[(state_hi * n) >> 32]);
# 0 when n == 0
fn 021cfd50 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  stub 021b8d38 force ret=pick:0/1/2/3/4 args=v,v,v,outm:4:0xFF
  stub 021b9934 force ret=u32 args=v,v

# wonder-guard test (target r2): battle type 0 and 021d59b4(target, 0x11) == 0x19 and !021d0644(c, mon, target, 4) and 021d055c(c, target,
# 4) -> LCG step, (state_hi * 3) >> 32 < 2
fn 021cfe08 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  stub 021b8588 force ret=pick:0/0/1 args=v
  stub 021d59b4 ret=pick:0x19/0x19/5 args=v,v
  stub 021d0644 ret=pick:0/0/1 args=p,v,v,v
  stub 021d055c ret=pick:0/1 args=p,v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 0201c1c8 ret=u32 args=v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021cf5b0 ret=pick:0/0/1 args=p,v,v,v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d5440 ret=u32 args=v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021d57fc ret=u32 args=v
  stub 021d5b40 ret=u32 args=v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d62c4 ret=pick:0/0/1 args=v,v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021d799c ret=pick:0/1/2/3/4/5 args=v,v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021e8b80 ret=pick:1/2/3/4 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# super-effective test: t = 021d57fc(target); any move j of mon (021d54c0, 0201c314) whose type 0201c1c8 is not resisted
# (021d799c(type, t) != 0) -> 0; at least 2 damaging moves: 021d055c(c, target, 4) -> LCG (hi * 3) >> 32 < 2; else 021d055c(c, target, 3)
# -> LCG hi bit 31 clear; else 0
fn 021cfe98 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  stub 021d57fc ret=u32 args=v
  stub 021d5448 ret=pick:0/2/3/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 0201c1c8 ret=u32 args=v
  stub 021d799c ret=pick:0/0/0/1 args=v,v
  stub 021d055c ret=pick:0/1 args=p,v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021b8588 force ret=u32 args=v
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021cf5b0 ret=pick:0/0/1 args=p,v,v,v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d0644 ret=pick:0/0/1 args=p,v,v,v
  stub 021d5440 ret=u32 args=v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v
  stub 021d5b40 ret=u32 args=v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d62c4 ret=pick:0/0/1 args=v,v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021e8b80 ret=pick:1/2/3/4 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# encore-like flag 0x1B test: m = 021e8b80(021d6314(mon, 0x1B)) (0 -> 0), d = 0201c314(m), immune (021d799c(0201c1c8(m), 021d57fc(target))
# == 0) and d: 021d055c(c, target, 4) -> LCG (hi * 3) >> 32 < 2, 021d055c(c, target, 3) -> LCG hi bit 31 clear; !d -> LCG hi bit 31 clear
fn 021cffac ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  stub 021d62c4 ret=pick:0/1/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021e8b80 ret=pick:0/1/2 args=v
  stub 0201c314 ret=pick:0/1 args=v
  stub 0201c1c8 ret=u32 args=v
  stub 021d57fc ret=u32 args=v
  stub 021d799c ret=pick:0/0/1 args=v,v
  stub 021d055c ret=pick:0/1 args=p,v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021b8588 force ret=u32 args=v
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021cf5b0 ret=pick:0/0/1 args=p,v,v,v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d0644 ret=pick:0/0/1 args=p,v,v,v
  stub 021d5440 ret=u32 args=v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v
  stub 021d5b40 ret=u32 args=v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# absorb-ability switch: 021d0644(c, mon, target, 4) and LCG (hi * 3) >> 32 == 0 -> 0; for each turn k of 021d6940(mon, 1, k, rec): the
# entries of the ROM table 0x021EFFC2 whose type is rec[4]: an ability id != 0 that 021d04e4(c, ability, &slot) finds, LCG hi bit 31 clear
# -> *out = slot, return 1; else 0
fn 021d0100 ret=u32 inputs=600
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 021d0644 ret=pick:0/1 args=p,v,v,v
  stub 021d6940 ret=pick:0/1/1/1 args=v,v,v,outm:8:0xFF/0xFF/0xFF/0xFF/0xF/0/0xFF/0xFF
  stub 021d04e4 ret=pick:0/1 args=p,v,outm:1:7
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 0201c1c8 ret=u32 args=v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 021b8588 force ret=u32 args=v
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021cf5b0 ret=pick:0/0/1 args=p,v,v,v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d5440 ret=u32 args=v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021d57fc ret=u32 args=v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v
  stub 021d5b40 ret=u32 args=v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d62c4 ret=pick:0/0/1 args=v,v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021d799c ret=pick:0/1/2/3/4/5 args=v,v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021e8b80 ret=pick:1/2/3/4 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# natural-cure switch: ability 0x1E and status flag 2 or 3 and 021d5c44(mon) >= 0x32000: 021d6940(mon, 1, 0, rec) ? (021d05c4(c, rec[4], 2,
# &slot) -> *out = slot, 1 : 0) : LCG hi bit 31 clear -> 1; else 0
fn 021d0248 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x10
  stub 021d59b4 ret=pick:0x1E/0x1E/5 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d5c44 ret=pick:0/0x31FFF/0x32000/0x40000 args=v
  stub 021d6940 ret=pick:0/1 args=v,v,v,outm:8:0xFF
  stub 021d05c4 ret=pick:0/1 args=p,v,v,outm:1:7
  stub 021b8588 force ret=u32 args=v
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d57fc ret=u32 args=v
  stub 021d5b7c ret=pick:0/1 args=v
  stub 021d799c ret=pick:0/1/2/3/4 args=v,v
  stub 021d7f2c ret=pick:0/1/2 args=v,v

# stat-boost switch: 021d0644(c, mon, target, 4) and LCG (hi * 10) >> 32 != 0 -> 0; sum of 021d59b4(mon, 1..7) >= 4 -> 0;
# 021d6940(mon, 1, 0, rec): for slots i from 021d7f2c(battle type, c[0x11C]) below 021b9b94(party), not taken (021cfbbc), 021d0644(c,
# m, target, 4): 021d799c(rec[4], 021d57fc(m)) == 0 -> LCG hi bit 31 clear -> *out = i, 1; < 3 -> LCG (hi * 3) >> 32 == 0 -> *out = i, 1
fn 021d0304 ret=u32 inputs=600
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  arg buf 0x10
  fix arg0 0x10C u8 alt range 0 5 || const 0xFF rep=6,1
  stub 021d0644 ret=pick:0/0/1 args=p,v,v,v
  stub 021d59b4 ret=pick:0/0/0/1 args=v,v
  stub 021d6940 ret=pick:0/1/1 args=v,v,v,outm:8:0xFF
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b8588 force ret=u32 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d57fc ret=u32 args=v
  stub 021d799c ret=pick:0/1/2/3/4 args=v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 0201c1c8 ret=u32 args=v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 021cf5b0 ret=pick:0/0/1 args=p,v,v,v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d5440 ret=u32 args=v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021d5b40 ret=u32 args=v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d62c4 ret=pick:0/0/1 args=v,v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021e8b80 ret=pick:1/2/3/4 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# party slot (from 021d7f2c(battle type, c[0x11C]) below 021b9b94) not taken, alive, with ability r1 -> *out = slot, 1; else 0
fn 021d04e4 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 alt range 0 3 || any
  arg buf 0x10
  fix arg0 0x10C u8 alt range 0 5 || const 0xFF rep=6,1
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b8588 force ret=u32 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d59b4 ret=pick:0/1/2/3 args=v,v

# some free party slot whose mon can hit r1 with power >= r2 (021d0644)
fn 021d055c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  fix arg0 0x10C u8 alt range 0 5 || const 0xFF rep=6,1
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b8588 force ret=u32 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d0644 ret=pick:0/0/1 args=p,v,v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 0201c1c8 ret=u32 args=v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 021cf5b0 ret=pick:0/0/1 args=p,v,v,v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d5440 ret=u32 args=v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021d57fc ret=u32 args=v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v
  stub 021d5b40 ret=u32 args=v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d62c4 ret=pick:0/0/1 args=v,v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021d799c ret=pick:0/1/2/3/4/5 args=v,v
  stub 021e8b80 ret=pick:1/2/3/4 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# free party slot, alive (021d5b7c), whose type resists r1 (021d799c(r1, 021d57fc(m)) <= r2) -> *r3 = slot, 1; else 0
fn 021d05c4 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg s32 alt range 0 4 || any
  arg buf 0x10
  fix arg0 0x10C u8 alt range 0 5 || const 0xFF rep=6,1
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b8588 force ret=u32 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b7c ret=pick:0/1 args=v
  stub 021d57fc ret=u32 args=v
  stub 021d799c ret=pick:0/1/2/3/4 args=v,v

# mon r1 alive has a usable damaging move (021d5584, 0201c314, !021cf5b0) whose effectiveness 021d799c(0201c1c8(move), 021d57fc(r2)) >= r3
fn 021d0644 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  arg s32 alt range 0 5 || any
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d57fc ret=u32 args=v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 021cf5b0 ret=pick:0/0/1 args=p,v,v,v
  stub 0201c1c8 ret=u32 args=v
  stub 021d799c ret=pick:0/1/2/3/4/5 args=v,v
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021d5440 ret=u32 args=v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v
  stub 021d5b40 ret=u32 args=v
  stub 021d62c4 ret=pick:0/0/1 args=v,v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021e8b80 ret=pick:1/2/3/4 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# random partner side for a double battle: list = {1} + 2 / 3 when the party mon of side 021b9ab0(2) / 021b9ab0(3) is alive; more than one:
# LCG step, list[(hi * n) >> 32]; else 1
fn 021d06c8 ret=u32
  nowork
  arg buf 0x1C0
  stub 021b9ab0 force ret=pick:0/1/2 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v

# request 3, mode 1 (AI trainer step; 4-way jump table, data 021d078a-021d0792): 0: 021cfba4(c), c[0x11D] = 0, c[0x1AF] = 0, state 1.
# 1: for mon c[0x11D] (c+8 = 021b9c00(party, i), c+0xC = &c[0x124 + 4i]): dead -> 021d8434; done (021cf470) ; 021d3dc4(c, mon, party) item
# -> 021d8358(rec, item, i, 0); 021b86b8 and 021cfbf0(c, mon, i, &slot) -> 021d8394(rec, i, slot); battle type 3: side = 021d06c8 != 1 ->
# 021d83fc(rec, side), c+8 = partner (021b9ab0 / 021b9c00), c[0x1AF] = 1, rec += 4; flag 0x19 -> 021d82d0(rec, m, 021d81f8(server, pokecon,
# mon, m, c+0xF4)); 021cf524 -> state 3; 021d5448; 021cf81c(c, mon, mask) != 4 -> AI 021880bc(c+0xF0 AI, mask, 021b8d8c(..)), state 2;
# else 021cf58c; state 3. 2: !02187fb0(AI): k = 02188134(AI); k == 4 -> 021d8414(rec); else 021d82d0(rec, 021d54c0(mon, k),
# 02188140(AI)); state 3. 3: c[0x11D]++; more -> state 1; else result = &c[0x124], size 4 * (count + c[0x1AF]), c[0x1AF] = 0, return 1
fn 021d0770 ret=u32 inputs=600
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 3 || any
  fix arg0 0x11C u8 alt range 0 6 || any
  fix arg0 0x11D u8 alt range 0 6 || any
  stub 021cfba4 ret=none args=p
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d8434 ret=none args=p
  stub 021cf470 force ret=pick:0/0/1 args=p,v,p
  stub 021d3dc4 force ret=pick:0/0/5 args=p,v,v
  stub 021d8358 ret=none args=p,v,v,v
  stub 021b86b8 force ret=pick:0/1 args=v
  stub 021cfbf0 ret=pick:0/1 args=p,v,v,outm:1:7
  stub 021d8394 ret=none args=p,v,v
  stub 021b8588 force ret=pick:0/3 args=v
  stub 021d06c8 ret=pick:1/2/3 args=p
  stub 021b9ab0 force ret=pick:0/1/2 args=v
  stub 021d83fc ret=none args=p,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021e8b80 ret=u32 args=v
  stub 021d81f8 ret=u32 args=v,v,v,v,p
  stub 021d82d0 ret=none args=p,v,v
  stub 021cf524 force ret=pick:0/1 args=p,v,p
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021cf81c ret=pick:0/1/4 args=p,v,p
  stub 021d5440 ret=u32 args=v
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021880bc ret=none args=v,p,v
  stub 021cf58c force ret=none args=p,p,v
  stub 02187fb0 ret=pick:0/1 args=v
  stub 02188134 ret=pick:0/1/4 args=v
  stub 02188140 ret=u32 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021d8414 ret=none args=p
  stub 0201beb8 ret=pick:0/1 args=v,v
  stub 0201c1c8 ret=u32 args=v
  stub 0201c1d8 ret=pick:0/5/40/80/120 args=v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 021b8d38 force ret=pick:0/1/2/3/4 args=v,v,v,outm:4:0xFF
  stub 021b937c force ret=u32 args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b9b98 force ret=pick:0/2/3/4/6 args=v
  stub 021cf5b0 ret=pick:0/0/1 args=p,v,v,v
  stub 021cf89c ret=pick:0/1 args=p,v
  stub 021cf8d4 ret=pick:4/4/4/0 args=p,v,p,p
  stub 021cf9b4 ret=pick:0/3/4/4 args=p,v,p,p
  stub 021cfac4 ret=pick:0/1 args=p,v
  stub 021cfadc ret=pick:0/1 args=p,v
  stub 021cfb8c ret=pick:0/1 args=p,v
  stub 021cfbe0 ret=none args=p,v,v
  stub 021cfd50 ret=nullable:4:u32 args=p,v
  stub 021cfdcc ret=pick:0/0/0/1 args=p,v
  stub 021cfe08 ret=pick:0/0/0/1 args=p,v,v
  stub 021cfe98 ret=pick:0/0/0/1 args=p,v,v
  stub 021cffac ret=pick:0/0/0/1 args=p,v,v
  stub 021d0100 ret=pick:0/0/0/1 args=p,v,v,outm:1:7
  stub 021d0248 ret=pick:0/0/0/1 args=p,v,outm:1:7
  stub 021d0304 ret=pick:0/0/0/1 args=p,v,v,outm:1:7
  stub 021d04e4 ret=pick:0/1 args=p,v,outm:1:7
  stub 021d055c ret=pick:0/1 args=p,v,v
  stub 021d05c4 ret=pick:0/1 args=p,v,v,outm:1:7
  stub 021d0644 ret=pick:0/0/1 args=p,v,v,v
  stub 021d0980 ret=pick:0/1/2/3 args=p,outm:6:7
  stub 021d09e0 ret=none args=p,p,v,v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d5750 ret=pick:0/1 args=v,v
  stub 021d57fc ret=u32 args=v
  stub 021d5814 ret=pick:0/0/1 args=v,v
  stub 021d59b4 ret=pick:0/0x67/5 args=v,v
  stub 021d5b40 ret=pick:0x127/0/5 args=v
  stub 021d5b7c ret=pick:0/1 args=v
  stub 021d5c44 ret=pick:0/0x31FFF/0x32000/0x40000 args=v
  stub 021d62d8 ret=pick:1/2/3/4/0xA5 args=v,v
  stub 021d631c ret=pick:0/1/2 args=v,v
  stub 021d67b8 ret=pick:1/2/3/4 args=v
  stub 021d6940 ret=pick:0/1/1/1 args=v,v,v,outm:8:0xFF/0xFF/0xFF/0xFF/0xF/0/0xFF/0xFF
  stub 021d6c24 ret=pick:0/1 args=v
  stub 021d799c ret=pick:0/1/2/3/4/5 args=v,v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021e8afc ret=pick:1/2/3 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021efc1c ret=pick:0/1 args=v,v,v,v
  stub 021efce4 ret=pick:0/1 args=v,v

# sort the slot list r1[0..n) (n <= 6: the scores are a 6-entry u16 stack array; n >= 0x100 never ends, the u8 counters wrap) by the best move power against r3 (descending; power 0201c1d8 (< 10 -> 60) scaled by the effectiveness
# 021d799c(0201c1c8(move), 021d57fc(target)) through a 6-way jump table, data 021d0a7a-021d0a86: 0 -> 0, 1 -> x/4, 2 -> x/2, 3 -> x, 4 ->
# 2x, 5 -> 4x (as u16)), list entries move with their scores
fn 021d09e0 ret=none
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg u32 range 0 6
  arg u32 any
  stub 021d57fc ret=u32 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d54c0 ret=u16 args=v,v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 0201c1c8 ret=u32 args=v
  stub 0201c1d8 ret=pick:0/5/40/80/120 args=v
  stub 021d799c ret=pick:0/1/2/3/4/5/6 args=v,v

# view block setup through three veneers: 021d0b3c(r3, c+0x118 party, r2, r1), 021d0b4c(r3, c[0x11C]), 021d0b5c([sp], r3)
fn 021d0b10 ret=none
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d0b3c ret=none args=v,v,v,v
  stub 021d0b4c ret=none args=v,v
  stub 021d0b5c ret=none args=v,v

# forced switch answers: start = 021b99c0(server, c[0x1A6]), n = 021b9b94(party); for k < c[0x1B3]: 021b8e28(server, c[0x1B5 + k], &a, &b);
# the next alive slot i >= start -> 021d8394(&c[0x124 + 4 * count++], a, i), start = i + 1; result = &c[0x124], size 4 * count
fn 021d0bb0 ret=none
  nowork
  arg buf 0x1C0
  fix arg0 0x1B3 u8 alt range 0 4 || range 0 8
  stub 021b9b94 force ret=pick:0/2/4/6 args=v
  stub 021b99c0 force ret=pick:0/1/2 args=v,v
  stub 021b8e28 force ret=none args=v,v,outm:1:0xFF,outm:1:0xFF
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d8394 ret=none args=p,v,v

# the switch requests of this client: n = (u8) 021eec20(adapter, &list) (adapter smoke port: force; the list is arg 2); out[k++] = list[i]
# for each i < n with 021b8de4(server, list[i]) == c[0x1A6]; returns k
fn 021d0c78 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x10
  fix arg0 0x1A6 u8 range 0 3
  stub 021eec20 force ret=pick:0/1/2/3/4/6 args=v,outp:2:0:1
  stub 021b8de4 ret=pick:0/1/2/3 args=v,v

# request 7, mode 0 (yes / no): 0: 021eaf18(c+0x58, 3, 5), 021eaf18(c+0x7C, 3, 1), 021eabc4(view, c+0x58, c+0x7C, 0), state 1. 1:
# 021eac54(view, &ans) -> c[0x1AC] = ans != 0, result = &c[0x1AC], size 1, return 1
fn 021d0cc4 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 1 || any
  stub 021eaf18 ret=none args=p,v,v
  stub 021eabc4 ret=none args=v,p,p,v
  stub 021eac54 ret=pick:0/1 args=v,outm:4:1/0/0/0

# request 5, mode 0: thunk to 021d0e7c(c, state, 2)
fn 021d0d38 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  stub 021d0e7c ret=pick:0/1 args=p,p,v
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b86c0 force ret=pick:0/1 args=v
  stub 021b8de4 ret=pick:0/1/2/3 args=v,v
  stub 021b8e28 force ret=none args=v,v,outm:1:0xFF,outm:1:0xFF
  stub 021b99c0 force ret=pick:0/1/2 args=v,v
  stub 021b9ab0 force ret=u8 args=v
  stub 021b9b94 force ret=pick:0/1/2/3/6 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021ce068 force ret=none args=p
  stub 021ce08c force ret=pick:0/1 args=p
  stub 021ce0e0 force ret=none args=p
  stub 021d0980 ret=pick:0/1/1/2/3 args=p,outm:6:7
  stub 021d0b3c ret=none args=v,v,v,v
  stub 021d0b4c ret=none args=v,v
  stub 021d0b5c ret=none args=v,v
  stub 021d0b6c ret=none args=p,p
  stub 021d0bb0 ret=none args=p
  stub 021d0c78 ret=pick:0/1/2/3 args=p,p
  stub 021d102c ret=pick:0/1 args=p
  stub 021d1058 ret=pick:0/4 args=p,p
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5b7c ret=pick:0/1 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021d8394 ret=none args=p,v,v
  stub 021d83c4 ret=none args=p
  stub 021d83fc ret=none args=p,v
  stub 021d8434 ret=none args=p
  stub 021e9a78 ret=none args=v,p,v,v,p
  stub 021e9b14 ret=none args=v
  stub 021e9b20 ret=pick:0/1 args=v
  stub 021ea7fc ret=none args=v
  stub 021ea814 ret=pick:0/1 args=v
  stub 021eec20 force ret=pick:0/1/2/3/4/6 args=v,outp:3:0:1

# request 6, mode 0: thunk to 021d0e7c(c, state, 1)
fn 021d0d44 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  stub 021d0e7c ret=pick:0/1 args=p,p,v
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b86c0 force ret=pick:0/1 args=v
  stub 021b8de4 ret=pick:0/1/2/3 args=v,v
  stub 021b8e28 force ret=none args=v,v,outm:1:0xFF,outm:1:0xFF
  stub 021b99c0 force ret=pick:0/1/2 args=v,v
  stub 021b9ab0 force ret=u8 args=v
  stub 021b9b94 force ret=pick:0/1/2/3/6 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021ce068 force ret=none args=p
  stub 021ce08c force ret=pick:0/1 args=p
  stub 021ce0e0 force ret=none args=p
  stub 021d0980 ret=pick:0/1/1/2/3 args=p,outm:6:7
  stub 021d0b3c ret=none args=v,v,v,v
  stub 021d0b4c ret=none args=v,v
  stub 021d0b5c ret=none args=v,v
  stub 021d0b6c ret=none args=p,p
  stub 021d0bb0 ret=none args=p
  stub 021d0c78 ret=pick:0/1/2/3 args=p,p
  stub 021d102c ret=pick:0/1 args=p
  stub 021d1058 ret=pick:0/4 args=p,p
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5b7c ret=pick:0/1 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021d8394 ret=none args=p,v,v
  stub 021d83c4 ret=none args=p
  stub 021d83fc ret=none args=p,v
  stub 021d8434 ret=none args=p
  stub 021e9a78 ret=none args=v,p,v,v,p
  stub 021e9b14 ret=none args=v
  stub 021e9b20 ret=pick:0/1 args=v
  stub 021ea7fc ret=none args=v
  stub 021ea814 ret=pick:0/1 args=v
  stub 021eec20 force ret=pick:0/1/2/3/4/6 args=v,outp:3:0:1

# requests 5 / 6, mode 1 (AI switch-in; r1 is unused, arg 1 holds the adapter list of 021d0c78): n = 021d0c78(c, c+0x1B5) -> c[0x1B3]; none -> 021d8434(c+0x124), size 4; k = 021d0980(c, list):
# k == 0 -> 021d102c(c) and (r = 021d1058(c, c+0x124)) -> size r, else 021d83c4(c+0x124), size 4; else 021cfd50(c, c[0x1B5]) -> 021d09e0(c,
# list, k, target); n > k: battle type 3 and (r = 021d1058) -> size r, return 1, else n = k; for i < n: 021b8e28(server, c[0x1B5 + i], &a,
# &b), 021d8394(&c[0x124 + 4i], b, list[i]); size 4n. Returns 1.
fn 021d0d50 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  stub 021d0c78 ret=pick:0/1/2/3 args=p,p
  stub 021eec20 force ret=pick:0/1/2/3/4/6 args=v,outp:1:0:1
  stub 021d0980 ret=pick:0/1/2/3 args=p,outm:6:7
  stub 021cfd50 ret=nullable:3:u32 args=p,v
  stub 021d09e0 ret=none args=p,p,v,v
  stub 021b8588 force ret=pick:0/3 args=v
  stub 021d1058 ret=pick:0/4 args=p,p
  stub 021b8e28 force ret=none args=v,v,outm:1:0xFF,outm:1:0xFF
  stub 021d8394 ret=none args=p,v,v
  stub 021d102c ret=pick:0/1 args=p
  stub 021d83c4 ret=none args=p
  stub 021d8434 ret=none args=p
  stub 0201c1c8 ret=u32 args=v
  stub 0201c1d8 ret=pick:0/5/40/80/120 args=v
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 021b8d38 force ret=pick:0/1/2/3/4 args=v,v,v,outm:4:0xFF
  stub 021b8de4 ret=pick:0/1/2/3 args=v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021b9ab0 force ret=u8 args=v
  stub 021b9b94 force ret=pick:0/1/2/3/6 args=v
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021d5584 ret=pick:0/1/1 args=v,v
  stub 021d57fc ret=u32 args=v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d5b7c ret=pick:0/1 args=v
  stub 021d799c ret=pick:0/1/2/3/4/5/6 args=v,v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021d83fc ret=none args=p,v

# requests 5 / 6 / 4, mode 2: thunk to 021ce1b0 (the replay step of request 3)
fn 021d0e74 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  stub 021ce1b0 force ret=pick:0/1 args=p,p

# requests 5 / 6 / 4, mode 0 step (r2 = kind 1 / 2 / 0; r3 is unused, arg 3 holds the adapter list of 021d0c78; 7-way jump table, data 021d0e9a-021d0ea8): 0: n = 021d0c78(c, c+0x1B5) -> c[0x1B3];
# none -> 021d8434(c+0x124), size 4, state 4; k = 021d0980(c, list): none -> (021d102c and 021d1058) or 021d83c4, state 4; single mon
# (021d102c, k == 1, 021b9c00(party, 0) dead) -> 021d8394(c+0x124, 0, list[0]), size 4, state 4; else 021b8e28(server, c[0x1B5], &a, &b),
# 021d0b10(c, kind, min(n, k), c+0x184, c+0x190), 021e9a78(view, c+0x184, a, 0, c+0x190), 021ce068(c), state 1. 1: 021ce08c(c) ?
# (021e9b14(view), state 2) : 021e9b20(view) -> 021d0b6c(c, c+0x190), state 3. 2: 021e9b20 -> 021d0bb0(c), state 3. 3: 021ce0e0(c),
# state 4. 4: 021b86c0(server) ? (c[0x1A9] = 1, 021ea7fc(view), state 5) : state 6. 5: 021ea814(view) -> state 6. 6: return 1.
fn 021d0e7c ret=u32 inputs=400
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg u32 range 0 2
  arg buf 0x10
  fix arg1 0 u32 alt range 0 6 || any
  fix arg0 0x196 u8 range 0 8
  fix arg0 0x1B3 u8 alt range 0 4 || range 0 8
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  stub 021d0c78 ret=pick:0/1/2/3 args=p,p
  stub 021eec20 force ret=pick:0/1/2/3/4/6 args=v,outp:3:0:1
  stub 021d8434 ret=none args=p
  stub 021d0980 ret=pick:0/1/1/2/3 args=p,outm:6:7
  stub 021d102c ret=pick:0/1 args=p
  stub 021d1058 ret=pick:0/4 args=p,p
  stub 021d83c4 ret=none args=p
  stub 021b9c00 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d8394 ret=none args=p,v,v
  stub 021b8e28 force ret=none args=v,v,outm:1:0xFF,outm:1:0xFF
  stub 021e9a78 ret=none args=v,p,v,v,p
  stub 021ce068 force ret=none args=p
  stub 021ce08c force ret=pick:0/1 args=p
  stub 021e9b14 ret=none args=v
  stub 021e9b20 ret=pick:0/1 args=v
  stub 021d0b6c ret=none args=p,p
  stub 021d0bb0 ret=none args=p
  stub 021ce0e0 force ret=none args=p
  stub 021b86c0 force ret=pick:0/1 args=v
  stub 021ea7fc ret=none args=v
  stub 021ea814 ret=pick:0/1 args=v
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b8de4 ret=pick:0/1/2/3 args=v,v
  stub 021b99c0 force ret=pick:0/1/2 args=v,v
  stub 021b9ab0 force ret=u8 args=v
  stub 021b9b94 force ret=pick:0/1/2/3/6 args=v
  stub 021d0b3c ret=none args=v,v,v,v
  stub 021d0b4c ret=none args=v,v
  stub 021d0b5c ret=none args=v,v
  stub 021d5b7c ret=pick:0/1 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021d83fc ret=none args=p,v
