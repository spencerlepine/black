# svflow spec: server2 (0x021B75A8-0x021B902C; B11, written by B11-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs .. B13-spec1 additions). No new grammar.
#
#
# What this area is (B11-spec area split, see crates/black-battle/src/server/mod.rs): the second part of battle_server.s
# (btl_main.c-like: BattleProc + main module M; btl_server.c proper starts at 0x021BA9C8 in server3):
#   * 75a8 7758 7968 7bb0 7cb4 7e18 8070 81dc 835c: sub-proc steps `u32 step(M, u32 *seq)` from the per-battle-kind tables at
#     0x021EFD04 (6 pointers each: 75a9 7759 7969 7bb1 {7cb5 | 7e19 | 81dd | 8071} 835d; which server1 function walks them is not checked here); 75a8-7bb0
#     are the comm handshake / exchange state machines (battle_net.s 021d4xxx calls), 7cb4/7e18/8070/81dc create the server
#     (021ba9c8) and the clients (021cd6f0) and wire them (server_side 021baa94/baacc/bab10, client 021cd960), 835c creates the view
#     and selects the main loop M[0x464] = 8434 (server console) or 84d8 (client console); 83f4 is a loop without server
#   * 8548 8564: client-slot loops; 8588..8750: main-module getters (M[0] = BATTLE_SETUP); 8778..8e28: target-spec expansion and
#     position <-> client-id maps; 8e70..9010: side / partner helpers.
# Function addresses = Thumb, all `nowork`. Thunks (not specced; callers stub the target): 021b8ddc -> 021b8d08, 021b8de4 -> 021b8cfc.
#
# Objects confirmed from the code (offsets in M, the 0x474-byte main module; arg0 = `arg buf 0x1000`):
#   +0x000 BATTLE_SETUP* (zone 0x480, `fix arg0 0 u32 addrof arg0 0x480`)   +0x004 battle view (021e8f20)   +0x008 / +0x00C server
#   object (server console / client console; 021ba9c8)   +0x010..0x01C 4 client objects (021cd6f0)   +0x020 + 0x28 i (i < 4): per-
#   client comm block (+0x20 word, +0x24 name string)   +0x0C0 word (8b78.., 4808 arg)   +0x0C4..0x0C7 u8 client done flags
#   +0x0C8 / +0x1B0 party containers (93e0 / 9460 / 969c / 985c: server3)   +0x2C4 comm data handle   +0x3E0 + 4 i received
#   blocks (0x3EC bytes)   +0x3F0 0x18-byte block (server create arg 1)   +0x408 0x20-byte comm record   +0x428..0x42D u8 position ->
#   client id map   +0x43C word   +0x448 2-word result pair (client 021cdd94 / server 021babdc)   +0x464 main-loop function
#   +0x468 u16 heap id   +0x46A..0x46F u8 comm fields (+0x46B client count, +0x46C own client id, +0x46D side flag, +0x46E "is
#   server")   +0x470..0x472 u8 loop counters / index   +0x473 flag bits (bit 0 comm error / quit, bit 1 done, bit 2 read by 8590)
# BATTLE_SETUP fields read here (setup = arena 0x480): +0x00 battle type (0..4: 86b8/86e4/8630/85ec/8f38), +0x04 rule (0..3:
#   8588/8640/8670/8cb0/8778/8e70/8ed8), +0x08 (8714 returns its address), +0x1A u16, +0x1C word, +0x20 u8, +0x22 u8 (0/1/2 comm
#   mode: 86c8/7e18), +0x24 / +0x28 / +0x30 words, +0x48 + 4 k, +0x58, +0x5C, +0x68 (0200ced4/0200cf28/0200cfa0 handle), +0x70,
#   +0x74 / +0x76 / +0x86 u16, +0x80 u8, +0x91 u8, +0x9C 0x18 bytes, +0xC1 flag bits.
# Data zones in arg0: 0x480 setup (0x100), 0x600 copy source and 0xA00 copy destination of 7bb0 (0x3EC each), 0xE00 string chars
# (7968). A port resolves an address as `addr - 0x02E1_0000`.
#
# Conventions (B11 CONVENTIONS, same as core4/core5/client/adapter): closure stubs (each block lists the stubs of its own callees
# and of all in-area descendants; checked with every server2 function treated as ported: no fault / stray write / unstubbed
# callee); callees in other svflow areas (server1 021b6818 / 6994, server3 021b9xxx-021ba9c8, server_side 021baa94+, client
# 021cdxxx, adapter) are `force` stubs; battle_net 021d4xxx, view 021e8xxx-021e9xxx, BattleMon 021d5xxx and ARM9 callees are plain
# stubs. 02007b44 (thunk with r1 = 0x24) is stubbed at its own address like in client2/core specs; 021d4a04 (thunk) at its target
# 021d4a6c. MI_CpuCopy8 (02082d44) runs as real code (svflow ALLOW).
#
# Ghidra errors seen (pseudo-C is unreliable here): 021b6100's sub-proc steps are split into sub_/LAB_ pairs (75a8 + LAB_75c6 ...);
# 8c50 / 85dc / 873c / 8fe4 / 9000 are tail calls (`bx r3`), not void; 87e8 / 8834 / 88ec get the out pointer in r3 from 8778
# (pass-through), 8778's sizes in Ghidra are wrong; 7cb4 / 7e18 / 8070 / 81dc ignore the seq pointer (one-shot, return 1);
# 85ec calls 021809d8 (an address in a runtime-loaded overlay, between 02034ac4(231) / 02034a5c(231) = overlay load / unload?).

area server2

# ==== main-module getters (M = arg0, `arg buf 0x1000`; BATTLE_SETUP *M at arena 0x480) ==========================

# for each of the 4 client slots M[0x10 + 4i] != 0: 021cd914(client) (client area: force)
fn 021b8548 ret=none
  nowork
  arg buf 0x1000
  fix arg0 0x10 u32 alt const 0 || any rep=4,4
  stub 021cd914 force ret=none args=v

# 0 if some client slot i has M[0x10 + 4i] != 0 and M[0xC4 + i] == 0, else 1
fn 021b8564 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0x10 u32 alt const 0 || any rep=4,4
  fix arg0 0xC4 u8 alt const 0 || any rep=4,1

# setup->rule (word +4)
fn 021b8588 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# bit 2 of M[0x473]
fn 021b8590 ret=u32
  nowork
  arg buf 0x1000

# setup byte +0x80
fn 021b85a0 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# 1 if type(86b8) == 1 && rule(8588) == 0 && setup+0x20 (86c0) == 0 && 020083c8(setup+0x5C) == 0, else 0
fn 021b85a8 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x480 u32 alt range 0 2 || any
  fix arg0 0x484 u32 alt const 0 || any
  fix arg0 0x4A0 u8 alt const 0 || any
  stub 021b86b8 ret=pick:0/1/1/2 args=ptr
  stub 021b8588 ret=pick:0/0/1 args=ptr
  stub 021b86c0 ret=pick:0/0/1 args=ptr
  stub 020083c8 ret=pick:0/0/1 args=v

# tail call 02013080(M[0x43C])
fn 021b85dc ret=u32
  nowork
  arg buf 0x1000
  stub 02013080 ret=u32 args=v

# if setup->type <= 1: load overlay-ish 02034ac4(231); r = 021809d8(); 02034a5c(231); r == 0 -> 0;
# else 021ba1a8(M, 0x200) == 0 -> 1. Otherwise 0.
fn 021b85ec ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x480 u32 alt range 0 2 || any
  stub 02034ac4 ret=none args=v
  stub 021809d8 ret=pick:0/1/1
  stub 02034a5c ret=none args=v
  stub 021ba1a8 force ret=pick:0/1 args=ptr,v

# setup->type <= 1 ? setup word +0x70 : 0
fn 021b8630 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x480 u32 alt range 0 2 || any

# rule -> 1/3/5/5, default 5
fn 021b8640 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 4 || any

# rule -> 1/2/3/1, default 1
fn 021b8670 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 4 || any

# pos < 2 * 8670(M) (8670 gets M in r0)
fn 021b86a0 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 7 || any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 4 || any
  stub 021b8670 ret=pick:1/2/3 args=ptr

# setup->type (word +0)
fn 021b86b8 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# setup byte +0x20
fn 021b86c0 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# setup byte +0x22 != 0
fn 021b86c8 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A2 u8 alt const 0 || any

# setup u16 +0x1A
fn 021b86dc ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# type -> 0/1/2/2, default 1
fn 021b86e4 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x480 u32 alt range 0 4 || any

# &setup->[+8] (returned as an offset into arg0)
fn 021b8714 ret=ofs:arg0
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# M[0xC0]
fn 021b871c ret=u32
  nowork
  arg buf 0x1000

# 0200ced4(setup+0x68 word, 021d5444(mon))
fn 021b8724 ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  stub 021d5444 ret=u32 args=v
  stub 0200ced4 ret=u32 args=v,v

# tail call 0200cf28(setup word +0x68)
fn 021b873c ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  stub 0200cf28 ret=u32 args=v

# setup word +0x58
fn 021b8748 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# if type(86b8) <= 1 and flag != 0: 0200cfa0(setup+0x68 word, BattleMon_GetPokemon(mon))
fn 021b8750 ret=none
  nowork
  arg buf 0x1000
  arg u32 alt const 0 || any
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  stub 021b86b8 ret=pick:0/1/2/3 args=ptr
  stub 021d5844 ret=u32 args=v
  stub 0200cfa0 ret=none args=v,v

# ==== target / position helpers (rule = setup word +4; positions 0..5, 6 = none) ===============================
# The 0x428 byte array M[0x428..0x42D] maps the 6 positions to client ids; M[0x46C] = own client id, M[0x46D] = side flag.

# expand a target spec (r1: bits 8-15 kind, 0-7 pos) into a list of positions at out (r2); returns the count.
# kind 0: out[0] = pos, count = pos != 6. pos 6: out[0] = 6, 0. Else by rule: 0/3/default -> 87e8, 1 -> 8834, 2 -> 88ec
# (they get (M, kind, pos, out)).
fn 021b8778 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt mask 0x0F07 || mask 0x0F0F || any
  arg buf 0x10
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 4 || any
  stub 021b87e8 ret=pick:0/1/2 args=ptr,v,v,out:6
  stub 021b8834 ret=pick:0/1/2/3/4 args=ptr,v,v,out:6
  stub 021b88ec ret=pick:0/1/2/3/4/5/6 args=ptr,v,v,out:6
  stub 021b8c50 ret=u8 args=ptr,v,v
  stub 021b8c20 ret=u8 args=v,v,v
  stub 021b8c38 ret=u8 args=v,v
  stub 021b8cb0 ret=u8 args=ptr,v
  stub 021b99f0 force ret=pick:0/1 args=v

# rule-0 (single) expansion: kind 0..8 (r1), pos r2, out r3
fn 021b87e8 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 9 || any
  arg u32 alt range 0 6 || any
  arg buf 0x10
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 4 || any
  stub 021b8c50 ret=u8 args=ptr,v,v
  stub 021b8c20 ret=u8 args=v,v,v

# rule-1 (double) expansion: kind 0..9 (r1), pos r2, out r3
fn 021b8834 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 10 || any
  arg u32 alt range 0 6 || any
  arg buf 0x10
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 4 || any
  stub 021b8c50 ret=pick:0/1/2/3/4/5 args=ptr,v,v
  stub 021b8c20 ret=u8 args=v,v,v
  stub 021b8cb0 ret=pick:0/1/2/3/4/5 args=ptr,v

# rule-2 (triple) expansion: kind 0..9 (r1), pos r2, out r3; 99f0(pos) = centre position?
fn 021b88ec ret=u32 inputs=600
  nowork
  arg buf 0x1000
  arg u32 alt range 0 10 || any
  arg u32 alt range 0 6 || any
  arg buf 0x10
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 4 || any
  stub 021b99f0 force ret=pick:0/1 args=v
  stub 021b8c50 ret=u8 args=ptr,v,v
  stub 021b8c20 ret=u8 args=v,v,v
  stub 021b8c38 ret=u8 args=v,v

# number of positions i in 0..5 with M[0x428 + i] == id
fn 021b8b78 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 4 || any
  fix arg0 0x428 u8 range 0 4 rep=6,1

# id < 4 and some M[0x428 + i] == id
fn 021b8b9c ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 5 || any
  fix arg0 0x428 u8 range 0 4 rep=6,1

# v & 1 (r1; r0 unused)
fn 021b8bc4 ret=u32
  nowork
  arg u32 any
  arg u32 any

# 8bc4(M, M[0x46C]) == side
fn 021b8bd0 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 2 || any
  stub 021b8bc4 ret=pick:0/1 args=p,v

# index of the (n+1)-th position i with M[0x428 + i] == id (r1, n = r2), 6 if none
fn 021b8bec ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 4 || any
  arg u32 alt range 0 3 || any
  fix arg0 0x428 u8 range 0 4 rep=6,1

# position of (rule r0, side r1 bit 0, slot r2): side 1 -> slot * 2 (byte), side 0 -> slot * 2 + 1
fn 021b8c20 ret=u32
  nowork
  arg u32 any
  arg u32 any
  arg u32 any

# (pos r0 bit 0, slot r1): pos odd -> slot * 2 + 1, even -> slot * 2 (byte)
fn 021b8c38 ret=u32
  nowork
  arg u32 any
  arg u32 any

# tail call 8c20(rule, side, slot)
fn 021b8c50 ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  stub 021b8c20 ret=u8 args=v,v,v

# (rule r0, pos r1): opposing position for rule 0..3 via 8c20 (rule, pos, slot)
fn 021b8c5c ret=u32
  nowork
  arg u32 alt range 0 4 || any
  arg u32 alt range 0 6 || any
  stub 021b8c20 ret=u8 args=v,v,v

# (M, pos): rule 2 -> (pos + 2) & 3, else pos
fn 021b8cb0 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 6 || any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 4 || any

# (r1 & 1) != (r2 & 1) (r0 unused)
fn 021b8ce8 ret=u32
  nowork
  arg u32 any
  arg u32 any
  arg u32 any

# M[0x428 + pos]
fn 021b8cfc ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 5 || range 0 0xB00

# index i (0..3) of the first ROM range [021efcf0[i], +5] containing v, else 0
fn 021b8d08 ret=u32
  nowork
  arg u32 alt range 0 0x20 || any

# list of the BattleMon ids for a target spec: 8778(M, spec, local) then per position 98ac(pokecon, pos) (via the thunk 021b98a4),
# skipping null / 021d5b68 (fainted?) mons; id = 021d5440(mon) -> out[k]; returns k
fn 021b8d38 ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 alt mask 0x0F07 || any
  arg buf 0x10
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 4 || any
  stub 021b8778 ret=pick:0/1/2/3/4/5/6 args=ptr,v,outm:8:7
  stub 021b98ac force ret=nullable:4:u32 args=v,v
  stub 021d5b68 ret=pick:0/0/1 args=v
  stub 021d5440 ret=u8 args=v
  stub 021b87e8 ret=pick:0/1/2 args=ptr,v,v,out:6
  stub 021b8834 ret=pick:0/1/2/3/4 args=ptr,v,v,out:6
  stub 021b88ec ret=pick:0/1/2/3/4/5/6 args=ptr,v,v,out:6
  stub 021b8c50 ret=pick:0/1/2/3/4/5 args=ptr,v,v
  stub 021b8c20 ret=pick:0/1/2/3/4/5 args=v,v,v
  stub 021b8c38 ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b8cb0 ret=pick:0/1/2/3/4/5 args=ptr,v
  stub 021b99f0 force ret=pick:0/1 args=v

# position of a mon: k = 8d08(id); s = 9864(pokecon, k, id); s < 0 -> 6; else 8bec(M, k, s) (6 stays 6)
fn 021b8d8c ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 alt range 0 0x20 || any
  fix arg0 0x428 u8 range 0 4 rep=6,1
  stub 021b8d08 ret=pick:0/1/2/3 args=v
  stub 021b9864 force ret=pick:0xFFFFFFFF/0/1/2/3 args=v,v,v
  stub 021b8bec ret=pick:0/1/2/3/4/5/6/6 args=ptr,v,v

# 8d8c(M, pokecon, id) == 6 ? 0xFF : 8e70(M, pos)
fn 021b8dc0 ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 alt range 0 0x20 || any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 3 || any
  fix arg0 0x428 u8 range 0 4 rep=6,1
  stub 021b8d8c ret=pick:0/1/2/3/4/5/6/6 args=ptr,v,v
  stub 021b8e70 ret=u8 args=ptr,v
  stub 021b8d08 ret=pick:0/1/2/3 args=v
  stub 021b9864 force ret=pick:0xFFFFFFFF/0/1/2/3 args=v,v,v
  stub 021b8bec ret=pick:0/1/2/3/4/5/6/6 args=ptr,v,v

# number of positions i < pos with M[0x428 + i] == M[0x428 + pos] (8cfc)
fn 021b8dec ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 6 || range 0 0x200
  fix arg0 0x428 u8 range 0 4 rep=6,1
  stub 021b8cfc ret=pick:0/1/2/3/4 args=ptr,v

# *out1 = 8cfc(M, pos); *out2 = number of i < pos with M[0x428 + i] == *out1
fn 021b8e28 ret=none
  nowork
  arg buf 0x1000
  arg u32 alt range 0 6 || range 0 0x200
  arg buf 4
  arg buf 4
  fix arg0 0x428 u8 range 0 4 rep=6,1
  stub 021b8cfc ret=pick:0/1/2/3/4 args=ptr,v

# (M, pos): side = (M[0x46D] & 1) == (pos & 1); rule 0 -> !side; rule 1 -> ROM table 021efcf4[side*3 + (pos >> 1)],
# else 021efcfa[side*3 + (pos >> 1)] (byte)
fn 021b8e70 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 6 || range 0 0x100
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 3 || any

# (M, v): rule 0: v odd -> M[0x46D] == 0, even -> M[0x46D]; else (M[0x46D] & 1) based offset of v - 2 / v - 3 (byte)
fn 021b8ed8 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 8 || any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x484 u32 alt range 0 3 || any
  fix arg0 0x46D u8 alt range 0 1 || any

# 86c8(M) (setup+0x22) && type not in {0, 1}
fn 021b8f38 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A2 u8 alt const 0 || any
  stub 021b86c8 ret=pick:0/1 args=ptr
  stub 021b86b8 ret=pick:0/1/2/3/4 args=ptr

# 969c(M+0xC8, M, id, 0); return 985c(M+0xC8, id)
fn 021b8f5c ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  stub 021b969c force ret=none args=ptr,ptr,v,v
  stub 021b985c force ret=u32 args=ptr,v

# if 86c8(M): id = (v + 2) & 3; if 8b9c(M, id): 969c(M+0xC8, M, id, 0); return 985c(M+0xC8, id). Else 0
fn 021b8f7c ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A2 u8 alt const 0 || any
  stub 021b86c8 ret=pick:0/1/1 args=ptr
  stub 021b8b9c ret=pick:0/1/1 args=ptr,v
  stub 021b969c force ret=none args=ptr,ptr,v,v
  stub 021b985c force ret=u32 args=ptr,v

# 86c8(M) && (v & 2) (r1)
fn 021b8fc0 ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A2 u8 alt const 0 || any
  stub 021b86c8 ret=pick:0/1/1 args=ptr

# tail call 8fc0(M, M[0x46C])
fn 021b8fe4 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A2 u8 alt const 0 || any
  stub 021b8fc0 ret=pick:0/1 args=ptr,v
  stub 021b86c8 ret=pick:0/1/1 args=ptr

# M[0x46C] (own client id)
fn 021b8ff4 ret=u32
  nowork
  arg buf 0x1000

# tail call 9010(M, M[0x46C])
fn 021b9000 ret=u32
  nowork
  arg buf 0x1000
  stub 021b9010 ret=pick:0/1/2/3/4 args=ptr,v
  stub 021b8b9c ret=pick:0/1 args=ptr,v

# partner id: k = (v + 2) & 3; 8b9c(M, k) ? k : 4
fn 021b9010 ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  stub 021b8b9c ret=pick:0/1 args=ptr,v

# ==== sub-proc state machines u32 step(M, u32 *seq) (tables at 0x021EFD04, 6 steps per battle kind) =============
# Each first checks 021d4270() (comm error?): nonzero -> M[0x473] |= 1, return that value. Else switch(*seq).
# Return 1 = finished. seq = arg1 (`arg buf 4`), every state reached through its value (+ out-of-range values).

# comm handshake / setup exchange: 0 send own id (021d4294(M[0x46C])); 1 wait 4348, read 4360/4378 into M[0x46A]/M[0x46E]/
# setup+0x91, seq = 4360 ? 2 : 3; 2 build the 0x20-byte record at M+0x408 (0x18 bytes from M+0x3F0, u16s from setup+0x86/0x74/
# 0x76, bytes from M[0x43C]/M[0x473] or 2/1 per 021ba1a8(M, 0x1000)) and send it (021d43b8(M+0x408)); 3 receive (021d440c(M+0x408))
# and copy it to setup+0x9C, M+0x3F0, setup+0x86 and M[0x45B]/M[0x43C]/M[0x440]/M[0x442] fields, return 1
fn 021b75a8 ret=u32 inputs=600
  nowork
  arg buf 0x1000
  arg buf 4
  fix arg0 0 u32 addrof arg0 0x480
  fix arg1 0 u32 alt range 0 4 || any
  stub 021d4270 ret=pick:0/0/0/0/1
  stub 021d4294 ret=pick:0/1 args=v
  stub 021d4348 ret=pick:0/1
  stub 021d4360 ret=pick:0/1
  stub 021d4378 ret=u8
  stub 021ba1a8 force ret=pick:0/1 args=ptr,v
  stub 021d43b8 ret=pick:0/1 args=in:0x20
  stub 021d440c ret=pick:0/1 args=out:0x20

# comm sync sequence (13 states): 021d4af4(n) / 021d4b18(n) send/wait sync n; 021d4438(setup+0x24) / 021d44cc(); per M[0x471]
# M[0x46E] M[0x468] M[0x470] M[0x472] M[0x2C4] (comm data handle from 021d4d24(0201a918(), M[0x468] & 0x7FFF | 0x8000));
# 021d4d64(handle, setup[0x24 + 4 * M[0x472]]); 021d4680 / 021d4784 polls; state 10: party-ish init 93e0(M+0xC8, M, 0) then
# per client id 0..3 present (8b9c): 021ba128 / 021ba648 (M, id, 021d47a4(id)) and 9460(M+0xC8, M, id); state 11: 93e0(M+0x1B0,
# M, 1) and 9460(M+0x1B0, M, id); state 12: 021D4800(); seq > 12 -> return 1
fn 021b7758 ret=u32 inputs=1000
  nowork
  arg buf 0x1000
  arg buf 4
  fix arg0 0 u32 addrof arg0 0x480
  fix arg1 0 u32 alt range 0 12 || range 0 12 || range 0 13 || any
  fix arg0 0x46E u8 oneof 0 1 0x80
  fix arg0 0x470 u8 oneof 0 1 0x80
  fix arg0 0x471 u8 oneof 0 1 0x80
  fix arg0 0x472 u8 oneof 0 1 0x80
  fix arg0 0x428 u8 range 0 4 rep=6,1
  stub 021d4270 ret=pick:0/0/0/0/1
  stub 021d4af4 ret=none args=v
  stub 021d4b18 ret=pick:0/1 args=v
  stub 021d4438 ret=pick:0/1 args=v
  stub 021d44cc ret=pick:0/1
  stub 0201a918 ret=u32
  stub 021d4d24 ret=u32 args=v,v
  stub 021d4d64 ret=none args=v,v
  stub 021d4680 ret=pick:0/1 args=v
  stub 021d4784 ret=pick:0/1 args=v
  stub 021d4d5c ret=none args=v
  stub 021b93e0 force ret=none args=ptr,p,v
  stub 021b8b9c ret=pick:0/1/1 args=ptr,v
  stub 021d47a4 ret=u32 args=v
  stub 021ba128 force ret=none args=ptr,v,v
  stub 021ba648 force ret=none args=ptr,v,v
  stub 021b9460 force ret=none args=ptr,ptr,v
  stub 021d4800 ret=none

# comm party exchange (11 states): 0/1 sync 4 (4af4 / 4b18); 2 021d4808(M[0xC0]); 3 if 021d4898(): for each client i < M[0x46B]
# with r = 021d48cc(i) != 0: 9e38(M+0x20+0x28i, M[0x468], r); for i != M[0x46C] with setup+0xC1 bit 2: rebuild the name string
# (02045b38 / 02045ec0 / 02045c04, copy <= 15 chars of 02045a90() + terminator 02045a98() into a stack u16[16] given to
# 020084d4(M[0x20+0x28i], buf), M[0x24+0x28i] freed (02045808) and replaced); 021ba670(M, i, M[0x20+0x28i]); 4 M[0x471] == 0 ->
# seq 10, else M[0x470] = 0; 5/6 sync M[0x470] + 8; 7 M[0x472] = M[0x470] ? 3 : 1, if M[0x46E] wait 021d4908(setup[0x48 +
# 4 * M[0x472]]), seq++, then if 021d49b0(): 9ef0(M+0x20+0x28*M[0x472], 021d49cc()), 021d49e0(), ++M[0x470] >= M[0x471] ? seq 10
# : seq 5; 10 021d4a6c() (via the thunk 021d4a04); seq > 10 -> 1. The name buffer's unwritten tail is the zeroed test stack.
fn 021b7968 ret=u32 inputs=1000
  nowork
  arg buf 0x1000
  arg buf 4
  fix arg0 0 u32 addrof arg0 0x480
  fix arg1 0 u32 alt range 0 10 || range 0 10 || range 0 11 || any
  fix arg0 0x46B u8 range 0 4
  fix arg0 0x46C u8 range 0 4
  fix arg0 0x46E u8 oneof 0 1 0x80
  fix arg0 0x470 u8 oneof 0 1 2
  fix arg0 0x471 u8 oneof 0 1 2 3
  fix arg0 0x541 u8 alt mask 4 || any
  stub 021d4270 ret=pick:0/0/0/0/1
  stub 021d4af4 ret=none args=v
  stub 021d4b18 ret=pick:0/1 args=v
  stub 021d4808 ret=pick:0/1 args=v
  stub 021d4898 ret=pick:0/1/1
  stub 021d48cc ret=nullable:3:u32 args=v
  stub 021b9e38 force ret=none args=ptr,v,v
  stub 02045b38 ret=u32 args=v,v,v,v
  stub 02045ec0 ret=u32 args=v,v
  stub 02045c04 ret=none args=v
  stub 02045a90 ret=ptr:arg0:0xE00:1 args=v
  stub 020458f8 ret=pick:0/3/15/20 args=v
  stub 02045a98 ret=u16
  stub 02045808 ret=none args=v
  stub 020084d4 ret=none args=v,in:0x20
  stub 021ba670 force ret=none args=ptr,v,v
  stub 021d4908 ret=pick:0/1 args=v
  stub 021d49b0 ret=pick:0/1
  stub 021d49cc ret=u32
  stub 021b9ef0 force ret=none args=ptr,v
  stub 021d49e0 ret=none
  stub 021d4a6c ret=none

# comm data exchange (4 states): 0 021d4500(), sync 3 (4af4); 1 wait 4b18(3); 2 021d450c(02007b44(02012ebc(setup[0x58]))) (02007b44 =
# thunk with r1 = 0x24, stubbed at its own address like the other specs do); 3 if 021d45fc(): for id 0..3 present (8b9c) and not
# 9f70(M, id): src = 021d4648(id) != 0 -> M[0x3E0 + 4 id] = 02007b20(M[0x468]) and MI_CpuCopy8(src, it, 02007b00()) (real copy);
# 021ba698(M, id) for every id; 021d4674(); seq > 3 -> 1. Copy source zone 0x600, destination zone 0xA00 (0x3EC bytes each).
fn 021b7bb0 ret=u32 inputs=600
  nowork
  arg buf 0x1000
  arg buf 4
  fix arg0 0 u32 addrof arg0 0x480
  fix arg1 0 u32 alt range 0 3 || range 0 4 || any
  fix arg0 0x428 u8 range 0 4 rep=6,1
  stub 021d4270 ret=pick:0/0/0/0/1
  stub 021d4500 ret=none
  stub 021d4af4 ret=none args=v
  stub 021d4b18 ret=pick:0/1 args=v
  stub 02012ebc ret=u32 args=v
  stub 02007b44 ret=u32 args=v
  stub 021d450c ret=pick:0/1 args=v
  stub 021d45fc ret=pick:0/1/1
  stub 021b8b9c ret=pick:0/1/1 args=ptr,v
  stub 021b9f70 force ret=pick:0/0/1 args=ptr,v
  stub 021d4648 ret=nullable:3:ptr:arg0:0x600:1 args=v
  stub 02007b20 ret=ptr:arg0:0xA00:1 args=v
  stub 02007b00 ret=pick:0x3EC
  stub 021ba698 force ret=none args=ptr,v
  stub 021d4674 ret=none

# ---- object creation steps (one shot, seq ignored; return 1) ---------------------------------------------------
# id = M[0x46C], mode = 6818(setup) (server1). M[0x46E] != 0 (this console is the server): M[8] = 021ba9c8(M, M+0x3F0, M+0x1B0,
# mode, M[0x468]) (server3 create), M[0x10 + 4 id] = 021cd6f0(M, M+0xC8, setup[0x20], setup[0x1C], id, 1, 0, mode, 0, M+0x3F0,
# M[0x468]) (client create), 021baa94(M[8], 021cd978(client), id, 1), 021baacc(M[8], setup[0x20], setup[0x1C], id != 0 ? 0 : 1, 1).
# Else: client create as above, M[0xC] = 021ba9c8(...), 021bab10(M[0xC], i, 1) for every present id i (8b9c), 021cd960(client, M[0xC]).
fn 021b7cb4 ret=u32
  nowork
  arg buf 0x1000
  arg buf 4
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x46C u8 alt range 0 3 || any
  fix arg0 0x46E u8 oneof 0 1 0x80
  fix arg0 0x428 u8 range 0 4 rep=6,1
  stub 021b6818 force ret=pick:0/1 args=ptr
  stub 021d4270 ret=pick:0/0/0/0/1
  stub 021ba9c8 force ret=u32 args=p,ptr,p,v,v
  stub 021cd6f0 force ret=u32 args=p,p,v,v,v,v,v,v,v,ptr,v
  stub 021cd978 force ret=u32 args=v
  stub 021baa94 force ret=none args=v,v,v,v
  stub 021baacc force ret=none args=v,v,v,v,v
  stub 021b8b9c ret=pick:0/1/1 args=ptr,v
  stub 021bab10 force ret=none args=v,v,v
  stub 021cd960 force ret=none args=v,v

# multi-console create: id = M[0x46C], mode = 6818(setup), k = setup[0x22] ? 1 : 2. Server console (M[0x46E] != 0): M[8] = server
# create, own client (id) create + 021baa94(M[8], 021cd978(c), id, k); setup[0x22] == 2: partner = 9010(M, id), clients 1 and 3
# created (M[0x14], M[0x1C]) and attached, 021baacc(M[8], setup[0x20], setup[0x1C], partner, k); else 021baacc for every i <
# M[0x46B], i != id. Client console: own client create, M[0xC] = server create, 021bab10(M[0xC], i, 6994(M, i)) for present i,
# 021cd960(client, M[0xC]). Returns 1.
fn 021b7e18 ret=u32
  nowork
  arg buf 0x1000
  arg buf 4
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x46C u8 alt range 0 3 || any
  fix arg0 0x46E u8 oneof 0 1 0x80
  fix arg0 0x46B u8 alt range 0 4 || any
  fix arg0 0x4A2 u8 oneof 0 1 2 2
  fix arg0 0x428 u8 range 0 4 rep=6,1
  stub 021b6818 force ret=pick:0/1 args=ptr
  stub 021d4270 ret=pick:0/0/0/0/1
  stub 021ba9c8 force ret=u32 args=p,ptr,p,v,v
  stub 021cd6f0 force ret=u32 args=p,p,v,v,v,v,v,v,v,ptr,v
  stub 021cd978 force ret=u32 args=v
  stub 021baa94 force ret=none args=v,v,v,v
  stub 021b9010 ret=pick:0/1/2/3/4 args=ptr,v
  stub 021baacc force ret=none args=v,v,v,v,v
  stub 021b8b9c ret=pick:0/1/1 args=ptr,v
  stub 021b6994 force ret=pick:0/1/2/3/4/5/6 args=ptr,v
  stub 021bab10 force ret=none args=v,v,v
  stub 021cd960 force ret=none args=v,v

# 7cb4 with client mode 3: server console: server create, own client create (mode 3), 021baa94(M[8], 021cd978(c), id, 3),
# 021baacc(M[8], setup[0x20], setup[0x1C], id == 0, 3); client console: client create, M[0xC] = server create,
# 021bab10(M[0xC], i, 6994(M, i)) for present i, 021cd960(client, M[0xC]). Returns 1.
fn 021b8070 ret=u32
  nowork
  arg buf 0x1000
  arg buf 4
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x46C u8 alt range 0 3 || any
  fix arg0 0x46E u8 oneof 0 1 0x80
  fix arg0 0x428 u8 range 0 4 rep=6,1
  stub 021b6818 force ret=pick:0/1 args=ptr
  stub 021d4270 ret=pick:0/0/0/0/1
  stub 021ba9c8 force ret=u32 args=p,ptr,p,v,v
  stub 021cd6f0 force ret=u32 args=p,p,v,v,v,v,v,v,v,ptr,v
  stub 021cd978 force ret=u32 args=v
  stub 021baa94 force ret=none args=v,v,v,v
  stub 021baacc force ret=none args=v,v,v,v,v
  stub 021b8b9c ret=pick:0/1/1 args=ptr,v
  stub 021b6994 force ret=pick:0/1/2/3/4/5/6 args=ptr,v
  stub 021bab10 force ret=none args=v,v,v
  stub 021cd960 force ret=none args=v,v

# 7cb4 variant for M[0x46B] consoles: server console: server create, own client create (mode 1), 021baa94(M[8], 021cd978(c), id, 1),
# then 021baacc(M[8], setup[0x20], setup[0x1C], i, 1) for every i < M[0x46B], i != id; client console: as 8070 (mode 1). Returns 1.
fn 021b81dc ret=u32
  nowork
  arg buf 0x1000
  arg buf 4
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x46C u8 alt range 0 3 || any
  fix arg0 0x46E u8 oneof 0 1 0x80
  fix arg0 0x46B u8 alt range 0 4 || any
  fix arg0 0x428 u8 range 0 4 rep=6,1
  stub 021b6818 force ret=pick:0/1 args=ptr
  stub 021d4270 ret=pick:0/0/0/0/1
  stub 021ba9c8 force ret=u32 args=p,ptr,p,v,v
  stub 021cd6f0 force ret=u32 args=p,p,v,v,v,v,v,v,v,ptr,v
  stub 021cd978 force ret=u32 args=v
  stub 021baa94 force ret=none args=v,v,v,v
  stub 021baacc force ret=none args=v,v,v,v,v
  stub 021b8b9c ret=pick:0/1/1 args=ptr,v
  stub 021b6994 force ret=pick:0/1/2/3/4/5/6 args=ptr,v
  stub 021bab10 force ret=none args=v,v,v
  stub 021cd960 force ret=none args=v,v

# ---- view / main-loop steps -------------------------------------------------------------------------------------
# seq 0: M[4] = 021e8f20(M, M[0x10 + 4 * M[0x46C]], M+0xC8, 6818(setup), 0x13) (battle view create), 021cd95c(own client, M[4]);
# server console (M[0x46E]): M[0x464] = 021b8435 (main loop 8434) and 021bab40(M[8]) (server start), else M[0x464] = 021b84d9
# (8434 without server); seq++ and 0. seq != 0: 1.
fn 021b835c ret=u32
  nowork
  arg buf 0x1000
  arg buf 4
  fix arg0 0 u32 addrof arg0 0x480
  fix arg1 0 u32 alt const 0 || any
  fix arg0 0x46C u8 alt range 0 3 || any
  fix arg0 0x46E u8 oneof 0 1 0x80
  stub 021d4270 ret=pick:0/0/0/0/1
  stub 021b6818 force ret=pick:0/1 args=ptr
  stub 021e8f20 ret=u32 args=p,v,p,v,v
  stub 021cd95c force ret=none args=v,v
  stub 021bab40 force ret=none args=v

# one frame without a local server: 021bab9c(M[8]) (result ignored); every client c with 021cd97c(c): 021cdd94(c, M+0x448)
# (result pair) and r = 1; 021e908c(M[4]) (view frame); returns r
fn 021b83f4 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0x10 u32 alt const 0 || any rep=4,4
  stub 021bab9c force ret=pick:0/1 args=v
  stub 021cd97c force ret=pick:0/0/1 args=v
  stub 021cdd94 force ret=none args=v,out:8
  stub 021e908c ret=none args=v

# main loop (M[0x464] on the server console): M[0x473] bit 0 set: bit 1 set -> 1, else continue; bit 0 clear: 021d4270() -> set
# bit 0 and 8548(M) (quit the clients). Then r = 021bab9c(M[8]) (server step) -> 021babdc(M[8], M+0x448); clients with
# M[0xC4 + i] == 0 get M[0xC4 + i] = 021cd97c(c); if 8564(M) (all done): 021cdd94(own client, M+0x448), r = 1; 021e908c(M[4]); r
fn 021b8434 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0x473 u8 alt mask 3 || any
  fix arg0 0x46C u8 alt range 0 3 || any
  fix arg0 0x10 u32 alt const 0 || any rep=4,4
  fix arg0 0xC4 u8 alt const 0 || any rep=4,1
  stub 021d4270 ret=pick:0/0/1
  stub 021b8548 ret=none args=ptr
  stub 021cd914 force ret=none args=v
  stub 021bab9c force ret=pick:0/1 args=v
  stub 021babdc force ret=none args=v,out:8
  stub 021cd97c force ret=pick:0/0/1 args=v
  stub 021b8564 ret=pick:0/1 args=ptr
  stub 021cdd94 force ret=none args=v,out:8
  stub 021e908c ret=none args=v

# main loop of a client console (M[0x464] = 021b84d9): the M[0x473] / 021d4270 / 8548 prologue of 8434, then the 83f4 client pass
# without the server step: every c with 021cd97c(c): 021cdd94(c, M+0x448), r = 1; 021e908c(M[4]); r
fn 021b84d8 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0x473 u8 alt mask 3 || any
  fix arg0 0x10 u32 alt const 0 || any rep=4,4
  stub 021d4270 ret=pick:0/0/1
  stub 021b8548 ret=none args=ptr
  stub 021cd914 force ret=none args=v
  stub 021cd97c force ret=pick:0/0/1 args=v
  stub 021cdd94 force ret=none args=v,out:8
  stub 021e908c ret=none args=v
