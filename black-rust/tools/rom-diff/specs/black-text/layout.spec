# fndiff spec: the text-measurement helpers of `printsys` (F9c), main ARM9 Thumb code
# 0x0201D26C..0x0201D338 (a different file from strbuf.c/wordset.c).  Grammar: svflow_specs/README.md
# (generic mode).  Run: tools/check-fn.sh black-text layout --per-function
#
# FUN_0201D26C(str, font, spacing, &next) -> width of one line; FUN_0201D304(strbuf, font, spacing) ->
# width of the longest line.  The glyph advance FUN_0201DC6C(font, code) is a stub (the real one
# calls a function pointer in the font object).  Strings are 0xFFFF-terminated; characters come from
# a small alphabet plus 0xFFFE/0xFFFF, with a tag candidate (0xF000, id, n <= 3, args) at three
# fixed positions so tag skipping stays inside the buffer.  The string pointer is arg0 (0x02E10000).
# Spacing is kept non-negative (black-fndiff num() rejects negatives).
area layout
set field

fn 0201d26c ret=u32
  nowork
  arg buf 0x60
  arg u32 range 0 0xffff
  arg u32 range 0 6
  arg buf 4
  stub 0201dc6c ret=mod:16 args=v,v
  fix arg0 0 u16 alt range 0x20 0x7e || range 0x20 0x7e || range 0x20 0x7e || range 0x3000 0x3010 || range 0x20 0x7e || range 0x20 0x7e || const 0xfffe || const 0xffff rep=47,2
  fix arg0 6 u16 alt const 0xf000 || range 0x20 0x7e
  fix arg0 10 u16 range 0 3
  fix arg0 26 u16 alt const 0xf000 || range 0x20 0x7e
  fix arg0 30 u16 range 0 3
  fix arg0 58 u16 alt const 0xf000 || range 0x20 0x7e
  fix arg0 62 u16 range 0 3
  fix arg0 94 u16 const 0xffff

fn 0201d304 ret=u32
  nowork
  arg buf 0x60
  arg u32 range 0 0xffff
  arg u32 range 0 6
  stub 0201dc6c ret=mod:16 args=v,v
  fix arg0 0 u16 range 0 20
  fix arg0 2 u16 range 0 20
  fix arg0 4 u32 const 0xB6F8D2EC
  fix arg0 8 u16 alt range 0x20 0x7e || range 0x20 0x7e || range 0x20 0x7e || range 0x3000 0x3010 || range 0x20 0x7e || range 0x20 0x7e || const 0xfffe || const 0xffff rep=43,2
  fix arg0 14 u16 alt const 0xf000 || range 0x20 0x7e
  fix arg0 18 u16 range 0 3
  fix arg0 34 u16 alt const 0xf000 || range 0x20 0x7e
  fix arg0 38 u16 range 0 3
  fix arg0 62 u16 alt const 0xf000 || range 0x20 0x7e
  fix arg0 66 u16 range 0 3
  fix arg0 94 u16 const 0xffff
