# fndiff spec: poke_tool.c record cipher and field get/put (P1a/P1b), main ARM9 code.
# Grammar: svflow_specs/README.md (generic mode + the P1 additions `sum16` and `crypt`).
# Run: tools/check-fn.sh black-pokemon poke_tool --per-function
#
# Record layout used by the fixes (see crates/black-pokemon/src/poke_tool.rs): PID @0, flags @4
# (bit0 party-plain, bit1 box-plain, bit2 bad egg), checksum @6, four shuffled 0x20-byte blocks @8..0x88
# (the shuffle row is PID bits 13-17, so any of the 4 blocks can be A: the "edge value" fixes below
# are planted at all four candidate offsets), party extension @0x88..0xDC.
#
# Plaintext prep (EDGE): in 2/3 of the inputs the PID has shuffle row 0 (blocks stored in order A B C D),
# and then species 0/Nidoran/Arceus/any, held item around the 0x272 cap, ability 121, EVs small enough to
# hit the 510 cap (block A), move slots 0/nonzero and PP ups 0..3 (block B) are planted at their real
# offsets; in the other inputs the shuffle row is random and the data stays random.
# VALID (wrappers and enter/exit only): 50% of the inputs get a correct checksum and the blocks are
# really encrypted (so the "checksum ok" branch of FUN_02018D34 and the plain path of get/put run),
# unless the box-plain flag is set (then the blocks stay plain).
area poke_tool
set field

# FUN_02019A50(void *data, uint size, uint seed): XOR halfword i (i < size/2) with the LCG32 key
# stream started at `seed` (FUN_02019A7C runs as real code); odd size ignores the last byte.
fn 02019a50 ret=none
  nowork
  region data 0x100
  arg region data
  arg u32 alt range 0 0x100 || oneof 0 1 2 3 4 0x54 0x80 0xFF 0x100
  arg u32 alt any || range 0 0xFFFF

# FUN_02019C38(mon, pid, blk): mon + 8 + shuffle table byte [((pid & 0x3E000) >> 13) * 4 + blk]
# (the pointer comes back as an offset into the argument buffer).
fn 02019c38 ret=ofs:arg0
  nowork
  arg buf 0x10
  arg u32 any
  arg u32 range 0 3

# FUN_02019BBC(item): Arceus plate item (0x12A..0x139) -> type; anything else 0.
fn 02019bbc ret=u32 inputs=400
  nowork
  arg u32 alt near 0x131 0x10 || near 0x12a 4 || near 0x139 4 || any

fn 02017dbc ret=bool
  nowork
  region mon 0xDC
  arg region mon

fn 02017de4 ret=u32
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt const 1 || const 1 || const 0 || any

fn 02018d34 ret=u32
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2

fn 02018d8c ret=none
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1

fn 02018d10 ret=u32
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2

fn 02018d6c ret=none
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1

# FUN_02019CE0(src, dst): copy 0xDC bytes.
fn 02019ce0 ret=none inputs=50
  nowork
  region src 0xDC
  region dst 0xDC
  arg region src
  arg region dst

fn 02019abc ret=u32 inputs=300
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  stub 02045f5c ret=none args=p,v,v,v
  stub 0201f448 ret=bool args=v,v

# --- get / put over every field ID (direct, blocks already plain) ---

fn 02018e34 ret=u32 inputs=2400
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 0xb3 || range 0 0xb3 || range 0 0xb3 || oneof 5 6 0x42 0x43 0x44 0x45 0x4c 0x6e 0x73 0x74 0xaa 0xab 0xad 0xae 0xaf 0xd 0xe 0xf 0x10 0x11 0x12 0x9e 0xa7 || oneof 0xb4 0xb5 0xff 0x100 0x80000000 0xFFFFFFFF
  arg u32 alt any || range 0 300 || near 0x272 3 || oneof 0 1 || mask 0x1f
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v

fn 02019388 ret=none inputs=2400
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 0xb3 || range 0 0xb3 || range 0 0xb3 || oneof 5 6 0x42 0x43 0x44 0x45 0x4c 0x6e 0x73 0x74 0xaa 0xab 0xad 0xae 0xaf 0xd 0xe 0xf 0x10 0x11 0x12 0x9e 0xa7 || oneof 0xb4 0xb5 0xff 0x100 0x80000000 0xFFFFFFFF
  arg u32 alt any || range 0 300 || near 0x272 3 || oneof 0 1 || mask 0x1f
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  stub 02017f50 ret=u32 args=v,v,v
  stub 02045a38 ret=none args=v,v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 0201f448 ret=bool args=v,v

# --- party wrappers: ids 0x9d..0xa8 live in the party extension ---

fn 02018db4 ret=u32 inputs=2400
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 0xb3 || range 0 0xb3 || range 0 0xb3 || oneof 5 6 0x42 0x43 0x44 0x45 0x4c 0x6e 0x73 0x74 0xaa 0xab 0xad 0xae 0xaf 0xd 0xe 0xf 0x10 0x11 0x12 0x9e 0xa7 || oneof 0xb4 0xb5 0xff 0x100 0x80000000 0xFFFFFFFF
  arg u32 alt any || range 0 300 || near 0x272 3 || oneof 0 1 || mask 0x1f
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02008f3c ret=none args=v,v

fn 02019308 ret=none inputs=2400
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 0xb3 || range 0 0xb3 || range 0 0xb3 || oneof 5 6 0x42 0x43 0x44 0x45 0x4c 0x6e 0x73 0x74 0xaa 0xab 0xad 0xae 0xaf 0xd 0xe 0xf 0x10 0x11 0x12 0x9e 0xa7 || oneof 0xb4 0xb5 0xff 0x100 0x80000000 0xFFFFFFFF
  arg u32 alt any || range 0 300 || near 0x272 3 || oneof 0 1 || mask 0x1f
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  stub 02017f50 ret=u32 args=v,v,v
  stub 02045a38 ret=none args=v,v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 0201f448 ret=bool args=v,v
  stub 02008f3c ret=none args=v,v

# --- public accessors: enter, access, exit (valid and invalid checksums) ---

fn 02017e88 ret=u32 inputs=2400
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 0xb3 || range 0 0xb3 || range 0 0xb3 || oneof 5 6 0x42 0x43 0x44 0x45 0x4c 0x6e 0x73 0x74 0xaa 0xab 0xad 0xae 0xaf 0xd 0xe 0xf 0x10 0x11 0x12 0x9e 0xa7 || oneof 0xb4 0xb5 0xff 0x100 0x80000000 0xFFFFFFFF
  arg u32 alt any || range 0 300 || near 0x272 3 || oneof 0 1 || mask 0x1f
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v

fn 02017e64 ret=none inputs=2400
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 0xb3 || range 0 0xb3 || range 0 0xb3 || oneof 5 6 0x42 0x43 0x44 0x45 0x4c 0x6e 0x73 0x74 0xaa 0xab 0xad 0xae 0xaf 0xd 0xe 0xf 0x10 0x11 0x12 0x9e 0xa7 || oneof 0xb4 0xb5 0xff 0x100 0x80000000 0xFFFFFFFF
  arg u32 alt any || range 0 300 || near 0x272 3 || oneof 0 1 || mask 0x1f
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  stub 02017f50 ret=u32 args=v,v,v
  stub 02045a38 ret=none args=v,v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 0201f448 ret=bool args=v,v

fn 02017e1c ret=u32 inputs=2400
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 0xb3 || range 0 0xb3 || range 0 0xb3 || oneof 5 6 0x42 0x43 0x44 0x45 0x4c 0x6e 0x73 0x74 0xaa 0xab 0xad 0xae 0xaf 0xd 0xe 0xf 0x10 0x11 0x12 0x9e 0xa7 || oneof 0xb4 0xb5 0xff 0x100 0x80000000 0xFFFFFFFF
  arg u32 alt any || range 0 300 || near 0x272 3 || oneof 0 1 || mask 0x1f
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02008f3c ret=none args=v,v

fn 02017e40 ret=none inputs=2400
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 0xb3 || range 0 0xb3 || range 0 0xb3 || oneof 5 6 0x42 0x43 0x44 0x45 0x4c 0x6e 0x73 0x74 0xaa 0xab 0xad 0xae 0xaf 0xd 0xe 0xf 0x10 0x11 0x12 0x9e 0xa7 || oneof 0xb4 0xb5 0xff 0x100 0x80000000 0xFFFFFFFF
  arg u32 alt any || range 0 300 || near 0x272 3 || oneof 0 1 || mask 0x1f
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || const 0x1ed || range 1 700 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  stub 02017f50 ret=u32 args=v,v,v
  stub 02045a38 ret=none args=v,v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 0201f448 ret=bool args=v,v
  stub 02008f3c ret=none args=v,v
