# fndiff spec (F14b): the field PROCLINK event, `event_field_proclink.c` first part (overlay 10 of the field set,
# 0x0216075C..0x02160A92; F1 put these 6 functions in row 10.21 `event_shortcut_menu.c`, the file boundary is 0x0216075C: see
# crates/black-field/README.md "F14b"). Run: tools/check-fn.sh black-field proclink_event --per-function
#
# The event launches the app named by the PROCLINK table (ROM 0x021712CC, 12 entries of 0x18 bytes: +0 overlay, +4 proc data
# (0 = nothing to run), +8 init(pl, mode, prev kind, prev param) -> param, +0xC return(pl, param) -> next kind, +0x10 special(pl,
# mode) (nullable: kind 4 picks it), +0x14 free(param)), reads the result back and loops (kind 4 = switch to the app pl+0xC).
#
# S (the event's argument `param_1`, the "S" of proclink.rs): +0 GAMESYS, +4 FIELDMAP, +8 parent event, +0xC first app index,
# +0x10 mode, +0x14 / +0x18 / +0x1C callbacks (record +0/+4/+8), +0x20 record argument, +0x24 result (written 0..3 by the event).
# PL (the event work, 0x74 bytes, GMEVENT work of 020122C0): +0 u16 id, +4 current app index, +8 previous app index, +0xC next app
# index (written by return callbacks), +0x10 kind (4 at the start), +0x14 the GMEVENT, +0x18 S, +0x1C param block of the running
# app, +0x20 CALLBACK RECORD (0x14 bytes {cbA, cbB, cbC, arg, owner} filled by 02160A28; the three callers 02160A4C / 02160A64 /
# 02160A7C call cbA / cbB / cbC with (record+0x10, record+0xC), no callback = 1), +0x34 0x18 bytes by 02164480, +0x4C 0x14 bytes
# by 0215E840, +0x60 u8, +0x62 u16, +0x68 u32 (the fields proclink.rs reads).
#
# Conventions (as F14 proclink): all blocks `nowork`; objects are `arg buf` arenas; GMEVENT works (020124E8) are `ptr:` answers
# into a trailing `arg buf` arena; MI_CpuFill8 02082BCC runs as real code (ALLOW). The table is the spec `global` 0x021712CC
# (19 entries drawn, so any app index 0..18 is covered; the real table has 12): its callbacks are FAKE addresses 0x02EE0011 (init),
# 0x02EE0021 (return), 0x02EE0031 (special), 0x02EE0041 (free) with `force` stubs at 0x02EE00x0. Closure stubs (a block lists
# the stubs of its callees and of all its in-area descendants; checked with every function treated as ported). Every out-of-area
# stub is `force`. The state machine 021607E0 is one function (a Thumb jump table of 14 cases, the labels 02160822 .. 0216097C
# are its cases; Ghidra's sub-functions there are not functions).
# Domain notes for 021607E0. (1) The real table (ROM) has `special` only in entry 4 (the report entry); the spec gives entries
# 0..2 none and 3..18 one, so both outcomes of the kind-4 test run. (2) State 3 calls `special` unconditionally (it is only entered
# from state 1 when `special` is set): the state is drawn from the event argument (`relm 0 0x8000000F`: 0..15 or a huge value, which
# is the default case) and the app index `pl+4` is either that value + 0..3 (so state 3 gets an entry >= 3) or 3..15; a state-3 call
# with a null `special` crashes the original and is not a valid input. (3) The callback record `pl+0x20` (read by the ported
# 02160A4C / 64 / 7C) holds no callback or the FAKE 0x02EE0051.
area proclink_event
set field

# FUN_0216075C(S, id): GMEVENT (proc 021607E1, work 0x74) for the app launcher; PL as above; returns the GMEVENT.
fn 0216075c ret=u32 inputs=200
  nowork
  arg buf 0x100
  arg u16 any
  arg buf 0x200
  stub 020122c0 force ret=u32 args=v,v,v,v
  stub 020124e8 force ret=ptr:arg2:0:1 args=v
  stub 02160a28 ret=none args=v,v,v,v,v,v
  stub 02164480 force ret=none args=out:0x18,v,v
  stub 0215e840 force ret=none args=v,out:0x14

# FUN_021607E0(event, seq, pl): the event proc, state machine over *seq (0..13; other values do nothing); returns 1 when done.
fn 021607e0 ret=u32 inputs=3000
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  arg buf 0x100
  global 021712cc 0x1C8
  fix arg1 0 u32 relm 0 0x8000000F const 0
  fix arg2 0x4 u32 alt relm 0 0xF range 0 3 || range 3 15
  fix arg2 0x8 u32 alt range 0 11 || range 0 15
  fix arg2 0xC u32 alt range 0 11 || range 0 15
  fix arg2 0x10 u32 alt range 0 5 || any
  fix arg2 0x18 u32 addrof arg3
  fix arg2 0x1C u32 alt const 0 || addrof arg3 0x80
  fix arg2 0x20 u32 alt const 0 || const 0x02EE0051 rep=3,4
  fix global0 0x04 u32 alt const 0 || any rep=19,0x18
  fix global0 0x08 u32 const 0x02EE0011 rep=19,0x18
  fix global0 0x0C u32 const 0x02EE0021 rep=19,0x18
  fix global0 0x10 u32 const 0 rep=3,0x18
  fix global0 0x58 u32 const 0x02EE0031 rep=16,0x18
  fix global0 0x14 u32 const 0x02EE0041 rep=19,0x18
  stub 02160a64 ret=pick+u32:0/1 args=v
  stub 02160a4c ret=pick+u32:0/1 args=v
  stub 02160a7c ret=pick+u32:0/1 args=v
  stub 02012374 force ret=none args=v,v
  stub 021be8d4 force ret=u32 args=v,v,v,v
  stub 020144dc force ret=u32 args=v,v
  stub 020145cc force ret=u32 args=v
  stub 0201210c force ret=u32 args=v
  stub 02188da0 force ret=none
  stub 02188db0 force ret=none args=v
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v
  stub 020120b4 force ret=none args=v,v,v,v
  stub 020120f0 force ret=pick+u32:0/1 args=v
  stub 020307b0 force ret=none args=v
  stub 02ee0010 force ret=u32 args=v,v,v,v
  stub 02ee0020 force ret=pick+u32:0/1/2/3/4 args=v,v
  stub 02ee0030 force ret=none args=v,v
  stub 02ee0040 force ret=none args=v
  stub 02ee0050 force ret=pick+u32:0/1 args=v,v

# FUN_02160A28(rec, owner, cbA, cbB, cbC, arg): fill the 0x14-byte callback record (MI_CpuFill8 then +0 cbA, +4 cbB, +8 cbC,
# +0xC arg, +0x10 owner; cbC and arg are stack words).
fn 02160a28 ret=none inputs=100
  nowork
  arg buf 0x40
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any

# FUN_02160A4C(rec): cbA(rec+0x10, rec+0xC), or 1 when rec+0 is null.
fn 02160a4c ret=u32 inputs=100
  nowork
  arg buf 0x20
  fix arg0 0x0 u32 alt const 0 || const 0x02EE0011
  stub 02ee0010 force ret=u32 args=v,v

# FUN_02160A64(rec): cbB(rec+0x10, rec+0xC), or 1 when rec+4 is null.
fn 02160a64 ret=u32 inputs=100
  nowork
  arg buf 0x20
  fix arg0 0x4 u32 alt const 0 || const 0x02EE0011
  stub 02ee0010 force ret=u32 args=v,v

# FUN_02160A7C(rec): cbC(rec+0x10, rec+0xC), or 1 when rec+8 is null.
fn 02160a7c ret=u32 inputs=100
  nowork
  arg buf 0x20
  fix arg0 0x8 u32 alt const 0 || const 0x02EE0011
  stub 02ee0010 force ret=u32 args=v,v
