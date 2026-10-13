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

# ---------------------------------------------------------------------------------------------------------------------------------- B13-spec2
# B13-spec2 specs every function the B13-spec1 header above lists as "not specced in this batch" (list in client/mod.rs).
# The four functions left by B13-spec1. Indirect calls (`blx` through c+0x28 / c+0xC8, the step function of the request table) go to
# `force` stubs at the target address: a port dispatches a function pointer through `env.call(ptr)`, so the ROM side must stop there too
# even once the target is ported. The pointer fields are fixed to step functions of the request table (0x021CDDDD = request 1 mode 0,
# 0x021CDF59 = request 2 mode 0). Adapter calls (021eea58 021eebe8 021eec20 021eec4c, all PORTED by B12) are `force` stubs.

# create(server, pokecon, r2, r3, [sp] client id, u16 [sp+4], mode [sp+8], [sp+0xC], [sp+0x10], 0x18-byte block* [sp+0x14], heap [sp+0x18]):
# c = Heap_AllocDebug(heap, 0x1B8, 1, "btl_client.c", 0x209) (here: arg 11), every field initialised, c+0x50 = adapter create
# 021eea58(r3, id, r2 != 0 && mode != 0, heap), c+0x118 = 021b9af8(pokecon, id), c+0x134 = Heap_AllocDebug(heap, 0xBC0, 1, .., 0x21A),
# c+0x34 = 021ba08c(server), 021d8230(c+0x2C), cursor init 021d3f04(c+0x18), 021d3d9c(c), c+0xF4 = the 0x18-byte block; mode 1 and
# [sp+0x10] == 0: c+0xF0 = AI create 02187f00 (overlay 96) else 0; c+0x112.. = 0 1 2 3; mode 0 and 021b90f8(server): c+0x10 =
# BattleRecord_New(heap) else 0. Returns c.
fn 021cd6f0 ret=ofs:arg11
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 any
  arg u32 alt range 0 3 || any
  arg u32 any
  arg u32 alt range 0 2 || any
  arg u32 any
  arg u32 alt const 0 || any
  arg buf 0x18
  arg u32 any
  arg buf 0x1C0
  stub 02030734 ret=ptr:arg11:0:1 args=v,v,v,v,v
  stub 021eea58 force ret=u32 args=v,v,v,v
  stub 021b9af8 ret=u32 args=v,v
  stub 021ba08c ret=u32 args=v
  stub 021d8230 ret=none args=p
  stub 021d3f04 ret=none args=p
  stub 021d3d9c force ret=none args=p
  stub 021b9fbc ret=u32 args=v,v
  stub 021ba820 ret=u32 args=v
  stub 02187f00 ret=u32 args=v,v,v,v,v
  stub 021b90f8 ret=pick:0/1 args=v
  stub 021ee5e4 ret=u32 args=v

# run the current step: (c->step)(c, c->step) (r1 is the pointer itself); returns its result
fn 021cd97c ret=u32
  nowork
  arg buf 0x1C0
  fix arg0 0x28 u32 oneof 0x021CDA19 0x021CDC0D
  stub 021cda18 force ret=pick:0/1 args=p,v
  stub 021cdc0c force ret=pick:0/1 args=p,v

# the main step (`.byte` block, 8-way jump table on the phase c[0x1A8]; table data 021cda3e-021cda4e): 021d3fdc(c, cursor) first.
# 0: code = 021eebe8(adapter); replay (mode 2) with flag bit 4 or 5 and no code -> 0; 021cdd34 (result clear); code 9: the
#    request pair c+0x2C/0x30 from 021eec20, phase 3; code: c+0xC8 = 021cdd50(c, code, &aux), aux ? (021d3fac(cursor) ? : phase 6)
#    : 021d3fc4(cursor); c+0xC8 ? phase 1 : phase 2 (c+0xCC = 0); no code: flag bit 1 -> 1
# 1: (c+0xC8)(c, &c+0xCC) done -> flag bit 1 -> 1, else phase 021d3f4c(cursor) ? 4 : 2
# 2: 021eec4c(adapter, c+0xD8, c+0xDC) -> phase 0; flag bit 1 -> 1      3: the same, phase 7
# 4: 021d3f60(cursor) == 1 -> phase 7, else (view: 02005ca4(1, 0x3E), 02006740()) phase 5      6: 021d3f4c -> phase 4
# 5: !02005e50(): n = 021d3fa8(cursor), 02005ca4(0, 1), 021ba73c(server, n), 02006734(0)      7: return 1; other phases 0
fn 021cda18 ret=u32
  nowork
  arg buf 0x1C0
  fix arg0 0x1A8 u8 alt range 0 7 || range 8 0xFF
  fix arg0 0x1A7 u8 alt const 2 || any
  fix arg0 0x54 u32 alt const 0 || any
  fix arg0 0xC8 u32 oneof 0x021CDDDD 0x021CDF59
  stub 021d3fdc ret=none args=p,p
  stub 021eebe8 force ret=pick:0/0/9/1/2/3/7 args=v
  stub 021eec20 force ret=none args=v,outp:0:0x1B8:1
  stub 021cdd50 ret=pick:0/0x021CDDDD/0x021CDF59 args=p,v,outm:4:1/0/0/0
  stub 021d3fac ret=pick:0/1 args=p
  stub 021d3fc4 ret=none args=p
  stub 021cdddc force ret=pick:0/1 args=p,p
  stub 021cdf58 force ret=pick:0/1 args=p,p
  stub 021d3f4c ret=pick:0/1 args=p
  stub 021eec4c force ret=pick:0/1 args=v,v,v
  stub 021d3f60 ret=pick:0/1/2 args=p
  stub 02005ca4 ret=none args=v,v
  stub 02006740 ret=none args=
  stub 02005e50 ret=pick:0/1 args=
  stub 021d3fa8 ret=u16 args=p
  stub 021ba73c ret=none args=v,v
  stub 02006734 ret=none args=v

# the wait-for-request step (`.byte` block, 6-way jump table on c[0x1A8]; table data 021cdc30-021cdc3c): 021d3fdc(c, cursor) first.
# 0: phase 1, then as 1.  1: 021d3f98(cursor) ? (view: 02005ca4(1, 1), 021eae24(view), 02005df8(0x1E), 021f88bc(0), 02006740()) phase 4
#    : code = 021eebe8(adapter); code: c+0xC8 = 021cdd50(c, code, 0), c+0xC8 ? phase 2 (c+0xCC = 0) : (021cdd34, phase 3)
# 2: (c+0xC8)(c, &c+0xCC) done -> phase 3, c+0xCC = 0, 021f8678() -> 021f88bc(1)      3: 021eec4c(adapter, c+0xD8, c+0xDC) -> phase 1
# 4: view and 021eae34(view) and !02005e50(): n = (u16) 021d3fa8(cursor), 021eae74(view, n, n), 021ba7f8(server)      5: return 1
fn 021cdc0c ret=u32 inputs=600
  nowork
  arg buf 0x1C0
  fix arg0 0x1A8 u8 alt range 0 5 || range 6 0xFF
  fix arg0 0x54 u32 alt const 0 || any
  fix arg0 0xC8 u32 oneof 0x021CDDDD 0x021CDF59
  stub 021d3fdc ret=none args=p,p
  stub 021d3f98 ret=pick:0/1 args=p
  stub 021eebe8 force ret=pick:0/0/1/2/3 args=v
  stub 021cdd50 ret=pick:0/0x021CDDDD/0x021CDF59 args=p,v,v
  stub 021cdddc force ret=pick:0/1 args=p,p
  stub 021cdf58 force ret=pick:0/1 args=p,p
  stub 02005ca4 ret=none args=v,v
  stub 021eae24 ret=none args=v
  stub 02005df8 ret=none args=v
  stub 021f88bc ret=none args=v
  stub 02006740 ret=none args=
  stub 021f8678 ret=pick:0/1 args=
  stub 021eec4c force ret=pick:0/1 args=v,v,v
  stub 021eae34 ret=pick:0/1 args=v
  stub 02005e50 ret=pick:0/1 args=
  stub 021d3fa8 ret=u16 args=p
  stub 021eae74 ret=none args=v,v,v
  stub 021ba7f8 ret=none args=v
