# fndiff spec (S5): bsubway_savedata.c of the main ARM9 (0x0200D858..0x0200E024): Battle Subway
# play data (block 57, 0x3C), score data (block 58, 0x1AC) and the 11 x 3 u16 record counters
# (0x42 bytes at +0x5A of block 60). Block 59 (online rooms, D9) is not in the spec.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage bsubway --per-function
#
# PLAY: +0 u16 bits, +2 u8, +3 u8, +4 u16, +6 u16, +8 [4], +0xC [28], +0x28 [20]; field ids 0..11.
# SCORE: +0 u16 BP, +2 u8, +3 u8, +4 u32 flags, +8 / +0x1A / +0x2C u16[9], +0x3E u16, +0x44 / +0xF8 [0xB4].
# Mode indices are kept in 0..8 (the arrays hold 9), counter indices in 0..10 x 0..2, flag bits
# >= 0 (a negative bit makes the original's mask loop run ~2^32 times). _s32_div_f, MI_CpuCopy8 and
# MI_CpuFill8 run as real code.
area bsubway
set field

fn 0200d858 ret=u32 inputs=10
  nowork

fn 0200d85c ret=none inputs=50
  nowork
  arg buf 0x3c

fn 0200d868 ret=none
  nowork
  arg buf 0x3c
  arg u32 any

fn 0200d87c ret=u32
  nowork
  arg buf 0x3c

# FUN_0200D884(play, id, out)
fn 0200d884 ret=u32 inputs=800
  nowork
  arg buf 0x3c
  arg u32 alt range 0 12 || range 0 12 || any
  arg buf 0x20

# FUN_0200D914(play, id, src)
fn 0200d914 ret=none inputs=800
  nowork
  arg buf 0x3c
  arg u32 alt range 0 12 || range 0 12 || any
  arg buf 0x20

# FUN_0200D9E8(play, a, b, c): capped adds
fn 0200d9e8 ret=none inputs=1000
  nowork
  arg buf 0x3c
  arg u32 alt range 0 3 || oneof 0xffffffff 0xfffffff0 || any
  arg u32 alt range 0 3 || oneof 0xffffffff 0xfffffff0 || any
  arg u32 alt range 0 3 || oneof 0xffffffff 0xfffffff0 || any
  fix arg0 3 u8 alt near 254 2 || any
  fix arg0 4 u16 alt near 0xfffe 2 || any
  fix arg0 6 u16 alt near 0xfffe 2 || any

fn 0200da14 ret=none inputs=50
  nowork
  arg buf 0x3c

fn 0200da28 ret=none inputs=300
  nowork
  arg buf 0x3c
  fix arg0 2 u8 alt near 254 1 || any

fn 0200da54 ret=u32
  nowork
  arg buf 0x3c

fn 0200da64 ret=u32 inputs=10
  nowork

fn 0200da6c ret=none inputs=50
  nowork
  arg buf 0x1ac

# FUN_0200DA80(score, n): BP += n, cap 9999 (signed)
fn 0200da80 ret=none inputs=600
  nowork
  arg buf 0x1ac
  arg u32 alt range 0 20 || near 9999 3 || oneof 0xffffffff 0x80000000 || any
  fix arg0 0 u16 alt near 9999 3 || range 0 9999 || any

# FUN_0200DA98(score, n): BP -= n, 0 if the s16 difference is negative
fn 0200da98 ret=none inputs=600
  nowork
  arg buf 0x1ac
  arg u32 alt range 0 20 || near 9999 3 || oneof 0xffffffff 0x8000 || any
  fix arg0 0 u16 alt near 9999 3 || range 0 30 || any

fn 0200dabc ret=u32
  nowork
  arg buf 0x1ac

# unlabeled 0x0200DAC0: bx lr (identity)
fn 0200dac0 ret=u32
  nowork
  arg u32 any

fn 0200dac4 ret=u32
  nowork
  arg buf 0x1ac
  arg u32 range 0 8

fn 0200dad8 ret=u32
  nowork
  arg buf 0x1ac
  arg u32 range 0 8

fn 0200daec ret=none inputs=400
  nowork
  arg buf 0x1ac
  arg u32 range 0 8
  arg u32 alt range 0 0x20 || any
  fix arg0 0x1a u16 alt range 0 0x20 || any rep=9,2

fn 0200db08 ret=none
  nowork
  arg buf 0x1ac
  arg u32 range 0 8
  arg u32 any

fn 0200db1c ret=none
  nowork
  arg buf 0x1ac
  arg u32 range 0 8

fn 0200db30 ret=none
  nowork
  arg buf 0x1ac
  arg u32 range 0 8

fn 0200db44 ret=bool inputs=300
  nowork
  arg buf 0x1ac
  arg u32 range 0 8
  fix arg0 0x2c u16 alt const 0 || any rep=9,2

fn 0200db60 ret=none inputs=300
  nowork
  arg buf 0x1ac
  arg u32 range 0 8
  fix arg0 0x2c u16 alt near 0xfffe 1 || any rep=9,2

fn 0200db80 ret=u32 inputs=400
  nowork
  arg buf 0x1ac
  arg u32 range 0 8
  fix arg0 0x2c u16 alt range 0 2 || near 0x8000 2 || any rep=9,2

# FUN_0200DBA0(score, bit, op)
fn 0200dba0 ret=u32 inputs=600
  nowork
  arg buf 0x1ac
  arg u32 alt range 0 12 || range 0 12 || range 0 0x7fff
  arg u32 alt range 0 3 || any

fn 0200dbf0 ret=none inputs=300
  nowork
  arg buf 0x1ac
  fix arg0 3 u8 alt range 8 11 || any

fn 0200dbfc ret=none inputs=300
  nowork
  arg buf 0x1ac
  fix arg0 3 u8 alt range 0 2 || any

fn 0200dc08 ret=u32
  nowork
  arg buf 0x1ac

# FUN_0200DC0C(score, kind): streak update for kinds 2 / 3
fn 0200dc0c ret=u32 inputs=600
  nowork
  arg buf 0x1ac
  arg u32 alt oneof 2 3 || range 0 5 || any
  fix arg0 2 u8 alt near 254 1 || any

fn 0200dc50 ret=none inputs=50
  nowork
  arg buf 0x1ac
  arg u32 alt range 0 1 || any
  arg buf 0xb4

fn 0200dc74 ret=none inputs=50
  nowork
  arg buf 0x1ac
  arg u32 alt range 0 1 || any
  arg buf 0xb4

# FUN_0200DC94(score, play): points
fn 0200dc94 ret=none inputs=1000
  nowork
  arg buf 0x1ac
  arg buf 0x3c
  fix arg1 2 u8 alt range 0 8 || any
  fix arg1 3 u8 alt range 0 50 || any
  fix arg1 4 u16 alt range 0 100 || any
  fix arg1 6 u16 alt near 970 2 || range 0 1000 || any

fn 0200dcf8 ret=u32
  nowork
  arg buf 0x1ac

fn 0200dcfc ret=u32
  nowork
  arg buf 0x1ac

# unlabeled 0x0200DFCC(c): MI_CpuFill8(c, 0, 0x42)
fn 0200dfcc ret=none inputs=50
  nowork
  arg buf 0x44

fn 0200dfd8 ret=u32
  nowork
  arg buf 0x44
  arg u32 range 0 10
  arg u32 range 0 2

# FUN_0200DFE4(c, i, j, v): clamp 0..0xFFFF (signed)
fn 0200dfe4 ret=none inputs=400
  nowork
  arg buf 0x44
  arg u32 range 0 10
  arg u32 range 0 2
  arg u32 alt near 0xffff 2 || oneof 0xffffffff 0x80000000 || any

fn 0200e008 ret=none inputs=300
  nowork
  arg buf 0x44
  arg u32 range 0 10
  arg u32 range 0 2
  fix arg0 0 u16 alt near 0xffff 1 || any rep=33,2
