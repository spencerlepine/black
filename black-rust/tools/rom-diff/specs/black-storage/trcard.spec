# fndiff spec (S5b): the trainer card save block 33 of the main ARM9 (0x020089A8..0x02008AEC): 0x658 bytes
# (signature 0x600, +0x600 u16, +0x602 / +0x603 u8, +0x604 8 x 4-byte entries, +0x624 u64 seconds, +0x638 u16[16]).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage trcard --per-function
# The RTC (FUN_0203F49C date/time out, FUN_02089004 to seconds) is stubbed; MI_CpuFill8 runs as real code.
area trcard
set field

fn 020089a8 ret=u32 inputs=10
  nowork

# sub_020089B0(block): zero
fn 020089b0 ret=none inputs=50
  nowork
  arg buf 0x658

# FUN_020089CC(block): the signature address (the block)
fn 020089cc ret=ofs:arg0 inputs=10
  nowork
  arg buf 0x658

# 0x020089D0(block): sum of the 0x180 signature words != 0 (blank signatures half of the time)
fn 020089d0 ret=bool inputs=200
  nowork
  arg buf 0x658
  fix arg0 0 u32 alt const 0 || const 0 || any rep=384,4

# FUN_020089F4 / FUN_02008a00: +0x602 get / = 1
fn 020089f4 ret=u8 inputs=50
  nowork
  arg buf 0x658

fn 02008a00 ret=none inputs=50
  nowork
  arg buf 0x658

# FUN_02008A0C / 0x02008A14: +0x600 get / set
fn 02008a0c ret=u16 inputs=50
  nowork
  arg buf 0x658

fn 02008a14 ret=none inputs=50
  nowork
  arg buf 0x658
  arg u16 any

# 0x02008A1C / FUN_02008A28: +0x603 set / get
fn 02008a1c ret=none inputs=50
  nowork
  arg buf 0x658
  arg u8 any

fn 02008a28 ret=u8 inputs=50
  nowork
  arg buf 0x658

# FUN_02008A34(block, lo, hi): +0x624 = u64
fn 02008a34 ret=none inputs=50
  nowork
  arg buf 0x658
  arg u32 any
  arg u32 any

# 0x02008A44(block): +0x624 as u64
fn 02008a44 ret=u64 inputs=50
  nowork
  arg buf 0x658

# 0x02008A54(block, i): entry i as a word
fn 02008a54 ret=u32 inputs=100
  nowork
  arg buf 0x658
  arg u32 range 0 7

# FUN_02008a7c(block, i, a, b, c): entry bytes, halfword 0xF0, time stamp from the RTC
fn 02008a7c ret=none inputs=200
  nowork
  arg buf 0x658
  arg u32 range 0 7
  arg u8 any
  arg u8 any
  arg u8 any
  stub 0203f49c ret=none args=out:16,out:12
  stub 02089004 ret=u64 args=in:16,in:12

# 0x02008ACC / 0x02008ADC(block, i[, v]): halfword i get / set
fn 02008acc ret=u16 inputs=100
  nowork
  arg buf 0x658
  arg u32 range 0 15

fn 02008adc ret=none inputs=100
  nowork
  arg buf 0x658
  arg u32 range 0 15
  arg u16 any
