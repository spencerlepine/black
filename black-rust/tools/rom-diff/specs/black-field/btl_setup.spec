# fndiff spec: btl_setup.c (F0-spec), main ARM9 Thumb code 0x02013090..0x02013A00 (T1 unit unk_02013270 and the start of
# unk_02013980).  Grammar: svflow_specs/README.md (generic mode).  Run: tools/check-fn.sh black-field btl_setup --per-function
#
# BATTLE SETUP (0xF0 bytes, "btl_setup.c", FUN_02013090 allocates and clears it; the harness buffer is 0x140 bytes, the extra bytes at
# +0xF0 serve as the block the allocator stub returns):
#   +0x00 u32 kind (0 wild, 1 trainer, 2 = ?, 3 = multi), +0x04 u32 (id passed by the caller), +0x08 .. +0x17 16-byte config copied from
#   the caller (FUN_02013220 default: +0xC = 10), +0x18 u16 / +0x1A u16 (message/bgm ids, set by the builders: 0x468/0x47D, 0x46B/0x47D, ...),
#   +0x1C u32, +0x20 u8, +0x21 u8, +0x22 u8 (number of trainers: 4 = double, 3 = ...), +0x23 u8 (1 once FUN_020138C4 ran),
#   +0x24 + 4*i (i < 4): party (FUN_0201A920 PARTY objects), +0x34 + 4*i (i = 1..3): 4-byte objects (FUN_0200846C), +0x48 + 4*i: trainer
#   data blocks (0x28 bytes, FUN_02013234: +0x14 = StrBuf, +0x18 and +0x20 = two sub records), +0x34 -> game data block, +0x58 u32 (game data),
#   +0x5C, +0x60, +0x64, +0x68, +0x6C, +0x70 u32 (copied from game data accessors), +0x74/+0x76 u16 (minutes * 60), +0x78 .. +0x7E bytes (7),
#   +0x7F u8 (= 1), +0x80 u8, +0x82 u16 FLAGS (FUN_02013144 ors a mask in, FUN_02013158 tests; 0x40 = party full and the save has no PC space?,
#   0x02 = event flag 0x96D, 0x400/0x800 = battle type from the game data), +0x84 u16 (0xFFFF), +0x8C u32 (1), +0x94 -> 0xC00-byte block,
#   +0xC1 u8 (bits 0 and 1 cleared), +0xC2 .. +0xC7 u8 x6.
# Event flag 0x96D (tested in FUN_02013360 through FUN_020142E8, see event_work.spec) sets flag mask 2.
# Callees outside this file are stubs: the allocator FUN_02030734 returns the address p + 0xF0 (inside the harness buffer); the party
# object functions (FUN_0201A920 etc.) are black-storage code; the game-data accessors are game_data.spec code (stubbed here).
area btl_setup
set field

# FUN_02013090(heap): p = alloc(heap, 0xF0, 0, "btl_setup.c", 0x31); FUN_020130C4(p); return p
fn 02013090 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x140
  arg u32 any
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v

# FUN_020130B4(p): FUN_020130DC(p); free(p)
fn 020130b4 ret=none inputs=150
  nowork
  arg buf 0x140
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4
  fix arg0 0x94 u32 alt const 0 || any
  stub 020307b0 ret=none args=v seq
  stub 02045808 ret=none args=v

# FUN_020130C4(p): zero 0xF0 bytes, then FUN_02013220(p + 8)
fn 020130c4 ret=none inputs=100
  nowork
  arg buf 0x140

# FUN_020130DC(p): for i < 4: free(+0x24+4i) if set; FUN_02013270(+0x48+4i) if set; i != 0 and +0x34+4i set: free it;
# then free(+0x94) if set; zero 0xF0 bytes
fn 020130dc ret=none inputs=150
  nowork
  arg buf 0x140
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4
  fix arg0 0x94 u32 alt const 0 || any
  stub 020307b0 ret=none args=v seq
  stub 02045808 ret=none args=v

# FUN_02013128(p, x, i): tail call FUN_0201AC2C(x, *(p + 0x24 + 4i))
fn 02013128 ret=none inputs=100
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 range 0 3
  stub 0201ac2c ret=none args=v,v

# FUN_0201313C(p, i): u32 at +0x24 + 4i
fn 0201313c ret=u32 inputs=100
  nowork
  arg buf 0x140
  arg u32 range 0 3

# FUN_02013144(p, mask): u16 +0x82 |= mask (low 16 bits)
fn 02013144 ret=none inputs=100
  nowork
  arg buf 0x140
  arg u32 any

# FUN_02013158(p, mask): u16 +0x82 & mask
fn 02013158 ret=u32 inputs=100
  nowork
  arg buf 0x140
  arg u32 any

# FUN_02013160(p, x, y): u16 +0x76 = 60 * FUN_0200AE54(x, 0x10); u16 +0x74 = 60 * FUN_0200AE54(x, 0xF); FUN_0200B040(x, p + 0x78);
# kind = FUN_0200AE54(x, 0x12): 1 -> +0x82 |= 0x800; 2 -> |= 0x400; 3 -> |= 0x400 | 0x800 (0 or > 3: nothing);
# for i < 4: if +0x24 + 4i: FUN_0201A30C(x, party, y), FUN_0201A2A8(x, party)
fn 02013160 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  stub 0200ae54 ret=pick:0/1/2/3/5/9 args=v,v
  stub 0200b040 ret=none args=v,p
  stub 0201a30c ret=none args=v,v,v
  stub 0201a2a8 ret=none args=v,v

# FUN_0201320C(q): +0 = 9, +4 = 0, u8 +8 = 0, u16 +0xA = 0, u8 +0xC = 0, u8 +0xD = 0, u8 +9 = 0
fn 0201320c ret=none inputs=100
  nowork
  arg buf 0x20

# FUN_02013220(q): +0 = 0, +4 = 0, u8 +9 = 0, u8 +8 = 0, u16 +0xA = 0, u8 +0xC = 10, u8 +0xD = 0
fn 02013220 ret=none inputs=100
  nowork
  arg buf 0x20

# FUN_02013234(heap): q = alloc(heap, 0x28, 1, "btl_setup.c", 0xEF); +0x14 = FUN_020457B0(0x10, heap); +0 = 0; FUN_0202428C(q + 0x18), FUN_0202428C(q + 0x20)
fn 02013234 ret=ofs:arg1 inputs=100
  nowork
  arg u32 any
  arg buf 0x60
  stub 02030734 ret=ptr:arg1:0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v

# FUN_02013270(q): FUN_02045808(+0x14); free(q)
fn 02013270 ret=none inputs=100
  nowork
  arg buf 0x40
  stub 02045808 ret=none args=v
  stub 020307b0 ret=none args=v

# FUN_02013284(p, x, i, pq, y, heap): if *pq == 0: *pq = FUN_0201A920(heap); slot = p + 0x48 + 4i; if *slot == 0: *slot = FUN_02013234(heap);
# if y != 0: FUN_0202A440(y, *slot, heap), FUN_0202A4B4(y, *pq, heap), FUN_0216AAE4(x, *pq, FUN_02012984(x)); t = FUN_0202A3FC(*(*slot + 4)); u16 +0x1A = f(t):
# t = 2, 3 -> 0x47E; 4, 0xB -> 0x480; 6, 9 -> 0x491; 7, 8 -> 0x47F; else (0, 1, 5, 10, > 0xB) -> 0x47D
fn 02013284 ret=none inputs=300
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 range 0 3
  arg u32 oneof 0x02E10028 0x02E1002C 0x02E10030
  arg u32 alt const 0 || any
  arg u32 any
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  stub 0201a920 ret=u32 args=v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v

# FUN_0201333C(p, x, heap): FUN_020127A4(x); +0x24 = FUN_0201A920(heap); FUN_0201AC2C(FUN_02012934(x), +0x24)
fn 0201333c ret=none inputs=100
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v

# FUN_02013360(p, x, q): init the setup around the game data x: u16 +0x84 = 0xFFFF, bytes +0x20..+0x23 = 0, +0x94 = 0, +0x1C = 0, +0xC1 bits 0,1 cleared, +0x58 = x,
# +0x24/+0x34 (4 each) = 0, +0xC2..+0xC7 = 0, +0x7F = 1, +0x78..+0x7E = 0, +0x34 = FUN_02012944(x), +0x60 = FUN_0201292C(x), +0x64 = FUN_02012924(x),
# +0x68 = FUN_0200CA1C(x), +0x70 = FUN_02012F20(x), +0x6C = FUN_02012F2C(x), +0x5C = FUN_0200873C(*x), u8 +0x80 = FUN_0200C258(FUN_0200C0E8()),
# copy 0x10 bytes q -> +8, u16 +0x18 = 0x468, +0x1A = 0x47C, +0x8C = 1; if event flag 0x96D (FUN_020142E8(FUN_0201296C(x), 0x96D)): FUN_02013144(p, 2)
fn 02013360 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg buf 0x20
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v

# FUN_0201347C(p, x, src, q, kind, heap): FUN_020130C4(p); FUN_02013360(p, x, q); FUN_0201333C(p, x, heap); +0x28 = FUN_0201A920(heap);
# if src: FUN_0201AC2C(src, +0x28); +4 = kind; +0 = 0; if FUN_0201AA18(+0x24) == FUN_0201AA14(+0x24) and FUN_020076D0(FUN_0201293C(x), &a, &b) == 0: FUN_02013144(p, 0x40)
fn 0201347c ret=none inputs=300
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 alt const 0 || any
  arg buf 0x20
  arg u32 any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 0201aa18 ret=pick:0/1/2 args=v
  stub 0201aa14 ret=pick:0/1/2 args=v
  stub 0201293c ret=u32 args=v
  stub 020076d0 ret=pick:0/1 args=v,p,p

# FUN_020134F4(p, x, kind, q, heap): FUN_020130C4(p); FUN_02013360(p, x, q); FUN_0201333C(p, x, heap); +0 = 1; +4 = kind; u16 +0x1A = 0x47D
fn 020134f4 ret=none inputs=100
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  arg buf 0x20
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v

# FUN_02013528(p, x, a, b, c, d, e, heap): FUN_020130C4(p); local q = {9, 1, 0...}; FUN_02013360(p, x, &q); FUN_0201333C(p, x, heap); +0 = 3; +4 = a;
# +0x1C = c; u8 +0x20 = d, +0x22 = b, +0x21 = e; u16 +0x84 = 0xFF; u16 +0x1A = 0x47D; for i = 1..3: if +0x34+4i == 0: = FUN_0200846C(heap); if +0x24+4i == 0: = FUN_0201A920(heap)
fn 02013528 ret=none inputs=300
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 0200846c ret=u32 args=v
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4

# FUN_02013610(p, x, q, y, heap): FUN_020134F4(p, x, 0, q, heap); FUN_02013284(p, x, 1, p + 0x28, y, heap)
fn 02013610 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg buf 0x20
  arg u32 alt const 0 || any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4

# FUN_02013644(p, x, q, y, heap): FUN_020134F4(p, x, 1, q, heap); FUN_02013284(p, x, 1, p + 0x28, y, heap)
fn 02013644 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg buf 0x20
  arg u32 alt const 0 || any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4

# FUN_02013678(p, x, q, y, heap): FUN_020134F4(p, x, 2, q, heap); FUN_02013284(p, x, 1, p + 0x28, y, heap)
fn 02013678 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg buf 0x20
  arg u32 alt const 0 || any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4

# FUN_020136AC(p, x, q, y, heap): FUN_020134F4(p, x, 3, q, heap); FUN_02013284(p, x, 1, p + 0x28, y, heap)
fn 020136ac ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg buf 0x20
  arg u32 alt const 0 || any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4

# FUN_020136E0(p, x, q, y, z, heap): FUN_020134F4(p, x, 1, q, heap); FUN_02013284(p, x, 1, p + 0x28, y, heap); FUN_02013284(p, x, 3, p + 0x30, z, heap); u8 +0x22 = 4
fn 020136e0 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg buf 0x20
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4

# FUN_02013734(p, x, q, y2, y1, y3, heap): FUN_020134F4(p, x, 1, q, heap); FUN_02013284(p, x, 2, p + 0x2C, y2, heap); (.., 1, p + 0x28, y1, heap);
# (.., 3, p + 0x30, y3, heap); u8 +0x22 = 3
fn 02013734 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg buf 0x20
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  arg u32 alt const 0 || any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4

# FUN_02013790(p, x, a, b, heap): FUN_02013528(p, x, 0, 0, a, b, FUN_0203D1A4(a), heap, a)
fn 02013790 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4
  stub 0203d1a4 ret=u8 args=v

# FUN_020137C0(p, x, a, b, heap): FUN_02013528(p, x, 1, 0, a, b, FUN_0203D1A4(a), heap, a)
fn 020137c0 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4
  stub 0203d1a4 ret=u8 args=v

# FUN_02013864(p, x, a, b, heap): FUN_02013528(p, x, 2, 0, a, b, FUN_0203D1A4(a), heap, a)
fn 02013864 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4
  stub 0203d1a4 ret=u8 args=v

# FUN_02013894(p, x, a, b, heap): FUN_02013528(p, x, 3, 0, a, b, FUN_0203D1A4(a), heap, a)
fn 02013894 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4
  stub 0203d1a4 ret=u8 args=v

# FUN_020137F0(p, x, a, b, c, heap): FUN_02013528(p, x, 1, 1, a, b, c, heap, b)
fn 020137f0 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4

# FUN_02013810(p, x, a, b, c, y1, y2, ..., heap): FUN_02013528(p, x, 1, 2, a, b, c, heap, b); +0 = 1; FUN_02013284(p, x, 1, p + 0x28, y1, heap);
# (.., 3, p + 0x30, y2, heap)
fn 02013810 ret=none inputs=200
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012944 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0200ca1c ret=u32 args=v
  stub 02012f20 ret=u32 args=v
  stub 02012f2c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200c0e8 ret=u32 args=
  stub 0200873c ret=u32 args=v
  stub 0200c258 ret=u8 args=v
  stub 0201296c ret=u32 args=v
  stub 020142e8 ret=pick:0/1 args=v,v
  stub 020127a4 ret=u32 args=v
  stub 0201a920 ret=u32 args=v
  stub 02012934 ret=u32 args=v
  stub 0201ac2c ret=none args=v,v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v
  stub 0202a440 ret=none args=v,v,v
  stub 0202a4b4 ret=none args=v,v,v
  stub 02012984 ret=u8 args=v
  stub 0216aae4 ret=none args=v,v,v
  stub 0202a3fc ret=pick:0/1/2/3/4/5/6/7/8/9/10/11/12 args=v
  stub 0200846c ret=u32 args=v
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x28 u32 alt const 0 || any
  fix arg0 0x2c u32 alt const 0 || any
  fix arg0 0x30 u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4

# FUN_020138C4(p, w, heap): zero 0xF0; for i < 4: +0x24+4i = FUN_0201A920(heap), +0x34+4i = FUN_0200846C(heap), +0x48+4i = FUN_02013234(heap);
# +0x58 = w; +0x5C = FUN_020082EC(heap); FUN_02008314(FUN_0200873C(*w), +0x5C); +0x68 = FUN_0200CA1C(w); u8 +0x23 = 1; FUN_02013980(p, heap)
fn 020138c4 ret=none inputs=150
  nowork
  arg buf 0x140
  arg u32 any
  arg u32 any
  stub 0201a920 ret=u32 args=v
  stub 0200846c ret=u32 args=v
  stub 02012ebc ret=u32 args=v
  stub 0200873c ret=u32 args=v
  stub 020082ec ret=u32 args=v
  stub 02008314 ret=none args=v,v
  stub 0200ca1c ret=u32 args=v
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
  stub 020457b0 ret=u32 args=v,v
  stub 0202428c ret=none args=v

# FUN_02013930(p): if +0x5C: free it, +0x5C = 0; FUN_020130DC(p)
fn 02013930 ret=none inputs=150
  nowork
  arg buf 0x140
  fix arg0 0x5C u32 alt const 0 || any
  fix arg0 0x24 u32 alt const 0 || any rep=4,4
  fix arg0 0x48 u32 alt const 0 || addrof arg0 0xF0 rep=4,4
  fix arg0 0x34 u32 alt const 0 || any rep=4,4
  fix arg0 0x94 u32 alt const 0 || any
  stub 020307b0 ret=none args=v seq
  stub 02045808 ret=none args=v

# FUN_0201394C(p): if +0 == 1: +0 = 2; t = FUN_0202A3FC(*(*(p + 0x4C) + 4)); t != 0xB: u16 +0x18 = 0x46B, +0x1A = 0x47D; else +0x18 = 0x470, +0x1A = 0x480
fn 0201394c ret=none inputs=200
  nowork
  arg buf 0x140
  fix arg0 0 u32 alt const 1 || any
  fix arg0 0x4C u32 addrof arg0 0xF0
  stub 0202a3fc ret=pick:0/1/11/11 args=v

# FUN_02013980(p, heap): if +0x94 == 0: +0x94 = alloc(heap, 0xC00, 0, "btl_setup.c", 0x39A)
fn 02013980 ret=none inputs=100
  nowork
  arg buf 0x140
  arg u32 any
  fix arg0 0x94 u32 alt const 0 || any
  stub 02030734 ret=ptr:arg0:0xf0:1 args=v,v,v,v,v
