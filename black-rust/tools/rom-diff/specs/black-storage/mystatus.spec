# fndiff spec (S5): mystatus.c of the main ARM9 (0x02008468..0x0200857C), the 0x20-byte player
# status. Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage mystatus --per-function
#
# MYSTATUS = 0x20 bytes: +0 name u16[8], +0x10 u32 ID, +0x14 u32 (set once), +0x18 / +0x19 u8,
# +0x1A language, +0x1B version, +0x1C u8, +0x1D gender, +0x1E unused. Every function takes the
# block as `arg buf 0x20`; setters take a full-width random value (the originals store its low byte).
# MI_CpuCopy8 / MI_CpuFill8 run as real code.
area mystatus
set field

fn 02008468 ret=u32 inputs=10
  nowork

# FUN_02008494(src, dst): MI_CpuCopy8(src, dst, 0x20)
fn 02008494 ret=none
  nowork
  arg buf 0x20
  arg buf 0x20

# FUN_020084A0(status): clear, +0x1A = 2, version 0x15
fn 020084a0 ret=none
  nowork
  arg buf 0x20

# FUN_020084BC(status): all 8 name halfwords 0 -> 1
fn 020084bc ret=bool inputs=400
  nowork
  arg buf 0x20
  fix arg0 0 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || any rep=8,2

# FUN_020084F4(status): returns its argument (the name is at +0)
fn 020084f4 ret=ofs:arg0
  nowork
  arg buf 0x20

fn 0200852c ret=none
  nowork
  arg buf 0x20
  arg u32 any

fn 02008530 ret=u32
  nowork
  arg buf 0x20

fn 02008534 ret=u32
  nowork
  arg buf 0x20

fn 0200853c ret=u32
  nowork
  arg buf 0x20

# FUN_02008540(status, v): +0x14 = v only if it is 0
fn 02008540 ret=none inputs=300
  nowork
  arg buf 0x20
  arg u32 any
  fix arg0 0x14 u32 alt const 0 || any

fn 0200854c ret=none
  nowork
  arg buf 0x20
  arg u32 any

fn 02008550 ret=u32
  nowork
  arg buf 0x20

fn 02008554 ret=u32
  nowork
  arg buf 0x20

fn 02008558 ret=none
  nowork
  arg buf 0x20
  arg u32 any

fn 0200855c ret=u32
  nowork
  arg buf 0x20

fn 02008560 ret=none
  nowork
  arg buf 0x20
  arg u32 any

fn 02008564 ret=u32
  nowork
  arg buf 0x20

fn 02008568 ret=none
  nowork
  arg buf 0x20
  arg u32 any

fn 0200856c ret=u32
  nowork
  arg buf 0x20

fn 02008570 ret=u32
  nowork
  arg buf 0x20

fn 02008574 ret=none
  nowork
  arg buf 0x20
  arg u32 any
  arg u32 any
