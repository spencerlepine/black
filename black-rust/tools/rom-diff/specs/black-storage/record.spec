# fndiff spec (S5b): the game records save block 38 of the main ARM9 (0x02008C20..0x02008EAC): 0x1E0 bytes
# (71 u32 + 96 u16 records, +0x1DC u16 checksum, +0x1DE u16 key; caps from rodata 0x020A70C4 / 0x0209DDB4).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage record --per-function
# The at-rest cipher is DS save plumbing the port replaces with plaintext: FUN_0203F2FC (checksum, reads the
# 0x1D8 enciphered bytes), FUN_0203F328 (encipher) and FUN_0203F37C (decipher) are stubbed (the port calls its
# RecordCrypt hooks in the same places). The V-count word 0x02FFFC3C the init reads is a random global.
# Record ids stay below 167 (the cap lookup reads the 167-byte table with the id unchecked); id 0 is drawn
# often (its open / seal flag is the id: no cipher calls). Records get values near their caps.
# MIi_CpuClear32 runs as real code.
area record
set field

fn 02008c20 ret=u32 inputs=10
  nowork

# sub_02008C28(block): zero, key from the V-count, seal
fn 02008c28 ret=none inputs=50
  nowork
  arg buf 0x1e0
  global 02fffc3c 4
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v

# FUN_02008C64(block, flag): seal when flag != 0
fn 02008c64 ret=none inputs=100
  nowork
  arg buf 0x1e0
  arg u32 alt const 0 || any
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v

# FUN_02008C94(block, flag): open when flag != 0
fn 02008c94 ret=none inputs=100
  nowork
  arg buf 0x1e0
  arg u32 alt const 0 || any
  stub 0203f37c ret=none args=p,v,v

# FUN_02008CB4(block, id): raw record (0 for id >= 167)
fn 02008cb4 ret=u32 inputs=400
  nowork
  arg buf 0x1e0
  arg u32 alt range 0 200 || near 71 2 || near 167 2

# FUN_02008CD4(block, id, v): store raw, return it
fn 02008cd4 ret=u32 inputs=400
  nowork
  arg buf 0x1e0
  arg u32 alt range 0 200 || near 71 2 || near 167 2
  arg u32 any

# FUN_02008CF8(id): the cap
fn 02008cf8 ret=u32 inputs=400
  nowork
  arg u32 range 0 166

# FUN_02008D1C(block, id, v): set min(v, cap)
fn 02008d1c ret=u32 inputs=800
  nowork
  arg buf 0x1e0
  arg u32 alt const 0 || range 0 166
  arg u32 alt range 0 1000 || near 9999 3 || near 0xffff 3 || near 999999 3 || any
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v
  stub 0203f37c ret=none args=p,v,v

# FUN_02008d58(block, id, v): keep the maximum
fn 02008d58 ret=u32 inputs=1200
  nowork
  arg buf 0x1e0
  arg u32 alt const 0 || range 0 166 || oneof 123 124
  arg u32 alt range 0 1000 || near 9999 3 || near 0xffff 3 || near 999999 3 || any
  fix arg0 0 u32 alt range 0 1000 || any rep=71,4
  fix arg0 0x11c u16 alt range 0 1000 || near 9999 3 || any rep=96,2
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v
  stub 0203f37c ret=none args=p,v,v

# sub_02008DAC(block, id, n): add and cap
fn 02008dac ret=u32 inputs=1200
  nowork
  arg buf 0x1e0
  arg u32 alt const 0 || range 0 166
  arg u32 alt range 0 1000 || near 9999 3 || any
  fix arg0 0 u32 alt range 0 1000 || near 999999999 3 || any rep=71,4
  fix arg0 0x11c u16 alt range 0 1000 || near 9999 3 || any rep=96,2
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v
  stub 0203f37c ret=none args=p,v,v

# FUN_02008DF0(block, id): add 1
fn 02008df0 ret=u32 inputs=600
  nowork
  arg buf 0x1e0
  arg u32 alt const 0 || range 0 166
  fix arg0 0 u32 alt range 0 1000 || near 999999999 3 || near 999999 3 || any rep=71,4
  fix arg0 0x11c u16 alt range 0 1000 || near 9999 3 || near 999 3 || any rep=96,2
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v
  stub 0203f37c ret=none args=p,v,v

# sub_02008DFC(block, id): capped record
fn 02008dfc ret=u32 inputs=600
  nowork
  arg buf 0x1e0
  arg u32 alt const 0 || range 0 166
  fix arg0 0 u32 alt range 0 1000 || near 999999999 3 || any rep=71,4
  fix arg0 0x11c u16 alt range 0 1000 || near 9999 3 || any rep=96,2
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v
  stub 0203f37c ret=none args=p,v,v

# FUN_02008E30(block): fold the five current records into their maxima
fn 02008e30 ret=none inputs=300
  nowork
  arg buf 0x1e0
  fix arg0 0 u32 alt range 0 1000 || near 999999999 3 || any rep=71,4
  fix arg0 0x11c u16 alt range 0 1000 || near 9999 3 || any rep=96,2
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v
  stub 0203f37c ret=none args=p,v,v

# FUN_02008e68(block, v): max of record 123 for v < 7
fn 02008e68 ret=none inputs=300
  nowork
  arg buf 0x1e0
  arg u32 alt range 0 10 || any
  fix arg0 0x184 u16 alt range 0 10 || any
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v
  stub 0203f37c ret=none args=p,v,v

# FUN_02008E78(block): record 123 (u8)
fn 02008e78 ret=u8 inputs=200
  nowork
  arg buf 0x1e0
  fix arg0 0x184 u16 alt range 0 10 || any
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v
  stub 0203f37c ret=none args=p,v,v

# FUN_02008e88(block, v): max of record 124 for v < 9999
fn 02008e88 ret=none inputs=300
  nowork
  arg buf 0x1e0
  arg u32 alt near 9999 3 || range 0 1000 || any
  fix arg0 0x186 u16 alt range 0 1000 || near 9999 3 || any
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v
  stub 0203f37c ret=none args=p,v,v

# 0x02008EA0(block): record 124 (tail call)
fn 02008ea0 ret=u32 inputs=200
  nowork
  arg buf 0x1e0
  fix arg0 0x186 u16 alt range 0 1000 || near 9999 3 || any
  stub 0203f2fc ret=u16 args=in:472,v
  stub 0203f328 ret=none args=ptr,v,v
  stub 0203f37c ret=none args=p,v,v
