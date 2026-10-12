# svflow spec: core6 (0x021C65A8-0x021C7CBC; B10k, written by B10k-spec). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md (B10-specs / B10-spec2 / B10f-spec / B10f-spec2 / B10i-spec / B10j-spec additions). No new grammar.
#
# What this area is: the sixth and last part of the flow core (btl_server_flow.c, up to the flow API at 0x021C7CBC): the message round
# trips between the server flow and the client (start 021d74f8, add arguments 021d75ec / 75a8 / 7630 / 7674, send 021d70f0 / 70e4, read back
# 021d775c, end 021d7540) for the damage / ability / item questions of the turn ("may this mon do X", "scale this by Y"), the type-chart
# check pair 6e3c / 7038 / 6f0c / 6f34 / 6fc0, one command emitter (70e4) and one statistics counter (7c24).
#   65a8 .. 6e00     list / mon message round trips (list = an object read through the `bx pc` veneers 6648 6658 6710 6828 6838 6928 6938)
#   6e3c             type-chart check of a move type against the defender (7038 twice, 6f0c, 5750 or 6fc0 at the end)
#   6f0c / 6f34      "does the defender's type pair resist" message pair (ef948(2) is a library accessor, stubbed)
#   6fc0             bracket on the packed queue-header word (work + 0x1D74: cc8d8 ... cc95c) around 7008 and 5750
#   70e4             emits command 0x4C / 0x4D / 0x4E on the queue behind work + 0xC (cd4b0, or cd568 at position r5 when r4 != 0)
#   7194 .. 7bf8    the remaining message round trips
#   7c18             `MI_CpuFill8(r0, 0, 0xC)` (8-byte wrapper: `ldr r3,[pc]; movs r1,0; movs r2,0xC; bx r3`; MI_CpuFill8 is in svflow.py ALLOW)
#   7c24             statistics counters: six u16 at r0 (saturating at 9999), selected by 8d08 / 9a24 of the two mons and stack0
#
# Register facts the pseudo-C hides (read off the disassembly): 75ec reads r0..r2 only (Ghidra shows 4 params) and passes the reply of slot
# 0x51 on to 021d75ec(0x44, reply) (a second hidden argument of a callee); 6f34 reads r0..r2 only and calls 5814(mon, 2) with two
# arguments (Ghidra shows 5); 71e0 writes bytes 0, 1, 2, 3 and 5 of its record (the pseudo-C shows 0, 2, 3); 6a04 returns the result of its last
# call 7b10 (Ghidra says void); 6e3c passes the 8cfc result to 7038 on the stack; 8cfc / 8d04 / 8da8 take the 57fc pair in r0 (Ghidra shows
# none). 7474 / 7374 take a flag on the stack. 6cac takes its out word on the stack. 65a8 writes a u16 through the pointer stored at
# work + 0x4A8 (region D here). 21c2ab0 is called with the work pointer in r0.
#
# SvFlowWork fields seen here: +0x04 main module (address only, 6bf8 / 76f4 / 7374 hand a word of it to a stub), +0x08 (7374: handed to 9934),
# +0x0C command queue buffer (70e4; region Q), +0x1D74 packed queue-header word (6fc0), +0x4A8 pointer to a u16 (65a8).
#
# Layout of the blocks: as core4 / core5. Every block lists (a) the stubs of its own callees AND of the callees of all its in-area descendants
# and (b) the resource lines of the function and its descendants (only 70e4 has any: region Q for cd4b0 / cd568). The only in-area call
# chains are 6e3c -> {7038, 6f0c, 6f34, 6fc0, 7008}, 6f0c -> 6f34, 6fc0 -> 7008, so porting any core6 function needs no spec edit,
# whatever order the area is ported in (checked: with every core6 function treated as ported, i.e. the whole tree real on the ROM side,
# 52 / 52 functions run without a fault, a stray write or an unstubbed callee; `svflow.py core6 --all --strict-callees` is clean).
#
# Rules (README): callees ported in cmd_queue run as REAL code on both sides (021cc8d8 021cc95c 021cd4b0 021cd568). Callees in the other
# flow areas (core3 bf324, core4 c2ab0, core5 5750, api 82b8 / 88e4) are `force` stubs, so these specs stay valid as those areas land
# and the Rust ports keep calling them through env.call. Everything else outside the area (ARM9 main 0201xxxx accessors, 021b8xxx / 021b9xxx
# BattleMon helpers, the 021d5xxx..7xxx message builders and type-chart helpers, 021e8xxx / 021ef948 accessors) is a stub; the type-chart
# helpers 794c / 79e8 / 8cfc / 8d04 / 8da8 / 7930 are leaf lookups that could run as real code, but they stay stubs (the callers' answers
# only matter as small numbers). Message argument ids are the first stub argument and are logged, so a port that sends a wrong id
# or value changes the hash. Stub arguments follow the REGISTERS set at each call site. A pointer that is only handed on is `p`, a scalar `v`.
# Stubs whose result is only a small number use `ret=pick:...`; the list-size veneers (6648 6828 6928) are `pick:0/../6`, so the member loops
# are short; the list-member veneers (6658 6838 6938) return a mon handle (u32).
#
# Argument-shape notes: the first argument of most functions is the object handed to 021d70f0 (the server / work pointer, `arg u32 any`);
# it is `arg work` where the function reads it (65a8, 6bf8, 6e3c, 6fc0, 70e4, 7374). 676c: r3 is not read; stack0 / stack1 are the out
# pointers (u8 reply / int reply). 6cac: stack0 is a u32 out pointer. 6a04 / 6b5c / 6e3c / 71e0 / 728c / 7374 / 7474 take a pointer to a
# record (move record: u16 id at +0, byte at +6, u32 at +8; 71e0 writes a 6-byte result). 7ba8: stack0 is an int in/out pointer.
# 7c24: r1 is not read; stack0 is the signed slot selector.
#
# Uninitialised stack: none found that changes a result (the emulator zeroes the stack anyway).
#
# `--strict-callees` clean; no ALLOW addition in svflow.py.
# Dead code: 4 instructions of 70e4 (the `iVar1 != 2` returns at 021c710e/7110 and 021c7160/7162: iVar1 is 0, 1 or 2).
#
# Skipped (not specced): 9 two-byte `bx pc` veneers into unloaded 0x0689xxxx code (list accessors; their callers stub them at these
# addresses) and the two 8-byte thunk halves 021c6fb4 / 021c70d8 that follow two of them (`ldr ip,[pc]; bx ip` to 0x0689c6e4 / 0x0689c540):
#   021c6648 021c6658 021c6710 021c6828 021c6838 021c6928 021c6938 021c6fb0 021c70d4
# 52 real functions are specced. Smoke ports (B10k-spec, in core6.rs): see the PORTED list there.

area core6

# # message 0x2d0d: list count 6648(list), members 6658(list, i); writes the u16 reply 0x13 through the pointer at work + 0x4a8
fn 021c65a8 ret=u32 inputs=300
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  region D 0x10
  fix work 0x4a8 u32 addrof region:D
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021c6648 ret=pick:0/1/2/3/4/5/6 args=v
  stub 021c6658 ret=u32 args=v,v

# # message 0x2d36, reply 0x51
fn 021c6668 ret=u32
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message 0x2d..: mon r1, optional second mon from 6710(list, 0), reply 0x41
fn 021c66ac ret=u32
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
  stub 021c6710 ret=nullable:3:u32 args=v,v

# # message: mon r1, value r2, reply 0x51
fn 021c6720 ret=u32
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

# # message 0x2d80: list r2 (6828 count, 6838 members); stack0 = u8 out (reply 6 or 0x1f), stack1 = int out (reply 0x4f); r3 is not read
fn 021c676c ret=bool inputs=300
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 4
  arg buf 4
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021c6828 ret=pick:0/1/2/3/4/5/6 args=v
  stub 021c6838 ret=u32 args=v,v

# # message, id 0x96
fn 021c6848 ret=none
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message, id 0x97
fn 021c6878 ret=none
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x2dd..: mon r1 + list r2 members, returns reply 0x41 == 0
fn 021c68a8 ret=bool
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
  stub 021c6928 ret=pick:0/1/2/3/4/5/6 args=v
  stub 021c6938 ret=u32 args=v,v

# # 5c08(mon r2) != 0x10: message 0x2df5 with both mons and that value, returns reply 0x42 != 0 (else 0)
fn 021c6948 ret=bool inputs=300
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
  stub 021d5c08 ret=pick:0/1/2/16 args=v

# # c1f4(r3) == 0: message 0x2e15, returns reply 0x51; else returns c1f4(r3)
fn 021c69b8 ret=u32
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
  stub 0201c1f4 ret=pick:0/0/1/5 args=v

# # message: r3 = move record (u16 id at +0, u32 at +8); bd28(id, 4) ... 7b10(reply 0x2b, reply 0x35)
fn 021c6a04 ret=u32 inputs=300
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201bd28 ret=u32 args=v,v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d7b10 ret=u32 args=v,v

# # 59b4(mon r1, 0xf) vs 59b4(mon r2, 0xf): message 0x2.. (reply 0x41); bf324 (core3); percent roll against bd28(r3, 4)
fn 021c6a9c ret=u32 inputs=400
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
  stub 021d59b4 ret=u32 args=v,v
  stub 0201bd28 ret=u32 args=v,v
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021bf324 force ret=pick:0/1 args=p,v,v

# # message with the byte at r3 + 6 (id 0x16), reply 0x51
fn 021c6b5c ret=u32
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message with mons r1, r2
fn 021c6bb8 ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # bd28(r3, 7) + 5f14(mon r1) -> message (0x2c); reply 0x41 == 0: c284(r3) / 7930(min(reply 0x2c, 4)); 911c(work[4], 3) -> 1
fn 021c6bf8 ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201bd28 ret=u32 args=v,v
  stub 021d5f14 ret=pick:0/1/2/3 args=v
  stub 0201c284 ret=pick:0/1 args=v
  stub 021d7930 ret=pick:0/1 args=v
  stub 021b911c ret=pick:0/1 args=v,v

# # out word at stack0; bd28(r2, 0x1e/0x1f), c1bc(r2) == 8 -> 0; message 0x50 with replies 0x35/0x41/0x51/0x36; scales via 7b3c
fn 021c6cac ret=u32 inputs=400
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 4
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201bd28 ret=pick:0/1/5/200 args=v,v
  stub 0201c1bc ret=pick:0/1/2/8 args=v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d7b3c ret=pick:0/1/2/3/100/200 args=v,v

# # message id 0x72
fn 021c6dd0 ret=none
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message id 0x73 sent through 021d70e4
fn 021c6e00 ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70e4 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # type effectiveness style check of move type (byte r3 + 6) against 57fc(mon r2): 7038 twice, 6f0c(…, 1/0), then 5750 / 6fc0 when r4 flag is set
fn 021c6e3c ret=u32 inputs=500
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  fix arg3 6 u8 alt const 0x11 || const 4 || range 0 20
  stub 021c7038 ret=pick:0/1/2/3 args=p,v,v,v,v
  stub 021c6f0c ret=pick:0/1 args=p,v,v
  stub 021c6f34 ret=pick:0/1 args=p,v,v
  stub 021c6fc0 ret=none args=p,v
  stub 021c7008 ret=none args=p,v
  stub 021d57fc ret=pick:0/1/2/3/4/5/6/7 args=v
  stub 021e8cfc ret=pick:0/1/2/3/4/17 args=v
  stub 021e8d04 ret=pick:0/1/2/3/4/17 args=v
  stub 021e8da8 ret=pick:0/1 args=v
  stub 021d79e8 ret=pick:0/1/2/4/5 args=v,v
  stub 021ef948 ret=pick:0/1 args=v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021c70d4 ret=none args=p,v
  stub 021d794c ret=pick:0/1/2/4 args=v,v
  stub 021d5814 ret=pick:0/1 args=v,v
  stub 021c6fb0 ret=none args=p,v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021c5750 force ret=none args=p,v

# # ef948(2) == 0 && 6f34(…) != 0
fn 021c6f0c ret=bool
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || range 0 5
  stub 021c6f34 ret=pick:0/1 args=p,v,v
  stub 021ef948 ret=pick:0/1 args=v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021d5814 ret=pick:0/1 args=v,v
  stub 021c6fb0 ret=none args=p,v

# # 5814(mon r1, 2) (when r2 != 0); message 0x2fbc with replies 0x51 / 0x41; 6fb0 veneer
fn 021c6f34 ret=u8 inputs=300
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || range 0 5
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021d5814 ret=pick:0/1 args=v,v
  stub 021c6fb0 ret=none args=p,v

# # cc8d8 bracket on work + 0x1d74 around 7008 and (when 88e4 != 2) 5750
fn 021c6fc0 ret=none
  arg work
  arg u32 any
  stub 021c7008 ret=none args=p,v
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 021c88e4 force ret=pick:0/1/2 args=p
  stub 021c5750 force ret=none args=p,v

# # message id 0x1b
fn 021c7008 ret=none
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x3002: mons r1, r2, ids r3 (0x16) and stack0 (0x15); replies 0x4b / 0x4c; 794c(r3, stack0)
fn 021c7038 ret=u8 inputs=300
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
  stub 021c70d4 ret=none args=p,v
  stub 021d794c ret=pick:0/1/2/4 args=v,v

# # cd4b0 / cd568 emitter of command 0x4c / 0x4d / 0x4e (u16 header from the stack when r4 != 0)
fn 021c70e4 ret=none inputs=400
  arg work
  arg u32 any
  arg u32 alt const 6 || range 0 5
  arg u32 alt const 6 || range 0 5
  arg u32 alt const 0 || any
  arg u16 alt range 0 2900 || near 2990 24
  region Q 0xC00
  fix work 0xC u32 addrof region:Q
  fix region:Q 0 u32 range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# # message id 0x4f, reply 0x51
fn 021c7194 ret=u32
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

# # out record at r3 (bytes 0..3 and 5); bd28(r2, 8) > 1: message 0x35 with replies 0x51 / 0x2a / 0x42
fn 021c71e0 ret=bool inputs=300
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 8
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201bd28 ret=pick:0/1/2/3/5/6/9 args=v,v
  stub 021d7ba4 ret=u32 args=v

# # record r3 (u16 at +0, byte at +6, u32 at +8); two sends 0x37 / 0x38; 7b10(reply 0x30, reply 0x31)
fn 021c728c ret=u16
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201c1d8 ret=u32 args=v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d7b10 ret=u32 args=v,v

# # message id 0x80, reply 0x51
fn 021c7330 ret=u32
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # record r3 as in 728c; replies 0x3b (0x1f -> mon from 9934(work[8])), 0x51; 58a8 / 5ad8 / 59b4 by flag
fn 021c7374 ret=u32 inputs=400
  arg work
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v
  stub 0201c1e8 ret=pick:0/1/2/3/4 args=v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d5ad8 ret=u32 args=v,v
  stub 021d58a8 ret=u32 args=v,v
  stub 021d7b10 ret=u32 args=v,v
  stub 021b9934 ret=u32 args=v,v
  stub 021d775c ret=pick:0/1/2/31/31/50/65535 args=v

# # two messages 0x3a / 0x3c; type multiplier id 9 / 0xb; c2ab0(work) == 4 && 5814(mon, 5) && id 0xb -> 7b10(x, 0x1800)
fn 021c7474 ret=u16 inputs=500
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201c1e8 ret=pick:0/1/2/3/4 args=v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d5ad8 ret=u32 args=v,v
  stub 021d58a8 ret=u32 args=v,v
  stub 021d7b10 ret=u32 args=v,v
  stub 021d5814 ret=pick:0/1 args=v,v
  stub 021c2ab0 force ret=pick:0/1/2/4 args=p

# # 5814(mon r1, r2); messages 0x40 (reply 0x51) and 0x41 (value 0x1000 / 0x1800), returns reply 0x35
fn 021c75ec ret=u32
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021d5814 ret=pick:0/1 args=v,v

# # bd28(r1, 10); message 0x6c with replies 0x41 / 0x45 / 0x26
fn 021c7664 ret=u32 inputs=300
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201bd28 ret=pick:0/1/5/200 args=v,v

# # message 0x6d (reply 0x26 / 0x41); 82b8(r0) or a percent roll against the reply
fn 021c76f4 ret=bool inputs=300
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021d78ac ret=pick:0/1/5/10/50/99 args=v
  stub 021c82b8 force ret=pick:0/1 args=p

# # message id 0x6e
fn 021c7780 ret=none
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message 0x5b with ids r3, r2, stack0, stack1; reply 0x41 == 0
fn 021c77b0 ret=bool
  arg u32 any
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

# # message id 0x5c
fn 021c7820 ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message id 0x5d; r1 is a raw value, r2 the mon
fn 021c785c ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message id 0x5e
fn 021c78a8 ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # messages 0x8c / 0x8d around 7b2c(reply 0x20, reply 0x35); r2 == 0 -> id 0x1f
fn 021c78f4 ret=u32 inputs=300
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 021d7674 ret=none args=v,v,v,v
  stub 021d7b2c ret=u32 args=v,v
  stub 021d76bc ret=none args=v,v

# # messages 0x7d (and 0x7e when reply 0x41 == 0)
fn 021c7998 ret=none
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d7630 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # bd28(r1, 0x1a); message 0x8f reply 0x35; scaled by 7b10 / 7b3c, clamped to 59b4(mon, 0xe)
fn 021c79e0 ret=u32 inputs=300
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v
  stub 0201bd28 ret=pick:0/1/5/200 args=v,v
  stub 021d59b4 ret=u32 args=v,v
  stub 021d7b3c ret=pick:0/1/2/3/100/200 args=v,v
  stub 021d7b10 ret=u32 args=v,v
  stub 021d75ec ret=none args=v,v

# # message id 0x9a, reply 0x41
fn 021c7a54 ret=u32
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

# # message id 0x9b
fn 021c7aa0 ret=none
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message id 0x9c
fn 021c7ad0 ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message id 0x9d
fn 021c7b0c ret=none
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message id 0x89 (raw values)
fn 021c7b3c ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message id 0x8a (raw value)
fn 021c7b7c ret=none
  arg u32 any
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # message id 0x9f; the int at stack0 is both sent (id 0x1e) and replaced by the reply
fn 021c7ba8 ret=none
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 4
  stub 021d74f8 ret=none args=v
  stub 021d75ec ret=none args=v,v
  stub 021d75a8 ret=none args=v,v
  stub 021d70f0 ret=none args=p,v
  stub 021d775c ret=pick:0/0/1/2/8/50/100/4096/65535 args=v
  stub 021d7540 ret=none args=v

# # message id 0x79
fn 021c7bf8 ret=none
  arg u32 any
  stub 021d74f8 ret=none args=v
  stub 021d70f0 ret=none args=p,v
  stub 021d7540 ret=none args=v

# # MI_CpuFill8(r0, 0, 0xc) (8-byte thunk to the library fill)
fn 021c7c18 ret=none
  arg buf 0x10

# # statistics counters (6 u16 at r0, cap 9999): index by 8d08 of both mons and 9a24, stack0 = slot selector
fn 021c7c24 ret=none inputs=400
  arg buf 0x10
  arg u32 any
  arg u32 any
  arg u32 any
  arg s32 alt range 0 6 || any
  fix arg0 0 u16 alt near 9999 3 || any rep=6,2
  stub 021d5440 ret=pick:0/1/2/3/4/5 args=v
  stub 021b8d08 ret=pick:0/1/2/3 args=v
  stub 021b9a24 ret=pick:0/1 args=v,v
