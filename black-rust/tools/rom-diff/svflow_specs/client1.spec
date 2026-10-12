# svflow spec: client1 (btl_client.c part 1, 0x021CD6EC-0x021CDDDC: client object lifecycle and accessors; B13-spec1). Harness-job
# owned; porting workers may not edit. Grammar: svflow_specs/README.md. The area split of btl_client.c is documented in the doc comment
# of crates/black-battle/src/client/mod.rs.
#
# The client object ("c", 0x1B8 bytes from Heap_AllocDebug in the create routine 021cd6f0; `arg buf 0x1C0` here, `nowork`): the object is
# not an SvFlowWork, so every function of the area is `nowork` and the object is the first buffer argument. Fields seen so far:
#   +0x000 server-side object (u32, handed to the main-module getters 021b8588 / 021b86c0 / 021b9064 ...)   +0x004 pokecon (u32)
#   +0x010 BattleRecord* (0 = no recording)   +0x014 BattleRecordReader*   +0x018 command cursor (0x10 bytes, 021d3f04-021d3fd0)
#   +0x028 current step function (Thumb pointer; 0x021CDC0D = the idle/"wait for next request" step, 0x021CDA19 = the main step)
#   +0x02C/+0x030 request argument pair   +0x050 adapter channel object (btl_adapter, D7 seam)   +0x054 view handle (battle_view_core)
#   +0x0A0 u32   +0x0D0 pending callback / +0x0D4 its state word   +0x0D8 result pointer / +0x0DC result size   +0x0E0 result word
#   +0x0E4 u16 / +0x0E6 u16 (BGM / SE ids of the request)   +0x134 heap block   +0x19E u16   +0x1A6 client id   +0x1A7 mode (0 normal, 2 replay)
#   +0x1A8 step phase   +0x1AB u8 counter (<= 14)   +0x1B2 u8 flag bits (1 = ?, 2 = request done, 4, 8, 0x10, 0x20)
# Table in ROM read by 021cdd50 (a real data table; ports embed it): 0x021EFFEC, 15 records of 16 bytes {u8 request code, u8 aux,
# 2 pad, fn mode0, fn mode1, fn mode2} (codes 1 2 3 7 5 6 4 8 0xA 0xB 0xC 0xE 0xF 0xD 0x10), indexed by mode (c+0x1A7).
#
# Stub conventions (core4/core5/adapter): every callee outside this area is a stub with the argument kinds of its registers; callees
# in the area that are not ported are stubs too (021cda0c). Callees that are ported elsewhere (adapter smoke ports 021eebdc ...) are
# `force` stubs: the Rust ports reach them through `env.call`.
#
# Not specced in this batch (see the doc comment of client/mod.rs): 021cd6f0 (create, 0x1F0 bytes), 021cd97c (indirect call through
# c+0x28), 021cda18 (the main step, a `.byte` block with a jump table).
area client1

# empty (bx lr)
fn 021cd6ec ret=none
  nowork

# free: BattleRecord_Free(c->record) if set, 02188080() if c+0xF0 is set, Heap_Free(c+0x134), the adapter object (thunk 021eeae0 ->
# Heap_Free), Heap_Free(c)
fn 021cd8e0 ret=none
  nowork
  arg buf 0x1C0
  fix arg0 0x10 u32 alt const 0 || any
  fix arg0 0xF0 u32 alt const 0 || any
  stub 021ee600 ret=none args=v
  stub 02188080 ret=none args=
  stub 020307b0 ret=none args=v

# flags |= 2 (request done)
fn 021cd914 ret=none
  nowork
  arg buf 0x1C0

# BattleRecord_GetData(c->record) or 0
fn 021cd924 ret=u32
  nowork
  arg buf 0x1C0
  fix arg0 0x10 u32 alt const 0 || any
  stub 021ee63c ret=u32 args=v

# set the replay reader: mode = 2, reader pointer at +0x14, cursor init with the reader's size
fn 021cd938 ret=none
  nowork
  arg buf 0x1C0
  arg u32 any
  stub 021ee830 ret=u32 args=v
  stub 021d3f48 ret=none args=p,v

# c->view = r1
fn 021cd95c ret=none
  nowork
  arg buf 0x1C0
  arg u32 any

# c+0xA0 = r1, flags |= 8
fn 021cd960 ret=none
  nowork
  arg buf 0x1C0
  arg u32 any

# adapter object
fn 021cd978 ret=u32
  nowork
  arg buf 0x1C0

# reset the replay: reader reset, adapter block reset (021eebdc, adapter smoke port: force), cursor reset(r1), step = idle (021cdc0d)
fn 021cd984 ret=none
  nowork
  arg buf 0x1C0
  arg u32 any
  stub 021ee674 ret=none args=v
  stub 021eebdc ret=none args=v force
  stub 021d3f64 ret=none args=p,v
  stub 021cda0c ret=none args=p,v

# cursor reset, step = main step (021cda19)
fn 021cd9b0 ret=none
  nowork
  arg buf 0x1C0
  stub 021d3f88 ret=none args=p
  stub 021cda0c ret=none args=p,v

# mode == 2
fn 021cd9c8 ret=bool
  nowork
  arg buf 0x1C0
  fix arg0 0x1A7 u8 alt const 2 || any

# halfword at +0x22 of the cursor block
fn 021cd9dc ret=u16
  nowork
  arg buf 0x1C0

# step == idle step (0x021CDC0D)
fn 021cd9e0 ret=bool
  nowork
  arg buf 0x1C0
  fix arg0 0x28 u32 alt const 0x021CDC0D || any

# counter += r1, clamped to 14
fn 021cd9f4 ret=none
  nowork
  arg buf 0x1C0
  arg u32 any
  fix arg0 0x1AB u8 alt range 0 14 || any

# step = r1, phase = 0
fn 021cda0c ret=none
  nowork
  arg buf 0x1C0
  arg u32 any

# result: word +0xE0 = 0, result pointer +0xD8 = &c[0xE0], size +0xDC = 4
fn 021cdd34 ret=none
  nowork
  arg buf 0x1C0

# request code -> handler of the current mode (table 0x021EFFEC); *out = the record's aux byte
fn 021cdd50 ret=u32
  nowork
  region out 4
  arg buf 0x1C0
  arg u32 alt range 0 0x12 || any
  arg u32 alt const 0 || addrof region:out
  fix arg0 0x1A7 u8 range 0 2

# out = (c+0x2C, c+0x30)
fn 021cdd94 ret=none
  nowork
  arg buf 0x1C0
  arg buf 0x10

# replay: bit 4 of the flags; else (c+0xE6 != 0) -> 021f8524(0), else 0
fn 021cdda0 ret=u32
  nowork
  arg buf 0x1C0
  fix arg0 0x1A7 u8 alt const 2 || any
  fix arg0 0xE6 u16 alt const 0 || any
  stub 021f8524 ret=u32 args=v

# bit 5 of the flags
fn 021cddcc ret=bool
  nowork
  arg buf 0x1C0
