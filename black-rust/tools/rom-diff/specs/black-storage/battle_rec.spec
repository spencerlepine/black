# fndiff spec (S5): battle_rec.c of the main ARM9: save block 48 (0x94 bytes; 0x0200B364..0x0200B3DC)
# and the pure battle-video header helpers (0x0200B784, 0x0200B98C, 0x0200BA38).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage battle_rec --per-function
#
# BLOCK48 = 0x94 bytes: +0 s32 id (-1 empty), +4 u32[32], +0x84 16 bytes. The save-control
# accessors fetch the block with FUN_020071CC(sav, 48) (replaced by ownership in the port):
# stubbed to the address of a `buf 0x94` argument the function never reads as a register.
# The mode table (rodata 0x0209DDCC, 41 u16) is read as real data; the port has the same values.
# HEADER = 0x44 bytes (species u16[12], level/flag u8[12] at +0x18, +0x24, +0x26 mode, +0x2A,
# +0x2B, +0x38 u64). MI_CpuFill8 / MIi_CpuCopy32 run as real code.
area battle_rec
set field

fn 0200b364 ret=u32 inputs=10
  nowork

# sub_0200B368(block): zero, id -1
fn 0200b368 ret=none inputs=50
  nowork
  arg buf 0x94

# FUN_0200B37C(block): id -1, words 0
fn 0200b37c ret=none inputs=50
  nowork
  arg buf 0x94

# FUN_0200B394(block, id): reset, id; returns block + 4
fn 0200b394 ret=ofs:arg0
  nowork
  arg buf 0x94
  arg u32 any

fn 0200b3a4 ret=ofs:arg0 inputs=20
  nowork
  arg buf 0x94

# unlabeled 0x0200B3A8(block): block + 0x84
fn 0200b3a8 ret=ofs:arg0 inputs=20
  nowork
  arg buf 0x94

fn 0200b3ac ret=u32
  nowork
  arg buf 0x94

# FUN_0200B3B0(src, sav): block = *src
fn 0200b3b0 ret=none inputs=50
  nowork
  arg buf 0x94
  arg u32 any
  arg buf 0x94
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# FUN_0200B3C8(sav, dst): *dst = block
fn 0200b3c8 ret=none inputs=50
  nowork
  arg u32 any
  arg buf 0x94
  arg buf 0x94
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# FUN_0200B784(mode, &count, &per): (4, 3) for table indices 16..19 and 22, else (2, 6)
fn 0200b784 ret=none inputs=600
  nowork
  arg u32 alt oneof 24 536 152 664 25 537 153 665 26 538 154 666 27 539 155 667 || oneof 28 540 156 668 0 1 4 || oneof 40 41 42 43 170 104 105 106 107 234 56 184 57 185 58 186 59 187 || any
  arg buf 4
  arg buf 4

# FUN_0200B98C(hdr, which, idx): header parameter (64-bit result; which 2 clamps to 9999)
fn 0200b98c ret=u64 inputs=1000
  nowork
  arg buf 0x44
  arg u32 alt range 0 8 || range 0 8 || any
  arg u32 range 0 11
  fix arg0 0 u16 alt near 0x289 2 || range 0 0x300 || any rep=12,2
  fix arg0 0x24 u16 alt near 9999 2 || any
  fix arg0 0x26 u16 alt oneof 24 536 0 4 187 59 1 || any

# FUN_0200BA38(level): level <= 100
fn 0200ba38 ret=bool inputs=200
  nowork
  arg u32 alt near 100 2 || any
