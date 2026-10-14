# fndiff spec (S5b): the DS-identity logs of the main ARM9 (0x0200E8E8..0x0200EA08): save block 65 (0x5C bytes)
# and save block 60 (0x9C bytes) both start with a 0x5A-byte log (MAC, four u8 counters, 10 x 8-byte entries).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage mac_log --per-function
# OS_GetMacAddress (02087C6C) and the compare 0203EFC8 are stubbed; MI_CpuFill8 / MI_CpuCopy8 / _u32_div_f and the
# counter clear 0x0200DFCC (black-storage bsubway) run as real code.
area mac_log
set field

fn 0200e8e8 ret=u32 inputs=10
  nowork

# sub_0200E8EC(block): zero
fn 0200e8ec ret=none inputs=50
  nowork
  arg buf 0x5c

# FUN_0200e904(log): zero 0x5A, MAC at +0
fn 0200e904 ret=none inputs=50
  nowork
  arg buf 0x5c
  stub 02087c6c ret=none args=out:6

# FUN_0200E918(log, i): counter i
fn 0200e918 ret=u8 inputs=50
  nowork
  arg buf 0x5c
  arg u32 range 0 3

# FUN_0200e920(log, i, n): counter i += n, clamped 0..255
fn 0200e920 ret=none inputs=400
  nowork
  arg buf 0x5c
  arg u32 range 0 3
  arg u32 alt range 0 300 || near 0xffffff00 300 || any

# FUN_0200E938(log, i, dst): MAC of entry i % 10
fn 0200e938 ret=none inputs=100
  nowork
  arg buf 0x5c
  arg u32 any
  arg buf 0x8

# FUN_0200E958(log, i, src): MAC of entry i % 10 = src
fn 0200e958 ret=none inputs=100
  nowork
  arg buf 0x5c
  arg u32 any
  arg buf 0x8

# FUN_0200E978(log, i, j, v): byte j after the MAC of entry i % 10
fn 0200e978 ret=none inputs=100
  nowork
  arg buf 0x5c
  arg u32 any
  arg u32 range 0 1
  arg u8 any

# FUN_0200E994(log, mac): mac among the first min(counter 0, 10) entries
fn 0200e994 ret=bool inputs=400
  nowork
  arg buf 0x5c
  arg buf 0x8
  fix arg0 6 u8 alt range 0 12 || any
  stub 0203efc8 ret=pick+u32:0/1/1/1 args=in:6,in:6,v

fn 0200e9dc ret=u32 inputs=10
  nowork

# sub_0200E9E0(block): log init, counters clear
fn 0200e9e0 ret=none inputs=50
  nowork
  arg buf 0x9c
  stub 02087c6c ret=none args=out:6

# 0x0200EA00 / 0x0200EA04(block): block + 0x5A
fn 0200ea00 ret=ofs:arg0 inputs=10
  nowork
  arg buf 0x9c

fn 0200ea04 ret=ofs:arg0 inputs=10
  nowork
  arg buf 0x9c
