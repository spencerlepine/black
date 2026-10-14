# fndiff spec (S4b): item use on a party Pokemon, outside battle: the party-summary overlay 91
# (0x021BBE74..0x021BEA7C). Run: tools/check-fn.sh black-storage item_use --per-function
#
# Boundary model. Item records are 36 bytes (ARC 024) planted in a 36-byte buffer argument; the archive
# loader FUN_02048F60 is stubbed to return that buffer (the item.spec trick), Heap_Free is a stub. The
# record loader FUN_02020F34 and the parameter reader FUN_02021014 run as real code. Usage-block bytes
# (record 0x10..0x23) are drawn sparse (mostly zero, single bits) so each parameter combination of the
# classifier and the effect code is reached; the signed amounts 0x34..0x39, 0x3C..0x3E (bytes 7..12 and
# 15..17) take small, negative and edge values.
# The Pokemon is a really encrypted party record (the evolution.spec recipe: PID shuffle row 0 mostly,
# checksum, ciphered blocks and extension); the getters/setters FUN_02017E1C / FUN_02017E40 run as real
# code. The callees of other areas are stubs: FUN_020185C4 (recalc), FUN_02018584 (growth curve),
# FUN_0201B920 / FUN_0201B970 (PP), FUN_0201B7B4 (friendship by item), FUN_0201C158 (max PP) and the
# gender/level/personal callees of the getters (their stub lines are only active if reached).
area item_use
set party-summary

# FUN_overlay_d_91__021bbe74(item): the use kind of an item (record loaded with heap 0x17).
fn 021bbe74 ret=u32 inputs=6000
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 36
  fix arg3 0xc u8 alt const 1 || const 1 || const 1 || const 1 || const 0 || any
  fix arg3 0x10 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x3F 0x0F 0x1F 0x21 0x41 0x81 0x30 0x60 || any
  fix arg3 0x11 u8 alt const 0 || const 0 || oneof 1 2 4 8 3 0xC 5 9 || any
  fix arg3 0x12 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg3 0x13 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg3 0x14 u8 alt const 0 || const 0x40 || const 0x80 || const 0x40 || const 0xC0 || any
  fix arg3 0x15 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x18 0x28 0x30 0x48 0x88 || any
  fix arg3 0x16 u8 alt const 0 || oneof 1 2 4 8 0x10 || oneof 3 5 7 0x11 0x1F || any
  fix arg3 0x17 u8 alt const 0 || const 0 || oneof 1 5 10 0x7F 0x80 0xFF 0xFE 0xF6 || any rep=6,1
  fix arg3 0x1d u8 alt oneof 0xff 0xfe 0xfd 20 50 200 || range 0 255
  fix arg3 0x1e u8 alt oneof 1 0x7f 5 10 || any
  fix arg3 0x1f u8 alt oneof 1 3 5 || const 0 || oneof 1 3 5 0xFF 0xFE 0xFB || any rep=3,1
  stub 02048f60 ret=pick:0x02E40000 args=v,v,v,v
  stub 020307b0 ret=none args=v

# FUN_overlay_d_91__021bc17c(work, item): 1 for PP Up / PP Max items and for single-move PP restores
# (parameter 0x27, read with FUN_02020FE4); the first halfword of the work buffer is the heap id.
fn 021bc17c ret=u32 inputs=6000
  nowork
  arg buf 4
  arg u32 any
  arg u32 any
  arg buf 36
  fix arg3 0xc u8 alt const 1 || const 1 || const 1 || const 1 || const 0 || any
  fix arg3 0x10 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x3F 0x0F 0x1F 0x21 0x41 0x81 0x30 0x60 || any
  fix arg3 0x11 u8 alt const 0 || const 0 || oneof 1 2 4 8 3 0xC 5 9 || any
  fix arg3 0x12 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg3 0x13 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg3 0x14 u8 alt const 0 || const 0x40 || const 0x80 || const 0x40 || const 0xC0 || any
  fix arg3 0x15 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x18 0x28 0x30 0x48 0x88 || any
  fix arg3 0x16 u8 alt const 0 || oneof 1 2 4 8 0x10 || oneof 3 5 7 0x11 0x1F || any
  fix arg3 0x17 u8 alt const 0 || const 0 || oneof 1 5 10 0x7F 0x80 0xFF 0xFE 0xF6 || any rep=6,1
  fix arg3 0x1d u8 alt oneof 0xff 0xfe 0xfd 20 50 200 || range 0 255
  fix arg3 0x1e u8 alt oneof 1 0x7f 5 10 || any
  fix arg3 0x1f u8 alt oneof 1 3 5 || const 0 || oneof 1 3 5 0xFF 0xFE 0xFB || any rep=3,1
  stub 02048f60 ret=pick:0x02E40000 args=v,v,v,v
  stub 020307b0 ret=none args=v

# FUN_overlay_d_91__021bc1b4(work, item): usage parameter 0x1B != 0 (FUN_02020FE4, heap = first halfword of work).
fn 021bc1b4 ret=u32 inputs=3000
  nowork
  arg buf 4
  arg u32 any
  arg u32 any
  arg buf 36
  fix arg3 0xc u8 alt const 1 || const 1 || const 1 || const 1 || const 0 || any
  fix arg3 0x10 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x3F 0x0F 0x1F 0x21 0x41 0x81 0x30 0x60 || any
  fix arg3 0x11 u8 alt const 0 || const 0 || oneof 1 2 4 8 3 0xC 5 9 || any
  fix arg3 0x12 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg3 0x13 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg3 0x14 u8 alt const 0 || const 0x40 || const 0x80 || const 0x40 || const 0xC0 || any
  fix arg3 0x15 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x18 0x28 0x30 0x48 0x88 || any
  fix arg3 0x16 u8 alt const 0 || oneof 1 2 4 8 0x10 || oneof 3 5 7 0x11 0x1F || any
  fix arg3 0x17 u8 alt const 0 || const 0 || oneof 1 5 10 0x7F 0x80 0xFF 0xFE 0xF6 || any rep=6,1
  fix arg3 0x1d u8 alt oneof 0xff 0xfe 0xfd 20 50 200 || range 0 255
  fix arg3 0x1e u8 alt oneof 1 0x7f 5 10 || any
  fix arg3 0x1f u8 alt oneof 1 3 5 || const 0 || oneof 1 3 5 0xFF 0xFE 0xFB || any rep=3,1
  stub 02048f60 ret=pick:0x02E40000 args=v,v,v,v
  stub 020307b0 ret=none args=v

# FUN_overlay_d_91__021bc228(work, item): 0x16 for PP Up / PP Max items, else 0x15.
fn 021bc228 ret=u32 inputs=4000
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 36
  fix arg3 0xc u8 alt const 1 || const 1 || const 1 || const 1 || const 0 || any
  fix arg3 0x10 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x3F 0x0F 0x1F 0x21 0x41 0x81 0x30 0x60 || any
  fix arg3 0x11 u8 alt const 0 || const 0 || oneof 1 2 4 8 3 0xC 5 9 || any
  fix arg3 0x12 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg3 0x13 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg3 0x14 u8 alt const 0 || const 0x40 || const 0x80 || const 0x40 || const 0xC0 || any
  fix arg3 0x15 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x18 0x28 0x30 0x48 0x88 || any
  fix arg3 0x16 u8 alt const 0 || oneof 1 2 4 8 0x10 || oneof 3 5 7 0x11 0x1F || any
  fix arg3 0x17 u8 alt const 0 || const 0 || oneof 1 5 10 0x7F 0x80 0xFF 0xFE 0xF6 || any rep=6,1
  fix arg3 0x1d u8 alt oneof 0xff 0xfe 0xfd 20 50 200 || range 0 255
  fix arg3 0x1e u8 alt oneof 1 0x7f 5 10 || any
  fix arg3 0x1f u8 alt oneof 1 3 5 || const 0 || oneof 1 3 5 0xFF 0xFE 0xFB || any rep=3,1
  stub 02048f60 ret=pick:0x02E40000 args=v,v,v,v
  stub 020307b0 ret=none args=v

# FUN_overlay_d_91__021bdd78(mon, item, slot, heap): can the item be used on the Pokemon.
fn 021bdd78 ret=u32 inputs=6000
  nowork
  region mon 0xDC
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 292 || const 292 || const 292 || range 1 649 || range 1 649 || any
  fix region:mon 0x14 u8 alt const 0 || const 255 || oneof 99 100 199 200 254 || range 0 255
  fix region:mon 0x18 u8 alt const 0 || const 0 || range 0 90 || range 0 40 || oneof 0 99 100 101 254 255 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x30 u8 alt range 0 60 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x88 u32 alt const 0 || range 1 5 || range 0 7 || any
  fix region:mon 0x8c u8 alt range 1 100 || oneof 99 100 101 || any
  fix region:mon 0x8e u16 alt const 0 || range 0 12 || range 0 400 || any
  fix region:mon 0x90 u16 alt range 1 12 || range 1 400 || const 1 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  arg region mon
  arg u32 any
  arg u32 range 0 3
  arg u32 any
  arg buf 36
  fix arg4 0xc u8 alt const 1 || const 1 || const 1 || const 1 || const 0 || any
  fix arg4 0x10 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x3F 0x0F 0x1F 0x21 0x41 0x81 0x30 0x60 || any
  fix arg4 0x11 u8 alt const 0 || const 0 || oneof 1 2 4 8 3 0xC 5 9 || any
  fix arg4 0x12 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg4 0x13 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg4 0x14 u8 alt const 0 || const 0x40 || const 0x80 || const 0x40 || const 0xC0 || any
  fix arg4 0x15 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x18 0x28 0x30 0x48 0x88 || any
  fix arg4 0x16 u8 alt const 0 || oneof 1 2 4 8 0x10 || oneof 3 5 7 0x11 0x1F || any
  fix arg4 0x17 u8 alt const 0 || const 0 || oneof 1 5 10 0x7F 0x80 0xFF 0xFE 0xF6 || any rep=6,1
  fix arg4 0x1d u8 alt oneof 0xff 0xfe 0xfd 20 50 200 || range 0 255
  fix arg4 0x1e u8 alt oneof 1 0x7f 5 10 || any
  fix arg4 0x1f u8 alt oneof 1 3 5 || const 0 || oneof 1 3 5 0xFF 0xFE 0xFB || any rep=3,1
  stub 02048f60 ret=pick:0x02E50000 args=v,v,v,v
  stub 020307b0 ret=none args=v
  stub 02017988 ret=none args=p,v
  stub 020185c4 ret=none args=ptr
  stub 02018584 ret=u32 args=v,v,v
  stub 0201b920 ret=mod:2 args=ptr,v
  stub 0201b970 ret=mod:2 args=ptr,v,v
  stub 0201b7b4 ret=none args=p,v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02017988 ret=none args=p,v
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02008f3c ret=none args=v,v

# FUN_overlay_d_91__021be2b4(mon, item, slot, location, heap): use the item on the Pokemon.
fn 021be2b4 ret=u32 inputs=6000
  nowork
  region mon 0xDC
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 292 || const 292 || const 292 || range 1 649 || range 1 649 || any
  fix region:mon 0x14 u8 alt const 0 || const 255 || oneof 99 100 199 200 254 || range 0 255
  fix region:mon 0x18 u8 alt const 0 || const 0 || range 0 90 || range 0 40 || oneof 0 99 100 101 254 255 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x30 u8 alt range 0 60 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x88 u32 alt const 0 || range 1 5 || range 0 7 || any
  fix region:mon 0x8c u8 alt range 1 100 || oneof 99 100 101 || any
  fix region:mon 0x8e u16 alt const 0 || range 0 12 || range 0 400 || any
  fix region:mon 0x90 u16 alt range 1 12 || range 1 400 || const 1 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  arg region mon
  arg u32 any
  arg u32 range 0 3
  arg u32 any
  arg u32 any
  arg buf 36
  fix arg5 0xc u8 alt const 1 || const 1 || const 1 || const 1 || const 0 || any
  fix arg5 0x10 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x3F 0x0F 0x1F 0x21 0x41 0x81 0x30 0x60 || any
  fix arg5 0x11 u8 alt const 0 || const 0 || oneof 1 2 4 8 3 0xC 5 9 || any
  fix arg5 0x12 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg5 0x13 u8 alt const 0 || const 0 || const 0 || const 0 || any
  fix arg5 0x14 u8 alt const 0 || const 0x40 || const 0x80 || const 0x40 || const 0xC0 || any
  fix arg5 0x15 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || oneof 0x18 0x28 0x30 0x48 0x88 || any
  fix arg5 0x16 u8 alt const 0 || oneof 1 2 4 8 0x10 || oneof 3 5 7 0x11 0x1F || any
  fix arg5 0x17 u8 alt const 0 || const 0 || oneof 1 5 10 0x7F 0x80 0xFF 0xFE 0xF6 || any rep=6,1
  fix arg5 0x1d u8 alt oneof 0xff 0xfe 0xfd 20 50 200 || range 0 255
  fix arg5 0x1e u8 alt oneof 1 0x7f 5 10 || any
  fix arg5 0x1f u8 alt oneof 1 3 5 || const 0 || oneof 1 3 5 0xFF 0xFE 0xFB || any rep=3,1
  stub 02048f60 ret=pick:0x02E60000 args=v,v,v,v
  stub 020307b0 ret=none args=v
  stub 02017988 ret=none args=p,v
  stub 020185c4 ret=none args=ptr
  stub 02018584 ret=u32 args=v,v,v
  stub 0201b920 ret=mod:2 args=ptr,v
  stub 0201b970 ret=mod:2 args=ptr,v,v
  stub 0201b7b4 ret=none args=p,v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02017988 ret=none args=p,v
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02008f3c ret=none args=v,v

# FUN_overlay_d_91__021be874(mon, slot, ups): add PP Ups to a move slot.
fn 021be874 ret=u32 inputs=2500
  nowork
  region mon 0xDC
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 292 || const 292 || const 292 || range 1 649 || range 1 649 || any
  fix region:mon 0x14 u8 alt const 0 || const 255 || oneof 99 100 199 200 254 || range 0 255
  fix region:mon 0x18 u8 alt const 0 || const 0 || range 0 90 || range 0 40 || oneof 0 99 100 101 254 255 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x30 u8 alt range 0 60 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x88 u32 alt const 0 || range 1 5 || range 0 7 || any
  fix region:mon 0x8c u8 alt range 1 100 || oneof 99 100 101 || any
  fix region:mon 0x8e u16 alt const 0 || range 0 12 || range 0 400 || any
  fix region:mon 0x90 u16 alt range 1 12 || range 1 400 || const 1 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  arg region mon
  arg u32 range 0 3
  # V-0032: the ups count near u32::MAX wraps in `cur + ups` (adds, then unsigned bls)
  arg u32 alt oneof 1 3 || range 0 5 || oneof 0xFFFFFFFF 0xFFFFFFFE 0xFFFFFFFD || any
  stub 02017988 ret=none args=p,v
  stub 020185c4 ret=none args=ptr
  stub 02018584 ret=u32 args=v,v,v
  stub 0201b920 ret=mod:2 args=ptr,v
  stub 0201b970 ret=mod:2 args=ptr,v,v
  stub 0201b7b4 ret=none args=p,v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02017988 ret=none args=p,v
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02008f3c ret=none args=v,v

# FUN_overlay_d_91__021be914(mon, hp, max, amount): raise the current HP.
fn 021be914 ret=none inputs=2500
  nowork
  region mon 0xDC
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 292 || const 292 || const 292 || range 1 649 || range 1 649 || any
  fix region:mon 0x14 u8 alt const 0 || const 255 || oneof 99 100 199 200 254 || range 0 255
  fix region:mon 0x18 u8 alt const 0 || const 0 || range 0 90 || range 0 40 || oneof 0 99 100 101 254 255 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x30 u8 alt range 0 60 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x88 u32 alt const 0 || range 1 5 || range 0 7 || any
  fix region:mon 0x8c u8 alt range 1 100 || oneof 99 100 101 || any
  fix region:mon 0x8e u16 alt const 0 || range 0 12 || range 0 400 || any
  fix region:mon 0x90 u16 alt range 1 12 || range 1 400 || const 1 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  arg region mon
  arg u32 alt range 0 400 || any
  arg u32 alt range 0 400 || const 1 || any
  arg u32 alt oneof 0xff 0xfe 0xfd 1 20 || range 0 300 || any
  stub 02017988 ret=none args=p,v
  stub 020185c4 ret=none args=ptr
  stub 02018584 ret=u32 args=v,v,v
  stub 0201b920 ret=mod:2 args=ptr,v
  stub 0201b970 ret=mod:2 args=ptr,v,v
  stub 0201b7b4 ret=none args=p,v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02017988 ret=none args=p,v
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02008f3c ret=none args=v,v

# FUN_overlay_d_91__021be948(ev, others, delta, flag): the new EV after adding delta (-1: not changeable).
fn 021be948 ret=i32 inputs=8000
  nowork
  arg s32 alt range 0 255 || near 100 3 || near 255 2 || any
  arg s32 alt range 0 510 || near 510 12 || any
  arg s32 alt range -12 12 || oneof 1 0xffffffff 10 0xfffffff6 100 || any
  arg u32 alt const 0 || const 1 || any

# FUN_overlay_d_91__021be9bc(mon, rec): does the item record raise the friendship of the Pokemon.
fn 021be9bc ret=u32 inputs=2500
  nowork
  region mon 0xDC
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 292 || const 292 || const 292 || range 1 649 || range 1 649 || any
  fix region:mon 0x14 u8 alt const 0 || const 255 || oneof 99 100 199 200 254 || range 0 255
  fix region:mon 0x18 u8 alt const 0 || const 0 || range 0 90 || range 0 40 || oneof 0 99 100 101 254 255 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x30 u8 alt range 0 60 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x88 u32 alt const 0 || range 1 5 || range 0 7 || any
  fix region:mon 0x8c u8 alt range 1 100 || oneof 99 100 101 || any
  fix region:mon 0x8e u16 alt const 0 || range 0 12 || range 0 400 || any
  fix region:mon 0x90 u16 alt range 1 12 || range 1 400 || const 1 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  arg region mon
  arg buf 36
  fix arg1 0x10 u8 alt const 0 || oneof 1 2 4 8 0x10 0x20 0x40 0x80 || any
  fix arg1 0x16 u8 alt const 0 || oneof 1 2 4 8 0x10 || oneof 3 5 7 0x11 0x1F || any
  fix arg1 0x1f u8 alt oneof 1 3 5 || const 0 || oneof 1 3 5 0xFF 0xFE 0xFB || any rep=3,1
  fix arg1 0xc u8 alt const 1 || const 1 || const 0 || any
  stub 02017988 ret=none args=p,v
  stub 020185c4 ret=none args=ptr
  stub 02018584 ret=u32 args=v,v,v
  stub 0201b920 ret=mod:2 args=ptr,v
  stub 0201b970 ret=mod:2 args=ptr,v,v
  stub 0201b7b4 ret=none args=p,v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02017988 ret=none args=p,v
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02008f3c ret=none args=v,v

# FUN_overlay_d_91__021bea48(mon, friendship, delta, item, location, heap): apply the item's friendship deltas.
fn 021bea48 ret=u32 inputs=2500
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt oneof 0 255 1 254 99 100 || range 0 255 || any
  arg s32 alt range -12 12 || oneof 0xFF 0xffffffff 5 || any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02017988 ret=none args=p,v
  stub 020185c4 ret=none args=ptr
  stub 02018584 ret=u32 args=v,v,v
  stub 0201b920 ret=mod:2 args=ptr,v
  stub 0201b970 ret=mod:2 args=ptr,v,v
  stub 0201b7b4 ret=none args=p,v,v,v
  stub 0201c158 ret=u32 args=v,v

# FUN_overlay_d_91__021bc24c(work, mon, item): an Arceus takes the form of a plate item (FUN_02019CF8 = plate_type).
fn 021bc24c ret=none inputs=1500
  nowork
  region mon 0xDC
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 0x1ed || const 0x1ed || const 0x289 || const 0x289 || range 1 649
  fix region:mon 0x40 u8 alt range 0 255
  fix region:mon 0x14 u8 alt const 0 || const 255 || oneof 99 100 199 200 254 || range 0 255
  fix region:mon 0x18 u8 alt const 0 || const 0 || range 0 90 || range 0 40 || oneof 0 99 100 101 254 255 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x30 u8 alt range 0 60 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x88 u32 alt const 0 || range 1 5 || range 0 7 || any
  fix region:mon 0x8c u8 alt range 1 100 || oneof 99 100 101 || any
  fix region:mon 0x8e u16 alt const 0 || range 0 12 || range 0 400 || any
  fix region:mon 0x90 u16 alt range 1 12 || range 1 400 || const 1 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  arg u32 any
  arg region mon
  arg u32 alt near 0x131 0x10 || near 0x12a 4 || near 0x139 4 || any
  stub 02017988 ret=none args=p,v
  stub 020185c4 ret=none args=ptr
  stub 02018584 ret=u32 args=v,v,v
  stub 0201b920 ret=mod:2 args=ptr,v
  stub 0201b970 ret=mod:2 args=ptr,v,v
  stub 0201b7b4 ret=none args=p,v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02017988 ret=none args=p,v
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02008f3c ret=none args=v,v

# FUN_overlay_d_91__021bc288(work, mon, item): a Genesect takes the form of a Drive item (FUN_02019D00).
fn 021bc288 ret=none inputs=1500
  nowork
  region mon 0xDC
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 0x1ed || const 0x1ed || const 0x289 || const 0x289 || range 1 649
  fix region:mon 0x40 u8 alt range 0 255
  fix region:mon 0x14 u8 alt const 0 || const 255 || oneof 99 100 199 200 254 || range 0 255
  fix region:mon 0x18 u8 alt const 0 || const 0 || range 0 90 || range 0 40 || oneof 0 99 100 101 254 255 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x30 u8 alt range 0 60 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x88 u32 alt const 0 || range 1 5 || range 0 7 || any
  fix region:mon 0x8c u8 alt range 1 100 || oneof 99 100 101 || any
  fix region:mon 0x8e u16 alt const 0 || range 0 12 || range 0 400 || any
  fix region:mon 0x90 u16 alt range 1 12 || range 1 400 || const 1 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  arg u32 any
  arg region mon
  arg u32 alt range 0x72 0x79 || any
  stub 02017988 ret=none args=p,v
  stub 020185c4 ret=none args=ptr
  stub 02018584 ret=u32 args=v,v,v
  stub 0201b920 ret=mod:2 args=ptr,v
  stub 0201b970 ret=mod:2 args=ptr,v,v
  stub 0201b7b4 ret=none args=p,v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02017988 ret=none args=p,v
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02008f3c ret=none args=v,v

# FUN_02017F90(mon): the status word of a party Pokemon (field 0x9D of FUN_02017E1C).
fn 02017f90 ret=u32 inputs=800
  nowork
  region mon 0xDC
  fix region:mon 0 u32 alt const 0 || const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x08 u16 alt const 292 || const 292 || const 292 || range 1 649 || range 1 649 || any
  fix region:mon 0x14 u8 alt const 0 || const 255 || oneof 99 100 199 200 254 || range 0 255
  fix region:mon 0x18 u8 alt const 0 || const 0 || range 0 90 || range 0 40 || oneof 0 99 100 101 254 255 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x30 u8 alt range 0 60 || any rep=4,1
  fix region:mon 0x34 u8 alt range 0 3 || range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x88 u32 alt const 0 || range 1 5 || range 0 7 || any
  fix region:mon 0x8c u8 alt range 1 100 || oneof 99 100 101 || any
  fix region:mon 0x8e u16 alt const 0 || range 0 12 || range 0 400 || any
  fix region:mon 0x90 u16 alt range 1 12 || range 1 400 || const 1 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  arg region mon
  stub 02017988 ret=none args=p,v
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=ptr,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02008f3c ret=none args=v,v

# FUN_02019D00(item): the Genesect form of a Drive item (0x74..0x77 -> 1..4), else 0.
fn 02019d00 ret=u32 inputs=400
  nowork
  arg u32 alt range 0x70 0x7b || any
