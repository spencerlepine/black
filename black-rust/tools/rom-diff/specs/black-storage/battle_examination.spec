# fndiff spec (S5): battle_examination.c of the main ARM9 (0x0200E588..0x0200E6B0): the 0x5C8-byte
# battle-examination data (5 x 0x120 entries, +0x5C2 u16 state, +0x5C4 u16 flags, +0x5C6 u16 CRC).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage battle_examination --per-function
#
# FUN_0203F1FC (CRC-16 of (data, len)) is stubbed with an `in:` argument (answers include the
# planted CRC values so both branches run). MI_CpuFill8 runs as real code.
area battle_examination
set field

fn 0200e588 ret=u32 inputs=10
  nowork

fn 0200e590 ret=none inputs=50
  nowork
  arg buf 0x5c8

# FUN_0200E5A0(d): CRC(d, 0x5C6) == d[0x5C6] && d[0x5C2] == 0xD68
fn 0200e5a0 ret=bool inputs=400
  nowork
  arg buf 0x5c8
  fix arg0 0x5c2 u16 alt const 0xd68 || any
  fix arg0 0x5c6 u16 alt oneof 0 1 0xffff || any
  stub 0203f1fc ret=pick+u16:0/1/0xffff args=in:0x5c6,v

fn 0200e5cc ret=bool
  nowork
  arg buf 0x5c8

# unlabeled 0x0200E5E4(d): d[0x5C4] & 0x7F
fn 0200e5e4 ret=u32
  nowork
  arg buf 0x5c8

# unlabeled 0x0200E5F8(d, i): d + 0x120 i for i < 5, else d
fn 0200e5f8 ret=ofs:arg0 inputs=200
  nowork
  arg buf 0x5c8
  arg u32 alt range 0 6 || any

# FUN_0200E684(d): if the CRC matches: state 0x7DA, CRC recomputed
fn 0200e684 ret=none inputs=400
  nowork
  arg buf 0x5c8
  fix arg0 0x5c6 u16 alt oneof 0 1 0xffff || any
  stub 0203f1fc ret=pick+u16:0/1/0xffff args=in:0x5c6,v
