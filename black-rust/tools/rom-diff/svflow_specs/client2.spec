# svflow spec: client2 (btl_client.c part 2, 0x021CDDDC-0x021CF5B0: request step functions and their small helpers; B13-spec1).
# Harness-job owned; porting workers may not edit. Grammar: svflow_specs/README.md. Area split: doc comment of client/mod.rs.
#
# Same object as client1.spec (`arg buf 0x1C0`, `nowork`). Extra fields used here: +0x0C result pointer (set by 021ce2e8), +0x0D0 /
# +0x0D4 pending callback and its state (021ce150), +0x0E4 BGM-fade flag (u16), +0x120 byte, +0x130 3 bytes (021cf41c/430/43c, summed by
# 021cf448 over c[0x11D] entries), +0x19C u16 (HP of the active mon, 021cdecc), +0x1A0 3 x u16 (021cf360/3b8), +0x1A6 client id, +0x1AA flag,
# +0x1AD flag (set by 021ce08c once 021f8524(1) answers).
#
# Closure stubs (core4/core5 convention): a block lists the stubs of its callees AND of the callees of all in-area descendants plus the
# resource lines the descendants need, so the functions can be ported in any order without a spec edit. Not specced in this batch: the
# step functions 021cdddc 021cde8c 021cdf58 021ce044 021ce040 021ce170/1b0 (jump tables, `.byte` blocks, `LAB_` entries), 021ce160
# (indirect call through c+0xD0), 021ce4d0 021ce7d4 021ce8f0 021cea94 021ceeb8 021ceffc 021cf078 021cf220 021cf2f0 (step functions).
area client2

# mon = 021cdf1c(c); if mon: c.u16[0x19C] = BattleMon_Get(mon, 0xD)
fn 021cdecc ret=none
  nowork
  arg buf 0x1C0
  stub 021cdf1c ret=nullable:3:u32 args=p
  stub 021d59b4 ret=u16 args=v,v
  stub 021b8588 ret=pick:0/0/0/3 args=v
  stub 021b86c0 ret=pick:0/0/0/1 args=v
  stub 021b9064 ret=u32 args=v,v,v
  stub 021b9920 ret=u32 args=v,v,v

# percentage: 0 without a mon or when HP >= c.u16[0x19C], else 100 * (c.u16[0x19C] - HP) / HP (HP = BattleMon_Get(mon, 0xD))
fn 021cdee8 ret=u32
  nowork
  arg buf 0x1C0
  stub 021cdf1c ret=nullable:3:u32 args=p
  stub 021d59b4 ret=u16 args=v,v
  stub 021b8588 ret=pick:0/0/0/3 args=v
  stub 021b86c0 ret=pick:0/0/0/1 args=v
  stub 021b9064 ret=u32 args=v,v,v
  stub 021b9920 ret=u32 args=v,v,v

# the active mon of this client: 0 when the battle type (021b8588) or the multi flag (021b86c0) is set, else
# 021b9920(pokecon, 021b9064(server, client id, 0) & 0xFF, 0)
fn 021cdf1c ret=u32
  nowork
  arg buf 0x1C0
  stub 021b8588 ret=pick:0/0/0/3 args=v
  stub 021b86c0 ret=pick:0/0/0/1 args=v
  stub 021b9064 ret=u32 args=v,v,v
  stub 021b9920 ret=u32 args=v,v,v

# random result 1..3 weighted by the mode r0 (0: <30 -> 3, <60 -> 2, else 1; 1: >=50 -> 3 else 2; 2: >=40 -> 1 else 3; 3: >=40 -> 1 else 2);
# 0203f0a8 (rand) * 100 >> 32 through the real 64-bit multiply 0209c054
fn 021cdfe4 ret=u8
  nowork
  arg u32 alt range 0 4 || any
  stub 0203f0a8 ret=u32 args=

# if the BGM-fade flag c.u16[0xE4] is set: 021f8504(1, 1, 1), c[0x1AD] = 0
fn 021ce068 ret=none
  nowork
  arg buf 0x1C0
  fix arg0 0xE4 u16 alt const 0 || any
  stub 021f8504 ret=none args=v,v,v

# with the flag set: c[0x1AD] becomes 1 once 021f8524(1) answers nonzero; returns c[0x1AD]; without the flag 0
fn 021ce08c ret=u8
  nowork
  arg buf 0x1C0
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  stub 021f8524 ret=pick:0/1 args=v

# 021ce08c(c) != 0: (r1 != 0: 021ce150(c, r1)); returns 1, else 0
fn 021ce0bc ret=u32
  nowork
  arg buf 0x1C0
  arg u32 alt const 0 || any
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  stub 021ce08c ret=pick:0/1 args=p
  stub 021ce150 ret=none args=p,v
  stub 021f8524 ret=pick:0/1 args=v

# if the BGM-fade flag is set: 021f8504(1, 0, 0)
fn 021ce0e0 ret=none
  nowork
  arg buf 0x1C0
  fix arg0 0xE4 u16 alt const 0 || any
  stub 021f8504 ret=none args=v,v,v

# message "<mon name / client id> sent out": battle type == 3 -> id 0x46 with the client id byte c[0x1A6], else id 0x45 with
# 021d5440(r1); then 021eaf18(r2, 1, id), 021eaf50(r2, value), 021eaf78(r2, 0)
fn 021ce0f8 ret=none
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  stub 021b8588 ret=pick:0/3 args=v
  stub 021eaf18 ret=none args=v,v,v
  stub 021d5440 ret=u32 args=v
  stub 021eaf50 ret=none args=v,v
  stub 021eaf78 ret=none args=v,v

# pending callback: c+0xD0 = r1, c+0xD4 = 0
fn 021ce150 ret=none
  nowork
  arg buf 0x1C0
  arg u32 any

# result block: c+0xC = &c[0x124], 021d8434(&c[0x124]), c+0xD8 = c+0xC, c+0xDC = 4
fn 021ce2e8 ret=none
  nowork
  arg buf 0x1C0
  stub 021d8434 ret=none args=p

# 021ea614(view), c[0x120] = 1
fn 021ce3b4 ret=none
  nowork
  arg buf 0x1C0
  stub 021ea614 ret=none args=v

# c.u16[0x1A0 + 2i] = 0 for i in 0..3
fn 021cf360 ret=none
  nowork
  arg buf 0x1C0

# clear slot r1 (0..2) of c.u16[0x1A0..]: only when c[0x1AA] == 0 and 021b8ff4(server) == c[0x1A6] and the slot is set;
# then 021b90bc(server, c[0x1A6])
fn 021cf3b8 ret=none
  nowork
  arg buf 0x1C0
  arg u32 range 0 2
  fix arg0 0x1AA u8 alt const 0 || any
  fix arg0 0x1A6 u8 range 0 3
  fix arg0 0x1A0 u16 alt const 0 || any rep=3,2
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021b90bc ret=none args=v,v

# if c[0x1AA] == 0: 021cf3b8(c, i) for i in 0..3
fn 021cf3f8 ret=none
  nowork
  arg buf 0x1C0
  fix arg0 0x1AA u8 alt const 0 || any
  fix arg0 0x1A6 u8 range 0 3
  fix arg0 0x1A0 u16 alt const 0 || any rep=3,2
  stub 021cf3b8 ret=none args=p,v
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021b90bc ret=none args=v,v

# MI_CpuFill8(c + 0x130, 0, 3) (tail call through a literal; the helper runs as real code on both sides)
fn 021cf41c ret=none
  nowork
  arg buf 0x1C0

# c[0x130 + r1] = r2
fn 021cf430 ret=none
  nowork
  arg buf 0x1C0
  arg u32 range 0 0x20
  arg u32 any

# c[0x130 + r1]
fn 021cf43c ret=u8
  nowork
  arg buf 0x1C0
  arg u32 range 0 0x20

# sum (low byte) of c[0x130 .. 0x130 + c[0x11D])
fn 021cf448 ret=u32
  nowork
  arg buf 0x1C0
  fix arg0 0x11D u8 alt const 0 || range 1 0x40

# ---------------------------------------------------------------------------------------------------------------------------------- B13-spec2
# B13-spec2 specs every function the B13-spec1 header above lists as "not specced in this batch" (list in client/mod.rs).
# The request steps and pending callbacks of this range. Shapes: request step `u32 step(c, u32 *state)` (arg1 = `arg buf 0x10`, the state
# word at +0, returns 1 when the request is answered); pending callback (set by 021ce150 at c+0xD0, run by 021ce160 with &c+0xD4)
# `u32 cb(c, u32 *state)` (always returns 0; it chains to the next callback with 021ce150). Callback addresses (Thumb): 0x021CE309 (start
# of action selection), 0x021CE4D1 (selection loop), 0x021CE3C9 (finish: write the answers), 0x021CF221 (next mon), 0x021CF2F1
# (end), 0x021CE7D5, 0x021CEDC1, 0x021CEA95, 0x021CEEB9, 0x021CF079, 0x021CED4D (menu pages). An indirect call through c+0xD0 goes to
# `force` stubs at those addresses (the port dispatches the pointer through `env.call`).
# Pointer fields: c+0x08 the mon being asked for (u32, only handed to stubs), c+0x0C pointer to its answer record (c+0x124 + 4 * index;
# 021cea94 dereferences it: fixed to c+0x124), c+0x58 / c+0x7C message buffers (only their address is passed). Index bytes: c+0x11C
# count, c+0x11D current, c+0x11E (s8) / c+0x11F, c+0x1AA / c+0x1AB, c+0x1AF; c+0x184 / c+0x190 per-request blocks handed to view calls.
# Veneers (Thumb `bx pc` + ARM `ldr ip; bx ip` to 0x0689xxxx): 021ce7c4, 021cee98, 021ceea8 are stubbed at the Thumb entry.
# BattleMon accessors (021d5440-021d7f04) are stubs as in the B13-spec1 blocks (`battle_mon` verifies them).

# request 1, mode 0 (step): 0: 021e90a0(view, 021b86b8(server) == 4), state 1. 1: 021e90d0(view) -> c.u16[0xE4] = 021ba5c4(server),
# c.u16[0xE6] = 021ba5d0(server); either set: 021f84e8(c.u16[0xE6]), and with 0xE6 set 021f8504(0, 1, 1); 021cdecc(c); idle step
# (021cd9e0) -> 021eaea8(view, (u16) 021d3fa8(cursor)); return 1. Other states 0.
fn 021cdddc ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b86b8 ret=pick:0/4 args=v
  stub 021e90a0 ret=none args=v,v
  stub 021e90d0 ret=pick:0/1 args=v
  stub 021ba5c4 ret=nullable:2:u16 args=v
  stub 021ba5d0 ret=nullable:2:u16 args=v
  stub 021f84e8 ret=none args=v
  stub 021f8504 ret=none args=v,v,v
  stub 021cdecc ret=none args=p
  stub 021cd9e0 force ret=pick:0/1 args=p
  stub 021d3fa8 ret=u16 args=p
  stub 021eaea8 ret=none args=v,v
  stub 021b8588 ret=pick:0/0/0/3 args=v
  stub 021b86c0 ret=pick:0/0/0/1 args=v
  stub 021b9064 ret=u32 args=v,v,v
  stub 021b9920 ret=u32 args=v,v,v
  stub 021cdf1c ret=nullable:3:u32 args=p
  stub 021d59b4 ret=u16 args=v,v

# request 1, mode 2 (replay step): view set: r = 021cdddc(c, state); r and 021ee688(reader, client id) -> 021d3f54(cursor); return r.
# No view: 021ee688(reader, client id), return 1.
fn 021cde8c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg0 0x54 u32 alt const 0 || any
  fix arg1 0 u32 alt range 0 1 || any
  stub 021cdddc ret=pick:0/1 args=p,p
  stub 021ee688 ret=pick:0/1 args=v,v
  stub 021d3f54 ret=none args=p
  stub 021b8588 ret=pick:0/0/0/3 args=v
  stub 021b86b8 ret=pick:0/4 args=v
  stub 021b86c0 ret=pick:0/0/0/1 args=v
  stub 021b9064 ret=u32 args=v,v,v
  stub 021b9920 ret=u32 args=v,v,v
  stub 021ba5c4 ret=nullable:2:u16 args=v
  stub 021ba5d0 ret=nullable:2:u16 args=v
  stub 021cd9e0 force ret=pick:0/1 args=p
  stub 021cdecc ret=none args=p
  stub 021cdf1c ret=nullable:3:u32 args=p
  stub 021d3fa8 ret=u16 args=p
  stub 021d59b4 ret=u16 args=v,v
  stub 021e90a0 ret=none args=v,v
  stub 021e90d0 ret=pick:0/1 args=v
  stub 021eaea8 ret=none args=v,v
  stub 021f84e8 ret=none args=v
  stub 021f8504 ret=none args=v,v,v

# request 2, mode 0 (step; 5-way jump table, data 021cdf70-021cdf7a): 0: 021eaa00(view, c[0xC4]), 021ce068(c), state 1. 1: 021ce08c(c) ?
# (021eaa5c(view), state 2) : 021eaa34(view, &b) -> c[0xC4] = b, state 3. 2: c[0xC4] = 021cdfe4(c[0xC4]), state 3. 3: 021ce0e0(c),
# state 4. 4: result = &c[0xC4], size 1, return 1.
fn 021cdf58 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 4 || any
  fix arg0 0xC4 u8 alt range 0 4 || any
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  stub 021eaa00 ret=none args=v,v
  stub 021ce068 ret=none args=p
  stub 021ce08c ret=pick:0/1 args=p
  stub 021eaa5c ret=none args=v
  stub 021eaa34 ret=pick:0/1 args=v,out:4
  stub 021cdfe4 ret=u8 args=v
  stub 021ce0e0 ret=none args=p
  stub 0203f0a8 ret=u32 args=
  stub 021f8504 ret=none args=v,v,v
  stub 021f8524 ret=pick:0/1 args=v

# request 2, mode 2: return 1
fn 021ce040 ret=u32
  nowork

# request 2, mode 1: c[0xC4] = 021cdfe4(c[0xC4]); result = &c[0xC4], size 1; return 1
fn 021ce044 ret=u32
  nowork
  arg buf 0x1C0
  fix arg0 0xC4 u8 alt range 0 4 || any
  stub 021cdfe4 ret=u8 args=v
  stub 0203f0a8 ret=u32 args=

# run the pending callback: (c->D0)(c, &c->D4), returns its result
fn 021ce160 ret=u32
  nowork
  arg buf 0x1C0
  fix arg0 0xD0 u32 oneof 0x021CE309 0x021CE4D1 0x021CE3C9 0x021CF221 0x021CF2F1 0x021CE7D5 0x021CEDC1 0x021CEA95 0x021CEEB9 0x021CF079 0x021CED4D
  stub 021ce308 force ret=pick:0/1 args=p,p
  stub 021ce4d0 force ret=pick:0/1 args=p,p
  stub 021ce3c8 force ret=pick:0/1 args=p,p
  stub 021cf220 force ret=pick:0/1 args=p,p
  stub 021cf2f0 force ret=pick:0/1 args=p,p
  stub 021ce7d4 force ret=pick:0/1 args=p,p
  stub 021cedc0 force ret=pick:0/1 args=p,p
  stub 021cea94 force ret=pick:0/1 args=p,p
  stub 021ceeb8 force ret=pick:0/1 args=p,p
  stub 021cf078 force ret=pick:0/1 args=p,p
  stub 021ced4c force ret=pick:0/1 args=p,p

# request 3, mode 0 (step; the action selection): 0: 021ce150(c, 0x021CE309), 021ce068(c), state 1. 1: 021ce08c(c) (result unused),
# 021ce160(c) -> 1. Other states 0.
fn 021ce170 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 1 || any
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  fix arg0 0xD0 u32 oneof 0x021CE309 0x021CE4D1 0x021CE3C9 0x021CF221
  stub 021ce150 ret=none args=p,v
  stub 021ce068 ret=none args=p
  stub 021ce08c ret=pick:0/1 args=p
  stub 021ce160 ret=pick:0/1 args=p
  stub 021ce308 force ret=pick:0/1 args=p,p
  stub 021ce4d0 force ret=pick:0/1 args=p,p
  stub 021ce3c8 force ret=pick:0/1 args=p,p
  stub 021cf220 force ret=pick:0/1 args=p,p
  stub 021ce7d4 force ret=pick:0/1 args=p,p
  stub 021cea94 force ret=pick:0/1 args=p,p
  stub 021ced4c force ret=pick:0/1 args=p,p
  stub 021cedc0 force ret=pick:0/1 args=p,p
  stub 021ceeb8 force ret=pick:0/1 args=p,p
  stub 021cf078 force ret=pick:0/1 args=p,p
  stub 021cf2f0 force ret=pick:0/1 args=p,p
  stub 021f8504 ret=none args=v,v,v
  stub 021f8524 ret=pick:0/1 args=v

# request 3, mode 2 (replay step): 0: flag bit 5 -> 021ce2e8(c), 1. rec = 021ee6b8(reader, client id, &count, &more) (arg2 = the record
# words, kind = rec[0] & 0xF); more -> 021d3f54(cursor); result = rec, size = 4 * count; kind 8: 021d3fd0(cursor), view ? (021eaf18(c+0x58,
# 1, 0xB7), 021ea614(view, c+0x58), state 1) : (021ce2e8, flags |= 0x10, 1); kind 9: flags |= 0x20, 021d3fd0, view ? (msg 0xB8, state 2) :
# (021ce2e8, 1); else 1. 1: 021ea710(view) -> 021ce2e8, flags |= 0x10, 1. 2: 021ea710(view) -> 021ce2e8, 1.
fn 021ce1b0 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x10
  fix arg1 0 u32 alt range 0 2 || any
  fix arg0 0x54 u32 alt const 0 || any
  fix arg2 0 u32 oneof 8 9 1 0x18 bits=0:5 rep=4,4
  stub 021ce2e8 ret=none args=p
  stub 021ee6b8 ret=ptr:arg2:0:4 args=v,v,outm:1:0x3F,outm:1:1
  stub 021d3f54 ret=none args=p
  stub 021d3fd0 ret=none args=p
  stub 021eaf18 ret=none args=p,v,v
  stub 021ea614 ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v
  stub 021d8434 ret=none args=p

# callback 0x021CE309 (start of the selection): c[0x11D] = 0, c[0x11E] = 0xFF, c[0x11F] = 0, flags &= ~4; 021d0b10(c, 0, c[0x11C], c+0x184,
# c+0x190); 021cf41c(c); 021cf360(c); 021d8434(c+0x124 + 4i) for i < 3; first i < c[0x11C] whose mon (021b990c(pokecon, id, i)) is not
# done (021cf470(c, mon, 0) == 0) -> c[0x11F] = i; 021ce150(c, 0x021CE4D1); returns 0
fn 021ce308 ret=u32
  nowork
  arg buf 0x1C0
  fix arg0 0x11C u8 alt range 0 6 || any
  stub 021d0b10 force ret=none args=p,v,v,p,p
  stub 021cf41c ret=none args=p
  stub 021cf360 ret=none args=p
  stub 021d8434 ret=none args=p
  stub 021b990c ret=u32 args=v,v,v
  stub 021cf470 ret=pick:0/1/1 args=p,v,v
  stub 021ce150 ret=none args=p,v
  stub 021d5778 ret=none args=v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d5bdc ret=pick:0/0/1 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67a8 ret=u32 args=v
  stub 021d67c4 ret=u32 args=v
  stub 021d82d0 ret=none args=p,v,v
  stub 021d8444 ret=none args=p
  stub 021e8b80 ret=u32 args=v

# callback 0x021CE3C9 (finish): 0: 021eae44(view), state 1. 1: 021eae54(view) -> for each remaining index c[0x11D] < c[0x11C]: c+8 = mon
# (021b990c), c+0xC = &c[0x124 + 4 * c[0x11D]], 021d8434(c+0xC); not done (021cf470(c, mon, c+0xC) == 0): r = 021cf81c(c, mon, &tmp),
# r < 4 ? 021d82d0(c+0xC, n = 021d54c0(mon, r), 021d81f8(server, pokecon, mon, n, c+0x38)) : 021cf58c(c+0xC, c, mon); c[0x11D]++; then
# state 2. 2: result = &c[0x124], size = 4 * c[0x11C], 021ce150(c, 0x021CF2F1). Returns 0.
fn 021ce3c8 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 2 || any
  fix arg0 0x11C u8 alt range 0 6 || range 0 0x24
  fix arg0 0x11D u8 alt range 0 6 || any
  stub 021eae44 ret=none args=v
  stub 021eae54 ret=pick:0/1 args=v
  stub 021b990c ret=u32 args=v,v,v
  stub 021d8434 ret=none args=p
  stub 021cf470 ret=pick:0/1 args=p,v,p
  stub 021cf81c force ret=pick:0/1/2/3/4 args=p,v,p
  stub 021d54c0 ret=u16 args=v,v
  stub 021d81f8 ret=u32 args=v,v,v,v,p
  stub 021d82d0 ret=none args=p,v,v
  stub 021cf58c ret=none args=p,p,v
  stub 021ce150 ret=none args=p,v
  stub 021d5778 ret=none args=v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d5bdc ret=pick:0/0/1 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67a8 ret=u32 args=v
  stub 021d67c4 ret=u32 args=v
  stub 021d8444 ret=none args=p
  stub 021e8b80 ret=u32 args=v

# callback 0x021CE4D1 (selection loop; 9-way jump table on the state, data 021ce4ea-021ce4fc; inner 5-way table on the menu answer
# 021e97a4(view), data 021ce5f4-021ce5fe). 0: c+8 = mon c[0x11D], c+0xC = its answer record, c[0x1AF] = 0, 021d8434; done (021cf470) ->
# 021ce150(c, 0x021CF221); 021ce8f0(c, c+0x19E) == 1 -> 021ce150(c, 0x021CE7D5); else state 1. 1: (s8) c[0x11E] != c[0x11D] or c[0x120]:
# 021ce0f8(c, mon, c+0x58), 021ea65c(view, c+0x58), c[0x120] = 0, c[0x11E] = c[0x11D], state 2; else state 3. 2: 021ea710(view) ? state 3
# : return. 3: 021e9760(view, mon, c[0x11D] > c[0x11F], c+0xC), state 4. 4: 021ce0bc(c, 0x021CE3C9) ? 021e979c(view) : menu answer
# 1 (fight) -> 021cf430 + 021ce150(c, 0x021CEA95); 2 (bag) -> flag 0x21 / c[0x1AA] == 1 + !021ba1ec -> 021e9964 + state 5, else 0x021CEEB9; 3
# (pokemon) -> flag 0x21 ? (021e9964, state 6) : (021cf430, 0x021CEDC1); 4 (run / back) -> c[0x11D] == c[0x11F] ? (flag 0x21 and
# 021b86e4 != 2 -> state 6, else 0x021CF079) : step back over the done mons (021cf470) and undo their answers (021cf43c, 021d8454,
# veneer 021ce7c4, 021cf3b8), 021ce150(c, 0x021CE4D1). 5: 021e9974(view) -> state 3. 6: 021e9954(view), state 7. 7: 021e9974(view) ->
# message 0xC5 (021eaf18 / 021d5440 / 021eaf50, 021ce3b4), state 8. 8: 021ea710(view) -> state 1. Returns 0.
fn 021ce4d0 ret=u32 inputs=1500
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 8 || const 4 || const 4 || any
  fix arg0 0x11C u8 alt range 0 6 || any
  fix arg0 0x11D u8 range 0 5
  fix arg0 0x11E u8 alt range 0 5 || const 0xFF
  fix arg0 0x11F u8 range 0 5
  fix arg0 0x120 u8 alt const 0 || any
  fix arg0 0x1AA u8 alt range 0 1 || any
  fix arg0 0x1A6 u8 range 0 3
  fix arg0 0x1A7 u8 alt range 0 2 || any
  fix arg0 0x19A u16 any
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  fix arg0 0x1A0 u16 alt const 0 || any rep=3,2
  stub 021b990c ret=u32 args=v,v,v
  stub 021d8434 ret=none args=p
  stub 021cf470 ret=pick:0/1 args=p,v,p
  stub 021ce150 ret=none args=p,v
  stub 021ce8f0 ret=pick:0/1 args=p,p
  stub 021ce0f8 ret=none args=p,v,p
  stub 021ea65c ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v
  stub 021e9760 ret=none args=v,v,v,p
  stub 021ce0bc ret=pick:0/1 args=p,v
  stub 021e979c ret=none args=v
  stub 021e97a4 ret=pick:0/1/2/3/4/5 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021e9964 ret=none args=v
  stub 021cf430 ret=none args=p,v,v
  stub 021ba1ec ret=pick:0/1 args=v
  stub 021b86e4 ret=pick:0/1/2 args=v
  stub 021cf43c ret=u8 args=p,v
  stub 021d8454 ret=pick:0/2/3 args=p
  stub 021ce7c4 ret=none args=p
  stub 021cf3b8 ret=none args=p,v
  stub 021e9974 ret=pick:0/1 args=v
  stub 021e9954 ret=none args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021d5440 ret=u32 args=v
  stub 021eaf50 ret=none args=p,v
  stub 021ce3b4 ret=none args=p
  stub 0202a298 ret=pick:0/1 args=v,v,v
  stub 021b8588 ret=pick:0/3 args=v
  stub 021b86b8 ret=pick:1/1/0 args=v
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021b904c ret=u32 args=v,v
  stub 021b90bc ret=none args=v,v
  stub 021b99b8 ret=pick:0/1/2/3 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9b94 ret=pick:0/1/2/3 args=v
  stub 021b9b98 ret=pick:0/1/2 args=v
  stub 021ba004 ret=nullable:4:u32 args=v,v
  stub 021ce08c ret=pick:0/1 args=p
  stub 021d5778 ret=none args=v,v
  stub 021d59b4 ret=u16 args=v,v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d5bdc ret=pick:0/0/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67a8 ret=u32 args=v
  stub 021d67c4 ret=u32 args=v
  stub 021d7b60 ret=u16 args=v,v
  stub 021d82d0 ret=none args=p,v,v
  stub 021d8444 ret=none args=p
  stub 021e8b80 ret=u32 args=v
  stub 021ea614 ret=none args=v
  stub 021eaf78 ret=none args=v,v
  stub 021f8524 ret=pick:0/1 args=v

# callback 0x021CE7D5 (4-way jump table, data 021ce7ee-021ce7f6; a "use the item" animation): 0: id = 021b904c(server, 0), p =
# 021ba004(server, id), 021f7f58(021ba020(server, id), 9, 0, 0, 0), 021f7998(0x270), 021ea6b0(view, p, c.u16[0x19E]); item 0x13/0x14 and
# 021d7f38(021ba020(..)) and !(flags & 1) and (021f87ec(0x47B), !021f8774()) -> 02005e30(8), state 1; else state 2. 1: !02005e50() ->
# 02005d90(0x47B, 0xFFFF), flags |= 1, state 2. 2: !021f7eb4() and 021ea710(view) -> 021f7998(0x271), state 3. 3: !021f7eb4() ->
# 021ce150(c, 0x021CE4D1). Returns 0.
fn 021ce7d4 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 3 || any
  fix arg0 0x19E u16 alt oneof 0x13 0x14 || any
  stub 021b904c ret=u32 args=v,v
  stub 021ba004 ret=u32 args=v,v
  stub 021ba020 ret=u32 args=v,v
  stub 021f7f58 ret=none args=v,v,v,v,v
  stub 021f7998 ret=none args=v
  stub 021ea6b0 ret=none args=v,v,v
  stub 021d7f38 ret=pick:0/1 args=v
  stub 021f87ec ret=none args=v
  stub 021f8774 ret=pick:0/1 args=
  stub 02005e30 ret=none args=v
  stub 02005e50 ret=pick:0/1 args=
  stub 02005d90 ret=none args=v,v
  stub 021f7eb4 ret=pick:0/1 args=
  stub 021ea710 ret=pick:0/1 args=v
  stub 021ce150 ret=none args=p,v

# battle-item shortcut check (only when 021b86b8(server) == 1): bag = 021ba004(server, id = 021b904c(server, 0)) (none -> 0); party =
# 021b9aec(pokecon, id); the first live mon (021b990c / 021d5b68, up to 021b99b8 slots); for each of the 4 items of the ROM table
# 0x021EFF78 not yet flagged (c[0x112 + i]) that 0202a298(bag, item, c.u16[0x19A]) has: item - 0x11 selects a test (4-way jump table, data
# 021ce9da-021ce9e2: HP <= 021d7b60(mon, 2) / !021d5b48(mon) / party size 021b9b94 > 1 and 021b9b98 == 1 (twice, the last with the HP
# test)) -> flag it, remember i; any item: c[0x112 + i] = 1. Found: *out (u16) = the item, return 1; else 0.
fn 021ce8f0 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg0 0x112 u8 alt const 0 || any rep=4,1
  stub 021b86b8 ret=pick:1/1/0 args=v
  stub 021b904c ret=u32 args=v,v
  stub 021ba004 ret=nullable:4:u32 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b990c ret=u32 args=v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021b99b8 ret=pick:0/1/2/3 args=v,v
  stub 0202a298 ret=pick:0/1 args=v,v,v
  stub 021d59b4 ret=u16 args=v,v
  stub 021d7b60 ret=u16 args=v,v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 021b9b94 ret=pick:0/1/2/3 args=v
  stub 021b9b98 ret=pick:0/1/2 args=v

# callback 0x021CEA95 (fight menu; 8-way jump table, data 021ceaae-021ceabe): 0: battle type 3 -> 021cecb4(c, c+0xA4), state 2; else
# 021cf524(c, mon, c+0xC) ? 021ce150(c, 0x021CF221) : state 1. 1: 021e97bc(view, mon, c+0xC), state 3. 2: 021e9808(view, c+0xA4, c+0xC),
# state 3. 3: 021ce0bc(c, 0x021CE3C9) ? 021e98ac(view) (thunk) : 021e98b4(view) (thunk) -> a = 021d8454(c+0xC): 0 -> 0x021CE4D1, 5 ->
# 0x021CF221, else move / target checks (battle type 3: side from c+0xBC bits 4-6, 021b9ab0, the mon of c+0xA4 + 8 * side; 021d832c,
# 021d5778, 021cf5b0, 021eaf78 / 021ce3b4, 021d5b68, 021d8340 / 021d83fc / 021d82d0 at c+0xC += 4, c[0x1AF]++) -> states 4 / 7 or
# 0x021CED4D. 4: 021d7ef0(battle type) ? state 5 : 0x021CF221. 5: 021e98bc(view, mon, c+0xC), state 6. 6: 021ce0bc -> 021e9920 :
# 021e98e8(view) 1 -> 0x021CF221, 2 -> state 1. 7: 021ea710(view) -> 021e9858(view), state 3. Returns 0.
fn 021cea94 ret=u32 inputs=1000
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 7 || const 3 || const 3 || any
  fix arg0 0x0C u32 addrof arg0 0x124
  fix arg0 0x1A0 u16 alt const 0 || any rep=3,2
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  stub 021b8588 ret=pick:0/3/3/1 args=v
  stub 021cecb4 ret=none args=p,p
  stub 021cf524 ret=pick:0/1 args=p,v,p
  stub 021ce150 ret=none args=p,v
  stub 021e97bc ret=none args=v,v,p
  stub 021e9808 ret=none args=v,p,p
  stub 021ce0bc ret=pick:0/1 args=p,v
  stub 021e98ac ret=none args=v
  stub 021e98b4 ret=pick:0/1 args=v
  stub 021d8454 ret=pick:0/5/1/2 args=p
  stub 021b9ab0 ret=pick:0/1/2 args=v
  stub 021d832c ret=pick:0/1 args=p
  stub 021d5778 ret=pick:0/1/4 args=v,v
  stub 021cf5b0 force ret=pick:0/1 args=p,v,v,p
  stub 021eaf78 ret=none args=p,v
  stub 021ce3b4 ret=none args=p
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d8340 ret=u32 args=p
  stub 021d83fc ret=none args=p,v
  stub 021d82d0 ret=none args=p,v,v
  stub 021d7ef0 ret=pick:0/1 args=v
  stub 021e98bc ret=none args=v,v,p
  stub 021e9920 ret=none args=v
  stub 021e98e8 ret=pick:0/1/2 args=v
  stub 021ea710 ret=pick:0/1 args=v
  stub 021e9858 ret=none args=v
  stub 021b9c00 ret=u32 args=v,v
  stub 021ce08c ret=pick:0/1 args=p
  stub 021cf58c ret=none args=p,p,v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021d5584 ret=pick:0/1 args=v,v
  stub 021d81f8 ret=u32 args=v,v,v,v,p
  stub 021ea614 ret=none args=v
  stub 021f8524 ret=pick:0/1 args=v

# fight-menu table (double battle type 3): for side i < 3: mon = 021b9c00(c+0x118 party, i), out[8i] = mon, out[8i + 4 + j] = 0 (j < 4);
# a live mon (021d5b68 == 0): out[8i + 4 + j] = move j usable (021d5584(mon, j) and !021cf5b0(c, mon, 021d54c0(mon, j), 0)) for j <
# 021d5448(mon)
fn 021cecb4 ret=none
  nowork
  arg buf 0x1C0
  arg buf 0x20
  stub 021b9c00 ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d5584 ret=pick:0/1 args=v,v
  stub 021d54c0 ret=u16 args=v,v
  stub 021cf5b0 force ret=pick:0/1 args=p,v,v,v

# callback 0x021CED4D (target page): 0: 021e9ce0(view, c[0x1B0], c[0x1B1]), state 1. 1: 021ce0bc(c, 0) ? (021e9b14(view), state 2) :
# 021e9b20(view) -> 021ce150(c, 0x021CEA95). 2: 021e9b20(view) -> 021ce150(c, 0x021CE3C9). Returns 0.
fn 021ced4c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 2 || any
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  stub 021e9ce0 ret=none args=v,v,v
  stub 021ce0bc ret=pick:0/1 args=p,v
  stub 021e9b14 ret=none args=v
  stub 021e9b20 ret=pick:0/1 args=v
  stub 021ce150 ret=none args=p,v
  stub 021ce08c ret=pick:0/1 args=p
  stub 021f8524 ret=pick:0/1 args=v

# callback 0x021CEDC1 (party page): 0: r = 021cf8d4(c, mon, &a, &b); f = r == 4 && a == 021d5440(mon) ... (f = r != 4 or a matches);
# 021e9a78(view, c+0x184, c[0x11D], f, c+0x190), state 1. 1: 021ce0bc(c, 0) ? (021e9b14(view), state 2) : 021e9b20(view) -> veneer
# 021cee98(c+0x190) ? 0x021CE4D1 : (n = veneer 021ceea8(c+0x190)) < 6 ? (021d8394(c+0xC, c[0x11D], n), 0x021CF221) : 0x021CE4D1.
# 2: 021e9b20(view) -> 0x021CE3C9. Returns 0.
fn 021cedc0 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 2 || any
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  stub 021cf8d4 force ret=pick:4/4/0/1 args=p,v,outm:2:3/0,outm:2:0xFF/0xFF
  stub 021d5440 ret=pick:0/1/2/3 args=v
  stub 021e9a78 ret=none args=v,p,v,v,p
  stub 021ce0bc ret=pick:0/1 args=p,v
  stub 021e9b14 ret=none args=v
  stub 021e9b20 ret=pick:0/1 args=v
  stub 021cee98 ret=pick:0/1 args=p
  stub 021ceea8 ret=pick:0/2/5/6/7 args=p
  stub 021d8394 ret=none args=p,v,v
  stub 021ce150 ret=none args=p,v
  stub 021ce08c ret=pick:0/1 args=p
  stub 021f8524 ret=pick:0/1 args=v

# callback 0x021CEEB9 (bag page): 0: 021e9d04(view, c[0x1AA], c[0x1AB], 021cf448(c), c[0x11D] == c[0x11F], 021ceffc(c)), state 1.
# 1: 021ce0bc(c, 0) ? (021e9e70(view), state 2) : 021e9e80(view) -> item = 021ea01c(view), page = 021ea044(view); item and page != 6:
# n = 021ea030(view), k = 021ea04c(view); c[0x1AB] -= n (clamped: below n -> 0 and n = 0); 021cf430(c, c[0x11D], n);
# 021d8358(c+0xC, item, page, k); 021ea058(view); 021d805c(item, 0xF) == 4 -> flags |= 4; 021cf37c(c, c[0x11D], item); 0x021CF221;
# else 0x021CE4D1. 2: 021e9e80(view) -> 0x021CE3C9. Returns 0.
fn 021ceeb8 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 2 || any
  fix arg0 0x11C u8 alt range 0 6 || any
  fix arg0 0x11D u8 range 0 5
  fix arg0 0x11F u8 range 0 5
  fix arg0 0x1AA u8 alt const 0 || any
  fix arg0 0x1A6 u8 range 0 3
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  stub 021cf448 ret=u8 args=p
  stub 021ceffc ret=pick:0/1 args=p
  stub 021e9d04 ret=none args=v,v,v,v,v,v
  stub 021ce0bc ret=pick:0/1 args=p,v
  stub 021e9e70 ret=none args=v
  stub 021e9e80 ret=pick:0/1 args=v
  stub 021ea01c ret=nullable:3:u16 args=v
  stub 021ea044 ret=pick:0/1/6 args=v
  stub 021ea030 ret=u8 args=v
  stub 021ea04c ret=u8 args=v
  stub 021cf430 ret=none args=p,v,v
  stub 021d8358 ret=none args=p,v,v,v
  stub 021ea058 ret=none args=v
  stub 021d805c ret=pick:0/4 args=v,v
  stub 021cf37c ret=none args=p,v,v
  stub 021ce150 ret=none args=p,v
  stub 021b8670 ret=pick:0/1/2/3 args=v
  stub 021b86b8 ret=pick:0/0/1 args=v
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021b9090 ret=none args=v,v,v
  stub 021b9af8 ret=u32 args=v,v
  stub 021b9b94 ret=pick:0/1/2/3/6 args=v
  stub 021b9c00 ret=u32 args=v,v
  stub 021ce08c ret=pick:0/1 args=p
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5c30 ret=pick:0/1 args=v
  stub 021f8524 ret=pick:0/1 args=v

# exactly one live mon in the first min(021b9b94(party = 021b9af8(pokecon, 1)), 021b8670(server)) slots and some live mon passes
# 021d5c30; only when 021b86b8(server) == 0
fn 021ceffc ret=u32
  nowork
  arg buf 0x1C0
  stub 021b86b8 ret=pick:0/0/1 args=v
  stub 021b9af8 ret=u32 args=v,v
  stub 021b9b94 ret=pick:0/1/2/3/6 args=v
  stub 021b8670 ret=pick:0/1/2/3 args=v
  stub 021b9c00 ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5c30 ret=pick:0/1 args=v

# callback 0x021CF079 (run; 6-way jump table, data 021cf094-021cf0a0): 0: by 021b86e4(server): 1 -> message 0x4C, state 1; 2 -> message
# 0x4D + 021eaf78(c+0x58, 0xFF), state 3; else 021cf918(c, &a, &b) == 4 -> state 5, else message b ? (0x365: a, b) : 0x4A, state 1.
# 1: 021ea710(view) -> 021ce0f8(c, mon, c+0x58), 021ea65c(view, c+0x58), state 2. 2: 021ea710 -> 0x021CE4D1. 3: 021ea710(view),
# 021ea720(view) -> yes/no window (021eaf18 x2, 021eabc4(view, c+0x58, c+0x7C, 1)), state 4. 4: 021ea710 -> 021eac54(view, &ans) ?
# (ans ? 0x021CE4D1 : state 5) : 021ce08c(c) -> 0x021CE3C9. 5: 021d8414(c+0xC), flags |= 4, 0x021CF221. Returns 0.
fn 021cf078 ret=u32 inputs=600
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 5 || any
  fix arg0 0xE4 u16 alt const 0 || any
  fix arg0 0x1AD u8 alt const 0 || any
  stub 021b86e4 ret=pick:0/1/2/3 args=v
  stub 021cf918 force ret=pick:4/0/1 args=p,outm:2:0xFF/0,outm:2:3/0
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021ce3b4 ret=none args=p
  stub 021eaf78 ret=none args=p,v
  stub 021ea710 ret=pick:0/1 args=v
  stub 021ce0f8 ret=none args=p,v,p
  stub 021ea65c ret=none args=v,p
  stub 021ea720 ret=pick:0/1 args=v
  stub 021eabc4 ret=none args=v,p,p,v
  stub 021eac54 ret=pick:0/1 args=v,outm:4:1/0/0/0
  stub 021ce08c ret=pick:0/1 args=p
  stub 021d8414 ret=none args=p
  stub 021ce150 ret=none args=p,v
  stub 021b8588 ret=pick:0/3 args=v
  stub 021d5440 ret=u32 args=v
  stub 021ea614 ret=none args=v
  stub 021f8524 ret=pick:0/1 args=v

# callback 0x021CF221 (next mon): 0: 021e9964(view), state 1. 1: 021e9974(view) -> c[0x11D]++; more mons: !(flags & 4) -> 0x021CE4D1, else
# the rest are done: for each remaining index, 021cf470(c, mon, &c[0x124 + 4i]) == 0 -> 021d8434(that record); all answered: c[0x1AF] +=
# c[0x11C] (count), result = &c[0x124], size = 4 * count, c[0x1AF] = 0, 0x021CF2F1. Returns 0.
fn 021cf220 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 1 || any
  fix arg0 0x11C u8 alt range 0 6 || range 0 0x24
  fix arg0 0x11D u8 alt range 0 6 || any
  stub 021e9964 ret=none args=v
  stub 021e9974 ret=pick:0/1 args=v
  stub 021b990c ret=u32 args=v,v,v
  stub 021cf470 ret=pick:0/1 args=p,v,p
  stub 021d8434 ret=none args=p
  stub 021ce150 ret=none args=p,v
  stub 021d5778 ret=none args=v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d5bdc ret=pick:0/0/1 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021d67a8 ret=u32 args=v
  stub 021d67c4 ret=u32 args=v
  stub 021d82d0 ret=none args=p,v,v
  stub 021d8444 ret=none args=p
  stub 021e8b80 ret=u32 args=v

# callback 0x021CF2F1 (end): 0: 021cf3f8(c); 021b86c0(server) ? (c[0x1A9] = 1, 021ea7fc(view), state 1) : state 2. 1: 021ea814(view) -> state
# 2. 2: !021f7eb4() -> 021ce0e0(c), state 3, return 1. Other states: 1. Returns 0 otherwise.
fn 021cf2f0 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 3 || any
  fix arg0 0x1AA u8 alt const 0 || any
  fix arg0 0x1A6 u8 range 0 3
  fix arg0 0x1A0 u16 alt const 0 || any rep=3,2
  fix arg0 0xE4 u16 alt const 0 || any
  stub 021cf3f8 ret=none args=p
  stub 021b86c0 ret=pick:0/1 args=v
  stub 021ea7fc ret=none args=v
  stub 021ea814 ret=pick:0/1 args=v
  stub 021f7eb4 ret=pick:0/1 args=
  stub 021ce0e0 ret=none args=p
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021b90bc ret=none args=v,v
  stub 021cf3b8 ret=none args=p,v
  stub 021f8504 ret=none args=v,v,v

# c[0x1AA] == 0 and 021b8ff4(server) == c[0x1A6]: c.u16[0x1A0 + 2 r1] = r2, 021b9090(server, c[0x1A6], r2)
fn 021cf37c ret=none
  nowork
  arg buf 0x1C0
  arg u32 range 0 2
  arg u32 any
  fix arg0 0x1AA u8 alt const 0 || any
  fix arg0 0x1A6 u8 range 0 3
  stub 021b8ff4 ret=pick:0/1/2/3 args=v
  stub 021b9090 ret=none args=v,v,v

# the mon has an answer already (r0 unused; r2 = its answer record, may be null): 021d5b68(mon) -> 021d8434(rec) ("skip"); 021d5bdc(mon,
# 0xC) -> 021d8444(rec); flag 0x19 (021d62c4) -> 021d82d0(rec, 021e8b80(021d6314(mon, 0x19)), 021d67c4(mon)); flag 0x1A -> 021d5778(mon,
# m = 021d67a8(mon)), 021d82d0(rec, m, 021d67c4(mon)); each returns 1; else 0
fn 021cf470 ret=u32
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d8434 ret=none args=p
  stub 021d5bdc ret=pick:0/0/1 args=v,v
  stub 021d8444 ret=none args=p
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d6314 ret=u32 args=v,v
  stub 021e8b80 ret=u32 args=v
  stub 021d67c4 ret=u32 args=v
  stub 021d82d0 ret=none args=p,v,v
  stub 021d67a8 ret=u32 args=v
  stub 021d5778 ret=none args=v,v

# no usable move (for j < 021d5448(mon): !021cf5b0(c, mon, 021d54c0(mon, j), 0) and 021d5584(mon, j) counts): 021cf58c(rec, c, mon)
# (struggle), return 1; else 0
fn 021cf524 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  stub 021d5448 ret=pick:0/1/2/4 args=v
  stub 021d54c0 ret=u16 args=v,v
  stub 021cf5b0 force ret=pick:0/1 args=p,v,v,v
  stub 021d5584 ret=pick:0/0/1 args=v,v
  stub 021cf58c ret=none args=p,p,v
  stub 021d81f8 ret=u32 args=v,v,v,v,p
  stub 021d82d0 ret=none args=p,v,v

# struggle: 021d82d0(rec, 0xA5, 021d81f8(c[0], c[4], mon, 0xA5, c+0x38))
fn 021cf58c ret=none
  nowork
  arg u32 any
  arg buf 0x1C0
  arg u32 any
  stub 021d81f8 ret=u32 args=v,v,v,v,p
  stub 021d82d0 ret=none args=p,v,v
