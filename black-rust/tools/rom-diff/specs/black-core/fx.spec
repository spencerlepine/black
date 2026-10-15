# fndiff spec: NitroSDK fixed-point math FX / VEC / MTX (Z24, widened by Z25), main ARM9 ARM code.
# Run: tools/check-fn.sh black-core fx --per-function
# The divider / square-root users (FX_Div 0207C768 .. FX_Atan2Idx 0207CFB8, VEC_Mag / Normalize / Distance) run on the ROM side
# directly since Z25: the emulator models the DS hardware divider and sqrt unit (tools/rom-diff/emu.py, I/O 0x04000280..0x040002BF,
# computed on every read of a result register; GBATEK rules for division by zero, MIN / -1 and the 32/32 upper word). Writes to that
# block are not strays and the block is cleared before every input (svflow.py). Words whose operand is a divisor / sqrt argument
# draw from the 23 boundary values of the old fx_hw_diff.py (0, +-1, MIN / MAX, +-0x1000 = 1.0, +-0x800 ..), +-4 around them, small
# and masked words, and plain random words; FX_Atan2Idx never gets i32::MIN (the original reads outside its table there).
# Vectors are 12-byte regions (VecFx32) or 8-byte regions of 3 halfwords (VecFx16); matrices 0x24 / 0x30 bytes.
# Words come from a long list of boundary values (0, +-1, i32::MIN / MAX, +-0x1000 = 1.0, +-0x800 rounding edge, 0x40000000 ..)
# mixed with random and masked draws; the 64-bit accumulations overflow on the big ones (validator lesson V-0034).
area fx
set field

# FUN_0207AF50 MTX_RotAxis33(MtxFx33 *dst, const VecFx32 *axis, fx32 sin, fx32 cos)
fn 0207af50 ret=none mode=arm inputs=1500
  nowork
  region dst 0x24
  region axis 0xC
  arg region dst
  arg region axis
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x7FF 0x10000 0xFFFF0000 0x40000000 0xC0000000 0xB505 0xFFFF4AFB || any || range 0 0x1000 || near 0xFFFFF000 0x400 || mask 0xFFFFF
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x7FF 0x10000 0xFFFF0000 0x40000000 0xC0000000 0xB505 0xFFFF4AFB || any || range 0 0x1000 || near 0xFFFFF000 0x400 || mask 0xFFFFF
  fix region:axis 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 0xB505 0xFFFF4AFB || any || range 0 0x1000 || near 0xFFFFF000 0x400 || mask 0xFFFFF rep=3,4

# FUN_0207B65C MTX_RotAxis43(MtxFx43 *dst, const VecFx32 *axis, fx32 sin, fx32 cos): calls 0207AF50 (ported, real code), zeroes the translation
fn 0207b65c ret=none mode=arm inputs=1000
  nowork
  region dst 0x30
  region axis 0xC
  arg region dst
  arg region axis
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0xB505 0xFFFF4AFB || any || range 0 0x1000 || mask 0xFFFFF
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0xB505 0xFFFF4AFB || any || range 0 0x1000 || mask 0xFFFFF
  fix region:axis 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0xB505 0xFFFF4AFB || any || range 0 0x1000 || mask 0xFFFFF rep=3,4

# FUN_0207BC08 MTX_MultVec43(const VecFx32 *v, const MtxFx43 *m, VecFx32 *dst)
fn 0207bc08 ret=none mode=arm inputs=1500
  nowork
  region v 0xC
  region m 0x30
  region dst 0xC
  arg region v
  arg region m
  arg region dst
  fix region:v 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 || any || any || range 0 0x20000 || near 0x1000 0x1000 || near 0xFFFFF000 0x1000 || mask 0xFFFFFF rep=3,4
  fix region:m 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 || any || any || range 0 0x2000 || near 0x1000 0x400 || near 0xFFFFF000 0x400 || mask 0xFFFFFF rep=12,4

# FUN_0207C9D0 VEC_Add(a, b, dst)
fn 0207c9d0 ret=none mode=arm inputs=600
  nowork
  region a 0xC
  region b 0xC
  region dst 0xC
  arg region a
  arg region b
  arg region dst
  fix region:a 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 || any || range 0 0x20000 || near 0x7FFFFFFF 4 || near 0x80000000 4 rep=3,4
  fix region:b 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 || any || range 0 0x20000 || near 0x7FFFFFFF 4 || near 0x80000000 4 rep=3,4

# FUN_0207CA00 VEC_Subtract(a, b, dst): a - b
fn 0207ca00 ret=none mode=arm inputs=600
  nowork
  region a 0xC
  region b 0xC
  region dst 0xC
  arg region a
  arg region b
  arg region dst
  fix region:a 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 || any || range 0 0x20000 || near 0x7FFFFFFF 4 || near 0x80000000 4 rep=3,4
  fix region:b 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 || any || range 0 0x20000 || near 0x7FFFFFFF 4 || near 0x80000000 4 rep=3,4

# FUN_0207CA30 VEC_Fx16Add(a, b, dst): halfword sums
fn 0207ca30 ret=none mode=arm inputs=600
  nowork
  region a 8
  region b 8
  region dst 8
  arg region a
  arg region b
  arg region dst
  fix region:a 0 u16 alt oneof 0 1 0xFFFF 0x7FFF 0x8000 0x8001 0x1000 0xF000 || any || any || range 0 0x2000 rep=3,2
  fix region:b 0 u16 alt oneof 0 1 0xFFFF 0x7FFF 0x8000 0x8001 0x1000 0xF000 || any || any || range 0 0x2000 rep=3,2

# FUN_0207CA68 VEC_DotProduct(a, b): ((a . b) + 0x800) >> 12 on 64 bits
fn 0207ca68 ret=u32 mode=arm inputs=1500
  nowork
  region a 0xC
  region b 0xC
  arg region a
  arg region b
  fix region:a 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 || any || any || range 0 0x20000 || near 0x1000 0x1000 || near 0xFFFFF000 0x1000 || mask 0xFFFFFF rep=3,4
  fix region:b 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 || any || any || range 0 0x20000 || near 0x1000 0x1000 || near 0xFFFFF000 0x1000 || mask 0xFFFFFF rep=3,4

# FUN_0207CAA4 VEC_Fx16DotProduct(a, b)
fn 0207caa4 ret=u32 mode=arm inputs=1500
  nowork
  region a 8
  region b 8
  arg region a
  arg region b
  fix region:a 0 u16 alt oneof 0 1 0xFFFF 0x7FFF 0x8000 0x8001 0x1000 0xF000 0xFFF 0x7F8 || any || any || range 0 0x2000 rep=3,2
  fix region:b 0 u16 alt oneof 0 1 0xFFFF 0x7FFF 0x8000 0x8001 0x1000 0xF000 0xFFF 0x7F8 || any || any || range 0 0x2000 rep=3,2

# FUN_0207CAE8 VEC_CrossProduct(a, b, dst)
fn 0207cae8 ret=none mode=arm inputs=1500
  nowork
  region a 0xC
  region b 0xC
  region dst 0xC
  arg region a
  arg region b
  arg region dst
  fix region:a 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 || any || any || range 0 0x20000 || near 0x1000 0x1000 || near 0xFFFFF000 0x1000 || mask 0xFFFFFF rep=3,4
  fix region:b 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 || any || any || range 0 0x20000 || near 0x1000 0x1000 || near 0xFFFFF000 0x1000 || mask 0xFFFFFF rep=3,4

# FUN_0207CB68 VEC_Fx16CrossProduct(a, b, dst): 32-bit products, difference may wrap
fn 0207cb68 ret=none mode=arm inputs=1500
  nowork
  region a 8
  region b 8
  region dst 8
  arg region a
  arg region b
  arg region dst
  fix region:a 0 u16 alt oneof 0 1 0xFFFF 0x7FFF 0x8000 0x8001 0x1000 0xF000 0xFFF 0x7F8 || any || any || range 0 0x2000 rep=3,2
  fix region:b 0 u16 alt oneof 0 1 0xFFFF 0x7FFF 0x8000 0x8001 0x1000 0xF000 0xFFF 0x7F8 || any || any || range 0 0x2000 rep=3,2

# FUN_0207CEE4 VEC_MultAdd(fx32 k, const VecFx32 *a, const VecFx32 *b, VecFx32 *dst): k * a + b
fn 0207cee4 ret=none mode=arm inputs=1500
  nowork
  region a 0xC
  region b 0xC
  region dst 0xC
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0x40000000 || any || any || range 0 0x20000 || near 0x1000 0x1000 || mask 0xFFFFFF
  arg region a
  arg region b
  arg region dst
  fix region:a 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 || any || any || range 0 0x20000 || near 0x1000 0x1000 || near 0xFFFFF000 0x1000 || mask 0xFFFFFF rep=3,4
  fix region:b 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0x40000000 || any || any || range 0 0x20000 || near 0x7FFFFFFF 4 || near 0x80000000 4 rep=3,4

# FUN_0207D184 FX_AcosIdx(fx32 c): binary search of the cosine column of FX_SinCosTable_ (0x020A1B38); the compare is on the low 16 bits,
# the half-table choice on the sign of the full word
fn 0207d184 ret=u32 mode=arm inputs=2500
  nowork
  arg u32 alt range 0 0x1000 || range 0xFFFFF000 0xFFFFFFFF || oneof 0 1 0xFFFFFFFF 0x1000 0xFFFFF000 0xFFF 0xFFFFF001 0x7FFF 0x8000 0xFFFF8000 0xFFFF7FFF 0x10000 0x7FFFFFFF 0x80000000 0x80000001 0xFFFF0000 0x8000FFFF 0x00018000 || any || mask 0xFFFF || near 0x1000 0x40 || near 0xFFFFF000 0x40 || near 0 0x40 || near 0xFFFFFFFF 0x40 || near 0x800 0x40 || near 0xFFFFF800 0x40

# FUN_0207C768 FX_Div(fx32 a, fx32 b): (a << 12) / b by the 64/32 divider (b = 0: the unit answers -1 / +1)
fn 0207c768 ret=u32 mode=arm inputs=3000
  nowork
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4

# FUN_0207C778 FX_Inv(fx32 x): 1.0 / x
fn 0207c778 ret=u32 mode=arm inputs=3000
  nowork
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4

# FUN_0207C7B4 FX_Sqrt(fx32 x): sqrt unit on x << 12
fn 0207c7b4 ret=u32 mode=arm inputs=3000
  nowork
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4

# FUN_0207C7E8 FX_InvSqrt(fx32 x): 1.0 / sqrt(x)
fn 0207c7e8 ret=u32 mode=arm inputs=3000
  nowork
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4

# FUN_0207C958 CP_Divide32 quotient (a, b): 32/32 divider, mode 0
fn 0207c958 ret=u32 mode=arm inputs=3000
  nowork
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4

# FUN_0207C994 CP_Divide32 remainder (a, b)
fn 0207c994 ret=u32 mode=arm inputs=3000
  nowork
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4

# FUN_0207CBD0 VEC_Mag(const VecFx32 *v): sqrt of the 64-bit square sum
fn 0207cbd0 ret=u32 mode=arm inputs=3000
  nowork
  region v 0xC
  arg region v
  fix region:v 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4 rep=3,4

# FUN_0207CC30 VEC_Fx16Mag(const VecFx16 *v)
fn 0207cc30 ret=u32 mode=arm inputs=3000
  nowork
  region v 8
  arg region v
  fix region:v 0 u16 alt oneof 0 1 0xFFFF 0x7FFF 0x8000 0x8001 0x1000 0xF000 0x800 0xF800 2 3 0xFFFE 0x1001 0xEFFF 0xFF || any || range 0 0x2000 || range 0xE000 0xFFFF || near 0x1000 4 || near 0xF000 4 || near 0x7FFF 4 || near 0x8000 4 rep=3,2

# FUN_0207CCA4 VEC_Normalize(const VecFx32 *v, VecFx32 *dst)
fn 0207cca4 ret=none mode=arm inputs=3000
  nowork
  region v 0xC
  region dst 0xC
  arg region v
  arg region dst
  fix region:v 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4 rep=3,4

# FUN_0207CDB8 VEC_Fx16Normalize(const VecFx16 *v, VecFx16 *dst)
fn 0207cdb8 ret=none mode=arm inputs=3000
  nowork
  region v 8
  region dst 8
  arg region v
  arg region dst
  fix region:v 0 u16 alt oneof 0 1 0xFFFF 0x7FFF 0x8000 0x8001 0x1000 0xF000 0x800 0xF800 2 3 0xFFFE 0x1001 0xEFFF 0xFF || any || range 0 0x2000 || range 0xE000 0xFFFF || near 0x1000 4 || near 0xF000 4 || near 0x7FFF 4 || near 0x8000 4 rep=3,2

# FUN_0207CF3C VEC_Distance(const VecFx32 *a, const VecFx32 *b)
fn 0207cf3c ret=u32 mode=arm inputs=3000
  nowork
  region a 0xC
  region b 0xC
  arg region a
  arg region b
  fix region:a 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4 rep=3,4
  fix region:b 0 u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x80000000 0x80000001 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFFF 4 || near 0x80000000 4 rep=3,4

# FUN_0207CFB8 FX_Atan2Idx(fx32 y, fx32 x): table index (u16); neither operand is i32::MIN
fn 0207cfb8 ret=u16 mode=arm inputs=3000
  nowork
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFF8 4 || near 0x80000010 4
  arg u32 alt oneof 0 1 0xFFFFFFFF 0x7FFFFFFF 0x1000 0xFFFFF000 0x800 0xFFFFF800 0x10000 0xFFFF0000 0x40000000 0xC0000000 2 3 0xFFFFFFFE 0x1001 0xFFFFEFFF 0x7FFF 0x8000 0xFFFF8000 0xFFFF || any || range 0 0x1FFFF || range 0xFFFE0001 0xFFFFFFFF || mask 0xFFFFFF || near 0x1000 4 || near 0xFFFFF000 4 || near 0x7FFFFFF8 4 || near 0x80000010 4
