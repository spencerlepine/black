# svflow spec: api (0x021C7CBC-0x021C8810; B10bc-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs .. B12-spec additions). No new grammar.
#
# What this area is: the flow API that the event handlers (abilities / items / moves / effects, 0x021D85F4+) call with the
# flow work pointer: SvFlowWork field getters (u16 words at +0x1F74..+0x1F86, +0x10, pointers +0x1C / +0x3E0 / +0x1FEC),
# pokeID -> BattleMon / position / client lookups through the main module (work + 4) and the pokecon (work + 8),
# small lists in the work (+0x782 action records of 0x10 bytes from +0x7E0, +0x788 list of 6 at +0x78B, +0x785 list of
# 0x18 at +0x791, flag byte +0x78A), wrappers around core flow functions (damage / heal / stat paths in core1..core6),
# and the move-record pointer table rebuild 021c85d4 (work + 0x874 + idx * 0x290 into the pointers +0x850..+0x870,
# +0x4A8..+0x4B4, +0x1AB0/+0x1AB4) with its push / pop wrappers 8588 / 859c / 85b8 (index at +0x1A64).
#
# Register facts the pseudo-C hides (read off the disassembly):
#  * the small getters (7cbc 7cc8 7cd4 7cdc 7ce8 8114 8120 812c 8130 8138 8140 830c) are real functions, not veneers;
#  * 7d00 takes (work, pokeID): r1 goes on unchanged to the `bx pc` veneer 7d24 (-> 0689d5c8(work + 0x1AB8, pokeID), a
#    6-entry table search returning the index or 6); 7d54 takes 3 args (Ghidra 4); 7dac returns 6e3c's result (Ghidra
#    void) and its r1 is passed on to the first 9934 call; 7e14 takes two stack words (sp+0x48 / +0x4C); 7f68 ignores r3;
#  * 8030 calls 021d80bc with 4 register arguments (main, pokecon, mon, r2) (the pushed r3 is not read);
#  * 82d0 ignores r0; 82b8 tail-calls 021b911c, which returns 0 and reads nothing; 84ac calls 021c2128(work, r1);
#  * 8534 hands its own stack slot (the pushed r3/r4) to 0689d558 as a 6-byte out buffer: if that ov95 search found 5-6
#    entries it would overwrite the low half of the caller's saved r4 (latent original bug; the stub writes nothing);
#  * 7f4c (test-and-set of work[0x78A] bit 3) is not in the Ghidra function list (it follows 7f24's literal pool).
#
# Layout (core4 convention, closure stubs): every block lists the stubs of its callees and of its in-area descendants'
# callees, plus their resource lines, so any porting order needs no spec edit (checked: with every api fn treated as
# ported, all blocks run without a fault, a stray write or an unstubbed callee). Callees in other server-flow areas
# (server_side 021bb85c / bb908 / bb914, core1 bc2b0 / bc948, core2 bd1e0 / bd2b4 / bd34c / bdbb8 / be52c, core3 bfa00 /
# bfeac / bff60, core4 c1e74 / c2128 / c2ab0 / c2af0 / c2e4c, core5 c64b4, core6 c6e3c / c6f0c, events cc668) are `force`
# stubs: the api ports call them through env.call and their own specs verify them. Main-module / pokecon / BattleMon /
# ARM9 callees and the `bx pc` veneers into overlay 95 (7d24 82e4 845c 8578 87a0 87b0 87c0 87d0 87e0 87f0 8800) are
# plain stubs. Thunks (`ldr r3,[pc]; bx r3`) 021b8ddc (-> 021b8d08) and 021b98a4 (-> 021b98ac) run and land on the
# stubs of their targets. Pure thunks of this area (81c8 8288 83cc 83d4 84d0 84d8 84e0) are specced as functions.
#
# Not specced (cannot run: tail calls into overlay 95, which is not loaded in the battle image): 7d34 (-> 0689d5f4),
# 7d44 (-> 0689d5fc), 814c (-> 0689d588), 82c4 (-> 06898cf4), 82f4 (-> 068982ac); the 2-byte `bx pc` veneers listed above
# and the ARM halves 82e8 / 857c. Callers stub those addresses.

area api

# getter: u16 work[0x1F7C]
fn 021c7cbc ret=u16
  arg work

# getter: u16 work[0x1F7E]
fn 021c7cc8 ret=u16
  arg work

# getter: u16 work[0x1F80]
fn 021c7cd4 ret=u16
  arg work

# getter: u16 work[0x1F82]
fn 021c7cdc ret=u16
  arg work

# getter: u16 work[0x1F86]
fn 021c7ce8 ret=u16
  arg work

# battle mon by pokeID: tail call 021b9934(work[8], r1) (284 callers)
fn 021c7cf4 ret=u32
  arg work
  arg u32 alt range 0 5 || any
  stub 021b9934 force ret=u32 args=v,v

# pokeID -> position: 7d24(work + 0x1AB8, pokeID) (veneer); 6 unless 021b86a0(work[4], pos)
fn 021c7d00 ret=u32
  arg work
  arg u32 alt range 0 5 || any
  stub 021b86a0 force ret=pick:0/1 args=v,v
  stub 021c7d24 ret=pick:0/1/2/3/4/5/6 args=p,v

# lists the pokeIDs of the mons (bd2b4 / bd34c iterator over the work) not on the side of r1 into the byte list r2;
# returns the count
fn 021c7d54 ret=u32
  arg work
  arg u32 any
  arg buf 0x40
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021bd2b4 force ret=none args=ptr,ptr
  stub 021bd34c force ret=pick:0/1/1 args=ptr,ptr,out:4 seq
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# (work, atk pokeID, def pokeID, move): counter work[0x774] around 64b4 (fills a 0x14-byte move record) and
# 6e3c (type check); returns 6e3c's result
fn 021c7dac ret=u32
  arg work
  arg u32 alt range 0 5 || any
  arg u32 alt range 0 5 || any
  arg u32 any
  stub 021b937c force ret=u32 args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021c64b4 force ret=none args=p,v,v,out:0x14
  stub 021c6e3c force ret=u32 args=ptr,v,v,in:0x10,v
  stub 021d6c24 ret=pick:0/1 args=v

# (work, atk, def, move, stack0 = use 7dac, stack1 = flag): 0 for move 0 or 0201c314(move) == 0; else 1e74 with the
# record from 64b4 and returns the u16 it writes to the stack
fn 021c7e14 ret=u16 inputs=300
  arg work
  arg u32 alt range 0 5 || any
  arg u32 alt range 0 5 || any
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  stub 0201c314 ret=pick:0/1/1 args=v
  stub 021b937c force ret=u32 args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021c1e74 force ret=none args=ptr,v,v,in:0x10,v,v,v,v,out:2
  stub 021c64b4 force ret=none args=p,v,v,out:0x14
  stub 021c6e3c force ret=u32 args=ptr,v,v,in:0x10,v
  stub 021c7dac ret=u32 args=ptr,v,v,v
  stub 021d6c24 ret=pick:0/1 args=v

# work[0x774] != 0
fn 021c7ec4 ret=bool
  arg work
  fix work 0x774 u32 alt const 0 || any

# 021b997c(work[4], 021b8d08(pokeID))
fn 021c7ed8 ret=u32
  arg work
  arg u32 alt range 0 5 || any
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b997c force ret=u32 args=v,v

# byte +9 of server_side 021bb85c(work[0], client)
fn 021c7eec ret=u8
  arg work
  arg u32 alt range 0 5 || any
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021bb85c force ret=ptr:work:0x400:8 args=ptr,v

# 3 when the battle rule 021b8588(work[4]) is 3, else 7eec
fn 021c7f04 ret=u32
  arg work
  arg u32 alt range 0 5 || any
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021bb85c force ret=ptr:work:0x400:8 args=ptr,v
  stub 021c7eec ret=u32 args=ptr,v

# 021b9bc4(7fcc(work, id), 7f04(work, id)) != 0
fn 021c7f24 ret=bool
  arg work
  arg u32 alt range 0 5 || any
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b9af8 force ret=u32 args=v,v
  stub 021b9bc4 force ret=pick:0/1 args=v,v
  stub 021bb85c force ret=ptr:work:0x400:8 args=ptr,v
  stub 021c7eec ret=u32 args=ptr,v
  stub 021c7f04 ret=u32 args=ptr,v
  stub 021c7fcc ret=u32 args=ptr,v

# test-and-set work[0x78A] bit 3: 0 if it was set, else sets it and returns 1 (not in the Ghidra fn list)
fn 021c7f4c ret=bool
  arg work

# positions from 021b8778(work[4], r1, out) -> pokeIDs of the mons there that are not fainted, into the byte list r2
# Z20: 021b98ac (the mon at a position) is `nullable`: the original tests the mon for null before 021d5b68 (an empty position)
fn 021c7f68 ret=u32
  arg work
  arg u32 any
  arg buf 8
  stub 021b8778 force ret=pick:0/1/2/3/4/5/6 args=v,v,outm:8:7
  stub 021b98ac force ret=nullable:4:u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5b68 ret=pick:0/1 args=v

# 021b9af8(work[8], client(pokeID)): the party of the pokeID's client
fn 021c7fcc ret=u32
  arg work
  arg u32 alt range 0 5 || any
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b9af8 force ret=u32 args=v,v

# the partner client's party (021b9010 / 021b8b9c on work[4]), 0 when there is none
fn 021c7fe0 ret=u32
  arg work
  arg u32 alt range 0 5 || any
  stub 021b8b9c force ret=pick:0/1 args=v,v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b9010 force ret=pick:0/1/2/3/4 args=v,v
  stub 021b9af8 force ret=u32 args=v,v

# tail call 021b8d8c(work[4], work[8], r1)
fn 021c8010 ret=u32
  arg work
  arg u32 any
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v

# pokeID of the mon at position r1: 5440(021b98ac(work[8], r1))
fn 021c8020 ret=u32
  arg work
  arg u32 any
  stub 021b98ac force ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# 021d80bc(work[4], work[8], mon(r1), r2)
fn 021c8030 ret=u32
  arg work
  arg u32 alt range 0 5 || any
  arg u32 any
  stub 021b9934 force ret=u32 args=v,v
  stub 021d80bc ret=u32 args=v,v,v,v

# finds the action record (work[0x782] records of 0x10 bytes, mon at +0x7E0, word +0x7E4) of pokeID r1 whose kind
# (word & 0xF) is not 6; writes the word to *r2 and returns 1
fn 021c804c ret=bool
  arg work
  arg u32 range 0 5
  arg buf 4
  fix work 0x782 u8 alt range 0 6 || any
  fix work 0x7E4 u32 alt const 6 || any bits=0:4 rep=8,16
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# 1 when every action record of a mon other than r1 has byte +0x7ED set
fn 021c80ac ret=bool
  arg work
  arg u32 range 0 5
  fix work 0x782 u8 alt range 0 6 || any
  fix work 0x7ED u8 alt const 0 || any rep=8,16
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# getter: u16 work[0x1F74]
fn 021c8114 ret=u16
  arg work

# work + 0x1FEC
fn 021c8120 ret=ofs:work
  arg work

# work + 0x1C
fn 021c812c ret=ofs:work
  arg work

# work + 0x3E0
fn 021c8130 ret=ofs:work
  arg work

# turn counter: low half of work[0x10]
fn 021c8138 ret=u16
  arg work

# byte work[0x7D9 + r1]
fn 021c8140 ret=u8
  arg work
  arg u32 range 0 0x80

# tail call 021b8588(work[4]) (battle rule)
fn 021c815c ret=u32
  arg work
  stub 021b8588 force ret=pick:0/1/2/3 args=v

# tail call 021b86b8(work[4])
fn 021c8168 ret=u32
  arg work
  stub 021b86b8 force ret=pick:0/1/2/3 args=v

# word +4 of 021b8714(work[4])
fn 021c8174 ret=u32
  arg work
  stub 021b8714 force ret=ptr:work:0x200:4 args=v

# tail call 021b8670(work[4])
fn 021c8180 ret=u32
  arg work
  stub 021b8670 force ret=pick:0/1/2/3 args=v

# 1 when 0201b718(work[0x4A4], species, 0, i) holds for some i < 7
fn 021c818c ret=bool
  arg work
  arg u32 alt range 0 5 || any
  stub 0201b718 ret=pick:0/0/0/1 args=v,v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5444 ret=u32 args=v

# thunk to core1 021bc948(work, mon, flag)
fn 021c81c8 ret=u16
  arg work
  arg u32 any
  arg u32 alt const 0 || const 1 || any
  stub 021bc948 force ret=u16 args=ptr,v,v

# counts the other mons (bd2b4 / bd34c iterator) whose 81c8(work, m, r2) value exceeds that of mon r1
fn 021c81d0 ret=u32
  arg work
  arg u32 any
  arg u32 alt const 0 || const 1 || any
  stub 021bc948 force ret=u16 args=ptr,v,v
  stub 021bd2b4 force ret=none args=ptr,ptr
  stub 021bd34c force ret=pick:0/1/1 args=ptr,ptr,out:4 seq
  stub 021c81c8 ret=u16 args=ptr,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# core6 6f0c(work, mon, 1)
fn 021c8234 ret=u32
  arg work
  arg u32 alt range 0 5 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021c6f0c force ret=pick:0/1 args=p,v,v

# 0 when the mon's field 0x11 is 0x67 or it has condition 0x13; else 021ef948(7) == 0
fn 021c824c ret=bool
  arg work
  arg u32 alt range 0 5 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021d59b4 ret=pick:0x67/0/1 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021ef948 ret=pick:0/1 args=v

# thunk to core4 021c2ab0(work) (26 callers)
fn 021c8288 ret=u32
  arg work
  stub 021c2ab0 force ret=u32 args=ptr

# core3 bfa00(work, mon(r1), mon(r2), r3)
fn 021c8290 ret=u32
  arg work
  arg u32 alt range 0 5 || any
  arg u32 alt range 0 5 || any
  arg u32 any
  stub 021b9934 force ret=u32 args=v,v
  stub 021bfa00 force ret=pick:0/1 args=ptr,v,v,v

# tail call 021b911c(work[4], r1) (a stub that returns 0 and reads nothing)
fn 021c82b8 ret=u32
  arg work
  arg u32 any
  stub 021b911c force ret=u32 args=

# r0 is not read: veneer 82e4(021b9ae0(r1), r2) (-> 06898ce0: side-condition word)
fn 021c82d0 ret=u32
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021b9ae0 force ret=pick:0/1 args=v
  stub 021c82e4 ret=u32 args=v,v

# tail call 021b9010(work[4], r1)
fn 021c8300 ret=u32
  arg work
  arg u32 any
  stub 021b9010 force ret=pick:0/1/2/3/4 args=v,v

# work[0x78A] bit 0
fn 021c830c ret=u32
  arg work

# appends byte r1 to the 6-entry list (count work[0x788], bytes at +0x78B)
fn 021c831c ret=none
  arg work
  arg u32 any
  fix work 0x788 u8 alt range 0 7 || any

# 021d5c30(mon)
fn 021c8338 ret=u32
  arg work
  arg u32 alt range 0 5 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5c30 ret=pick:0/1 args=v

# 021d69ac(mon, 4) - 1 below 0x18 (and the field nonzero)
fn 021c8348 ret=bool
  arg work
  arg u32 alt range 0 5 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021d69ac ret=pick:0/1/2/0x18/0x19/0x20/0xFF args=v,v

# 8348 or condition 0x21
fn 021c8374 ret=bool
  arg work
  arg u32 alt range 0 5 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021c8348 ret=pick:0/1 args=ptr,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d69ac ret=pick:0/1/2/0x18/0x19/0x20/0xFF args=v,v

# max(1, 021d7b10(021d67e8(mon), core4 2af0(work, mon)))
fn 021c83a0 ret=u32
  arg work
  arg u32 alt range 0 5 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021c2af0 force ret=u32 args=ptr,v
  stub 021d67e8 ret=u32 args=v
  stub 021d7b10 ret=pick:0/1/2/100 args=v,v

# thunk to core4 021c2e4c(work)
fn 021c83cc ret=u32
  arg work
  stub 021c2e4c force ret=u32 args=ptr

# thunk to core1 021bc2b0(work)
fn 021c83d4 ret=u32
  arg work
  stub 021bc2b0 force ret=pick:0/1 args=ptr

# byte +4 of the record behind work[0x4A8] := r1
fn 021c83dc ret=none
  arg work
  arg u32 any
  region D 0x10
  fix work 0x4A8 u32 addrof region:D

# if 021b86b8(work[4]) <= 1: server_side bb908(work[0], r1), 1; else 0
fn 021c83e8 ret=bool
  arg work
  arg u32 any
  stub 021b86b8 force ret=pick:0/1/2/3 args=v
  stub 021bb908 force ret=none args=p,v

# server_side bb914(work[0]) when 021b86b8(work[4]) <= 1 and the pokeID's client is 021b8ff4(work[4])
fn 021c8408 ret=none
  arg work
  arg u32 alt range 0 5 || any
  stub 021b86b8 force ret=pick:0/1/2/3 args=v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021bb914 force ret=none args=ptr

# core2 be52c(work[0x4B0], r1, r2, veneer 845c(work + 0x1AB8, r2))
fn 021c8434 ret=none
  arg work
  arg u32 any
  arg u32 alt range 0 5 || any
  stub 021be52c force ret=none args=ptr,v,v,v
  stub 021c845c ret=pick:0/1/2/3/4/5/6 args=p,v

# core3 bfeac(work, mon(r1), mon(r2), r3)
fn 021c846c ret=u32
  arg work
  arg u32 alt range 0 5 || any
  arg u32 alt range 0 5 || any
  arg u32 any
  stub 021b9934 force ret=u32 args=v,v
  stub 021bfeac force ret=u32 args=ptr,v,v,ptr

# core3 bff60(work, mon, 1)
fn 021c8494 ret=none
  arg work
  arg u32 alt range 0 5 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021bff60 force ret=none args=ptr,v,v

# clears work[0x78A] bit 5, core4 2128(work, r1), then *r2 (byte) = bit 5; returns 2128's result
fn 021c84ac ret=u32
  arg work
  arg u32 any
  arg buf 4
  stub 021c2128 force ret=pick:0/1 args=ptr,v

# thunk to core2 021bdbb8(work, r1)
fn 021c84d0 ret=none
  arg work
  arg u32 any
  stub 021bdbb8 force ret=none args=ptr,v

# thunk to events 021cc668(work, mon)
fn 021c84d8 ret=u32
  arg work
  arg u32 any
  stub 021cc668 force ret=u32 args=ptr,v

# thunk to 021c84f4
fn 021c84e0 ret=none
  arg work
  arg u32 alt range 0 5 || any
  fix work 0x785 u8 alt range 0 0x18 || any
  fix work 0x791 u8 range 0 5 rep=24,1
  stub 021c84f4 ret=none args=ptr,v

# work[0x785] := 0
fn 021c84e8 ret=none
  arg work

# adds byte r1 to the set at work + 0x791 (count work[0x785], max 0x18) unless it is there
fn 021c84f4 ret=none
  arg work
  arg u32 alt range 0 5 || any
  fix work 0x785 u8 alt range 0 0x18 || any
  fix work 0x791 u8 range 0 5 rep=24,1

# 1 when veneer 8578(work + 0x1AB8, client(work[0x791 + i]), sp) finds something for some i < work[0x785]
fn 021c8534 ret=bool
  arg work
  fix work 0x785 u8 alt range 0 8 || any
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021c8578 ret=pick:0/0/1 args=p,v,p

# work[0x1A64] := 0; tail call 85d4(work, 0, 1)
fn 021c8588 ret=none
  arg work
  stub 021bd1e0 force ret=none args=ptr
  stub 021c85d4 ret=none args=ptr,v,v
  stub 021c87a0 ret=none args=p
  stub 021c87b0 ret=none args=p
  stub 021c87c0 ret=none args=p
  stub 021c87d0 ret=none args=p
  stub 021c87e0 ret=none args=p
  stub 021c87f0 ret=none args=p
  stub 021c8800 ret=none args=p

# if work[0x1A64] < 6: ++ and 85d4(work, new, 1)
fn 021c859c ret=none
  arg work
  fix work 0x1A64 u32 alt range 0 7 || any
  stub 021bd1e0 force ret=none args=ptr
  stub 021c85d4 ret=none args=ptr,v,v
  stub 021c87a0 ret=none args=p
  stub 021c87b0 ret=none args=p
  stub 021c87c0 ret=none args=p
  stub 021c87d0 ret=none args=p
  stub 021c87e0 ret=none args=p
  stub 021c87f0 ret=none args=p
  stub 021c8800 ret=none args=p

# if work[0x1A64] != 0: -- and 85d4(work, new, 0)
fn 021c85b8 ret=none
  arg work
  fix work 0x1A64 u32 range 0 7
  stub 021bd1e0 force ret=none args=ptr
  stub 021c85d4 ret=none args=ptr,v,v
  stub 021c87a0 ret=none args=p
  stub 021c87b0 ret=none args=p
  stub 021c87c0 ret=none args=p
  stub 021c87d0 ret=none args=p
  stub 021c87e0 ret=none args=p
  stub 021c87f0 ret=none args=p
  stub 021c8800 ret=none args=p

# points the move-record table at block idx (work + 0x874 + idx * 0x290); with flag: clears the records (seven ov95
# veneers = MI_CpuFill8(p, 0, 0x48), core2 bd1e0, MI_CpuFill8 of the smaller ones)
fn 021c85d4 ret=none
  arg work
  arg u32 range 0 7
  arg u32 alt const 0 || const 1 || any
  stub 021bd1e0 force ret=none args=ptr
  stub 021c87a0 ret=none args=p
  stub 021c87b0 ret=none args=p
  stub 021c87c0 ret=none args=p
  stub 021c87d0 ret=none args=p
  stub 021c87e0 ret=none args=p
  stub 021c87f0 ret=none args=p
  stub 021c8800 ret=none args=p
