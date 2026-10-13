# svflow spec: core4 (0x021C0F24-0x021C4838; B10i, written by B10i-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs / B10-spec2 / B10f-spec / B10f-spec2 additions). No new grammar.
#
# What this area is: the fourth part of the flow core (btl_server_flow.c): the per-target damage pipeline and its side-effect
# helpers. 0f24 applies one move record to every target of the per-turn list (B[0] = count) and runs the damage bracket 0a4c +
# 11a4; 11a4 -> 11f0 is the per-target computation (scratch arrays at 021f3b20.., the only function that touches them), 1e74 the
# base damage (CalcBaseDamage 021d7a44, STAB / type / random / critical scaling), 236c / 2488 / 25a4 / 2910 / 282c the secondary
# effect roll and the failure-message switch, 2e64 / 2f70 / 3008 the multi-hit loop and its accuracy gate, 3438 / 3d08 / 3e0c /
# 3aa8 / 376c / 3810 the per-mon effect loops, 446c the turn-end state machine (state byte work+0x77E, 0..6) with 4614 / 4744.
# Many small builders follow the same shape: a message is started (021d74f8), arguments are added (021d75ec / 75a8 / 7630),
# sent (021d70f0), read back (021d775c) and ended (021d7540); a "record bracket" reserves a record with cc8d8 on the packed
# queue-header word at work+0x1D74 and closes it with cc95c.
#
# SvFlowWork fields seen here ("P"/"B" = regions, `arg work`):
#   +0x04 main module, +0x08 pokecon (party container), +0x0C command queue buffer (pointer), +0x10 u32 turn counter (446c, capped
#   at 0x270F), +0x1D74 packed queue header word, +0x1AB8 / +0x1AE4 helper structs (address only; 1AE4 is the hem record),
#   +0x1F88 24-word table (events 021cc8b8 reads table[mon id] in 3438), +0x4A8 pointer P (per-turn struct: bytes 2, 3, flag byte 5
#   bits 0 / 1), +0x4B4 pointer B (target list: byte 0 = count, 1 = hit counter, 2, 3, 5), +0x778 u32 counter (2e4c),
#   +0x77E u8 state of 446c, +0x789 / +0x78A flag bytes, +0x854 / +0x860 / +0x86C / +0x870 values or pointers handed to callees,
#   +0x1000.. (ptr:work) = the side record returned by 021bb85c (byte 9 = count, 4744 keeps it <= 4: the mon ids go into a 4-byte
#   stack buffer).
#
# Layout of the blocks (generated, then hand-reviewed): every block carries (a) the stubs of its own callees AND of the callees of
# all its in-area descendants, and (b) the resource lines of the function and its descendants (region Q for the command queue,
# P / B for the work pointers at +0x4A8 / +0x4B4, the scratch globals, the bound on work+0x1009 for 4744 and the counter edge of
# 2e4c). Porting any core4 function therefore needs no spec edit, whatever order the area is ported in (checked: with every
# core4 function treated as ported, i.e. the whole tree real on the ROM side, 102 / 102 functions run without a fault, a stray
# write or an unstubbed callee). Arguments that a descendant dereferences are buffers in every ancestor (the move record is
# `buf 0x10` wherever it is passed; the mon-pointer array of 1788 / 17cc is `buf 0x40`).
# Scratch globals: a function that reaches the command queue declares `global 021f3b44 0x40` (the put_cmd scratch words); one
# that also reaches 11f0 (scratch arrays at 021f3b20, up to 021f3b50) declares ONE `global 021f3b20 0x64` instead, because the two
# ranges overlap: the put_cmd scratch is then bytes 0x24..0x64 of that global (`&mut globals[0][0x24..]`).
#
# Rules (README): callees ported in cmd_queue run as REAL code on both sides: 021cc8d8 021cc95c 021cc918 021cd4b0 021cd4fc
# 021cd568 021cd64c (queue), and the events word-table getter 021cc8b8. Every function that reaches cd4b0 / cd4fc / cd568 /
# cd64c declares the region Q block (queue at work+0xC, write count below the 3000 cap, scratch words 021f3b44, the fatal
# 02034714). Callees ported in core1 / core2 / server_side (021bc2b0, 021bd2b4, 021bd34c, 021bd43c, 021bd4e4, 021bd9d4, 021bda00,
# 021bdd7c, 021bb85c) and the core3 callees (021bf324, 021bfa00, 021bfa60, 021c0a4c, 021c0c34, 021c0c38, 021c0cb4, 021c0ce8,
# 021c0d70; core3 is being ported now) are `force` stubs, so these specs stay valid as those areas land and the Rust ports keep
# calling them through env.call (callers' hashes do not change when they get ported; each is verified by its own area).
# Everything else outside this area, and every core4 function not yet in PORTED, is a stub (a stub of a core4 function becomes
# inactive the moment that address is in a PORTED list). `bx pc` veneers (list below) are stubbed at their own address.
# Stub argument kinds follow the REGISTERS set at each call site (hidden pass-through of r0..r3 is common: 7330 gets work and the
# mon from 20bc's own r0 / r1, 7664 / 75ec read r2 as well, 99a0 takes (work, hem record)). Pointers into the work or into a
# region that are only handed on are `p`, scalars `v`. Stack words past r3 are declared in call order. Stubs whose result is only
# a small number use `ret=pick:...`, decorrelated and with the interesting values (0, 1, thresholds the code compares against)
# repeated; callbacks run in loops (`list` veneers 157c / 159c, 1b3c / 1b4c, ...) are `seq` + `nullable:3:u32`, so every loop ends.
# 021d775c (reply slot read-back) answers 0 / 1 / 2 / 8 / 50 / 100 / 4096 / 0xFFFF: the code tests it against 0, 8 (2f70) and
# 0xFFFF (236c). 021d59b4 (BattleMon_Get) is `u32` unless a function compares its answer with a constant (3e0c, 282c, 1e74, 2910 have
# a `stub` override with those constants).
#
# Argument-shape notes: `stub ... out:4` on a pointer to the CALLER's own stack word (25a4 -> 26fc, 4104 -> 6d9c, 232c -> 236c) is
# the local variable the callee fills; a port keeps it as a local. 4104 and 3c60 are varargs-style functions (`push {r0-r3}`
# prologue, `bx r3` return): the register arguments are the first four words of the argument list. 4104 hands the addresses of
# its own spilled r2 / stack-0 / stack-1 slots to 5840 / 59c4 as in/out words (the stubs ignore them: `p`) and reads r3's slot
# back as the u16 written by 6d9c; the returned value is that u16.
#
# Jump tables: 2910 (8 cases, halfword offsets at 021c2936) and 446c (7 cases, offsets at 021c44a8) switch with `add pc, r0`;
# Ghidra splits neither, the case bodies and the literal pools are part of their functions (the specs reach the cases through
# the reason argument r3 and the state byte).
#
# Uninitialised stack: none found that changes a result (every stack slot read is written first, including the out:N stub
# targets); the emulator zeroes the stack anyway.
#
# `--strict-callees` clean. 0209c054 (64-bit multiply, from 4744) runs as real code on both sides; it is a pure helper and was
# added to the svflow.py ALLOW list next to 0209c2b0 (the port computes ((rand * n) >> 32)). The random-number source 0203f0a8
# (4744) is a stub.
#
# Skipped (not specced): 62 two-byte `bx pc` veneers into unloaded 0x0689xxxx code (list iterators, flow-API getters; each is
# followed by an 8-byte ARM thunk `ldr ip,[pc]; bx ip` that Ghidra names thunk_EXT_FUN_0689xxxx; the thunk halves 021c1c6c 021c1c7c
# 021c2aa4 021c3694 021c3a9c 021c3fd0 021c43c4 021c4460 are part of veneers 021c1c68 021c1c78 021c2aa0 021c3690 021c3a98 021c3fcc
# 021c43c0 021c445c):
#   021c1144 021c1154 021c1164 021c1174 021c14b0 021c14c0 021c14d0 021c156c 021c157c 021c158c 021c159c 021c1b2c 021c1b3c 021c1b4c
#   021c1c48 021c1c58 021c1c68 021c1c78 021c1d30 021c1d40 021c1d50 021c21f4 021c2564 021c2574 021c2584 021c2594 021c2aa0 021c2e1c
#   021c2e2c 021c2e3c 021c3244 021c3254 021c3264 021c331c 021c332c 021c333c 021c3428 021c3660 021c3670 021c3680 021c3690 021c37e0
#   021c37f0 021c3800 021c3a98 021c3dfc 021c3fac 021c3fbc 021c3fcc 021c40f4 021c4208 021c43c0 021c43d0 021c43e0 021c443c 021c444c
#   021c445c 021c45e4 021c45f4 021c4604 021c4724 021c4734
# (their callers stub them at these addresses). 021c1194 is a real 4-byte getter and is specced. 102 real functions are specced.
# Smoke ports (B10i-spec, in core4.rs): see the PORTED list there.
area core4

# # apply one target's damage record: for every mon in the record list (B[0] = count, 0..3) run the damage bracket (0a4c, 11a4),
# # then the post-hit helpers; returns the summed damage (rec = move record: u16 id at +0; mon A r2, mon B r3, stack arg 0 handed on)
fn 021c0f24 ret=u32 inputs=300
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg buf 0x10
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  region B 0x20
  fix work 0x4b4 u32 addrof region:B
  global 021f3b20 0x64
  fix work 0x778 u32 alt const 0xFFFFFFFF || any
  stub 02034714 noreturn ret=none args=v,v,v
  fix region:B 0 u8 range 0 4
  stub 021c1144 ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8590 force ret=pick:0/1 args=v
  stub 021c1154 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c1164 ret=u32 args=p,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c0a4c force ret=none args=p,v,v,p,p,v,v
  stub 021c11a4 ret=u32 args=p,p,v,v,p,p,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c195c ret=none args=p,v,v
  stub 021bfa60 force ret=pick:0/1 args=p,v,v,p
  stub 021c15ac ret=none args=p,v
  stub 021c1174 ret=u32 args=p,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021c0c34 force ret=pick:0/1 args=v
  stub 021c11f0 ret=u32 args=p,v,v,p,p,p,v,v
  stub 021c18d4 ret=none args=p,v,v,v
  stub 021c1910 ret=none args=p,v,p,v
  stub 021c14b0 ret=u32 args=p,v
  stub 021c0d70 force ret=u32 args=p
  stub 021c0c38 force ret=pick:0/1/2/3/4/5/6 args=p,p,p,p,p,p
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c4104 ret=u32 args=p,v,v,v,v,v,p
  stub 021c14c0 ret=none args=v,v,v,v
  stub 021c1184 ret=pick:0/1 args=p
  stub 021c0cb4 force ret=pick:0/1/2/3/4/5/6 args=p,p,p
  stub 021c15e8 ret=none args=p,v,v,p
  stub 021c0ce8 force ret=pick:0/1/2/3/4/5/6 args=p,p,p,p,p,p,p
  stub 021c1788 ret=none args=p,v,v,v,p
  stub 021c5798 force ret=none args=p,p,v,p,p,p
  stub 021c1194 ret=pick:0/1 args=p
  stub 021c5840 force ret=none args=p,v,p,p,v
  stub 021c1198 ret=none args=p,v
  stub 021c59c4 force ret=none args=p,v,p,p,v
  stub 021c14d0 ret=none args=v,v,v,v
  stub 021c1828 ret=none args=p,v,v,v,p,v
  stub 021d5fc0 ret=none args=v,v
  stub 021c16f8 ret=none args=p,v,v
  stub 021c1ccc ret=u32 args=p,p,v,v
  stub 021c1dcc ret=none args=p,p,v,v,v
  stub 021c1880 ret=none args=p,p,v,v,v,v
  stub 021c4218 ret=none args=p,v,v,v,v,v,v,v
  stub 021c4f64 force ret=pick:0/1 args=p,v
  stub 021c4268 ret=none args=p,v,v,p,v,v,v,v
  stub 021d7630 ret=none args=v,v
  stub 0201c1bc ret=pick:0/1/2/3/4/5/5/6/7/8/9/9/10/11/12/13/13/20 args=v
  stub 021c2cd8 ret=none args=p,v,v,v
  stub 021c232c ret=none args=p,p,v,v
  stub 021c236c ret=u32 args=p,v,v,v,out:4
  stub 021c25a4 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c8810 force ret=none args=p
  stub 021c264c ret=pick:0/0xffff/1/5/100 args=p,v,v,v
  stub 021c26a8 ret=none args=p,v,v,v,v
  stub 021c26fc ret=none args=p,v,v,v,out:4
  stub 021c8828 force ret=pick:0/1 args=p
  stub 021c2758 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c27a0 ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c2910 ret=none args=p,v,v,v,v,v,v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d8484 ret=none args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=
  stub 0201bd28 ret=u32 args=v,v
  stub 0201bf58 ret=u16 args=v
  stub 021d7c70 ret=none args=v16,v,out:4
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021c2d10 ret=pick:0/1 args=p,p,v,v
  stub 021c2e64 ret=pick:0/1 args=p,p,v,v,v
  stub 021c2e4c ret=u32 args=p
  stub 0201c020 ret=pick:0/1/2/3/5 args=v
  stub 021c2f70 ret=none args=p,v,v,v,v,outm:4:0xF/0/0/0,out:4
  stub 021c3008 ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c78a8 force ret=none args=p,v,v,v,v
  stub 021c312c ret=u32 args=p,v,v,v,v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021c5a4c force ret=none args=p,v,v,v
  stub 021c77b0 force ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c5a8c force ret=none args=p,v,v,v,v,v
  stub 021c785c force ret=none args=p,v,v,v,v
  stub 021c7820 force ret=none args=p,v,v
  stub 0201c0bc ret=pick:0/1/2/3/8 args=v,v,out:4
  stub 021d7b3c ret=pick:0/1/2/3/100/200 args=v,v
  stub 021c1e14 ret=u32 args=p,v,v,v
  stub 021c78f4 force ret=nullable:3:u32 args=p,v,v,v
  stub 021c3370 ret=pick:0/1 args=p,v
  stub 021c334c ret=pick:0/1 args=p,v,v,v
  stub 021c3394 ret=pick:0/1 args=p,v,v
  stub 021c33e8 ret=none args=p,v,v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 021c7664 force ret=pick:0/1/50/99/100/200 args=p,v,v
  stub 021c1d60 ret=u32 args=p,v,v
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021c76f4 force ret=pick:0/1 args=p,v,v
  stub 021c7780 force ret=none args=p,v
  stub 021c1758 ret=none args=p,v
  stub 021d6848 ret=none args=v,in:7
  stub 021c17cc ret=none args=p,v,v,v,p
  stub 021c1624 ret=none args=p,v,v,p
  stub 021d6d9c ret=pick:0/1 args=v,out:4
  stub 021c41c0 ret=none args=p,v
  stub 021c4208 ret=u32 args=p,v
  stub 021d6d70 ret=none args=v

# # true when byte 1 of the struct is 0
fn 021c1184 ret=bool
  arg buf 0x10
  fix arg0 1 u8 alt const 0 || any

# # returns byte 3 of the struct (4-byte getter)
fn 021c1194 ret=u8
  arg buf 0x10

# # if byte 5 of the struct is 3, set it to r1
fn 021c1198 ret=none
  arg buf 0x10
  arg u8 any
  fix arg0 5 u8 alt const 3 || any

# # damage on one target when 0c34(record) says there is one: 11f0 (the full per-target computation) then 18d4 (the log record)
fn 021c11a4 ret=u32 inputs=300
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b20 0x64
  fix work 0x778 u32 alt const 0xFFFFFFFF || any
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c0c34 force ret=pick:0/1 args=v
  stub 021c11f0 ret=u32 args=p,v,v,p,p,p,v,v
  stub 021c18d4 ret=none args=p,v,v,v
  stub 021c1910 ret=none args=p,v,p,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021c14b0 ret=u32 args=p,v
  stub 021c0d70 force ret=u32 args=p
  stub 021c0c38 force ret=pick:0/1/2/3/4/5/6 args=p,p,p,p,p,p
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c4104 ret=u32 args=p,v,v,v,v,v,p
  stub 021c14c0 ret=none args=v,v,v,v
  stub 021c1184 ret=pick:0/1 args=p
  stub 021c0cb4 force ret=pick:0/1/2/3/4/5/6 args=p,p,p
  stub 021c15e8 ret=none args=p,v,v,p
  stub 021c0ce8 force ret=pick:0/1/2/3/4/5/6 args=p,p,p,p,p,p,p
  stub 021c1788 ret=none args=p,v,v,v,p
  stub 021c5798 force ret=none args=p,p,v,p,p,p
  stub 021c1194 ret=pick:0/1 args=p
  stub 021c5840 force ret=none args=p,v,p,p,v
  stub 021c1198 ret=none args=p,v
  stub 021c59c4 force ret=none args=p,v,p,p,v
  stub 021c14d0 ret=none args=v,v,v,v
  stub 021c1828 ret=none args=p,v,v,v,p,v
  stub 021d5fc0 ret=none args=v,v
  stub 021c16f8 ret=none args=p,v,v
  stub 021c1ccc ret=u32 args=p,p,v,v
  stub 021c1dcc ret=none args=p,p,v,v,v
  stub 021c1880 ret=none args=p,p,v,v,v,v
  stub 021c4218 ret=none args=p,v,v,v,v,v,v,v
  stub 021c4f64 force ret=pick:0/1 args=p,v
  stub 021c4268 ret=none args=p,v,v,p,v,v,v,v
  stub 021d7630 ret=none args=v,v
  stub 0201c1bc ret=pick:0/1/2/3/4/5/5/6/7/8/9/9/10/11/12/13/13/20 args=v
  stub 021c2cd8 ret=none args=p,v,v,v
  stub 021c232c ret=none args=p,p,v,v
  stub 021c236c ret=u32 args=p,v,v,v,out:4
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c25a4 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c8810 force ret=none args=p
  stub 021c264c ret=pick:0/0xffff/1/5/100 args=p,v,v,v
  stub 021c26a8 ret=none args=p,v,v,v,v
  stub 021c26fc ret=none args=p,v,v,v,out:4
  stub 021c8828 force ret=pick:0/1 args=p
  stub 021c2758 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c27a0 ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c2910 ret=none args=p,v,v,v,v,v,v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d8484 ret=none args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=
  stub 0201bd28 ret=u32 args=v,v
  stub 0201bf58 ret=u16 args=v
  stub 021d7c70 ret=none args=v16,v,out:4
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021c2d10 ret=pick:0/1 args=p,p,v,v
  stub 021c2e64 ret=pick:0/1 args=p,p,v,v,v
  stub 021c2e4c ret=u32 args=p
  stub 0201c020 ret=pick:0/1/2/3/5 args=v
  stub 021c2f70 ret=none args=p,v,v,v,v,outm:4:0xF/0/0/0,out:4
  stub 021c3008 ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c78a8 force ret=none args=p,v,v,v,v
  stub 021c312c ret=u32 args=p,v,v,v,v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021c5a4c force ret=none args=p,v,v,v
  stub 021c77b0 force ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c5a8c force ret=none args=p,v,v,v,v,v
  stub 021c785c force ret=none args=p,v,v,v,v
  stub 021c7820 force ret=none args=p,v,v
  stub 0201c0bc ret=pick:0/1/2/3/8 args=v,v,out:4
  stub 021d7b3c ret=pick:0/1/2/3/100/200 args=v,v
  stub 021c1e14 ret=u32 args=p,v,v,v
  stub 021c78f4 force ret=nullable:3:u32 args=p,v,v,v
  stub 021c3370 ret=pick:0/1 args=p,v
  stub 021c334c ret=pick:0/1 args=p,v,v,v
  stub 021c3394 ret=pick:0/1 args=p,v,v
  stub 021c33e8 ret=none args=p,v,v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 021c7664 force ret=pick:0/1/50/99/100/200 args=p,v,v
  stub 021c1d60 ret=u32 args=p,v,v
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021c76f4 force ret=pick:0/1 args=p,v,v
  stub 021c7780 force ret=none args=p,v
  stub 021c1758 ret=none args=p,v
  stub 021d6848 ret=none args=v,in:7
  stub 021c17cc ret=none args=p,v,v,v,p
  stub 021c1624 ret=none args=p,v,v,p
  stub 021d6d9c ret=pick:0/1 args=v,out:4
  stub 021c41c0 ret=none args=p,v
  stub 021c4208 ret=u32 args=p,v
  stub 021d6d70 ret=none args=v

# # the per-target damage computation: scratch arrays at 021f3b20 (u8 flags, u8 [3b23], u16 [3b26], u32 [3b2c], mon pointers [3b38]),
# # list count from 0c38/0cb4/0ce8 (0..6); r1 = mon, r3 = damage list, stack arg 0 = move record, stack arg 1 = the per-turn struct B
# # (bytes 1, 3, 5), stack arg 3 bit 0 = flag
fn 021c11f0 ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg buf 0x10
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b20 0x64
  fix work 0x778 u32 alt const 0xFFFFFFFF || any
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c14b0 ret=u32 args=p,v
  stub 021c0d70 force ret=u32 args=p
  stub 021c0c38 force ret=pick:0/1/2/3/4/5/6 args=p,p,p,p,p,p
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c4104 ret=u32 args=p,v,v,v,v,v,p
  stub 021c14c0 ret=none args=v,v,v,v
  stub 021c1184 ret=pick:0/1 args=p
  stub 021c0cb4 force ret=pick:0/1/2/3/4/5/6 args=p,p,p
  stub 021c15e8 ret=none args=p,v,v,p
  stub 021c0ce8 force ret=pick:0/1/2/3/4/5/6 args=p,p,p,p,p,p,p
  stub 021c1788 ret=none args=p,v,v,v,p
  stub 021c5798 force ret=none args=p,p,v,p,p,p
  stub 021c1194 ret=pick:0/1 args=p
  stub 021c5840 force ret=none args=p,v,p,p,v
  stub 021c1198 ret=none args=p,v
  stub 021c59c4 force ret=none args=p,v,p,p,v
  stub 021c14d0 ret=none args=v,v,v,v
  stub 021c1828 ret=none args=p,v,v,v,p,v
  stub 021d5fc0 ret=none args=v,v
  stub 021c16f8 ret=none args=p,v,v
  stub 021c1ccc ret=u32 args=p,p,v,v
  stub 021c1dcc ret=none args=p,p,v,v,v
  stub 021c1880 ret=none args=p,p,v,v,v,v
  stub 021c4218 ret=none args=p,v,v,v,v,v,v,v
  stub 021c4f64 force ret=pick:0/1 args=p,v
  stub 021c4268 ret=none args=p,v,v,p,v,v,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 0201c1bc ret=pick:0/1/2/3/4/5/5/6/7/8/9/9/10/11/12/13/13/20 args=v
  stub 021c2cd8 ret=none args=p,v,v,v
  stub 021c232c ret=none args=p,p,v,v
  stub 021c236c ret=u32 args=p,v,v,v,out:4
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c25a4 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c8810 force ret=none args=p
  stub 021c264c ret=pick:0/0xffff/1/5/100 args=p,v,v,v
  stub 021c26a8 ret=none args=p,v,v,v,v
  stub 021c26fc ret=none args=p,v,v,v,out:4
  stub 021c8828 force ret=pick:0/1 args=p
  stub 021c2758 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c27a0 ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c2910 ret=none args=p,v,v,v,v,v,v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d8484 ret=none args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=
  stub 0201bd28 ret=u32 args=v,v
  stub 0201bf58 ret=u16 args=v
  stub 021d7c70 ret=none args=v16,v,out:4
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021c2d10 ret=pick:0/1 args=p,p,v,v
  stub 021c2e64 ret=pick:0/1 args=p,p,v,v,v
  stub 021c2e4c ret=u32 args=p
  stub 0201c020 ret=pick:0/1/2/3/5 args=v
  stub 021c2f70 ret=none args=p,v,v,v,v,outm:4:0xF/0/0/0,out:4
  stub 021c3008 ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c78a8 force ret=none args=p,v,v,v,v
  stub 021c312c ret=u32 args=p,v,v,v,v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021c5a4c force ret=none args=p,v,v,v
  stub 021c77b0 force ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c5a8c force ret=none args=p,v,v,v,v,v
  stub 021c785c force ret=none args=p,v,v,v,v
  stub 021c7820 force ret=none args=p,v,v
  stub 0201c0bc ret=pick:0/1/2/3/8 args=v,v,out:4
  stub 021d7b3c ret=pick:0/1/2/3/100/200 args=v,v
  stub 021c1e14 ret=u32 args=p,v,v,v
  stub 021c78f4 force ret=nullable:3:u32 args=p,v,v,v
  stub 021c3370 ret=pick:0/1 args=p,v
  stub 021c334c ret=pick:0/1 args=p,v,v,v
  stub 021c3394 ret=pick:0/1 args=p,v,v
  stub 021c33e8 ret=none args=p,v,v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 021c7664 force ret=pick:0/1/50/99/100/200 args=p,v,v
  stub 021c1d60 ret=u32 args=p,v,v
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021c76f4 force ret=pick:0/1 args=p,v,v
  stub 021c7780 force ret=none args=p,v
  stub 021c1758 ret=none args=p,v
  stub 021d6848 ret=none args=v,in:7
  stub 021c17cc ret=none args=p,v,v,v,p
  stub 021c1624 ret=none args=p,v,v,p
  stub 021d6d9c ret=pick:0/1 args=v,out:4
  stub 021c41c0 ret=none args=p,v
  stub 021c4208 ret=u32 args=p,v
  stub 021d6d70 ret=none args=v

# # for every element of the list object (seq): run 6b5c; when it hits, set the per-turn flag, run the 0x197b record bracket around 6bb8
fn 021c14e0 ret=none inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  stub 021c156c ret=none args=v
  stub 021c157c seq ret=nullable:3:u32 args=v
  stub 021c6b5c force ret=pick:0/1 args=p,v,v,p
  stub 021c6bb8 force ret=none args=p,v,v
  stub 021c158c ret=none args=v,v
  stub 021c159c seq ret=nullable:3:u32 args=v

# # message 0x4f if r1 < 3 (signed), 0x4e if r1 > 3
fn 021c15ac ret=none
  arg work
  arg s32 alt range 0 6 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# # min(r1, BattleMon field 0xd of r0)
fn 021c15d4 ret=u32
  arg u32 any
  arg u32 any
  stub 021d59b4 ret=u32 args=v,v

# # 0x19aa record bracket around 1624
fn 021c15e8 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 021c1624 ret=none args=p,v,v,p
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x45 about two mons and the move record (u16 id at +0, u8 at +6)
fn 021c1624 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # event 0x3a (chance gate on the mon's HP): *hp_slot = HP-1 when the event answers 1; returns the answer (0 when HP > *hp_slot)
fn 021c1678 ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 4
  stub 021d59b4 ret=u32 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message by mode r2: 1 -> message 0x202 through the command queue (cd64c); otherwise a 0x1a04 record bracket around 1758
fn 021c16f8 ret=none
  arg work
  arg u32 any
  arg u32 alt range 0 4 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c1758 ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x75 about one mon
fn 021c1758 ret=none
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # 0x1a29 record bracket around 17cc
fn 021c1788 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 range 0 8
  arg buf 0x40
  stub 021c17cc ret=none args=p,v,v,v,p
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x44: mon r2 and the first r3 (0..8) mon pointers of the array at stack arg 0
fn 021c17cc ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 range 0 8
  arg buf 0x40
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # copy a 7-byte struct (move id, stack arg 1, move type, ids) to 6848 and queue command 0x2d (stack arg 0 = move record)
fn 021c1828 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d6848 ret=none args=v,in:7

# # by the move's category (0201c1bc of the record's id): 7 -> 2cd8; 6 -> 2cd8 and 4 -> 232c when stack arg 1 is 0
fn 021c1880 ret=none
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  fix work 0x778 u32 alt const 0xFFFFFFFF || any
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0201c1bc ret=pick:0/1/2/3/4/5/5/6/7/8/9/9/10/11/12/13/13/20 args=v
  stub 021c2cd8 ret=none args=p,v,v,v
  stub 021c232c ret=none args=p,p,v,v
  stub 021c236c ret=u32 args=p,v,v,v,out:4
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c25a4 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c8810 force ret=none args=p
  stub 021c264c ret=pick:0/0xffff/1/5/100 args=p,v,v,v
  stub 021c26a8 ret=none args=p,v,v,v,v
  stub 021c26fc ret=none args=p,v,v,v,out:4
  stub 021c8828 force ret=pick:0/1 args=p
  stub 021c2758 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c27a0 ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c2910 ret=none args=p,v,v,v,v,v,v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d8484 ret=none args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=
  stub 0201bd28 ret=u32 args=v,v
  stub 0201bf58 ret=u16 args=v
  stub 021d7c70 ret=none args=v,v,out:4
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021c2d10 ret=pick:0/1 args=p,p,v,v
  stub 021c2e64 ret=pick:0/1 args=p,p,v,v,v
  stub 021c2e4c ret=u32 args=p
  stub 0201c020 ret=pick:0/1/2/3/5 args=v
  stub 021c2f70 ret=none args=p,v,v,v,v,outm:4:0xF/0/0/0,out:4
  stub 021c3008 ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c78a8 force ret=none args=p,v,v,v,v
  stub 021c312c ret=u32 args=p,v,v,v,v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021c5a4c force ret=none args=p,v,v,v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c77b0 force ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c5a8c force ret=none args=p,v,v,v,v,v
  stub 021c785c force ret=none args=p,v,v,v,v
  stub 021c7820 force ret=none args=p,v,v
  stub 0201c0bc ret=pick:0/1/2/3/8 args=v,v,out:4

# # 0x1a92 record bracket around 1910
fn 021c18d4 ret=none
  arg work
  arg u32 any
  arg buf 0x10
  arg u32 any
  stub 021c1910 ret=none args=p,v,p,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x4d about mon r1, the move record and r3
fn 021c1910 ret=none
  arg work
  arg u32 any
  arg buf 0x10
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # when 5b40(r1) says so: 0x1abc record bracket around 199c
fn 021c195c ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x91 about mon r1 and value r2
fn 021c199c ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # 0x1ada record bracket around 1a10
fn 021c19d8 ret=none
  arg work
  arg u32 any
  arg buf 0x10
  stub 021c1a10 ret=none args=p,v,p
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x81 about mon r1 and the move record
fn 021c1a10 ret=none
  arg work
  arg u32 any
  arg buf 0x10
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # post-hit sequence: 1ce4, then five 1b5c passes (messages 0x84, 0x83, 0x85, 0x86, 0x87) with a loop over list r3 in between, then 1c88
fn 021c1a54 ret=none inputs=300
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021c1ce4 ret=none args=p,p,v,v
  stub 021c1b5c ret=none args=p,v,v,p,v,v,v
  stub 021c1b2c ret=none args=v
  stub 021c1b3c seq ret=nullable:3:u32 args=v
  stub 021c195c ret=none args=p,v,v
  stub 021c1b4c seq ret=nullable:3:u32 args=v
  stub 021c1c88 ret=none args=p,v,v,p,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021c1c48 ret=pick:0/1/2/3/4/5/6 args=v
  stub 021c1c58 ret=u32 args=v,v
  stub 021c1c68 ret=pick:0/1 args=v,v,out:4
  stub 021c1c78 ret=pick:0/1 args=v,v,out:4
  stub 021d7630 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021c1d30 ret=none args=v
  stub 021c1d40 seq ret=nullable:3:u32 args=v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c557c force ret=none args=p,v,v,v
  stub 021c1d50 seq ret=nullable:3:u32 args=v

# # per-mon message loop over list r2: collects mons for which 1c68/1c78 answers, message 0x82 with their count and total
fn 021c1b5c ret=none inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg u32 alt const 0 || const 1
  arg u32 any
  stub 021c1c48 ret=pick:0/1/2/3/4/5/6 args=v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021c1c58 ret=u32 args=v,v
  stub 021c1c68 ret=pick:0/1 args=v,v,out:4
  stub 021c1c78 ret=pick:0/1 args=v,v,out:4
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message 0x88 about mon r1 and the move record (u16 id at +0)
fn 021c1c88 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # looks up 7664(move id) and runs 1d60 on mon r3
fn 021c1ccc ret=u32
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  stub 021c7664 force ret=pick:0/1/50/99/100/200 args=p,v,v
  stub 021c1d60 ret=u32 args=p,v,v
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021d5fc0 ret=none args=v,v
  stub 021c76f4 force ret=pick:0/1 args=p,v,v
  stub 021c7780 force ret=none args=p,v

# # for a move whose byte 6 is 9: every mon of the list (seq) whose 62a8 == 3 and 6d7c == 0 gets 557c
fn 021c1ce4 ret=none inputs=300
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  fix arg1 6 u8 alt const 9 || any
  stub 021c1d30 ret=none args=v
  stub 021c1d40 seq ret=nullable:3:u32 args=v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c557c force ret=none args=p,v,v,v
  stub 021c1d50 seq ret=nullable:3:u32 args=v

# # chance gate r2 (>= 100: record bracket 0x1bad around 7780); returns 1 when one of the effects (5bb0 / 76f4) applied
fn 021c1d60 ret=u32
  arg work
  arg u32 any
  arg u32 alt range 0 120 || any
  stub 021d5bb0 ret=pick:0/1 args=v,v
  stub 021d5fc0 ret=none args=v,v
  stub 021c76f4 force ret=pick:0/1 args=p,v,v
  stub 021c7780 force ret=none args=p,v

# # for move category 8: 1e14 on the result of 7b3c(stack arg 0, 0201bd28(move, 0x19)), then 5d88
fn 021c1dcc ret=none
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0201c1bc ret=pick:0/1/2/3/4/5/5/6/7/8/9/9/10/11/12/13/13/20 args=v
  stub 0201bd28 ret=u32 args=v,v
  stub 021d7b3c ret=pick:0/1/2/3/100/200 args=v,v
  stub 021c1e14 ret=u32 args=p,v,v,v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021c78f4 force ret=nullable:3:u32 args=p,v,v,v
  stub 021c3370 ret=pick:0/1 args=p,v
  stub 021c334c ret=pick:0/1 args=p,v,v,v
  stub 021c3394 ret=pick:0/1 args=p,v,v
  stub 021c33e8 ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d5b48 ret=pick:0/1 args=v

# # 0x1bf2 record bracket around 78f4; returns 334c's result
fn 021c1e14 ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c78f4 force ret=nullable:3:u32 args=p,v,v,v
  stub 021c3370 ret=pick:0/1 args=p,v
  stub 021c334c ret=pick:0/1 args=p,v,v,v
  stub 021c3394 ret=pick:0/1 args=p,v,v
  stub 021c33e8 ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d5b48 ret=pick:0/1 args=v

# # the base damage computation (event 0x38 ... 0x37, CalcBaseDamage 021d7a44, stab/type/random/critical scaling); *out = damage; returns the "event answered" flag
fn 021c1e74 ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg u32 alt const 0x1000 || any
  arg u32 any
  arg u32 any
  arg buf 4
  stub 0201c1e8 ret=pick:1/1/1/0/2/3/4 args=v
  stub 021d62a8 ret=pick:4/4/0/1/2/3 args=v
  stub 021d59b4 ret=pick:0x3e/0x3e/1/2/60/100 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021c728c force ret=pick:0/1/40/60/100/150/200 args=p,v,v,p
  stub 021c7374 force ret=pick:0/1/40/60/100/150/200 args=p,v,v,p,v
  stub 021c7474 force ret=pick:0/1/40/60/100/150/200 args=p,v,v,p,v
  stub 021d7a44 ret=pick:1/20/100/500 args=v,v,v,v
  stub 021d7b10 ret=pick:0/1/100/1000/60000 args=v,v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d7c38 ret=pick:0x1000/0x2000/0x800/0 args=v,v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021c75ec force ret=pick:0x1000/0x800/0x2000/0x1800 args=p,v,v
  stub 021d7a6c ret=pick:0/1/20/100/500 args=v,v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d7540 ret=none args=v
  stub 021d7630 ret=none args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=

# # try the damage side effect on mon r1: 6cac decides, then the hem bracket at work+0x1ae4, 20bc / 20d0 apply it
fn 021c2048 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c6cac force ret=nullable:3:u32 args=p,v,v,v,outm:4:1/0/0/0
  stub 021c8838 force ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c8860 force ret=none args=p,v
  stub 021c20bc ret=pick:0/1 args=p,v,v
  stub 021c20d0 ret=u32 args=p,v,v,p
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d5fc0 ret=none args=v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c8810 force ret=none args=p
  stub 021c195c ret=none args=p,v,v
  stub 021c4f64 force ret=pick:0/1 args=p,v
  stub 021bc2b0 force ret=pick:0/1 args=p
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021c7330 force ret=pick:0/1 args=p,v

# # 7330 is non-zero
fn 021c20bc ret=bool
  arg work
  arg u32 any
  arg u32 any
  stub 021c7330 force ret=pick:0/1 args=p,v

# # apply -r2 to mon r1 (0 = nothing): 5b74, flag, hem record r3, 195c, 4f64 chain; returns 1 when applied
fn 021c20d0 ret=u32
  arg work
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 any
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d5fc0 ret=none args=v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c8810 force ret=none args=p
  stub 021c195c ret=none args=p,v,v
  stub 021c4f64 force ret=pick:0/1 args=p,v
  stub 021bc2b0 force ret=pick:0/1 args=p
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # side effect on mon r1 keyed by its 5b40 id: placeholder command 0x42 (cd4fc / cd568), cc918 record, 2204 (event 0x90), 6dd0, 88e4 gate, 805c check, 2250
fn 021c2128 ret=u32 inputs=300
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c21f4 ret=pick:0/1 args=p,v
  stub 021c2204 ret=pick:0/1 args=p,v,v
  stub 021c6dd0 force ret=none args=p,v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021d805c ret=pick:0/1 args=v,v
  stub 021c2250 ret=none args=p,v,v,v
  stub 021c7ad0 force ret=none args=p,v,v
  stub 021c60ec force ret=none args=p,v,v
  stub 021dcf24 ret=none args=v
  stub 021d5b44 ret=none args=v,v
  stub 021dcf5c ret=none args=v
  stub 021c7b0c force ret=none args=p,v
  stub 021d6730 ret=none args=v,v
  stub 021c6134 force ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # event message 0x90 about mon r1 and value r2; returns the answer of 775c(0x41)
fn 021c2204 ret=u32
  arg work
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # state change of mon r1: record brackets around 7ad0 and 7b0c, command 0x1e, 5b44(mon, r2); when r3: 6730, command 0x17 and 6134
fn 021c2250 ret=none
  arg work
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5b40 ret=u32 args=v
  stub 021c7ad0 force ret=none args=p,v,v
  stub 021c60ec force ret=none args=p,v,v
  stub 021dcf24 ret=none args=v
  stub 021d5b44 ret=none args=v,v
  stub 021dcf5c ret=none args=v
  stub 021c7b0c force ret=none args=p,v
  stub 021d6730 ret=none args=v,v
  stub 021c6134 force ret=none args=p,v,v

# # 5be4(work, mon, 0) then 4f64(work, mon)
fn 021c2314 ret=none
  arg work
  arg u32 any
  stub 021c5be4 force ret=none args=p,v,v
  stub 021c4f64 force ret=pick:0/1 args=p,v

# # move-effect gate: 236c(move id) then 25a4 unless 5b68(mon r3)
fn 021c232c ret=none
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c236c ret=u32 args=p,v,v,v,out:4
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c25a4 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c8810 force ret=none args=p
  stub 021c264c ret=pick:0/0xffff/1/5/100 args=p,v,v,v
  stub 021c26a8 ret=none args=p,v,v,v,v
  stub 021c26fc ret=none args=p,v,v,v,out:4
  stub 021c8828 force ret=pick:0/1 args=p
  stub 021c2758 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c27a0 ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c2910 ret=none args=p,v,v,v,v,v,v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d8484 ret=none args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=
  stub 0201bd28 ret=u32 args=v,v
  stub 0201bf58 ret=u16 args=v
  stub 021d7c70 ret=none args=v,v,out:4
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021b911c force ret=pick:0/1 args=v,v

# # the move-effect roll: move data 0xb / 0xc, 7c70, event 0x1d/0x26/0x41 answers, 0..100 roll against the percentage; *out = answer; returns the move id or 0
fn 021c236c ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 4
  stub 0201bd28 ret=u32 args=v,v
  stub 0201bf58 ret=u16 args=v
  stub 021d7c70 ret=none args=v,v,out:4
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021b911c force ret=pick:0/1 args=v,v

# # the same effect for every mon of the list r3 (veneers): 25a4 with the roll results, P flag, else 5718
fn 021c2488 ret=none inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0201bd28 ret=u32 args=v,v
  stub 0201bf58 ret=u16 args=v
  stub 021c2564 ret=pick:0/1 args=v
  stub 021c2574 ret=none args=v
  stub 021c2584 seq ret=nullable:3:u32 args=v
  stub 021d7c70 ret=none args=v,v,out:4
  stub 021c25a4 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c2594 seq ret=nullable:3:u32 args=v
  stub 021c5718 force ret=none args=p
  stub 021c8810 force ret=none args=p
  stub 021c264c ret=pick:0/0xffff/1/5/100 args=p,v,v,v
  stub 021c26a8 ret=none args=p,v,v,v,v
  stub 021c26fc ret=none args=p,v,v,v,out:4
  stub 021c8828 force ret=pick:0/1 args=p
  stub 021c2758 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c27a0 ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c2910 ret=none args=p,v,v,v,v,v,v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d8484 ret=none args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=

# # event 0x3f/0x1d message round: 25a4 (stack arg 0 = r4 sentinel 0xffff -> 264c, else 26a8), 26fc, 2758; hem bracket at work+0x1ae4
fn 021c25a4 ret=bool inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt const 0xffff || any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c8810 force ret=none args=p
  stub 021c264c ret=pick:0/0xffff/1/5/100 args=p,v,v,v
  stub 021c26a8 ret=none args=p,v,v,v,v
  stub 021c26fc ret=none args=p,v,v,v,out:4
  stub 021c8828 force ret=pick:0/1 args=p
  stub 021c2758 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c27a0 ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c2910 ret=none args=p,v,v,v,v,v,v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d8484 ret=none args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=

# # message 0x60 about two mons and r3; returns the 775c(0x1d) answer
fn 021c264c ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message 0x61 about two mons, stack arg 0 and r1
fn 021c26a8 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x62: *ptr (stack arg 0) is both an argument and the answer of 775c(0x1e)
fn 021c26fc ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 4
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # when 27a0 answers 0: 2910 with (r1, r2, r3, stack args 0 / 2) and return 1, else 0
fn 021c2758 ret=bool
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c27a0 ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c2910 ret=none args=p,v,v,v,v,v,v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021c99a0 force ret=none args=p,p
  stub 021c8810 force ret=none args=p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d8484 ret=none args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=

# # 282c immunity check -> 5c74 when stack arg 2; else 2b3c / 2ba4 messages and the 0x78a flag; returns the check result 2b3c
fn 021c27a0 ret=bool
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u8 any
  arg u32 alt const 0 || any
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=

# # the immunity check by reason r2 (3, 4, 5, 0x10, 0x12, 0xe, 6..): 0 = none, 1 = hard, 2 = absorbed (ability checks), 3 = other; stack arg 0 byte compared with 0 / 2
fn 021c282c ret=u32
  arg work
  arg u32 any
  arg u32 alt oneof 3 4 5 14 16 18 || range 0 20 || any
  arg u32 any
  arg u8 alt const 2 || const 0 || any
  stub 021d59b4 ret=pick:0x79/0x79/1/2 args=v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=

# # failure message switch on the reason r3 (0..7): messages 0x257 / 0x25a / 0x259 / 0x256 / 0x25b / 0x96 / 0x97 via 5480, then 2aa0, 5444 special case, 2c50 / 2bf0 / 2ca8, 195c
fn 021c2910 ret=none inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg u32 alt range 0 9 || const 16 || any
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d59b4 ret=pick:1/1/0x7f/0x10/2 args=v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021c99a0 force ret=none args=p,p
  stub 021c8810 force ret=none args=p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d8484 ret=none args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v

# # event 0x41: returns ef884() when the event answers 0, else 0
fn 021c2ab0 ret=u32
  arg work
  stub 021d74f8 ret=none args=v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=

# # message 0x7b about mon r1; returns the 775c(0x35) answer
fn 021c2af0 ret=u32
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # event 0x65 message about (r1 or 0x1f), r2, r3; returns the 775c(0x41) answer
fn 021c2b3c ret=u32
  arg work
  arg u32 alt const 0 || any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # event 0x67 message about three values
fn 021c2ba4 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # event 0x68 message about mon r1, (r2 or 0x1f), r3 and stack arg 0
fn 021c2bf0 ret=none
  arg work
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # event 0x69 message about mon r1, (r2 or 0x1f) and r3
fn 021c2c50 ret=none
  arg work
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # event 0x6a message about mon r1 (70e4)
fn 021c2ca8 ret=none
  arg work
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # if r3 is not fainted and 2d10 passes: 2e64
fn 021c2cd8 ret=none
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  fix work 0x778 u32 alt const 0xFFFFFFFF || any
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c2d10 ret=pick:0/1 args=p,p,v,v
  stub 021c2e64 ret=pick:0/1 args=p,p,v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c2e4c ret=u32 args=p
  stub 0201c020 ret=pick:0/1/2/3/5 args=v
  stub 021c2f70 ret=none args=p,v,v,v,v,outm:4:0xF/0/0/0,out:4
  stub 021c3008 ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c78a8 force ret=none args=p,v,v,v,v
  stub 021c312c ret=u32 args=p,v,v,v,v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021c5a4c force ret=none args=p,v,v,v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c77b0 force ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c5a8c force ret=none args=p,v,v,v,v,v
  stub 021c785c force ret=none args=p,v,v,v,v
  stub 021c7820 force ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201c0bc ret=pick:0/1/2/3/8 args=v,v,out:4
  stub 021d75a8 ret=none args=v,v
  stub 0201bd28 ret=u32 args=v,v
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021b911c force ret=pick:0/1 args=v,v

# # event 0x51 roll: move data 0x12, answers 0x41 / 0x26, 0..100 roll; returns 1 on success
fn 021c2d10 ret=bool inputs=300
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  stub 0201bd28 ret=u32 args=v,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021b911c force ret=pick:0/1 args=v,v

# # for every mon of the list r3 (veneers): 2e64 with flag 1, P flag on success
fn 021c2dc4 ret=none inputs=300
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  global 021f3b44 0x40
  fix work 0x778 u32 alt const 0xFFFFFFFF || any
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c2e1c ret=none args=v
  stub 021c2e2c seq ret=nullable:3:u32 args=v
  stub 021c2e64 ret=pick:0/1 args=p,p,v,v,v
  stub 021c2e3c seq ret=nullable:3:u32 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c2e4c ret=u32 args=p
  stub 0201c020 ret=pick:0/1/2/3/5 args=v
  stub 021c2f70 ret=none args=p,v,v,v,v,outm:4:0xF/0/0/0,out:4
  stub 021c3008 ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c78a8 force ret=none args=p,v,v,v,v
  stub 021c312c ret=u32 args=p,v,v,v,v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021c5a4c force ret=none args=p,v,v,v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c77b0 force ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c5a8c force ret=none args=p,v,v,v,v,v
  stub 021c785c force ret=none args=p,v,v,v,v
  stub 021c7820 force ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201c0bc ret=pick:0/1/2/3/8 args=v,v,out:4
  stub 021d75a8 ret=none args=v,v

# # ++*(work + 0x778), wrapping 0 -> 1; returns the new value
fn 021c2e4c ret=u32
  arg work
  fix work 0x778 u32 alt const 0xFFFFFFFF || any

# # multi-hit loop: for hit h in 0..count(move) run 2f70, then 3008 (per target) or the 6-mon loop when the answer is 10; log 0x2037 record around 78a8
fn 021c2e64 ret=bool inputs=400
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  fix work 0x778 u32 alt const 0xFFFFFFFF || any
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c2e4c ret=u32 args=p
  stub 0201c020 ret=pick:0/1/2/3/5 args=v
  stub 021c2f70 ret=none args=p,v,v,v,v,outm:4:0xF/0/0/0,out:4
  stub 021c3008 ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c78a8 force ret=none args=p,v,v,v,v
  stub 021c312c ret=u32 args=p,v,v,v,v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021c5a4c force ret=none args=p,v,v,v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c77b0 force ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c5a8c force ret=none args=p,v,v,v,v,v
  stub 021c785c force ret=none args=p,v,v,v,v
  stub 021c7820 force ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201c0bc ret=pick:0/1/2/3/8 args=v,v,out:4
  stub 021d75a8 ret=none args=v,v

# # event 0x59: hit/target counts for hit idx r2 (0201c0bc), answers 0x1f / 0x20 / 0x35; *out1 = answer (8 -> 0), *out2 = answer * factor
fn 021c2f70 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 4
  arg buf 4
  stub 0201c0bc ret=pick:0/1/2/3/8 args=v,v,out:4
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # accuracy / effect gate for one hit: 312c, 5d14, 77b0 + 5a8c + 785c (hit) or 7820 (miss) with the 0x209d / 0x20a6 records
fn 021c3008 ret=bool inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u8 any
  arg u16 any
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c312c ret=u32 args=p,v,v,v,v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021c5a4c force ret=none args=p,v,v,v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c77b0 force ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c5a8c force ret=none args=p,v,v,v,v,v
  stub 021c785c force ret=none args=p,v,v,v,v
  stub 021c7820 force ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # event 0x5a message about three values and r1's second argument; returns the answer of 775c(0x20)
fn 021c312c ret=u32
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # move-effect for every mon of the list r3 (veneers): 0201bd28 / 0201bf58, 7c70, 2e64 and 25a4 per mon, P flag
fn 021c3184 ret=none inputs=300
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  global 021f3b44 0x40
  fix work 0x778 u32 alt const 0xFFFFFFFF || any
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0201bd28 ret=u32 args=v,v
  stub 0201bf58 ret=u16 args=v
  stub 021d7c70 ret=none args=v,v,out:4
  stub 021c3244 ret=none args=v
  stub 021c3254 seq ret=nullable:3:u32 args=v
  stub 021c2e64 ret=pick:0/1 args=p,p,v,v,v
  stub 021c25a4 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c3264 seq ret=nullable:3:u32 args=v
  stub 021c8810 force ret=none args=p
  stub 021c264c ret=pick:0/0xffff/1/5/100 args=p,v,v,v
  stub 021c26a8 ret=none args=p,v,v,v,v
  stub 021c26fc ret=none args=p,v,v,v,out:4
  stub 021c8828 force ret=pick:0/1 args=p
  stub 021c2758 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c27a0 ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c2910 ret=none args=p,v,v,v,v,v,v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d8484 ret=none args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=
  stub 021c2e4c ret=u32 args=p
  stub 0201c020 ret=pick:0/1/2/3/5 args=v
  stub 021c2f70 ret=none args=p,v,v,v,v,outm:4:0xF/0/0/0,out:4
  stub 021c3008 ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c78a8 force ret=none args=p,v,v,v,v
  stub 021c312c ret=u32 args=p,v,v,v,v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021c5a4c force ret=none args=p,v,v,v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c77b0 force ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c5a8c force ret=none args=p,v,v,v,v,v
  stub 021c785c force ret=none args=p,v,v,v,v
  stub 021c7820 force ret=none args=p,v,v
  stub 0201c0bc ret=pick:0/1/2/3/8 args=v,v,out:4

# # for every mon of the list r3: 3370 / 79e0 + 334c, else 5b48 message; P flag
fn 021c3274 ret=none inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c331c ret=none args=v
  stub 021c332c seq ret=nullable:3:u32 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c3370 ret=pick:0/1 args=p,v
  stub 021c79e0 force ret=u16 args=p,v,v
  stub 021c334c ret=pick:0/1 args=p,v,v,v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 021c333c seq ret=nullable:3:u32 args=v
  stub 021c3394 ret=pick:0/1 args=p,v,v
  stub 021c33e8 ret=none args=p,v,v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d5b7c ret=pick:0/1/1 args=v

# # 3394 gate, then 33e8 (returns 1) or 0
fn 021c334c ret=bool
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c3394 ret=pick:0/1 args=p,v,v
  stub 021c33e8 ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v

# # 5b7c(r1) == 0 or 5b48(r1) != 0
fn 021c3370 ret=bool
  arg work
  arg u32 any
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d5b48 ret=pick:0/1 args=v

# # when mon r1 has flag 0xf: message 0x377 (if r2) + flag 0x20 at work+0x78a; returns the flag
fn 021c3394 ret=bool
  arg work
  arg u32 any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v

# # command 0x4d for mon r1 (3428 id != 6), then 5b74
fn 021c33e8 ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v

# # per-mon loop over the list r3 (veneer iterator): immunity / fail checks (bf324, 6948, bfa00) -> 572c; HP and ability gates via the events table at work+0x1f88
# # (cc8b8), record bracket 0x216b around 6a9c, damage application (3720 / 36a0 / 36e8), 1828, 4218, 1c88 and 4f64
fn 021c3438 ret=none inputs=500
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021c3660 seq ret=nullable:3:u32 args=v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bf324 force ret=pick:0/1 args=p,v,v
  stub 021c6948 force ret=pick:0/1 args=p,v,v
  stub 021bfa00 force ret=pick:0/1 args=p,v,v,v
  stub 021c572c force ret=none args=p,v
  stub 021c5774 force ret=none args=p,v
  stub 021c6a9c force ret=pick:0/1 args=p,v,v,v
  stub 021c3670 ret=u32 args=p,v
  stub 021d7ad4 ret=pick:0/1/2/3/4 args=v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c3720 ret=u32 args=p,v,p,v
  stub 021c3680 ret=none args=v,v,v,v
  stub 021c1678 ret=pick:0/0/1/2/3/4 args=p,v,v,v,out:2
  stub 021c36a0 ret=none args=p,v,p,v
  stub 021c36e8 ret=none args=p,v,p,v,v,v
  stub 021c3690 ret=none args=v,v,v,v
  stub 021c1828 ret=none args=p,v,v,v,p,v
  stub 021c4218 ret=none args=p,v,v,v,v,v,v,v
  stub 021c1c88 ret=none args=p,v,v,p,v
  stub 021c4f64 force ret=pick:0/1 args=p,v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021c4268 ret=none args=p,v,v,p,v,v,v,v
  stub 021d7630 ret=none args=v,v
  stub 021d6848 ret=none args=v,in:7
  stub 021c604c force ret=none args=p,v,v
  stub 021c16f8 ret=none args=p,v,v
  stub 021c1758 ret=none args=p,v
  stub 021d5fb8 ret=none args=v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d6d90 ret=u16 args=v
  stub 021c41c0 ret=none args=p,v
  stub 021c4208 ret=u32 args=p,v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d6d70 ret=none args=v

# # command 3 + 0x32 + 0x34 for mon r1 (5fb8 first); r2 = move record (u16 id at +0), r3 = flag byte
fn 021c36a0 ret=none
  arg work
  arg u32 any
  arg buf 0x10
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d5fb8 ret=none args=v

# # 604c, command 0x32 for mon r1 (move record r2, flag r3), then 16f8 with the mode in stack arg 0
fn 021c36e8 ret=none
  arg work
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg u32 alt range 0 4 || any
  arg u16 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c604c force ret=none args=p,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c16f8 ret=none args=p,v,v
  stub 021c1758 ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # commands 0x34 / 0x32 for mon r1, then 41c0; returns 6d90(mon) (u16)
fn 021c3720 ret=u32
  arg work
  arg u32 any
  arg buf 0x10
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d6d90 ret=u16 args=v
  stub 021c41c0 ret=none args=p,v
  stub 021c4208 ret=u32 args=p,v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d6d70 ret=none args=v

# # for every mon of the list r3 (veneers): 3810 (the full per-mon effect), P flag on success, else 5718
fn 021c376c ret=none inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c37e0 ret=none args=v
  stub 021c37f0 seq ret=nullable:3:u32 args=v
  stub 021c3810 ret=pick:0/1 args=p,v,v,v,outm:4:1/0/0/0,v,v,v
  stub 021c5718 force ret=none args=p
  stub 021c3800 seq ret=nullable:3:u32 args=v
  stub 021c39cc ret=pick:0/1/2 args=p
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d69ac ret=pick:0/1/2/9/24/25/26/255 args=v,v
  stub 021c3a4c ret=pick:0/1 args=p,v,v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021b8e28 force ret=none args=v,v,outm:1:3,out:1
  stub 021bb85c force ret=ptr:work:0x1000:1 args=v,v
  stub 021c39f4 ret=pick:0/1/2/3/0xffffffff args=p,p
  stub 021b990c force ret=nullable:4:u32 args=v,v,v
  stub 021bdd7c force ret=none args=p,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021bd9d4 force ret=none args=p,v,v,v,v
  stub 021bda00 force ret=pick:0/1 args=p
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8244 ret=none args=v,v
  stub 021b9b94 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 021b99c0 force ret=pick:0/1/2 args=v,v
  stub 021b9c00 force ret=nullable:3:u32 args=v,v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d78ac ret=lt:0 args=v
  stub 021d74f8 ret=none args=v
  stub 021d7630 ret=none args=v,v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021c3a98 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b86b8 force ret=pick:0/1/2 args=v

# # one mon's effect: reasons 39cc / 8d8c / 5b68 / 62c4 / 69ac gate it, record 0x221c around 3a4c, then either 39f4 + bdd7c/bd9d4 (r6 == 1) or 8d08 + 8244 + bdd7c
fn 021c3810 ret=bool inputs=500
  arg work
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  arg buf 4
  arg u16 any
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c39cc ret=pick:0/1/2 args=p
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d8c force ret=pick:0/1/2/3/4/5/6 args=v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d69ac ret=pick:0/1/2/9/24/25/26/255 args=v,v
  stub 021c3a4c ret=pick:0/1 args=p,v,v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021b8e28 force ret=none args=v,v,outm:1:3,out:1
  stub 021bb85c force ret=ptr:work:0x1000:1 args=v,v
  stub 021c39f4 ret=pick:0/1/2/3/0xffffffff args=p,p
  stub 021b990c force ret=nullable:4:u32 args=v,v,v
  stub 021bdd7c force ret=none args=p,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021bd9d4 force ret=none args=p,v,v,v,v
  stub 021bda00 force ret=pick:0/1 args=p
  stub 021b8d08 force ret=pick:0/1/2/3 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8244 ret=none args=v,v
  stub 021b9b94 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 021b99c0 force ret=pick:0/1/2 args=v,v
  stub 021b9c00 force ret=nullable:3:u32 args=v,v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d78ac ret=lt:0 args=v
  stub 021d74f8 ret=none args=v
  stub 021d7630 ret=none args=v,v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021c3a98 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b86b8 force ret=pick:0/1/2 args=v

# # 21b8588 / 21b86b8 flags: returns 1 unless both are 0
fn 021c39cc ret=bool
  arg work
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b86b8 force ret=pick:0/1/2 args=v

# # pick a random slot of the party side described by struct r1 (count at [r1+4] via 9b94, first slot byte [r1+0xb]) whose 5b7c answers 1; returns the slot or -1
fn 021c39f4 ret=u32
  arg work
  arg buf 0x10
  stub 021b9b94 force ret=pick:0/1/2/3/4/5/6 args=v
  stub 021b99c0 force ret=pick:0/1/2 args=v,v
  stub 021b9c00 force ret=nullable:3:u32 args=v,v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d78ac ret=lt:0 args=v

# # event 0x8b message about mon r2; returns the 775c(0x41) answer
fn 021c3a4c ret=u32
  arg work
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d7630 ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021c3a98 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # effect by move: 0201c184(move) -> 3c10 + 3b68 (P flag) or 5718; else record 0x22c8 around 3ccc with the 88e4 gate
fn 021c3aa8 ret=none inputs=300
  arg work
  arg buf 0x10
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 0201c184 ret=pick:0/1/2/3/4/5 args=v
  stub 021c3c10 ret=pick:0/1/2/3 args=p,v,v
  stub 021c3b68 ret=pick:0/1 args=p,v,v
  stub 021c5718 force ret=none args=p
  stub 021c3ccc ret=none args=p,v,v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021c3b8c ret=pick:0/1 args=p,v,v
  stub 021c3bb8 ret=none args=p,v,v
  stub 021ef8a4 ret=none args=v,v
  stub 021c3be0 ret=none args=p,v
  stub 021c7998 force ret=none args=p,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=
  stub 021ef894 ret=pick:0/1/0xff args=
  stub 021d75ec ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v

# # 3b8c then 3bb8; returns 3b8c
fn 021c3b68 ret=bool
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c3b8c ret=pick:0/1 args=p,v,v
  stub 021c3bb8 ret=none args=p,v,v
  stub 021ef8a4 ret=none args=v,v
  stub 021c3be0 ret=none args=p,v
  stub 021c7998 force ret=none args=p,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=
  stub 021ef894 ret=pick:0/1/0xff args=

# # (r1 < 5 and r1 == ef884() and not (r2 == 0xff and ef894() == 0xff)) -> 1
fn 021c3b8c ret=bool
  arg work
  arg u32 alt range 0 6 || any
  arg u32 alt const 255 || any
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=
  stub 021ef894 ret=pick:0/1/0xff args=

# # ef8a4, command 0x3f, 3be0
fn 021c3bb8 ret=none
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021ef8a4 ret=none args=v,v
  stub 021c3be0 ret=none args=p,v
  stub 021c7998 force ret=none args=p,v

# # 0x2303 record bracket around 7998
fn 021c3be0 ret=none
  arg work
  arg u32 any
  stub 021c7998 force ret=none args=p,v

# # event 0x7c message about value r1 and mon r2; returns the 775c(0x24) answer (u8)
fn 021c3c10 ret=u8
  arg work
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # varargs (push {r0-r3}): ef8c8(r1, r2) -> command 0x21 or (r3 != 0 and 8a98(r2) != 0x1f) -> ef8f8 + command 0x22; returns 1 when a command was queued
fn 021c3c60 ret=bool
  arg work
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021ef8c8 ret=pick:0/1 args=v,v
  stub 021e8a98 ret=pick:0/0x1f/1/2/3 args=v
  stub 021ef8f8 ret=none args=v,v

# # event 0x9e message
fn 021c3ccc ret=none
  arg work
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # by the move id (u16 at +0): 0xa4 -> 4034 (+ P flags), 285 -> 3e0c, else record 0x237f around 4304 with the 88e4 gate and message 0x47
fn 021c3d08 ret=none inputs=500
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  fix arg1 0 u16 alt const 164 || const 285 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c3dfc ret=pick:0/1 args=v
  stub 021c3e0c ret=none args=p,v,v
  stub 021c4034 ret=pick:0/1 args=p,v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021c4304 ret=pick:0/1 args=p,p,v,v,out:4
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021c43c0 ret=pick:0/1 args=v
  stub 021c43d0 ret=pick:0/1/2/3/4/5/6 args=v
  stub 021d7630 ret=none args=v,v
  stub 021c43e0 ret=u32 args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021d7b74 ret=pick:0/1/10/100 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021c40f4 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d6d5c ret=none args=v,v
  stub 021c5dac force ret=none args=p,v,v,v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021c3fac ret=u32 args=v,v
  stub 021c3fbc ret=pick:0/1 args=v
  stub 021c3fcc ret=pick:0/1 args=v
  stub 021c7b3c force ret=none args=p,v,v,v
  stub 021d66d0 ret=none args=v,v
  stub 021d8564 ret=none args=v,v
  stub 021c7b7c force ret=none args=p,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c3fdc ret=none args=p,v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v

# # Skill Swap style exchange between mon r1 and the first mon of list r2 (3fac): BattleMon field 0x10 (ability) of both, commands 0x4a / 0x58, records 0x23b4 around 7b3c / 7b7c, 66d0, 8564; Klutz (0x67) / Unnerve (0x7f) re-checks via 195c / 3fdc; message 0x47 when refused
fn 021c3e0c ret=none inputs=500
  arg work
  arg u32 any
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  region P 0x20
  fix work 0x4a8 u32 addrof region:P
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d59b4 ret=pick:0x67/0x7f/5/6/0x67/0x7f args=v,v
  stub 021c3fac ret=u32 args=v,v
  stub 021c3fbc ret=pick:0/1 args=v
  stub 021c3fcc ret=pick:0/1 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c7b3c force ret=none args=p,v,v,v
  stub 021d66d0 ret=none args=v,v
  stub 021d8564 ret=none args=v,v
  stub 021c7b7c force ret=none args=p,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c195c ret=none args=p,v,v
  stub 021c3fdc ret=none args=p,v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # for every mon returned by bd34c (forced stub, seq): 195c unless 9a24
fn 021c3fdc ret=none inputs=300
  arg work
  arg u32 any
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # pays 7b74(mon, 4) (a quarter of the max HP) when its HP is above that: 5b74, 195c, 6d5c, commands 0x27 / 0x51, message 0x311, returns 1; otherwise 5dac and 0 (also 0 when 6d7c(mon))
fn 021c4034 ret=bool
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021d7b74 ret=pick:0/1/10/100 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c40f4 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d6d5c ret=none args=v,v
  stub 021c5dac force ret=none args=p,v,v,v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # varargs (push {r0-r3}); r1 = mon, r3 = u16, stack arg 0 = flag byte, stack arg 1 = value, stack arg 2 = move record; command 0x54, 5840 / 59c4 (in/out slots), 41c0, 1dcc, 1880, 4218; returns the u16 slot
fn 021c4104 ret=u16 inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  fix work 0x778 u32 alt const 0xFFFFFFFF || any
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021c5840 force ret=none args=p,v,p,p,v
  stub 021c59c4 force ret=none args=p,v,p,p,v
  stub 021d6d9c ret=pick:0/1 args=v,out:4
  stub 021c41c0 ret=none args=p,v
  stub 021c1dcc ret=none args=p,p,v,v,v
  stub 021c1880 ret=none args=p,p,v,v,v,v
  stub 021c4218 ret=none args=p,v,v,v,v,v,v,v
  stub 021c4268 ret=none args=p,v,v,p,v,v,v,v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 0201c1bc ret=pick:0/1/2/3/4/5/5/6/7/8/9/9/10/11/12/13/13/20 args=v
  stub 021c2cd8 ret=none args=p,v,v,v
  stub 021c232c ret=none args=p,p,v,v
  stub 021c236c ret=u32 args=p,v,v,v,out:4
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021c25a4 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c8810 force ret=none args=p
  stub 021c264c ret=pick:0/0xffff/1/5/100 args=p,v,v,v
  stub 021c26a8 ret=none args=p,v,v,v,v
  stub 021c26fc ret=none args=p,v,v,v,out:4
  stub 021c8828 force ret=pick:0/1 args=p
  stub 021c2758 ret=pick:0/1 args=p,v,v,v,v,v,v
  stub 021c99a0 force ret=none args=p,p
  stub 021c27a0 ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c2910 ret=none args=p,v,v,v,v,v,v,v
  stub 021c5c1c force ret=none args=p,v,v,v
  stub 021c5480 force ret=none args=p,v,v
  stub 021c2aa0 ret=none args=v,v,v,p
  stub 021d5444 ret=pick:0/492/492/1 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d8510 ret=none args=v
  stub 021d66d8 ret=none args=v,v
  stub 021d8484 ret=none args=v
  stub 021d7d6c ret=pick:0/1 args=v
  stub 021c2bf0 ret=none args=p,v,v,v,v
  stub 021c2ca8 ret=none args=p,v
  stub 021c3fdc ret=none args=p,v
  stub 021c2c50 ret=none args=p,v,v,v
  stub 021c195c ret=none args=p,v,v
  stub 021d5b40 ret=u32 args=v
  stub 021c199c ret=none args=p,v,v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021b9a24 force ret=pick:0/1 args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021c282c ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c5c74 force ret=none args=p,v,v,v
  stub 021c2b3c ret=pick:0/1 args=p,v,v,v
  stub 021c2ba4 ret=none args=p,v,v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021d62a8 ret=pick:0/1/2/3/4 args=v
  stub 021c2ab0 ret=pick:0/1/2 args=p
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8d24 ret=pick:0/1 args=v,v
  stub 021ef884 ret=pick:0/1/2/3/4/0xff args=
  stub 0201bd28 ret=u32 args=v,v
  stub 0201bf58 ret=u16 args=v
  stub 021d7c70 ret=none args=v,v,out:4
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021b911c force ret=pick:0/1 args=v,v
  stub 021c2d10 ret=pick:0/1 args=p,p,v,v
  stub 021c2e64 ret=pick:0/1 args=p,p,v,v,v
  stub 021c2e4c ret=u32 args=p
  stub 0201c020 ret=pick:0/1/2/3/5 args=v
  stub 021c2f70 ret=none args=p,v,v,v,v,outm:4:0xF/0/0/0,out:4
  stub 021c3008 ret=pick:0/1 args=p,v,v,v,v,v,v,v,v,v
  stub 021c78a8 force ret=none args=p,v,v,v,v
  stub 021c312c ret=u32 args=p,v,v,v,v,v
  stub 021d5d14 ret=pick:0/1 args=v,v,v
  stub 021c5a4c force ret=none args=p,v,v,v
  stub 021d6d7c ret=pick:0/1 args=v
  stub 021c77b0 force ret=pick:0/1 args=p,v,v,v,v,v
  stub 021c5a8c force ret=none args=p,v,v,v,v,v
  stub 021c785c force ret=none args=p,v,v,v,v
  stub 021c7820 force ret=none args=p,v,v
  stub 0201c0bc ret=pick:0/1/2/3/8 args=v,v,out:4
  stub 021d7b3c ret=pick:0/1/2/3/100/200 args=v,v
  stub 021c1e14 ret=u32 args=p,v,v,v
  stub 021c78f4 force ret=nullable:3:u32 args=p,v,v,v
  stub 021c3370 ret=pick:0/1 args=p,v
  stub 021c334c ret=pick:0/1 args=p,v,v,v
  stub 021c3394 ret=pick:0/1 args=p,v,v
  stub 021c33e8 ret=none args=p,v,v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d5b7c ret=pick:0/1/1 args=v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 021c4208 ret=u32 args=p,v
  stub 021d6d70 ret=none args=v

# # commands 0x28 (4208 id) / 0x52, 5d88, 6d70 for mon r1
fn 021c41c0 ret=none
  arg work
  arg u32 any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c4208 ret=u32 args=p,v
  stub 021c5d88 force ret=none args=p,v,v
  stub 021d6d70 ret=none args=v

# # 0x245e record bracket around 4268
fn 021c4218 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021c4268 ret=none args=p,v,v,p,v,v,v,v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x4a / 0x4b / 0x4c about mon r1, mon r2 and the move record r3 (u16 +0/+2, u8 +6, u32 +8) and four values
fn 021c4268 ret=none
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0xa0 / 0xa1 about the list r3 (43c0 flag, 43d0 count, 43e0 mons); *out = answer; returns the 88e4 gate
fn 021c4304 ret=bool inputs=300
  arg work
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg buf 4
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021c43c0 ret=pick:0/1 args=v
  stub 021c43d0 ret=pick:0/1/2/3/4/5/6 args=v
  stub 021d7630 ret=none args=v,v
  stub 021c43e0 ret=u32 args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021c88e4 force ret=pick:0/1/2 args=p

# # for every mon returned by bd34c (forced stub, seq): 444c(list, mon); 443c / 445c around it
fn 021c43f0 ret=none inputs=300
  arg work
  arg u32 any
  stub 021c443c ret=none args=v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021c444c ret=none args=v,v
  stub 021c445c ret=none args=v,v

# # the turn-end state machine (state byte at work+0x77e, 0..6): 43f0 once, then cases calling 4d1c / 4838 / 493c / 4b8c / 4c24, 4744, 4614; returns 1 when interrupted
fn 021c446c ret=u32 inputs=500
  arg work
  fix work 0x77e u8 range 0 8
  fix work 0x10 u32 alt range 0 0x2800 || any
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  fix work 0x1009 u8 range 0 4
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021c43f0 ret=none args=p,v
  stub 021c4744 ret=none args=p
  stub 021c4d1c force ret=pick:0/1 args=p,v
  stub 021bc2b0 force ret=pick:0/1 args=p
  stub 021c4838 force ret=pick:0/1 args=p,v,v
  stub 021c493c force ret=pick:0/1 args=p,v
  stub 021c4b8c ret=none args=p
  stub 021c4c24 force ret=none args=p
  stub 021c45e4 ret=none args=v
  stub 021c45f4 seq ret=nullable:3:u32 args=v
  stub 021d6408 ret=none args=v
  stub 021d6e88 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c4604 seq ret=nullable:3:u32 args=v
  stub 021c4614 ret=none args=p
  stub 021d701c ret=none args=
  stub 021b8588 force ret=pick:0/1/2/3 args=v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021b9064 force ret=pick:0/1/2/3 args=v,v,v
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9b98 force ret=pick:0/1/1/2 args=v
  stub 021b9ce8 force ret=u32 args=v
  stub 021c4724 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c4734 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021b8dec force ret=pick:0/1/2/3 args=v,v
  stub 021b99f0 force ret=pick:0/1 args=v
  stub 021bd43c force ret=none args=p,v,v,v,v
  stub 021bd4e4 force ret=none args=p,v,v,v
  stub 021b8630 force ret=pick:0/1/2 args=v
  stub 0202e7e0 ret=pick:0/1/2/3 args=
  stub 021bb85c force ret=ptr:work:0x1000:1 args=v,v
  stub 021b990c force ret=nullable:4:u32 args=v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 0203f0a8 ret=u32 args=
  stub 021b9934 force ret=u32 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021c334c ret=pick:0/1 args=p,v,v,v
  stub 0202e7e4 ret=none args=v
  stub 021c3394 ret=pick:0/1 args=p,v,v
  stub 021c33e8 ret=none args=p,v,v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
  stub 021c443c ret=none args=v
  stub 021bd2b4 force ret=none args=p,p
  stub 021bd34c force ret=pick:0/1 args=p,p,out:4 seq
  stub 021c444c ret=none args=v,v
  stub 021c445c ret=none args=v,v

# # when 8588 == 2 and both sides' party slots pass (9b98 == 1): 4724 / 4734 slots, command 0x50 and the four bd43c / bd4e4 forced calls
fn 021c4614 ret=none inputs=800
  arg work
  stub 021b8588 force ret=pick:2/2/2/0/1 args=v
  stub 021b9b98 force ret=pick:1/1/1/0/2 args=v
  stub 021b8dec force ret=pick:0/1 args=v,v
  stub 021b99f0 force ret=pick:0/0/0/1 args=v
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021b9064 force ret=pick:0/1/2/3 args=v,v,v
  stub 021b9aec force ret=u32 args=v,v
  stub 021b9ce8 force ret=u32 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021c4724 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c4734 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021bd43c force ret=none args=p,v,v,v,v
  stub 021bd4e4 force ret=none args=p,v,v,v

# # picks a random alive mon of the side (bb85c record, byte 9 = count <= 4) and applies half / full of its max HP through 334c, message 0x53
fn 021c4744 ret=none inputs=400
  arg work
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  fix work 0x1009 u8 range 0 4
  stub 02034714 noreturn ret=none args=v,v,v
  stub 021b8630 force ret=pick:0/1/2 args=v
  stub 0202e7e0 ret=pick:0/1/2/3 args=
  stub 021b8ff4 force ret=pick:0/1/2/3 args=v
  stub 021bb85c force ret=ptr:work:0x1000:1 args=v,v
  stub 021b990c force ret=nullable:4:u32 args=v,v,v
  stub 021d5b68 ret=pick:0/1 args=v
  stub 021d5b48 ret=pick:0/1 args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 0203f0a8 ret=u32 args=
  stub 021b9934 force ret=u32 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021c334c ret=pick:0/1 args=p,v,v,v
  stub 0202e7e4 ret=none args=v
  stub 021c3394 ret=pick:0/1 args=p,v,v
  stub 021c33e8 ret=none args=p,v,v
  stub 021c3428 ret=pick:0/1/2/3/4/5/6 args=p,v
  stub 021c5b74 force ret=none args=p,v,v,v
  stub 021d62c4 ret=pick:0/1 args=v,v
