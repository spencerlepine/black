# svflow spec: adapter (btl_adapter.c, asm/battle_adapter.s, 0x021EEA30-0x021EF71C, battle set; B12-spec). Harness-job owned; porting
# workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs / B10-spec2 / B10f-spec / B12-spec additions). New in B12-spec: the stub argument kind
# `outp:K:OFS:N` (README), used where a stub hands the function a pointer through an out word that the function then dereferences.
#
# What this area is: the file holds two halves that the build put into one object.
#  (1) 0x021EEA30-0x021EEC76, the ADAPTER proper (D7 seam, see the doc comment of server_flow/adapter.rs): a 4-slot table of
#      per-client channel objects (word table at 0x021F6388, flag word at 0x021F6384, both in `global 021f6384 0x14`) and the
#      calls the server (server_side.spec: 021bb724/768/798/7d0/7fc) and client (btl_client, 0x021CD6F0+) make on the objects.
#  (2) 0x021EEC78-0x021EF71C, the battle MESSAGE PRINTER of the UI half of overlay 93 (callers 0x021E9034-0x021EE200): context at
#      0x021F6398 (`global 021f6398 0x60`), word-set helpers eed00..eed90, the tag expander eefe0, the id dispatch eeef4.
# 45 real functions are specced (0 veneers; 3 are 4-byte thunks: 021eeae0 -> Heap_Free, 021ef6a8 -> 021ef6b0, 021ef6e8 -> 021ef6f0).
# Ghidra splits `LAB_021ef47a` off 021ef460 (same function, 021ef460-021ef4fe) and lists 021ef71c (the next file) in the range:
# neither is a function of this file. Instruction coverage (tools/rom-diff/fncov-style hook, 200+ inputs per function, every instruction run at least 6 times): all
# 1371 instructions run except 021ef2de (`b 021ef32e`, dead: r4 is 0/1/2 there); `--strict-callees` is clean; with every function treated
# as ported (the whole tree real on the ROM side) all 45 run without a fault, a stray write or an unstubbed callee.
#
# Adapter object ("obj", 0xBD0 bytes from Heap_AllocDebug in 021eea58; `arg buf 0x1000` here):
#   +0x00 owner  +0x04 reply ptr  +0x08 reply size  +0x0C u8 (low nibble reply state 0/1; high nibble transport: 0 local, else comm)
#   +0x0D u8 client id  +0x0E u8 send phase (0 idle, 1 queued, 3 sent, 4 done)  +0x0F u8 code  +0x10 u16 code  +0x12 u16 size
#   +0x14 payload (<= 0xBBC bytes). A pointer the comm layer returns (021d4c6c) is `outp:0:0x800:64`: an address in 0x800..0x8FC of obj.
# Message-printer context at 0x021F6398 (ctx; the harness global is 0x60 bytes): +0 system, +4 pokecon, +8 / +0xC message buffers
# (opaque handles), +0x10..+0x2C 8 message-bank handles, +0x30..+0x4C 8 message arguments (written by 021eee24), +0x50 u16,
# +0x54 u32 (side/mon id), +0x58 u16 ("sub-message" flag; 0 or not), +0x5A u16 (flag 021ef6b0 raises while it expands).
# Tables read by the real code on the ROM side, so ports embed them: 0x021F3990 (8 u16 bank ids), 0x021F39A0 (27 u16 message ids,
# remapped by 021eee80), 0x021F39D8 (14 x {u16 id, u16 pad, u32 handler}; handlers 021ef460, ef508, ef3a4, ef595, ef205).
#
# Rules followed (same as core4): every block lists the stubs of its callees AND of the callees of all its in-area descendants, and the
# resource lines of the whole sub-tree, so porting in any order needs no spec edit. In-area functions are stubs while unported and go
# inactive when ported. Callees outside the area: comm layer 021d4b58/4b98/4bd4/4c60/4c6c/4c84/4cec, allocator 02030734/020307b0,
# message bank / window 0201xxxx / 0204xxxx, mon queries 021b86b8/8bd0/8ce8/8d08/8ff4/9940/9f70/9fac/a004/a020/a064/a080,
# BattleMon_GetVisibleMon 021d5864 / GetPokemon 021d5844. 02082d44 (MI_CpuCopy8, in eeb00) runs as real code on both sides (ALLOW).
# Thunks outside the file whose targets are stubbed: 021ba1a8 -> 02013158 (`ldr r0,[r0]; bx`: eec78's first argument is a pointer
# whose word 0 the thunk reads), 021d4c58 -> 021d4cec (both in the svflow.py ALLOW list, with 021b8ddc).
#
# String cursor of the tag expander (eef20, eef98, eefe0 and everything above them): the message data is `arg buf 0x100` (the "arena";
# its index is the last declared argument), halfwords 0-3 are tag starts (0xF000) / chars / terminators (0xFFFF), 4-7 likewise, 8 and
# up terminators. 02045a98 (terminator) answers 0xFFFF, 0201d5c4 (tag start) 0xF000, 02045a90 returns the arena start, 0201d688
# ("after this tag") a terminator slot: at most one tag is processed per pass, and every loop ends. Tag index (0201d670) is 0..7
# (the code indexes an 8-byte stack array with it: evidence, 021eefe0 clears 8 bytes), group (0201d5f4) 0..3 (1 and 2 matter),
# type (0201d658) 0..18 weighted towards the cases of the 021eefe0 jump table (0x21ef0f0, 16 halfword offsets; 3 and f share a body, 4 5 8 a b d and above f do nothing).
# 021eee24 is varargs (`push {r0-r3}`): register and stack arguments are one list; word 2 is the count (bounded to 0..8, the argument
# slots), words 3.. the values (vals[3..11) in the port), the arena is the 12th argument.
# 021eea58: the slot index is bounded to 0..3 (4-entry table; the callers pass client ids), table entries are 0 or the arena address
# (the pointer Heap_AllocDebug returns, `ptr:arg4:0:1`); the flag word is 0 or random. Its r3 is the heap id (stub argument 0 of the
# allocator), [sp] = 0x7e is the line number. The return is `ofs:arg4` (the pointer as an offset in the arena, 0 stays 0).
# 021eeb2c / 021eeb84 pass r3 through to the send (021d4b58 takes 4 registers), hence 4 declared arguments.
# 021eec20/eebe8 read through the pointer 021d4c6c writes (`outp`), so they take the object as the arena and need no other buffer.
#
# Traps for other specs (do NOT edit them here): server_side.spec stubs 021eea58, eead0, eeae8, eeafc, eeb00, eeb2c, eeb64 and eeb70
# (many lines). When one of them is ported its stub lines there need `force` (the Rust callers go through env.call). No other spec
# in svflow_specs or specs/ names an address of this file.
# Smoke ports (B12-spec, in server_flow/adapter.rs): 021eea30 021eea54 021eeba4 021eebdc 021eebe8 021eec20.
area adapter

# reset the adapter table: table[0..4] = 0, flag = r0
fn 021eea30 ret=none
  nowork
  arg u32 any
  global 021f6384 0x14

# empty (bx lr)
fn 021eea54 ret=none
  nowork

# get-or-create the adapter object of client idx: table[idx] (alloc 0xBD0 bytes through Heap_AllocDebug when empty); flag != 0 releases the slot
fn 021eea58 ret=ofs:arg4
  nowork
  arg u32 any
  arg s32 range 0 3
  arg u32 any
  arg u32 any
  arg buf 0x1000                   # the heap block that Heap_AllocDebug (stub ptr:arg4) returns
  global 021f6384 0x14
  fix global0 0 u32 alt const 0 || any
  fix global0 4 u32 alt const 0 || addrof arg4 0 rep=4,4
  stub 02030734 ret=ptr:arg4:0:1 args=v,v,v,v,v

# reset the adapter object: state byte +0xE = 0, +0xF = 0, low nibble of +0xC = 0
fn 021eead0 ret=none
  nowork
  arg buf 0x1000                   # adapter object

# thunk (ldr r3,[pc]; bx r3) to Heap_Free (020307b0)
fn 021eeae0 ret=none
  nowork
  arg u32 any
  stub 020307b0 ret=none args=v

# if the flag at 021f6384 is set: comm reset (021d4cec)
fn 021eeae8 ret=none
  nowork
  global 021f6384 0x14
  fix global0 0 u32 alt const 0 || any
  stub 021d4cec ret=none args=

# empty (bx lr)
fn 021eeafc ret=none
  nowork

# queue a message: state 1, code, copy size bytes of data to +0x14
fn 021eeb00 ret=none
  nowork
  arg buf 0x1000                   # adapter object
  arg u16 any
  arg buf 0x1000                   # source data
  arg u32 alt range 0 8 || range 0 0x400

# advance the send state machine (1 -> 3 -> 4); 1 when done
fn 021eeb2c ret=bool
  nowork
  arg buf 0x1000                   # adapter object
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 0xC u8 alt const 0 || range 0 15 || any    # high nibble 0 = local adapter, else the comm layer is used
  fix arg0 0xE u8 alt const 1 || const 3 || const 4 || any
  stub 021eeb84 ret=pick:0/1 args=p,v,v,v
  stub 021eeba4 ret=pick:0/1 args=p
  stub 021d4b58 ret=pick:0/1 args=v,p,v,v
  stub 021d4b98 ret=pick:0/1 args=
  stub 021d4bd4 ret=u16 args=v,out:4

# returns obj[4]; *out = obj[8] when out != 0
fn 021eeb64 ret=u32
  nowork
  region out 8
  arg buf 0x1000                   # adapter object
  arg u32 alt const 0 || addrof region:out

# state byte = 0; if the comm nibble is set: comm reset (021d4cec)
fn 021eeb70 ret=none
  nowork
  arg buf 0x1000                   # adapter object
  fix arg0 0xC u8 alt const 0 || range 0 15 || any    # high nibble 0 = local adapter, else the comm layer is used
  stub 021d4cec ret=none args=

# if the comm nibble is set: comm send (021d4b58) of the +0x10 block, else 1
fn 021eeb84 ret=u32
  nowork
  arg buf 0x1000                   # adapter object
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 0xC u8 alt const 0 || range 0 15 || any    # high nibble 0 = local adapter, else the comm layer is used
  stub 021d4b58 ret=pick:0/1 args=v,p,v,v

# receive step: local -> obj[0xC] & 0xF; networked -> comm receive when ready, state nibble = 1
fn 021eeba4 ret=u32
  nowork
  arg buf 0x1000                   # adapter object
  fix arg0 0xC u8 alt const 0 || range 0 15 || any    # high nibble 0 = local adapter, else the comm layer is used
  stub 021d4b98 ret=pick:0/1 args=
  stub 021d4bd4 ret=u16 args=v,out:4

# obj[0x10] = 0 (u16), obj[0x12] = 4 (u16)
fn 021eebdc ret=none
  nowork
  arg buf 0x1000                   # adapter object

# first byte of the received block
fn 021eebe8 ret=u32
  nowork
  arg buf 0x1000                   # adapter object
  fix arg0 0xC u8 alt const 0 || range 0 15 || any    # high nibble 0 = local adapter, else the comm layer is used
  stub 021d4c60 ret=pick:0/1 args=
  stub 021d4c6c ret=u16 args=outp:0:0x800:64

# size of the received block; *out = its data address
fn 021eec20 ret=u16
  nowork
  arg buf 0x1000                   # adapter object
  arg buf 8
  fix arg0 0xC u8 alt const 0 || range 0 15 || any    # high nibble 0 = local adapter, else the comm layer is used
  stub 021d4c6c ret=u16 args=outp:0:0x800:64

# post (a, b): comm send2 or the local slot {obj[4] = a, obj[8] = b, state nibble 1}
fn 021eec4c ret=u32
  nowork
  arg buf 0x1000                   # adapter object
  arg u32 any
  arg u32 any
  fix arg0 0xC u8 alt const 0 || range 0 15 || any    # high nibble 0 = local adapter, else the comm layer is used
  stub 021d4c84 ret=u32 args=v,v

# message context init (ctx = 021f6398)
fn 021eec78 ret=none
  nowork
  arg buf 0x10                     # system object (word 0 is read by the thunk 021ba1a8)
  arg u32 any
  arg u32 any
  arg u32 any
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  stub 02013158 ret=u16 args=v,v
  stub 0201ec64 ret=u32 args=v
  stub 020457b0 ret=u32 args=v,v
  stub 02045b38 ret=u32 args=v,v,v,v

# message context free
fn 021eecd8 ret=none
  nowork
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  stub 0201ed04 ret=none args=v
  stub 02045808 ret=none args=v
  stub 02045c04 ret=none args=v

# word-set: visible mon
fn 021eed00 ret=none
  nowork
  arg u32 any
  arg u32 any
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v

# word-set: pokemon
fn 021eed24 ret=none
  nowork
  arg u32 any
  arg u32 any
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5844 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v

# word-set: visible mon (EE84)
fn 021eed48 ret=none
  nowork
  arg u32 any
  arg u32 any
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v

# word-set from a value
fn 021eed6c ret=none
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v

# word-set by kind (4 / reachable / not)
fn 021eed90 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt const 4 || range 0 8 || any
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v

# varargs: copy `count` words to ctx+0x30, zero the rest, then eee80(a, id, ctx+0x30)
fn 021eee24 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt range 0 8 || const 8
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg11 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg11 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg11 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 021eee80 ret=none args=v,v,p
  stub 021b8bd0 force ret=pick:0/1 args=v,v
  stub 021eeec8 ret=none args=v,v,p
  stub 02045cac ret=none args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg11:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg11:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v

# id remap through the 27-entry table (+1 when 021b8bd0 says so), then eeec8
fn 021eee80 ret=none
  nowork
  arg u32 any
  arg u32 alt oneof 0x94 0x96 0x98 0x9A 0x9C 0x9E 0xA0 0xA2 0xB0 0x84 0x86 0x80 0x82 0x7C 0x7E 0x88 0x8A 0x8C 0x8E 0x90 0x92 0xA4 0xA6 0xA8 0xAA 0xAC 0xAE || any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 021b8bd0 force ret=pick:0/1 args=v,v
  stub 021eeec8 ret=none args=v,v,p
  stub 02045cac ret=none args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v

# set the message (45CAC), expand its tags (eefe0), show it (F250)
fn 021eeec8 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 02045cac ret=none args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v

# dispatch on the message id through the 14-entry table at 021f39d8, default ef1cc
fn 021eeef4 ret=none
  nowork
  arg u32 any
  arg u32 alt oneof 0x1B 0x5A 0x99 0xAE 0x3AA 0x21 0x414 0x417 0x14D 0x471 9 0xC 0x12 0x15 || any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 021ef460 ret=none args=v,v,p
  stub 021ef508 ret=none args=v,v,p
  stub 021ef3a4 ret=none args=v,v,p
  stub 021ef594 ret=none args=v,v,p
  stub 021ef204 ret=none args=v,v,p
  stub 021ef1cc ret=none args=v,v,p
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 021eef20 seq ret=nullable:2:ptr:arg3:0x10:16 args=p,outm:4:1/0/0/0,outm:2:0xF/0,outm:1:7
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v
  stub 02045cac ret=none args=v,v,v
  stub 021eef98 ret=pick:0/1/2/2/3 args=v
  stub 021ef280 ret=none args=v,v,p
  stub 021b8d08 force ret=u32 args=v
  stub 021b8ce8 force ret=pick:0/1 args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v

# read the next tag of a string: *group, *type, *index; returns the pointer after it, 0 at the end
fn 021eef20 ret=ofs:arg4
  nowork
  arg u32 alt const 0 || addrof arg4 0
  arg buf 4
  arg buf 4
  arg buf 4
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  fix arg4 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg4 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg4 0x10 u16 const 0xFFFF rep=64,2
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg4:0x10:16 args=p

# count the tags of the message that are (group 1, type 1/2/12)
fn 021eef98 ret=u8
  nowork
  arg u32 any
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  fix arg1 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg1 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg1 0x10 u16 const 0xFFFF rep=64,2
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg1:0x10:16 args=p
  stub 02045a90 ret=ptr:arg1:0:1 args=v
  stub 021eef20 seq ret=nullable:2:ptr:arg1:0x10:16 args=p,outm:4:1/0/0/0,outm:2:0xF/0,outm:1:7

# expand the tags of a message: per tag, dispatch on its type to the word-set functions
fn 021eefe0 ret=none inputs=800
  nowork
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg u32 any
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v

# pick the message variant (ef204 / ef280) by the tag count
fn 021ef1cc ret=none inputs=400
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 021eef20 seq ret=nullable:2:ptr:arg3:0x10:16 args=p,outm:4:1/0/0/0,outm:2:0xF/0,outm:1:7
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v
  stub 02045cac ret=none args=v,v,v
  stub 021eef98 ret=pick:0/1/2/2/3 args=v
  stub 021ef280 ret=none args=v,v,p
  stub 021ef204 ret=none args=v,v,p
  stub 021b8d08 force ret=u32 args=v
  stub 021b8ce8 force ret=pick:0/1 args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v

# message id + variant offset (0 / 1 / 2 by the side), expand, show
fn 021ef204 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v
  stub 021b8d08 force ret=u32 args=v
  stub 021b8ce8 force ret=pick:0/1 args=v,v,v
  stub 02045cac ret=none args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v

# like ef204 for two mons (variant 0 / 1 / 2 each)
fn 021ef280 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v
  stub 021b8d08 force ret=u32 args=v
  stub 021b8ce8 force ret=pick:0/1 args=v,v,v
  stub 02045cac ret=none args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v

# mon message with a count (handler for 0x3AA)
fn 021ef3a4 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg2 0xC u32 alt const 2 || range 0 4 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021b8d08 force ret=u32 args=v
  stub 021b8ce8 force ret=pick:0/1 args=v,v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021eed00 ret=none args=v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 02045cac ret=none args=v,v,v
  stub 0201f250 ret=none args=v,v,v

# mon message with a count (handler for 0x1B / 0x5A)
fn 021ef460 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg2 8 u32 alt const 2 || range 0 4 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021eed00 ret=none args=v,v
  stub 021b8d08 force ret=u32 args=v
  stub 021b8ce8 force ret=pick:0/1 args=v,v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 02045cac ret=none args=v,v,v
  stub 0201f250 ret=none args=v,v,v

# mon message (handler for 0x99 / 0xAE)
fn 021ef508 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021eed00 ret=none args=v,v
  stub 021b8d08 force ret=u32 args=v
  stub 021b8ce8 force ret=pick:0/1 args=v,v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 02045cac ret=none args=v,v,v
  stub 0201f250 ret=none args=v,v,v

# message 0x21 / 0x22 / 0x23 by whether the mon is the player's / active
fn 021ef594 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg2 0 u32 alt range 0 3 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 02045cac ret=none args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v

# mon message by id * 3 + side
fn 021ef5e8 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021b8d08 force ret=u32 args=v
  stub 021b8ce8 force ret=pick:0/1 args=v,v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021eed00 ret=none args=v,v
  stub 02045cac ret=none args=v,v,v
  stub 0201f250 ret=none args=v,v,v

# set message 3 * (b - 1) + 1 + (c != 0) when b != 0 (45CAC)
fn 021ef664 ret=none
  nowork
  arg u32 any
  arg s32 alt const 0 || range 1 5 || any
  arg u32 alt const 0 || any
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  stub 02045cac ret=none args=v,v,v

# tail call 45CAC(ctx[0x1C], b, a)
fn 021ef694 ret=none
  nowork
  arg u32 any
  arg u32 any
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  stub 02045cac ret=none args=v,v,v

# thunk to ef6b0
fn 021ef6a8 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v
  stub 021ef6b0 ret=none args=v,v,p
  stub 02045cac ret=none args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v

# message with the flag ctx+0x5A set while expanding
fn 021ef6b0 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v
  stub 02045cac ret=none args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v

# thunk to ef6f0
fn 021ef6e8 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v
  stub 021ef6f0 ret=none args=v,v,p
  stub 02045cac ret=none args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v

# message (ctx handle 0x28), expand, show
fn 021ef6f0 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x40                     # the message argument array (words)
  arg buf 0x100                    # the message string: tags (0xF000) and chars in halfwords 0-7, terminators (0xFFFF) from halfword 8
  global 021f6398 0x60
  fix global0 0x58 u16 alt const 0 || any
  fix global0 0x5A u16 alt const 0 || any
  fix arg3 0 u16 alt const 0xF000 || const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 8 u16 alt const 0xF000 || const 0x20 || const 0xFFFF rep=4,2
  fix arg3 0x10 u16 const 0xFFFF rep=64,2
  stub 021b9940 force ret=u32 args=v,v
  stub 021d5864 ret=u32 args=v
  stub 0201eef4 ret=none args=v,v,v
  stub 021d5844 ret=u32 args=v
  stub 0201ee84 ret=none args=v,v,v
  stub 021ba020 force ret=u8 args=v,v
  stub 0201f0f8 ret=none args=v,v,v
  stub 021b9f70 force ret=pick:0/1 args=v,v
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021ba004 force ret=pick:0/1 args=v,v
  stub 0201f134 ret=none args=v,v,v
  stub 021b9fac force ret=u32 args=v,v,out:4
  stub 0202a3cc ret=u32 args=v
  stub 0201ee0c ret=none args=v,v,v,v,v,v
  stub 021ba064 force ret=u32 args=v,v
  stub 0201f02c ret=none args=v,v,v
  stub 021ba080 force ret=u32 args=v
  stub 02045a98 ret=pick:0xFFFF args=
  stub 0201d5c4 ret=pick:0xF000 args=
  stub 02045a90 ret=ptr:arg3:0:1 args=v
  stub 0201d5cc ret=pick:0/1/1/1 args=p
  stub 0201d670 seq ret=pick:0/1/2/3/4/5/6/7 args=p,v
  stub 0201d5f4 seq ret=pick:0/1/1/1/2/2/3 args=p
  stub 0201d658 seq ret=pick:0/0/0/1/1/1/2/2/2/3/3/3/6/6/6/7/7/7/9/9/9/12/12/12/14/14/14/14/15/15/15/4/5/8/10/11/13/16 args=p
  stub 0201d688 ret=ptr:arg3:0x10:16 args=p
  stub 0201efa4 ret=none args=v,v,v,v,v,v
  stub 021eed90 ret=none args=v,v,v
  stub 021eed48 ret=none args=v,v
  stub 021eed00 ret=none args=v,v
  stub 021eed24 ret=none args=v,v
  stub 021eed6c ret=none args=v,v,v
  stub 0201ef8c ret=none args=v,v,v
  stub 0201ef34 ret=none args=v,v,v
  stub 0201ef40 ret=none args=v,v,v
  stub 0201f020 ret=none args=v,v,v
  stub 02045cac ret=none args=v,v,v
  stub 021eefe0 ret=none args=v,p,v
  stub 0201f250 ret=none args=v,v,v
