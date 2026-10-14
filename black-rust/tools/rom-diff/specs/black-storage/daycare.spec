# fndiff spec (S5b): the daycare save block 50 of the main ARM9 (0x0200BC10..0x0200BDBC): 0x1CC bytes,
# two 0xE4-byte slots (+0 u32 occupied, +4 0xDC-byte party Pokemon, +0xE0 u32 exp), +0x1C8 u32 egg flag.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage daycare --per-function
# The occupied words get 1 half of the time so both branches run. MI_CpuFill8 runs as real code.
area daycare
set field

# FUN_0200BC10(block): egg flag = 1 (only written when it is not 1)
fn 0200bc10 ret=none inputs=100
  nowork
  arg buf 0x1cc
  fix arg0 0x1c8 u32 alt oneof 0 1 || any

# FUN_0200bc20(block): egg flag = 0
fn 0200bc20 ret=none inputs=50
  nowork
  arg buf 0x1cc

# FUN_0200BC2C(block, i): address of the Pokemon of slot i (valid and occupied) or of slot 0
fn 0200bc2c ret=ofs:arg0 inputs=300
  nowork
  arg buf 0x1cc
  arg u32 alt range 0 3 || oneof 0xffffffff 0x80000000 || any
  fix arg0 0 u32 alt oneof 1 || oneof 0 2 || any
  fix arg0 0xe4 u32 alt oneof 1 || oneof 0 2 || any

# FUN_0200bc4c(block, i, mon): store the Pokemon in a free valid slot
fn 0200bc4c ret=none inputs=400
  nowork
  arg buf 0x1cc
  arg u32 alt range 0 3 || oneof 0xffffffff 0x80000000 || any
  arg buf 0xdc
  fix arg0 0 u32 alt oneof 1 || oneof 0 2 || any
  fix arg0 0xe4 u32 alt oneof 1 || oneof 0 2 || any

# FUN_0200bc88(block, i): free an occupied valid slot
fn 0200bc88 ret=none inputs=300
  nowork
  arg buf 0x1cc
  arg u32 alt range 0 3 || oneof 0xffffffff 0x80000000 || any
  fix arg0 0 u32 alt oneof 1 || oneof 0 2 || any
  fix arg0 0xe4 u32 alt oneof 1 || oneof 0 2 || any

# sub_0200bca0(block): egg flag
fn 0200bca0 ret=u32 inputs=50
  nowork
  arg buf 0x1cc

# FUN_0200BCA8(block, i): occupied word of slot i (0 for i >= 2 unsigned)
fn 0200bca8 ret=u32 inputs=200
  nowork
  arg buf 0x1cc
  arg u32 alt range 0 3 || oneof 0xffffffff 0x80000000 || any

# FUN_0200BCB8(block): number of slots with occupied == 1
fn 0200bcb8 ret=u32 inputs=200
  nowork
  arg buf 0x1cc
  fix arg0 0 u32 alt oneof 1 || oneof 0 2 || any
  fix arg0 0xe4 u32 alt oneof 1 || oneof 0 2 || any

# unlabeled 0x0200BCD8(block, i): exp of slot i (0 for i >= 2 unsigned)
fn 0200bcd8 ret=u32 inputs=200
  nowork
  arg buf 0x1cc
  arg u32 alt range 0 3 || oneof 0xffffffff 0x80000000 || any

# FUN_0200BCEC(block, i, j): swap two valid slots
fn 0200bcec ret=none inputs=300
  nowork
  arg buf 0x1cc
  arg u32 alt range 0 3 || oneof 0xffffffff || any
  arg u32 alt range 0 3 || oneof 0xffffffff || any

# FUN_0200BD48(block, n): exp += n on occupied slots, saturating
fn 0200bd48 ret=none inputs=400
  nowork
  arg buf 0x1cc
  arg u32 alt range 0 100 || any
  fix arg0 0 u32 alt oneof 1 || oneof 0 2 || any
  fix arg0 0xe4 u32 alt oneof 1 || oneof 0 2 || any
  fix arg0 0xe0 u32 alt near 0xffffffff 200 || any
  fix arg0 0x1c4 u32 alt near 0xffffffff 200 || any

# sub_0200bd98(): size
fn 0200bd98 ret=u32 inputs=10
  nowork

# sub_0200bda0(block): zero
fn 0200bda0 ret=none inputs=50
  nowork
  arg buf 0x1cc
