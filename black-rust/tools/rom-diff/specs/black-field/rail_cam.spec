# fndiff spec (F3b): `field_rail.c` part 3 (overlay 21 Thumb code 0x021B9B7C..0x021BB088; F1 row 21.53): the procs of the RAIL DATA's
# proc tables: position procs (ROM 0x021DDDA0: 021B9B7C straight, 021B9C28 / 021B9D60 curves), line-hit procs (ROM 0x021DDDAC:
# 021B9E84 straight, 021B9FC8 stepped; helper 021BAE7C projects a point on a segment), camera procs (ROM 0x021DDDB8: 021BA25C ..
# 021BACA8; orbit helper 021BA7E0, point-camera helper 021BB014). Run: tools/check-fn.sh black-field rail_cam --per-function
#
# Function list: as the `rail` spec; 021B9BE8 (a `bl` target from data) is a label inside 021B9B7C. Rail data lookups that hand out
# records answer from the arena of the RAIL universe (rail.spec header) or the ROM default records 0x021D9A0C / 0x021D9A30 (real ROM
# on both sides). The field camera (fieldmap 0218E124 .. 0218F05C), the `rail` / `rail_move` getters and the NitroSDK vector / fx
# functions are `force` stubs (D3: the camera is driven through hooks); FX_Mul products are inline (real code on both sides); the sine
# table FX_SinCosTable_ (ARM9 0x020A1B38) is a `global` in the blocks that read it (the orbit, angles masked to u16 by the callers).
#
# Conventions (headers of the F5b / F6b / F14 specs): all blocks `nowork`; closure stubs (checked with every function treated as
# ported: no fault, stray write or unstubbed callee); stack locals reach stubs as `in:` / `out:`.
area rail_cam
set field

# FUN_021B9B7C(w, out): position proc straight.
fn 021b9b7c ret=none inputs=60
  nowork
  arg u32 const 0x2E30100
  arg u32 const 0x2E303C0
  arg buf 0x3000
  fix arg2 0x2 u16 alt const 0 || const 1 || any
  fix arg2 0x4 u16 alt const 0 || const 1
  fix arg2 0x8 u32 addrof arg2 0x2000
  fix arg2 0xC u32 range 3 4
  fix arg2 0x14 u32 alt const 0 || const 0x2E30100 || oneof 0x2E32000 0x2E3207C
  fix arg2 0x18 u32 alt const 0 || const 1
  fix arg2 0x1C u32 addrof arg2 0x100
  fix arg2 0x20 u16 alt const 7 || range 0 7
  fix arg2 0x22 u16 alt const 7 || range 0 7
  fix arg2 0x24 u16 alt const 7 || range 0 7
  fix arg2 0x26 u16 alt const 7 || range 0 7
  fix arg2 0x28 u16 alt const 7 || range 0 7
  fix arg2 0x2A u16 alt const 7 || range 0 7
  fix arg2 0x2C u16 alt const 7 || range 0 7
  fix arg2 0x30 u32 addrof arg2 0x1000
  fix arg2 0x34 u32 addrof arg2 0x1400
  fix arg2 0x38 u32 addrof arg2 0x1600
  fix arg2 0x3C u32 addrof arg2 0x1700
  fix arg2 0x40 u32 addrof arg2 0x1800
  fix arg2 0x44 u32 addrof arg2 0x1840
  fix arg2 0x48 u32 addrof arg2 0x1880
  fix arg2 0x4C u32 alt range 0 16 || any
  fix arg2 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg2 0x100 u16 alt const 0 || const 1 || const 2
  fix arg2 0x102 u16 alt const 0 || const 1
  fix arg2 0x104 u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150 0x2E311C0 0x2E31230 0x2E312A0 0x2E31400 0x2E31448 0x2E31490 0x2E314D8 0x2E31520 0x2E31568 0x2E315B0
  fix arg2 0x108 u16 alt const 0 || const 1
  fix arg2 0x10C u32 alt range 0 0x80 || any
  fix arg2 0x110 u32 alt range 0 0x80 || any
  fix arg2 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg2 0x118 u32 alt range 0 0x40 || any
  fix arg2 0x124 u32 addrof arg2 0x20
  fix arg2 0x128 u32 addrof arg2 0x50
  fix arg2 0x12C u8 alt const 0 || const 1
  fix arg2 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg2 0x12E u16 range 0 5
  fix arg2 0x130 u16 range 0 4
  fix arg2 0x134 u8 alt const 0 || const 1
  fix arg2 0x174 u8 alt const 0 || const 1
  fix arg2 0x178 u32 addrof arg2 0x0
  fix arg2 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg2 0x2002 u16 alt const 0 || const 1
  fix arg2 0x2004 u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150 0x2E311C0 0x2E31230 0x2E312A0 0x2E31400 0x2E31448 0x2E31490 0x2E314D8 0x2E31520 0x2E31568 0x2E315B0
  fix arg2 0x2008 u16 alt const 0 || const 1
  fix arg2 0x200C u32 alt range 0 0x80 || any
  fix arg2 0x2010 u32 alt range 0 0x80 || any
  fix arg2 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg2 0x2018 u32 alt range 0 0x40 || any
  fix arg2 0x2024 u32 addrof arg2 0x20
  fix arg2 0x2028 u32 addrof arg2 0x50
  fix arg2 0x202C u8 alt const 0 || const 1
  fix arg2 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg2 0x202E u16 range 0 5
  fix arg2 0x2030 u16 range 0 4
  fix arg2 0x2034 u8 alt const 0 || const 1
  fix arg2 0x2074 u8 alt const 0 || const 1
  fix arg2 0x2078 u32 addrof arg2 0x0
  fix arg2 0x207C u16 alt const 0 || const 1 || const 2
  fix arg2 0x207E u16 alt const 0 || const 1
  fix arg2 0x2080 u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150 0x2E311C0 0x2E31230 0x2E312A0 0x2E31400 0x2E31448 0x2E31490 0x2E314D8 0x2E31520 0x2E31568 0x2E315B0
  fix arg2 0x2084 u16 alt const 0 || const 1
  fix arg2 0x2088 u32 alt range 0 0x80 || any
  fix arg2 0x208C u32 alt range 0 0x80 || any
  fix arg2 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg2 0x2094 u32 alt range 0 0x40 || any
  fix arg2 0x20A0 u32 addrof arg2 0x20
  fix arg2 0x20A4 u32 addrof arg2 0x50
  fix arg2 0x20A8 u8 alt const 0 || const 1
  fix arg2 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg2 0x20AA u16 range 0 5
  fix arg2 0x20AC u16 range 0 4
  fix arg2 0x20B0 u8 alt const 0 || const 1
  fix arg2 0x20F0 u8 alt const 0 || const 1
  fix arg2 0x20F4 u32 addrof arg2 0x0
  fix arg2 0x20FA u16 const 0
  fix arg2 0x25C u16 range 0 3
  fix arg2 0x25E u8 alt const 0 || const 1
  fix arg2 0x274 u8 alt const 0 || const 1
  fix arg2 0x276 u8 range 0 4
  fix arg2 0x278 u32 addrof arg2 0x0
  fix arg2 0x1000 u32 alt const 0 || const 6
  fix arg2 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg2 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1070 u32 alt const 1 || const 0
  fix arg2 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg2 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg2 0x10E0 u32 alt const 2 || const 1
  fix arg2 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg2 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1150 u32 alt const 3 || const 2
  fix arg2 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg2 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x11C0 u32 alt const 4 || const 3
  fix arg2 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg2 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1230 u32 alt const 5 || const 4
  fix arg2 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg2 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x12A0 u32 alt const 6 || const 5
  fix arg2 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg2 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1400 u32 const 0
  fix arg2 0x1404 u32 const 1
  fix arg2 0x1408 u32 alt range 1 4 || range 0 5
  fix arg2 0x140C u32 alt range 0 6 || const 9
  fix arg2 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1414 u32 alt range 0 8 || any
  fix arg2 0x1448 u32 const 1
  fix arg2 0x144C u32 const 2
  fix arg2 0x1450 u32 alt range 1 4 || range 0 5
  fix arg2 0x1454 u32 alt range 0 6 || const 9
  fix arg2 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x145C u32 alt range 0 8 || any
  fix arg2 0x1490 u32 const 2
  fix arg2 0x1494 u32 const 3
  fix arg2 0x1498 u32 alt range 1 4 || range 0 5
  fix arg2 0x149C u32 alt range 0 6 || const 9
  fix arg2 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x14A4 u32 alt range 0 8 || any
  fix arg2 0x14D8 u32 const 3
  fix arg2 0x14DC u32 const 4
  fix arg2 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg2 0x14E4 u32 alt range 0 6 || const 9
  fix arg2 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x14EC u32 alt range 0 8 || any
  fix arg2 0x1520 u32 const 4
  fix arg2 0x1524 u32 const 5
  fix arg2 0x1528 u32 alt range 1 4 || range 0 5
  fix arg2 0x152C u32 alt range 0 6 || const 9
  fix arg2 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1534 u32 alt range 0 8 || any
  fix arg2 0x1568 u32 const 5
  fix arg2 0x156C u32 const 6
  fix arg2 0x1570 u32 alt range 1 4 || range 0 5
  fix arg2 0x1574 u32 alt range 0 6 || const 9
  fix arg2 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x157C u32 alt range 0 8 || any
  fix arg2 0x15B0 u32 const 6
  fix arg2 0x15B4 u32 const 0
  fix arg2 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg2 0x15BC u32 alt range 0 6 || const 9
  fix arg2 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x15C4 u32 alt range 0 8 || any
  fix arg2 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg2 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg2 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg2 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg2 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg2 0x340 u16 alt range 0 3 || range 0 6
  fix arg2 0x342 u8 alt const 0 || const 1
  fix arg2 0x343 u8 range 0 4
  fix arg2 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg2 0x346 u16 alt range 0 8 || any
  fix arg2 0x25C u16 alt range 0 3 || range 0 6
  fix arg2 0x25E u8 alt const 0 || const 1
  fix arg2 0x25F u8 range 0 4
  fix arg2 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg2 0x262 u16 alt range 0 8 || any
  fix arg2 0x15C u16 alt range 0 3 || range 0 6
  fix arg2 0x15E u8 alt const 0 || const 1
  fix arg2 0x15F u8 range 0 4
  fix arg2 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg2 0x162 u16 alt range 0 8 || any
  stub 021b82f4 force ret=u32 args=v  # rail: POINT / LINE of the RAIL WORK (w)
  stub 021b82f8 force ret=pick+u32:0/0x10/0x40 args=v  # rail: position along the line (w)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 021b833c force ret=pick:0x2E31000/0x2E31070/0x2E310E0/0x2E31150/0x2E311C0/0x2E31230/0x2E312A0 args=v  # rail: start point of the line (w)
  stub 021b834c force ret=pick:0x2E31000/0x2E31070/0x2E310E0/0x2E31150/0x2E311C0/0x2E31230/0x2E312A0 args=v  # rail: end point of the line (w)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cee4 force ret=none args=v,in:12,in:12,out:12  # VEC_MultAdd (fx, a, b, &out)
  stub 021b8300 force ret=pick+u32:0/0x10/0xFFFFFFF0 args=v  # rail: lateral offset (w)
  stub 021b8308 force ret=pick+u32:0x1000/0x2000 args=v  # rail: unit (w)
  stub 0207cae8 force ret=none args=in:12,in:12,out:12  # VEC_CrossProduct (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)

# FUN_021B9C28(w, out): position proc curve.
fn 021b9c28 ret=none inputs=200
  nowork
  arg u32 const 0x2E30100
  arg u32 const 0x2E303C0
  arg buf 0x3000
  fix arg2 0x2 u16 alt const 0 || const 1 || any
  fix arg2 0x4 u16 alt const 0 || const 1
  fix arg2 0x8 u32 addrof arg2 0x2000
  fix arg2 0xC u32 range 3 4
  fix arg2 0x14 u32 alt const 0 || const 0x2E30100 || oneof 0x2E32000 0x2E3207C
  fix arg2 0x18 u32 alt const 0 || const 1
  fix arg2 0x1C u32 addrof arg2 0x100
  fix arg2 0x20 u16 alt const 7 || range 0 7
  fix arg2 0x22 u16 alt const 7 || range 0 7
  fix arg2 0x24 u16 alt const 7 || range 0 7
  fix arg2 0x26 u16 alt const 7 || range 0 7
  fix arg2 0x28 u16 alt const 7 || range 0 7
  fix arg2 0x2A u16 alt const 7 || range 0 7
  fix arg2 0x2C u16 alt const 7 || range 0 7
  fix arg2 0x30 u32 addrof arg2 0x1000
  fix arg2 0x34 u32 addrof arg2 0x1400
  fix arg2 0x38 u32 addrof arg2 0x1600
  fix arg2 0x3C u32 addrof arg2 0x1700
  fix arg2 0x40 u32 addrof arg2 0x1800
  fix arg2 0x44 u32 addrof arg2 0x1840
  fix arg2 0x48 u32 addrof arg2 0x1880
  fix arg2 0x4C u32 alt range 0 16 || any
  fix arg2 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg2 0x100 u16 alt const 0 || const 1 || const 2
  fix arg2 0x102 u16 alt const 0 || const 1
  fix arg2 0x104 u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150 0x2E311C0 0x2E31230 0x2E312A0 0x2E31400 0x2E31448 0x2E31490 0x2E314D8 0x2E31520 0x2E31568 0x2E315B0
  fix arg2 0x108 u16 alt const 0 || const 1
  fix arg2 0x10C u32 alt range 0 0x80 || any
  fix arg2 0x110 u32 alt range 0 0x80 || any
  fix arg2 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg2 0x118 u32 alt range 0 0x40 || any
  fix arg2 0x124 u32 addrof arg2 0x20
  fix arg2 0x128 u32 addrof arg2 0x50
  fix arg2 0x12C u8 alt const 0 || const 1
  fix arg2 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg2 0x12E u16 range 0 5
  fix arg2 0x130 u16 range 0 4
  fix arg2 0x134 u8 alt const 0 || const 1
  fix arg2 0x174 u8 alt const 0 || const 1
  fix arg2 0x178 u32 addrof arg2 0x0
  fix arg2 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg2 0x2002 u16 alt const 0 || const 1
  fix arg2 0x2004 u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150 0x2E311C0 0x2E31230 0x2E312A0 0x2E31400 0x2E31448 0x2E31490 0x2E314D8 0x2E31520 0x2E31568 0x2E315B0
  fix arg2 0x2008 u16 alt const 0 || const 1
  fix arg2 0x200C u32 alt range 0 0x80 || any
  fix arg2 0x2010 u32 alt range 0 0x80 || any
  fix arg2 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg2 0x2018 u32 alt range 0 0x40 || any
  fix arg2 0x2024 u32 addrof arg2 0x20
  fix arg2 0x2028 u32 addrof arg2 0x50
  fix arg2 0x202C u8 alt const 0 || const 1
  fix arg2 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg2 0x202E u16 range 0 5
  fix arg2 0x2030 u16 range 0 4
  fix arg2 0x2034 u8 alt const 0 || const 1
  fix arg2 0x2074 u8 alt const 0 || const 1
  fix arg2 0x2078 u32 addrof arg2 0x0
  fix arg2 0x207C u16 alt const 0 || const 1 || const 2
  fix arg2 0x207E u16 alt const 0 || const 1
  fix arg2 0x2080 u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150 0x2E311C0 0x2E31230 0x2E312A0 0x2E31400 0x2E31448 0x2E31490 0x2E314D8 0x2E31520 0x2E31568 0x2E315B0
  fix arg2 0x2084 u16 alt const 0 || const 1
  fix arg2 0x2088 u32 alt range 0 0x80 || any
  fix arg2 0x208C u32 alt range 0 0x80 || any
  fix arg2 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg2 0x2094 u32 alt range 0 0x40 || any
  fix arg2 0x20A0 u32 addrof arg2 0x20
  fix arg2 0x20A4 u32 addrof arg2 0x50
  fix arg2 0x20A8 u8 alt const 0 || const 1
  fix arg2 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg2 0x20AA u16 range 0 5
  fix arg2 0x20AC u16 range 0 4
  fix arg2 0x20B0 u8 alt const 0 || const 1
  fix arg2 0x20F0 u8 alt const 0 || const 1
  fix arg2 0x20F4 u32 addrof arg2 0x0
  fix arg2 0x20FA u16 const 0
  fix arg2 0x25C u16 range 0 3
  fix arg2 0x25E u8 alt const 0 || const 1
  fix arg2 0x274 u8 alt const 0 || const 1
  fix arg2 0x276 u8 range 0 4
  fix arg2 0x278 u32 addrof arg2 0x0
  fix arg2 0x1000 u32 alt const 0 || const 6
  fix arg2 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg2 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1070 u32 alt const 1 || const 0
  fix arg2 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg2 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg2 0x10E0 u32 alt const 2 || const 1
  fix arg2 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg2 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1150 u32 alt const 3 || const 2
  fix arg2 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg2 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x11C0 u32 alt const 4 || const 3
  fix arg2 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg2 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1230 u32 alt const 5 || const 4
  fix arg2 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg2 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x12A0 u32 alt const 6 || const 5
  fix arg2 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg2 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1400 u32 const 0
  fix arg2 0x1404 u32 const 1
  fix arg2 0x1408 u32 alt range 1 4 || range 0 5
  fix arg2 0x140C u32 alt range 0 6 || const 9
  fix arg2 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1414 u32 alt range 0 8 || any
  fix arg2 0x1448 u32 const 1
  fix arg2 0x144C u32 const 2
  fix arg2 0x1450 u32 alt range 1 4 || range 0 5
  fix arg2 0x1454 u32 alt range 0 6 || const 9
  fix arg2 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x145C u32 alt range 0 8 || any
  fix arg2 0x1490 u32 const 2
  fix arg2 0x1494 u32 const 3
  fix arg2 0x1498 u32 alt range 1 4 || range 0 5
  fix arg2 0x149C u32 alt range 0 6 || const 9
  fix arg2 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x14A4 u32 alt range 0 8 || any
  fix arg2 0x14D8 u32 const 3
  fix arg2 0x14DC u32 const 4
  fix arg2 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg2 0x14E4 u32 alt range 0 6 || const 9
  fix arg2 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x14EC u32 alt range 0 8 || any
  fix arg2 0x1520 u32 const 4
  fix arg2 0x1524 u32 const 5
  fix arg2 0x1528 u32 alt range 1 4 || range 0 5
  fix arg2 0x152C u32 alt range 0 6 || const 9
  fix arg2 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1534 u32 alt range 0 8 || any
  fix arg2 0x1568 u32 const 5
  fix arg2 0x156C u32 const 6
  fix arg2 0x1570 u32 alt range 1 4 || range 0 5
  fix arg2 0x1574 u32 alt range 0 6 || const 9
  fix arg2 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x157C u32 alt range 0 8 || any
  fix arg2 0x15B0 u32 const 6
  fix arg2 0x15B4 u32 const 0
  fix arg2 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg2 0x15BC u32 alt range 0 6 || const 9
  fix arg2 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x15C4 u32 alt range 0 8 || any
  fix arg2 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg2 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg2 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg2 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg2 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg2 0x340 u16 alt range 0 3 || range 0 6
  fix arg2 0x342 u8 alt const 0 || const 1
  fix arg2 0x343 u8 range 0 4
  fix arg2 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg2 0x346 u16 alt range 0 8 || any
  fix arg2 0x25C u16 alt range 0 3 || range 0 6
  fix arg2 0x25E u8 alt const 0 || const 1
  fix arg2 0x25F u8 range 0 4
  fix arg2 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg2 0x262 u16 alt range 0 8 || any
  fix arg2 0x15C u16 alt range 0 3 || range 0 6
  fix arg2 0x15E u8 alt const 0 || const 1
  fix arg2 0x15F u8 range 0 4
  fix arg2 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg2 0x162 u16 alt range 0 8 || any
  stub 021b82f4 force ret=u32 args=v  # rail: POINT / LINE of the RAIL WORK (w)
  stub 021b833c force ret=pick:0x2E31000/0x2E31070/0x2E310E0/0x2E31150/0x2E311C0/0x2E31230/0x2E312A0 args=v  # rail: start point of the line (w)
  stub 021b834c force ret=pick:0x2E31000/0x2E31070/0x2E310E0/0x2E31150/0x2E311C0/0x2E31230/0x2E312A0 args=v  # rail: end point of the line (w)
  stub 021b82f8 force ret=pick+u32:0/0x10/0x40 args=v  # rail: position along the line (w)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 021b835c force ret=pick:0x021D9A30/0x2E31700/0x2E31724/0x2E31748/0x2E3176C/0x2E31790/0x2E317B4/0x2E317D8 args=v  # rail: line record of the line (w)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 021b9b24 force ret=none args=out:12,in:12,in:12,v  # rail: interpolation (&out, a, b, t)
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12  # VEC_Add (a, b, &out)
  stub 021b8300 force ret=pick+u32:0/0x10/0xFFFFFFF0 args=v  # rail: lateral offset (w)
  stub 021b8308 force ret=pick+u32:0x1000/0x2000 args=v  # rail: unit (w)
  stub 0207cae8 force ret=none args=in:12,in:12,outm:12:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF  # VEC_CrossProduct
  stub 0207cee4 force ret=none args=v,in:12,in:12,out:12  # VEC_MultAdd (fx, a, b, &out)

# FUN_021B9D60(w, out): position proc curve, free axis (the interpolation answers zero x / z words often).
fn 021b9d60 ret=none inputs=300
  nowork
  arg u32 const 0x2E30100
  arg u32 const 0x2E303C0
  arg buf 0x3000
  fix arg2 0x2 u16 alt const 0 || const 1 || any
  fix arg2 0x4 u16 alt const 0 || const 1
  fix arg2 0x8 u32 addrof arg2 0x2000
  fix arg2 0xC u32 range 3 4
  fix arg2 0x14 u32 alt const 0 || const 0x2E30100 || oneof 0x2E32000 0x2E3207C
  fix arg2 0x18 u32 alt const 0 || const 1
  fix arg2 0x1C u32 addrof arg2 0x100
  fix arg2 0x20 u16 alt const 7 || range 0 7
  fix arg2 0x22 u16 alt const 7 || range 0 7
  fix arg2 0x24 u16 alt const 7 || range 0 7
  fix arg2 0x26 u16 alt const 7 || range 0 7
  fix arg2 0x28 u16 alt const 7 || range 0 7
  fix arg2 0x2A u16 alt const 7 || range 0 7
  fix arg2 0x2C u16 alt const 7 || range 0 7
  fix arg2 0x30 u32 addrof arg2 0x1000
  fix arg2 0x34 u32 addrof arg2 0x1400
  fix arg2 0x38 u32 addrof arg2 0x1600
  fix arg2 0x3C u32 addrof arg2 0x1700
  fix arg2 0x40 u32 addrof arg2 0x1800
  fix arg2 0x44 u32 addrof arg2 0x1840
  fix arg2 0x48 u32 addrof arg2 0x1880
  fix arg2 0x4C u32 alt range 0 16 || any
  fix arg2 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg2 0x100 u16 alt const 0 || const 1 || const 2
  fix arg2 0x102 u16 alt const 0 || const 1
  fix arg2 0x104 u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150 0x2E311C0 0x2E31230 0x2E312A0 0x2E31400 0x2E31448 0x2E31490 0x2E314D8 0x2E31520 0x2E31568 0x2E315B0
  fix arg2 0x108 u16 alt const 0 || const 1
  fix arg2 0x10C u32 alt range 0 0x80 || any
  fix arg2 0x110 u32 alt range 0 0x80 || any
  fix arg2 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg2 0x118 u32 alt range 0 0x40 || any
  fix arg2 0x124 u32 addrof arg2 0x20
  fix arg2 0x128 u32 addrof arg2 0x50
  fix arg2 0x12C u8 alt const 0 || const 1
  fix arg2 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg2 0x12E u16 range 0 5
  fix arg2 0x130 u16 range 0 4
  fix arg2 0x134 u8 alt const 0 || const 1
  fix arg2 0x174 u8 alt const 0 || const 1
  fix arg2 0x178 u32 addrof arg2 0x0
  fix arg2 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg2 0x2002 u16 alt const 0 || const 1
  fix arg2 0x2004 u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150 0x2E311C0 0x2E31230 0x2E312A0 0x2E31400 0x2E31448 0x2E31490 0x2E314D8 0x2E31520 0x2E31568 0x2E315B0
  fix arg2 0x2008 u16 alt const 0 || const 1
  fix arg2 0x200C u32 alt range 0 0x80 || any
  fix arg2 0x2010 u32 alt range 0 0x80 || any
  fix arg2 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg2 0x2018 u32 alt range 0 0x40 || any
  fix arg2 0x2024 u32 addrof arg2 0x20
  fix arg2 0x2028 u32 addrof arg2 0x50
  fix arg2 0x202C u8 alt const 0 || const 1
  fix arg2 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg2 0x202E u16 range 0 5
  fix arg2 0x2030 u16 range 0 4
  fix arg2 0x2034 u8 alt const 0 || const 1
  fix arg2 0x2074 u8 alt const 0 || const 1
  fix arg2 0x2078 u32 addrof arg2 0x0
  fix arg2 0x207C u16 alt const 0 || const 1 || const 2
  fix arg2 0x207E u16 alt const 0 || const 1
  fix arg2 0x2080 u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150 0x2E311C0 0x2E31230 0x2E312A0 0x2E31400 0x2E31448 0x2E31490 0x2E314D8 0x2E31520 0x2E31568 0x2E315B0
  fix arg2 0x2084 u16 alt const 0 || const 1
  fix arg2 0x2088 u32 alt range 0 0x80 || any
  fix arg2 0x208C u32 alt range 0 0x80 || any
  fix arg2 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg2 0x2094 u32 alt range 0 0x40 || any
  fix arg2 0x20A0 u32 addrof arg2 0x20
  fix arg2 0x20A4 u32 addrof arg2 0x50
  fix arg2 0x20A8 u8 alt const 0 || const 1
  fix arg2 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg2 0x20AA u16 range 0 5
  fix arg2 0x20AC u16 range 0 4
  fix arg2 0x20B0 u8 alt const 0 || const 1
  fix arg2 0x20F0 u8 alt const 0 || const 1
  fix arg2 0x20F4 u32 addrof arg2 0x0
  fix arg2 0x20FA u16 const 0
  fix arg2 0x25C u16 range 0 3
  fix arg2 0x25E u8 alt const 0 || const 1
  fix arg2 0x274 u8 alt const 0 || const 1
  fix arg2 0x276 u8 range 0 4
  fix arg2 0x278 u32 addrof arg2 0x0
  fix arg2 0x1000 u32 alt const 0 || const 6
  fix arg2 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg2 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1070 u32 alt const 1 || const 0
  fix arg2 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg2 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg2 0x10E0 u32 alt const 2 || const 1
  fix arg2 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg2 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1150 u32 alt const 3 || const 2
  fix arg2 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg2 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x11C0 u32 alt const 4 || const 3
  fix arg2 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg2 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1230 u32 alt const 5 || const 4
  fix arg2 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg2 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg2 0x12A0 u32 alt const 6 || const 5
  fix arg2 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg2 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg2 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg2 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg2 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1400 u32 const 0
  fix arg2 0x1404 u32 const 1
  fix arg2 0x1408 u32 alt range 1 4 || range 0 5
  fix arg2 0x140C u32 alt range 0 6 || const 9
  fix arg2 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1414 u32 alt range 0 8 || any
  fix arg2 0x1448 u32 const 1
  fix arg2 0x144C u32 const 2
  fix arg2 0x1450 u32 alt range 1 4 || range 0 5
  fix arg2 0x1454 u32 alt range 0 6 || const 9
  fix arg2 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x145C u32 alt range 0 8 || any
  fix arg2 0x1490 u32 const 2
  fix arg2 0x1494 u32 const 3
  fix arg2 0x1498 u32 alt range 1 4 || range 0 5
  fix arg2 0x149C u32 alt range 0 6 || const 9
  fix arg2 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x14A4 u32 alt range 0 8 || any
  fix arg2 0x14D8 u32 const 3
  fix arg2 0x14DC u32 const 4
  fix arg2 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg2 0x14E4 u32 alt range 0 6 || const 9
  fix arg2 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x14EC u32 alt range 0 8 || any
  fix arg2 0x1520 u32 const 4
  fix arg2 0x1524 u32 const 5
  fix arg2 0x1528 u32 alt range 1 4 || range 0 5
  fix arg2 0x152C u32 alt range 0 6 || const 9
  fix arg2 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x1534 u32 alt range 0 8 || any
  fix arg2 0x1568 u32 const 5
  fix arg2 0x156C u32 const 6
  fix arg2 0x1570 u32 alt range 1 4 || range 0 5
  fix arg2 0x1574 u32 alt range 0 6 || const 9
  fix arg2 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x157C u32 alt range 0 8 || any
  fix arg2 0x15B0 u32 const 6
  fix arg2 0x15B4 u32 const 0
  fix arg2 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg2 0x15BC u32 alt range 0 6 || const 9
  fix arg2 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg2 0x15C4 u32 alt range 0 8 || any
  fix arg2 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg2 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg2 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg2 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg2 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg2 0x340 u16 alt range 0 3 || range 0 6
  fix arg2 0x342 u8 alt const 0 || const 1
  fix arg2 0x343 u8 range 0 4
  fix arg2 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg2 0x346 u16 alt range 0 8 || any
  fix arg2 0x25C u16 alt range 0 3 || range 0 6
  fix arg2 0x25E u8 alt const 0 || const 1
  fix arg2 0x25F u8 range 0 4
  fix arg2 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg2 0x262 u16 alt range 0 8 || any
  fix arg2 0x15C u16 alt range 0 3 || range 0 6
  fix arg2 0x15E u8 alt const 0 || const 1
  fix arg2 0x15F u8 range 0 4
  fix arg2 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg2 0x162 u16 alt range 0 8 || any
  stub 021b82f4 force ret=u32 args=v  # rail: POINT / LINE of the RAIL WORK (w)
  stub 021b833c force ret=pick:0x2E31000/0x2E31070/0x2E310E0/0x2E31150/0x2E311C0/0x2E31230/0x2E312A0 args=v  # rail: start point of the line (w)
  stub 021b834c force ret=pick:0x2E31000/0x2E31070/0x2E310E0/0x2E31150/0x2E311C0/0x2E31230/0x2E312A0 args=v  # rail: end point of the line (w)
  stub 021b82f8 force ret=pick+u32:0/0x10/0x40 args=v  # rail: position along the line (w)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 021b835c force ret=pick:0x021D9A30/0x2E31700/0x2E31724/0x2E31748/0x2E3176C/0x2E31790/0x2E317B4/0x2E317D8 args=v  # rail: line record of the line (w)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 021b9b24 force ret=none args=outm:12:3/0/0/0/0xFF/0xFF/0xFF/0xFF/3/0/0/0,in:12,in:12,v  # rail: interpolation (&out, a, b, t)
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12  # VEC_Add (a, b, &out)
  stub 021b8300 force ret=pick+u32:0/0x10/0xFFFFFFF0 args=v  # rail: lateral offset (w)
  stub 021b8308 force ret=pick+u32:0x1000/0x2000 args=v  # rail: unit (w)
  stub 0207cae8 force ret=none args=in:12,in:12,outm:12:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF  # VEC_CrossProduct
  stub 0207cee4 force ret=none args=v,in:12,in:12,out:12  # VEC_MultAdd (fx, a, b, &out)

# FUN_021B9E84(i, R, hit, loc, pos): line-hit proc straight.
fn 021b9e84 ret=u32 inputs=600
  nowork
  arg u32 alt const 0 || range 0 6
  arg u32 const 0x2E60000
  arg u32 const 0x2E60380
  arg u32 const 0x2E60360
  arg u32 const 0x2E603C0
  arg buf 0x3000
  fix arg5 0x2 u16 alt const 0 || const 1 || any
  fix arg5 0x4 u16 alt const 0 || const 1
  fix arg5 0x8 u32 addrof arg5 0x2000
  fix arg5 0xC u32 range 3 4
  fix arg5 0x14 u32 alt const 0 || const 0x2E60100 || oneof 0x2E62000 0x2E6207C
  fix arg5 0x18 u32 alt const 0 || const 1
  fix arg5 0x1C u32 addrof arg5 0x100
  fix arg5 0x20 u16 alt const 7 || range 0 7
  fix arg5 0x22 u16 alt const 7 || range 0 7
  fix arg5 0x24 u16 alt const 7 || range 0 7
  fix arg5 0x26 u16 alt const 7 || range 0 7
  fix arg5 0x28 u16 alt const 7 || range 0 7
  fix arg5 0x2A u16 alt const 7 || range 0 7
  fix arg5 0x2C u16 alt const 7 || range 0 7
  fix arg5 0x30 u32 addrof arg5 0x1000
  fix arg5 0x34 u32 addrof arg5 0x1400
  fix arg5 0x38 u32 addrof arg5 0x1600
  fix arg5 0x3C u32 addrof arg5 0x1700
  fix arg5 0x40 u32 addrof arg5 0x1800
  fix arg5 0x44 u32 addrof arg5 0x1840
  fix arg5 0x48 u32 addrof arg5 0x1880
  fix arg5 0x4C u32 alt range 0 16 || any
  fix arg5 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg5 0x100 u16 alt const 0 || const 1 || const 2
  fix arg5 0x102 u16 alt const 0 || const 1
  fix arg5 0x104 u32 oneof 0x2E61000 0x2E61070 0x2E610E0 0x2E61150 0x2E611C0 0x2E61230 0x2E612A0 0x2E61400 0x2E61448 0x2E61490 0x2E614D8 0x2E61520 0x2E61568 0x2E615B0
  fix arg5 0x108 u16 alt const 0 || const 1
  fix arg5 0x10C u32 alt range 0 0x80 || any
  fix arg5 0x110 u32 alt range 0 0x80 || any
  fix arg5 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg5 0x118 u32 alt range 0 0x40 || any
  fix arg5 0x124 u32 addrof arg5 0x20
  fix arg5 0x128 u32 addrof arg5 0x50
  fix arg5 0x12C u8 alt const 0 || const 1
  fix arg5 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg5 0x12E u16 range 0 5
  fix arg5 0x130 u16 range 0 4
  fix arg5 0x134 u8 alt const 0 || const 1
  fix arg5 0x174 u8 alt const 0 || const 1
  fix arg5 0x178 u32 addrof arg5 0x0
  fix arg5 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg5 0x2002 u16 alt const 0 || const 1
  fix arg5 0x2004 u32 oneof 0x2E61000 0x2E61070 0x2E610E0 0x2E61150 0x2E611C0 0x2E61230 0x2E612A0 0x2E61400 0x2E61448 0x2E61490 0x2E614D8 0x2E61520 0x2E61568 0x2E615B0
  fix arg5 0x2008 u16 alt const 0 || const 1
  fix arg5 0x200C u32 alt range 0 0x80 || any
  fix arg5 0x2010 u32 alt range 0 0x80 || any
  fix arg5 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg5 0x2018 u32 alt range 0 0x40 || any
  fix arg5 0x2024 u32 addrof arg5 0x20
  fix arg5 0x2028 u32 addrof arg5 0x50
  fix arg5 0x202C u8 alt const 0 || const 1
  fix arg5 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg5 0x202E u16 range 0 5
  fix arg5 0x2030 u16 range 0 4
  fix arg5 0x2034 u8 alt const 0 || const 1
  fix arg5 0x2074 u8 alt const 0 || const 1
  fix arg5 0x2078 u32 addrof arg5 0x0
  fix arg5 0x207C u16 alt const 0 || const 1 || const 2
  fix arg5 0x207E u16 alt const 0 || const 1
  fix arg5 0x2080 u32 oneof 0x2E61000 0x2E61070 0x2E610E0 0x2E61150 0x2E611C0 0x2E61230 0x2E612A0 0x2E61400 0x2E61448 0x2E61490 0x2E614D8 0x2E61520 0x2E61568 0x2E615B0
  fix arg5 0x2084 u16 alt const 0 || const 1
  fix arg5 0x2088 u32 alt range 0 0x80 || any
  fix arg5 0x208C u32 alt range 0 0x80 || any
  fix arg5 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg5 0x2094 u32 alt range 0 0x40 || any
  fix arg5 0x20A0 u32 addrof arg5 0x20
  fix arg5 0x20A4 u32 addrof arg5 0x50
  fix arg5 0x20A8 u8 alt const 0 || const 1
  fix arg5 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg5 0x20AA u16 range 0 5
  fix arg5 0x20AC u16 range 0 4
  fix arg5 0x20B0 u8 alt const 0 || const 1
  fix arg5 0x20F0 u8 alt const 0 || const 1
  fix arg5 0x20F4 u32 addrof arg5 0x0
  fix arg5 0x20FA u16 const 0
  fix arg5 0x25C u16 range 0 3
  fix arg5 0x25E u8 alt const 0 || const 1
  fix arg5 0x274 u8 alt const 0 || const 1
  fix arg5 0x276 u8 range 0 4
  fix arg5 0x278 u32 addrof arg5 0x0
  fix arg5 0x1000 u32 alt const 0 || const 6
  fix arg5 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg5 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1070 u32 alt const 1 || const 0
  fix arg5 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg5 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg5 0x10E0 u32 alt const 2 || const 1
  fix arg5 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg5 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1150 u32 alt const 3 || const 2
  fix arg5 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg5 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg5 0x11C0 u32 alt const 4 || const 3
  fix arg5 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg5 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1230 u32 alt const 5 || const 4
  fix arg5 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg5 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg5 0x12A0 u32 alt const 6 || const 5
  fix arg5 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg5 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1400 u32 const 0
  fix arg5 0x1404 u32 const 1
  fix arg5 0x1408 u32 alt range 1 4 || range 0 5
  fix arg5 0x140C u32 alt range 0 6 || const 9
  fix arg5 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1414 u32 alt range 0 8 || any
  fix arg5 0x1448 u32 const 1
  fix arg5 0x144C u32 const 2
  fix arg5 0x1450 u32 alt range 1 4 || range 0 5
  fix arg5 0x1454 u32 alt range 0 6 || const 9
  fix arg5 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x145C u32 alt range 0 8 || any
  fix arg5 0x1490 u32 const 2
  fix arg5 0x1494 u32 const 3
  fix arg5 0x1498 u32 alt range 1 4 || range 0 5
  fix arg5 0x149C u32 alt range 0 6 || const 9
  fix arg5 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x14A4 u32 alt range 0 8 || any
  fix arg5 0x14D8 u32 const 3
  fix arg5 0x14DC u32 const 4
  fix arg5 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg5 0x14E4 u32 alt range 0 6 || const 9
  fix arg5 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x14EC u32 alt range 0 8 || any
  fix arg5 0x1520 u32 const 4
  fix arg5 0x1524 u32 const 5
  fix arg5 0x1528 u32 alt range 1 4 || range 0 5
  fix arg5 0x152C u32 alt range 0 6 || const 9
  fix arg5 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1534 u32 alt range 0 8 || any
  fix arg5 0x1568 u32 const 5
  fix arg5 0x156C u32 const 6
  fix arg5 0x1570 u32 alt range 1 4 || range 0 5
  fix arg5 0x1574 u32 alt range 0 6 || const 9
  fix arg5 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x157C u32 alt range 0 8 || any
  fix arg5 0x15B0 u32 const 6
  fix arg5 0x15B4 u32 const 0
  fix arg5 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg5 0x15BC u32 alt range 0 6 || const 9
  fix arg5 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x15C4 u32 alt range 0 8 || any
  fix arg5 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg5 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg5 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg5 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg5 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg5 0x340 u16 alt range 0 3 || range 0 6
  fix arg5 0x342 u8 alt const 0 || const 1
  fix arg5 0x343 u8 range 0 4
  fix arg5 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg5 0x346 u16 alt range 0 8 || any
  fix arg5 0x25C u16 alt range 0 3 || range 0 6
  fix arg5 0x25E u8 alt const 0 || const 1
  fix arg5 0x25F u8 range 0 4
  fix arg5 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg5 0x262 u16 alt range 0 8 || any
  fix arg5 0x15C u16 alt range 0 3 || range 0 6
  fix arg5 0x15E u8 alt const 0 || const 1
  fix arg5 0x15F u8 range 0 4
  fix arg5 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg5 0x162 u16 alt range 0 8 || any
  fix arg5 0x1034 u32 range 0 0x4000
  fix arg5 0x10A4 u32 range 0 0x4000
  fix arg5 0x1114 u32 range 0 0x4000
  fix arg5 0x1184 u32 range 0 0x4000
  fix arg5 0x11F4 u32 range 0 0x4000
  fix arg5 0x1264 u32 range 0 0x4000
  fix arg5 0x12D4 u32 range 0 0x4000
  fix arg5 0x384 u32 alt range 0 0x4000 || range 0x9000 0xA000 || oneof 0x7FFF 0x8000 0x8001
  fix arg5 0x1020 u32 alt const 0 || range 0 6 || any rep=4,4
  fix arg5 0x1090 u32 alt const 0 || range 0 6 || any rep=4,4
  fix arg5 0x1100 u32 alt const 0 || range 0 6 || any rep=4,4
  fix arg5 0x1170 u32 alt const 0 || range 0 6 || any rep=4,4
  fix arg5 0x11E0 u32 alt const 0 || range 0 6 || any rep=4,4
  fix arg5 0x1250 u32 alt const 0 || range 0 6 || any rep=4,4
  fix arg5 0x12C0 u32 alt const 0 || range 0 6 || any rep=4,4
  stub 021b8230 force ret=pick:0x2E61400/0x2E61448/0x2E61490/0x2E614D8/0x2E61520/0x2E61568/0x2E615B0 args=v,v  # rail: line i (R, i)
  stub 021b8224 force ret=pick:0x2E61000/0x2E61070/0x2E610E0/0x2E61150/0x2E611C0/0x2E61230/0x2E612A0 args=v,v  # rail: point i (R, i)
  stub 021b8248 force ret=pick+u32:0x1000/0x2000 args=v  # rail: unit (R)
  stub 021bae7c ret=pick:0/1 args=v,v,in:12,out:12,out:4,out:12  # rail: project a hit on a segment (pa, pb, hit, &p, &q, &side)
  stub 0207cf3c force ret=pick+u32:0x1000/0x2000/0xFFFFF/0x100000 args=in:12,in:12  # VEC_Distance (a, b)
  stub 0207c768 force ret=pick+u32:0/0x8000/0x10000/0x11000/0x20000 args=v,v  # FX_Div (a, b)
  stub 021b823c force ret=u32 args=v,v  # rail: line record i (R, i)
  stub 021b8378 force ret=pick:0/1 args=v  # rail: x / 16 rounded (x)
  stub 021b8250 force ret=none args=v,in:8,outw:8:0x03010000/0x03010001/0x00010000/0x00010001/0x00020001  # rail: clamp a LOCATION (R, &loc, &out)
  stub 021b7d38 force ret=none args=v,in:8,out:12  # rail: position of a LOCATION (R, &loc, &out)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,outm:12:0xFF/0x0F/0/0/0/0/0/0/0xFF/0x0F/0/0  # VEC_Normalize (a, &out): flat, so hits occur
  stub 0207cbd0 force ret=pick+u32:0x1000/0x2000/0x3000 args=in:12  # VEC_Mag (a)
  stub 0207ca68 force ret=pick+u32:0/0x800/0x1000/0x1001/0xFFFFFFFF args=in:12,in:12  # VEC_DotProduct (a, b)
  stub 0207cae8 force ret=none args=in:12,in:12,out:12  # VEC_CrossProduct (a, b, &out)
  stub 0207c9d0 force ret=none args=in:12,in:12,outm:12:0xFF/0xFF/0/0/0/0/0/0/0xFF/0xFF/0/0  # VEC_Add (a, b, &out): height 0

# FUN_021B9FC8(i, R, hit, loc, pos): line-hit proc stepped.
fn 021b9fc8 ret=u32 inputs=3000
  nowork
  arg u32 alt const 0 || range 0 6
  arg u32 const 0x2E60000
  arg u32 const 0x2E60380
  arg u32 const 0x2E60360
  arg u32 const 0x2E603C0
  arg buf 0x3000
  fix arg5 0x2 u16 alt const 0 || const 1 || any
  fix arg5 0x4 u16 alt const 0 || const 1
  fix arg5 0x8 u32 addrof arg5 0x2000
  fix arg5 0xC u32 range 3 4
  fix arg5 0x14 u32 alt const 0 || const 0x2E60100 || oneof 0x2E62000 0x2E6207C
  fix arg5 0x18 u32 alt const 0 || const 1
  fix arg5 0x1C u32 addrof arg5 0x100
  fix arg5 0x20 u16 alt const 7 || range 0 7
  fix arg5 0x22 u16 alt const 7 || range 0 7
  fix arg5 0x24 u16 alt const 7 || range 0 7
  fix arg5 0x26 u16 alt const 7 || range 0 7
  fix arg5 0x28 u16 alt const 7 || range 0 7
  fix arg5 0x2A u16 alt const 7 || range 0 7
  fix arg5 0x2C u16 alt const 7 || range 0 7
  fix arg5 0x30 u32 addrof arg5 0x1000
  fix arg5 0x34 u32 addrof arg5 0x1400
  fix arg5 0x38 u32 addrof arg5 0x1600
  fix arg5 0x3C u32 addrof arg5 0x1700
  fix arg5 0x40 u32 addrof arg5 0x1800
  fix arg5 0x44 u32 addrof arg5 0x1840
  fix arg5 0x48 u32 addrof arg5 0x1880
  fix arg5 0x4C u32 alt range 0 16 || any
  fix arg5 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg5 0x100 u16 alt const 0 || const 1 || const 2
  fix arg5 0x102 u16 alt const 0 || const 1
  fix arg5 0x104 u32 oneof 0x2E61000 0x2E61070 0x2E610E0 0x2E61150 0x2E611C0 0x2E61230 0x2E612A0 0x2E61400 0x2E61448 0x2E61490 0x2E614D8 0x2E61520 0x2E61568 0x2E615B0
  fix arg5 0x108 u16 alt const 0 || const 1
  fix arg5 0x10C u32 alt range 0 0x80 || any
  fix arg5 0x110 u32 alt range 0 0x80 || any
  fix arg5 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg5 0x118 u32 alt range 0 0x40 || any
  fix arg5 0x124 u32 addrof arg5 0x20
  fix arg5 0x128 u32 addrof arg5 0x50
  fix arg5 0x12C u8 alt const 0 || const 1
  fix arg5 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg5 0x12E u16 range 0 5
  fix arg5 0x130 u16 range 0 4
  fix arg5 0x134 u8 alt const 0 || const 1
  fix arg5 0x174 u8 alt const 0 || const 1
  fix arg5 0x178 u32 addrof arg5 0x0
  fix arg5 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg5 0x2002 u16 alt const 0 || const 1
  fix arg5 0x2004 u32 oneof 0x2E61000 0x2E61070 0x2E610E0 0x2E61150 0x2E611C0 0x2E61230 0x2E612A0 0x2E61400 0x2E61448 0x2E61490 0x2E614D8 0x2E61520 0x2E61568 0x2E615B0
  fix arg5 0x2008 u16 alt const 0 || const 1
  fix arg5 0x200C u32 alt range 0 0x80 || any
  fix arg5 0x2010 u32 alt range 0 0x80 || any
  fix arg5 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg5 0x2018 u32 alt range 0 0x40 || any
  fix arg5 0x2024 u32 addrof arg5 0x20
  fix arg5 0x2028 u32 addrof arg5 0x50
  fix arg5 0x202C u8 alt const 0 || const 1
  fix arg5 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg5 0x202E u16 range 0 5
  fix arg5 0x2030 u16 range 0 4
  fix arg5 0x2034 u8 alt const 0 || const 1
  fix arg5 0x2074 u8 alt const 0 || const 1
  fix arg5 0x2078 u32 addrof arg5 0x0
  fix arg5 0x207C u16 alt const 0 || const 1 || const 2
  fix arg5 0x207E u16 alt const 0 || const 1
  fix arg5 0x2080 u32 oneof 0x2E61000 0x2E61070 0x2E610E0 0x2E61150 0x2E611C0 0x2E61230 0x2E612A0 0x2E61400 0x2E61448 0x2E61490 0x2E614D8 0x2E61520 0x2E61568 0x2E615B0
  fix arg5 0x2084 u16 alt const 0 || const 1
  fix arg5 0x2088 u32 alt range 0 0x80 || any
  fix arg5 0x208C u32 alt range 0 0x80 || any
  fix arg5 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg5 0x2094 u32 alt range 0 0x40 || any
  fix arg5 0x20A0 u32 addrof arg5 0x20
  fix arg5 0x20A4 u32 addrof arg5 0x50
  fix arg5 0x20A8 u8 alt const 0 || const 1
  fix arg5 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg5 0x20AA u16 range 0 5
  fix arg5 0x20AC u16 range 0 4
  fix arg5 0x20B0 u8 alt const 0 || const 1
  fix arg5 0x20F0 u8 alt const 0 || const 1
  fix arg5 0x20F4 u32 addrof arg5 0x0
  fix arg5 0x20FA u16 const 0
  fix arg5 0x25C u16 range 0 3
  fix arg5 0x25E u8 alt const 0 || const 1
  fix arg5 0x274 u8 alt const 0 || const 1
  fix arg5 0x276 u8 range 0 4
  fix arg5 0x278 u32 addrof arg5 0x0
  fix arg5 0x1000 u32 alt const 0 || const 6
  fix arg5 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg5 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1070 u32 alt const 1 || const 0
  fix arg5 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg5 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg5 0x10E0 u32 alt const 2 || const 1
  fix arg5 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg5 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1150 u32 alt const 3 || const 2
  fix arg5 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg5 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg5 0x11C0 u32 alt const 4 || const 3
  fix arg5 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg5 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1230 u32 alt const 5 || const 4
  fix arg5 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg5 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg5 0x12A0 u32 alt const 6 || const 5
  fix arg5 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg5 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg5 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg5 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg5 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1400 u32 const 0
  fix arg5 0x1404 u32 const 1
  fix arg5 0x1408 u32 alt range 1 4 || range 0 5
  fix arg5 0x140C u32 alt range 0 6 || const 9
  fix arg5 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1414 u32 alt range 0 8 || any
  fix arg5 0x1448 u32 const 1
  fix arg5 0x144C u32 const 2
  fix arg5 0x1450 u32 alt range 1 4 || range 0 5
  fix arg5 0x1454 u32 alt range 0 6 || const 9
  fix arg5 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x145C u32 alt range 0 8 || any
  fix arg5 0x1490 u32 const 2
  fix arg5 0x1494 u32 const 3
  fix arg5 0x1498 u32 alt range 1 4 || range 0 5
  fix arg5 0x149C u32 alt range 0 6 || const 9
  fix arg5 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x14A4 u32 alt range 0 8 || any
  fix arg5 0x14D8 u32 const 3
  fix arg5 0x14DC u32 const 4
  fix arg5 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg5 0x14E4 u32 alt range 0 6 || const 9
  fix arg5 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x14EC u32 alt range 0 8 || any
  fix arg5 0x1520 u32 const 4
  fix arg5 0x1524 u32 const 5
  fix arg5 0x1528 u32 alt range 1 4 || range 0 5
  fix arg5 0x152C u32 alt range 0 6 || const 9
  fix arg5 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x1534 u32 alt range 0 8 || any
  fix arg5 0x1568 u32 const 5
  fix arg5 0x156C u32 const 6
  fix arg5 0x1570 u32 alt range 1 4 || range 0 5
  fix arg5 0x1574 u32 alt range 0 6 || const 9
  fix arg5 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x157C u32 alt range 0 8 || any
  fix arg5 0x15B0 u32 const 6
  fix arg5 0x15B4 u32 const 0
  fix arg5 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg5 0x15BC u32 alt range 0 6 || const 9
  fix arg5 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg5 0x15C4 u32 alt range 0 8 || any
  fix arg5 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg5 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg5 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg5 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg5 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg5 0x340 u16 alt range 0 3 || range 0 6
  fix arg5 0x342 u8 alt const 0 || const 1
  fix arg5 0x343 u8 range 0 4
  fix arg5 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg5 0x346 u16 alt range 0 8 || any
  fix arg5 0x25C u16 alt range 0 3 || range 0 6
  fix arg5 0x25E u8 alt const 0 || const 1
  fix arg5 0x25F u8 range 0 4
  fix arg5 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg5 0x262 u16 alt range 0 8 || any
  fix arg5 0x15C u16 alt range 0 3 || range 0 6
  fix arg5 0x15E u8 alt const 0 || const 1
  fix arg5 0x15F u8 range 0 4
  fix arg5 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg5 0x162 u16 alt range 0 8 || any
  fix arg5 0x1034 u32 range 0 0x4000
  fix arg5 0x10A4 u32 range 0 0x4000
  fix arg5 0x1114 u32 range 0 0x4000
  fix arg5 0x1184 u32 range 0 0x4000
  fix arg5 0x11F4 u32 range 0 0x4000
  fix arg5 0x1264 u32 range 0 0x4000
  fix arg5 0x12D4 u32 range 0 0x4000
  fix arg5 0x384 u32 alt range 0 0x4000 || range 0x9000 0xA000 || oneof 0x7FFF 0x8000 0x8001
  stub 021b8230 force ret=pick:0x2E61400/0x2E61448/0x2E61490/0x2E614D8/0x2E61520/0x2E61568/0x2E615B0 args=v,v  # rail: line i (R, i)
  stub 021b8224 force ret=pick:0x2E61000/0x2E61070/0x2E610E0/0x2E61150/0x2E611C0/0x2E61230/0x2E612A0 args=v,v  # rail: point i (R, i)
  stub 021bae7c ret=pick:0/1 args=v,v,in:12,out:12,out:4,out:12  # rail: project a hit on a segment (pa, pb, hit, &p, &q, &side)
  stub 021b824c force ret=u32 args=v  # rail: the main RAIL WORK (R)
  stub 021b7ee4 force ret=none args=v,in:8  # rail: place the RAIL WORK at a LOCATION (w, &loc)
  stub 021b82fc force ret=pick:0/0x10/0x20/0x40/0x80 args=v  # rail: line length (w; bounded: the stepped loop runs length / 16 times)
  stub 021b8028 force ret=none args=v,out:12  # rail: position (w, &out)
  stub 021b8248 force ret=pick+u32:0x1000/0x2000 args=v  # rail: unit (R)
  stub 021b8304 force ret=pick+u32:0/0x10/0xFFFFFFF0 args=v  # rail: half width (w)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,outm:12:0xFF/0x0F/0/0/0/0/0/0/0xFF/0x0F/0/0  # VEC_Normalize (a, &out): flat, so hits occur
  stub 0207cbd0 force ret=pick+u32:0x1000/0x2000/0x3000 args=in:12  # VEC_Mag (a)
  stub 0207cae8 force ret=none args=in:12,in:12,out:12  # VEC_CrossProduct (a, b, &out)
  stub 0207c768 force ret=pick+u32:0/0x8000/0x10000/0x11000/0x20000 args=v,v  # FX_Div (a, b)
  stub 0207ca68 force ret=pick+u32:0/0x800/0x1000/0x1001/0xFFFFFFFF args=in:12,in:12  # VEC_DotProduct (a, b)
  stub 0207c9d0 force ret=none args=in:12,in:12,outm:12:0xFF/0xFF/0/0/0/0/0/0/0xFF/0xFF/0/0  # VEC_Add (a, b, &out): height 0
  stub 0207cf3c force ret=pick+u32:0x1000/0x2000/0xFFFFF/0x100000 args=in:12,in:12  # VEC_Distance (a, b)
  stub 021b8378 force ret=pick:0/1 args=v  # rail: x / 16 rounded (x)
  stub 021b8250 force ret=none args=v,in:8,outw:8:0x03010000/0x03010001/0x00010000/0x00010001/0x00020001  # rail: clamp a LOCATION (R, &loc, &out)
  stub 021b7d38 force ret=none args=v,in:8,out:12  # rail: position of a LOCATION (R, &loc, &out)

# FUN_021BA25C(R): camera proc: target = position + record vector.
fn 021ba25c ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ecf4 force ret=none args=v,in:12  # field camera: target (cam, &v)
  stub 0218ec60 force ret=none args=v  # field camera (cam)

# FUN_021BA2B4(R): camera proc: record angles / distance.
fn 021ba2b4 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)

# FUN_021BA310(R): camera proc: angles + position without lateral offset.
fn 021ba310 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 021ba2b4 ret=none args=v  # rail: camera proc angles (R)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 021b8314 force ret=none args=v,out:12  # rail: position without the lateral offset (w, &out)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)

# FUN_021BA348(R): camera proc: interpolated angles / distance.
fn 021ba348 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b833c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: start point of the line (w)
  stub 021b834c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: end point of the line (w)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 021b836c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v,v  # rail: camera record of a point (w, pt)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
  stub 021b82f8 force ret=pick+u32:0/0x10/0x40 args=v  # rail: position along the line (w)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)

# FUN_021BA494(R): camera proc: interpolated + current position.
fn 021ba494 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 021ba348 ret=none args=v  # rail: camera proc interpolated angles (R)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b833c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: start point of the line (w)
  stub 021b834c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: end point of the line (w)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 021b836c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v,v  # rail: camera record of a point (w, pt)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
  stub 021b82f8 force ret=pick+u32:0/0x10/0x40 args=v  # rail: position along the line (w)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)

# FUN_021BA4C4(R): camera proc: interpolated + position without lateral offset.
fn 021ba4c4 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 021ba348 ret=none args=v  # rail: camera proc interpolated angles (R)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 021b8314 force ret=none args=v,out:12  # rail: position without the lateral offset (w, &out)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 021b833c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: start point of the line (w)
  stub 021b834c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: end point of the line (w)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 021b836c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v,v  # rail: camera record of a point (w, pt)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
  stub 021b82f8 force ret=pick+u32:0/0x10/0x40 args=v  # rail: position along the line (w)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)

# FUN_021BA4FC(R): camera proc: fixed target / position.
fn 021ba4fc ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 0218ecf4 force ret=none args=v,in:12  # field camera: target (cam, &v)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)

# FUN_021BA550(R): camera proc: interpolated target / position.
fn 021ba550 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 021b833c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: start point of the line (w)
  stub 021b834c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: end point of the line (w)
  stub 021b836c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v,v  # rail: camera record of a point (w, pt)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
  stub 021b82f8 force ret=pick+u32:0/0x10/0x40 args=v  # rail: position along the line (w)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 0218ecf4 force ret=none args=v,in:12  # field camera: target (cam, &v)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)

# FUN_021BA6E8(R): camera proc: orbit, interpolated.
fn 021ba6e8 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 021b82f0 force ret=pick+u32:0/1/2 args=v  # rail: kind (w)
  stub 021b833c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: start point of the line (w)
  stub 021b834c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: end point of the line (w)
  stub 021b836c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v,v  # rail: camera record of a point (w, pt)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
  stub 021b82f8 force ret=pick+u32:0/0x10/0x40 args=v  # rail: position along the line (w)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 021ba7e0 ret=none args=v,v,v,in:12  # rail: orbit (R, ang, dist, &centre)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 0218ecf4 force ret=none args=v,in:12  # field camera: target (cam, &v)

# FUN_021BA7B0(R): camera proc: orbit.
fn 021ba7b0 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 021ba7e0 ret=none args=v,v,v,in:12  # rail: orbit (R, ang, dist, &centre)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 0218ecf4 force ret=none args=v,in:12  # field camera: target (cam, &v)

# FUN_021BA7E0(R, ang, dist, centre): orbit.
fn 021ba7e0 ret=none inputs=150
  nowork
  arg u32 const 0x2E50000
  arg u32 range 0 0xFFFF
  arg u32 any
  arg u32 const 0x2E50380
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg4 0x2 u16 alt const 0 || const 1 || any
  fix arg4 0x4 u16 alt const 0 || const 1
  fix arg4 0x8 u32 addrof arg4 0x2000
  fix arg4 0xC u32 range 3 4
  fix arg4 0x14 u32 alt const 0 || const 0x2E50100 || oneof 0x2E52000 0x2E5207C
  fix arg4 0x18 u32 alt const 0 || const 1
  fix arg4 0x1C u32 addrof arg4 0x100
  fix arg4 0x20 u16 alt const 7 || range 0 7
  fix arg4 0x22 u16 alt const 7 || range 0 7
  fix arg4 0x24 u16 alt const 7 || range 0 7
  fix arg4 0x26 u16 alt const 7 || range 0 7
  fix arg4 0x28 u16 alt const 7 || range 0 7
  fix arg4 0x2A u16 alt const 7 || range 0 7
  fix arg4 0x2C u16 alt const 7 || range 0 7
  fix arg4 0x30 u32 addrof arg4 0x1000
  fix arg4 0x34 u32 addrof arg4 0x1400
  fix arg4 0x38 u32 addrof arg4 0x1600
  fix arg4 0x3C u32 addrof arg4 0x1700
  fix arg4 0x40 u32 addrof arg4 0x1800
  fix arg4 0x44 u32 addrof arg4 0x1840
  fix arg4 0x48 u32 addrof arg4 0x1880
  fix arg4 0x4C u32 alt range 0 16 || any
  fix arg4 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg4 0x100 u16 alt const 0 || const 1 || const 2
  fix arg4 0x102 u16 alt const 0 || const 1
  fix arg4 0x104 u32 oneof 0x2E51000 0x2E51070 0x2E510E0 0x2E51150 0x2E511C0 0x2E51230 0x2E512A0 0x2E51400 0x2E51448 0x2E51490 0x2E514D8 0x2E51520 0x2E51568 0x2E515B0
  fix arg4 0x108 u16 alt const 0 || const 1
  fix arg4 0x10C u32 alt range 0 0x80 || any
  fix arg4 0x110 u32 alt range 0 0x80 || any
  fix arg4 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg4 0x118 u32 alt range 0 0x40 || any
  fix arg4 0x124 u32 addrof arg4 0x20
  fix arg4 0x128 u32 addrof arg4 0x50
  fix arg4 0x12C u8 alt const 0 || const 1
  fix arg4 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg4 0x12E u16 range 0 5
  fix arg4 0x130 u16 range 0 4
  fix arg4 0x134 u8 alt const 0 || const 1
  fix arg4 0x174 u8 alt const 0 || const 1
  fix arg4 0x178 u32 addrof arg4 0x0
  fix arg4 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg4 0x2002 u16 alt const 0 || const 1
  fix arg4 0x2004 u32 oneof 0x2E51000 0x2E51070 0x2E510E0 0x2E51150 0x2E511C0 0x2E51230 0x2E512A0 0x2E51400 0x2E51448 0x2E51490 0x2E514D8 0x2E51520 0x2E51568 0x2E515B0
  fix arg4 0x2008 u16 alt const 0 || const 1
  fix arg4 0x200C u32 alt range 0 0x80 || any
  fix arg4 0x2010 u32 alt range 0 0x80 || any
  fix arg4 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg4 0x2018 u32 alt range 0 0x40 || any
  fix arg4 0x2024 u32 addrof arg4 0x20
  fix arg4 0x2028 u32 addrof arg4 0x50
  fix arg4 0x202C u8 alt const 0 || const 1
  fix arg4 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg4 0x202E u16 range 0 5
  fix arg4 0x2030 u16 range 0 4
  fix arg4 0x2034 u8 alt const 0 || const 1
  fix arg4 0x2074 u8 alt const 0 || const 1
  fix arg4 0x2078 u32 addrof arg4 0x0
  fix arg4 0x207C u16 alt const 0 || const 1 || const 2
  fix arg4 0x207E u16 alt const 0 || const 1
  fix arg4 0x2080 u32 oneof 0x2E51000 0x2E51070 0x2E510E0 0x2E51150 0x2E511C0 0x2E51230 0x2E512A0 0x2E51400 0x2E51448 0x2E51490 0x2E514D8 0x2E51520 0x2E51568 0x2E515B0
  fix arg4 0x2084 u16 alt const 0 || const 1
  fix arg4 0x2088 u32 alt range 0 0x80 || any
  fix arg4 0x208C u32 alt range 0 0x80 || any
  fix arg4 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg4 0x2094 u32 alt range 0 0x40 || any
  fix arg4 0x20A0 u32 addrof arg4 0x20
  fix arg4 0x20A4 u32 addrof arg4 0x50
  fix arg4 0x20A8 u8 alt const 0 || const 1
  fix arg4 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg4 0x20AA u16 range 0 5
  fix arg4 0x20AC u16 range 0 4
  fix arg4 0x20B0 u8 alt const 0 || const 1
  fix arg4 0x20F0 u8 alt const 0 || const 1
  fix arg4 0x20F4 u32 addrof arg4 0x0
  fix arg4 0x20FA u16 const 0
  fix arg4 0x25C u16 range 0 3
  fix arg4 0x25E u8 alt const 0 || const 1
  fix arg4 0x274 u8 alt const 0 || const 1
  fix arg4 0x276 u8 range 0 4
  fix arg4 0x278 u32 addrof arg4 0x0
  fix arg4 0x1000 u32 alt const 0 || const 6
  fix arg4 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg4 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg4 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg4 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg4 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg4 0x1070 u32 alt const 1 || const 0
  fix arg4 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg4 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg4 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg4 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg4 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg4 0x10E0 u32 alt const 2 || const 1
  fix arg4 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg4 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg4 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg4 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg4 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg4 0x1150 u32 alt const 3 || const 2
  fix arg4 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg4 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg4 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg4 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg4 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg4 0x11C0 u32 alt const 4 || const 3
  fix arg4 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg4 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg4 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg4 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg4 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg4 0x1230 u32 alt const 5 || const 4
  fix arg4 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg4 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg4 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg4 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg4 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg4 0x12A0 u32 alt const 6 || const 5
  fix arg4 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg4 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg4 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg4 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg4 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg4 0x1400 u32 const 0
  fix arg4 0x1404 u32 const 1
  fix arg4 0x1408 u32 alt range 1 4 || range 0 5
  fix arg4 0x140C u32 alt range 0 6 || const 9
  fix arg4 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg4 0x1414 u32 alt range 0 8 || any
  fix arg4 0x1448 u32 const 1
  fix arg4 0x144C u32 const 2
  fix arg4 0x1450 u32 alt range 1 4 || range 0 5
  fix arg4 0x1454 u32 alt range 0 6 || const 9
  fix arg4 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg4 0x145C u32 alt range 0 8 || any
  fix arg4 0x1490 u32 const 2
  fix arg4 0x1494 u32 const 3
  fix arg4 0x1498 u32 alt range 1 4 || range 0 5
  fix arg4 0x149C u32 alt range 0 6 || const 9
  fix arg4 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg4 0x14A4 u32 alt range 0 8 || any
  fix arg4 0x14D8 u32 const 3
  fix arg4 0x14DC u32 const 4
  fix arg4 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg4 0x14E4 u32 alt range 0 6 || const 9
  fix arg4 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg4 0x14EC u32 alt range 0 8 || any
  fix arg4 0x1520 u32 const 4
  fix arg4 0x1524 u32 const 5
  fix arg4 0x1528 u32 alt range 1 4 || range 0 5
  fix arg4 0x152C u32 alt range 0 6 || const 9
  fix arg4 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg4 0x1534 u32 alt range 0 8 || any
  fix arg4 0x1568 u32 const 5
  fix arg4 0x156C u32 const 6
  fix arg4 0x1570 u32 alt range 1 4 || range 0 5
  fix arg4 0x1574 u32 alt range 0 6 || const 9
  fix arg4 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg4 0x157C u32 alt range 0 8 || any
  fix arg4 0x15B0 u32 const 6
  fix arg4 0x15B4 u32 const 0
  fix arg4 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg4 0x15BC u32 alt range 0 6 || const 9
  fix arg4 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg4 0x15C4 u32 alt range 0 8 || any
  fix arg4 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg4 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg4 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg4 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg4 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg4 0x340 u16 alt range 0 3 || range 0 6
  fix arg4 0x342 u8 alt const 0 || const 1
  fix arg4 0x343 u8 range 0 4
  fix arg4 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg4 0x346 u16 alt range 0 8 || any
  fix arg4 0x25C u16 alt range 0 3 || range 0 6
  fix arg4 0x25E u8 alt const 0 || const 1
  fix arg4 0x25F u8 range 0 4
  fix arg4 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg4 0x262 u16 alt range 0 8 || any
  fix arg4 0x15C u16 alt range 0 3 || range 0 6
  fix arg4 0x15E u8 alt const 0 || const 1
  fix arg4 0x15F u8 range 0 4
  fix arg4 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg4 0x162 u16 alt range 0 8 || any
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 0218ecf4 force ret=none args=v,in:12  # field camera: target (cam, &v)

# FUN_021BA8E0(R): camera proc: orbit around a moving centre.
fn 021ba8e0 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b82f0 force ret=pick+u32:0/1/2 args=v  # rail: kind (w)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
  stub 021b82f8 force ret=pick+u32:0/0x10/0x40 args=v  # rail: position along the line (w)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 021ba7e0 ret=none args=v,v,v,in:12  # rail: orbit (R, ang, dist, &centre)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 0218ecf4 force ret=none args=v,in:12  # field camera: target (cam, &v)

# FUN_021BA9B4(R): camera proc: face the record point.
fn 021ba9b4 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 0207cfb8 force ret=u16 args=v,v  # FX_Atan2Idx (y, x)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)

# FUN_021BAA30(R): camera proc: behind the RAIL WORK.
fn 021baa30 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 021b8050 force ret=none args=v,out:6  # rail: direction of travel, 3 x s16 (w, &out)
  stub 0207cfb8 force ret=u16 args=v,v  # FX_Atan2Idx (y, x)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)

# FUN_021BAA9C(R): camera proc: behind, with pitch.
fn 021baa9c ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 021b8050 force ret=none args=v,out:6  # rail: direction of travel, 3 x s16 (w, &out)
  stub 0207cfb8 force ret=u16 args=v,v  # FX_Atan2Idx (y, x)
  stub 0207cc30 force ret=u32 args=in:6  # VecFx16 length (v)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)

# FUN_021BAB30(R): camera proc: behind, with position.
fn 021bab30 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 021b8050 force ret=none args=v,out:6  # rail: direction of travel, 3 x s16 (w, &out)
  stub 0207cfb8 force ret=u16 args=v,v  # FX_Atan2Idx (y, x)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 021b8314 force ret=none args=v,out:12  # rail: position without the lateral offset (w, &out)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)

# FUN_021BABB4(R): camera proc: camera of the start point.
fn 021babb4 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b82f0 force ret=pick+u32:0/1/2 args=v  # rail: kind (w)
  stub 021b833c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: start point of the line (w)
  stub 021bb014 ret=none args=v,v,v  # rail: camera of a point (R, w, pt)
  stub 021b8328 force ret=none args=v,v,out:12  # rail: position at along (w, along, &out)
  stub 021b836c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v,v  # rail: camera record of a point (w, pt)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)

# FUN_021BABDC(R): camera proc: camera of the end point.
fn 021babdc ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b82f0 force ret=pick+u32:0/1/2 args=v  # rail: kind (w)
  stub 021b834c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: end point of the line (w)
  stub 021bb014 ret=none args=v,v,v  # rail: camera of a point (R, w, pt)
  stub 021b833c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: start point of the line (w)
  stub 021b8328 force ret=none args=v,v,out:12  # rail: position at along (w, along, &out)
  stub 021b836c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v,v  # rail: camera record of a point (w, pt)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)

# FUN_021BAC04(R): camera proc: angles + vector.
fn 021bac04 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)
  stub 0218ecd4 force ret=none args=v,in:12  # field camera: vector (cam, &v)

# FUN_021BAC74(R): camera proc: angles + vector + position.
fn 021bac74 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 021bac04 ret=none args=v  # rail: camera proc angles + vector (R)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 021b79d4 force ret=none args=v,out:12  # rail: current position (R, &out)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 0218f05c force ret=none args=v  # field camera (cam)
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b830c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v  # rail: camera record of the RAIL WORK (w)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)
  stub 0218ecd4 force ret=none args=v,in:12  # field camera: vector (cam, &v)

# FUN_021BACA8(R): camera proc: interpolated angles + vector.
fn 021baca8 ret=none inputs=150
  nowork
  arg u32 const 0x2E20000
  arg buf 0x3000
  global 020a1b38 0x4000
  fix arg1 0x2 u16 alt const 0 || const 1 || any
  fix arg1 0x4 u16 alt const 0 || const 1
  fix arg1 0x8 u32 addrof arg1 0x2000
  fix arg1 0xC u32 range 3 4
  fix arg1 0x14 u32 alt const 0 || const 0x2E20100 || oneof 0x2E22000 0x2E2207C
  fix arg1 0x18 u32 alt const 0 || const 1
  fix arg1 0x1C u32 addrof arg1 0x100
  fix arg1 0x20 u16 alt const 7 || range 0 7
  fix arg1 0x22 u16 alt const 7 || range 0 7
  fix arg1 0x24 u16 alt const 7 || range 0 7
  fix arg1 0x26 u16 alt const 7 || range 0 7
  fix arg1 0x28 u16 alt const 7 || range 0 7
  fix arg1 0x2A u16 alt const 7 || range 0 7
  fix arg1 0x2C u16 alt const 7 || range 0 7
  fix arg1 0x30 u32 addrof arg1 0x1000
  fix arg1 0x34 u32 addrof arg1 0x1400
  fix arg1 0x38 u32 addrof arg1 0x1600
  fix arg1 0x3C u32 addrof arg1 0x1700
  fix arg1 0x40 u32 addrof arg1 0x1800
  fix arg1 0x44 u32 addrof arg1 0x1840
  fix arg1 0x48 u32 addrof arg1 0x1880
  fix arg1 0x4C u32 alt range 0 16 || any
  fix arg1 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg1 0x100 u16 alt const 0 || const 1 || const 2
  fix arg1 0x102 u16 alt const 0 || const 1
  fix arg1 0x104 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x108 u16 alt const 0 || const 1
  fix arg1 0x10C u32 alt range 0 0x80 || any
  fix arg1 0x110 u32 alt range 0 0x80 || any
  fix arg1 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x118 u32 alt range 0 0x40 || any
  fix arg1 0x124 u32 addrof arg1 0x20
  fix arg1 0x128 u32 addrof arg1 0x50
  fix arg1 0x12C u8 alt const 0 || const 1
  fix arg1 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x12E u16 range 0 5
  fix arg1 0x130 u16 range 0 4
  fix arg1 0x134 u8 alt const 0 || const 1
  fix arg1 0x174 u8 alt const 0 || const 1
  fix arg1 0x178 u32 addrof arg1 0x0
  fix arg1 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg1 0x2002 u16 alt const 0 || const 1
  fix arg1 0x2004 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2008 u16 alt const 0 || const 1
  fix arg1 0x200C u32 alt range 0 0x80 || any
  fix arg1 0x2010 u32 alt range 0 0x80 || any
  fix arg1 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2018 u32 alt range 0 0x40 || any
  fix arg1 0x2024 u32 addrof arg1 0x20
  fix arg1 0x2028 u32 addrof arg1 0x50
  fix arg1 0x202C u8 alt const 0 || const 1
  fix arg1 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x202E u16 range 0 5
  fix arg1 0x2030 u16 range 0 4
  fix arg1 0x2034 u8 alt const 0 || const 1
  fix arg1 0x2074 u8 alt const 0 || const 1
  fix arg1 0x2078 u32 addrof arg1 0x0
  fix arg1 0x207C u16 alt const 0 || const 1 || const 2
  fix arg1 0x207E u16 alt const 0 || const 1
  fix arg1 0x2080 u32 oneof 0x2E21000 0x2E21070 0x2E210E0 0x2E21150 0x2E211C0 0x2E21230 0x2E212A0 0x2E21400 0x2E21448 0x2E21490 0x2E214D8 0x2E21520 0x2E21568 0x2E215B0
  fix arg1 0x2084 u16 alt const 0 || const 1
  fix arg1 0x2088 u32 alt range 0 0x80 || any
  fix arg1 0x208C u32 alt range 0 0x80 || any
  fix arg1 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg1 0x2094 u32 alt range 0 0x40 || any
  fix arg1 0x20A0 u32 addrof arg1 0x20
  fix arg1 0x20A4 u32 addrof arg1 0x50
  fix arg1 0x20A8 u8 alt const 0 || const 1
  fix arg1 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg1 0x20AA u16 range 0 5
  fix arg1 0x20AC u16 range 0 4
  fix arg1 0x20B0 u8 alt const 0 || const 1
  fix arg1 0x20F0 u8 alt const 0 || const 1
  fix arg1 0x20F4 u32 addrof arg1 0x0
  fix arg1 0x20FA u16 const 0
  fix arg1 0x25C u16 range 0 3
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x274 u8 alt const 0 || const 1
  fix arg1 0x276 u8 range 0 4
  fix arg1 0x278 u32 addrof arg1 0x0
  fix arg1 0x1000 u32 alt const 0 || const 6
  fix arg1 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg1 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1070 u32 alt const 1 || const 0
  fix arg1 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg1 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x10E0 u32 alt const 2 || const 1
  fix arg1 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg1 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1150 u32 alt const 3 || const 2
  fix arg1 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg1 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x11C0 u32 alt const 4 || const 3
  fix arg1 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg1 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1230 u32 alt const 5 || const 4
  fix arg1 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg1 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg1 0x12A0 u32 alt const 6 || const 5
  fix arg1 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg1 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg1 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg1 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg1 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1400 u32 const 0
  fix arg1 0x1404 u32 const 1
  fix arg1 0x1408 u32 alt range 1 4 || range 0 5
  fix arg1 0x140C u32 alt range 0 6 || const 9
  fix arg1 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1414 u32 alt range 0 8 || any
  fix arg1 0x1448 u32 const 1
  fix arg1 0x144C u32 const 2
  fix arg1 0x1450 u32 alt range 1 4 || range 0 5
  fix arg1 0x1454 u32 alt range 0 6 || const 9
  fix arg1 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x145C u32 alt range 0 8 || any
  fix arg1 0x1490 u32 const 2
  fix arg1 0x1494 u32 const 3
  fix arg1 0x1498 u32 alt range 1 4 || range 0 5
  fix arg1 0x149C u32 alt range 0 6 || const 9
  fix arg1 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14A4 u32 alt range 0 8 || any
  fix arg1 0x14D8 u32 const 3
  fix arg1 0x14DC u32 const 4
  fix arg1 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg1 0x14E4 u32 alt range 0 6 || const 9
  fix arg1 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x14EC u32 alt range 0 8 || any
  fix arg1 0x1520 u32 const 4
  fix arg1 0x1524 u32 const 5
  fix arg1 0x1528 u32 alt range 1 4 || range 0 5
  fix arg1 0x152C u32 alt range 0 6 || const 9
  fix arg1 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x1534 u32 alt range 0 8 || any
  fix arg1 0x1568 u32 const 5
  fix arg1 0x156C u32 const 6
  fix arg1 0x1570 u32 alt range 1 4 || range 0 5
  fix arg1 0x1574 u32 alt range 0 6 || const 9
  fix arg1 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x157C u32 alt range 0 8 || any
  fix arg1 0x15B0 u32 const 6
  fix arg1 0x15B4 u32 const 0
  fix arg1 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg1 0x15BC u32 alt range 0 6 || const 9
  fix arg1 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg1 0x15C4 u32 alt range 0 8 || any
  fix arg1 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg1 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg1 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg1 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg1 0x340 u16 alt range 0 3 || range 0 6
  fix arg1 0x342 u8 alt const 0 || const 1
  fix arg1 0x343 u8 range 0 4
  fix arg1 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x346 u16 alt range 0 8 || any
  fix arg1 0x25C u16 alt range 0 3 || range 0 6
  fix arg1 0x25E u8 alt const 0 || const 1
  fix arg1 0x25F u8 range 0 4
  fix arg1 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x262 u16 alt range 0 8 || any
  fix arg1 0x15C u16 alt range 0 3 || range 0 6
  fix arg1 0x15E u8 alt const 0 || const 1
  fix arg1 0x15F u8 range 0 4
  fix arg1 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg1 0x162 u16 alt range 0 8 || any
  fix arg1 0x1600 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1624 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1648 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x166C u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1690 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16B4 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x16D8 u32 alt range 0 0x10000 || any rep=9,4
  fix arg1 0x1604 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1608 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1628 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x162C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x164C u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1650 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1670 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1674 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1694 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x1698 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16B8 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16BC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16DC u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  fix arg1 0x16E0 u32 alt const 0 || const 0x8000 || const 0x7FFF || any
  stub 021b8220 force ret=u32 args=v  # rail: the current RAIL WORK (R)
  stub 021b833c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: start point of the line (w)
  stub 021b834c force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v  # rail: end point of the line (w)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec60 force ret=none args=v  # field camera (cam)
  stub 021b836c force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v,v  # rail: camera record of a point (w, pt)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
  stub 021b82f8 force ret=pick+u32:0/0x10/0x40 args=v  # rail: position along the line (w)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)
  stub 0218ecd4 force ret=none args=v,in:12  # field camera: vector (cam, &v)

# FUN_021BAE7C(pa, pb, hit, p, q, side): project hit on a segment.
fn 021bae7c ret=u32 inputs=400
  nowork
  arg u32 oneof 0x2E71000 0x2E71070 0x2E710E0 0x2E71150
  arg u32 oneof 0x2E71000 0x2E71070 0x2E710E0 0x2E71150
  arg u32 const 0x2E70380
  arg u32 const 0x2E703C0
  arg u32 const 0x2E703F0
  arg u32 const 0x2E703A0
  arg buf 0x3000
  fix arg6 0x2 u16 alt const 0 || const 1 || any
  fix arg6 0x4 u16 alt const 0 || const 1
  fix arg6 0x8 u32 addrof arg6 0x2000
  fix arg6 0xC u32 range 3 4
  fix arg6 0x14 u32 alt const 0 || const 0x2E70100 || oneof 0x2E72000 0x2E7207C
  fix arg6 0x18 u32 alt const 0 || const 1
  fix arg6 0x1C u32 addrof arg6 0x100
  fix arg6 0x20 u16 alt const 7 || range 0 7
  fix arg6 0x22 u16 alt const 7 || range 0 7
  fix arg6 0x24 u16 alt const 7 || range 0 7
  fix arg6 0x26 u16 alt const 7 || range 0 7
  fix arg6 0x28 u16 alt const 7 || range 0 7
  fix arg6 0x2A u16 alt const 7 || range 0 7
  fix arg6 0x2C u16 alt const 7 || range 0 7
  fix arg6 0x30 u32 addrof arg6 0x1000
  fix arg6 0x34 u32 addrof arg6 0x1400
  fix arg6 0x38 u32 addrof arg6 0x1600
  fix arg6 0x3C u32 addrof arg6 0x1700
  fix arg6 0x40 u32 addrof arg6 0x1800
  fix arg6 0x44 u32 addrof arg6 0x1840
  fix arg6 0x48 u32 addrof arg6 0x1880
  fix arg6 0x4C u32 alt range 0 16 || any
  fix arg6 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg6 0x100 u16 alt const 0 || const 1 || const 2
  fix arg6 0x102 u16 alt const 0 || const 1
  fix arg6 0x104 u32 oneof 0x2E71000 0x2E71070 0x2E710E0 0x2E71150 0x2E711C0 0x2E71230 0x2E712A0 0x2E71400 0x2E71448 0x2E71490 0x2E714D8 0x2E71520 0x2E71568 0x2E715B0
  fix arg6 0x108 u16 alt const 0 || const 1
  fix arg6 0x10C u32 alt range 0 0x80 || any
  fix arg6 0x110 u32 alt range 0 0x80 || any
  fix arg6 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg6 0x118 u32 alt range 0 0x40 || any
  fix arg6 0x124 u32 addrof arg6 0x20
  fix arg6 0x128 u32 addrof arg6 0x50
  fix arg6 0x12C u8 alt const 0 || const 1
  fix arg6 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg6 0x12E u16 range 0 5
  fix arg6 0x130 u16 range 0 4
  fix arg6 0x134 u8 alt const 0 || const 1
  fix arg6 0x174 u8 alt const 0 || const 1
  fix arg6 0x178 u32 addrof arg6 0x0
  fix arg6 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg6 0x2002 u16 alt const 0 || const 1
  fix arg6 0x2004 u32 oneof 0x2E71000 0x2E71070 0x2E710E0 0x2E71150 0x2E711C0 0x2E71230 0x2E712A0 0x2E71400 0x2E71448 0x2E71490 0x2E714D8 0x2E71520 0x2E71568 0x2E715B0
  fix arg6 0x2008 u16 alt const 0 || const 1
  fix arg6 0x200C u32 alt range 0 0x80 || any
  fix arg6 0x2010 u32 alt range 0 0x80 || any
  fix arg6 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg6 0x2018 u32 alt range 0 0x40 || any
  fix arg6 0x2024 u32 addrof arg6 0x20
  fix arg6 0x2028 u32 addrof arg6 0x50
  fix arg6 0x202C u8 alt const 0 || const 1
  fix arg6 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg6 0x202E u16 range 0 5
  fix arg6 0x2030 u16 range 0 4
  fix arg6 0x2034 u8 alt const 0 || const 1
  fix arg6 0x2074 u8 alt const 0 || const 1
  fix arg6 0x2078 u32 addrof arg6 0x0
  fix arg6 0x207C u16 alt const 0 || const 1 || const 2
  fix arg6 0x207E u16 alt const 0 || const 1
  fix arg6 0x2080 u32 oneof 0x2E71000 0x2E71070 0x2E710E0 0x2E71150 0x2E711C0 0x2E71230 0x2E712A0 0x2E71400 0x2E71448 0x2E71490 0x2E714D8 0x2E71520 0x2E71568 0x2E715B0
  fix arg6 0x2084 u16 alt const 0 || const 1
  fix arg6 0x2088 u32 alt range 0 0x80 || any
  fix arg6 0x208C u32 alt range 0 0x80 || any
  fix arg6 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg6 0x2094 u32 alt range 0 0x40 || any
  fix arg6 0x20A0 u32 addrof arg6 0x20
  fix arg6 0x20A4 u32 addrof arg6 0x50
  fix arg6 0x20A8 u8 alt const 0 || const 1
  fix arg6 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg6 0x20AA u16 range 0 5
  fix arg6 0x20AC u16 range 0 4
  fix arg6 0x20B0 u8 alt const 0 || const 1
  fix arg6 0x20F0 u8 alt const 0 || const 1
  fix arg6 0x20F4 u32 addrof arg6 0x0
  fix arg6 0x20FA u16 const 0
  fix arg6 0x25C u16 range 0 3
  fix arg6 0x25E u8 alt const 0 || const 1
  fix arg6 0x274 u8 alt const 0 || const 1
  fix arg6 0x276 u8 range 0 4
  fix arg6 0x278 u32 addrof arg6 0x0
  fix arg6 0x1000 u32 alt const 0 || const 6
  fix arg6 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg6 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg6 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg6 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg6 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg6 0x1070 u32 alt const 1 || const 0
  fix arg6 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg6 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg6 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg6 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg6 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg6 0x10E0 u32 alt const 2 || const 1
  fix arg6 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg6 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg6 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg6 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg6 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg6 0x1150 u32 alt const 3 || const 2
  fix arg6 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg6 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg6 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg6 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg6 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg6 0x11C0 u32 alt const 4 || const 3
  fix arg6 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg6 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg6 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg6 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg6 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg6 0x1230 u32 alt const 5 || const 4
  fix arg6 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg6 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg6 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg6 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg6 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg6 0x12A0 u32 alt const 6 || const 5
  fix arg6 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg6 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg6 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg6 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg6 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg6 0x1400 u32 const 0
  fix arg6 0x1404 u32 const 1
  fix arg6 0x1408 u32 alt range 1 4 || range 0 5
  fix arg6 0x140C u32 alt range 0 6 || const 9
  fix arg6 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg6 0x1414 u32 alt range 0 8 || any
  fix arg6 0x1448 u32 const 1
  fix arg6 0x144C u32 const 2
  fix arg6 0x1450 u32 alt range 1 4 || range 0 5
  fix arg6 0x1454 u32 alt range 0 6 || const 9
  fix arg6 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg6 0x145C u32 alt range 0 8 || any
  fix arg6 0x1490 u32 const 2
  fix arg6 0x1494 u32 const 3
  fix arg6 0x1498 u32 alt range 1 4 || range 0 5
  fix arg6 0x149C u32 alt range 0 6 || const 9
  fix arg6 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg6 0x14A4 u32 alt range 0 8 || any
  fix arg6 0x14D8 u32 const 3
  fix arg6 0x14DC u32 const 4
  fix arg6 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg6 0x14E4 u32 alt range 0 6 || const 9
  fix arg6 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg6 0x14EC u32 alt range 0 8 || any
  fix arg6 0x1520 u32 const 4
  fix arg6 0x1524 u32 const 5
  fix arg6 0x1528 u32 alt range 1 4 || range 0 5
  fix arg6 0x152C u32 alt range 0 6 || const 9
  fix arg6 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg6 0x1534 u32 alt range 0 8 || any
  fix arg6 0x1568 u32 const 5
  fix arg6 0x156C u32 const 6
  fix arg6 0x1570 u32 alt range 1 4 || range 0 5
  fix arg6 0x1574 u32 alt range 0 6 || const 9
  fix arg6 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg6 0x157C u32 alt range 0 8 || any
  fix arg6 0x15B0 u32 const 6
  fix arg6 0x15B4 u32 const 0
  fix arg6 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg6 0x15BC u32 alt range 0 6 || const 9
  fix arg6 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg6 0x15C4 u32 alt range 0 8 || any
  fix arg6 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg6 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg6 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg6 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg6 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg6 0x340 u16 alt range 0 3 || range 0 6
  fix arg6 0x342 u8 alt const 0 || const 1
  fix arg6 0x343 u8 range 0 4
  fix arg6 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg6 0x346 u16 alt range 0 8 || any
  fix arg6 0x25C u16 alt range 0 3 || range 0 6
  fix arg6 0x25E u8 alt const 0 || const 1
  fix arg6 0x25F u8 range 0 4
  fix arg6 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg6 0x262 u16 alt range 0 8 || any
  fix arg6 0x15C u16 alt range 0 3 || range 0 6
  fix arg6 0x15E u8 alt const 0 || const 1
  fix arg6 0x15F u8 range 0 4
  fix arg6 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg6 0x162 u16 alt range 0 8 || any
  fix arg6 0x1034 u32 range 0 0x4000
  fix arg6 0x10A4 u32 range 0 0x4000
  fix arg6 0x1114 u32 range 0 0x4000
  fix arg6 0x1184 u32 range 0 0x4000
  fix arg6 0x11F4 u32 range 0 0x4000
  fix arg6 0x1264 u32 range 0 0x4000
  fix arg6 0x12D4 u32 range 0 0x4000
  fix arg6 0x384 u32 alt range 0 0x4000 || range 0x9000 0xA000 || oneof 0x7FFF 0x8000 0x8001
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,outm:12:0xFF/0x0F/0/0/0/0/0/0/0xFF/0x0F/0/0  # VEC_Normalize (a, &out): flat, so hits occur
  stub 0207cbd0 force ret=pick+u32:0x1000/0x2000/0x3000 args=in:12  # VEC_Mag (a)
  stub 0207c768 force ret=pick+u32:0/0x8000/0x10000/0x11000/0x20000 args=v,v  # FX_Div (a, b)
  stub 0207ca68 force ret=pick+u32:0/0x800/0x1000/0x1001/0xFFFFFFFF args=in:12,in:12  # VEC_DotProduct (a, b)
  stub 0207cae8 force ret=none args=in:12,in:12,out:12  # VEC_CrossProduct (a, b, &out)
  stub 0207c9d0 force ret=none args=in:12,in:12,outm:12:0xFF/0xFF/0/0/0/0/0/0/0xFF/0xFF/0/0  # VEC_Add (a, b, &out): height 0
  stub 0207cf3c force ret=pick+u32:0x1000/0x2000/0xFFFFF/0x100000 args=in:12,in:12  # VEC_Distance (a, b)

# FUN_021BB014(R, w, pt): camera of a point.
fn 021bb014 ret=none inputs=60
  nowork
  arg u32 const 0x2E40000
  arg u32 const 0x2E40100
  arg u32 oneof 0x2E41000 0x2E41070 0x2E410E0 0x2E41150
  arg buf 0x3000
  fix arg3 0x2 u16 alt const 0 || const 1 || any
  fix arg3 0x4 u16 alt const 0 || const 1
  fix arg3 0x8 u32 addrof arg3 0x2000
  fix arg3 0xC u32 range 3 4
  fix arg3 0x14 u32 alt const 0 || const 0x2E40100 || oneof 0x2E42000 0x2E4207C
  fix arg3 0x18 u32 alt const 0 || const 1
  fix arg3 0x1C u32 addrof arg3 0x100
  fix arg3 0x20 u16 alt const 7 || range 0 7
  fix arg3 0x22 u16 alt const 7 || range 0 7
  fix arg3 0x24 u16 alt const 7 || range 0 7
  fix arg3 0x26 u16 alt const 7 || range 0 7
  fix arg3 0x28 u16 alt const 7 || range 0 7
  fix arg3 0x2A u16 alt const 7 || range 0 7
  fix arg3 0x2C u16 alt const 7 || range 0 7
  fix arg3 0x30 u32 addrof arg3 0x1000
  fix arg3 0x34 u32 addrof arg3 0x1400
  fix arg3 0x38 u32 addrof arg3 0x1600
  fix arg3 0x3C u32 addrof arg3 0x1700
  fix arg3 0x40 u32 addrof arg3 0x1800
  fix arg3 0x44 u32 addrof arg3 0x1840
  fix arg3 0x48 u32 addrof arg3 0x1880
  fix arg3 0x4C u32 alt range 0 16 || any
  fix arg3 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg3 0x100 u16 alt const 0 || const 1 || const 2
  fix arg3 0x102 u16 alt const 0 || const 1
  fix arg3 0x104 u32 oneof 0x2E41000 0x2E41070 0x2E410E0 0x2E41150 0x2E411C0 0x2E41230 0x2E412A0 0x2E41400 0x2E41448 0x2E41490 0x2E414D8 0x2E41520 0x2E41568 0x2E415B0
  fix arg3 0x108 u16 alt const 0 || const 1
  fix arg3 0x10C u32 alt range 0 0x80 || any
  fix arg3 0x110 u32 alt range 0 0x80 || any
  fix arg3 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg3 0x118 u32 alt range 0 0x40 || any
  fix arg3 0x124 u32 addrof arg3 0x20
  fix arg3 0x128 u32 addrof arg3 0x50
  fix arg3 0x12C u8 alt const 0 || const 1
  fix arg3 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg3 0x12E u16 range 0 5
  fix arg3 0x130 u16 range 0 4
  fix arg3 0x134 u8 alt const 0 || const 1
  fix arg3 0x174 u8 alt const 0 || const 1
  fix arg3 0x178 u32 addrof arg3 0x0
  fix arg3 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg3 0x2002 u16 alt const 0 || const 1
  fix arg3 0x2004 u32 oneof 0x2E41000 0x2E41070 0x2E410E0 0x2E41150 0x2E411C0 0x2E41230 0x2E412A0 0x2E41400 0x2E41448 0x2E41490 0x2E414D8 0x2E41520 0x2E41568 0x2E415B0
  fix arg3 0x2008 u16 alt const 0 || const 1
  fix arg3 0x200C u32 alt range 0 0x80 || any
  fix arg3 0x2010 u32 alt range 0 0x80 || any
  fix arg3 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg3 0x2018 u32 alt range 0 0x40 || any
  fix arg3 0x2024 u32 addrof arg3 0x20
  fix arg3 0x2028 u32 addrof arg3 0x50
  fix arg3 0x202C u8 alt const 0 || const 1
  fix arg3 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg3 0x202E u16 range 0 5
  fix arg3 0x2030 u16 range 0 4
  fix arg3 0x2034 u8 alt const 0 || const 1
  fix arg3 0x2074 u8 alt const 0 || const 1
  fix arg3 0x2078 u32 addrof arg3 0x0
  fix arg3 0x207C u16 alt const 0 || const 1 || const 2
  fix arg3 0x207E u16 alt const 0 || const 1
  fix arg3 0x2080 u32 oneof 0x2E41000 0x2E41070 0x2E410E0 0x2E41150 0x2E411C0 0x2E41230 0x2E412A0 0x2E41400 0x2E41448 0x2E41490 0x2E414D8 0x2E41520 0x2E41568 0x2E415B0
  fix arg3 0x2084 u16 alt const 0 || const 1
  fix arg3 0x2088 u32 alt range 0 0x80 || any
  fix arg3 0x208C u32 alt range 0 0x80 || any
  fix arg3 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg3 0x2094 u32 alt range 0 0x40 || any
  fix arg3 0x20A0 u32 addrof arg3 0x20
  fix arg3 0x20A4 u32 addrof arg3 0x50
  fix arg3 0x20A8 u8 alt const 0 || const 1
  fix arg3 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg3 0x20AA u16 range 0 5
  fix arg3 0x20AC u16 range 0 4
  fix arg3 0x20B0 u8 alt const 0 || const 1
  fix arg3 0x20F0 u8 alt const 0 || const 1
  fix arg3 0x20F4 u32 addrof arg3 0x0
  fix arg3 0x20FA u16 const 0
  fix arg3 0x25C u16 range 0 3
  fix arg3 0x25E u8 alt const 0 || const 1
  fix arg3 0x274 u8 alt const 0 || const 1
  fix arg3 0x276 u8 range 0 4
  fix arg3 0x278 u32 addrof arg3 0x0
  fix arg3 0x1000 u32 alt const 0 || const 6
  fix arg3 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg3 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg3 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg3 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg3 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg3 0x1070 u32 alt const 1 || const 0
  fix arg3 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg3 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg3 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg3 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg3 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg3 0x10E0 u32 alt const 2 || const 1
  fix arg3 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg3 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg3 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg3 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg3 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg3 0x1150 u32 alt const 3 || const 2
  fix arg3 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg3 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg3 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg3 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg3 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg3 0x11C0 u32 alt const 4 || const 3
  fix arg3 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg3 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg3 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg3 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg3 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg3 0x1230 u32 alt const 5 || const 4
  fix arg3 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg3 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg3 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg3 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg3 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg3 0x12A0 u32 alt const 6 || const 5
  fix arg3 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg3 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg3 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg3 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg3 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg3 0x1400 u32 const 0
  fix arg3 0x1404 u32 const 1
  fix arg3 0x1408 u32 alt range 1 4 || range 0 5
  fix arg3 0x140C u32 alt range 0 6 || const 9
  fix arg3 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg3 0x1414 u32 alt range 0 8 || any
  fix arg3 0x1448 u32 const 1
  fix arg3 0x144C u32 const 2
  fix arg3 0x1450 u32 alt range 1 4 || range 0 5
  fix arg3 0x1454 u32 alt range 0 6 || const 9
  fix arg3 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg3 0x145C u32 alt range 0 8 || any
  fix arg3 0x1490 u32 const 2
  fix arg3 0x1494 u32 const 3
  fix arg3 0x1498 u32 alt range 1 4 || range 0 5
  fix arg3 0x149C u32 alt range 0 6 || const 9
  fix arg3 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg3 0x14A4 u32 alt range 0 8 || any
  fix arg3 0x14D8 u32 const 3
  fix arg3 0x14DC u32 const 4
  fix arg3 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg3 0x14E4 u32 alt range 0 6 || const 9
  fix arg3 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg3 0x14EC u32 alt range 0 8 || any
  fix arg3 0x1520 u32 const 4
  fix arg3 0x1524 u32 const 5
  fix arg3 0x1528 u32 alt range 1 4 || range 0 5
  fix arg3 0x152C u32 alt range 0 6 || const 9
  fix arg3 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg3 0x1534 u32 alt range 0 8 || any
  fix arg3 0x1568 u32 const 5
  fix arg3 0x156C u32 const 6
  fix arg3 0x1570 u32 alt range 1 4 || range 0 5
  fix arg3 0x1574 u32 alt range 0 6 || const 9
  fix arg3 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg3 0x157C u32 alt range 0 8 || any
  fix arg3 0x15B0 u32 const 6
  fix arg3 0x15B4 u32 const 0
  fix arg3 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg3 0x15BC u32 alt range 0 6 || const 9
  fix arg3 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg3 0x15C4 u32 alt range 0 8 || any
  fix arg3 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg3 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg3 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg3 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg3 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg3 0x340 u16 alt range 0 3 || range 0 6
  fix arg3 0x342 u8 alt const 0 || const 1
  fix arg3 0x343 u8 range 0 4
  fix arg3 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg3 0x346 u16 alt range 0 8 || any
  fix arg3 0x25C u16 alt range 0 3 || range 0 6
  fix arg3 0x25E u8 alt const 0 || const 1
  fix arg3 0x25F u8 range 0 4
  fix arg3 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg3 0x262 u16 alt range 0 8 || any
  fix arg3 0x15C u16 alt range 0 3 || range 0 6
  fix arg3 0x15E u8 alt const 0 || const 1
  fix arg3 0x15F u8 range 0 4
  fix arg3 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg3 0x162 u16 alt range 0 8 || any
  stub 021b833c force ret=pick:0x2E41000/0x2E41070 args=v  # rail: start point of the line (w)
  stub 021b8328 force ret=none args=v,v,out:12  # rail: position at along (w, along, &out)
  stub 021b836c force ret=pick:0x021D9A0C/0x2E41600/0x2E41624/0x2E41648/0x2E4166C/0x2E41690/0x2E416B4/0x2E416D8 args=v,v  # rail: camera record of a point (w, pt)
  stub 021b821c force ret=u32 args=v  # rail: the field camera (R + 0x10)
  stub 0218e124 force ret=none args=v,v  # field camera: mode (cam, m)
  stub 0218ec54 force ret=none args=v  # field camera (cam)
  stub 0218ec90 force ret=none args=v,in:12  # field camera: position (cam, &v)
  stub 0218ed34 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed48 force ret=none args=v,v  # field camera: angle (cam, u16)
  stub 0218ed58 force ret=none args=v,v  # field camera: distance (cam, d)
  stub 021b82fc force ret=pick+u32:0x10/0x20/0x40/0x80 args=v  # rail: line length (w)
