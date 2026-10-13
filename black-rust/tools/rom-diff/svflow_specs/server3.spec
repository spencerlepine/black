# svflow spec: server3 (0x021B902C-0x021BAA94; B11-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (no new syntax). Conventions (shared with server1 / server2, see the server1.spec header):
# `nowork`; the object a function works on is `arg buf N` with dereferenced pointer fields fixed into data zones of the same
# buffer (`fix arg0 OFS u32 addrof arg0 ZONE`); closure stubs (every block lists the stubs of its callees and of the callees of
# all its in-area descendants, so porting in any order needs no spec edit: checked by running every function with the whole
# area treated as ported); callees in server1 / server2 and in the other svflow areas are `force` stubs.
# Thunks in this range (not specced; callers stub the target): 021b917c -> 021ba54c, 021b98a4 -> 021b98ac, 021b99b8 -> 021b6994.
#
# ======== first half (0x021B902C-0x021B9B40) ========
# Harness-job owned; porting workers may not edit. Grammar: svflow_specs/README.md. Conventions: B11-spec (see area header).
#
# Objects confirmed from the code of this half:
#   M (main module, 0x474 bytes; `arg buf 0x1000`, M at offset 0):
#     +0x000 BATTLE_SETUP* (fixed to arena 0x480: `fix arg0 0 u32 addrof arg0 0x480`), +0x010.. 4 client pointers (9b10: c = M[0x10 + 4*cl]),
#     +0x0C8 pokecon #2 (0xE8 bytes) and +0x1B0 pokecon #1 (92d4 / 9358 read both; 9244 passes M+0x1B0 to 9af8),
#     +0x298 / +0x2A8 4-word tables of party sources (ba178), +0x2B8 scratch party (969c / 97a0 hand it to 0201A948/A98C/AC2C),
#     +0x2C0 Pokemon for Zoroark's Illusion (9460 allocates it once with 020173F0), +0x3E0 4 per-client view objects (994c / 997c),
#     +0x428 u8[6] client id per battle position (9120 / 98ac; 8b9c / 8cfc in server2), +0x430 u32 value doubled once (919c, bit 6),
#     +0x434 u32 money-like counter capped at 99999 (91e4 / 9210), +0x438 u32 (9244), +0x444 u32 (916c: set only while 7),
#     +0x468 u16 (passed to 02007F1C/7E50, 020173F0, 0201B7A0 | 0x8000), +0x46C u8 own client id, +0x46E u8 flag (9b10),
#     +0x473 flag byte: bit 3 (919c done / 91e4 frozen), bit 4 (9244 done), bit 5 (92a0), bit 6 (doubling, 919c / 9210 / 9234).
#   BATTLE_SETUP (arena 0x480): +0x00 battle type (<= 1 checked by 9244 / 92b0), +0x23 u8 comm flag, +0x44 u8[4] (997c), +0x58,
#     +0x60 / +0x64 objects (90e8 / 90f0), +0x80 u8 (9244), +0x90 u8 (9120: == 6 then rewritten to a count), +0x94 (90f8), +0xC1 flag
#     byte bit 0 (9184), +0xC2 u8[6] (97a0 sets 1 for an Illusion mon).
#   Pokecon PC (0xE8 bytes; init 93e0(pc, M, flag)): +0x00 M, +0x04 4 parties * 0x1C, +0x74 4 words = the party source of each client
#     (985c), +0x84 24 BattleMon pointers (9934 / 9940; 9460 stores 021d4d84 results there, 9424 / 9674 free them with 021D53B8),
#     +0xE4 u32 flag handed to ba178.
#   Party (0x1C bytes, PC + 4 + client * 0x1C): +0x00 6 BattleMon pointers, +0x18 u8 count (9b40 clear, 9b54 append, 9b94 count,
#     9bf0 / 9c00 get with bound, 9c24 rotate to the end, 9b60 dead to the end, 9b98 alive count).
#   ROM tables: 0x021EFCF0 u8[4] first BattleMon slot per client {0, 12, 6, 18} (9460 / 97a0); 0x021EFD70 6 records of 10 bytes
#     {u16 n0, u16 n1, u8 list0[3], u8 list1[3]} (99e0 returns record i, 9a6c searches both lists).
# Specs of pokecon functions: PC is `arg buf 0x100` (M as a separate buffer when the function dereferences it); the 4 party count
#   bytes (PC+0x1C/0x38/0x54/0x70) are fixed to 0..6 so a real 9bf0/9c00/9b54/9c24 never leaves the buffer.
# Stubs of 9aec / 9af8 (party address) return `ptr:argK:4:1` (party 0) so that porting 9bf0 / 9b94 / 9b98 before them stays safe.
# 9a6c: arg 2 is NOT an argument of the function (it ignores r2): a data block that the 99e0 stub points into (`ptr:arg2:0:1`).
# Ghidra errors in this half: 9064 has 3 args (r0 = M only handed to 8b9c), 9a24 / 99c0 / 9a50 / 9b04 pass r0..r1 through to callees,
#   937c ignores r0 (args r1 = pokecon, r2 = mon), 990c / 9920 are tail calls (`bx r3`) to 9bf0 / 9c00, 9460 takes (pc, M, client).
#
# ======== second half (0x021B9B40-0x021BAA94) ========
# Conventions: /tmp/blk/b11/CONVENTIONS.md (closure stubs, `force` on other areas, nowork + arena buffers).
#
# Objects confirmed from the code of this half:
#
# Party container P (`arg buf 0x20`; 4 of them inline in each pokecon, see below):
#   +0x00..+0x14  BattleMon *mon[6]        +0x18 u8 count   (9b40 init, 9b54 append, 9c24 remove-and-push-back, 9c10 swap,
#   9c50 rotate slot 0 with the 9ab0/9ac8 slot of a position, 9bf0/9c00 get, 9b94 count, 9b98/9bc4 count alive (021d5b7c), 9c98 / 9cbc find)
#
# Pokecon PC (inline in M at M+0xC8 and M+0x1B0; 0xE8 bytes):
#   +0x04 + 0x1C*i  party container of client i (4)   (021b9aec)
#   +0x74 + 4*i     Pokemon party pointer of client i (021b985c)
#   +0x84 + 4*j     BattleMon pointer of mon id j (24) (021b9940)
#   +0xE4           value passed to 021ba178 as the "use copy" flag
#
# Main module M (`arg buf 0x1000`, M itself 0x474 bytes; *M = BATTLE_SETUP fixed to arena 0x480 when dereferenced):
#   +0x000 BATTLE_SETUP*   +0x004 view/handle (021e9034 / 021e8f20 result)   +0x008 S (btl_server.c object, 021bb964 / bb8bc / bb8dc / bab40)
#   +0x010 + 4*i client object pointers (021cdda0 / cddcc / cd924 / cd95c / cd984 / cd9b0)
#   +0x020 + 0x28*i 4 player records (9db8 clear, 9de8 free, 9e38/9e7c/9ef0 fill a record):
#        rec +0x00 ptr (status/trainer data, NULL = empty -> 9f70 true), +0x04 ptr (strbuf / name), +0x08 u16, +0x0A u16,
#        +0x0C u32, +0x10 u16[4], +0x18 8 bytes (0202428c/02024570), +0x20 8 bytes; seen through M: +0x20 ptr0, +0x24 ptr1,
#        +0x28 u16, +0x2A u16, +0x2C u32, +0x30 u16[4] (9f84), +0x38 / +0x40 sub-records (ba02c)
#   +0x0C8 pokecon 0, +0x1B0 pokecon 1
#   +0x298 + 4*i party copies (0201a920 allocs, 4), +0x2A8 + 4*i second party set (4), +0x2B8 one more party, +0x2BC (ba08c)
#   +0x2C0 Pokemon* (illusion partner, created by 020173f0 in 9460, compared in 9d10)
#   +0x3E0 + 4*i 4 pointers (freed by 9de8; 02007b50 / 02007c70 objects in 997c)
#   +0x3F0 0x18-byte block (021d787c init in ba73c; copied to setup+0x9C by ba6c0)
#   +0x428 u8[6] client ids (021b8b9c), +0x440 u16, +0x442 u16, +0x444 u32 result code, +0x448 block (021d826c / 8270),
#   +0x468 u16 (flags into 0201b7a0 / b7b4 / b818, bit 15 forced), +0x46C u8 own client id, +0x473 u8 flag bit 0
#
# BATTLE_SETUP fields used here (zone arena 0x480): +0x00 battle type, +0x21 u8 client id, +0x23 u8 comm flag, +0x24 + 4*i / +0x34 + 4*i
#   per-client pointers (0201ac2c / 02008494 targets), +0x44 + i u8, +0x6C ptr, +0x70 ptr, +0x7F u8, +0x8C u32 result, +0x90 u8,
#   +0x94 ptr (record buffer) +0x98 u32 size, +0x9C 0x18 bytes, +0xB4 u16, +0xB6..+0xBF u8 stats, +0xC0 u8, +0xC2 + i u8,
#   +0xC8 + 6*k + j u8[4][6] (ba2e4), +0xE0 + 4*k u32
#
# Server object S (btl_server.c, 0xCC4 bytes; create 021ba9c8 = Heap_AllocDebug(heap, 0xCC4, 1, "btl_server.c", 0xAD)):
#   +0x00 state fn (021babec(S, 0x021BAC05))  +0x0C main module M (arg r0)  +0x10 r2 (pokecon / party container arg)
#   +0x14 + 12*i 4 client slots (021bb828 marks each unused)  +0x44 flow work (021bb968(S, M, S[0x10], S+0xF0, S[0x4C], heap))
#   +0x4C r3  +0x50 0x18 bytes copied from *r1  +0xAC StrBuf (020457b0(0x20, heap))  +0xB0 021bc550() result  +0xED u8 = 0
#   +0xF0 inline area (S+0xCB0 points at it)  +0xCB0 = S+0xF0  +0xCBA u8 = 0  +0xCBF u8 = 0  +0xCC0 u16 heap id
#   021d787c(r1) is called on the 0x18-byte source block (init/consume, stubbed).
#
# Ghidra errors in this half: 021ba9c8 decompiled as `void F(void)` (all 4 register args, the stack heap id, the whole body
# after the alloc and the return of S are dropped); 021ba080 / 021ba1a8 / 021ba820 are tail calls through `bx r3` (021ba080 ->
# 0202e7dc, 021ba1a8 -> 02013158, 021ba820 -> 021bb964), Ghidra shows them as indirect calls.
# Fake pointers / zones: setup at arena 0x480, ptr results of 021b8714 at arena 0x600, record buffers at 0x700 / 0x900.
# Calls to ARM main-module helpers 0201xxxx/0202xxxx/0200xxxx are plain stubs; server1/server2/server_side/core1/api/client
# callees are `force` stubs.
area server3

# ---- small pure helpers -------------------------------------------------------------------------------------------------

# a == b, or a == (b + 2) & 3 (same side / partner position)
fn 021b902c ret=u32
  nowork
  arg u32 alt range 0 4 || any
  arg u32 alt rel 0 const 0 || rel 0 const 2 || range 0 4 || any

# 9064(M, M[0x46C], b) & 0xFF
fn 021b904c ret=u8
  nowork
  arg buf 0x1000
  arg u32 alt const 0 || any
  fix arg0 0x46C u8 range 0 3
  stub 021b9064 ret=pick:0/1/2/3 args=v,v,v
  stub 021b8b9c force ret=pick:0/1 args=v,v

# r = (a & 1) ^ 1; if b: c = r + ((b & 1) << 1) (u8); if 8b9c(M, c) r = c; return r
fn 021b9064 ret=u32
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  stub 021b8b9c force ret=pick:0/1 args=v,v

# if setup[0x23] == 0 and cl == M[0x46C]: 02007F1C(setup[0x60], x, 1, M[0x468])
fn 021b9090 ret=none
  nowork
  arg buf 0x1000
  arg u32 alt range 0 3 || any
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A3 u8 alt const 0 || any
  fix arg0 0x46C u8 range 0 3
  stub 02007f1c ret=none args=v,v,v,v

# same as 9090 with 02007E50
fn 021b90bc ret=none
  nowork
  arg buf 0x1000
  arg u32 alt range 0 3 || any
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A3 u8 alt const 0 || any
  fix arg0 0x46C u8 range 0 3
  stub 02007e50 ret=none args=v,v,v,v

# setup[0x60]
fn 021b90e8 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# setup[0x64]
fn 021b90f0 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# setup[0x94] != 0 && setup[0x23] == 0
fn 021b90f8 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x514 u32 alt const 0 || any
  fix arg0 0x4A3 u8 alt const 0 || any

# setup[0x23] (comm flag)
fn 021b9114 ret=u8
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# returns 0 (callers pass 2 unread args)
fn 021b911c ret=u32
  nowork

# if setup[0x90] == 6: c = 8cfc(M, n); setup[0x90] = number of positions p < n (u8 loop) with M[0x428 + p] == c
fn 021b9120 ret=none
  nowork
  arg buf 0x1000
  arg u32 alt range 0 6 || any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x510 u8 alt const 6 || any
  fix arg0 0x428 u8 range 0 3 rep=6,1
  stub 021b8cfc force ret=pick:0/1/2/3 args=v,v

# if M[0x444] == 7: M[0x444] = v
fn 021b916c ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 0x444 u32 alt const 7 || range 0 8

# setup[0xC1] |= 1
fn 021b9184 ret=none
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480

# unless flag bit 3: (bit 6: M[0x430] *= 2), M[0x430] = 0201119C(M[0x430]), set bit 3; return M[0x430]
fn 021b919c ret=u32
  nowork
  arg buf 0x1000
  stub 0201119c ret=u32 args=v

# unless flag bit 3: M[0x434] += v, capped at 99999
fn 021b91e4 ret=none
  nowork
  arg buf 0x1000
  arg u32 alt range 0 1000 || any
  fix arg0 0x434 u32 alt range 0 100000 || any

# M[0x434], doubled (capped at 99999) when flag bit 6
fn 021b9210 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0x434 u32 alt range 0 100000 || any

# flag |= 0x40
fn 021b9234 ret=none
  nowork
  arg buf 0x1000

# unless flag bit 4: if setup[0] <= 1: M[0x438] = min(021d7ff8(setup[0x80], 9af8(M+0x1B0, M[0x46C])), 0200C274(02012F08(setup[0x58])));
# flag |= 0x10; returns M[0x438] as stored, 0 when the flag was set or setup[0] > 1 (V-0022; client4 021d1410 / 021d16b0 read it)
fn 021b9244 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x480 u32 alt range 0 2 || any
  stub 02012f08 ret=u32 args=v
  stub 0200c274 ret=pick:0/1/50/100/0xFFFFFFFF args=v
  stub 021b9af8 ret=ptr:arg0:0x1B4:1 args=v,v
  stub 021d7ff8 ret=pick:0/1/50/100/0xFFFFFFFF args=v,v

# flag |= 0x20
fn 021b92a0 ret=none
  nowork
  arg buf 0x1000

# if setup[0] <= 1 and setup[0x23] == 0: 92d4(M, mon, b ? 5 : 4)
fn 021b92b0 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 alt const 0 || any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x480 u32 alt range 0 2 || any
  fix arg0 0x4A3 u8 alt const 0 || any
  stub 021b92d4 ret=none args=v,v,v
  # closure (92d4)
  stub 021d5440 ret=pick:0/1/5/6/11/12/17/23 args=v
  stub 021b9940 ret=u32 args=v,v
  stub 021d5844 ret=u32 args=v
  stub 021b8714 force ret=ptr:arg0:0x600:1 args=v
  stub 0201b7a0 ret=none args=v,v,v,v

# id = 021d5440(mon); for the mon of id in pokecon #1 (M+0x1B0) and #2 (M+0xC8): 0201B7A0(pokemon, x, s[0xA], M[0x468] | 0x8000), s = 8714(M)
fn 021b92d4 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  stub 021d5440 ret=pick:0/1/5/6/11/12/17/23 args=v
  stub 021b9940 ret=u32 args=v,v
  stub 021d5844 ret=u32 args=v
  stub 021b8714 force ret=ptr:arg0:0x600:1 args=v
  stub 0201b7a0 ret=none args=v,v,v,v

# 021d54f4(9934(M+0xC8, id), 9934(M+0x1B0, id))
fn 021b9358 ret=u32
  nowork
  arg buf 0x1000
  arg u32 range 0 23
  stub 021b9934 ret=u32 args=v,v
  stub 021d54f4 ret=u32 args=v,v

# (r0 unused, pc, mon): if 021d6c24(mon): the party member of mon's client whose Pokemon is mon's visible one (Illusion), else mon
fn 021b937c ret=u32
  nowork
  arg u32 const 0
  arg buf 0x100
  arg u32 any
  fix arg1 0x1C u8 range 0 6 rep=4,0x1C
  stub 021d6c24 ret=pick:0/1/1 args=v
  stub 021d5440 ret=u32 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v   # through the thunk 021b8ddc
  stub 021d5864 ret=pick:1/2/3 args=v
  stub 021b9af8 ret=ptr:arg1:4:1 args=v,v
  stub 021b9b94 ret=pick:0/1/3/6 args=v
  stub 021b9c00 ret=u32 args=v,v
  stub 021d5844 ret=pick:1/2/3 args=v

# ---- pokecon (0xE8 bytes) ------------------------------------------------------------------------------------------------

# init: pc[0] = M, pc[0xE4] = flag, 4 parties cleared (9b40), 24 BattleMon slots and the 4 party sources zeroed
fn 021b93e0 ret=none
  nowork
  arg buf 0x100
  arg u32 any
  arg u32 any
  stub 021b9b40 ret=none args=v

# free: every non-null BattleMon slot -> 021D53B8 + zero; the 4 parties cleared
fn 021b9424 ret=none
  nowork
  arg buf 0x100
  fix arg0 0x84 u32 alt const 0 || any rep=24,4
  stub 021d53b8 ret=none args=v
  stub 021b9b40 ret=none args=v

# set up client cl: party source src = ba178(M, cl, pc[0xE4]); one BattleMon per Pokemon of src (021d4d84) into the slots from
# 0x021EFCF0[cl] and into party cl; Illusion (ability 0x95) partners; for client 1 in a non-link battle a Zoroark (species 0x23B)
# with Illusion gets a disguise from client 0's species 243/244/245 (M[0x2C0], created once by 020173F0); then 9b60(party)
fn 021b9460 ret=none inputs=400
  nowork
  arg buf 0x100
  arg buf 0x1000
  arg u32 alt const 1 || range 0 3
  # the parties are empty when a client is set up (93e0 -> 9b40 cleared them); the append loop adds up to 6 mons
  fix arg0 0x1C u8 const 0 rep=4,0x1C
  fix arg1 0x2C0 u32 alt const 0 || any
  stub 021ba178 ret=u32 args=v,v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=u32 args=v,v
  stub 021d4d84 ret=u32 args=v,v,v
  stub 021b9b54 ret=none args=v,v
  stub 021b9634 ret=pick:0xFFFFFFFF/0/1/2/5 args=v
  stub 021d59b4 ret=pick:0x95/0x95/0/1 args=v,v
  stub 021d5848 ret=none args=v,v
  stub 021b86b8 force ret=pick:0/0/1 args=v
  stub 021d5444 ret=pick:0x23B/0x23B/1 args=v
  stub 02017e1c ret=pick:243/244/245/0/1 args=v,v,v
  stub 020173f0 ret=u32 args=v,v,v,v,v
  stub 021b9b60 ret=none args=v
  # closure (9b60)
  stub 021d5b7c ret=pick:0/1 args=v
  stub 021b9c24 ret=none args=v,v

# index of the last Pokemon of src with 02017E1C(p, 0x4C, 0) == 0 and 02017E1C(p, 0xA0, 0) != 0, else -1
fn 021b9634 ret=u32
  nowork
  arg u32 any
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=u32 args=v,v
  stub 02017e1c ret=pick:0/1 args=v,v,v

# free the 24 BattleMon slots (021D53B8 + zero)
fn 021b9674 ret=none
  nowork
  arg buf 0x100
  fix arg0 0x84 u32 alt const 0 || any rep=24,4
  stub 021d53b8 ret=none args=v

# (pc, M, cl, x): sync party cl back into its source: 021d6b84(mon, x), add each Pokemon to M[0x2B8], match Illusion disguises,
# 0201AC2C(M[0x2B8], src), 021d6c3c(mon, src[i]), disguises re-applied
fn 021b969c ret=none
  nowork
  arg buf 0x100
  arg buf 0x300
  arg u32 range 0 3
  arg u32 any
  fix arg0 0x1C u8 range 0 6 rep=4,0x1C
  stub 021b985c ret=u32 args=v,v
  stub 021b9aec ret=ptr:arg0:4:1 args=v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201a948 ret=none args=v
  stub 021b9bf0 ret=u32 args=v,v
  stub 021d6b84 ret=none args=v,v
  stub 021d5844 ret=pick:1/2/3 args=v
  stub 0201a98c ret=none args=v,v
  stub 021d6c24 ret=pick:0/1 args=v
  stub 021d5864 ret=pick:1/2/3 args=v
  stub 0201ac2c ret=none args=v,v
  stub 0201ab28 ret=u32 args=v,v
  stub 021d6c3c ret=none args=v,v
  stub 021d5848 ret=none args=v,v

# (pc, M, cl): for slot 0x021EFCF0[cl] + i the party member with that id: 021d6b84(mon, 1), add its Pokemon to M[0x2B8],
# setup[0xC2 + i] = 1 if 021d67d0(mon); then 0201AC2C(M[0x2B8], src)
fn 021b97a0 ret=none
  nowork
  arg buf 0x100
  arg buf 0x1000
  arg u32 range 0 3
  fix arg0 0x1C u8 range 0 6 rep=4,0x1C
  fix arg1 0 u32 addrof arg1 0x480
  stub 021b985c ret=u32 args=v,v
  stub 021b9aec ret=ptr:arg0:4:1 args=v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201a948 ret=none args=v
  stub 021b9bf0 ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/6/7/12/13/18/19 args=v
  stub 021d6b84 ret=none args=v,v
  stub 021d5844 ret=u32 args=v
  stub 0201a98c ret=none args=v,v
  stub 021d67d0 ret=pick:0/1 args=v
  stub 0201ac2c ret=none args=v,v

# pc[0x74 + 4 * cl] (party source)
fn 021b985c ret=u32
  nowork
  arg buf 0x100
  arg u32 alt range 0 3 || range 0 34

# index in party cl of the BattleMon whose id (021d5440) is id, else -1
fn 021b9864 ret=u32
  nowork
  arg buf 0x100
  arg u32 range 0 3
  arg u32 alt range 0 3 || any
  fix arg0 0x1C u8 range 0 6 rep=4,0x1C
  stub 021b9b94 ret=pick:0/1/3/6 args=v
  stub 021b9c00 ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3 args=v

# BattleMon at position pos: client c = 8cfc(M, pos), k = positions below pos with client c, party c member k (0 if k >= count)
fn 021b98ac ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 6 || any
  fix arg0 0 u32 addrof arg0 0x400
  fix arg0 0x828 u8 range 0 3 rep=6,1
  fix arg0 0x1C u8 range 0 6 rep=4,0x1C
  stub 021b8cfc force ret=pick:0/1/2/3 args=v,v
  stub 021b9b94 ret=pick:0/1/2/6 args=v
  stub 021b9c00 ret=u32 args=v,v

# tail call 9bf0(pc + 4 + cl * 0x1C, i)
fn 021b990c ret=u32
  nowork
  arg buf 0x100
  arg u32 range 0 3
  arg u32 alt range 0 6 || any
  fix arg0 0x1C u8 range 0 6 rep=4,0x1C
  stub 021b9bf0 ret=u32 args=v,v

# tail call 9c00(pc + 4 + cl * 0x1C, i)
fn 021b9920 ret=u32
  nowork
  arg buf 0x100
  arg u32 range 0 3
  arg u32 alt range 0 6 || any
  fix arg0 0x1C u8 range 0 6 rep=4,0x1C
  stub 021b9c00 ret=u32 args=v,v

# pc[0x84 + 4 * id]
fn 021b9934 ret=u32
  nowork
  arg buf 0x100
  arg u32 range 0 30

# pc[0x84 + 4 * id] (same code as 9934)
fn 021b9940 ret=u32
  nowork
  arg buf 0x100
  arg u32 range 0 30

# ---- main-module view objects / misc -------------------------------------------------------------------------------------

# v = M[0x3E0 + 4 * 8cfc(M, 8ed8(M, x & 0xFF))]; if v: *out = v, 1 else 0
fn 021b994c ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  arg buf 4
  fix arg0 0x3E0 u32 alt const 0 || any rep=4,4
  stub 021b8ed8 force ret=pick:0/1/2/3/4/5 args=v,v
  stub 021b8cfc force ret=pick:0/1/2/3 args=v,v   # through the thunk 021b8de4

# non-link: p = M[0x3E0 + 4 i]; p && 02007B50(p) ? 02007C70(p) : 0; link (9114 != 0): setup[0x44 + i]
fn 021b997c ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 3 || range 0 40
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x3E0 u32 alt const 0 || any rep=44,4
  stub 021b9114 ret=pick:0/1 args=v
  stub 02007b50 ret=pick:0/1 args=v
  stub 02007c70 ret=u32 args=v

# (M, r1 passed through): r = 6994(M, r1); + 2 if 8588(M) == 3 (u8)
fn 021b99c0 ret=u32
  nowork
  arg u32 any
  arg u32 any
  stub 021b6994 force ret=pick:0/1/2/3/4/0xFF args=v,v
  stub 021b8588 force ret=pick:0/1/3/3 args=v

# &table[i], table = 0x021EFD70 (10-byte records)
fn 021b99e0 ret=u32
  nowork
  arg u32 any

# x == 2 || x == 3
fn 021b99f0 ret=u32
  nowork
  arg u32 alt range 0 5 || any

# if !99f0(x): *out = 8c20(2, x, (x >> 1) & 0xFF), 1; else 0
fn 021b9a00 ret=u32
  nowork
  arg u32 alt range 0 5 || any
  arg buf 4
  stub 021b99f0 ret=pick:0/1 args=v
  stub 021b8c20 force ret=u32 args=v,v,v

# 9a40(a) == 9a40(b)
fn 021b9a24 ret=u32
  nowork
  arg u32 alt range 0 23 || any
  arg u32 alt range 0 23 || any
  stub 021b9a40 ret=pick:0/1 args=v

# id >= 12 (side of a BattleMon id)
fn 021b9a40 ret=u32
  nowork
  arg u32 alt range 0 24 || any

# 9a5c(9a40(id)) (the other side)
fn 021b9a50 ret=u32
  nowork
  arg u32 alt range 0 24 || any
  stub 021b9a40 ret=pick:0/1 args=v
  stub 021b9a5c ret=u32 args=v

# x == 0
fn 021b9a5c ret=u32
  nowork
  arg u32 alt const 0 || any

# v is in list0 or list1 of record i of 0x021EFD70 (99e0); arg 2 = data block the 99e0 stub points to (not a real argument)
fn 021b9a6c ret=u32
  nowork
  arg u32 range 0 5
  arg u32 alt range 0 7 || any
  arg buf 0x10
  fix arg2 0 u16 range 0 3
  fix arg2 2 u16 range 0 3
  fix arg2 4 u8 range 0 7 rep=6,1
  stub 021b99e0 ret=ptr:arg2:0:1 args=v

# 2 -> 1, 3 -> 2, else 0
fn 021b9ab0 ret=u32
  nowork
  arg u32 alt range 0 5 || any

# 2 -> 2, 3 -> 1, else 0
fn 021b9ac8 ret=u32
  nowork
  arg u32 alt range 0 5 || any

# x & 1
fn 021b9ae0 ret=u32
  nowork
  arg u32 any

# pc + 4 + cl * 0x1C (party address)
fn 021b9aec ret=u32
  nowork
  arg u32 any
  arg u32 any

# pc + 4 + cl * 0x1C (same code as 9aec)
fn 021b9af8 ret=u32
  nowork
  arg u32 any
  arg u32 any

# alive count of party cl: 9b98(9af8(pc, cl))
fn 021b9b04 ret=u32
  nowork
  arg buf 0x100
  arg u32 range 0 3
  fix arg0 0x1C u8 range 0 6 rep=4,0x1C
  stub 021b9af8 ret=ptr:arg0:4:1 args=v,v
  stub 021b9b98 ret=pick:0/1/2/6 args=v
  # closure (9b98)
  stub 021d5b7c ret=pick:0/1 args=v

# if M[0x46E] and 8b9c(M, cl) and the client object M[0x10 + 4 cl] != 0: 021cd9f4(client, v)
fn 021b9b10 ret=none
  nowork
  arg buf 0x1000
  arg u32 alt range 0 3 || range 0 0x300
  arg u32 any
  fix arg0 0x46E u8 alt const 0 || any
  fix arg0 0x10 u32 alt const 0 || any rep=4,4
  stub 021b8b9c force ret=pick:0/1/1 args=v,v
  stub 021cd9f4 force ret=none args=v,v

# ==== party container (P = arg buf 0x20) ====================================================================================

# P.count = 0, P.mon[0..5] = NULL
fn 021b9b40 ret=none
  nowork
  arg buf 0x20

# P.mon[P.count++] = mon (no bound check)
fn 021b9b54 ret=none
  nowork
  arg buf 0x40
  arg u32 any
  fix arg0 0x18 u8 alt range 0 6 || range 0 15

# drop the dead mons: for each slot, 021d5b7c(mon) == 0 -> 9c24(P, i) (move it to the end)
fn 021b9b60 ret=none
  nowork
  arg buf 0x20
  fix arg0 0x18 u8 range 0 6
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021b9c24 ret=none args=v,v

# P.count
fn 021b9b94 ret=u8
  nowork
  arg buf 0x20

# number of mons with 021d5b7c(mon) != 0
fn 021b9b98 ret=u8
  nowork
  arg buf 0x20
  fix arg0 0x18 u8 range 0 6
  stub 021d5b7c ret=pick:0/1 args=v

# number of mons from index `start` on with 021d5b7c(mon) != 0 (signed compare with count)
fn 021b9bc4 ret=u8
  nowork
  arg buf 0x20
  arg u32 range 0 7
  fix arg0 0x18 u8 range 0 6
  stub 021d5b7c ret=pick:0/1 args=v

# P.mon[i] if i < count (unsigned) else NULL
fn 021b9bf0 ret=u32
  nowork
  arg buf 0x20
  arg u32 alt range 0 7 || any
  fix arg0 0x18 u8 alt range 0 6 || any

# same code as 9bf0 (second accessor)
fn 021b9c00 ret=u32
  nowork
  arg buf 0x20
  arg u32 alt range 0 7 || any
  fix arg0 0x18 u8 alt range 0 6 || any

# swap P.mon[i] and P.mon[j]
fn 021b9c10 ret=none
  nowork
  arg buf 0x20
  arg u32 range 0 5
  arg u32 range 0 5

# remove P.mon[i] and put it at index count-1 (others shift down)
fn 021b9c24 ret=none
  nowork
  arg buf 0x20
  arg u8 range 0 6
  fix arg0 0x18 u8 range 0 6

# position swap: pos 0/1 -> nothing; else a = 9ab0(pos), b = 9ac8(pos): mon[0] <- mon[a], mon[a] <- mon[b], mon[b] <- old mon[0];
# optional outs: *out1 = old mon[0], *out2 = new mon[0]
fn 021b9c50 ret=none
  nowork
  region o1 4
  region o2 4
  arg buf 0x20
  arg u32 alt range 0 5 || any
  arg u32 alt const 0 || addrof region:o1
  arg u32 alt const 0 || addrof region:o2
  stub 021b9ab0 ret=pick:0/1/2 args=v
  stub 021b9ac8 ret=pick:0/1/2 args=v

# index of mon pointer `m` in P, or -1
fn 021b9c98 ret=u32
  nowork
  arg buf 0x20
  arg u32 range 0 3
  fix arg0 0 u32 range 0 3 rep=6,4
  fix arg0 0x18 u8 range 0 6

# index of the mon whose id (021d5440) is `id`, or -1
fn 021b9cbc ret=u32
  nowork
  arg buf 0x20
  arg u32 range 0 3
  fix arg0 0x18 u8 range 0 6
  stub 021d5440 ret=pick:0/1/2/3 args=v

# first mon with 021d5b7c(mon) != 0, or NULL
fn 021b9ce8 ret=u32
  nowork
  arg buf 0x20
  fix arg0 0x18 u8 range 0 6
  stub 021d5b7c ret=pick:0/0/1 args=v

# Illusion refresh of a party: last alive index L (from the end, 021d5b7c); start = 3 if 021b8588(M) == 3 else 021b8b78(M, r2);
# for i in start..count: species (BattleMon_Get(mon, 0x11)) == 0x95 (Zoroark): i < L -> SetIllusionMon(mon, GetPokemon(mon[L])),
# else ClearIllusionMon unless GetVisibleMon(mon) == M[0x2C0]; then 021d6c24(mon) for every mon. Args (M, P, r2).
fn 021b9d10 ret=none
  nowork
  arg buf 0x1000
  arg buf 0x20
  arg u32 any
  fix arg0 0x2C0 u32 const 0x1234
  fix arg1 0x18 u8 range 0 6
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b8b78 force ret=pick:0/1/2/3/6 args=v,v
  stub 021d59b4 ret=pick:0x95/0x95/1 args=v,v
  stub 021d5844 ret=u32 args=v
  stub 021d5848 ret=none args=v,v
  stub 021d5864 ret=pick:0x1234/5 args=v
  stub 021d5854 ret=none args=v
  stub 021d6c24 ret=none args=v

# ==== player records in M (M+0x20 + 0x28*i) ==================================================================================

# clear the 4 records (+0x20, +0x2A, +0x2C) and the 4 words at M+0x3E0
fn 021b9db8 ret=none
  nowork
  arg buf 0x1000

# free (020307b0) the non-null record pointers +0x20 / +0x24 and M+0x3E0[i], null them
fn 021b9de8 ret=none
  nowork
  arg buf 0x1000
  fix arg0 0x20 u32 alt const 0 || any rep=4,0x28
  fix arg0 0x24 u32 alt const 0 || any rep=4,0x28
  fix arg0 0x3E0 u32 alt const 0 || any rep=4,4
  stub 020307b0 ret=none args=v

# fill a record from (r1, r2): rec[0] = 0200846c(r1); 02008494(r2, rec[0]); rec+0xA = 0; rec+8 = 02008550(rec[0]) != 0;
# rec[1] = 02008510(rec[0], 0x12); init rec+0x18 / rec+0x20 (0202428c)
fn 021b9e38 ret=none
  nowork
  arg buf 0x28
  arg u32 any
  arg u32 any
  stub 0200846c ret=u32 args=v
  stub 02008494 ret=none args=v,v
  stub 02008550 ret=pick:0/1 args=v
  stub 02008510 ret=u32 args=v,v
  stub 0202428c ret=none args=v

# fill a record from a source block (NULL -> empty record: rec[0] = rec[1] = 0, u16s 0, init +0x18 / +0x20):
# +0xA = src[0], +8 = src[1], rec[1] = 02045874(src[0x14], 0x12), rec[3] = src[2], copy 8 bytes src+0xC -> rec+0x10,
# 02024570(rec+0x18, src+0x18), 02024570(rec+0x20, src+0x20)
fn 021b9e7c ret=none
  nowork
  region src 0x40
  arg buf 0x28
  arg u32 alt const 0 || addrof region:src
  stub 02045874 ret=u32 args=v,v
  stub 02024570 ret=none args=v,v
  stub 0202428c ret=none args=v

# same as 9e7c but rec[1] = new StrBuf 020457b0(0x20, 0x12) filled by 02045a00(strbuf, src+0x28, src[0x68] + 1)
fn 021b9ef0 ret=none
  nowork
  region src 0x80
  arg buf 0x28
  arg u32 alt const 0 || addrof region:src
  stub 020457b0 ret=u32 args=v,v
  stub 02045a00 ret=none args=v,v,v
  stub 02024570 ret=none args=v,v
  stub 0202428c ret=none args=v

# record i empty? (M[0x20 + 0x28*i] == NULL)
fn 021b9f70 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 3 || range 0 50
  fix arg0 0x20 u32 alt const 0 || any rep=51,0x28

# record i u16 [+0x30 + 2k] (k < 4) if 9f70(M, i), else 0
fn 021b9f84 ret=u32
  nowork
  arg buf 0x1000
  arg u32 range 0 3
  arg u32 alt range 0 5 || any
  fix arg0 0x20 u32 alt const 0 || any rep=4,0x28
  stub 021b9f70 ret=pick:0/1/1 args=v,v

# *out = u16 [+0x48 + 0x28*i]; returns [+0x44 + 0x28*i]
fn 021b9fac ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 3 || range 0 50
  arg buf 4

# battle type 0: 021ba1a8(M, 8) -> 0x800, else 021b8588(M) == 1 -> 0x80, else 0;
# other types: 9f70(M, i) ? [+0x4C + 0x28*i] : 0
fn 021b9fbc ret=u32
  nowork
  arg buf 0x1000
  arg u32 range 0 3
  fix arg0 0x20 u32 alt const 0 || any rep=4,0x28
  stub 021b86b8 force ret=pick:0/1/2 args=v
  stub 021ba1a8 ret=pick:0/1 args=v,v
  stub 02013158 ret=pick:0/1 args=v,v
  stub 021b8588 force ret=pick:0/1/2 args=v
  stub 021b9f70 ret=pick:0/1 args=v,v

# 9f70(M, i) ? u16 [+0x4A + 0x28*i] : 0
fn 021ba004 ret=u32
  nowork
  arg buf 0x1000
  arg u32 range 0 3
  fix arg0 0x20 u32 alt const 0 || any rep=4,0x28
  stub 021b9f70 ret=pick:0/1 args=v,v

# u16 [+0x48 + 0x28*i]
fn 021ba020 ret=u32
  nowork
  arg buf 0x1000
  arg u32 alt range 0 3 || range 0 50

# battle type 2 and 9f70(M, i): flag == 1 -> &M[0x60 + 0x28*i], else &M[0x58 + 0x28*i]; otherwise NULL
fn 021ba02c ret=ofs:arg0
  nowork
  arg buf 0x1000
  arg u32 range 0 3
  arg u32 oneof 0 1 2
  fix arg0 0x20 u32 alt const 0 || any rep=4,0x28
  stub 021b86b8 force ret=pick:0/1/2/2 args=v
  stub 021b9f70 ret=pick:0/1/1 args=v,v

# 9f70(M, i) ? 0 : [+0x20 + 0x28*i]
fn 021ba064 ret=u32
  nowork
  arg buf 0x1000
  arg u32 range 0 3
  fix arg0 0x20 u32 alt const 0 || any rep=4,0x28
  stub 021b9f70 ret=pick:0/1 args=v,v

# tail call 0202e7dc(setup[0x70])
fn 021ba080 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  stub 0202e7dc ret=u32 args=v

# M[0x2BC]
fn 021ba08c ret=u32
  nowork
  arg buf 0x1000

# ==== party copies M+0x298.. ===============================================================================================

# allocate 4 + 4 + 1 parties: M[0x298 + 4i] / M[0x2A8 + 4i] / M[0x2B8] = 0201a920(0x12)
fn 021ba094 ret=none
  nowork
  arg buf 0x1000
  stub 0201a920 seq ret=u32 args=v

# free (020307b0) the non-null party pointers M[0x298 + 4i], M[0x2A8 + 4i], M[0x2B8], null them
fn 021ba0cc ret=none
  nowork
  arg buf 0x1000
  fix arg0 0x298 u32 alt const 0 || any rep=9,4
  stub 020307b0 ret=none args=v

# if M[0x298 + 4i]: clear both parties of client i (0201a948), copy `src` into both (0201ac2c(src, dst)), then if the second
# one is not empty 0201ab28(second, 0)
fn 021ba128 ret=none
  nowork
  arg buf 0x1000
  arg u32 range 0 3
  arg u32 any
  fix arg0 0x298 u32 alt const 0 || any rep=8,4
  stub 0201a948 ret=none args=v
  stub 0201ac2c ret=none args=v,v
  stub 0201aa18 ret=pick:0/1/6 args=v
  stub 0201ab28 ret=u32 args=v,v

# flag ? (M[0x2A8 + 4i] after 0201ab28(p, 0) when not empty) : M[0x298 + 4i]
fn 021ba178 ret=u32
  nowork
  arg buf 0x1000
  arg u32 range 0 3
  arg u32 alt const 0 || any
  stub 0201aa18 ret=pick:0/1/6 args=v
  stub 0201ab28 ret=u32 args=v,v

# tail call 02013158(*M, r1)
fn 021ba1a8 ret=u32
  nowork
  arg buf 0x1000
  arg u32 any
  stub 02013158 ret=pick:0/1 args=v,v

# setup[0x23] == 0 -> 02008df0(setup[0x6c], r1)
fn 021ba1b4 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A3 u8 alt const 0 || any
  stub 02008df0 ret=none args=v,v

# setup[0x23] == 0 -> 02008dac(setup[0x6c], r1, r2)
fn 021ba1cc ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A3 u8 alt const 0 || any
  stub 02008dac ret=none args=v,v,v

# *M + 0x78
fn 021ba1e4 ret=u32
  nowork
  arg buf 0x1000

# 021b6818(setup) == 1 && setup[0x7f] != 0
fn 021ba1ec ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4FF u8 alt const 0 || any
  stub 021b6818 force ret=pick:0/1 args=v

# 021b6818(setup) == 1 ? ba1ec(M) : (battle type 0 or 1)
fn 021ba20c ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4FF u8 alt const 0 || any
  stub 021b6818 force ret=pick:0/1 args=v
  stub 021ba1ec ret=pick:0/1 args=v
  stub 021b86b8 force ret=pick:0/1/2/3 args=v

# 92d4(M, mon, 0) then 021bb8bc(S = M[8], mon)
fn 021ba238 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  stub 021b92d4 ret=none args=v,v,v
  stub 021bb8bc force ret=none args=v,v
  # closure of 92d4
  stub 021d5440 ret=pick:0/1/5/12/23 args=v
  stub 021b9940 ret=u32 args=v,v
  stub 021d5844 ret=u32 args=v
  stub 021b8714 force ret=ptr:arg0:0x600:1 args=v
  stub 0201b7a0 ret=none args=v,v,v,v

# like 92d4 with 0201b7b4: id = 021d5440(mon); for both pokecons (M+0x1B0, M+0xC8) the mon of that id, its Pokemon;
# 0201b7b4(pp, r2, (*8714(M))+0xA, M[0x468] | 0x8000) for each
fn 021ba250 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  arg u32 any
  stub 021d5440 ret=pick:0/1/5/12/23 args=v
  stub 021b9940 ret=u32 args=v,v
  stub 021d5844 ret=u32 args=v
  stub 021b8714 force ret=ptr:arg0:0x600:1 args=v
  stub 0201b7b4 ret=none args=v,v,v,v

# S = M[8]; S != NULL -> 021bb8dc(S, r1)
fn 021ba2d4 ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 8 u32 alt const 0 || any
  stub 021bb8dc force ret=none args=v,v

# ==== end of battle: results back into the setup ==========================================================================

# Not in comm mode (setup[0x23] == 0):
#  type <= 1: 97a0(PC1, M, id); party = 985c(PC1, id); type 1 && 021d7f60(ba020(M, 1)) -> 0201b818(party, 8714(M)->+0xA, M[0x468]|0x8000);
#    type 1: for each mon of the party: the setup-side Pokemon setup[0x24] slot k vs party slot k item (02017e1c(pp, 6, 0)) ->
#    021d805c(item, 0x10) ? (re-set the item when it differs: 02017e40) : 02017e40(pp2, 6, item); then copy the party into
#    setup[0x24] (0201ac2c) and touch each mon (0201ab28).
#  type 0: client k = 9064(M, id, 0); 969c(PC1, M, k, 1); 0201ac2c(985c(PC1, k), setup[0x28]).
#  type != 4: for c in 0..3: k = ba710(id, c), setup+0xC8+6k zeroed; if 8b9c(M, k): for each mon of PC0's container k:
#    status byte setup[0xC8 + 6k + (021d5440(mon) - base[k])] = 2 if 021d5b68(mon), 1 if 021d62a8(mon); sums of Get(mon, 0xD)
#    and Get(mon, 0xE); setup[0xE0 + 4k] = 100 * sum13 / sum14 (_u32_div_f, real code).
fn 021ba2e4 ret=none
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x480 u32 range 0 5
  fix arg0 0x4A3 u8 alt const 0 || any
  fix arg0 0x46C u8 range 0 3
  fix arg0 0xE4 u8 range 0 6 rep=4,0x1C
  fix arg0 0x1CC u8 range 0 6 rep=4,0x1C
  stub 021b97a0 ret=none args=v,v,v
  stub 021b985c ret=u32 args=v,v
  stub 021ba020 ret=u32 args=v,v
  stub 021d7f60 ret=pick:0/1 args=v
  stub 021b8714 force ret=ptr:arg0:0x600:1 args=v
  stub 0201b818 ret=none args=v,v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=u32 args=v,v
  stub 02017e1c ret=pick:0/1/2 args=v,v,v
  stub 021d805c ret=pick:0/1 args=v,v
  stub 02017e40 ret=none args=v,v,v
  stub 0201ac2c ret=none args=v,v
  stub 021b9064 ret=pick:0/1/2/3 args=v,v,v
  stub 021b969c ret=none args=v,v,v,v
  stub 021ba710 ret=pick:0/1/2/3 args=v,v
  stub 021b8b9c force ret=pick:0/1/1 args=v,v
  stub 021b9aec ret=u32 args=v,v
  stub 021b9b94 ret=pick:0/1/2/3/6 args=v
  stub 021b9bf0 ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/6/12/23 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d62a8 ret=pick:0/1 args=v
  # closure of 97a0 / 969c
  stub 0201a948 ret=none args=v
  stub 021d6b84 ret=none args=v,v
  stub 021d5844 ret=u32 args=v
  stub 0201a98c ret=none args=v,v
  stub 021d67d0 ret=pick:0/1 args=v
  stub 021d6c24 ret=pick:0/1 args=v
  stub 021d5864 ret=u32 args=v
  stub 021d6c3c ret=none args=v,v
  stub 021d5848 ret=none args=v,v

# battle type 2 or 3
fn 021ba534 ret=u32
  nowork
  arg buf 0x1000
  stub 021b86b8 force ret=pick:0/1/2/3/4 args=v

# result code: type 4 -> 1; M[0x473] bit 0 -> 6; setup[0x90] != 6 -> 5; 021d826c(M+0x448) -> 021d8270(M+0x448, M[0x46C], type);
# else (M[0x444] == 7 -> M[0x444] = 1) M[0x444]; stored to setup[0x8C] and returned
fn 021ba54c ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x510 u8 alt const 6 || any
  fix arg0 0x444 u32 alt const 7 || range 0 8
  stub 021b86b8 force ret=pick:0/1/2/4 args=v
  stub 021d826c ret=pick:0/1 args=v
  stub 021d8270 ret=u32 args=v,v,v

# u16 M[0x442]
fn 021ba5c4 ret=u16
  nowork
  arg buf 0x1000

# u16 M[0x440]
fn 021ba5d0 ret=u16
  nowork
  arg buf 0x1000

# comm: 021cdda0(client[M[0x46C]]); else M[0x440] ? 021cdda0(client[M[0x46C]]) : 0
fn 021ba5d8 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A3 u8 alt const 0 || any
  fix arg0 0x440 u16 alt const 0 || any
  fix arg0 0x46C u8 alt range 0 3 || any
  stub 021cdda0 force ret=u32 args=v

# comm: 021cddcc(client[M[0x46C]]) -> M[0x444] = 2; returns the 021cddcc result (0 when not comm)
fn 021ba618 ret=u32
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A3 u8 alt const 0 || any
  fix arg0 0x46C u8 alt range 0 3 || any
  stub 021cddcc force ret=pick:0/1 args=v

# k = ba710(M[0x46C], r1); setup[0x24 + 4k] != 0 -> 0201ac2c(r2, it)
fn 021ba648 ret=none
  nowork
  arg buf 0x1000
  arg u32 range 0 3
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A4 u32 alt const 0 || any rep=4,4
  stub 021ba710 ret=pick:0/1/2/3 args=v,v
  stub 0201ac2c ret=none args=v,v

# k = ba710(M[0x46C], r1); setup[0x34 + 4k] != 0 -> 02008494(r2, it)
fn 021ba670 ret=none
  nowork
  arg buf 0x1000
  arg u32 range 0 3
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4B4 u32 alt const 0 || any rep=4,4
  stub 021ba710 ret=pick:0/1/2/3 args=v,v
  stub 02008494 ret=none args=v,v

# k = ba710(M[0x46C], r1); setup[0x44 + k] = 997c(M, r1)
fn 021ba698 ret=none
  nowork
  arg buf 0x1000
  arg u32 alt range 0 3 || range 0 200
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x3E0 u32 alt const 0 || any rep=4,4
  stub 021ba710 ret=pick:0/1/2/3 args=v,v
  stub 021b997c ret=u8 args=v,v
  # closure of 997c
  stub 021b9114 ret=pick:0/1 args=v
  stub 02007b50 ret=pick:0/1 args=v
  stub 02007c70 ret=u32 args=v

# 90f8(M) (record mode) and src = 021cd924(client[M[0x46C]], &size) != 0: copy size bytes src -> setup[0x94] (MI_CpuCopy8),
# setup[0x98] = size, setup[0x9C..0xB4) = M[0x3F0..0x408)
fn 021ba6c0 ret=none
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x514 u32 addrof arg0 0x700
  fix arg0 0x4A3 u8 alt const 0 || any
  fix arg0 0x46C u8 alt range 0 3 || any
  stub 021b90f8 ret=pick:0/1/1 args=v
  stub 021cd924 force ret=nullable:3:ptr:arg0:0x900:1 args=v,outm:4:0x7F/0/0/0

# relation of two client ids: equal -> 0, same side (bit 0) -> 2, else b <= 1 ? 1 : 3
fn 021ba710 ret=u8
  nowork
  arg u32 alt range 0 5 || any
  arg u32 alt range 0 5 || any

# ==== battle end / module teardown =========================================================================================

# st = 6818(setup); 021d787c(M+0x3F0); 68d8(M, setup); 9424(PC0); 9424(PC1); for each client c with 8b9c(M, c):
# 9460(PC1, M, c), 9460(PC0, M, c), 021cd984(client[c], r1); 021e9034(M[4]); 021e8e28(0x13);
# M[4] = 021e8f20(M, client[setup[0x21]], PC0, st, 0x13); 021cd95c(client[setup[0x21]]); 021bab40(M[8])
fn 021ba73c ret=none
  nowork
  arg buf 0x1000
  arg u32 any
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 0x4A1 u8 range 0 3
  fix arg0 0xE4 u8 range 0 6 rep=4,0x1C
  fix arg0 0x1CC u8 range 0 6 rep=4,0x1C
  fix arg0 0x2C0 u32 alt const 0 || any
  stub 021b6818 force ret=pick:0/1 args=v
  stub 021d787c ret=none args=v
  stub 021b68d8 force ret=none args=v,v
  stub 021b9424 ret=none args=v
  stub 021b8b9c force ret=pick:0/1/1 args=v,v
  stub 021b9460 ret=none args=v,v,v
  stub 021cd984 force ret=none args=v,v
  stub 021e9034 ret=none args=v
  stub 021e8e28 ret=none args=v
  stub 021e8f20 ret=u32 args=v,v,v,v,v
  stub 021cd95c force ret=none args=v
  stub 021bab40 force ret=none args=v
  # closure of 9424 / 9460 (and 9460's 9634 / 9b54 / 9b60 / ba178)
  stub 021d53b8 ret=none args=v
  stub 021b9b40 ret=none args=v
  stub 021ba178 ret=u32 args=v,v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=u32 args=v,v
  stub 021d4d84 ret=u32 args=v,v,v
  stub 021b9b54 ret=none args=v,v
  stub 021b9634 ret=pick:0/1/2/0xFFFFFFFF args=v
  stub 021d59b4 ret=pick:0x95/0x95/7/20 args=v,v
  stub 021d5848 ret=none args=v,v
  stub 021b86b8 force ret=pick:0/0/1 args=v
  stub 021d5444 ret=pick:0x23B/0x23B/1 args=v
  stub 02017e1c ret=pick:0xF3/0xF4/0xF5/0/1 args=v,v,v
  stub 020173f0 ret=u32 args=v,v,v,v,v
  stub 021b9b60 ret=none args=v
  stub 021d5b7c ret=pick:0/1 args=v
  stub 021b9c24 ret=none args=v,v

# for each client c with 8b9c(M, c): 021cd9b0(client[c])
fn 021ba7f8 ret=none
  nowork
  arg buf 0x1000
  stub 021b8b9c force ret=pick:0/1 args=v,v
  stub 021cd9b0 force ret=none args=v

# tail call 021bb964(M[8]) (S -> flow work)
fn 021ba820 ret=u32
  nowork
  arg buf 0x1000
  stub 021bb964 force ret=u32 args=v

# clear the result bytes setup[0xB6..0xC0] and u16 setup[0xB4] (arg = BATTLE_SETUP)
fn 021ba82c ret=none
  nowork
  arg buf 0x100

# statistics into the setup: S = M[8] (NULL -> nothing); w = 021bb964(S); setup[0xB6] = 021c8138(w), [0xB7] = 021c8140(w, 0),
# [0xB8..0xBC] = 021c7cbc / 7cc8 / 7cd4 / 7cdc / 7ce8(w); setup[0x8C] == 1 -> [0xBD] = 1; own party (PC1 container 0):
# for each mon [0xC0] += 021d54b4(mon), u16 [0xB4] += Get(mon, 0xD); [0xBF] = count - alive; [0xBE] = count - alive of
# container 1, plus container 3 when 8b9c(M, 3) and 9aec != NULL
fn 021ba878 ret=none
  nowork
  arg buf 0x1000
  fix arg0 0 u32 addrof arg0 0x480
  fix arg0 8 u32 alt const 0 || any
  fix arg0 0x50C u32 alt const 1 || any
  fix arg0 0x1CC u8 range 0 6 rep=4,0x1C
  stub 021bb964 force ret=u32 args=v
  stub 021c8138 force ret=u32 args=v
  stub 021c8140 force ret=u32 args=v,v
  stub 021c7cbc force ret=u32 args=v
  stub 021c7cc8 force ret=u32 args=v
  stub 021c7cd4 force ret=u32 args=v
  stub 021c7cdc force ret=u32 args=v
  stub 021c7ce8 force ret=u32 args=v
  stub 021b9aec ret=nullable:4:u32 args=v,v
  stub 021b9b94 ret=pick:0/1/2/3/6 args=v
  stub 021b9b98 ret=pick:0/1/2 args=v
  stub 021b9bf0 ret=u32 args=v,v
  stub 021d54b4 ret=u32 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021b8b9c force ret=pick:0/1 args=v,v
  # closure of 9b98
  stub 021d5b7c ret=pick:0/1 args=v

# ==== btl_server.c: create the server object =================================================================================

# S = 02030734(heap, 0xCC4, 1, "btl_server.c", 0xAD) (Heap_AllocDebug); S+0xC = M (r0), S+0x10 = r2, S+0xCC0 = heap (stack u16),
# S+0xCB0 = S+0xF0, S+0xED = 0, S+0x44 = 0, S+0xCBA = 0, S+0xCBF = 0, S+0x4C = r3, S+0x50 = 0x18 bytes from r1,
# S+0xAC = 020457b0(0x20, heap); 021d787c(r1); 021bb828(slot i) for the 4 slots; S+0x44 = 021bb968(S, M, S[0x10], S+0xF0,
# S[0x4C], heap); S+0xB0 = 021bc550(); 021babec(S, 0x021BAC05). Returns S.
# The allocation lands in the extra argument buffer 5 (a 6th stack word the function never reads).
fn 021ba9c8 ret=ofs:arg5
  nowork
  arg u32 any
  arg buf 0x20
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x1000
  stub 02030734 ret=ptr:arg5:0:1 args=v,v,v,p,v
  stub 020457b0 ret=u32 args=v,v
  stub 021d787c ret=none args=v
  stub 021bb828 force ret=none args=v
  stub 021bb968 force ret=u32 args=v,v,v,v,v,v
  stub 021bc550 force ret=u32
  stub 021babec force ret=none args=v,v
