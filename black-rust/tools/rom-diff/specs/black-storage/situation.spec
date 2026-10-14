# fndiff spec (S5b): the situation save block 28 of the main ARM9 (0x02008828..0x020089A8): 0x9C bytes
# (location records, field-status mirror, counters, the saved player position at +0x80).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage situation --per-function
# The block fetch FUN_020071CC(sav, 28) is stubbed with a pointer into the last buffer (the port owns the block);
# the field-status byte accessors FUN_02014468 / FUN_02014464 are stubbed. MI_CpuFill8 runs as real code.
area situation
set field

fn 02008828 ret=u32 inputs=10
  nowork

# sub_0200882C(block): zero, +0x76 = 0x1AB
fn 0200882c ret=none inputs=50
  nowork
  arg buf 0x9c

# FUN_02008844 / 48 / 4C / 50(block): record addresses
fn 02008844 ret=ofs:arg0 inputs=10
  nowork
  arg buf 0x9c

fn 02008848 ret=ofs:arg0 inputs=10
  nowork
  arg buf 0x9c

fn 0200884c ret=ofs:arg0 inputs=10
  nowork
  arg buf 0x9c

fn 02008850 ret=ofs:arg0 inputs=10
  nowork
  arg buf 0x9c

# FUN_02008860(sav, src): save the player position from the game data record
fn 02008860 ret=none inputs=100
  nowork
  arg u32 any
  arg buf 0x44
  arg buf 0x9c
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# FUN_020088B8(sav, dst): load it back
fn 020088b8 ret=none inputs=100
  nowork
  arg u32 any
  arg buf 0x44
  arg buf 0x9c
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# FUN_0200890C(sav, dst): copy the 0x1C-byte position record out
fn 0200890c ret=none inputs=50
  nowork
  arg u32 any
  arg buf 0x1c
  arg buf 0x9c
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# FUN_0200892C(block, fs): +0x72 = field status +6
fn 0200892c ret=none inputs=50
  nowork
  arg buf 0x9c
  arg u32 any
  stub 02014468 ret=u8 args=v

# FUN_0200893C(block, fs): field status +6 = +0x72 (tail call)
fn 0200893c ret=none inputs=50
  nowork
  arg buf 0x9c
  arg u32 any
  stub 02014464 ret=none args=v,v

# 0x0200894C(block, v): +0x73 = v
fn 0200894c ret=none inputs=50
  nowork
  arg buf 0x9c
  arg u8 any

# FUN_02008954(block, out): *out = +0x73
fn 02008954 ret=none inputs=50
  nowork
  arg buf 0x9c
  arg buf 0x4

# 0x0200895C(block, v): +0x74 = v
fn 0200895c ret=none inputs=50
  nowork
  arg buf 0x9c
  arg u8 any

# FUN_02008964(block, out): *out = +0x74
fn 02008964 ret=none inputs=50
  nowork
  arg buf 0x9c
  arg buf 0x4

# 0x0200896C(block, v): +0x78 = v
fn 0200896c ret=none inputs=50
  nowork
  arg buf 0x9c
  arg u32 any

# FUN_02008970(block, out): *out = +0x78
fn 02008970 ret=none inputs=50
  nowork
  arg buf 0x9c
  arg buf 0x4

# FUN_02008978 / 80: +0x7C get / set
fn 02008978 ret=u16 inputs=50
  nowork
  arg buf 0x9c

fn 02008980 ret=none inputs=50
  nowork
  arg buf 0x9c
  arg u16 any

# FUN_02008988 / 90: +0x7E set / get
fn 02008988 ret=none inputs=50
  nowork
  arg buf 0x9c
  arg u16 any

fn 02008990 ret=u16 inputs=50
  nowork
  arg buf 0x9c

# FUN_02008998 / sub_020089A0: +0x76 get / set
fn 02008998 ret=u16 inputs=50
  nowork
  arg buf 0x9c

fn 020089a0 ret=none inputs=50
  nowork
  arg buf 0x9c
  arg u16 any
