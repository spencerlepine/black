# fndiff spec (S5b): save block 67 of the main ARM9 (0x0200E730..0x0200E874): two 0x20-byte Pokemon records.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage poke_records --per-function
# The Pokemon parameter getter FUN_02017E1C (its third argument, a buffer for parameter 116, is ignored: the stub
# writes nothing), the species-name message read FUN_02045F5C and the string copy FUN_0201F41C are stubbed; the
# message handle word 0x020A7370 is a random global. MI_CpuFill8 runs as real code.
area poke_records
set field

fn 0200e730 ret=u32 inputs=10
  nowork

fn 0200e734 ret=none inputs=50
  nowork
  arg buf 0x40

# FUN_0200E740(block, i): bit 0 of +0x1A
fn 0200e740 ret=u32 inputs=50
  nowork
  arg buf 0x40
  arg u32 range 0 1

# FUN_0200e74c(block, i): bits 1..7 of +0x1A, 65..100
fn 0200e74c ret=u32 inputs=300
  nowork
  arg buf 0x40
  arg u32 range 0 1

# 0x0200E76C(block, i): +0x1B != 0xFF
fn 0200e76c ret=bool inputs=100
  nowork
  arg buf 0x40
  arg u32 range 0 1
  fix arg0 0x1b u8 alt const 0xff || any
  fix arg0 0x3b u8 alt const 0xff || any

fn 0200e780 ret=u8 inputs=50
  nowork
  arg buf 0x40
  arg u32 range 0 1

# FUN_0200E788(block, a, b): message id
fn 0200e788 ret=u32 inputs=200
  nowork
  arg buf 0x40
  arg u32 alt oneof 1 || range 0 3 || any
  arg u32 alt oneof 21 || range 0 30 || any

fn 0200e7a4 ret=u32 inputs=50
  nowork
  arg buf 0x40
  arg u32 range 0 1

fn 0200e7ac ret=ofs:arg0 inputs=20
  nowork
  arg buf 0x40
  arg u32 range 0 1

fn 0200e7b4 ret=u32 inputs=50
  nowork
  arg buf 0x40
  arg u32 range 0 1

# FUN_0200E7BC(block, i, mon): fill record i
fn 0200e7bc ret=none inputs=400
  nowork
  arg buf 0x40
  arg u32 range 0 1
  arg u32 any
  global 020a7370 4
  stub 02017e1c ret=pick+u32:0/1/5/200 args=v,v,p
  stub 02045f5c ret=none args=v,v,out:22,v
  stub 0201f41c ret=none args=in:22,ptr,v
