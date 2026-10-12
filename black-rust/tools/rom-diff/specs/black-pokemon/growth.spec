# fndiff spec: EXP growth curves (ARC 017) and level-from-EXP (P3), main ARM9 code (T3-FINDINGS.md (a)).
# Run: tools/check-fn.sh black-pokemon growth --per-function
#
# The curve cache is the global 0x02146398 (101 x u32 = 0x194 bytes). `ArcLoadMember` (0x02048F4C,
# ARC 0x11, member = curve id) is stubbed with a pointer-only argument: it writes nothing, so the
# cache keeps the table the input generator planted. That table is a `ramp` (non-decreasing running
# sum of draws, up to ~600000 like a real EXP curve), so a lookup can end at every level 1..100.
# The curve id of a species comes from the personal-data routines (area `personal`), which are
# stubbed here: FUN_02019C54 (record pointer), FUN_0201AE1C (field getter), FUN_02019C90 (both).
area growth
set field

# FUN_02019B94(curve, dst): tail call ArcLoadMember(dst, 0x11, curve) (dst = the cache).
fn 02019b94 ret=none
  nowork
  global 02146398 0x194
  arg u32 alt range 0 7 || any
  arg u32 const 0x02146398
  ramp global0 0 101 u32 range 0 12000
  stub 02048f4c ret=none args=p,v,v

# FUN_02019BA4(curve, level): load the curve into the cache and return cache[level] (level 0..100).
fn 02019ba4 ret=u32
  nowork
  global 02146398 0x194
  arg u32 alt range 0 7 || any
  arg u32 alt range 0 100 || oneof 0 1 2 99 100
  ramp global0 0 101 u32 range 0 12000
  stub 02048f4c ret=none args=p,v,v

# FUN_0201850C(species, form, exp): level from EXP. Curve id = personal field 0x17 of the record
# FUN_02019C54 returns; result = (first level in 1..100 whose table entry is above exp) - 1, capped at 100.
fn 0201850c ret=u32 inputs=400
  nowork
  global 02146398 0x194
  arg u16 any
  arg u16 alt const 0 || range 0 3 || any
  arg u32 alt range 0 650000 || range 0 650000 || near 12000 12000 || range 0 5 || range 0 5 || any
  ramp global0 0 101 u32 alt range 0 12000 || range 0 12000 || range 0 3
  stub 02019c54 ret=u32 args=v,v
  stub 0201ae1c ret=mod:8 args=v,v
  stub 02048f4c ret=none args=p,v,v

# FUN_02018584(species, form, level): EXP needed for `level` = cache[level] of the species' curve.
fn 02018584 ret=u32
  nowork
  global 02146398 0x194
  arg u16 any
  arg u16 alt const 0 || range 0 3 || any
  arg u32 alt range 0 100 || oneof 0 1 2 99 100
  ramp global0 0 101 u32 range 0 12000
  stub 02019c90 ret=mod:8 args=v,v,v
  stub 02048f4c ret=none args=p,v,v
