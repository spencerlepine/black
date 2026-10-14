# fndiff spec (S5b): perapvoice.c of the main ARM9 (0x02007B00..0x02007CB0): save block 36 (0x3EC bytes:
# +0 u32 exists, +4 1000 bytes of packed 4-bit samples).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage perapvoice --per-function
# MIi_CpuClear32 / MI_CpuCopy8 run as real code; the party queries are stubbed.
area perapvoice
set field

fn 02007b00 ret=u32 inputs=10
  nowork

# FUN_02007b08(block): zero
fn 02007b08 ret=none inputs=50
  nowork
  arg buf 0x3ec

# FUN_02007b50(block): exists word
fn 02007b50 ret=u32 inputs=50
  nowork
  arg buf 0x3ec

# 0x02007B54(block): exists = 0
fn 02007b54 ret=none inputs=20
  nowork
  arg buf 0x3ec

# 0x02007B5C(block): exists = 1
fn 02007b5c ret=none inputs=20
  nowork
  arg buf 0x3ec

# FUN_02007b64(block): block + 4
fn 02007b64 ret=ofs:arg0 inputs=10
  nowork
  arg buf 0x3ec

# FUN_02007b68(dst, src): unpack 1000 bytes to 8000 samples, zero to 0x1FF4
fn 02007b68 ret=none inputs=30
  nowork
  arg buf 0x2000
  arg buf 0x3e8

# FUN_02007bd0(block, src): pack 8000 signed samples
fn 02007bd0 ret=none inputs=30
  nowork
  arg buf 0x3ec
  arg buf 0x1f40

# 0x02007C20(dst, src): copy the block
fn 02007c20 ret=none inputs=20
  nowork
  arg buf 0x3ec
  arg buf 0x3ec

# FUN_02007C34(block, party): no Chatot in the party -> exists = 0
fn 02007c34 ret=none inputs=400
  nowork
  arg buf 0x3ec
  arg u32 any
  stub 0201aa18 ret=pick+u32:0/1/2/3/6 args=v
  stub 0201ab28 ret=u32 args=v,v
  stub 02017e1c ret=pick+u32:441/25/0 args=v,v,v

# FUN_02007C70(block): cry kind from three sample bytes
fn 02007c70 ret=u32 inputs=600
  nowork
  arg buf 0x3ec
  fix arg0 0 u32 alt oneof 1 || any
