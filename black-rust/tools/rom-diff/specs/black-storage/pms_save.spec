# fndiff spec (S5b): the easy-chat save blocks of the main ARM9: block 39 (0x02024910..0x020249B0; 0xA8 bytes:
# u32 unlock bits, u32, 20 x 8-byte sentences) and block 62 (0x0200E6B0..0x0200E730; 5 x 8-byte sentences).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage pms_save --per-function
# The pms_word.c setters FUN_020242A4 / FUN_02024344 / FUN_020242D8 and MI_CpuCopy8 run as real code (the port
# writes the same fields); the unlock table rodata 0x0209F784 is read as real data.
area pms_save
set field

fn 0200e6b0 ret=u32 inputs=10
  nowork

# sub_0200E6B4(block): default sentences
fn 0200e6b4 ret=none inputs=50
  nowork
  arg buf 0x28

# FUN_0200E6FC(block, i, dst): sentence i out
fn 0200e6fc ret=none inputs=100
  nowork
  arg buf 0x28
  arg u32 range 0 4
  arg buf 0x8

# FUN_0200E714(block, i, src): sentence i in
fn 0200e714 ret=none inputs=100
  nowork
  arg buf 0x28
  arg u32 range 0 4
  arg buf 0x8

# 0x02024910(block, i): sentence address
fn 02024910 ret=ofs:arg0 inputs=50
  nowork
  arg buf 0xa8
  arg u32 range 0 19

# FUN_02024918(block, i, src): sentence i = src (i in 0..20, signed)
fn 02024918 ret=none inputs=200
  nowork
  arg buf 0xa8
  arg u32 alt range 0 22 || oneof 0xffffffff 0x80000000
  arg buf 0x8

fn 02024934 ret=u32 inputs=10
  nowork

# sub_02024938(block): clear, unlock the group of kind 2
fn 02024938 ret=none inputs=50
  nowork
  arg buf 0xa8

# 0x02024994(block, bit): unlock bit (register shift: low byte counts)
fn 02024994 ret=bool inputs=300
  nowork
  arg buf 0xa8
  arg u32 alt range 0 40 || oneof 0x100 0x101 0x11f || any

# FUN_020249A0(block, bit): set unlock bit
fn 020249a0 ret=none inputs=300
  nowork
  arg buf 0xa8
  arg u32 alt range 0 40 || oneof 0x100 0x101 0x11f || any
