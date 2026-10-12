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
