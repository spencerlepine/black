# fndiff spec (S5): regulation.c of the main ARM9 (0x0200ADE8..0x0200B364): regulations (0xBC),
# regulation cards (0x160) and save block 47 (0x2FC = 2 cards + 2 x 30-byte party summaries).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage regulation --per-function
#
# Parameter ids 0..22 (FUN_0200AE54 / FUN_0200AEE8 jump tables), card ids 0..12 (FUN_0200B078 /
# FUN_0200B110); out-of-range ids are drawn too. Setter values hit every bound (signed compares).
# FUN_0203F1FC (CRC-16 of (data, len)) is stubbed with an `in:` argument so the answer depends on
# the card bytes. The party callees of FUN_0200B2A4 (count FUN_0201AA18, slot FUN_0201AB28, field
# FUN_02017E1C) are stubbed. MI_CpuCopy8 / MI_CpuFill8 run as real code.
area regulation
set field

fn 0200ade8 ret=u32 inputs=10
  nowork

# FUN_0200AE08(src, dst): MI_CpuCopy8(src, dst, 0xBC)
fn 0200ae08 ret=none
  nowork
  arg buf 0xbc
  arg buf 0xbc

# FUN_0200AE14: bx lr
fn 0200ae14 ret=none inputs=20
  nowork
  arg buf 0xbc

# FUN_0200AE54(reg, id)
fn 0200ae54 ret=u32 inputs=600
  nowork
  arg buf 0xbc
  arg u32 alt range 0 23 || range 0 23 || any

# FUN_0200AEE8(reg, id, v): store if v passes the id's bound
fn 0200aee8 ret=none inputs=3000
  nowork
  arg buf 0xbc
  arg u32 alt range 0 23 || range 0 23 || any
  arg u32 alt range 0 8 || near 100 2 || near 600 2 || near 0x28b 2 || near 99 1 || oneof 0xffffffff 0x80000000 0xfffffff9 || any

# FUN_0200AFD4(reg, id, n): bit n of the bit set of id 9 / 10 / 14 (n kept inside id 14's 7 bytes)
fn 0200afd4 ret=bool inputs=1000
  nowork
  arg buf 0xbc
  arg u32 alt oneof 9 10 14 || oneof 9 10 14 || range 0 16 || any
  arg u32 range 0 55

# FUN_0200B040(reg, out): out[0..7] = +0xAC..+0xB2, out[7] = min(param 13, 1 when >= 2)
fn 0200b040 ret=none inputs=300
  nowork
  arg buf 0xbc
  arg buf 8
  fix arg0 0xab u8 alt range 0 3 || any

# FUN_0200B074(card): card + 0xC0
fn 0200b074 ret=ofs:arg0 inputs=20
  nowork
  arg buf 0x160

# FUN_0200B078(card, id): card params (id 0 / > 12: the card address)
fn 0200b078 ret=u32 inputs=600
  nowork
  arg buf 0x160
  arg u32 alt range 0 13 || range 0 13 || any

# FUN_0200B110(card, id, v)
fn 0200b110 ret=none inputs=600
  nowork
  arg buf 0x160
  arg u32 alt range 0 13 || range 0 13 || any
  arg u32 any

# FUN_0200B1A8(card): FUN_0200B1C4(card) == card[0x15E]
fn 0200b1a8 ret=bool inputs=400
  nowork
  arg buf 0x160
  stub 0203f1fc ret=pick+u16:0/1/0xffff args=in:0x15e,v
  fix arg0 0x15e u16 alt oneof 0 1 0xffff || any

# FUN_0200B1C4(card): FUN_0203F1FC(card, 0x15E) (tail call)
fn 0200b1c4 ret=u16
  nowork
  arg buf 0x160
  stub 0203f1fc ret=u16 args=in:0x15e,v

# FUN_0200B1D4(block, i): block + i * 0x160
fn 0200b1d4 ret=ofs:arg0 inputs=20
  nowork
  arg buf 0x2fc
  arg u32 range 0 1

# unlabeled 0x0200B1E0(block, i, src): card i = *src (0x160 bytes)
fn 0200b1e0 ret=none inputs=50
  nowork
  arg buf 0x2fc
  arg u32 range 0 1
  arg buf 0x160

# FUN_0200B1F8(sum): clear 30 bytes
fn 0200b1f8 ret=none inputs=50
  nowork
  arg buf 0x20

# FUN_0200B204(block, i): block + 0x2C0 + 30i
fn 0200b204 ret=ofs:arg0 inputs=20
  nowork
  arg buf 0x2fc
  arg u32 range 0 1

# FUN_0200B214(sum, kind, idx)
fn 0200b214 ret=u32 inputs=600
  nowork
  arg buf 0x20
  arg u32 alt range 0 4 || any
  arg u32 alt range 0 5 || range 0 5 || oneof 6 7

# unlabeled 0x0200B254(sum, kind, idx, v)
fn 0200b254 ret=none inputs=600
  nowork
  arg buf 0x20
  arg u32 alt range 0 4 || any
  arg u32 alt range 0 5 || range 0 5 || oneof 6 7
  arg u32 alt range 0 2 || any

# FUN_0200B2A4(sum, party)
fn 0200b2a4 ret=none inputs=600
  nowork
  arg buf 0x20
  arg u32 any
  stub 0201aa18 ret=pick:0/1/2/3/4/5/6 args=v
  stub 0201ab28 ret=u32 args=v,v
  stub 02017e1c ret=pick+u32:0/1/2 args=v,v,v

# sub_0200B348(block): MI_CpuFill8(block, 0, 0x2FC)
fn 0200b348 ret=none inputs=20
  nowork
  arg buf 0x2fc
