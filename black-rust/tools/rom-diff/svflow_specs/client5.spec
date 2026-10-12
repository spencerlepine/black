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
  stub 021b9934 ret=u32 args=v,v
  stub 021d5f90 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5fa4(mon, u16 at args[1]); returns 1
fn 021d36ec ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5fa4 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5608(mon, u8 args[1], u8 args[2]); returns 1
fn 021d370c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5608 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5640(mon, u8 args[1], u8 args[2]); returns 1
fn 021d3734 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5640 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d56cc(mon, u8 args[1]); returns 1
fn 021d375c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d56cc ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5fb8(mon, no extra argument); returns 1
fn 021d377c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5fb8 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5674(mon, u8 args[1], u8 args[2]); returns 1
fn 021d3794 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5674 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d56a0(mon, u8 args[1], u8 args[2]); returns 1
fn 021d37b8 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d56a0 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5dc0(mon, args[1], u8 args[2]); returns 1
fn 021d37dc ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5dc0 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5e38(mon, args[1], u8 args[2]); returns 1
fn 021d37fc ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5e38 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5efc(mon, no extra argument); returns 1
fn 021d3894 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5efc ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5f08(mon, no extra argument); returns 1
fn 021d38ac ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5f08 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5f38(mon, args[1]); returns 1
fn 021d38c4 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5f38 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6034(mon, args[1], args[2]); returns 1
fn 021d38e0 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6034 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d61f0(mon, no extra argument); returns 1
fn 021d3900 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d61f0 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6238(mon, args[1]); returns 1
fn 021d3918 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6238 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d66b0(mon, args[1]); returns 1
fn 021d3988 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d66b0 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6730(mon, args[1]); returns 1
fn 021d3a20 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6730 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6744(mon, args[1], args[2], args[3], args[4], args[5]); returns 1
fn 021d3a40 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6744 ret=none args=v,v,v,v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5fe4(mon, args[1]); returns 1
fn 021d3a7c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5fe4 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6008(mon, args[1]); returns 1
fn 021d3a98 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6008 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5fc0(mon, args[1]); returns 1
fn 021d3ab4 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5fc0 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6440(mon, args[1]); returns 1
fn 021d3ad0 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6440 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d66d0(mon, args[1]); returns 1
fn 021d3aec ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d66d0 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5b44(mon, args[1]); returns 1
fn 021d3b08 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5b44 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5734(mon, args[1], args[2], args[3], args[4]); returns 1
fn 021d3b28 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5734 ret=none args=v,v,v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d64f0(mon, no extra argument); returns 1
fn 021d3b5c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d64f0 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d69a0(mon, args[1], args[2]); returns 1
fn 021d3bc0 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d69a0 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6d5c(mon, args[1]); returns 1
fn 021d3c0c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6d5c ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6d70(mon, no extra argument); returns 1
fn 021d3c2c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6d70 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6738(mon, no extra argument); returns 1
fn 021d3c90 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6738 ret=none args=v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d6264(mon, u8 args[1]); returns 1
fn 021d3ca8 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6264 ret=none args=v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d5970(mon, args[1], args[2]); returns 1
fn 021d3d2c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5970 ret=none args=v,v,v

# view-command handler: BattleMon = 021b9934(pokecon, u8 args[0]); 1d67d8(mon, args[1]); returns 1
fn 021d3d50 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d67d8 ret=none args=v,v

# 7 BattleMon view calls 021d5ea8(mon, k, u8 args[k + 0]) for k = 1..7; returns 1
fn 021d381c ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
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
  stub 021b9934 ret=u32 args=v,v
  stub 021d65c8 ret=none args=v,v

# args[0] == client id (c[0x1A6]): counter += u8 args[1] (021cd9f4, client1) else 021b9b10(server, u8 args[0]); returns 1
fn 021d3c44 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  fix arg0 0x1A6 u8 range 0 3
  fix arg2 0 u32 alt range 0 3 || range 0x100 0x103 || any
  stub 021cd9f4 ret=none args=p,v
  stub 021b9b10 ret=none args=v,v

# x = 021b9aec(pokecon, u8 args[0]); 021b9d10(server, x, u8 args[0]); returns 1
fn 021d3c70 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9aec ret=u32 args=v,v
  stub 021b9d10 ret=none args=v,v,v

# 021d6848(mon, &{u16 args[4], u16 args[5], u8 args[3], u8 args[1], u8 args[2], pad}); returns 1
fn 021d3cc8 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d6848 ret=none args=v,in:7

# mon = 021b9934(pokecon, u8 args[0]); 021d6408(mon); 021d6e88(mon); returns 1
fn 021d3cfc ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
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
  stub 021b9f84 ret=u32 args=v,v,v
