# svflow spec: client4 (btl_client.c part 4, 0x021D10B8-0x021D2FD8: playback-table command handlers (view/sound/effect relays); B13-spec1).
# Harness-job owned; porting workers may not edit. Grammar: svflow_specs/README.md. Area split: doc comment of client/mod.rs.
#
# What this area is: handlers of the playback table at 0x021F00DC.. (pairs {command id 0x30-0x5D, Thumb fn}; fn 021d1e6d ... 021d2fd9) and the request
# steps 021d10b9 ... 021d1a75 of the table at 0x021EFFEC (see client1.spec). Both families call into the battle view (battle_view_core,
# 021ea000-021eb000), the sound/message layer (0200xxxx) and the BattleMon API; step functions have the shape
# `u32 step(client c, u32 *state)` (state machine on `*state`, returns 1 when finished), command handlers `u32 handler(c, u32 unused, u32 *args)`.
# Only the leaf-most functions are specced in this batch: 021d2990 (relay of a sound id), 021d299c (two-state step), 021d2c7c (message-window
# poll by id). Everything else in the range (state machines with 4-12 view calls each, 021d10b8 ... 021d2fd8) is for B13-spec2.
# Same client object as client1.spec (`arg buf 0x1C0`, `nowork`); state words are `arg buf 0x10`.
area client4

# 021f878c(args[0]); returns 1 (r0, r1 unused)
fn 021d2990 ret=u32
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 021f878c ret=none args=v

# two-state step: state 0: 021ea8e4(c.view), state = 1, return 0; state 1: return 021ea8f4(c.view) != 0 (1) else 0; any other state: 0
fn 021d299c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 2 || any
  stub 021ea8e4 ret=none args=v
  stub 021ea8f4 ret=pick:0/1 args=v

# id 3: 020061e4(0x56B) (no result), id 5: 02006254(0x56B) != 0 -> 1, else 0
fn 021d2c7c ret=u32
  nowork
  arg u32 alt range 0 6 || any
  stub 020061e4 ret=none args=v
  stub 02006254 ret=pick:0/1 args=v

# ---------------------------------------------------------------------------------------------------------------------------------- B13-spec2
# B13-spec2 specs every function the B13-spec1 header above lists as "not specced in this batch" (list in client/mod.rs).
# The request steps 021d10b8 .. 021d1a74 (requests 4, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF, 0x10, 8; `u32 step(c, u32 *state)`, arg 1 =
# `arg buf 0x10`) and the playback handlers 021d1bd0 .. 021d2ca8 of the table 0x021F00DC (`u32 handler(c, u32 *state, u32 *args)`: arg 1
# the state word, arg 2 the command arguments as words, `arg buf 0x40`; each returns 1 when the command is played). Argument masks (u8 /
# u16 of args[i]) are the port's job.
# Adapter calls (021eec20 data pointer + size) are `force` stubs (adapter smoke ports); the data they return lives in an extra buffer
# argument that the function itself never reads as a register (its r2 / r3 is overwritten first), through `outp:K:0:1`.
# 021d1a74 (request 8, the command player): the command queue is c+0x134 (pointer, fixed to an extra buffer), filled from the adapter data
# with MI_CpuCopy32 02082a90 (real code on both sides, svflow ALLOW), read by the cmd_queue port 021cd5e0 (force stub here: verified by the
# cmd_queue area) and dispatched through the playback table (ROM data) to `force` stubs of two handlers (0x021D2031, 0x021D20BD).
# 021d215c pops bytes through the thunk 021cd644 -> cmd_queue 021ccad4 (PORTED, real code on both sides) from the queue at c+0x134.
# 021d2a20 / 021d2ca8 (learn a move after a level-up) keep their state in the overlay globals 0x021F3B64-0x021F3B7C (`global 021f3b64
# 0x20`; 021d2ca8's state word is 0x021F3B6C when 021d2a20 runs it).
# Black-pokemon accessors used on the Pokemon of a BattleMon (02017e1c get, 02017e40 set, 020181f4, 02018328, 0201bd28) are stubs.
# Veneers 021d121c / 021d122c are stubbed at the Thumb entry. Server queries (021b8xxx / 021b9xxx), view calls (021e9xxx / 021eaxxx), sound
# (0200xxxx, overlay 94 021f7xxx / 021f8xxx) and the UI-side object c+0x34 (021ef9c0 / 021ef9c8) are stubs.

# request 4, mode 0 (6-way jump table, data 021d10d4-021d10e0): 0: p = adapter data (021eec20); p[0] == 0x1F -> state 4; else message 0x14
# (021eaf18(c+0x58, 1, 0x14), 021eaf50(c+0x58, 1), 021eaf50(c+0x58, p[0]), 021eaf78(c+0x58, 0xFF), 021ea614(view, c+0x58)), state 1.
# 1: 021ea710(view), 021ea720(view) -> yes/no (021eaf18 x2, 021eabc4(view, c+0x58, c+0x7C, 1)), state 2. 2: 021ea710 and 021eac54(view,
# &ans) -> ans ? state 4 : (021d0b10(c, 0, 1, c+0x184, c+0x190), 021e9a78(view, c+0x184, 0, 0, c+0x190), state 3). 3: 021e9b20(view) ->
# veneer 021d121c(c+0x190) ? state 4 : n = veneer 021d122c(c+0x190), 0 < n < 6 -> 021d8394(c+0x124, 0, n), state 5, else state 4.
# 4: 021d8434(c+0x124), state 5, result = c+0x124, size 4, return 1. 5: the same result, return 1.
fn 021d10b8 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x10
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 0 u8 alt const 0x1F || any
  stub 021eec20 force ret=u32 args=v,outp:2:0:1
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021eaf78 ret=none args=p,v
  stub 021ea614 ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v
  stub 021ea720 ret=pick:0/1 args=v
  stub 021eabc4 ret=none args=v,p,p,v
  stub 021eac54 ret=pick:0/1 args=v,outm:4:1/0/0/0
  stub 021d0b10 force ret=none args=p,v,v,p,p
  stub 021e9a78 ret=none args=v,p,v,v,p
  stub 021e9b20 ret=pick:0/1 args=v
  stub 021d121c ret=pick:0/1 args=p
  stub 021d122c ret=pick:0/1/5/6 args=p
  stub 021d8394 ret=none args=p,v,v
  stub 021d8434 ret=none args=p

# request 0xA, mode 0 (record the server data): size = 021eec20(adapter, &p); recording (c+0x10) -> BattleRecord_Append(rec, p, size);
# recorder c+0xA0 -> 021bb4e0(c+0xA0, p, size) (server_flow port: force), c[0x1AE] = 021ee638(p). Returns 1. (r1 is unused; arg 1 holds
# the adapter data.)
fn 021d123c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg0 0x10 u32 alt const 0 || any
  fix arg0 0xA0 u32 alt const 0 || any
  stub 021eec20 force ret=u32 args=v,outp:1:0:1
  stub 021ee608 ret=none args=v,v,v
  stub 021bb4e0 force ret=none args=v,v,v
  stub 021ee638 ret=u8 args=v

# request 0xD, mode 2: view set -> 021d1290(c, state, view), else 1 (arg 2 holds the adapter data of 021d13a4)
fn 021d127c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x10
  fix arg0 0x54 u32 alt const 0 || any
  fix arg1 0 u32 alt range 0 3 || any
  fix arg2 2 u16 alt range 0 2 || any
  stub 021d1290 ret=pick:0/1 args=p,p,v
  stub 021eec20 force ret=u32 args=v,outp:2:0:1
  stub 02005d28 ret=none args=
  stub 02005d90 ret=none args=v,v
  stub 02005e30 ret=none args=v
  stub 02005e50 ret=pick:0/1 args=
  stub 021b86c8 force ret=pick:0/1/0x100 args=v
  stub 021b86dc force ret=u32 args=v
  stub 021b902c ret=pick:0/1 args=v,v
  stub 021b904c ret=u32 args=v,v
  stub 021b916c ret=none args=v,v
  stub 021d13a4 ret=pick:0/1/2/3 args=p
  stub 021ea614 ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v

# request 0xD, mode 0 (4-way jump table, data 021d12a8-021d12b0): 0: flag bit 5 -> 1; id = 021b904c(server, 0), r = 021d13a4(c), m =
# (u8) 021b86c8(server), 021b916c(server, r); r 0 / 1 / 2 -> message 0x33|0x32 / 0x31|0x30 / 0x35|0x34 (by m) with id (and 021b904c(server,
# 1) when m), 021ea614(view, c+0x58); r 1 -> 02005d90(021b86dc(server), 0xFFFF), state 1, else 02005e30(0x1E), state 2; other r -> 1.
# 1: 021ea710(view) -> 1. 2: 021ea710 -> state 3. 3: !02005e50() -> 02005d28(), 1. (r2 is unused; arg 2 holds the adapter data.)
fn 021d1290 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x10
  fix arg1 0 u32 alt range 0 3 || any
  fix arg2 2 u16 alt range 0 2 || any
  stub 021b904c ret=u32 args=v,v
  stub 021d13a4 ret=pick:0/1/2/3 args=p
  stub 021eec20 force ret=u32 args=v,outp:2:0:1
  stub 021b86c8 force ret=pick:0/1/0x100 args=v
  stub 021b916c ret=none args=v,v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021ea614 ret=none args=v,p
  stub 021b86dc force ret=u32 args=v
  stub 02005d90 ret=none args=v,v
  stub 02005e30 ret=none args=v
  stub 021ea710 ret=pick:0/1 args=v
  stub 02005e50 ret=pick:0/1 args=
  stub 02005d28 ret=none args=
  stub 021b902c ret=pick:0/1 args=v,v

# the battle result from the adapter data p (021eec20): u16 p[1] 0 -> 021b902c((u8) p[0], c[0x1A6]) ? p[1] : 1; 1 -> 021b902c(..) ? p[1] :
# 0; else p[1]. (r1 is unused; arg 1 holds the adapter data.)
fn 021d13a4 ret=u16
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 2 u16 alt range 0 2 || any
  stub 021eec20 force ret=u32 args=v,outp:1:0:1
  stub 021b902c ret=pick:0/1 args=v,v

# request 0xB, mode 2: view set -> 021d1410(c, state, view), else 1
fn 021d13fc ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x10
  fix arg0 0x54 u32 alt const 0 || any
  fix arg1 0 u32 alt range 0 11 || any
  fix arg2 2 u16 alt range 0 2 || any
  stub 021d1410 ret=pick:0/1 args=p,p,v
  stub 021eec20 force ret=u32 args=v,outp:2:0:1
  stub 021b86dc force ret=u32 args=v
  stub 021b8ff4 force ret=u32 args=v
  stub 021b902c ret=pick:0/1 args=v,v
  stub 021b904c ret=pick:0/1 args=v,v
  stub 021b919c ret=nullable:2:u32 args=v
  stub 021b9210 ret=nullable:2:u32 args=v
  stub 021b9244 ret=nullable:2:u32 args=v
  stub 021ba004 ret=u32 args=v,v
  stub 021ba020 ret=u32 args=v,v
  stub 021ba1a8 ret=pick:0/1 args=v,v
  stub 021d13a4 ret=pick:0/1/1/2 args=p
  stub 021ea614 ret=none args=v,p
  stub 021ea6b0 ret=none args=v,v,v
  stub 021ea710 ret=pick:0/1 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021f7998 ret=none args=v
  stub 021f7eb4 ret=pick:0/1 args=
  stub 021f7f58 ret=none args=v,v,v,v,v
  stub 021f878c ret=none args=v

# request 0xB, mode 0 (battle end; 12-way jump table, data 021d142a-021d1442): 0: 021d13a4(c) == 1 -> win: 021f878c(021b86dc(server)),
# message 0x2C (one trainer) / 0x2D (two) with the 021b904c(server, 0 / 1) ids, state 1; else 021ba1a8(server, 0x100) (thunk; stubbed at
# its address) ? return 1 : message 0x36 with 021b8ff4(server), state 9. 1: 021ea710 -> 021f7f58(021ba020(server, 021b904c(server, 0)), 9,
# 0, 0, 0), 021f7998(0x270), state 2. 2: !021f7eb4() -> 021ea6b0(view, 021ba004(server, id0), 1), state 3. 3: 021ea710 -> two trainers ?
# (021f7998(0x271), state 4) : state 7. 4: !021f7eb4() -> 021f7f58 (second trainer), 021f7998(0x270), state 5. 5: !021f7eb4() ->
# 021ea6b0(view, 021ba004(server, id1), 1), state 6. 6: 021ea710 -> state 7. 7: money 021b919c(server) -> message 0x3A (021b8ff4, money);
# 021b9210(server) ? state 8 : state 0xB. 8: 021ea710 -> 021b9210 -> message 0x3B, state 0xB. 9: 021ea710 -> 021b9244(server) -> message
# 0x38 (client id, value), state 0xA. 0xA: 021ea710 -> message 0x39, state 0xB. 0xB: 021ea710 -> 1. (r2 unused; arg 2 = adapter data.)
fn 021d1410 ret=u32 inputs=400
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x10
  fix arg1 0 u32 alt range 0 11 || any
  fix arg2 2 u16 alt range 0 2 || any
  stub 021d13a4 ret=pick:0/1/1/2 args=p
  stub 021eec20 force ret=u32 args=v,outp:2:0:1
  stub 021b86dc force ret=u32 args=v
  stub 021f878c ret=none args=v
  stub 021b904c ret=pick:0/1 args=v,v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021ea614 ret=none args=v,p
  stub 021ba1a8 ret=pick:0/1 args=v,v
  stub 021b8ff4 force ret=u32 args=v
  stub 021ea710 ret=pick:0/1 args=v
  stub 021ba020 ret=u32 args=v,v
  stub 021f7f58 ret=none args=v,v,v,v,v
  stub 021f7998 ret=none args=v
  stub 021f7eb4 ret=pick:0/1 args=
  stub 021ba004 ret=u32 args=v,v
  stub 021ea6b0 ret=none args=v,v,v
  stub 021b919c ret=nullable:2:u32 args=v
  stub 021b9210 ret=nullable:2:u32 args=v
  stub 021b9244 ret=nullable:2:u32 args=v
  stub 021b902c ret=pick:0/1 args=v,v

# request 0xC, mode 2: view set -> 021d171c(c, state, view), else 1
fn 021d1708 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x10
  fix arg0 0x54 u32 alt const 0 || any
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 2 u16 alt range 0 2 || any
  stub 021d171c ret=pick:0/1 args=p,p,v
  stub 021eec20 force ret=u32 args=v,outp:2:0:1
  stub 0202435c ret=u32 args=v,v
  stub 02024450 ret=pick:0/1 args=v
  stub 02045808 ret=none args=v
  stub 021b86dc force ret=u32 args=v
  stub 021b902c ret=pick:0/1 args=v,v
  stub 021b904c ret=pick:0/1 args=v,v
  stub 021b916c ret=none args=v,v
  stub 021ba004 ret=nullable:2:u32 args=v,v
  stub 021ba020 ret=u32 args=v,v
  stub 021ba02c ret=nullable:2:u32 args=v,v,v
  stub 021d13a4 ret=pick:0/1/2 args=p
  stub 021d185c ret=none args=p,v,v
  stub 021ea5e4 ret=none args=v,v
  stub 021ea5f0 ret=none args=v,p
  stub 021ea600 ret=none args=v
  stub 021ea710 ret=pick:0/1 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021f7998 ret=none args=v
  stub 021f7eb4 ret=pick:0/1 args=
  stub 021f7f58 ret=none args=v,v,v,v,v
  stub 021f878c ret=none args=v

# request 0xC, mode 0 (trainer intro / outro; 6-way jump table, data 021d1736-021d1742): 0: flag bit 5 -> 1; r = 021d13a4(c),
# 021b916c(server, r); r 1 -> 021f878c(021b86dc(server)); r <= 1: 021f7f58(021ba020(server, 021b904c(server, 0)), 9, 0, 0, 0),
# 021f7998(0x270), 021d185c(c, r, 0), state 1; else 1. 1: !021f7eb4() -> 021ea600(view), state 2. 2: 021ea710 -> one trainer -> 1, else
# 021f7998(0x271), state 3. 3: !021f7eb4() -> r = 021d13a4(c), 021f7f58 (trainer 1), 021f7998(0x270), 021d185c(c, r, 1), state 4.
# 4: !021f7eb4() -> 021ea600(view), state 5. 5: 021ea710 -> 1. (r2 unused; arg 2 = adapter data.)
fn 021d171c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x10
  fix arg1 0 u32 alt range 0 5 || any
  fix arg2 2 u16 alt range 0 2 || any
  stub 021d13a4 ret=pick:0/1/2 args=p
  stub 021eec20 force ret=u32 args=v,outp:2:0:1
  stub 021b916c ret=none args=v,v
  stub 021b86dc force ret=u32 args=v
  stub 021f878c ret=none args=v
  stub 021b904c ret=pick:0/1 args=v,v
  stub 021ba020 ret=u32 args=v,v
  stub 021f7f58 ret=none args=v,v,v,v,v
  stub 021f7998 ret=none args=v
  stub 021d185c ret=none args=p,v,v
  stub 021f7eb4 ret=pick:0/1 args=
  stub 021ea600 ret=none args=v
  stub 021ea710 ret=pick:0/1 args=v
  stub 0202435c ret=u32 args=v,v
  stub 02024450 ret=pick:0/1 args=v
  stub 02045808 ret=none args=v
  stub 021b902c ret=pick:0/1 args=v,v
  stub 021ba004 ret=nullable:2:u32 args=v,v
  stub 021ba02c ret=nullable:2:u32 args=v,v,v
  stub 021ea5e4 ret=none args=v,v
  stub 021ea5f0 ret=none args=v,p
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v

# trainer message: id = 021b904c(server, r2), t = 021ba02c(server, id, r1); t and 02024450(t): s = 0202435c(t, (c.u16[0x19A] & 0x7FFF) |
# 0x8000), 021ea5e4(view, s), 02045808(s); else n = 021ba004(server, id): message 7/0 with n and r1 (021eaf18 / 021eaf50 x2),
# 021ea5f0(view, c+0x58)
fn 021d185c ret=none
  nowork
  arg buf 0x1C0
  arg u32 any
  arg u32 alt range 0 1 || any
  stub 021b904c ret=u32 args=v,v
  stub 021ba02c ret=nullable:2:u32 args=v,v,v
  stub 02024450 ret=pick:0/1 args=v
  stub 0202435c ret=u32 args=v,v
  stub 021ea5e4 ret=none args=v,v
  stub 02045808 ret=none args=v
  stub 021ba004 ret=nullable:2:u32 args=v,v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021ea5f0 ret=none args=v,p

# request 0xE, mode 0: 0: !021ba1a8(server, 0x200) and money 021b9210(server) -> message 0x3B (021b8ff4, money), 021ea614; state 1, then as
# 1. 1: 021ea710(view) -> 1.
fn 021d18ec ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 1 || any
  stub 021ba1a8 ret=pick:0/1 args=v,v
  stub 021b9210 ret=nullable:2:u32 args=v
  stub 021b8ff4 force ret=u32 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021ea614 ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v

# request 0xF, mode 0 (4-way jump table, data 021d1978-021d1980): 0: message 0x36 with c[0x1A6], 021ea614, state 1. 1: 021ea710 ->
# 021b9244(server) -> message 0x37 (client id, value); state 2. 2: 021ea710 -> message 0x39 with c[0x1A6], state 3. 3: 021ea710 -> state 4.
# Other states: 1.
fn 021d1960 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 4 || any
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021ea614 ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v
  stub 021b9244 ret=nullable:2:u32 args=v

# request 0x10, mode 0: 0: message 0xB7 (021eaf18(c+0x58, 1, 0xB7), 021ea614(view, c+0x58)), state 1. 1: 021ea710 -> state 2. 2: 1.
fn 021d1a18 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  fix arg1 0 u32 alt range 0 2 || any
  stub 021eaf18 ret=none args=p,v,v
  stub 021ea614 ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v

# request 8, mode 2: view set -> 021d1a74(c, state, view), else 1
fn 021d1a60 ret=u32 inputs=300
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x100
  arg buf 0x100
  fix arg0 0x54 u32 alt const 0 || any
  fix arg0 0x134 u32 addrof arg2 0
  fix arg0 0x17C u32 oneof 0x021D2031 0x021D20BD
  fix arg1 0 u32 alt range 0 4 || any
  stub 021d1a74 ret=pick:0/1 args=p,p,v
  stub 021eec20 force ret=pick:0/8/0x20/0xF0 args=v,outp:3:0:1
  stub 021cd5e0 force seq ret=pick:0x30/0x31/0x5E/0x99 args=v,p
  stub 021d2030 force seq ret=pick:0/1 args=p,p,p
  stub 021d20bc force seq ret=pick:0/1 args=p,p,p
  stub 021b9184 ret=none args=v
  stub 021bb570 force ret=pick:0/1 args=v,v,v,v
  stub 021d3f4c ret=pick:0/0/1 args=p
  stub 021ea824 ret=none args=v

# request 8, mode 0: the command player (5-way jump table, data 021d1a90-021d1a9a; arg 2 = the queue c+0x134, arg 3 = the adapter data).
# 0: size = 021eec20(adapter, &p); recorder c+0xA0, c[0x1AE] and flag bit 3: 021bb570(c+0xA0, c[0x1AE], p, size) (server_flow port: force)
# -> 021b9184(server), flags &= ~8; c[0x1AE] = 0; MI_CpuCopy32(p, q+8, (u16) size), q[0] = size, q[4] = 0; c[0x1A9] -> c[0x1A9] = 0,
# 021ea824(view); state 1. 1: q[4] == q[0] -> state 4, return 1; 021d3f4c(cursor) -> 1; state 2; cmd = 021cd5e0(q, c+0x138) -> c+0x178;
# 0x5E -> 1; found in the playback table (0x5D entries {id, fn}): c+0x17C = fn, c+0x180 = 0, state 3, then (c+0x17C)(c, c+0x180, c+0x138)
# done -> state 1 and again; not found -> state 1 and again. 3: the same handler call. 4: 1.
fn 021d1a74 ret=u32 inputs=300
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x100
  arg buf 0x100
  fix arg0 0x134 u32 addrof arg2 0
  fix arg0 0xA0 u32 alt const 0 || any
  fix arg0 0x1AE u8 alt const 0 || any
  fix arg0 0x1A9 u8 alt const 0 || any
  fix arg0 0x17C u32 oneof 0x021D2031 0x021D20BD
  fix arg1 0 u32 alt range 0 4 || any
  fix arg2 4 u32 alt const 0 || any
  stub 021eec20 force ret=pick:0/8/0x20/0xF0 args=v,outp:3:0:1
  stub 021bb570 force ret=pick:0/1 args=v,v,v,v
  stub 021b9184 ret=none args=v
  stub 021ea824 ret=none args=v
  stub 021d3f4c ret=pick:0/0/1 args=p
  stub 021cd5e0 force seq ret=pick:0x30/0x31/0x5E/0x99 args=v,p
  stub 021d2030 force seq ret=pick:0/1 args=p,p,p
  stub 021d20bc force seq ret=pick:0/1 args=p,p,p

# command 0x3B (HP-bar message): 0: id = 021d1c40(c, (u8) args[0], &f); message id (021eaf18(c+0x58, 1, id)), f -> 021eaf50(c+0x58,
# args[0]), 021eaf50(c+0x58, args[1]), 021ea614(view, c+0x58), state 1. 1: 021ea710(view) -> 1.
fn 021d1bd0 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  fix arg2 0 u8 range 0 3
  fix arg0 0x1A6 u8 range 0 3
  stub 021d1c40 ret=pick:0x19/0x1A/0x1E args=p,v,outm:4:1/0/0/0
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021ea614 ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v
  stub 021b8588 force ret=pick:0/0/0/1 args=v
  stub 021b86c0 force ret=pick:0/0/0/1 args=v
  stub 021b9f70 ret=pick:0/1 args=v,v
  stub 021cdf1c force ret=nullable:4:u32 args=p
  stub 021d59b4 ret=pick:100/100/150/60 args=v,v

# message id for client r1: own client (c[0x1A6]): *out = 0; battle type or the multi flag -> 0x1A; HP loss 021cdee8(c) (client2 smoke port,
# real code: its stubs are listed) >= 75 -> 0x1D, > 50 -> 0x1C, > 25 -> 0x1B, != 0 -> 0x1A, 0 -> 0x19; another client: *out = 1,
# 021b9f70(server, r1) ? 0x1E : 0x1F
fn 021d1c40 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 range 0 3
  arg buf 0x10
  fix arg0 0x1A6 u8 range 0 3
  fix arg0 0x19C u16 range 0 200
  stub 021b8588 force ret=pick:0/0/0/1 args=v
  stub 021b86c0 force ret=pick:0/0/0/1 args=v
  stub 021cdf1c force ret=nullable:4:u32 args=p
  stub 021d59b4 ret=pick:100/100/150/60 args=v,v
  stub 021b9f70 ret=pick:0/1 args=v,v

# command 0x3C: 0: 021ea458(view, 021b8e70(server, (u8) args[0]), (u16) args[1]), state 1. 1: 021ea478(view) -> 1.
fn 021d1cac ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8e70 force ret=u32 args=v,v
  stub 021ea458 ret=none args=v,v,v
  stub 021ea478 ret=pick:0/1 args=v

# command 0x3D (a = u8 args[0], b = u8 args[1], u8 args[3] = show message): 0: args[3]: n = 021d5440(021b9920(pokecon, a, b)); message
# 021b8ce8(server, c[0x1A6], a) ? (021b9f70(server, a) ? 0xE + a : 0x11 + a) : (a == c[0x1A6] ? 021d1df4(c) : 0x11 + a), then n,
# 021ea614; state 1. 1: 021ea710 -> 021ea488(view, 021b8bec(server, a, b), a, b), state 2. 2: 021ea4c4(view) -> 021cdecc(c), 1.
fn 021d1cf4 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  fix arg2 0 u8 range 0 3
  fix arg2 0xC u8 alt const 0 || any
  fix arg0 0x1A6 u8 range 0 3
  stub 021b9920 ret=u32 args=v,v,v
  stub 021d5440 ret=u32 args=v
  stub 021b8ce8 force ret=pick:0/1 args=v,v,v
  stub 021d1df4 ret=pick:0xB/0x15/0x18 args=p
  stub 021eaf18 ret=none args=p,v,v
  stub 021b9f70 ret=pick:0/1 args=v,v
  stub 021eaf50 ret=none args=p,v
  stub 021ea614 ret=none args=v,p
  stub 021ea710 ret=pick:0/1 args=v
  stub 021b8bec force ret=u32 args=v,v,v
  stub 021ea488 ret=none args=v,v,v,v
  stub 021ea4c4 ret=pick:0/1 args=v
  stub 021cdecc force ret=none args=p
  stub 021b8588 force ret=pick:0/0/1 args=v
  stub 021b9064 ret=u32 args=v,v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d5c44 ret=pick:0x4D800/0x37000/0x36FFF/0x20800/0xA000/0x9FFF args=v

# HP-state message of the own active mon: battle type set -> 0xB; mon = 021b9920(pokecon, (u8) 021b9064(server, c[0x1A6], 0), 0); dead
# (021d5b68) -> 0xB; ratio 021d5c44(mon) >= 0x4D800 -> 0xB, >= 0x37000 -> 0x15, >= 0x20800 -> 0x16, >= 0xA000 -> 0x17, else 0x18
fn 021d1df4 ret=u32
  nowork
  arg buf 0x1C0
  stub 021b8588 force ret=pick:0/0/1 args=v
  stub 021b9064 ret=u32 args=v,v,v
  stub 021b9920 ret=u32 args=v,v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d5c44 ret=pick:0x4D800/0x37000/0x36FFF/0x20800/0xA000/0x9FFF args=v

# command 0x5A (message): 0: idle (021cd9e0) -> 1; 021ea678(view, (u16) args[0], &args[1]), state 1. 1: 021ea710(view) -> 1.
fn 021d1e6c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021ea678 ret=none args=v,v,p
  stub 021ea710 ret=pick:0/1 args=v

# command 0x5C (message + SE): 0: idle -> 1; 021ea678(view, (u16) args[0], &args[2]), state 1. 1: 021ea720(view) -> 020061e4(args[1]);
# 021ea710(view) -> 1.
fn 021d1eb4 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021ea678 ret=none args=v,v,p
  stub 021ea720 ret=pick:0/1 args=v
  stub 020061e4 ret=none args=v
  stub 021ea710 ret=pick:0/1 args=v

# command 0x5B (message, other window): as 021d1e6c with 021ea694
fn 021d1f10 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021ea694 ret=none args=v,v,p
  stub 021ea710 ret=pick:0/1 args=v

# command 0x5D: as 021d1eb4 with 021ea694
fn 021d1f58 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021ea694 ret=none args=v,v,p
  stub 021ea720 ret=pick:0/1 args=v
  stub 020061e4 ret=none args=v
  stub 021ea710 ret=pick:0/1 args=v

# command 0x59: 0: idle -> 1; 021ea6f4(view, (u8) args[0], (u16) args[1]), state 1. 1: 021ea710(view) -> 1.
fn 021d1fb4 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021ea6f4 ret=none args=v,v,v
  stub 021ea710 ret=pick:0/1 args=v

# move-effect parameter for the animation: r1 == 0xAE and 021b98ac(pokecon, r2) -> 021d80a8(mon); else 0201bd28(r1, 0x1B)
fn 021d2000 ret=u32
  nowork
  arg buf 0x1C0
  arg u32 alt const 0xAE || range 0 0x200
  arg u32 any
  stub 0201bd28 ret=u32 args=v,v
  stub 021b98ac ret=nullable:2:u32 args=v,v
  stub 021d80a8 ret=u32 args=v

# command 0x30 (move animation): 0: idle -> 1; !021b8590(server) -> 1; 021ea0a0(view, a0, a1, a2, 021d2000(c, a2, a0), a3, 0) (a0 / a1 / a3
# u8, a2 u16), state 1. 1: 021ea0e4(view) -> 1.
fn 021d2030 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021b8590 force ret=pick:0/1/1 args=v
  stub 021d2000 ret=u32 args=p,v,v
  stub 021ea0a0 ret=none args=v,v,v,v,v,v,v
  stub 021ea0e4 ret=pick:0/1 args=v
  stub 0201bd28 ret=u32 args=v,v
  stub 021b98ac ret=nullable:2:u32 args=v,v
  stub 021d80a8 ret=u32 args=v

# command 0x31: 0: 021ea354(view, 021b8dc0(server, pokecon, (u8) args[0]), args[1] != 0), state 1. 1: !021f7eb4() -> 1.
fn 021d20bc ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  fix arg2 4 u32 alt const 0 || any
  stub 021b8dc0 force ret=u32 args=v,v,v
  stub 021ea354 ret=none args=v,v,v
  stub 021f7eb4 ret=pick:0/1 args=

# command 0x32: 0: 021ea0f4(view, (u16) args[2], 021b8d8c(server, pokecon, (u8) args[0]), (u8) args[1]), state 1. 1: 021ea120(view) -> 1.
fn 021d210c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021ea0f4 ret=none args=v,v,v,v
  stub 021ea120 ret=pick:0/1 args=v

# command 0x33: 0: n = (u16) args[0] (<= 8: an 8-byte stack list; the u8 counter never ends for n > 0xFF), list[i] = queue pop (thunk 021cd644 -> cmd_queue 021ccad4 on c+0x134, real code)
# for i < n; 021ea280(view, n, args[1], list, (u16) args[2]), state 1. 1: 021ea2bc(view) -> 1. (arg 3 is the queue.)
fn 021d215c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  arg buf 0x400
  fix arg1 0 u32 alt range 0 1 || any
  fix arg2 0 u16 range 0 8
  fix arg0 0x134 u32 addrof arg3 0
  fix arg3 4 u32 range 0 0x3D0
  stub 021ea280 ret=none args=v,v,v,in:8,v
  stub 021ea2bc ret=pick:0/1 args=v

# command 0x34: 0: 021ea384(view, 021b8d8c(server, pokecon, (u8) args[0])), state 1. 1: 021ea3a0(view) -> 021ea678(view, 0x61, args),
# state 2. 2: 021ea710(view) -> 1.
fn 021d21d4 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021ea384 ret=none args=v,v
  stub 021ea3a0 ret=pick:0/1 args=v
  stub 021ea678 ret=none args=v,v,p
  stub 021ea710 ret=pick:0/1 args=v

# command 0x35: 0: 021f81d0(args[1], 021b8e70(server, 021b8d8c(server, pokecon, (u8) args[0]))), state 1. 1: 1.
fn 021d2234 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b8e70 force ret=u32 args=v,v
  stub 021f81d0 ret=none args=v,v

# command 0x36: 0: not idle -> 021ea0a0(view, s, s, 1, 0201bd28(1, 0x1B), 0, 0) with s = 021b8d8c(server, pokecon, (u8) args[0]); state 1.
# 1: 021ea0e4(view) -> s again, !idle -> 020061e4(0x564); 021ea0f4(view, 1, s, 1), state 2. 2: 021ea120(view) -> 1.
fn 021d2278 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 0201bd28 ret=u32 args=v,v
  stub 021ea0a0 ret=none args=v,v,v,v,v,v,v
  stub 021ea0e4 ret=pick:0/1 args=v
  stub 020061e4 ret=none args=v
  stub 021ea0f4 ret=none args=v,v,v,v
  stub 021ea120 ret=pick:0/1 args=v

# command 0x39 (faint): 0: 021ea40c(view, 021b8d8c(server, pokecon, (u8) args[0])), state 1. 1: 021ea428(view) -> 021d646c(021b9934(pokecon,
# (u8) args[0])), 1.
fn 021d2320 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021ea40c ret=none args=v,v
  stub 021ea428 ret=pick:0/1 args=v
  stub 021b9934 ret=u32 args=v,v
  stub 021d646c ret=none args=v

# command 0x3A: 0: 021ea438(view, 021b8d8c(server, pokecon, (u8) args[0])), state 1. 1: 021ea448(view) -> 1.
fn 021d2374 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021ea438 ret=none args=v,v
  stub 021ea448 ret=pick:0/1 args=v

# command 0x38: p = 021b8e70(server, 021b8d8c(server, pokecon, (u8) args[0])); 0: idle -> 1; !021b8590(server) -> 1; 021ea7c8(view, p),
# state 1. 1: 021ea7e8(view, p) -> 1.
fn 021d23b8 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b8e70 force ret=u32 args=v,v
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021b8590 force ret=pick:0/1/1 args=v
  stub 021ea7c8 ret=none args=v,v
  stub 021ea7e8 ret=pick:0/1 args=v,v

# command 0x37: as 021d23b8 with 021ea7d8
fn 021d2424 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021b8e70 force ret=u32 args=v,v
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021b8590 force ret=pick:0/1/1 args=v
  stub 021ea7d8 ret=none args=v,v
  stub 021ea7e8 ret=pick:0/1 args=v,v

# command 0x3F (k = u8 args[0], < 5 here: the state-1 read of the ROM table 0x021EFFAE + 4k is unguarded): 0: 021ef9c0(c+0x34, k, (u16)
# args[1]); idle -> 1; k < 5 -> 021f7998(u16 0x021EFFB0[2k]); state 1. 1: !021f7eb4() -> 021ea678(view, u16 0x021EFFAE[2k], 0), state 2.
# 2: 021ea710(view) -> 1.
fn 021d2490 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 2 || any
  fix arg2 0 u8 range 0 4
  stub 021ef9c0 ret=none args=v,v,v
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021f7998 ret=none args=v
  stub 021f7eb4 ret=pick:0/1 args=
  stub 021ea678 ret=none args=v,v,v
  stub 021ea710 ret=pick:0/1 args=v

# command 0x40 (5-way jump table on u8 args[0], data 021d2538-021d2542): 0: kinds 0..3 -> 021ea678(view, 0x59 / 0x5A / 0x5B / 0x5C, 0), state
# 1; 4 or more -> 1. 1: 021ea710(view) -> 021ef9c8(c+0x34), 1.
fn 021d2510 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  fix arg2 0 u8 alt range 0 4 || any
  stub 021ea678 ret=none args=v,v,v
  stub 021ea710 ret=pick:0/1 args=v
  stub 021ef9c8 ret=none args=v

# command 0x41: 0: 021ea384(view, 021b8d8c(server, pokecon, (u8) args[0])), state 1. 1: 021ea3a0(view) -> 1.
fn 021d2580 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021ea384 ret=none args=v,v
  stub 021ea3a0 ret=pick:0/1 args=v

# command 0x42: s = 021b8d8c(server, pokecon, (u8) args[0]); 0: idle -> 1; !021b8590 -> 1; 021ea838(view, s), state 1. 1: 021ea850(view, s)
# -> 1.
fn 021d25c4 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021b8590 force ret=pick:0/1/1 args=v
  stub 021ea838 ret=none args=v,v
  stub 021ea850 ret=pick:0/1 args=v,v

# command 0x43: as 021d2580
fn 021d2628 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021ea384 ret=none args=v,v
  stub 021ea3a0 ret=pick:0/1 args=v

# command 0x44 (swap two party slots, k = u8 args[0], slots u8 args[1] / args[2]): 0: a = 021b8dec(server, slot1), b = 021b8dec(server,
# slot2), p = 021b8e70(server, slot1), q = 021b8e70(server, slot2), BattleParty_SwapSlots(021b9aec(pokecon, k), a, b), 021ea904(view, k, p,
# q, a, b), state 1. 1: 021ea950(view) -> 1.
fn 021d266c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8dec force ret=u32 args=v,v
  stub 021b8e70 force ret=u32 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9c10 ret=none args=v,v,v
  stub 021ea904 ret=none args=v,v,v,v,v,v
  stub 021ea950 ret=pick:0/1 args=v

# command 0x50 (4-way jump table, data 021d271c-021d2724): 0: 021f7998(0x281), state 1. 1: !021f7eb4(): swap (k = u8 args[0], slot u8
# args[1]) with slot 1: 021b8bec / 021b8e70 / BattleParty_SwapSlots / 021ea904(view, k, p, q, slot, 1), state 2. 2: 021ea950(view) -> the
# same with args[2] / args[3], state 3. 3: 021ea950(view) -> 1.
fn 021d2700 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 3 || any
  stub 021f7998 ret=none args=v
  stub 021f7eb4 ret=pick:0/1 args=
  stub 021b8bec force ret=u32 args=v,v,v
  stub 021b8e70 force ret=u32 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9c10 ret=none args=v,v,v
  stub 021ea904 ret=none args=v,v,v,v,v,v
  stub 021ea950 ret=pick:0/1 args=v

# command 0x51: 0: 021f7e94(021b8e70(server, (u8) args[0])), state 1. 1: !021f7eb4() -> 1.
fn 021d2814 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8e70 force ret=u32 args=v,v
  stub 021f7e94 ret=none args=v
  stub 021f7eb4 ret=pick:0/1 args=

# command 0x52: 0: p = 021b8e70(server, (u8) args[0]); idle -> 021f88bc(1); 021f7ea4(p), state 1. 1: !021f7eb4() -> idle -> 021f88bc(0);
# 1.
fn 021d284c ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b8e70 force ret=u32 args=v,v
  stub 021cd9e0 force ret=pick:0/1 args=p
  stub 021f88bc ret=none args=v
  stub 021f7ea4 ret=none args=v
  stub 021f7eb4 ret=pick:0/1 args=

# command 0x53: 0: m1 = 021b9934(pokecon, (u8) args[0]), m2 = 021b9934(pokecon, (u8) args[1]), s1 / s2 = 021b8dc0(server, pokecon, args[0]
# / args[1]); 021d6c50(m1, m2); 021ea8b4(view, s1, s2), state 1. 1: 021ea8d4(view) -> 1.
fn 021d28ac ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021b9934 ret=u32 args=v,v
  stub 021b8dc0 force ret=u32 args=v,v,v
  stub 021d6c50 ret=none args=v,v
  stub 021ea8b4 ret=none args=v,v,v
  stub 021ea8d4 ret=pick:0/1 args=v

# command 0x54: 0: idle -> 1; 021ea1b8(view, (u16) args[2], 021b8d8c(server, pokecon, (u8) args[0]), GetTypeEffectivenessClass(args[1])),
# state 1. 1: 021ea1f0(view) -> 1.
fn 021d2930 ret=u32
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 1 || any
  stub 021cd9e0 force ret=pick:0/0/1 args=p
  stub 021b8d8c force ret=u32 args=v,v,v
  stub 021d7ad4 ret=u32 args=v
  stub 021ea1b8 ret=none args=v,v,v,v
  stub 021ea1f0 ret=pick:0/1 args=v

# command 0x3E (EV / stat gains): pp = BattleMon_GetPokemon(021b9934(pokecon, (u8) args[0])); for i < 6 with the ROM table 0x021EFF88 (u16
# pairs at 2i: id, next != 0): v = 02017e1c(pp, id, 0) + args[1 + i], at most 0xFF -> 02017e40(pp, id, v). Returns 1.
fn 021d29cc ret=u32
  nowork
  arg buf 0x1C0
  arg u32 any
  arg buf 0x40
  stub 021b9934 ret=u32 args=v,v
  stub 021d5844 ret=u32 args=v
  stub 02017e1c ret=u32 args=v,v,v
  stub 02017e40 ret=none args=v,v,v

# command 0x45 (level up and learn moves; 14-way jump table, data 021d2a6c-021d2a88; state in the globals 0x021F3B64..): k = u8 args[0], m =
# 021b9934(pokecon, k), s = 021b8dc0(server, pokecon, k), f = s != 0xFF and 021f8204(s, s). 0: g[0xC] = args[1], g[8] = 0, state 1 and on.
# 1: g[0xC] ? (021d69b8(m, &g[0xC], 0x021F3B78) ? state 3 : f ? (021f8134(s, g[0xC]), state 2) : state 0xD; g[0xC] = 0) : state 0xD. 2:
# 021f8180() -> state 0xD. 3: 021d6ea0(m), 021d6bfc(m), f -> 021f8150(s, m), state 4, else state 6. 4: !021f8180() and !02006238() ->
# 021ea3b8(view, s, 0x25D), state 5. 5: 021ea3d4(view) -> state 6. 6: message 0x3C (k, g[0x14]), state 7. 7: 021ea720 -> fanfare
# (02005de4(1), 02005e64(), 02005d90(0x515, 0xFFFF)); 021ea710 and !02005f38() -> 02005e9c(), 02005de4(0), g[8] = 0, state 8. 8: g[8]++;
# 020362dc() & 3 or 02035c60() or g[8] > 0x50 -> g[8] = 0, 021ba238(server, m), 021eada4(view, m, 0x021F3B78), state 9. 9: 021eadb4(view)
# and a key (020362dc / 02035c60) -> 021eadc4(view), state 0xA. 0xA: 021eadd4 and a key -> 021eade4, state 0xB. 0xB: 021eadf4(view) -> g[8]
# = 0, state 0xC. 0xC: 021d2ca8(c, 0x021F3B6C, m) -> state 1. 0xD: 021b9358(server, k), 1.
fn 021d2a20 ret=u32 inputs=600
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg buf 0x40
  global 021f3b64 0x20
  fix arg1 0 u32 alt range 0 13 || any
  fix global0 8 u32 alt range 0x48 0x52 || any
  fix global0 0xC u32 alt const 0 || any
  stub 021b9934 ret=u32 args=v,v
  stub 021b8dc0 force ret=pick:0/1/0xFF args=v,v,v
  stub 021f8204 ret=pick:0/1 args=v,v
  stub 021d69b8 ret=pick:0/1 args=v,p,p
  stub 021f8134 ret=none args=v,v
  stub 021f8180 ret=pick:0/1 args=
  stub 021d6ea0 ret=none args=v
  stub 021d6bfc ret=none args=v
  stub 021f8150 ret=none args=v,v
  stub 02006238 ret=pick:0/1 args=
  stub 021ea3b8 ret=none args=v,v,v
  stub 021ea3d4 ret=pick:0/1 args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021ea614 ret=none args=v,p
  stub 021ea720 ret=pick:0/1 args=v
  stub 02005de4 ret=none args=v
  stub 02005e64 ret=none args=
  stub 02005d90 ret=none args=v,v
  stub 021ea710 ret=pick:0/1 args=v
  stub 02005f38 ret=pick:0/1 args=
  stub 02005e9c ret=none args=
  stub 020362dc ret=pick:0/0/1/4 args=
  stub 02035c60 ret=pick:0/0/1 args=
  stub 021ba238 ret=none args=v,v
  stub 021eada4 ret=none args=v,v,v
  stub 021eadb4 ret=pick:0/1 args=v
  stub 021eadc4 ret=none args=v
  stub 021eadd4 ret=pick:0/1 args=v
  stub 021eade4 ret=none args=v
  stub 021eadf4 ret=pick:0/1 args=v
  stub 021d2ca8 ret=pick:0/1 args=p,v,v
  stub 021b9358 ret=none args=v,v
  stub 02017e1c ret=u32 args=v,v,v
  stub 020181f4 ret=none args=v,v,v
  stub 02018328 ret=pick:0/0xFFFE/0x8021/0x21 args=v,p,v
  stub 021b9af8 ret=u32 args=v,v
  stub 021b9cbc ret=u32 args=v,v
  stub 021d5440 ret=u32 args=v
  stub 021d5844 ret=u32 args=v
  stub 021ea638 ret=none args=v,p,v
  stub 021eabc4 ret=none args=v,p,p,v
  stub 021eac54 ret=pick:0/1 args=v,outm:4:1/0/0/0
  stub 021eac64 ret=none args=v,v,v
  stub 021eac80 ret=pick:0/1 args=v,outm:1:7
  stub 021eaf78 ret=none args=p,v

# learn a move (state word r1, mon r2; 14-way jump table, data 021d2cce-021d2cea; move g.u16[0] and g[0x10] in the globals): n = 021d5440(mon).
# 0: g[0x10] = 0, g[0] = 0, state 1. 1: g[0] = 02018328(BattleMon_GetPokemon(mon), 0x021F3B74, c.u16[0x19A]); 0 -> 1; 0xFFFE -> 0; bit 15
# -> g[0] &= 0x7FFF, state 5; else state 2. 2: m = 021b9934(pokecon, n), 021d6bfc(m), message 5/3 (n, g[0]), state 3 and on. 3: 021ea720 ->
# fanfare 0x515; 021ea710 -> state 4. 4: !02005f38() -> 02005e9c(), 02005de4(0), state 1. 5: message 5/4 + 021eaf78(c+0x58, 0xFF), state 6
# and on. 6: 021ea720 -> yes/no window (021eaf18 x2 on stack buffers, 021eabc4(view, a, b, 1)); 021ea710 -> state 7. 7: 021eac54(view, &ans)
# -> ans ? state 0xA : (021eac64(view, (u8) 021b9cbc(021b9af8(pokecon, c[0x1A6]), n), g[0]), state 8). 8: 021eac80(view, &slot) -> slot 4
# -> state 5; else old = (u16) 02017e1c(pp, slot + 0x36, 0), 020181f4(pp, g[0], slot), message 5/5 (n, old), 021ea638(view, c+0x58,
# 0x021D2C7D), state 9. 9: 021ea710 -> state 2. 0xA: message 5/7 + 0xFF, state 0xB. 0xB: 021ea720 -> yes/no (6/4, 6/5 with g[0]); 021ea710 ->
# state 0xC. 0xC: 021eac54 -> ans ? state 5 : (message 5/8, state 0xD). 0xD: 021ea710 -> state 1.
fn 021d2ca8 ret=u32 inputs=600
  nowork
  arg buf 0x1C0
  arg buf 0x10
  arg u32 any
  global 021f3b64 0x20
  fix arg1 0 u32 alt range 0 13 || any
  fix global0 0 u16 alt oneof 0 0xFFFE 0x8005 || any
  stub 021d5440 ret=u32 args=v
  stub 021d5844 ret=u32 args=v
  stub 02018328 ret=pick:0/0xFFFE/0x8021/0x21 args=v,p,v
  stub 021b9934 ret=u32 args=v,v
  stub 021d6bfc ret=none args=v
  stub 021eaf18 ret=none args=p,v,v
  stub 021eaf50 ret=none args=p,v
  stub 021eaf78 ret=none args=p,v
  stub 021ea614 ret=none args=v,p
  stub 021ea720 ret=pick:0/1 args=v
  stub 02005de4 ret=none args=v
  stub 02005e64 ret=none args=
  stub 02005d90 ret=none args=v,v
  stub 021ea710 ret=pick:0/1 args=v
  stub 02005f38 ret=pick:0/1 args=
  stub 02005e9c ret=none args=
  stub 021eabc4 ret=none args=v,p,p,v
  stub 021eac54 ret=pick:0/1 args=v,outm:4:1/0/0/0
  stub 021b9af8 ret=u32 args=v,v
  stub 021b9cbc ret=u32 args=v,v
  stub 021eac64 ret=none args=v,v,v
  stub 021eac80 ret=pick:0/1 args=v,outm:1:7
  stub 02017e1c ret=u32 args=v,v,v
  stub 020181f4 ret=none args=v,v,v
  stub 021ea638 ret=none args=v,p,v
