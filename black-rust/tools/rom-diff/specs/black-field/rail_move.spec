# fndiff spec (F3b): `field_rail.c` part 2 (overlay 21 Thumb code 0x021B88DC..0x021B9B7C; F1 row 21.53): the line movement 021B88DC
# (one step along / back / sideways, onto the next line at the ends), OBJ reset, active / camera record / position / LOCATION of a
# RAIL WORK, line length and interpolated half width (021B9280), the blocked-line list (18 u16), directions, the side step and the
# crossing checks, the point consistency check, the segment-plane hit (021B9828) and the slerp / interpolation (021B9A50 / 021B9B24).
# Run: tools/check-fn.sh black-field rail_move --per-function
#
# Function list: as the `rail` spec; not specced: 021B9366 (padding before 021B9368), 021B95A0 / 021B95C8 (labels inside 021B9530 /
# 021B95C0, `bl` targets from another overlay's code that the field-set scan decodes).
#
# The arena (last argument `buf 0x3000`) is the RAIL universe of rail.spec (header there). Callees in the `rail` and `rail_cam` areas
# and the NitroSDK vector / fx functions (D3 math hooks) are `force` stubs; FX_Mul products are inline in the code (real on both
# sides). The sine table FX_SinCosTable_ (ARM9 0x020A1B38) is a `global` (random contents, the same on both sides) in the blocks that
# read it (021B9A50 / 021B9B24; FX_AcosIdx answers u16 so the index stays in the table, `t` is 0..1.0).
#
# Conventions (headers of the F5b / F6b / F14 specs): all blocks `nowork`; closure stubs (checked with every function treated as
# ported: no fault, stray write or unstubbed callee); stack locals reach stubs as `in:` / `out:`.
area rail_move
set field

# FUN_021B88DC(w, dir, step): one step of the line movement.
fn 021b88dc ret=u32 inputs=3000
  nowork
  arg u32 oneof 0x2E40100 0x2E42000 0x2E4207C
  arg u32 alt range 0 5 || any
  arg u32 oneof 1 2 4 8 16
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
  fix arg3 0x10C u32 alt range 0 0x80 || range 0xFFFFFF80 0xFFFFFFFF || oneof 0x10 0x20 0x30 0xFFFFFFF0 0xFFFFFFE0
  fix arg3 0x114 u32 alt range 0 0x80 || range 0xFFFFFF80 0xFFFFFFFF || oneof 0x10 0x20 0x30 0xFFFFFFF0 0xFFFFFFE0
  fix arg3 0x200C u32 alt range 0 0x80 || range 0xFFFFFF80 0xFFFFFFFF || oneof 0x10 0x20 0x30 0xFFFFFFF0 0xFFFFFFE0
  fix arg3 0x2014 u32 alt range 0 0x80 || range 0xFFFFFF80 0xFFFFFFFF || oneof 0x10 0x20 0x30 0xFFFFFFF0 0xFFFFFFE0
  fix arg3 0x2088 u32 alt range 0 0x80 || range 0xFFFFFF80 0xFFFFFFFF || oneof 0x10 0x20 0x30 0xFFFFFFF0 0xFFFFFFE0
  fix arg3 0x2090 u32 alt range 0 0x80 || range 0xFFFFFF80 0xFFFFFFFF || oneof 0x10 0x20 0x30 0xFFFFFFF0 0xFFFFFFE0
  fix arg3 0x10C u32 alt range 0 0x80 || range 0xFFFFFF80 0xFFFFFFFF || oneof 0x10 0x20 0x30 0xFFFFFFF0 0xFFFFFFE0 || oneof 0xF 0xE 0xC 0x8 0x1F 0x1E 0x1C 0x18
  fix arg3 0x200C u32 alt range 0 0x80 || range 0xFFFFFF80 0xFFFFFFFF || oneof 0x10 0x20 0x30 0xFFFFFFF0 0xFFFFFFE0 || oneof 0xF 0xE 0xC 0x8 0x1F 0x1E 0x1C 0x18
  fix arg3 0x2088 u32 alt range 0 0x80 || range 0xFFFFFF80 0xFFFFFFFF || oneof 0x10 0x20 0x30 0xFFFFFFF0 0xFFFFFFE0 || oneof 0xF 0xE 0xC 0x8 0x1F 0x1E 0x1C 0x18
  fix arg3 0x110 u32 alt const 0x10 || const 0x20 || range 0 0x80
  fix arg3 0x2010 u32 alt const 0x10 || const 0x20 || range 0 0x80
  fix arg3 0x208C u32 alt const 0x10 || const 0x20 || range 0 0x80
  fix arg3 0x104 u32 oneof 0x2E41400 0x2E41448 0x2E41490 0x2E414D8 0x2E41520 0x2E41568 0x2E415B0
  fix arg3 0x2004 u32 oneof 0x2E41400 0x2E41448 0x2E41490 0x2E414D8 0x2E41520 0x2E41568 0x2E415B0
  fix arg3 0x2080 u32 oneof 0x2E41400 0x2E41448 0x2E41490 0x2E414D8 0x2E41520 0x2E41568 0x2E415B0
  stub 021b93d4 ret=pick:0/1/2/3/4 args=v  # rail: direction back (d)
  stub 021b9404 ret=pick:0/1/2/3/4 args=v  # rail: direction right (d)
  stub 021b9434 ret=pick:0/1/2/3/4 args=v  # rail: direction left (d)
  stub 021b9530 ret=none args=v,v,v,v,v  # rail: side step (blk, w, da, dl, dir)
  stub 021b9338 ret=u32 args=v,v,v,v  # rail: clamp the lateral offset (w, ln, len, dir) -> dir
  stub 021b87d8 force ret=pick:0x2E41000/0x2E41070/0x2E410E0/0x2E41150/0x2E411C0/0x2E41230/0x2E412A0 args=v,v  # rail: point (rd, i)
  stub 021b8514 force ret=pick:0/0/0/0/0x2E41400/0x2E41448/0x2E41490/0x2E414D8/0x2E41520/0x2E41568/0x2E415B0 args=v,v,v,v,v  # rail: line of a point in a direction (pt, d, rd, blk, open): often none
  stub 021b8674 force ret=pick:0/1 args=v,v,v  # rail: line starts at the point (rd, ln, p)
  stub 021b96ac ret=pick:0/1 args=v,v  # rail: point line list consistent (pt, rd)
  stub 021b9278 ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b854c force ret=pick:0/0/0/1/2/3/4 args=v,v,v,v,v,v,v  # rail: move onto a line -> dir or 0 (often 0)
  stub 021b868c force ret=pick:0/1 args=v,v,v  # rail: line ends at the point (rd, ln, p)
  stub 021b94d8 ret=pick:0/1 args=v,v,v  # rail: line blocked (blk, ln, rd)
  stub 021b86a4 force ret=none args=v,v,outv:4:0/0x10/0x40/0x100,outv:4:0/0x10/0x40/0x100,v,v  # rail: widths around a line start (rd, ln, &side, &start, blk, unit)
  stub 021b87fc force ret=pick:0x2E41400/0x2E41448/0x2E41490/0x2E414D8/0x2E41520/0x2E41568/0x2E415B0 args=v,v  # rail: line (rd, i)
  stub 021b94c0 ret=pick:0/1 args=v,v  # rail: line blocked (blk, id)

# FUN_021B9164(w, rd, R): reset a RAIL WORK.
fn 021b9164 ret=none inputs=20
  nowork
  arg u32 const 0x2E40100
  arg u32 const 0x2E40020
  arg u32 const 0x2E40000
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

# FUN_021B919C(w, rd): reset with w + 0x78.
fn 021b919c ret=none inputs=20
  nowork
  arg u32 const 0x2E30100
  arg u32 const 0x2E30020
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

# FUN_021B91A8(o): u16 +2.
fn 021b91a8 ret=u32 inputs=20
  nowork
  arg u32 const 0x2E22000
  arg buf 0x3000
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

# FUN_021B91AC(w): active.
fn 021b91ac ret=u32 inputs=100
  nowork
  arg u32 oneof 0x2E20100 0x2E22000 0x2E2207C
  arg buf 0x3000
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

# FUN_021B91C8(w): camera record.
fn 021b91c8 ret=u32 inputs=60
  nowork
  arg u32 oneof 0x2E20100 0x2E22000 0x2E2207C
  arg buf 0x3000
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
  stub 021b8820 force ret=pick:0x021D9A0C/0x2E21600/0x2E21624/0x2E21648/0x2E2166C/0x2E21690/0x2E216B4/0x2E216D8 args=v,v  # rail: camera record (rd, i)

# FUN_021B91F8(w, out): position.
fn 021b91f8 ret=none inputs=200
  nowork
  arg u32 oneof 0x2E30100 0x2E32000 0x2E3207C
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
  stub 021b8848 force ret=pick:0x021D9A30/0x2E31700/0x2E31724/0x2E31748/0x2E3176C/0x2E31790/0x2E317B4/0x2E317D8 args=v,v  # rail: line record (rd, i)
  stub 021b8894 force ret=pick:0/0x02EE0021 args=v,v  # rail: position proc (rd, i)
  stub 02ee0020 force ret=none args=v,out:12  # position proc (w, &pos)

# FUN_021B9238(w, out): LOCATION.
fn 021b9238 ret=none inputs=100
  nowork
  arg u32 oneof 0x2E30100 0x2E32000 0x2E3207C
  arg u32 const 0x2E30360
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
  stub 021b97d0 ret=pick:0/1/2/3 args=v,v  # rail: point index (rd, p)
  stub 021b97fc ret=pick:0/1/2/3 args=v,v  # rail: line index (rd, ln)

# FUN_021B9278(ln): length * 16.
fn 021b9278 ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E21400 0x2E21448 0x2E21490 0x2E214D8
  arg buf 0x3000
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

# FUN_021B9280(ln, pos, len, rd): half width at pos.
fn 021b9280 ret=u32 inputs=600
  nowork
  arg u32 oneof 0x2E51400 0x2E51448 0x2E51490 0x2E514D8
  arg u32 alt range 0 0x80 || any
  arg u32 alt range 0 0x80 || oneof 0 16 17
  arg u32 const 0x2E50020
  arg buf 0x3000
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
  fix arg4 0x1020 u32 alt range 0 0x40 || range 0xFFFFFFF0 0xFFFFFFFF || any rep=4,4
  fix arg4 0x1090 u32 alt range 0 0x40 || range 0xFFFFFFF0 0xFFFFFFFF || any rep=4,4
  fix arg4 0x1100 u32 alt range 0 0x40 || range 0xFFFFFFF0 0xFFFFFFFF || any rep=4,4
  fix arg4 0x1170 u32 alt range 0 0x40 || range 0xFFFFFFF0 0xFFFFFFFF || any rep=4,4
  fix arg4 0x11E0 u32 alt range 0 0x40 || range 0xFFFFFFF0 0xFFFFFFFF || any rep=4,4
  fix arg4 0x1250 u32 alt range 0 0x40 || range 0xFFFFFFF0 0xFFFFFFFF || any rep=4,4
  fix arg4 0x12C0 u32 alt range 0 0x40 || range 0xFFFFFFF0 0xFFFFFFFF || any rep=4,4

# FUN_021B9338(w, ln, len, dir): clamp the lateral offset.
fn 021b9338 ret=u32 inputs=300
  nowork
  arg u32 const 0x2E50100
  arg u32 oneof 0x2E51400 0x2E51448 0x2E51490 0x2E514D8
  arg u32 alt range 0 0x80 || any
  arg u32 any
  arg buf 0x3000
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
  stub 021b9280 ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)

# FUN_021B9368(R, loc, dir, n, out): walk 16 frames.
fn 021b9368 ret=u32 inputs=300
  nowork
  arg u32 const 0x2E60000
  arg u32 const 0x2E60340
  arg u32 range 0 5
  arg u32 alt range 0 16 || any
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
  fix arg5 0x340 u32 alt const 0x10000 || any
  fix arg5 0x15C u32 alt const 0x10000 || any
  fix arg5 0x344 u32 alt const 0 || any
  fix arg5 0x160 u32 alt const 0 || any
  stub 021b7ee4 force ret=none args=v,in:8  # rail: place the RAIL WORK at a LOCATION (w, &loc)
  stub 021b80e0 force ret=pick:0/1 args=v,v,v  # rail: start a move (w, speed, dir)
  stub 021b810c force ret=none args=v  # rail: one frame of a move (w)
  stub 021b8028 force ret=none args=v,out:12  # rail: position (w, &out)
  stub 021b7de0 force ret=pick:0/1 args=v,in:8  # rail: LOCATION valid (w, &loc)

# FUN_021B93D4(d): direction.
fn 021b93d4 ret=u32 inputs=60
  nowork
  arg u32 alt range 0 6 || any
  arg buf 0x3000
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

# FUN_021B9404(d): direction.
fn 021b9404 ret=u32 inputs=60
  nowork
  arg u32 alt range 0 6 || any
  arg buf 0x3000
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

# FUN_021B9434(d): direction.
fn 021b9434 ret=u32 inputs=60
  nowork
  arg u32 alt range 0 6 || any
  arg buf 0x3000
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

# FUN_021B9464(blk): clear.
fn 021b9464 ret=none inputs=10
  nowork
  arg u32 const 0x2E20050
  arg buf 0x3000
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

# FUN_021B9478(blk, id): block (list entries mostly in use, so a full list occurs).
fn 021b9478 ret=u32 inputs=1000
  nowork
  arg u32 const 0x2E30050
  arg u32 alt range 0 6 || any
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
  fix arg2 0x50 u16 alt const 0xFFFF || range 0 5 || range 0 5 || range 0 5 || range 0 5 || range 0 5 || range 0 5 || range 0 5 || range 0 5 || range 0 5 || range 0 5 || range 0 5 rep=18,2

# FUN_021B949C(blk, id): unblock.
fn 021b949c ret=u32 inputs=100
  nowork
  arg u32 const 0x2E30050
  arg u32 alt range 0 6 || any
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

# FUN_021B94C0(blk, id): blocked.
fn 021b94c0 ret=u32 inputs=100
  nowork
  arg u32 const 0x2E30050
  arg u32 alt range 0 6 || any
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

# FUN_021B94D8(blk, ln, rd): line blocked.
fn 021b94d8 ret=u32 inputs=200
  nowork
  arg u32 const 0x2E40050
  arg u32 oneof 0x2E41400 0x2E41448 0x2E41490 0x2E414D8
  arg u32 const 0x2E40020
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
  stub 021b94c0 ret=pick:0/1 args=v,v  # rail: line blocked (blk, id)

# FUN_021B9510(blk, rd, i): line unless blocked.
fn 021b9510 ret=u32 inputs=200
  nowork
  arg u32 const 0x2E40050
  arg u32 const 0x2E40020
  arg u32 alt range 0 7 || any
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
  stub 021b94c0 ret=pick:0/1 args=v,v  # rail: line blocked (blk, id)
  stub 021b87fc force ret=pick:0x2E41400/0x2E41448/0x2E41490/0x2E414D8/0x2E41520/0x2E41568/0x2E415B0 args=v,v  # rail: line (rd, i)

# FUN_021B9530(blk, w, da, dl, dir): side step.
fn 021b9530 ret=none inputs=600
  nowork
  arg u32 const 0x2E60050
  arg u32 oneof 0x2E60100 0x2E62000 0x2E6207C
  arg u32 alt range 0 16 || range 0xFFFFFFF0 0xFFFFFFFF
  arg u32 alt range 0 16 || range 0xFFFFFFF0 0xFFFFFFFF
  arg u32 range 0 5
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
  fix arg5 0x10C u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF
  fix arg5 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF
  fix arg5 0x200C u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF
  fix arg5 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF
  fix arg5 0x2088 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF
  fix arg5 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF
  stub 021b94d8 ret=pick:0/1 args=v,v,v  # rail: line blocked (blk, ln, rd)
  stub 021b87d8 force ret=pick:0x2E61000/0x2E61070/0x2E610E0/0x2E61150/0x2E611C0/0x2E61230/0x2E612A0 args=v,v  # rail: point (rd, i)
  stub 021b8514 force ret=pick:0/0x2E61400/0x2E61448/0x2E61490/0x2E614D8/0x2E61520/0x2E61568/0x2E615B0 args=v,v,v,v,v  # rail: line of a point in a direction (pt, d, rd, blk, open)
  stub 021b86a4 force ret=none args=v,v,outv:4:0/0x10/0x40/0x100,outv:4:0/0x10/0x40/0x100,v,v  # rail: widths around a line start (rd, ln, &side, &start, blk, unit)
  stub 021b94c0 ret=pick:0/1 args=v,v  # rail: line blocked (blk, id)

# FUN_021B95C0(blk, ln, pos, lat, rd, unit): allowed.
fn 021b95c0 ret=u32 inputs=400
  nowork
  arg u32 const 0x2E70050
  arg u32 oneof 0x2E71400 0x2E71448 0x2E71490 0x2E714D8
  arg u32 alt range 0 0x100 || any
  arg u32 alt range 0 0x100 || range 0xFFFFFF00 0xFFFFFFFF
  arg u32 const 0x2E70020
  arg u32 any
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
  stub 021b94d8 ret=pick:0/1 args=v,v,v  # rail: line blocked (blk, ln, rd)
  stub 021b86a4 force ret=none args=v,v,outv:4:0/0x10/0x40/0x100,outv:4:0/0x10/0x40/0x100,v,v  # rail: widths around a line start (rd, ln, &side, &start, blk, unit)
  stub 021b94c0 ret=pick:0/1 args=v,v  # rail: line blocked (blk, id)

# FUN_021B960C(pt, ln, rd): direction of ln at pt.
fn 021b960c ret=u32 inputs=200
  nowork
  arg u32 oneof 0x2E41000 0x2E41070 0x2E410E0 0x2E41150
  arg u32 oneof 0x2E41400 0x2E41448 0x2E41490 0x2E414D8
  arg u32 const 0x2E40020
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
  stub 021b87fc force ret=pick:0x2E41400/0x2E41448/0x2E41490/0x2E414D8/0x2E41520/0x2E41568/0x2E415B0 args=v,v  # rail: line (rd, i)

# FUN_021B963C(ln, rd): line consistent at both ends (directions 1 / 3 only, so a consistent line occurs).
fn 021b963c ret=u32 inputs=600
  nowork
  arg u32 oneof 0x2E31400 0x2E31448 0x2E31490 0x2E314D8
  arg u32 const 0x2E30020
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
  fix arg2 0x1010 u32 alt const 1 || const 3 rep=4,4
  fix arg2 0x1080 u32 alt const 1 || const 3 rep=4,4
  fix arg2 0x10F0 u32 alt const 1 || const 3 rep=4,4
  fix arg2 0x1160 u32 alt const 1 || const 3 rep=4,4
  fix arg2 0x11D0 u32 alt const 1 || const 3 rep=4,4
  fix arg2 0x1240 u32 alt const 1 || const 3 rep=4,4
  fix arg2 0x12B0 u32 alt const 1 || const 3 rep=4,4
  fix arg2 0x1408 u32 const 1
  fix arg2 0x1450 u32 const 1
  fix arg2 0x1498 u32 const 1
  fix arg2 0x14E0 u32 const 1
  fix arg2 0x1528 u32 const 1
  fix arg2 0x1570 u32 const 1
  fix arg2 0x15B8 u32 const 1
  fix arg2 0x1400 u32 alt const 0 || const 0xFFFF
  fix arg2 0x1404 u32 alt const 1 || const 0xFFFF
  fix arg2 0x1448 u32 alt const 1 || const 0xFFFF
  fix arg2 0x144C u32 alt const 2 || const 0xFFFF
  fix arg2 0x1490 u32 alt const 2 || const 0xFFFF
  fix arg2 0x1494 u32 alt const 3 || const 0xFFFF
  fix arg2 0x14D8 u32 alt const 3 || const 0xFFFF
  fix arg2 0x14DC u32 alt const 4 || const 0xFFFF
  fix arg2 0x1520 u32 alt const 4 || const 0xFFFF
  fix arg2 0x1524 u32 alt const 5 || const 0xFFFF
  fix arg2 0x1568 u32 alt const 5 || const 0xFFFF
  fix arg2 0x156C u32 alt const 6 || const 0xFFFF
  fix arg2 0x15B0 u32 alt const 6 || const 0xFFFF
  fix arg2 0x15B4 u32 alt const 0 || const 0xFFFF
  stub 021b87d8 force ret=pick:0x2E31000/0x2E31070/0x2E310E0/0x2E31150/0x2E311C0/0x2E31230/0x2E312A0 args=v,v  # rail: point (rd, i)
  stub 021b960c ret=pick:0/1/2/3/4 args=v,v,v  # rail: direction of a line at a point (pt, ln, rd)
  stub 021b93d4 ret=pick:0/1/2/3/4 args=v  # rail: direction back (d)
  stub 021b87fc force ret=pick:0x2E31400/0x2E31448/0x2E31490/0x2E314D8/0x2E31520/0x2E31568/0x2E315B0 args=v,v  # rail: line (rd, i)

# FUN_021B96AC(pt, rd): point line list consistent.
fn 021b96ac ret=u32 inputs=600
  nowork
  arg u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150
  arg u32 const 0x2E30020
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
  stub 021b87fc force ret=pick:0x2E31400/0x2E31448/0x2E31490/0x2E314D8/0x2E31520/0x2E31568/0x2E315B0 args=v,v  # rail: line (rd, i)
  stub 021b87d8 force ret=pick:0x2E31000/0x2E31070/0x2E310E0/0x2E31150/0x2E311C0/0x2E31230/0x2E312A0 args=v,v  # rail: point (rd, i)
  stub 021b93d4 ret=pick:0/1/2/3/4 args=v  # rail: direction back (d)

# FUN_021B9768(): empty.
fn 021b9768 ret=none inputs=5
  nowork
  arg buf 0x3000
  fix arg0 0x2 u16 alt const 0 || const 1 || any
  fix arg0 0x4 u16 alt const 0 || const 1
  fix arg0 0x8 u32 addrof arg0 0x2000
  fix arg0 0xC u32 range 3 4
  fix arg0 0x14 u32 alt const 0 || const 0x2E10100 || oneof 0x2E12000 0x2E1207C
  fix arg0 0x18 u32 alt const 0 || const 1
  fix arg0 0x1C u32 addrof arg0 0x100
  fix arg0 0x20 u16 alt const 7 || range 0 7
  fix arg0 0x22 u16 alt const 7 || range 0 7
  fix arg0 0x24 u16 alt const 7 || range 0 7
  fix arg0 0x26 u16 alt const 7 || range 0 7
  fix arg0 0x28 u16 alt const 7 || range 0 7
  fix arg0 0x2A u16 alt const 7 || range 0 7
  fix arg0 0x2C u16 alt const 7 || range 0 7
  fix arg0 0x30 u32 addrof arg0 0x1000
  fix arg0 0x34 u32 addrof arg0 0x1400
  fix arg0 0x38 u32 addrof arg0 0x1600
  fix arg0 0x3C u32 addrof arg0 0x1700
  fix arg0 0x40 u32 addrof arg0 0x1800
  fix arg0 0x44 u32 addrof arg0 0x1840
  fix arg0 0x48 u32 addrof arg0 0x1880
  fix arg0 0x4C u32 alt range 0 16 || any
  fix arg0 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg0 0x100 u16 alt const 0 || const 1 || const 2
  fix arg0 0x102 u16 alt const 0 || const 1
  fix arg0 0x104 u32 oneof 0x2E11000 0x2E11070 0x2E110E0 0x2E11150 0x2E111C0 0x2E11230 0x2E112A0 0x2E11400 0x2E11448 0x2E11490 0x2E114D8 0x2E11520 0x2E11568 0x2E115B0
  fix arg0 0x108 u16 alt const 0 || const 1
  fix arg0 0x10C u32 alt range 0 0x80 || any
  fix arg0 0x110 u32 alt range 0 0x80 || any
  fix arg0 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg0 0x118 u32 alt range 0 0x40 || any
  fix arg0 0x124 u32 addrof arg0 0x20
  fix arg0 0x128 u32 addrof arg0 0x50
  fix arg0 0x12C u8 alt const 0 || const 1
  fix arg0 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg0 0x12E u16 range 0 5
  fix arg0 0x130 u16 range 0 4
  fix arg0 0x134 u8 alt const 0 || const 1
  fix arg0 0x174 u8 alt const 0 || const 1
  fix arg0 0x178 u32 addrof arg0 0x0
  fix arg0 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg0 0x2002 u16 alt const 0 || const 1
  fix arg0 0x2004 u32 oneof 0x2E11000 0x2E11070 0x2E110E0 0x2E11150 0x2E111C0 0x2E11230 0x2E112A0 0x2E11400 0x2E11448 0x2E11490 0x2E114D8 0x2E11520 0x2E11568 0x2E115B0
  fix arg0 0x2008 u16 alt const 0 || const 1
  fix arg0 0x200C u32 alt range 0 0x80 || any
  fix arg0 0x2010 u32 alt range 0 0x80 || any
  fix arg0 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg0 0x2018 u32 alt range 0 0x40 || any
  fix arg0 0x2024 u32 addrof arg0 0x20
  fix arg0 0x2028 u32 addrof arg0 0x50
  fix arg0 0x202C u8 alt const 0 || const 1
  fix arg0 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg0 0x202E u16 range 0 5
  fix arg0 0x2030 u16 range 0 4
  fix arg0 0x2034 u8 alt const 0 || const 1
  fix arg0 0x2074 u8 alt const 0 || const 1
  fix arg0 0x2078 u32 addrof arg0 0x0
  fix arg0 0x207C u16 alt const 0 || const 1 || const 2
  fix arg0 0x207E u16 alt const 0 || const 1
  fix arg0 0x2080 u32 oneof 0x2E11000 0x2E11070 0x2E110E0 0x2E11150 0x2E111C0 0x2E11230 0x2E112A0 0x2E11400 0x2E11448 0x2E11490 0x2E114D8 0x2E11520 0x2E11568 0x2E115B0
  fix arg0 0x2084 u16 alt const 0 || const 1
  fix arg0 0x2088 u32 alt range 0 0x80 || any
  fix arg0 0x208C u32 alt range 0 0x80 || any
  fix arg0 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg0 0x2094 u32 alt range 0 0x40 || any
  fix arg0 0x20A0 u32 addrof arg0 0x20
  fix arg0 0x20A4 u32 addrof arg0 0x50
  fix arg0 0x20A8 u8 alt const 0 || const 1
  fix arg0 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg0 0x20AA u16 range 0 5
  fix arg0 0x20AC u16 range 0 4
  fix arg0 0x20B0 u8 alt const 0 || const 1
  fix arg0 0x20F0 u8 alt const 0 || const 1
  fix arg0 0x20F4 u32 addrof arg0 0x0
  fix arg0 0x20FA u16 const 0
  fix arg0 0x25C u16 range 0 3
  fix arg0 0x25E u8 alt const 0 || const 1
  fix arg0 0x274 u8 alt const 0 || const 1
  fix arg0 0x276 u8 range 0 4
  fix arg0 0x278 u32 addrof arg0 0x0
  fix arg0 0x1000 u32 alt const 0 || const 6
  fix arg0 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg0 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg0 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg0 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg0 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg0 0x1070 u32 alt const 1 || const 0
  fix arg0 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg0 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg0 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg0 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg0 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg0 0x10E0 u32 alt const 2 || const 1
  fix arg0 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg0 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg0 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg0 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg0 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg0 0x1150 u32 alt const 3 || const 2
  fix arg0 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg0 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg0 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg0 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg0 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg0 0x11C0 u32 alt const 4 || const 3
  fix arg0 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg0 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg0 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg0 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg0 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg0 0x1230 u32 alt const 5 || const 4
  fix arg0 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg0 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg0 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg0 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg0 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg0 0x12A0 u32 alt const 6 || const 5
  fix arg0 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg0 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg0 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg0 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg0 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg0 0x1400 u32 const 0
  fix arg0 0x1404 u32 const 1
  fix arg0 0x1408 u32 alt range 1 4 || range 0 5
  fix arg0 0x140C u32 alt range 0 6 || const 9
  fix arg0 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg0 0x1414 u32 alt range 0 8 || any
  fix arg0 0x1448 u32 const 1
  fix arg0 0x144C u32 const 2
  fix arg0 0x1450 u32 alt range 1 4 || range 0 5
  fix arg0 0x1454 u32 alt range 0 6 || const 9
  fix arg0 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg0 0x145C u32 alt range 0 8 || any
  fix arg0 0x1490 u32 const 2
  fix arg0 0x1494 u32 const 3
  fix arg0 0x1498 u32 alt range 1 4 || range 0 5
  fix arg0 0x149C u32 alt range 0 6 || const 9
  fix arg0 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg0 0x14A4 u32 alt range 0 8 || any
  fix arg0 0x14D8 u32 const 3
  fix arg0 0x14DC u32 const 4
  fix arg0 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg0 0x14E4 u32 alt range 0 6 || const 9
  fix arg0 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg0 0x14EC u32 alt range 0 8 || any
  fix arg0 0x1520 u32 const 4
  fix arg0 0x1524 u32 const 5
  fix arg0 0x1528 u32 alt range 1 4 || range 0 5
  fix arg0 0x152C u32 alt range 0 6 || const 9
  fix arg0 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg0 0x1534 u32 alt range 0 8 || any
  fix arg0 0x1568 u32 const 5
  fix arg0 0x156C u32 const 6
  fix arg0 0x1570 u32 alt range 1 4 || range 0 5
  fix arg0 0x1574 u32 alt range 0 6 || const 9
  fix arg0 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg0 0x157C u32 alt range 0 8 || any
  fix arg0 0x15B0 u32 const 6
  fix arg0 0x15B4 u32 const 0
  fix arg0 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg0 0x15BC u32 alt range 0 6 || const 9
  fix arg0 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg0 0x15C4 u32 alt range 0 8 || any
  fix arg0 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg0 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg0 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg0 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg0 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg0 0x340 u16 alt range 0 3 || range 0 6
  fix arg0 0x342 u8 alt const 0 || const 1
  fix arg0 0x343 u8 range 0 4
  fix arg0 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg0 0x346 u16 alt range 0 8 || any
  fix arg0 0x25C u16 alt range 0 3 || range 0 6
  fix arg0 0x25E u8 alt const 0 || const 1
  fix arg0 0x25F u8 range 0 4
  fix arg0 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg0 0x262 u16 alt range 0 8 || any
  fix arg0 0x15C u16 alt range 0 3 || range 0 6
  fix arg0 0x15E u8 alt const 0 || const 1
  fix arg0 0x15F u8 range 0 4
  fix arg0 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg0 0x162 u16 alt range 0 8 || any

# FUN_021B976C(a, p, b): debug hook of a point.
fn 021b976c ret=none inputs=10
  nowork
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 any
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
  stub 021b9768 ret=none args=  # rail: empty debug hook

# FUN_021B977C(w): debug hook of the RAIL WORK.
fn 021b977c ret=none inputs=60
  nowork
  arg u32 oneof 0x2E20100 0x2E22000 0x2E2207C
  arg buf 0x3000
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
  stub 021b976c ret=none args=v,v,v  # rail: debug hook of a point (a, p, b)
  stub 021b87d8 force ret=pick:0x2E21000/0x2E21070/0x2E210E0/0x2E21150/0x2E211C0/0x2E21230/0x2E212A0 args=v,v  # rail: point (rd, i)
  stub 021b9768 ret=none args=  # rail: empty debug hook

# FUN_021B97D0(rd, p): point index.
fn 021b97d0 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30020
  arg u32 alt const 0 || const 0x2E310E0 || const 0x2E312A0
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

# FUN_021B97FC(rd, ln): line index.
fn 021b97fc ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30020
  arg u32 oneof 0x2E31400 0x2E31448 0x2E31490 0x2E314D8
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

# FUN_021B9828(d, n, p, a, b, out): segment / plane hit.
fn 021b9828 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E70380
  arg u32 const 0x2E703A0
  arg u32 const 0x2E70380
  arg u32 const 0x2E703A0
  arg u32 const 0x2E70380
  arg u32 const 0x2E703C0
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
  fix arg6 0x380 u32 alt range 0 0x10000 || range 0xFFFF0000 0xFFFFFFFF || any rep=3,4
  fix arg6 0x3A0 u32 alt range 0 0x10000 || range 0xFFFF0000 0xFFFFFFFF || any rep=3,4
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cbd0 force ret=pick+u32:0x1000/0x2000/0x3000 args=in:12  # VEC_Mag (a)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 0207cae8 force ret=none args=in:12,in:12,out:12  # VEC_CrossProduct (a, b, &out)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)

# FUN_021B9A50(out, a, b, t): slerp (FX_AcosIdx answers small angles and the first table entries are often 0, so the
# parallel case occurs).
fn 021b9a50 ret=none inputs=300
  nowork
  arg u32 const 0x2E503C0
  arg u32 const 0x2E50380
  arg u32 const 0x2E503A0
  arg u32 range 0 0x1000
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
  fix global0 0 u32 alt const 0 || any rep=4,4
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 0207ca68 force ret=u32 args=in:12,in:12  # VEC_DotProduct (a, b)
  stub 0207d184 force ret=pick+u16:0/0x10/0x20/0x30 args=v  # FX_AcosIdx (x)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 020520a4 force ret=none args=in:12,v,out:12  # VEC_MultScalar-like (a, fx, &out)
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12  # VEC_Add (a, b, &out)

# FUN_021B9B24(out, a, b, t): interpolation.
fn 021b9b24 ret=none inputs=200
  nowork
  arg u32 const 0x2E503C0
  arg u32 const 0x2E50380
  arg u32 const 0x2E503A0
  arg u32 range 0 0x1000
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
  stub 021b9a50 ret=none args=out:12,in:12,in:12,v  # rail: slerp (&out, a, b, t)
  stub 0207cbd0 force ret=pick+u32:0x1000/0x2000/0x3000 args=in:12  # VEC_Mag (a)
  stub 0207cee4 force ret=none args=v,in:12,in:12,out:12  # VEC_MultAdd (fx, a, b, &out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 0207ca68 force ret=u32 args=in:12,in:12  # VEC_DotProduct (a, b)
  stub 0207d184 force ret=u16 args=v  # FX_AcosIdx (x)
  stub 0207c768 force ret=pick+u32:0/0x800/0x1000/0x1001/0x2000 args=v,v  # FX_Div (a, b)
  stub 020520a4 force ret=none args=in:12,v,out:12  # VEC_MultScalar-like (a, fx, &out)
  stub 0207c9d0 force ret=none args=in:12,in:12,out:12  # VEC_Add (a, b, &out)
