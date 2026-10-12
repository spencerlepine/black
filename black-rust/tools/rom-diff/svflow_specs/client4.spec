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
