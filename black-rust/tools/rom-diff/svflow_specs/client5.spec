# svflow spec: client5 (btl_client.c part 5, 0x021D2FD8-0x021D3F04: server-command handlers of the playback table and client getters;
# B13-spec1). Harness-job owned; porting workers may not edit. Grammar: svflow_specs/README.md. Area split: doc comment of client/mod.rs.
#
# What this area is: the handlers of the command table at 0x021EFFDC.. (pairs {u32 command id, Thumb fn}; ids 0x30-0x42, 0x57-0x5D, 0x59-0x5D ...,
# fn 021d2fd9 ... 021d3d51): each is `u32 handler(client c, u32 unused, u32 *args)` and returns 1 ("done") after relaying one server command to
# the battle view / BattleMon. Most are the same shape: `mon = 021b9934(c.pokecon, u8 args[0])` followed by one BattleMon view call
# `021d5xxx..021d6xxx(mon, args[1], ...)`. `arg buf 0x40` is the argument record (random words; the masks (u8/u16) are the port's job).
# Same client object as client1.spec (`arg buf 0x1C0`, `nowork`): +0x000 server, +0x004 pokecon, +0x034 view-side object, +0x118 party
# container, +0x0E8 4 x u16, +0x1A6 client id, +0x1AB counter.
# Stub conventions: the callee list is the register arguments of the call (the mon is the first), results of the view calls are unused (`ret=none`).
# 021b9934 (pokecon, id) -> BattleMon* is one stub address shared by all handlers.
# Handlers of the table that are NOT specced in this batch: 021d2fd8 .. 021d3568 (command handlers with loops/state), 021d3568 .. 021d35c8, 021d35c8,
# 021d3630 021d3688 (jump into big view routines), 021d3934, 021d39a8 (several in-area calls), 021d3a10, 021d3dc4 021d3e70 021d3ec8.
area client5

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5f90(mon, u16 at args[1]); returns 1
fn 021d36cc ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5f90 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5fa4(mon, u16 at args[1]); returns 1
fn 021d36ec ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5fa4 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5608(mon, u8 args[1], u8 args[2]); returns 1
fn 021d370c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5608 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5640(mon, u8 args[1], u8 args[2]); returns 1
fn 021d3734 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5640 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d56cc(mon, u8 args[1]); returns 1
fn 021d375c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d56cc ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5fb8(mon, no extra argument); returns 1
fn 021d377c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5fb8 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5674(mon, u8 args[1], u8 args[2]); returns 1
fn 021d3794 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5674 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d56a0(mon, u8 args[1], u8 args[2]); returns 1
fn 021d37b8 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d56a0 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5dc0(mon, args[1], u8 args[2]); returns 1
fn 021d37dc ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5dc0 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5e38(mon, args[1], u8 args[2]); returns 1
fn 021d37fc ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5e38 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5efc(mon, no extra argument); returns 1
fn 021d3894 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5efc ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5f08(mon, no extra argument); returns 1
fn 021d38ac ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5f08 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5f38(mon, args[1]); returns 1
fn 021d38c4 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5f38 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6034(mon, args[1], args[2]); returns 1
fn 021d38e0 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6034 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d61f0(mon, no extra argument); returns 1
fn 021d3900 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d61f0 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6238(mon, args[1]); returns 1
fn 021d3918 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6238 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d66b0(mon, args[1]); returns 1
fn 021d3988 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d66b0 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6730(mon, args[1]); returns 1
fn 021d3a20 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6730 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6744(mon, args[1], args[2], args[3], args[4], args[5]); returns 1
fn 021d3a40 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6744 ret=none args=v,v,v,v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5fe4(mon, args[1]); returns 1
fn 021d3a7c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5fe4 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6008(mon, args[1]); returns 1
fn 021d3a98 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6008 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5fc0(mon, args[1]); returns 1
fn 021d3ab4 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5fc0 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6440(mon, args[1]); returns 1
fn 021d3ad0 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6440 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d66d0(mon, args[1]); returns 1
fn 021d3aec ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d66d0 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5b44(mon, args[1]); returns 1
fn 021d3b08 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5b44 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5734(mon, args[1], args[2], args[3], args[4]); returns 1
fn 021d3b28 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5734 ret=none args=v,v,v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d64f0(mon, no extra argument); returns 1
fn 021d3b5c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d64f0 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d69a0(mon, args[1], args[2]); returns 1
fn 021d3bc0 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d69a0 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6d5c(mon, args[1]); returns 1
fn 021d3c0c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6d5c ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6d70(mon, no extra argument); returns 1
fn 021d3c2c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6d70 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6738(mon, no extra argument); returns 1
fn 021d3c90 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6738 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6264(mon, u8 args[1]); returns 1
fn 021d3ca8 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6264 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5970(mon, args[1], args[2]); returns 1
fn 021d3d2c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5970 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d67d8(mon, args[1]); returns 1
fn 021d3d50 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d67d8 ret=none args=v,v

# 7 BattleMon view calls 021d5ea8(mon, k, u8 args[k + 0]) for k = 1..7; returns 1
fn 021d381c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5ea8 ret=none args=v,v,v

# 021ef9f4(c.view_side, args[0], args[1], 0); returns 1
fn 021d3b74 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021ef9f4 ret=none args=v,v,v,v

# 021efad8(c.view_side, args[0], u8 args[1]); returns 1
fn 021d3b88 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021efad8 ret=none args=v,v,v

# 021efb34(c.view_side, u8 args[0]); returns 1
fn 021d3b9c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021efb34 ret=none args=v,v

# 021efa94(c.view_side, args[0]); returns 1
fn 021d3bb0 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021efa94 ret=none args=v,v

# 021d65c8(mon_b, mon_a) with mon_a = 021b9934(pokecon, u8 args[0]), mon_b = 021b9934(pokecon, u8 args[1]); returns 1
fn 021d3be0 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d65c8 ret=none args=v,v

# args[0] == client id (c[0x1A6]): counter += u8 args[1] (021cd9f4, client1) else 021b9b10(server, u8 args[0]); returns 1
fn 021d3c44 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  fix arg0 0x1A6 u8 range 0 3
  fix arg2 0 u32 alt range 0 3 || range 0x100 0x103 || any
  stub 021cd9f4 ret=none args=ptr,v
  stub 021b9b10 force ret=none args=v,v

# x = 021b9aec(pokecon, u8 args[0]); 021b9d10(server, x, u8 args[0]); returns 1
fn 021d3c70 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9d10 force ret=none args=v,v,v

# 021d6848(mon, &{u16 args[4], u16 args[5], u8 args[3], u8 args[1], u8 args[2], pad}); returns 1
fn 021d3cc8 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6848 ret=none args=v,in:7

# mon = 021b9934(pokecon, u8 args[0]); 021d6408(mon); 021d6e88(mon); returns 1
fn 021d3cfc ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 force ret=u32 args=v,v
  stub 021d6408 ret=none args=v
  stub 021d6e88 ret=none args=v

# 021efc78(c.view_side, 0, 0); returns 1
fn 021d3d1c ret=u32
  nowork
  arg buf 0x1C0
  stub 021efc78 ret=none args=v,v,v

# client id
fn 021d3d70 ret=u8
  nowork
  arg buf 0x1C0

# party container
fn 021d3d7c ret=u32
  nowork
  arg buf 0x1C0

# counter
fn 021d3d90 ret=u8
  nowork
  arg buf 0x1C0

# c.u16[0xE8 + 2i] = 021b9f84(server, client id, i) for i in 0..4
fn 021d3d9c ret=none
  nowork
  arg buf 0x1C0
  stub 021b9f84 force ret=u32 args=v,v,v

# ---------------------------------------------------------------------------------------------------------------------------------- B13-spec2
# B13-spec2 specs every function the B13-spec1 header above lists as "not specced in this batch" (list in client/mod.rs).
# The rest of the playback handlers (`u32 handler(c, u32 *state, u32 *args)`: arg 1 = state word `arg buf 0x10`, arg 2 = the argument words
# `arg buf 0x40`; one-shot handlers ignore the state) and the trainer-item helpers 021d3dc4 / 021d3e70 / 021d3ec8 (021d0770 uses them).
# 021d32e0 keeps a counter in the overlay global 0x021F3B64 + 4 (`global 021f3b64 0x20`, shared with client4 021d2a20).
# ROM tables read as data: 0x021EFF80 (4 x u16 message ids, 021d2fd8), 0x021EFF74 (4 bytes), 0x021EFF94 (6 x {u8, u8}), 0x021EFFA0 (7 x
# {u8, u8}). Veneers 021d3558, 021d3a10 (Thumb `bx pc` to ARM thunks to 0x0689xxxx) are stubbed at the Thumb entry.
# 021b9a24 (same side) is a plain stub here (it is real code only under `env bpp`).
# Note for B13: the B13-spec1 block of 021d3c44 lists `stub 021cd9f4`, which is PORTED (client1 counter_add): that stub is inactive, so the ROM
# side runs the real counter_add and a port of 021d3c44 must call client1::counter_add (not env.call).

# command 0x46 (exp gain; 5-way jump table, data 021d2ff6-021d3000): 0: 021f7cfc(021b8e70(server, (u8) args[0]), (u16) args[5], (u8)
# args[1], args[2], args[4]), state 1. 1: !021f7eb4(): args[2] -> message 0x41 with 021d5440(021b98ac(pokecon, (u8) args[0])),
# 021f883c(0x518); else message u16 0x021EFF80[args[1]] (args[1] < 4) or 0x3D; 021ea614(view, c+0x58), state 2. 2: 021ea710(view) ->
# args[2] ? (021ba2d4(server, 021b98ac(..)), state 3) : 1. 3: !02005f38() -> 021f7c0c(), 02005d90(0x47C, 0xFFFF), 021b92a0(server); args[3]
# -> message 0x42 (021b98ac, 021d5440), 021ea614, state 4; else 1. 4: 021ea710 -> state 5. Other states: 1.
fn 021d2fd8 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 4 u32 alt range 0 4 || any
  fix arg2 8 u32 alt const 0 || any
  fix arg2 0xC u32 alt const 0 || any
  stub 021b8e70 force ret=u32 args=v,v
  stub 021f7cfc ret=none args=v,v,v,v,v
  stub 021f7eb4 ret=pick:0/1 args=
  stub 021b98ac force ret=u32 args=v,v
  stub 021eaf18 ret=none args=p,v,v
  stub 021d5440 ret=u32 args=v
  stub 021eaf50 ret=none args=p,v
  stub 021f883c ret=none args=v
  stub 021ea614 ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v
  stub 021ba2d4 force ret=none args=v,v
  stub 02005f38 ret=pick:0/1 args=
  stub 021f7c0c ret=none args=
  stub 02005d90 ret=none args=v,v
  stub 021b92a0 force ret=none args=v

# command 0x47: 0: 021f7d44(021b8e70(server, (u8) args[0]), (u16) args[1]), state 1. 1: !021f7eb4() -> message 0x43, 021ea614, state 2.
# 2: 021ea710(view) -> 1.
fn 021d3138 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 021b8e70 force ret=u32 args=v,v
  stub 021f7d44 ret=none args=v,v
  stub 021f7eb4 ret=pick:0/1 args=
  stub 021eaf18 ret=none args=p,v,v
  stub 021ea614 ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v

# command 0x48 (k = u8 args[0], slot = u8 args[1]): 0: 021b9c50(021b9aec(pokecon, k), slot, 0, 0), 021eaa6c(view, k, slot), state 1. 1:
# 021eaad8(view) -> args[1] == 1 -> 1; else f = (u8) 021b9f70(server, (u8) args[0]): message (slot 3 ? (f ? 0x26 : 0x27) : (f ? 0x28 : 0x29))
# 021ea678(view, id, args), state 2. 2: 021ea710(view) -> 1.
fn 021d31a0 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  fix arg2 4 u8 alt range 1 3 || any
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9c50 force ret=none args=v,v,v,v
  stub 021eaa6c ret=none args=v,v,v
  stub 021eaad8 ret=pick:0/1 args=v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021ea678 ret=none args=v,v,p
  stub 021ea710 ret=pick:0/1 args=v

# command 0x49 (k = u8 args[0], s = 021b8d8c(server, pokecon, k)): 0: idle (021cd9e0) -> 021d66d0(021b9934(pokecon, k), args[1]), 1; else
# 021ea768(view, s, 0), state 1. 1: 021ea778(view, s) -> 021d66d0(mon, args[1]), 021ea7a8(view, s), state 2. 2: 021ea7b8(view, s) -> state
# 3. 3: 1.
fn 021d324c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 3 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021cd9e0 force ret=pick:0/0/1 args=ptr
  stub 021b9934 force ret=u32 args=v,v
  stub 021d66d0 ret=none args=v,v
  stub 021ea768 ret=none args=v,v,v
  stub 021ea778 ret=pick:0/1 args=v,v
  stub 021ea7a8 ret=none args=v,v
  stub 021ea7b8 ret=pick:0/1 args=v,v

# command 0x4A (two mons a = u8 args[0], b = u8 args[1], values u16 args[2] / args[3]; 7-way jump table, data 021d3346-021d3354): s = 021b8d8c
# (a), t = 021b8d8c(b), ma / mb = 021b9934(pokecon, a / b). 0: idle or same side (021b9a24(a, b)) -> 021d66d0(ma, args[2]), 021d66d0(mb,
# args[3]), 1; else 021ea768(view, s, 0), g[4] = 8, state 1. 1: 021ea778(view, s); g[4] ? g[4]-- : (021ea768(view, t, 0), state 2).
# 2: both 021ea778(view, s) and (view, t) -> state 3. 3: both 021d66d0, g[4] = 0, state 4. 4: ++g[4] > 8 -> 021ea7a8(view, s), state 5.
# 5: 021ea7b8(view, s) -> 021ea7a8(view, t), state 6. 6: 021ea7b8(view, t) -> state 7. 7: 1.
fn 021d32e0 ret=u32 inputs=400
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  global 021f3b64 0x20
  fix arg1 0 u32 alt range 0 7 || any
  fix global0 4 u32 alt range 0 9 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021cd9e0 force ret=pick:0/0/1 args=ptr
  stub 021d66d0 ret=none args=v,v
  stub 021b9a24 force ret=pick:0/0/1 args=v,v
  stub 021ea768 ret=none args=v,v,v
  stub 021ea778 ret=pick:0/1/1 args=v,v
  stub 021ea7a8 ret=none args=v,v
  stub 021ea7b8 ret=pick:0/1 args=v,v

# command 0x4B: 0: s = 021b8d8c(server, pokecon, (u8) args[0]), m = 021b9934(pokecon, args[0]), i = 021cd9e0(c) (idle); 021d6c2c(m);
# 021ea868(view, s, i), state 1. 1: 021ea878(view) -> state 2. 2: 1.
fn 021d344c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021cd9e0 force ret=pick:0/1 args=ptr
  stub 021d6c2c ret=none args=v
  stub 021ea868 ret=none args=v,v,v
  stub 021ea878 ret=pick:0/1 args=v

# command 0x4C (SE): state 0: idle -> 1; 021f7998(args[0]), state 1. Other states: !021f7eb4() -> 1.
fn 021d34b0 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021cd9e0 force ret=pick:0/0/1 args=ptr
  stub 021f7998 ret=none args=v
  stub 021f7eb4 ret=pick:0/1 args=

# command 0x4D: p = 021b8e70(server, (u8) args[0]); state 0: idle -> 1; !021b8590(server) and veneer 021d3558((u16) args[1]) -> 1;
# 021ea3b8(view, p, (u16) args[1]), state 1. Other states: 021ea3d4(view) -> 1.
fn 021d34e8 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8e70 force ret=u32 args=v,v
  stub 021cd9e0 force ret=pick:0/0/1 args=ptr
  stub 021b8590 force ret=pick:0/1 args=v
  stub 021d3558 ret=pick:0/1 args=v
  stub 021ea3b8 ret=none args=v,v,v
  stub 021ea3d4 ret=pick:0/1 args=v

# command 0x4E: state 0: idle -> 1; 021ea3e8(view, 021b8e70(server, (u8) args[0]), 021b8e70(server, (u8) args[1]), (u16) args[2]), state 1.
# Other states: 021ea3f8(view) -> 1.
fn 021d3568 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021cd9e0 force ret=pick:0/0/1 args=ptr
  stub 021b8e70 force ret=u32 args=v,v
  stub 021ea3e8 ret=none args=v,v,v,v
  stub 021ea3f8 ret=pick:0/1 args=v

# command 0x4F: 0: p = 021b8e70(server, 021b8d8c(server, pokecon, (u8) args[0])); 021d66d8(021b9934(pokecon, args[0]), (u8) args[1]);
# 021ea888(view, p), state 1. 1: 021ea8a4(view) -> state 2. 2: 1.
fn 021d35c8 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b8e70 force ret=u32 args=v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021d66d8 ret=none args=v,v
  stub 021ea888 ret=none args=v,v
  stub 021ea8a4 ret=pick:0/1 args=v

# command 0x57: s = 021b8d8c(server, pokecon, (u8) args[0]); 0: idle -> 1; 021ea768(view, s, 1), state 1. 1: 021ea778(view, s) -> state 2.
# 2: 1.
fn 021d3630 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021cd9e0 force ret=pick:0/0/1 args=ptr
  stub 021ea768 ret=none args=v,v,v
  stub 021ea778 ret=pick:0/1 args=v,v

# command 0x58: s = 021b8d8c(server, pokecon, (u8) args[0]) (passed in r1); 0: 021ea788(view, s), state 1. 1: 021ea798(view, s) -> 1.
fn 021d3688 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021ea788 ret=none args=v,v
  stub 021ea798 ret=pick:0/1 args=v,v

# command 0x12 (switch in): from the client object's own argument copy (c+0x138 id, c+0x13C slot a, c+0x140 slot b, u8 each) and u16 args[3]:
# party = 021b9aec(pokecon, c+0x138); a != b -> BattleParty_SwapSlots(party, a, b); m = 021b9bf0(party, a); 021d63ec(m, args[3]);
# 021d6580(m). Returns 1.
fn 021d3934 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  fix arg0 0x140 u8 alt range 0 5 || any
  fix arg0 0x13C u8 range 0 5
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9c10 force ret=none args=v,v,v
  stub 021b9bf0 force ret=u32 args=v,v
  stub 021d63ec ret=none args=v,v
  stub 021d6580 ret=none args=v

# command 0x15 (n = args[1] rounds, k = args[0] mons from the list 021d8090(args[2], list)): for i < n: e = veneer 021d3a10(i), 0 -> stop;
# each alive listed mon m = 021b9934(pokecon, list[j]) -> 021d6064(m, e, 0, 0). Returns 1. (k <= 8: an 8-byte stack list.)
fn 021d39a8 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  fix arg2 0 u32 range 0 8
  fix arg2 4 u32 alt range 0 4 || range 0 0x10
  stub 021d8090 ret=none args=v,outm:8:0xFF
  stub 021d3a10 ret=nullable:4:u32 args=v
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d6064 ret=none args=v,v,v,v

# trainer item to use on mon r1 (party r2): flag 0x13 (021d62c4) -> 0; for i < 4 while the ROM byte 0x021EFF74[i] >= 021b9b94(party): item
# c.u16[0xE8 + 2i] != 0: 021d805c(item, 0x29) ? (HP 021d59b4(mon, 0xD) <= 021d7b60(mon, 4)) : (021d3e70(item, mon) or 021d3ec8(item, mon))
# -> c.u16[0xE8 + 2i] = 0, return item. Returns 0.
fn 021d3dc4 ret=u16
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 any
  fix arg0 0xE8 u16 alt const 0 || any rep=4,2
  stub 021b9b94 force ret=pick:0/1/2/3/6 args=v
  stub 021d62c4 ret=pick:0/0/1 args=v,v
  stub 021d805c ret=pick:0/1 args=v,v
  stub 021d59b4 ret=u16 args=v,v
  stub 021d7b60 ret=u16 args=v,v
  stub 021d3e70 ret=pick:0/1 args=v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021d5f14 ret=pick:0/1 args=v

# item r0 cures / boosts mon r1: for the 6 {kind, stat} pairs of the ROM table 0x021EFF94: 021d805c(item, kind) and 021d5d14(mon, stat, 1) -> 1;
# 021d805c(item, 0x24) and !021d5f14(mon) -> 1; else 0
fn 021d3e70 ret=u32
  nowork
  arg u32 any
  arg u32 any
  stub 021d805c ret=pick:0/0/0/1 args=v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021d5f14 ret=pick:0/1 args=v

# item r0 heals a status of mon r1: for the 7 {kind, flag} pairs of the ROM table 0x021EFFA0: 021d805c(item, kind) and 021d62c4(mon, flag) -> 1
fn 021d3ec8 ret=u32
  nowork
  arg u32 any
  arg u32 any
  stub 021d805c ret=pick:0/0/0/1 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v

# view-side accessor (called from overlay 95; found by the B13-spec2 reachability scan, not a label in battle_client.s): tail call to
# 021ef9a8(c+0x34) (`ldr r0, [r0, 0x34]; ldr r3, =0x021EF9A9; bx r3`; the callee returns (u8) *obj)
fn 021d3d84 ret=u32
  nowork
  arg buf 0x1C0
  stub 021ef9a8 ret=u8 args=v
