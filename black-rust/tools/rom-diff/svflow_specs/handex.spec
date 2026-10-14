# svflow spec: handex (0x021C8918-0x021CA8CC; B10bc-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs .. B12-spec additions). No new grammar.
#
# What this area is: HANDEX execution. 021c8918 runs one HANDEX record (allocated in the manager at work + 0x1D74 by hem's
# 021c88a4, executed and freed by hem's 021c88c8) through a 60-way switch on its id byte; the 59 procs 021c8c6c..021ca7d4
# turn the record into server commands (cmd_queue put_cmd 021cd4b0 / put_cmd_list 021cd64c / put_cmd_at 021cd568 on the
# queue behind work + 0xC), messages (021c99a0 with a str record, layout in hem.spec) and core flow calls (damage, heal,
# status, stat stages, switch, items: core1..core6). Proc results (0 / 1) become manager bits 28/29 (cmd_queue cc974).
# 021c99a0 is the message helper (34 callers in core2..core5 and here). 021ca7f4 is not a HANDEX proc: core2 calls it
# (a 4-way switch on the battle rule into events caa58 / cacf4 / ca8f8 / cafd8); it is here because B9 cut the area there.
#
# HANDEX record (arg1 = `buf 0x80` in every proc block; real records are at most 0x74 bytes): word 0 = id (bits 0-7),
# pokeID (bits 8-12, 0x1F = none), size (13-22), bit 23 = "bracket the effect with 5a1c / 5a34 (core5) on the pokeID's mon",
# bit 24 = "skip unless the manager state is nonzero", bit 25 = "skip when the pokeID's mon is fainted", bit 26 = allocated;
# proc-specific fields from +4 (mon ids as bytes, u16 move / item / value ids, str records at +8 / +0xC / +0x14 / +0x18 /
# +0x2C / +0x50 depending on the id; see each block's comment).
#
# Register facts the pseudo-C hides (read off the disassembly): rizin shows `invalid` at 021c8bba (it is `bl 021ca270`,
# case 0x2C) and at 021c91bc (`bl 021c9240`, a veneer); 8de4 / 8fa4 / 9010 are called with v in r2 but never read it; 8fa4
# fetches mon(pokeID) and drops it; 9448 returns (u8) 021d5efc(mon) (the BattleMon function, not core5's 021c5efc); 98e4
# passes r1 = 0x3C6B / r2 = 0x3C6D to cc8d8 / cc95c, which do not read them; 9068 builds a str record inside the work at
# +0x1AE4, hands it to 99a0 and clears it with hem's 8810; 9a24 calls the veneer 9a50 with 5 arguments (the last a stack word
# rec[0x1C]); 9250 calls core4 3008 with 10 arguments and 9178 calls core4 27a0 with 7 (the existing core specs stub 27a0
# with 6); 9f6c returns D[5] & 1 of the record D behind work + 0x4A8, read before it restores D's three halfwords; ca7f4's
# events callees read stack words that hold its r1 / r2 (caa58 gets 7 arguments, cacf4 / cafd8 8, ca8f8 6); 9e3c calls
# 021d7040 through the thunk 021dcf9c (stubbed at the target).
#
# Layout (core4 convention, closure stubs): every block lists the stubs of its callees and of its in-area descendants'
# callees plus their resource lines, so any porting order needs no spec edit (checked: with every handex fn treated as
# ported, all 62 blocks run without a fault, a stray write or an unstubbed callee). The 8918 block therefore carries the
# stubs of every proc's callees (and record bounds a proc needs: counts at +4 / +0xF / +0x14). Callees in cmd_queue (all
# PORTED) run as real code on both sides with the queue region Q (write count below the 3000 cap; the fatal 02034714 is a
# noreturn stub). Callees in other server-flow areas (core1..core6, events, api, hem) are `force` stubs: the handex ports
# call them through env.call and their own specs verify them. BattleMon / pokecon / main-module / message-builder callees
# and the `bx pc` veneers into overlay 95 are plain stubs (argument lists = the registers the callee reads; veneer targets
# read off OVY_95). A proc seen from 8918 is `ret=pick:0/1 args=p,v` (`p,v,v` for the procs that get v).
#
# Not specced: the 2-byte `bx pc` veneers into overlay 95 and their 8-byte ARM halves (callers stub them at these addresses):
#   021c8cac 8cfc 8e48 8f14 8f94 9168 916c 9240 9374 9384 9438 95c0 961c 96b4 96c4 96c8 9818 9888 988c 9a50 9a54 9ba8 9bac
#   021ca0b8 a0c8 a0d8 a0e8 a0ec a0f8 a0fc a108 a118 a11c a238 a5bc a8c8
# 62 real functions are specced (8918, the 59 procs, 99a0, ca7f4).

area handex

# HANDEX executor (one record): v = cc960(mgr) (manager bits 0-9), st = cc96c(mgr) ? (u8) cc99c(mgr) : 1 (mgr = work + 0x1D74).
# Skips everything (returns, no cc974) when rec bit 24 is set and st == 0, or when rec bit 25 is set and the mon
# 021b9940(work[8], pokeID) is fainted (021d5b68). Else switch (rec & 0xFF): id <= 0x3B -> st = proc(work, rec[, v]);
# 0x0D and ids > 0x3B keep st; then cc974(mgr, st). The return value is whatever r0 holds (callers ignore it).
# Case table (id:proc, * = called with v in r2):
# 00:9de8  01:8c6c  02:8cbc  03:8d0c  04:96d4  05:8d2c*
# 06:8de4*  07:8e58  08:8f24  09:8fa4*  0a:9010*  0b:9068*
# 0c:9178  0d:----  0e:9250*  0f:9394  10:947c  11:94c8
# 12:9448  13:95d0  14:962c  15:9728  16:9750  17:9778
# 18:97a4  19:97d0  1a:9828  1b:9898  1c:98c4  1d:98e4
# 1e:9a24  1f:9a60  20:9bb8  21:9dcc  22:9e3c  23:9ecc
# 24:9cc0  25:9f0c  26:9f4c  27:9f6c  28:a128  29:a158
# 2a:a1d0  2b:a248  2c:a270  2d:a2e8  2e:a318  2f:a370
# 30:a394  31:a3ac  32:a3d0  33:a47c  34:a570  35:a5cc
# 36:a6c4  37:a6fc  38:a738  39:a748  3a:a7c4  3b:a7d4
fn 021c8918 ret=none inputs=1200
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  region D 0x10
  fix work 0x4A8 u32 addrof region:D
  fix arg1 0 u32 alt range 0 0x3B || any bits=0:8
  fix arg1 4 u8 range 0 28
  fix arg1 0xF u8 range 0 0x70
  fix arg1 0x14 u8 range 0 12
  stub 0203efc8 ret=none args=p,v,v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021b8dec force ret=u32 args=v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021b9940 force ret=u32 args=v,v
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9c98 force ret=pick:0/1/2/0xFFFFFFFF args=v,v
  stub 021bb930 force ret=none args=ptr,v
  stub 021bc2b0 force ret=pick:0/0/1 args=p
  stub 021bd148 force ret=pick:0/1 args=p,v
  stub 021bd170 force ret=pick:0/1 args=p,v
  stub 021bd1b0 force ret=pick:0/1 args=p,v
  stub 021bd1e0 force ret=none args=outk:6:0xFF/0xFF/0xFF/0xFF/0xFF/3
  stub 021bd200 force ret=none args=outk:6:0/0/0xFF/0xFF/0/3,p,v,v
  stub 021bd43c force ret=none args=p,v,v,v,v
  stub 021bd4e4 force ret=none args=p,v,v,v
  stub 021bd60c force ret=pick:0/1 args=p,v,v
  stub 021bd9d4 force ret=none args=p,v,v,v,v
  stub 021bda00 force ret=none args=p
  stub 021bdd00 force ret=pick:0/1 args=p,v,v
  stub 021bf3d8 force ret=none args=p,in:0x10,v,v,ptr
  stub 021bf4ac force ret=none args=p,in:0x10,v,v,ptr
  stub 021bf83c force ret=none args=p,in:0x10,v,v
  stub 021bf8d4 force ret=none args=p,in:0x10,v,v
  stub 021c0780 force ret=none args=p,v,v,v
  stub 021c0898 force ret=pick:0/1 args=p,v,v,v
  stub 021c091c force ret=none args=outki:0x4A8:6:0/0/0/0/0/3,in:0x10,v,v,ptr,v
  stub 021c195c force ret=none args=p,v,v
  stub 021c1d60 force ret=pick:0/1 args=p,v,v
  stub 021c1e14 force ret=pick:0/1 args=p,v,v,v
  stub 021c20bc force ret=pick:0/1 args=p,v,v
  stub 021c20d0 force ret=u32 args=p,v,v,p
  stub 021c2128 force ret=pick:0/1 args=p,v
  stub 021c2250 force ret=none args=p,v,v,v
  stub 021c2314 force ret=none args=p,v
  stub 021c27a0 force ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c2910 force ret=none args=p,v,v,v,v,v,v,v
  stub 021c2ca8 force ret=none args=p,v
  stub 021c3008 force ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c3370 force ret=pick:0/1 args=p,v
  stub 021c3394 force ret=pick:0/1 args=p,v,v
  stub 021c33e8 force ret=none args=p,v,v
  stub 021c3810 force ret=pick:0/1 args=p,v,v,v,outm:4:1/0/0/0,v,v,in:0x24
  stub 021c3b8c force ret=pick:0/1 args=p,v,v
  stub 021c3bb8 force ret=none args=p,v,v
  stub 021c3c60 force ret=pick:0/1 args=p,v,v,v
  stub 021c3fdc force ret=none args=p,v
  stub 021c4bac force ret=none args=p,v,v
  stub 021c4c50 force ret=none args=p,v
  stub 021c5418 force ret=pick:0/1 args=p,v,v
  stub 021c5620 force ret=none args=p,v,v,outn:4:0xFF/0xFF/0xFF/0xFF
  stub 021c56a8 force ret=pick:0/0/0x13/5/1 args=v,in:4 seq
  stub 021c5718 force ret=none args=p
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c6080 force ret=none args=p,v,v,v,v
  stub 021c60d4 force ret=none args=p,v
  stub 021c60ec force ret=none args=p,v,v
  stub 021c6110 force ret=none args=p,v,v
  stub 021c64b4 force ret=none args=p,v,v,out:0x14
  stub 021c6e00 force ret=none args=p,v,v
  stub 021c6f34 force ret=pick:0/1 args=p,v,v
  stub 021c70e4 force ret=none args=p,v,v,v,v,v
  stub 021c7194 force ret=pick:0/1 args=p,v,v
  stub 021c7a54 force ret=pick:0/1 args=p,v,v
  stub 021c7aa0 force ret=none args=p,v
  stub 021c7b3c force ret=none args=p,v,v,v
  stub 021c7b7c force ret=none args=p,v
  stub 021c7ba8 force ret=none args=p,v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF
  stub 021c7bf8 force ret=none args=p
  stub 021c7f68 force ret=pick:0/1/2/3 args=p,v,outm:4:0x1f
  stub 021c8374 force ret=pick:0/0/1 args=p,v
  stub 021c8810 force ret=none args=ptr
  stub 021c8828 force ret=pick:0/1 args=ptr
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021c8c6c ret=pick:0/1 args=p,v
  stub 021c8cac ret=pick:0/1 args=p,v
  stub 021c8cbc ret=pick:0/1 args=p,v
  stub 021c8cfc ret=pick:0/1 args=p,v
  stub 021c8d0c ret=pick:0/1 args=p,v
  stub 021c8d2c ret=pick:0/1 args=p,v,v
  stub 021c8de4 ret=pick:0/1 args=p,v,v
  stub 021c8e48 ret=pick:0/1 args=p,v
  stub 021c8e58 ret=pick:0/1 args=p,v
  stub 021c8f14 ret=pick:0/1 args=p,v
  stub 021c8f24 ret=pick:0/1 args=p,v
  stub 021c8f94 ret=pick:0/1 args=p,v
  stub 021c8fa4 ret=pick:0/1 args=p,v,v
  stub 021c9010 ret=pick:0/1 args=p,v,v
  stub 021c9068 ret=pick:0/1 args=p,v,v
  stub 021c9168 ret=pick:0/1 args=v,v,v,v,p
  stub 021c9178 ret=pick:0/1 args=p,v
  stub 021c9240 ret=pick:0/1 args=p,v
  stub 021c9250 ret=pick:0/1 args=p,v,v
  stub 021c9374 ret=pick:0/1 args=p,v
  stub 021c9384 ret=pick:0/1 args=p,v
  stub 021c9394 ret=pick:0/1 args=p,v
  stub 021c9438 ret=pick:0/1 args=p,v
  stub 021c9448 ret=pick:0/1 args=p,v
  stub 021c947c ret=pick:0/1 args=p,v
  stub 021c94c8 ret=pick:0/1 args=p,v
  stub 021c95c0 ret=pick:0/1 args=p,v
  stub 021c95d0 ret=pick:0/1 args=p,v
  stub 021c961c ret=pick:0/1 args=p,v
  stub 021c962c ret=pick:0/1 args=p,v
  stub 021c96b4 ret=pick:0/1 args=p,v
  stub 021c96c4 ret=pick:0/1 args=v
  stub 021c96d4 ret=pick:0/1 args=p,v
  stub 021c9728 ret=pick:0/1 args=p,v
  stub 021c9750 ret=pick:0/1 args=p,v
  stub 021c9778 ret=pick:0/1 args=p,v
  stub 021c97a4 ret=pick:0/1 args=p,v
  stub 021c97d0 ret=pick:0/1 args=p,v
  stub 021c9818 ret=pick:0/1 args=v,v,v
  stub 021c9828 ret=pick:0/1 args=p,v
  stub 021c9888 ret=pick:0/1 args=v,v
  stub 021c9898 ret=pick:0/1 args=p,v
  stub 021c98c4 ret=pick:0/1 args=p,v
  stub 021c98e4 ret=pick:0/1 args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021c9a24 ret=pick:0/1 args=p,v
  stub 021c9a50 ret=pick:0/1 args=v,v,v,in:0x10,v
  stub 021c9a60 ret=pick:0/1 args=p,v
  stub 021c9ba8 ret=pick:0/0/1 args=v
  stub 021c9bb8 ret=pick:0/1 args=p,v
  stub 021c9cc0 ret=pick:0/1 args=p,v
  stub 021c9dcc ret=pick:0/1 args=p,v
  stub 021c9de8 ret=pick:0/1 args=p,v
  stub 021c9e3c ret=pick:0/1 args=p,v
  stub 021c9ecc ret=pick:0/1 args=p,v
  stub 021c9f0c ret=pick:0/1 args=p,v
  stub 021c9f4c ret=pick:0/1 args=p,v
  stub 021c9f6c ret=pick:0/1 args=p,v
  stub 021ca0b8 ret=none args=v
  stub 021ca0c8 ret=none args=v
  stub 021ca0d8 ret=none args=v,v
  stub 021ca0e8 ret=none args=v,v
  stub 021ca0f8 ret=none args=v
  stub 021ca108 ret=pick:0/0/1 args=v
  stub 021ca118 ret=pick:0/0/1 args=v
  stub 021ca128 ret=pick:0/1 args=p,v
  stub 021ca158 ret=pick:0/1 args=p,v
  stub 021ca1d0 ret=pick:0/1 args=p,v
  stub 021ca238 ret=u32 args=p,v
  stub 021ca248 ret=pick:0/1 args=p,v
  stub 021ca270 ret=pick:0/1 args=p,v
  stub 021ca2e8 ret=pick:0/1 args=p,v
  stub 021ca318 ret=pick:0/1 args=p,v
  stub 021ca370 ret=pick:0/1 args=p,v
  stub 021ca394 ret=pick:0/1 args=p,v
  stub 021ca3ac ret=pick:0/1 args=p,v
  stub 021ca3d0 ret=pick:0/1 args=p,v
  stub 021ca47c ret=pick:0/1 args=p,v
  stub 021ca570 ret=pick:0/1 args=p,v
  stub 021ca5bc ret=pick:0/1 args=p,v
  stub 021ca5cc ret=pick:0/1 args=p,v
  stub 021ca6c4 ret=pick:0/1 args=p,v
  stub 021ca6fc ret=pick:0/1 args=p,v
  stub 021ca738 ret=pick:0/1 args=p,v
  stub 021ca748 ret=pick:0/1 args=p,v
  stub 021ca7c4 ret=pick:0/1 args=p,v
  stub 021ca7d4 ret=pick:0/1 args=p,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d55d8 ret=pick:0/1 args=v,v,v
  stub 021d5734 ret=none args=v,v,v,v,v
  stub 021d5970 ret=none args=v,v,v
  stub 021d59b4 ret=pick:0/1/2 args=v,v
  stub 021d5b40 ret=u32 args=v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d5bdc ret=pick:0/1 args=v,v
  stub 021d5c30 ret=pick:0/1 args=v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021d5ea8 ret=none args=v,v,v
  stub 021d5efc ret=pick:0/1 args=v
  stub 021d5f08 ret=none args=v
  stub 021d5fa4 ret=none args=v,v
  stub 021d5fc0 ret=none args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d6440 ret=none args=v,v
  stub 021d65c8 ret=none args=v,v
  stub 021d66b0 ret=none args=v,v
  stub 021d66d0 ret=none args=v,v
  stub 021d66d8 ret=none args=v,v
  stub 021d6738 ret=none args=v
  stub 021d67d8 ret=none args=v,v
  stub 021d69ac ret=pick:0/1/5/0x18/0x19/0x20/0xff args=v,v
  stub 021d6c24 ret=pick:0/0/1 args=v
  stub 021d6c2c ret=none args=v
  stub 021d6c50 ret=pick:0/1/1 args=v,v
  stub 021d6d54 ret=pick:0/1 args=v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021d7040 ret=none args=v
  stub 021d8484 ret=none args=v
  stub 021d8510 ret=none args=v
  stub 021dcf78 ret=nullable:3:u32 args=v,v
  stub 021e0434 ret=none args=v
  stub 021e8cfc ret=pick:0/1/2/3/4/17 args=v
  stub 021e8da8 ret=pick:0/1 args=v
  stub 021ef8e8 ret=pick:0/1 args=v

# proc: if veneer 8cac(work+0x1AB8, pokeID) -> 60d4(work, mon(pokeID)), 1; else 0
fn 021c8c6c ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c60d4 force ret=none args=p,v
  stub 021c8cac ret=pick:0/1 args=p,v

# proc: if veneer 8cfc(work+0x1AB8, pokeID) -> 5a1c(work, mon(pokeID)), 1; else 0
fn 021c8cbc ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5a1c force ret=none args=p,v
  stub 021c8cfc ret=pick:0/1 args=p,v

# proc: 5a34(work, mon(pokeID)); returns 1
fn 021c8d0c ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5a34 force ret=none args=p,v

# proc (3 args): target = mon(rec[6]); unless 3370(work, target): [bit23: 5a1c(mon(pokeID))]; ok = rec[7] != 0 or
# !3394(work, target, 1); if ok: 33e8(work, target, u16 rec[4]), then 99a0(rec+8) if its kind byte != 0, else
# put_cmd_list(Q, 0x5B, 0x38C, rec[6], v, END) when v != 0; [bit23: 5a34]. Returns ok (0 when 3370 says so)
fn 021c8d2c ret=u32
  arg work
  arg buf 0x80
  arg u32 alt const 0 || range 0 0x3FF
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix arg1 7 u8 alt const 0 || any
  fix arg1 0xA u16 alt range 0 2 || any bits=0:8
  stub 021b9934 force ret=u32 args=v,v
  stub 021c3370 force ret=pick:0/1 args=p,v
  stub 021c3394 force ret=pick:0/1 args=p,v,v
  stub 021c33e8 force ret=none args=p,v,v
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24

# proc (3 args; v unused): attacker = rec[7] == 0x1F ? 0 : mon(rec[7]); veneer 8e48(work+0x1AB8, rec[6]) and target
# mon(rec[6]) alive and 1e14(work, target, attacker, u16 rec[4]) -> 99a0(rec+8) if kind != 0; returns 1, else 0
fn 021c8de4 ret=u32
  arg work
  arg buf 0x80
  arg u32 alt const 0 || range 0 0x3FF
  fix arg1 7 u8 alt const 0x1F || any
  fix arg1 0xA u16 alt range 0 2 || any bits=0:8
  stub 021b9934 force ret=u32 args=v,v
  stub 021c1e14 force ret=pick:0/1 args=p,v,v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c8e48 ret=pick:0/1 args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5b68 ret=pick:0/1 args=v

# proc: veneer 8f14(work+0x1AB8, rec[6]); mon = mon(rec[6]) alive, (rec[7] bit0 -> !5c30(mon)), 20bc(work, mon, u16 rec[4]):
# [bit23: 5a1c(mon(pokeID) or 0 for 0x1F)]; rec[7] bit1 -> 70e4(work, u16 rec[8], rec[0xA], rec[0xB], 0, 0);
# 20d0(work, mon, u16 rec[4], rec+0xC); [bit23: 5a34]; 1. Else 0
fn 021c8e58 ret=u32
  arg work
  arg buf 0x80
  fix arg1 0 u32 alt const 31 || any bits=8:5
  stub 021b9934 force ret=u32 args=v,v
  stub 021c20bc force ret=pick:0/1 args=p,v,v
  stub 021c20d0 force ret=u32 args=p,v,v,p
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c70e4 force ret=none args=p,v,v,v,v,v
  stub 021c8f14 ret=pick:0/1 args=p,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5c30 ret=pick:0/1 args=v

# proc: for i < rec[4]: id = rec[7+i]; veneer 8f94(work+0x1AB8, id), mon(id) alive -> 5b74(work, mon, rec[0x10+4i],
# rec[5] == 0), and 195c(work, mon, 1) when rec[6] == 0; result 1 if any. (count bounded: indexes stay in the record)
fn 021c8f24 ret=u32
  arg work
  arg buf 0x80
  fix arg1 4 u8 alt range 0 6 || range 7 28
  fix arg1 5 u8 alt const 0 || any
  fix arg1 6 u8 alt const 0 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021c195c force ret=none args=p,v,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021c8f94 ret=pick:0/1 args=p,v
  stub 021d5b68 ret=pick:0/1 args=v

# proc (3 args; v unused): mon(pokeID) is fetched and dropped; t = mon(rec[5]); if 5b7c(t) or rec[7] bit1:
# f = !(rec[7] bit0); unless 55d8(t, rec[6], f): 6080(work, t, rec[6], rec[4], f), 99a0(rec+8), 1. Else 0
fn 021c8fa4 ret=u32
  arg work
  arg buf 0x80
  arg u32 alt const 0 || range 0 0x3FF
  fix arg1 0xA u16 alt range 0 2 || any bits=0:8
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c6080 force ret=none args=p,v,v,v,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d55d8 ret=pick:0/1 args=v,v,v
  stub 021d5b7c ret=pick:0/1/1 args=v

# proc (3 args; v unused): t = mon(rec[5]); if alive or rec[7] bit1: (0898(work, t, rec[6], rec[4]) ->
# 99a0(rec+8), and 7194(work, t, rec[6]) -> 2128(work, t)); 1. Else 0
fn 021c9010 ret=u32
  arg work
  arg buf 0x80
  arg u32 alt const 0 || range 0 0x3FF
  fix arg1 0xA u16 alt range 0 2 || any bits=0:8
  stub 021b9934 force ret=u32 args=v,v
  stub 021c0898 force ret=pick:0/1 args=p,v,v,v
  stub 021c2128 force ret=pick:0/1 args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c7194 force ret=pick:0/1 args=p,v,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5b68 ret=pick:0/1 args=v

# proc (3 args): [bit23: 5a1c(mon(pokeID))]; for i < rec[0x14]: t = mon(rec[8+i]); if 5b7c(t): cursor = rec[4];
# while (k = 56a8(t, &cursor)): r = 1; 5620(work, t, k, &w); rec[0x15] == 0 -> (veneer 9168(k, w, t, v,
# work+0x1AE4) -> 99a0(work+0x1AE4), 8810(work+0x1AE4)), else 99a0(rec+0x18); k == 0x13 -> 195c(work, t, 0).
# [bit23: 5a34]. Returns r. The work str at +0x1AE4 is cleared with hem's 8810 (force stub)
fn 021c9068 ret=u32
  arg work
  arg buf 0x80
  arg u32 alt const 0 || range 0 0x3FF
  fix arg1 0x14 u8 alt range 0 4 || range 5 12
  fix arg1 0x15 u8 alt const 0 || any
  fix arg1 0x1A u16 alt range 0 2 || any bits=0:8
  fix work 0x1AE6 u16 alt range 0 2 || any bits=0:8
  stub 021b9934 force ret=u32 args=v,v
  stub 021c195c force ret=none args=p,v,v
  stub 021c5620 force ret=none args=p,v,v,outn:4:0xFF/0xFF/0xFF/0xFF
  stub 021c56a8 force ret=pick:0/0/0x13/5/1 args=v,in:4 seq
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c8810 force ret=none args=ptr
  stub 021c9168 ret=pick:0/1 args=v,v,v,v,p
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5b7c ret=pick:0/1/1 args=v

# proc: a = pokeID == 0x1F ? 0 : mon(pokeID); f = !8828(rec+0x14) && rec[0xD] == 0; veneer 9240(work+0x1AB8, rec[0xF])
# (the pseudo-C shows `invalid`: it is `bl 021c9240`), t = mon(rec[0xF]) alive, !27a0(work, t, a, rec[4], rec[8],
# rec[0x10], rec[0xC]) -> [bit23: 5a1c(a)], 2910(work, t, a, rec[4], rec[8], f, rec[0xE], rec+0x14), [bit23: 5a34(a)], 1
fn 021c9178 ret=u32
  arg work
  arg buf 0x80
  fix arg1 0 u32 alt const 31 || any bits=8:5
  fix arg1 0xD u8 alt const 0 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021c27a0 force ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c2910 force ret=none args=p,v,v,v,v,v,v,v
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c8828 force ret=pick:0/1 args=ptr
  stub 021c9240 ret=pick:0/1 args=p,v
  stub 021d5b68 ret=pick:0/1 args=v

# proc (3 args): a = mon(pokeID); pass 1 over i < rec[0xF] (id = rec[0x10+i]): veneer 9374(work+0x1AB8, id), alive,
# 5d14(t, rec[4], (s8) rec[0xC]) -> any = 1, stop. any && bit23 -> 5a1c(a). Pass 2: veneer 9384, alive, 3008(work, pokeID,
# t, rec[4], (s8) rec[0xC], 0x1F, v, rec[8], rec[0xE], rec[0xD] == 0) -> 99a0(rec+0x18), r = 1. any && bit23 -> 5a34(a)
fn 021c9250 ret=u32
  arg work
  arg buf 0x80
  arg u32 alt const 0 || range 0 0x3FF
  fix arg1 0xF u8 alt range 0 6 || range 7 0x70
  fix arg1 0xD u8 alt const 0 || any
  fix arg1 0x1A u16 alt range 0 2 || any bits=0:8
  stub 021b9934 force ret=u32 args=v,v
  stub 021c3008 force ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c9374 ret=pick:0/1 args=p,v
  stub 021c9384 ret=pick:0/1 args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5d14 ret=pick:0/1 args=v,v,v

# proc: veneer 9438(work+0x1AB8, rec[4]), mon(rec[4]) alive: 5ea8(mon, k, rec[4+k]) for k = 1..7, then
# put_cmd(Q, 0x0B, rec[4], rec[5], rec[6..0xB]); 1. Else 0
fn 021c9394 ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021c9438 ret=pick:0/1 args=p,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5ea8 ret=none args=v,v,v

# proc: mon(rec[4]) alive: put_cmd(Q, 0x0C, rec[4]); returns (u8) 5efc(mon) (the BattleMon fn 021d5efc, not core5 5efc).
# Else 0
fn 021c9448 ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5efc ret=pick:0/1 args=v

# proc: for i < rec[4]: mon(rec[5+i]) alive -> put_cmd(Q, 0x0D, rec[5+i]), 5f08(mon), r = 1. Returns r
fn 021c947c ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix arg1 4 u8 alt range 0 6 || range 7 0x7A
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5f08 ret=none args=v

# proc: veneer 95c0(work+0x1AB8, rec[0xE]) -> mon = mon(rec[0xE]); for bit k of rec[0xF] (k = 0..4, field 8+k):
# 5970(mon, 8+k, u16 rec[4+2k]), put_cmd(Q, 0x13, rec[0xE], 8+k, (u8) rec[4+2k]); then 99a0(rec+0x10); 1. Else 0
fn 021c94c8 ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix arg1 0x12 u16 alt range 0 2 || any bits=0:8
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c95c0 ret=pick:0/1 args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5970 ret=none args=v,v,v

# proc: veneer 961c(work+0x1AB8, rec[4]); mon(rec[4]) alive or rec[5] != 0 -> 99a0(rec+8), 2314(work, mon), 1. Else 0
fn 021c95d0 ret=u32
  arg work
  arg buf 0x80
  fix arg1 5 u8 alt const 0 || any
  fix arg1 0xA u16 alt range 0 2 || any bits=0:8
  stub 021b9934 force ret=u32 args=v,v
  stub 021c2314 force ret=none args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c961c ret=pick:0/1 args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5b68 ret=pick:0/1 args=v

# proc: veneer 96b4(work+0x1AB8, rec[6]), mon(rec[6]) alive, !veneer 96c4(5444(mon)) -> put_cmd(Q, 0x16, rec[6],
# u16 rec[4]), 66b0(mon, u16 rec[4]); rec[7] == 0 && 8da8(u16 rec[4]) -> put_cmd_list(Q, 0x5B, 0x380, rec[6],
# 8cfc(u16 rec[4]), END); 1. Else 0
fn 021c962c ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix arg1 7 u8 alt const 0 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021c96b4 ret=pick:0/1 args=p,v
  stub 021c96c4 ret=pick:0/1 args=v
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d66b0 ret=none args=v,v
  stub 021e8cfc ret=pick:0/1/2/3/4/17 args=v
  stub 021e8da8 ret=pick:0/1 args=v

# proc: a = pokeID == 0x1F ? 0 : mon(pokeID); [bit23 && a: 5a1c]; 99a0(rec+4); [bit23 && a: 5a34]; returns 1
fn 021c96d4 ret=u32
  arg work
  arg buf 0x80
  fix arg1 0 u32 alt const 31 || any bits=8:5
  fix arg1 6 u16 alt range 0 2 || any bits=0:8
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24

# proc: mon(rec[8]) alive -> 5fc0(mon, rec[4]), 1. Else 0
fn 021c9728 ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5fc0 ret=none args=v,v

# proc: mon(rec[8]) alive -> 6440(mon, rec[4]), 1. Else 0
fn 021c9750 ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d6440 ret=none args=v,v

# proc: mon(rec[8]) alive -> 60ec(work, mon, rec[4]), 1. Else 0
fn 021c9778 ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c60ec force ret=none args=p,v,v
  stub 021d5b68 ret=pick:0/1 args=v

# proc: mon(rec[8]) alive -> 6110(work, mon, rec[4]), 1. Else 0
fn 021c97a4 ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c6110 force ret=none args=p,v,v
  stub 021d5b68 ret=pick:0/1 args=v

# proc: w = rec[8]; 7ba8(work, pokeID, rec[4], rec[0xC], &w) (stack0 = in/out word); veneer 9818(rec[0xC], rec[4], w)
# -> 99a0(rec+0x10), 1. Else 0
fn 021c97d0 ret=u32
  arg work
  arg buf 0x80
  fix arg1 0x12 u16 alt range 0 2 || any bits=0:8
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c7ba8 force ret=none args=p,v,v,v,outm:4:0xFF/0xFF/0xFF/0xFF
  stub 021c9818 ret=pick:0/1 args=v,v,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24

# proc: for i in 0..14: if (i >> 3) + 1 < rec[4] and rec[4 + (i >> 3) + 1] bit (i & 7): veneer 9888(rec[7], i) ->
# 4bac(work, i, rec[7]), r = 1. Returns r (the bit mask bytes overlap the count/side bytes rec[5]/rec[6]/rec[7])
fn 021c9828 ret=u32
  arg work
  arg buf 0x80
  fix arg1 4 u8 alt range 0 3 || any
  stub 021c4bac force ret=none args=p,v,v
  stub 021c9888 ret=pick:0/1 args=v,v

# proc: 3c60(work, rec[4], rec[8], rec[0xC]) -> 99a0(rec+0x10), 1. Else 0
fn 021c9898 ret=u32
  arg work
  arg buf 0x80
  fix arg1 0x12 u16 alt range 0 2 || any bits=0:8
  stub 021c3c60 force ret=pick:0/1 args=p,v,v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24

# proc: ef8e8(rec[4]) -> 4c50(work, rec[4]) (bl to 021c4c50, core5), 1. Else 0
fn 021c98c4 ret=u32
  arg work
  arg buf 0x80
  stub 021c4c50 force ret=none args=p,v
  stub 021ef8e8 ret=pick:0/1 args=v

# proc: a = mon(pokeID); rec[4] != 0: 3b8c(work, rec[4], rec[5]) -> [bit23: 5a1c(a)], 3bb8(work, rec[4], rec[5]),
# 99a0(rec+8), r = 1. rec[4] == 0 and rec[6] != 0: [bit23: 5a1c(a)], 99a0(rec+8), old = cc8d8(work+0x1D74)
# (r1 = 0x3C6B is not read), 7bf8(work), cc95c(work+0x1D74, old) (r2 = 0x3C6D not read), r = 1. r && bit23 -> 5a34(a)
fn 021c98e4 ret=u32
  arg work
  arg buf 0x80
  fix arg1 4 u8 alt const 0 || any
  fix arg1 6 u8 alt const 0 || any
  fix arg1 0xA u16 alt range 0 2 || any bits=0:8
  stub 021b9934 force ret=u32 args=v,v
  stub 021c3b8c force ret=pick:0/1 args=p,v,v
  stub 021c3bb8 force ret=none args=p,v,v
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c7bf8 force ret=none args=p
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24

# message helper (34 callers): str record r1 (hem layout); kind 1 -> 5dac / 2 -> 5efc (work, id, count, &args),
# or with bit 15 set kind 1 -> 5e48 / 2 -> 5f98 (work, id, (u16) rec[+0x20], count, stack: &args); returns 1, or 0 for
# another kind
fn 021c99a0 ret=u32
  arg work
  arg buf 0x24
  fix arg1 2 u16 alt range 0 3 || any bits=0:8
  fix arg1 2 u16 alt range 0 8 || any bits=8:7
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20

# proc: veneer 9a50(rec[4], rec[8], pokeID, rec+0xC, stack: rec[0x1C]) != 0
fn 021c9a24 ret=u32
  arg work
  arg buf 0x80
  stub 021c9a50 ret=pick:0/1 args=v,v,v,in:0x10,v

# type change (?): mon = rec[6]; v = (u16) BattleMon_Get(mon, 0x10); 0 when veneer 9ba8(v) or mon fainted, or when
# rec[7] == 0 and rec[4] (u16) == v; else cmds 0x57 / 0x49 / 0x1d / 0x58 on the queue, str at +0xC, 7b3c / 7b7c inside
# header brackets (cc8d8 / cc95c on work + 0x1D74), handler re-registration 8510 / 8484; v 0x67 -> 195c(mon, 0), 0x7f -> 3fdc
fn 021c9a60 ret=u32 inputs=400
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix arg1 4 u16 alt oneof 0x67 0x7f 1 2 || any
  fix arg1 7 u8 alt const 0 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021c195c force ret=none args=p,v,v
  stub 021c3fdc force ret=none args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c7b3c force ret=none args=p,v,v,v
  stub 021c7b7c force ret=none args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021c9ba8 ret=pick:0/0/1 args=v
  stub 021d59b4 ret=pick:0x67/0x7f/1/2 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d66d0 ret=none args=v,v
  stub 021d8484 ret=none args=v
  stub 021d8510 ret=none args=v

# mon = rec[6]; when pokeID (rec bits 8-12) != rec[6]: question 7a54(mon, rec[4]) in a header bracket, a yes -> 7aa0 and
# return 0; else cmd 0x57 (flag 23), str at +0xC, 2250(mon, rec[4], 0), 0x58, 6738 + cmd 0x2b for rec[7] / rec[8]
# (second mon rec[9]), 195c(mon, 0); returns 1
fn 021c9bb8 ret=u32 inputs=300
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix arg1 1 u8 range 0 1 bits=0:5
  fix arg1 6 u8 alt range 0 1 || any
  fix arg1 7 u8 alt const 0 || any
  fix arg1 8 u8 alt const 0 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021c195c force ret=none args=p,v,v
  stub 021c2250 force ret=none args=p,v,v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c7a54 force ret=pick:0/1 args=p,v,v
  stub 021c7aa0 force ret=none args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d6738 ret=none args=v

# swap-like: mons rec[4] and pokeID; u16 5b40 of both; question 7a54 (yes -> 7aa0, return 0); else three str records
# +8 / +0x2C / +0x50 inside 5a1c / 5a34 (flag 23), 2250 and 195c for both mons; returns 1
fn 021c9cc0 ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c195c force ret=none args=p,v,v
  stub 021c2250 force ret=none args=p,v,v,v
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c7a54 force ret=pick:0/1 args=p,v,v
  stub 021c7aa0 force ret=none args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5b40 ret=u32 args=v

# 195c(work, mon rec[4], rec[8] word); returns 1
fn 021c9dcc ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c195c force ret=none args=p,v,v

# mon = pokeID; fainted and rec[4] bit 1 clear -> 0; rec[4] bit 0 and 5b48(mon) -> 0; else 2128(work, mon) != 0
fn 021c9de8 ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c2128 force ret=pick:0/1 args=p,v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 021d5b68 ret=pick:0/1 args=v

# mon = rec[4]; fainted -> 0; p = dcf78(mon, rec[6] u16), 0 -> 0; placeholder cmd 0x42, 6e00(work, mon, pokeID) in a
# header bracket, 88e4 == 2 -> put_cmd_at(pos, 0x42, rec[4]); d7040(p) (through the thunk 021dcf9c); returns 1
fn 021c9e3c ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021c6e00 force ret=none args=p,v,v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d7040 ret=none args=v
  stub 021dcf78 ret=nullable:3:u32 args=v,v

# mon = pokeID; rec[4] word == 0 -> 60d4(work, mon) + str at +8; then 2250(work, mon, 0, 1); returns 1
fn 021c9ecc ret=u32
  arg work
  arg buf 0x80
  fix arg1 4 u32 alt const 0 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021c2250 force ret=none args=p,v,v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c60d4 force ret=none args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24

# cmd 0x1f (rec[4], rec[5], stack rec[6], rec[7], rec[8] u16), then 5734(mon, rec[5], rec[8], rec[6], stack rec[7])
fn 021c9f0c ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021d5734 ret=none args=v,v,v,v,v

# 0780(work, mon rec[4], rec[5], rec[6]); returns 1
fn 021c9f4c ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c0780 force ret=none args=p,v,v,v

# move-like: mons rec[4] / rec[5]; 64b4(work, rec[6] u16, mon, &m) fills a 0x14-byte stack record m; veneers on the
# objects at work + 0x860 / + 0x868; a108 -> 5718(work), return 0; bf83c / bf3d8 / bf4ac / bf8d4 (work, &m, mon,
# work[0x868][, work + 0x1F88]); a118 -> 0; else saves the 3 halfwords of D = *(work + 0x4A8), bd1e0(D), bd200(D, work, mon,
# w868), D[4] = 1, c091c(work, &m, mon, w868, work + 0x1F88, 1), restores D and returns D[5] & 1 (read before the restore)
# Z20: the three D-record stubs write D as the originals do (by disassembly / the core2 and core3 ports): bd1e0 resets bytes 0-4 and
# bits 0-1 of byte 5, bd200 sets bytes 2, 3 and bits 0-1 of byte 5, c091c (D reached through work + 0x4A8) bits 0-1 of byte 5
# (`outk` / `outki`, mix()-derived values, the other bits kept), so the save / restore order and the D[5] read are compared.
# Same lines in 021c8918's block (it runs this proc).
fn 021c9f6c ret=u32 inputs=300
  arg work
  arg buf 0x80
  region D 0x10
  fix work 0x4A8 u32 addrof region:D
  stub 021b9934 force ret=u32 args=v,v
  stub 021bd1e0 force ret=none args=outk:6:0xFF/0xFF/0xFF/0xFF/0xFF/3
  stub 021bd200 force ret=none args=outk:6:0/0/0xFF/0xFF/0/3,p,v,v
  stub 021bf3d8 force ret=none args=p,in:0x10,v,v,ptr
  stub 021bf4ac force ret=none args=p,in:0x10,v,v,ptr
  stub 021bf83c force ret=none args=p,in:0x10,v,v
  stub 021bf8d4 force ret=none args=p,in:0x10,v,v
  stub 021c091c force ret=none args=outki:0x4A8:6:0/0/0/0/0/3,in:0x10,v,v,ptr,v
  stub 021c5718 force ret=none args=p
  stub 021c64b4 force ret=none args=p,v,v,out:0x14
  stub 021ca0b8 ret=none args=v
  stub 021ca0c8 ret=none args=v
  stub 021ca0d8 ret=none args=v,v
  stub 021ca0e8 ret=none args=v,v
  stub 021ca0f8 ret=none args=v
  stub 021ca108 ret=pick:0/0/1 args=v
  stub 021ca118 ret=pick:0/0/1 args=v

# bd60c(work, mon pokeID, 1) != 0 -> work[0x14] = 5, return 1; else 0
fn 021ca128 ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021bd60c force ret=pick:0/1 args=p,v,v

# mon = rec[0x4C]; 0 when bc2b0(work), 8374(work, rec[0x4C]) or work[0x14] != 0; str +4; bdd00(work, mon, rec[0x4D]) -> 0;
# else bb930(server, b8d8c(main, pokecon, rec[0x4C])), str +0x28, work[0x14] = 1, return 1
fn 021ca158 ret=u32 inputs=400
  arg work
  arg buf 0x80
  fix work 0x14 u32 alt const 0 || any
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021bb930 force ret=none args=ptr,v
  stub 021bc2b0 force ret=pick:0/0/1 args=p
  stub 021bdd00 force ret=pick:0/1 args=p,v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c8374 force ret=pick:0/0/1 args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24

# mons a = rec[4], b = rec[5]: 62c4(a, 0x10) -> 2ca8(work, b); 65c8(b, a); cmd 0x26 (rec[4], rec[5]); 6d7c(b) -> cmd 0x51
# (veneer a238(work + 0x1AB8, rec[5])); returns 1
fn 021ca1d0 ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021c2ca8 force ret=none args=p,v
  stub 021ca238 ret=u32 args=p,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d65c8 ret=none args=v,v
  stub 021d6d7c ret=pick:0/1 args=v

# 1d60(work, mon rec[4], rec[5]) != 0
fn 021ca248 ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c1d60 force ret=pick:0/1 args=p,v,v

# 5fa4(mon rec[4], rec[6] u16); cmd 2; work[0x7A9 + rec[4]] = 0; str +8; pos = b8d8c(main, pokecon, rec[4]); pos != 6 ->
# bd9d4(work, client(rec[4]), x, x, 1) with x = b8dec(main, pos), bda00(work); returns 1 (the pseudo-C hides the bl at 8bba)
fn 021ca270 ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021b8dec force ret=u32 args=v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021bd9d4 force ret=none args=p,v,v,v,v
  stub 021bda00 force ret=none args=p
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5fa4 ret=none args=v,v

# 67d8(mon rec[4], rec[6] u16); cmd 0x14 (rec[4], rec[6]); str +8; returns 1
fn 021ca2e8 ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d67d8 ret=none args=v,v

# 3810(work, mon pokeID, mon rec[6], rec[7] & 0xF, stack: &out, rec[4] u16, rec[7] >> 4, rec + 8) != 0
fn 021ca318 ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c3810 force ret=pick:0/1 args=p,v,v,v,outm:4:1/0/0/0,v,v,in:0x24

# bd148(work, rec[4]) != 0 -> str +8, return 1; else 0
fn 021ca370 ret=u32
  arg work
  arg buf 0x80
  stub 021bd148 force ret=pick:0/1 args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24

# bd170(work, rec[4] u16) != 0
fn 021ca394 ret=u32
  arg work
  arg buf 0x80
  stub 021bd170 force ret=pick:0/1 args=p,v

# bd1b0(work, rec[4]) != 0 -> str +8, return 1; else 0
fn 021ca3ac ret=u32
  arg work
  arg buf 0x80
  stub 021bd1b0 force ret=pick:0/1 args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24

# party swap: rec[4] != rec[5] on the same client, both alive, party indices i, j >= 0 (BattleParty_GetMonIndex 021b9c98)
# -> bd43c(work, client, i, j, 0), str +8, bd4e4(work, client, i, j), return 1; else 0
fn 021ca3d0 ret=u32 inputs=400
  arg work
  arg buf 0x80
  fix arg1 4 u8 alt range 0 5 || any
  fix arg1 5 u8 alt range 0 5 || any
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021b9934 force ret=u32 args=v,v
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9c98 force ret=pick:0/1/2/0xFFFFFFFF args=v,v
  stub 021bd43c force ret=none args=p,v,v,v,v
  stub 021bd4e4 force ret=none args=p,v,v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5b68 ret=pick:0/1 args=v

# a = pokeID mon, b = rec[4] mon: 0 when 6c24(a) / 6c24(b) / 5c30(a) or !6c50(a, b); else e0434 / 8510 / 8484 on a,
# cmd 0x53 (ids), str +8 (in 5a1c / 5a34 when flag 23); Get(a, 0x10) != Get(a, 0x11) -> 7b7c in a header bracket; returns 1
fn 021ca47c ret=u32 inputs=600
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c7b7c force ret=none args=p,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d59b4 ret=pick:1/2 args=v,v
  stub 021d5c30 ret=pick:0/1 args=v
  stub 021d6c24 ret=pick:0/0/1 args=v
  stub 021d6c50 ret=pick:0/1/1 args=v,v
  stub 021d8484 ret=none args=v
  stub 021d8510 ret=none args=v
  stub 021e0434 ret=none args=v

# veneer a5bc(work + 0x1AB8, rec[4]) and 6c24(mon) -> 6c2c(mon), cmd 0x4b, str +8, return 1; else 0
fn 021ca570 ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021ca5bc ret=pick:0/1 args=p,v
  stub 021d6c24 ret=pick:0/0/1 args=v
  stub 021d6c2c ret=none args=v

# n = 7f68(work, 0x400 | b8d8c(main, pokecon, pokeID), ids) (api, fills a stack id list); per id: 5bdc(mon, 3) ->
# 5418(work, mon, 3), 69ac(mon, 4) in 1..0x18 -> 0203efc8(work, mon, 1); 6f34(work, mon, 1); 62c4 0x1e / 0x20 ->
# 5620(work, mon, flag, 0); any of them -> cmd list 0x5b (0x43b, id); returns 1
fn 021ca5cc ret=u32 inputs=300
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0203efc8 ret=none args=p,v,v
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5418 force ret=pick:0/1 args=p,v,v
  stub 021c5620 force ret=none args=p,v,v,outn:4:0xFF/0xFF/0xFF/0xFF
  stub 021c6f34 force ret=pick:0/1 args=p,v,v
  stub 021c7f68 force ret=pick:0/1/2/3 args=p,v,outm:4:0x1f
  stub 021d5bdc ret=pick:0/1 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d69ac ret=pick:0/1/5/0x18/0x19/0x20/0xff args=v,v

# mon rec[4] alive and 5418(work, mon, rec[8] word) -> str +0xC, return 1; else 0
fn 021ca6c4 ret=u32
  arg work
  arg buf 0x80
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5418 force ret=pick:0/1 args=p,v,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d5b68 ret=pick:0/1 args=v

# rec[0xB] -> cmd 0x56 (0); 70e4(work, rec[4] u16, rec[6], rec[7], stack rec[0xA], rec[8] u16); str +0xC; returns 1
fn 021ca6fc ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix arg1 0xB u8 alt const 0 || any
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c70e4 force ret=none args=p,v,v,v,v,v
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24

# cmd 0x56 (0); returns 1 (rec unused)
fn 021ca738 ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# mon rec[4] alive, !6d54(mon), (u8) Get(mon, 0x13) != rec[5] -> 66d8(mon, rec[5]), cmd 0x4f, str +8 (5a1c / 5a34 when
# flag 23), return 1; else 0
fn 021ca748 ret=u32
  arg work
  arg buf 0x80
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  fix arg1 5 u8 alt range 0 2 || any
  stub 021b9934 force ret=u32 args=v,v
  stub 021c5a1c force ret=none args=p,v
  stub 021c5a34 force ret=none args=p,v
  stub 021c5dac force ret=none args=p,v,v,in:0x20
  stub 021c5e48 force ret=none args=p,v,v,v,in:0x20
  stub 021c5efc force ret=none args=p,v,v,in:0x20
  stub 021c5f98 force ret=none args=p,v,v,v,in:0x20
  stub 021c99a0 ret=pick:0/1 args=p,in:0x24
  stub 021d59b4 ret=pick:0/1/2 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d6d54 ret=pick:0/1 args=v

# D = *(work + 0x4A8): D[4] = rec[4]; returns 1
fn 021ca7c4 ret=u32
  arg work
  arg buf 0x80
  region D 0x10
  fix work 0x4A8 u32 addrof region:D

# D = *(work + 0x4A8): D[5] |= 1; returns 1 (rec unused)
fn 021ca7d4 ret=u32
  arg work
  arg buf 0x80
  region D 0x10
  fix work 0x4A8 u32 addrof region:D

# not a HANDEX proc (not dispatched by 8918; core2 calls it): (work, mon, r2, r3 = record (reads +0x10 bit 0), stack0).
# r6 = 0x1F, or cb214(work, mon, r3) when +0x10 bit 0 is clear; veneer ca8c8(stack0); work[0x77F] = 6; by
# 8588(main): 1 -> caa58, 2 -> cacf4, else ca8f8 (work, r1, r2, r3, stack r6, stack0, [r1, r2]); result 1 -> id of
# ca8d8(stack0, 0) to work[0x77F] (veneer ca8e8); cafd8(work, kind, r1, r3, stack r2, stack0, r1, r2) == 0 -> 0
fn 021ca7f4 ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x20
  arg u32 any
  stub 021b8588 force ret=pick:0/1/2/3/4 args=v
  stub 021ca8c8 ret=none args=v
  stub 021ca8d8 ret=nullable:2:u32 args=v,v
  stub 021ca8e8 ret=u32 args=p,v
  stub 021ca8f8 force ret=pick:0/1/2 args=p,v,v,v,v,v
  stub 021caa58 force ret=pick:0/1/2 args=p,v,v,v,v,v,v
  stub 021cacf4 force ret=pick:0/1/2 args=p,v,v,v,v,v,v,v
  stub 021cafd8 force ret=pick:0/1 args=p,v,v,v,v,v,v,v
  stub 021cb214 force ret=pick:0/1/2/0x1f args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
