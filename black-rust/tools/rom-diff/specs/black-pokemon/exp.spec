# fndiff spec: EXP of a record at its current level (P3), poke_tool.c of main ARM9.
# Run: tools/check-fn.sh black-pokemon exp --per-function
#
# FUN_0201854C(mon) reads species (id 5), form (id 0x6F) and level (id 0x9E) with the box getter
# FUN_02017E88 (ported in poke_tool.rs, runs as real code on really encrypted records, same planting
# recipe as create.spec) and returns FUN_02018584(species, form, level) (area growth, stubbed here).
area exp
set field

fn 0201854c ret=u32 inputs=400
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || any
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || range 1 649 || range 1 649 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x8c u8 alt range 1 100 || range 1 100 || range 1 100 || any
  fix region:mon 0x8e u16 alt range 0 400 || any
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || const 0 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  stub 02018584 ret=u32 args=v,v,v
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02045a38 ret=none args=v,v,v
  stub 0201f448 ret=bool args=v,v
  stub 02008f3c ret=none args=v,v
