# fndiff spec (S5): save block 27 (basic data: config + mystatus + playtime + save bookkeeping),
# main ARM9 0x02008700..0x02008828. Grammar: svflow_specs/README.md (generic mode).
# Run: tools/check-fn.sh black-storage save_info --per-function
#
# BLOCK27 = 0x68 bytes: +0 config (4), +4 mystatus (0x20), +0x24 playtime (8), +0x2C u8 flag,
# +0x30 u16[10] extra-area marks (0xC21E), +0x44 u32[9] extra-area words.
# The accessors take the save control (`arg u32 any`) and fetch the block with
# FUN_020071CC(sav, 27) (save_control.c, replaced in the port by ownership): stubbed to return
# the address of a `buf 0x68` argument that the function itself never reads as a register
# (arg1 when the function has one argument, arg2 otherwise). The port's test wiring calls the
# stub the same way, so the log matches. Indices are kept inside the arrays (marks 0..9, words 0..8).
# The init's callees (MIi_CpuFill32, config / mystatus / playtime init) run as real code.
area save_info
set field

fn 02008700 ret=u32 inputs=10
  nowork

# FUN_02008704(block): clear, config init, mystatus init, playtime init, +0x2C = 0
fn 02008704 ret=none
  nowork
  arg buf 0x68

# FUN_02008730(sav): block + 4
fn 02008730 ret=ofs:arg1
  nowork
  arg u32 any
  arg buf 0x68
  stub 020071cc ret=ptr:arg1:0:1 args=v,v

# FUN_0200873C(sav): block (tail call)
fn 0200873c ret=ofs:arg1
  nowork
  arg u32 any
  arg buf 0x68
  stub 020071cc ret=ptr:arg1:0:1 args=v,v

# FUN_02008748(sav): block + 0x24
fn 02008748 ret=ofs:arg1
  nowork
  arg u32 any
  arg buf 0x68
  stub 020071cc ret=ptr:arg1:0:1 args=v,v

fn 02008754 ret=u32
  nowork
  arg u32 any
  arg buf 0x68
  stub 020071cc ret=ptr:arg1:0:1 args=v,v

fn 02008764 ret=none
  nowork
  arg u32 any
  arg buf 0x68
  stub 020071cc ret=ptr:arg1:0:1 args=v,v

fn 02008774 ret=none
  nowork
  arg u32 any
  arg buf 0x68
  stub 020071cc ret=ptr:arg1:0:1 args=v,v

# FUN_02008784(sav, i): mark[i] == 0xC21E
fn 02008784 ret=bool inputs=400
  nowork
  arg u32 any
  arg u32 range 0 9
  arg buf 0x68
  fix arg2 0x30 u16 alt const 0xc21e || near 0xc21e 1 || any rep=10,2
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

fn 020087a8 ret=none
  nowork
  arg u32 any
  arg u32 range 0 9
  arg buf 0x68
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# unlabeled 0x020087C0(sav, i): mark[i] = 0
fn 020087c0 ret=none
  nowork
  arg u32 any
  arg u32 range 0 9
  arg buf 0x68
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# FUN_020087D4(sav, i): block + 0x44 + 4i
fn 020087d4 ret=ofs:arg2
  nowork
  arg u32 any
  arg u32 range 0 8
  arg buf 0x68
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# FUN_020087E8(sav, src): block+4 = 0x20 bytes at src + 0x20
fn 020087e8 ret=none
  nowork
  arg u32 any
  arg buf 0x40
  arg buf 0x68
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# FUN_02008808(sav, dst): dst + 0x20 = block+4 (0x20 bytes)
fn 02008808 ret=none
  nowork
  arg u32 any
  arg buf 0x40
  arg buf 0x68
  stub 020071cc ret=ptr:arg2:0:1 args=v,v
