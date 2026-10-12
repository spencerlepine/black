# fndiff spec: gf_standard.c RNGs (Z15), main ARM9 code.
# Run: tools/check-fn.sh black-core rng --per-function
#
# Mersenne Twister struct (heap, pointer in the global 0x021469F8): mt[624] @0, index @0x9C0,
# mag[2] @0x9C4 ({0, 0x9908B0DF}; FUN_0203F0A8 reads the table from the struct). The pointer
# global is pointed at region `st`; the rest of the region (and mag, unless planted) is random.
# 64-bit LCG context: the record at the fixed address 0x020AA194 holds, at +0x10, the pointer to
# the context {x u64, mul u64, add u64}.
# FUN_0203F258 callees (all unported SDK calls, stubbed): 02088168 entropy, 020813D8 sha1 init,
# 0208142C sha1 update (the data is NOT compared: its first call hashes uninitialised stack, so
# only the length is logged), 020814C0 sha1 final.
area rng
set field

# FUN_0203F034(uint seed): init_genrand on the struct at *0x021469F8 (index ends at 624)
fn 0203f034 ret=none mode=arm
  nowork
  global 021469f8 4
  region st 0xA00
  fix global0 0 u32 addrof region:st
  arg u32 any

# FUN_0203F0A8(): next tempered output; regenerates the block when index >= 624 (index 0x271 = unseeded)
fn 0203f0a8 ret=u32 mode=arm inputs=600
  nowork
  global 021469f8 4
  region st 0xA00
  fix global0 0 u32 addrof region:st
  fix region:st 0x9c0 u32 alt range 0 0x26f || range 0 0x26f || range 0 0x26f || near 0x26f 3 || near 0x271 3 || oneof 0x26e 0x26f 0x270 0x271 0x272 0x273 0x1000 0x7FFFFFFF || range 0x270 0x7FFFFFFF
  fix region:st 0x9c4 u32 alt const 0 || const 0 || const 0 || any
  fix region:st 0x9c8 u32 alt const 0x9908B0DF || const 0x9908B0DF || const 0x9908B0DF || any

# FUN_020056EC(uint max): 64-bit LCG step on the global context, returns hi (max==0) or hi*max>>32 (Thumb)
fn 020056ec ret=u32 inputs=800
  nowork
  global 020aa194 0x14
  region ctx 0x18
  fix global0 0x10 u32 addrof region:ctx
  arg u32 alt const 0 || const 0 || const 1 || const 0xFFFFFFFF || range 1 0x100 || oneof 2 3 6 0x64 0x80000000 0xFFFF || any

# FUN_02005728(uint max): same step, always scaled hi*max>>32 (Thumb)
fn 02005728 ret=u32 inputs=800
  nowork
  global 020aa194 0x14
  region ctx 0x18
  fix global0 0x10 u32 addrof region:ctx
  arg u32 alt const 0 || const 1 || const 0xFFFFFFFF || range 1 0x100 || oneof 2 3 6 0x64 0x80000000 0xFFFF || any

# FUN_0203F258(ctx): x = first 8 bytes of SHA-1(stack garbage[20], entropy[32]); mul, add constants
fn 0203f258 ret=none mode=arm
  nowork
  region ctx 0x18
  arg region ctx
  stub 02088168 ret=none args=out:32
  stub 020813d8 ret=none args=p
  stub 0208142c ret=none args=p,p,v
  stub 020814c0 ret=none args=p,out:20

# FUN_0203F2FC(const u8 *data, uint len): sum of the first len bytes
fn 0203f2fc ret=u32 mode=arm
  nowork
  region data 0x100
  arg region data
  arg u32 alt range 0 0x100 || oneof 0 1 2 3 0x7F 0x80 0xFF 0x100

# FUN_0203F328(void *data, uint size, uint seed): XOR halfword i < size/2 with the LCG32 key stream
fn 0203f328 ret=none mode=arm
  nowork
  region data 0x100
  arg region data
  arg u32 alt range 0 0x100 || oneof 0 1 2 3 4 0x54 0x80 0xFF 0x100
  arg u32 alt any || range 0 0xFFFF

# FUN_0203F388(uint *state): *state = *state * 0x41C64E6D + 0x6073; return *state >> 16
fn 0203f388 ret=u32 mode=arm
  nowork
  region state 4
  arg region state
