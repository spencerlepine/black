# fndiff spec: NitroSDK fixed-point math FX / VEC / MTX (Z24), main ARM9 ARM code, functions WITHOUT the divider / sqrt unit.
# Run: tools/check-fn.sh black-core fx --per-function
# The hardware users (FX_Div 0207C768 .. FX_Atan2Idx 0207CFB8, VEC_Mag / Normalize / Distance) are not here: the generic emulator
# leaves the I/O page (REG_DIV* 0x04000280, REG_SQRT* 0x040002B0) as plain memory, so the ROM side cannot compute them.
# They are verified by tools/rom-diff/fx_hw_diff.py (divider and sqrt modelled) against crates/black-core/src/fx/tests.rs.
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
