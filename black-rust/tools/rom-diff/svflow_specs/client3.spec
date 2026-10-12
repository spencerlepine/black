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
  stub 021b9b94 ret=pick:0/1/2/3/6 args=v
  stub 021b8588 ret=pick:0/1/2/3 args=v
  stub 021d7f2c ret=pick:0/1/2 args=v,v
  stub 021b9c00 ret=u32 args=v,v
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
  stub 021b8588 ret=pick:0/3 args=v
  stub 021b9c00 ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v

# two checks (sides 2 and 3: 021b9ab0 -> party slot -> 021d5b68): the first failing one is reported to r1 through 021d83fc(r1, side) and
# returns 4; both passing returns 0
fn 021d1058 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x40
  stub 021b9ab0 ret=u8 args=v
  stub 021b9c00 ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d83fc ret=none args=p,v
