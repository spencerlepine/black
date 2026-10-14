# fndiff spec (S5): save block 52 (money, badges, counters; 0xEC bytes), an unnamed file of the main
# ARM9 at 0x0200C054..0x0200C3F4. Grammar: svflow_specs/README.md (generic mode).
# Run: tools/check-fn.sh black-storage misc --per-function
#
# Layout: +0 u32 money, +4 u32 badges, +8 u16, +0xA bits, +0xC [8], +0x14 u32 (28 + 4 bits),
# +0x18 / +0x1C u32 counters, +0x20 / +0x32 u16[9] names, +0x44 / +0x45 / +0x46 bytes, +0x47 [3],
# +0x4A u16[3], +0x50 u64, +0x58 8 x u16[6], +0xB8 u32[8], +0xDC u32, +0xE0 u16.
# The init's message-bank calls (FUN_02045B38 open, FUN_02045EC0 string, FUN_02045A38 string ->
# 9-unit name, FUN_02045808 free, FUN_02045C04 close) are stubbed. Index arguments stay inside
# their arrays. MI_CpuFill8 runs as real code.
area misc
set field

fn 0200c054 ret=u32 inputs=10
  nowork

# sub_0200C058(m): new-game init
fn 0200c058 ret=none inputs=100
  nowork
  arg buf 0xec
  stub 02045b38 ret=u32 args=v,v,v,v
  stub 02045ec0 ret=u32 args=v,v
  stub 02045a38 ret=none args=v,out:0x12,v
  stub 02045808 ret=none args=v
  stub 02045c04 ret=none args=v

# unlabeled 0x0200C0FC(m, i, v): +0xC + i = v
fn 0200c0fc ret=none
  nowork
  arg buf 0xec
  arg u32 range 0 7
  arg u32 any

# unlabeled 0x0200C104(m, i)
fn 0200c104 ret=u32
  nowork
  arg buf 0xec
  arg u32 range 0 7

# unlabeled 0x0200C10C(m): clear +0xC..+0x13
fn 0200c10c ret=none inputs=50
  nowork
  arg buf 0xec

# unlabeled 0x0200C11C(m, a, b, c, d)
fn 0200c11c ret=none inputs=400
  nowork
  arg buf 0xec
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt oneof 0 1 2 || any

# unlabeled 0x0200C164(m, &a, &b, &c, &d)
fn 0200c164 ret=none
  nowork
  arg buf 0xec
  arg buf 4
  arg buf 4
  arg buf 4
  arg buf 4

fn 0200c184 ret=u32
  nowork
  arg buf 0xec

fn 0200c18c ret=none
  nowork
  arg buf 0xec
  arg u32 any

fn 0200c1a4 ret=u32
  nowork
  arg buf 0xec

fn 0200c1b0 ret=none
  nowork
  arg buf 0xec
  arg u32 any

fn 0200c1c4 ret=u32
  nowork
  arg buf 0xec

fn 0200c1c8 ret=u32 inputs=300
  nowork
  arg buf 0xec
  fix arg0 0x18 u32 alt near 99999 2 || any

fn 0200c1dc ret=u32
  nowork
  arg buf 0xec

fn 0200c1e0 ret=u32 inputs=300
  nowork
  arg buf 0xec
  fix arg0 0x1c u32 alt near 99999 2 || any

fn 0200c1f4 ret=ofs:arg0 inputs=20
  nowork
  arg buf 0xec

fn 0200c20c ret=ofs:arg0 inputs=20
  nowork
  arg buf 0xec

fn 0200c224 ret=u32
  nowork
  arg buf 0xec

fn 0200c22c ret=none
  nowork
  arg buf 0xec
  arg u32 any

# FUN_0200C234(m, n): badge n
fn 0200c234 ret=bool inputs=400
  nowork
  arg buf 0xec
  arg u32 alt range 0 8 || range 0 40 || any

# unlabeled 0x0200C248(m, n): give badge n
fn 0200c248 ret=none inputs=400
  nowork
  arg buf 0xec
  arg u32 alt range 0 8 || range 0 40 || any

fn 0200c258 ret=u32
  nowork
  arg buf 0xec

fn 0200c274 ret=u32
  nowork
  arg buf 0xec

# FUN_0200C278(m, n): money += n (cap 9,999,999)
fn 0200c278 ret=u32 inputs=600
  nowork
  arg buf 0xec
  arg u32 alt range 0 1000 || near 9999999 2 || oneof 0xffffffff 0x80000000 || any
  fix arg0 0 u32 alt near 9999999 3 || range 0 10000000 || any

# FUN_0200C29C(m, n): money -= n (0 when short)
fn 0200c29c ret=u32 inputs=600
  nowork
  arg buf 0xec
  arg u32 alt range 0 1000 || near 9999999 2 || any
  fix arg0 0 u32 alt range 0 1000 || range 0 10000000 || any

# unlabeled 0x0200C2B0(m, i, src): record i = src[0..6]
fn 0200c2b0 ret=none
  nowork
  arg buf 0xec
  arg u32 range 0 7
  arg buf 0xc

# FUN_0200C2D0(m, i, dst)
fn 0200c2d0 ret=none
  nowork
  arg buf 0xec
  arg u32 range 0 7
  arg buf 0xc

# unlabeled 0x0200C2F0(m, i, v): +0x45 |= v << 2i
fn 0200c2f0 ret=none inputs=400
  nowork
  arg buf 0xec
  arg u32 alt range 0 3 || range 0 20 || any
  arg u32 alt range 0 3 || any

fn 0200c308 ret=u32 inputs=400
  nowork
  arg buf 0xec
  arg u32 alt range 0 3 || range 0 20 || any

# unlabeled 0x0200C31C(m)
fn 0200c31c ret=u32
  nowork
  arg buf 0xec

# unlabeled 0x0200C324(m, v)
fn 0200c324 ret=none
  nowork
  arg buf 0xec
  arg u32 any

fn 0200c32c ret=u32
  nowork
  arg buf 0xec

# unlabeled 0x0200C334(m, v)
fn 0200c334 ret=none
  nowork
  arg buf 0xec
  arg u32 any

fn 0200c33c ret=u32
  nowork
  arg buf 0xec
  arg u32 range 0 2

fn 0200c344 ret=none
  nowork
  arg buf 0xec
  arg u32 range 0 2
  arg u32 any

fn 0200c34c ret=u32
  nowork
  arg buf 0xec
  arg u32 range 0 2

fn 0200c358 ret=none
  nowork
  arg buf 0xec
  arg u32 range 0 2
  arg u32 any

# FUN_0200C364(m, i, n): += n, cap 0xFFFF
fn 0200c364 ret=none inputs=400
  nowork
  arg buf 0xec
  arg u32 range 0 2
  arg u32 alt range 0 10 || oneof 0xffffffff 0x10000 || any
  fix arg0 0x4a u16 alt near 0xffff 2 || any rep=3,2

fn 0200c37c ret=u64
  nowork
  arg buf 0xec

fn 0200c384 ret=none
  nowork
  arg buf 0xec
  arg u32 any
  arg u32 any

# FUN_0200C38C(m, i, v): i < 8 (signed)
fn 0200c38c ret=none inputs=300
  nowork
  arg buf 0xec
  arg u32 alt range 0 9 || oneof 0x7fffffff
  arg u32 any

fn 0200c39c ret=u32 inputs=300
  nowork
  arg buf 0xec
  arg u32 alt range 0 9 || oneof 0x7fffffff

fn 0200c3b0 ret=u32
  nowork
  arg buf 0xec

fn 0200c3b8 ret=none
  nowork
  arg buf 0xec
  arg u32 any

# FUN_0200C3C0(m, n): -= n, 0 when short
fn 0200c3c0 ret=none inputs=400
  nowork
  arg buf 0xec
  arg u32 alt range 0 100 || any
  fix arg0 0xe0 u16 alt range 0 100 || any

fn 0200c3e0 ret=u32
  nowork
  arg buf 0xec

fn 0200c3ec ret=none
  nowork
  arg buf 0xec
  arg u32 any
