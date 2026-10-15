# fndiff spec (F3b): `field_rail.c` part 1 (overlay 21 Thumb code 0x021B7884..0x021B88DC; F1 row 21.53): the RAIL system, the main
# RAIL WORK, LOCATIONs, the per-frame line movement driver and the rail-data lookups. Run:
# tools/check-fn.sh black-field rail --per-function
#
# File bound: `Heap_AllocDebug(.., "field_rail.c", 0x12E / 0x137 / 0x13F)` in 021B7884 (string 0x021DE2A0); rail_attr.c starts at
# 021BB088. The file is split in three areas by address: `rail` (here), `rail_move` (021B88DC..021B9B7C: the line movement 021B88DC,
# OBJ / blocked-line / direction helpers, the segment / vector math) and `rail_cam` (021B9B7C..021BB088: the position, line-hit and
# camera procs of the ROM tables 0x021DDDA0 / 0x021DDDAC / 0x021DDDB8). Function list: bl targets of the field set, odd code
# pointers, the Ghidra index and a Thumb walk from every entry (fragments: none in this part).
#
# Records (full description in crates/black-field/src/rail.rs): RAIL system R (0x74; +0x14 current RAIL WORK, +0x1C main RAIL WORK,
# +0x20 RAIL DATA RD, +0x50 blocked lines), RAIL DATA (counts, -> points 0x70 / lines 0x48 / camera and line records 0x24 / three
# proc tables, +0x2C unit), RAIL WORK W (0x7C), LOCATION (8: u16 index, u8 kind, u8 dir, s16 x, u16 z).
#
# The universe (every block): the last argument is the arena `buf 0x3000`: R at +0, the main W at +0x100, the 7E60 object at +0x200,
# input / output LOCATIONs and vectors at +0x340.. (fixed), seven points at +0x1000, seven lines at +0x1400 (a consistent loop
# P0-L0-P1-..-L6-P0: line k runs from point k to k+1, a point lists a line that touches it first; for 021B7EE4 point 0 also lists a
# random one), camera records +0x1600, line records +0x1700, the three proc
# tables +0x1800 / +0x1840 / +0x1880 (entries are the FAKE callees 0x02EE0011 / 21 / 31 or 0, stubbed `force` at 02EE0010 / 20 / 30),
# OBJs at +0x2000. The object arguments are the fixed addresses of these records. Indices past the counts read the first record /
# the ROM default records 0x021D9A0C / 0x021D9A30 (real ROM data on both sides). The move speed `W + 0x30` is drawn 0..4 (it indexes
# the 5-byte step table ROM 0x021DE298).
#
# Conventions (headers of the F5b / F6b / F14 specs): all blocks `nowork`; closure stubs (a block lists the stubs of its callees and of
# the callees of all its in-area descendants; checked with every function treated as ported: no fault, stray write or unstubbed
# callee); every out-of-area stub is `force` (the other two rail areas included); stack locals reach stubs as `in:` / `out:`; memcmp
# 0203EFC8 runs as real code (ALLOW; the port inlines it). 021B7EE4 reads uninitialised stack words for a point whose first listed
# line touches it at neither end; called first in every block that reaches it, they read 0 (Z19 entry state).
area rail
set field

# FUN_021B7884(heap, n, x): create the RAIL system (R 0x74, n OBJs of 0x7C, the main RAIL WORK). Allocations: seqptr
# into the arena from +0x400 (step 0x200; n is 0..4).
fn 021b7884 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 range 0 4
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
  stub 02030734 force ret=seqptr:arg3:0x400:0x200 args=v,v,v,v,v  # Heap_AllocDebug: R, the OBJs, the main RAIL WORK
  stub 021b9164 force ret=none args=v,v,v  # rail: OBJ init (w, rd, R)

# FUN_021B791C(R): free the main RAIL WORK, the OBJs, R.
fn 021b791c ret=none inputs=20
  nowork
  arg u32 const 0x2E20000
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
  stub 020307b0 force ret=none args=v  # Heap_Free (block)

# FUN_021B7934(R, data): take a RAIL DATA (copy 0x30 bytes from data).
fn 021b7934 ret=none inputs=30
  nowork
  arg u32 const 0x2E30000
  arg u32 const 0x2E31000
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
  stub 021b9464 force ret=none args=v  # rail: clear the blocked-line list (blk)
  stub 021b87c8 ret=none args=v,v  # rail: copy a RAIL DATA (dst, src)

# FUN_021B7954(R): drop the RAIL DATA.
fn 021b7954 ret=none inputs=20
  nowork
  arg u32 const 0x2E20000
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
  stub 021b87bc ret=none args=v  # rail: zero a RAIL DATA (rd)
  stub 021b9464 force ret=none args=v  # rail: clear the blocked-line list (blk)

# FUN_021B7970(R): take the first free OBJ (n >= 3; the stub answers 0 from its third call on).
fn 021b7970 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E20000
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
  stub 021b91a8 force ret=nth:pick:0/1;pick:0/1;pick:0 args=v  # rail: OBJ in use (o)

# FUN_021B79AC(R, obj): reset an OBJ and free it.
fn 021b79ac ret=none inputs=20
  nowork
  arg u32 const 0x2E30000
  arg u32 const 0x2E32000
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
  stub 021b9164 force ret=none args=v,v,v  # rail: OBJ init (w, rd, R)

# FUN_021B79C4(R, w): current RAIL WORK = w, +0x18 = 1.
fn 021b79c4 ret=none inputs=20
  nowork
  arg u32 const 0x2E30000
  arg u32 any
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

# FUN_021B79CC(R): no current RAIL WORK.
fn 021b79cc ret=none inputs=20
  nowork
  arg u32 const 0x2E20000
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

# FUN_021B79D4(R, out): the current RAIL WORK's position.
fn 021b79d4 ret=none inputs=40
  nowork
  arg u32 const 0x2E30000
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
  stub 021b8028 ret=none args=v,out:12  # rail: position (w, &out)

# FUN_021B79E4(R): R when no RAIL DATA, else u16 +4.
fn 021b79e4 ret=u32 inputs=40
  nowork
  arg u32 const 0x2E20000
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

# FUN_021B79F0(R, force): run the camera proc of the current RAIL WORK.
fn 021b79f0 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E30000
  arg u32 alt const 0 || any
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
  stub 021b91ac force ret=pick:0/1 args=v  # rail: RAIL WORK active (w)
  stub 021b8048 ret=pick:0/1 args=v  # rail: moved (w)
  stub 021b8820 ret=pick:0x021D9A0C/0x2E31600/0x2E31624/0x2E31648/0x2E3166C/0x2E31690/0x2E316B4/0x2E316D8 args=v,v  # rail: camera record (rd, i)
  stub 021b8870 ret=pick:0/0x02EE0011 args=v,v  # rail: camera proc (rd, i)
  stub 02ee0010 force ret=none args=v  # camera proc (R)

# FUN_021B7A84(R, a, b, loc, pos): the nearest line hit of the segment a..b.
fn 021b7a84 ret=u32 inputs=400
  nowork
  arg u32 const 0x2E60000
  arg u32 const 0x2E60380
  arg u32 const 0x2E603A0
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
  stub 021b9510 force ret=pick:0/0x2E61400/0x2E61448/0x2E61490/0x2E614D8/0x2E61520/0x2E61568/0x2E615B0 args=v,v,v  # rail: line i unless blocked (blk, rd, i)
  stub 021b87d8 ret=pick:0x2E61000/0x2E61070/0x2E610E0/0x2E61150/0x2E611C0/0x2E61230/0x2E612A0 args=v,v  # rail: point (rd, i)
  stub 021b8848 ret=pick:0x021D9A30/0x2E61700/0x2E61724/0x2E61748/0x2E6176C/0x2E61790/0x2E617B4/0x2E617D8 args=v,v  # rail: line record (rd, i)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cae8 force ret=none args=in:12,in:12,out:12  # VEC_CrossProduct (a, b, &out)
  stub 021b9828 force ret=pick:0/1 args=in:12,in:12,in:12,in:12,in:12,out:12  # rail: segment hits the line plane (d, n, p, a, b, &hit)
  stub 021b88b8 ret=pick:0/0x02EE0031 args=v,v  # rail: line-hit proc (rd, i)
  stub 0207cf3c force ret=pick+u32:0x1000/0x2000/0x3000 args=in:12,in:12  # VEC_Distance (a, b)
  stub 02ee0030 force ret=pick:0/1 args=v,v,in:12,out:8,out:12  # line-hit proc (i, R, &hit, &loc, &pos)

# FUN_021B7C04(R, id, f): block (f == 0) / unblock a line.
fn 021b7c04 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E40000
  arg u32 alt range 0 5 || any
  arg u32 alt const 0 || any
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
  stub 021b94c0 force ret=pick:0/1 args=v,v  # rail: line blocked (blk, id)
  stub 021b9478 force ret=u32 args=v,v  # rail: block a line (blk, id)
  stub 021b949c force ret=u32 args=v,v  # rail: unblock a line (blk, id)

# FUN_021B7C38(R): the unit length as fx32.
fn 021b7c38 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E20000
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

# FUN_021B7C58(R, data, line, z, x, out): the LOCATION of a line position in another RAIL DATA (data = this one).
fn 021b7c58 ret=none inputs=200
  nowork
  arg u32 const 0x2E70000
  arg u32 const 0x2E70020
  arg u32 range 0 3
  arg u32 alt range 0 8 || any
  arg u32 alt range 0 8 || range 0xFFF8 0xFFFF || any
  arg u32 const 0x2E70360
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
  stub 021b7ee4 ret=none args=v,in:8  # rail: place the RAIL WORK at a LOCATION (w, &loc)
  stub 021b919c force ret=none args=v,v  # rail: OBJ reset (w, rd)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b8658 ret=none args=v,v,v,v,v,v,v  # rail: set the line state (w, ln, dir, pos, lat, len, wid)
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)

# FUN_021B7C90(R, loc, dir, out): one step from loc; position to out; 0 when the LOCATION did not change.
fn 021b7c90 ret=u32 inputs=400
  nowork
  arg u32 const 0x2E50000
  arg u32 const 0x2E50340
  arg u32 range 0 5
  arg u32 const 0x2E503C0
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
  fix arg4 0x342 u8 const 1
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
  stub 021b7ee4 ret=none args=v,in:8  # rail: place the RAIL WORK at a LOCATION (w, &loc)
  stub 021b80e0 ret=pick:0/1 args=v,v,v  # rail: start a move (w, speed, dir)
  stub 021b810c ret=none args=v  # rail: one frame of a move (w)
  stub 021b8028 ret=none args=v,out:12  # rail: position (w, &out)
  stub 021b7de0 ret=pick:0/1 args=v,in:8  # rail: LOCATION valid (w, &loc)
  stub 021b919c force ret=none args=v,v  # rail: OBJ reset (w, rd)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b8658 ret=none args=v,v,v,v,v,v,v  # rail: set the line state (w, ln, dir, pos, lat, len, wid)
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)
  stub 021b91ac force ret=pick:0/1 args=v  # rail: RAIL WORK active (w)
  stub 021b88dc force ret=pick:0/1/2/3/4 args=outk:2:1/0,v,v  # rail: line movement of one step (w, dir, step) -> new dir (may change the kind, bit 0 of w + 0)
  stub 021b977c force ret=none args=v  # rail: kind changed (w)
  stub 021b9238 force ret=none args=v,outm:8:3/0/1/0xFF/0xFF/0xFF/0xF/0  # rail: LOCATION of the RAIL WORK (w, &loc)

# FUN_021B7CD8(R, loc, dir, out): one step from loc; LOCATION to out.
fn 021b7cd8 ret=u32 inputs=400
  nowork
  arg u32 const 0x2E50000
  arg u32 const 0x2E50340
  arg u32 range 0 5
  arg u32 const 0x2E50360
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
  fix arg4 0x342 u8 const 1
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
  stub 021b7ee4 ret=none args=v,in:8  # rail: place the RAIL WORK at a LOCATION (w, &loc)
  stub 021b80e0 ret=pick:0/1 args=v,v,v  # rail: start a move (w, speed, dir)
  stub 021b810c ret=none args=v  # rail: one frame of a move (w)
  stub 021b7dc0 ret=none args=v,out:8  # rail: LOCATION (w, &out)
  stub 021b7de0 ret=pick:0/1 args=v,in:8  # rail: LOCATION valid (w, &loc)
  stub 021b919c force ret=none args=v,v  # rail: OBJ reset (w, rd)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b8658 ret=none args=v,v,v,v,v,v,v  # rail: set the line state (w, ln, dir, pos, lat, len, wid)
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)
  stub 021b91ac force ret=pick:0/1 args=v  # rail: RAIL WORK active (w)
  stub 021b88dc force ret=pick:0/1/2/3/4 args=outk:2:1/0,v,v  # rail: line movement of one step (w, dir, step) -> new dir (may change the kind, bit 0 of w + 0)
  stub 021b977c force ret=none args=v  # rail: kind changed (w)
  stub 021b9238 force ret=none args=v,outm:8:3/0/1/0xFF/0xFF/0xFF/0xF/0  # rail: LOCATION of the RAIL WORK (w, &loc)

# FUN_021B7D38(R, loc, out): the position of loc.
fn 021b7d38 ret=none inputs=200
  nowork
  arg u32 const 0x2E40000
  arg u32 const 0x2E40340
  arg u32 const 0x2E403C0
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
  stub 021b7ee4 ret=none args=v,in:8  # rail: place the RAIL WORK at a LOCATION (w, &loc)
  stub 021b8028 ret=none args=v,out:12  # rail: position (w, &out)
  stub 021b919c force ret=none args=v,v  # rail: OBJ reset (w, rd)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b8658 ret=none args=v,v,v,v,v,v,v  # rail: set the line state (w, ln, dir, pos, lat, len, wid)
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)

# FUN_021B7D50(R, loc): the half width at loc / 16.
fn 021b7d50 ret=u32 inputs=200
  nowork
  arg u32 const 0x2E30000
  arg u32 const 0x2E30340
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
  stub 021b7ee4 ret=none args=v,in:8  # rail: place the RAIL WORK at a LOCATION (w, &loc)
  stub 021b919c force ret=none args=v,v  # rail: OBJ reset (w, rd)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b8658 ret=none args=v,v,v,v,v,v,v  # rail: set the line state (w, ln, dir, pos, lat, len, wid)
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)

# FUN_021B7D68(R, loc): run the camera proc of loc.
fn 021b7d68 ret=none inputs=300
  nowork
  arg u32 const 0x2E30000
  arg u32 const 0x2E30340
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
  stub 021b7ee4 ret=none args=v,in:8  # rail: place the RAIL WORK at a LOCATION (w, &loc)
  stub 021b8820 ret=pick:0x021D9A0C/0x2E31600/0x2E31624/0x2E31648/0x2E3166C/0x2E31690/0x2E316B4/0x2E316D8 args=v,v  # rail: camera record (rd, i)
  stub 021b8870 ret=pick:0/0x02EE0011 args=v,v  # rail: camera proc (rd, i)
  stub 02ee0010 force ret=none args=v  # camera proc (R)
  stub 021b919c force ret=none args=v,v  # rail: OBJ reset (w, rd)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b8658 ret=none args=v,v,v,v,v,v,v  # rail: set the line state (w, ln, dir, pos, lat, len, wid)
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)

# FUN_021B7DC0(w, out): the LOCATION.
fn 021b7dc0 ret=none inputs=20
  nowork
  arg u32 const 0x2E30100
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

# FUN_021B7DE0(w, loc): LOCATION valid.
fn 021b7de0 ret=u32 inputs=400
  nowork
  arg u32 const 0x2E30100
  arg u32 const 0x2E30340
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
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)

# FUN_021B7E60(p, dir, loc, pos): cached step of p + 0x5C.
fn 021b7e60 ret=u32 inputs=300
  nowork
  arg u32 const 0x2E50200
  arg u32 range 0 5
  arg u32 alt const 0 || const 0x2E50360
  arg u32 alt const 0 || const 0x2E503C0
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
  stub 021b7cd8 ret=pick:0/1 args=v,in:8,v,out:8  # rail: one step (R, &loc, dir, &out)
  stub 021b7d38 ret=none args=v,in:8,out:12  # rail: position of a LOCATION (R, &loc, &out)
  stub 021b7ee4 ret=none args=v,in:8  # rail: place the RAIL WORK at a LOCATION (w, &loc)
  stub 021b80e0 ret=pick:0/1 args=v,v,v  # rail: start a move (w, speed, dir)
  stub 021b810c ret=none args=v  # rail: one frame of a move (w)
  stub 021b7dc0 ret=none args=v,out:8  # rail: LOCATION (w, &out)
  stub 021b7de0 ret=pick:0/1 args=v,in:8  # rail: LOCATION valid (w, &loc)
  stub 021b8028 ret=none args=v,out:12  # rail: position (w, &out)
  stub 021b919c force ret=none args=v,v  # rail: OBJ reset (w, rd)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b8658 ret=none args=v,v,v,v,v,v,v  # rail: set the line state (w, ln, dir, pos, lat, len, wid)
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)
  stub 021b91ac force ret=pick:0/1 args=v  # rail: RAIL WORK active (w)
  stub 021b88dc force ret=pick:0/1/2/3/4 args=outk:2:1/0,v,v  # rail: line movement of one step (w, dir, step) -> new dir (may change the kind, bit 0 of w + 0)
  stub 021b977c force ret=none args=v  # rail: kind changed (w)
  stub 021b9238 force ret=none args=v,outm:8:3/0/1/0xFF/0xFF/0xFF/0xF/0  # rail: LOCATION of the RAIL WORK (w, &loc)

# FUN_021B7ED4(w, out): the previous position.
fn 021b7ed4 ret=none inputs=20
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

# FUN_021B7EE4(w, loc): place the RAIL WORK at loc (point 0: its first line also drawn at random).
fn 021b7ee4 ret=none inputs=500
  nowork
  arg u32 oneof 0x2E30100 0x2E32000 0x2E3207C
  arg u32 const 0x2E30340
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
  fix arg2 0x1000 u32 alt range 0 6 || const 0
  stub 021b919c force ret=none args=v,v  # rail: OBJ reset (w, rd)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b8658 ret=none args=v,v,v,v,v,v,v  # rail: set the line state (w, ln, dir, pos, lat, len, wid)
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)

# FUN_021B8028(w, out): the position.
fn 021b8028 ret=none inputs=20
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

# FUN_021B8038(w): byte +0x32.
fn 021b8038 ret=u32 inputs=20
  nowork
  arg u32 const 0x2E20100
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

# FUN_021B8040(w): byte +0x2C.
fn 021b8040 ret=u32 inputs=20
  nowork
  arg u32 const 0x2E20100
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

# FUN_021B8048(w): byte +0x34.
fn 021b8048 ret=u32 inputs=20
  nowork
  arg u32 const 0x2E20100
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

# FUN_021B8050(w, out): the unit direction of travel (3 x s16).
fn 021b8050 ret=none inputs=200
  nowork
  arg u32 const 0x2E30100
  arg u32 const 0x2E303E0
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
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)
  stub 0207ca00 force ret=none args=in:12,in:12,out:12  # VEC_Subtract (a, b, &out)
  stub 0207cca4 force ret=none args=in:12,out:12  # VEC_Normalize (a, &out)
  stub 021b9238 force ret=none args=v,outm:8:3/0/1/0xFF/0xFF/0xFF/0xF/0  # rail: LOCATION of the RAIL WORK (w, &loc)
  stub 021b9368 force ret=u32 args=v,in:8,v,v,out:12  # rail: resolve a position past the line end (R, &loc, x, n, &pos)

# FUN_021B80D8(w): word +8 of its POINT / LINE.
fn 021b80d8 ret=u32 inputs=20
  nowork
  arg u32 const 0x2E20100
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

# FUN_021B80E0(w, speed, dir): start a move.
fn 021b80e0 ret=u32 inputs=40
  nowork
  arg u32 const 0x2E40100
  arg u32 any
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

# FUN_021B810C(w): one frame of a move.
fn 021b810c ret=none inputs=2000
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
  fix arg1 2 u16 alt const 1 || any
  fix arg1 0x108 u16 alt const 1 || const 0
  fix arg1 0x2008 u16 alt const 1 || const 0
  fix arg1 0x2084 u16 alt const 1 || const 0
  fix arg1 0x12C u8 alt const 1 || const 0
  fix arg1 0x202C u8 alt const 1 || const 0
  fix arg1 0x20A8 u8 alt const 1 || const 0
  stub 021b91ac force ret=pick:0/1 args=v  # rail: RAIL WORK active (w)
  stub 021b88dc force ret=pick:0/1/2/3/4 args=outk:2:1/0,v,v  # rail: line movement of one step (w, dir, step) -> new dir (may change the kind, bit 0 of w + 0)
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)
  stub 021b977c force ret=none args=v  # rail: kind changed (w)
  stub 021b9238 force ret=none args=v,outm:8:3/0/1/0xFF/0xFF/0xFF/0xF/0  # rail: LOCATION of the RAIL WORK (w, &loc)

# FUN_021B821C(x): the word +0x10.
fn 021b821c ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E20000 0x2E20100
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

# FUN_021B8220(x): the word +0x14.
fn 021b8220 ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E20000 0x2E20100
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

# FUN_021B8224(R, i): thunk: point i of R + 0x20.
fn 021b8224 ret=u32 inputs=40
  nowork
  arg u32 const 0x2E30000
  arg u32 alt range 0 6 || const 0xFFFF || any
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

# FUN_021B8230(R, i): thunk: line i of R + 0x20.
fn 021b8230 ret=u32 inputs=40
  nowork
  arg u32 const 0x2E30000
  arg u32 alt range 0 6 || const 0xFFFF || any
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

# FUN_021B823C(R, i): thunk: line record i of R + 0x20.
fn 021b823c ret=u32 inputs=40
  nowork
  arg u32 const 0x2E30000
  arg u32 alt range 0 6 || const 0xFFFF || any
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

# FUN_021B8248(x): the word +0x4C.
fn 021b8248 ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E20000 0x2E20100
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

# FUN_021B824C(x): the word +0x1C.
fn 021b824c ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E20000 0x2E20100
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

# FUN_021B8250(R, loc, out): loc clamped to its line.
fn 021b8250 ret=none inputs=300
  nowork
  arg u32 const 0x2E40000
  arg u32 const 0x2E40340
  arg u32 const 0x2E40360
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
  fix arg3 0x342 u8 const 1
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
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)

# FUN_021B82F0(w): u16 +0.
fn 021b82f0 ret=u32 inputs=20
  nowork
  arg u32 const 0x2E20100
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

# FUN_021B82F4(x): the word +4.
fn 021b82f4 ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E20000 0x2E20100
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

# FUN_021B82F8(x): the word +0xC.
fn 021b82f8 ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E20000 0x2E20100
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

# FUN_021B82FC(x): the word +0x10.
fn 021b82fc ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E20000 0x2E20100
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

# FUN_021B8300(x): the word +0x14.
fn 021b8300 ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E20000 0x2E20100
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

# FUN_021B8304(x): the word +0x18.
fn 021b8304 ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E20000 0x2E20100
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

# FUN_021B8308(x): the word +0x1C.
fn 021b8308 ret=u32 inputs=20
  nowork
  arg u32 oneof 0x2E20000 0x2E20100
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

# FUN_021B830C(w): thunk to 021B91C8(w).
fn 021b830c ret=u32 inputs=20
  nowork
  arg u32 const 0x2E20100
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
  stub 021b91c8 force ret=u32 args=v  # rail: camera record of the RAIL WORK (w)

# FUN_021B8314(w, out): the position without the lateral offset.
fn 021b8314 ret=none inputs=20
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
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)

# FUN_021B8328(w, along, out): the position at along.
fn 021b8328 ret=none inputs=20
  nowork
  arg u32 const 0x2E40100
  arg u32 any
  arg u32 const 0x2E403C0
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
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)

# FUN_021B833C(w): the start point of its line.
fn 021b833c ret=u32 inputs=60
  nowork
  arg u32 const 0x2E20100
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

# FUN_021B834C(w): the end point of its line.
fn 021b834c ret=u32 inputs=60
  nowork
  arg u32 const 0x2E20100
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

# FUN_021B835C(w): the line record of its line.
fn 021b835c ret=u32 inputs=60
  nowork
  arg u32 const 0x2E20100
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

# FUN_021B836C(w, pt): the camera record of a point.
fn 021b836c ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30100
  arg u32 oneof 0x2E31000 0x2E31070 0x2E310E0 0x2E31150
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

# FUN_021B8378(x): x / 16 rounded to nearest.
fn 021b8378 ret=u32 inputs=200
  nowork
  arg u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
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

# FUN_021B83B0(a, b, r): a and b near on the ground.
fn 021b83b0 ret=u32 inputs=600
  nowork
  arg u32 const 0x2E40380
  arg u32 const 0x2E403A0
  arg u32 alt range 0 0x10000 || any
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
  fix arg3 0x380 u32 alt range 0 0x8000 || range 0xFFFF8000 0xFFFFFFFF || any rep=3,4
  fix arg3 0x3A0 u32 alt range 0 0x8000 || range 0xFFFF8000 0xFFFFFFFF || any rep=3,4
  fix arg3 0x384 u32 alt const 0 || const 0x8000 || const 0x7FFF || const 0xFFFF8000
  fix arg3 0x3A4 u32 alt const 0 || const 0x8000 || const 0x7FFF || const 0xFFFF8000

# FUN_021B83FC(w, idx, z, x): the LOCATION numbers of w.
fn 021b83fc ret=none inputs=200
  nowork
  arg u32 const 0x2E50100
  arg u32 const 0x2E503F0
  arg u32 const 0x2E503F4
  arg u32 const 0x2E503F8
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
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)

# FUN_021B8484(w, idx, z, x): place w on line idx at z / x.
fn 021b8484 ret=none inputs=300
  nowork
  arg u32 const 0x2E50100
  arg u32 range 0 3
  arg u32 alt range 0 8 || any
  arg u32 alt range 0 8 || range 0xFFF8 0xFFFF || any
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
  stub 021b7dc0 ret=none args=v,out:8  # rail: LOCATION (w, &out)
  stub 021b7ee4 ret=none args=v,in:8  # rail: place the RAIL WORK at a LOCATION (w, &loc)
  stub 021b91f8 force ret=none args=v,out:12  # rail: position of the RAIL WORK (w, &pos)
  stub 021b919c force ret=none args=v,v  # rail: OBJ reset (w, rd)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b8658 ret=none args=v,v,v,v,v,v,v  # rail: set the line state (w, ln, dir, pos, lat, len, wid)

# FUN_021B8514(pt, dir, rd, blk, open): the line of pt leaving in dir.
fn 021b8514 ret=u32 inputs=200
  nowork
  arg u32 oneof 0x2E61000 0x2E61070 0x2E610E0 0x2E61150
  arg u32 alt range 0 5 || any
  arg u32 const 0x2E60020
  arg u32 const 0x2E60050
  arg u32 alt const 0 || const 1
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
  stub 021b9510 force ret=pick:0/0x2E61400/0x2E61448/0x2E61490/0x2E614D8/0x2E61520/0x2E61568/0x2E615B0 args=v,v,v  # rail: line i unless blocked (blk, rd, i)
  stub 021b87fc ret=pick:0x2E61400/0x2E61448/0x2E61490/0x2E614D8/0x2E61520/0x2E61568/0x2E615B0 args=v,v  # rail: line (rd, i)

# FUN_021B854C(w, ln, dir, pos, lat, len, wid): move w onto ln.
fn 021b854c ret=u32 inputs=800
  nowork
  arg u32 const 0x2E80100
  arg u32 oneof 0x2E81400 0x2E81448 0x2E81490 0x2E814D8
  arg u32 any
  arg u32 alt range 0xFFFFFF80 0xFFFFFFFF || range 0 0x80 || any
  arg u32 alt range 0xFFFFFFC0 0xFFFFFFFF || range 0 0x40 || any
  arg u32 alt rel 3 range 0 1 || range 0 0x80 || any
  arg u32 alt range 0 0x40 || any
  arg buf 0x3000
  fix arg7 0x2 u16 alt const 0 || const 1 || any
  fix arg7 0x4 u16 alt const 0 || const 1
  fix arg7 0x8 u32 addrof arg7 0x2000
  fix arg7 0xC u32 range 3 4
  fix arg7 0x14 u32 alt const 0 || const 0x2E80100 || oneof 0x2E82000 0x2E8207C
  fix arg7 0x18 u32 alt const 0 || const 1
  fix arg7 0x1C u32 addrof arg7 0x100
  fix arg7 0x20 u16 alt const 7 || range 0 7
  fix arg7 0x22 u16 alt const 7 || range 0 7
  fix arg7 0x24 u16 alt const 7 || range 0 7
  fix arg7 0x26 u16 alt const 7 || range 0 7
  fix arg7 0x28 u16 alt const 7 || range 0 7
  fix arg7 0x2A u16 alt const 7 || range 0 7
  fix arg7 0x2C u16 alt const 7 || range 0 7
  fix arg7 0x30 u32 addrof arg7 0x1000
  fix arg7 0x34 u32 addrof arg7 0x1400
  fix arg7 0x38 u32 addrof arg7 0x1600
  fix arg7 0x3C u32 addrof arg7 0x1700
  fix arg7 0x40 u32 addrof arg7 0x1800
  fix arg7 0x44 u32 addrof arg7 0x1840
  fix arg7 0x48 u32 addrof arg7 0x1880
  fix arg7 0x4C u32 alt range 0 16 || any
  fix arg7 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg7 0x100 u16 alt const 0 || const 1 || const 2
  fix arg7 0x102 u16 alt const 0 || const 1
  fix arg7 0x104 u32 oneof 0x2E81000 0x2E81070 0x2E810E0 0x2E81150 0x2E811C0 0x2E81230 0x2E812A0 0x2E81400 0x2E81448 0x2E81490 0x2E814D8 0x2E81520 0x2E81568 0x2E815B0
  fix arg7 0x108 u16 alt const 0 || const 1
  fix arg7 0x10C u32 alt range 0 0x80 || any
  fix arg7 0x110 u32 alt range 0 0x80 || any
  fix arg7 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg7 0x118 u32 alt range 0 0x40 || any
  fix arg7 0x124 u32 addrof arg7 0x20
  fix arg7 0x128 u32 addrof arg7 0x50
  fix arg7 0x12C u8 alt const 0 || const 1
  fix arg7 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg7 0x12E u16 range 0 5
  fix arg7 0x130 u16 range 0 4
  fix arg7 0x134 u8 alt const 0 || const 1
  fix arg7 0x174 u8 alt const 0 || const 1
  fix arg7 0x178 u32 addrof arg7 0x0
  fix arg7 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg7 0x2002 u16 alt const 0 || const 1
  fix arg7 0x2004 u32 oneof 0x2E81000 0x2E81070 0x2E810E0 0x2E81150 0x2E811C0 0x2E81230 0x2E812A0 0x2E81400 0x2E81448 0x2E81490 0x2E814D8 0x2E81520 0x2E81568 0x2E815B0
  fix arg7 0x2008 u16 alt const 0 || const 1
  fix arg7 0x200C u32 alt range 0 0x80 || any
  fix arg7 0x2010 u32 alt range 0 0x80 || any
  fix arg7 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg7 0x2018 u32 alt range 0 0x40 || any
  fix arg7 0x2024 u32 addrof arg7 0x20
  fix arg7 0x2028 u32 addrof arg7 0x50
  fix arg7 0x202C u8 alt const 0 || const 1
  fix arg7 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg7 0x202E u16 range 0 5
  fix arg7 0x2030 u16 range 0 4
  fix arg7 0x2034 u8 alt const 0 || const 1
  fix arg7 0x2074 u8 alt const 0 || const 1
  fix arg7 0x2078 u32 addrof arg7 0x0
  fix arg7 0x207C u16 alt const 0 || const 1 || const 2
  fix arg7 0x207E u16 alt const 0 || const 1
  fix arg7 0x2080 u32 oneof 0x2E81000 0x2E81070 0x2E810E0 0x2E81150 0x2E811C0 0x2E81230 0x2E812A0 0x2E81400 0x2E81448 0x2E81490 0x2E814D8 0x2E81520 0x2E81568 0x2E815B0
  fix arg7 0x2084 u16 alt const 0 || const 1
  fix arg7 0x2088 u32 alt range 0 0x80 || any
  fix arg7 0x208C u32 alt range 0 0x80 || any
  fix arg7 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg7 0x2094 u32 alt range 0 0x40 || any
  fix arg7 0x20A0 u32 addrof arg7 0x20
  fix arg7 0x20A4 u32 addrof arg7 0x50
  fix arg7 0x20A8 u8 alt const 0 || const 1
  fix arg7 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg7 0x20AA u16 range 0 5
  fix arg7 0x20AC u16 range 0 4
  fix arg7 0x20B0 u8 alt const 0 || const 1
  fix arg7 0x20F0 u8 alt const 0 || const 1
  fix arg7 0x20F4 u32 addrof arg7 0x0
  fix arg7 0x20FA u16 const 0
  fix arg7 0x25C u16 range 0 3
  fix arg7 0x25E u8 alt const 0 || const 1
  fix arg7 0x274 u8 alt const 0 || const 1
  fix arg7 0x276 u8 range 0 4
  fix arg7 0x278 u32 addrof arg7 0x0
  fix arg7 0x1000 u32 alt const 0 || const 6
  fix arg7 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg7 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1070 u32 alt const 1 || const 0
  fix arg7 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg7 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg7 0x10E0 u32 alt const 2 || const 1
  fix arg7 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg7 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1150 u32 alt const 3 || const 2
  fix arg7 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg7 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg7 0x11C0 u32 alt const 4 || const 3
  fix arg7 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg7 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1230 u32 alt const 5 || const 4
  fix arg7 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg7 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg7 0x12A0 u32 alt const 6 || const 5
  fix arg7 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg7 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1400 u32 const 0
  fix arg7 0x1404 u32 const 1
  fix arg7 0x1408 u32 alt range 1 4 || range 0 5
  fix arg7 0x140C u32 alt range 0 6 || const 9
  fix arg7 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1414 u32 alt range 0 8 || any
  fix arg7 0x1448 u32 const 1
  fix arg7 0x144C u32 const 2
  fix arg7 0x1450 u32 alt range 1 4 || range 0 5
  fix arg7 0x1454 u32 alt range 0 6 || const 9
  fix arg7 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x145C u32 alt range 0 8 || any
  fix arg7 0x1490 u32 const 2
  fix arg7 0x1494 u32 const 3
  fix arg7 0x1498 u32 alt range 1 4 || range 0 5
  fix arg7 0x149C u32 alt range 0 6 || const 9
  fix arg7 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x14A4 u32 alt range 0 8 || any
  fix arg7 0x14D8 u32 const 3
  fix arg7 0x14DC u32 const 4
  fix arg7 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg7 0x14E4 u32 alt range 0 6 || const 9
  fix arg7 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x14EC u32 alt range 0 8 || any
  fix arg7 0x1520 u32 const 4
  fix arg7 0x1524 u32 const 5
  fix arg7 0x1528 u32 alt range 1 4 || range 0 5
  fix arg7 0x152C u32 alt range 0 6 || const 9
  fix arg7 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1534 u32 alt range 0 8 || any
  fix arg7 0x1568 u32 const 5
  fix arg7 0x156C u32 const 6
  fix arg7 0x1570 u32 alt range 1 4 || range 0 5
  fix arg7 0x1574 u32 alt range 0 6 || const 9
  fix arg7 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x157C u32 alt range 0 8 || any
  fix arg7 0x15B0 u32 const 6
  fix arg7 0x15B4 u32 const 0
  fix arg7 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg7 0x15BC u32 alt range 0 6 || const 9
  fix arg7 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x15C4 u32 alt range 0 8 || any
  fix arg7 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg7 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg7 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg7 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg7 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg7 0x340 u16 alt range 0 3 || range 0 6
  fix arg7 0x342 u8 alt const 0 || const 1
  fix arg7 0x343 u8 range 0 4
  fix arg7 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg7 0x346 u16 alt range 0 8 || any
  fix arg7 0x25C u16 alt range 0 3 || range 0 6
  fix arg7 0x25E u8 alt const 0 || const 1
  fix arg7 0x25F u8 range 0 4
  fix arg7 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg7 0x262 u16 alt range 0 8 || any
  fix arg7 0x15C u16 alt range 0 3 || range 0 6
  fix arg7 0x15E u8 alt const 0 || const 1
  fix arg7 0x15F u8 range 0 4
  fix arg7 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg7 0x162 u16 alt range 0 8 || any
  stub 021b95c0 force ret=pick:0/1 args=v,v,v,v,v,v  # rail: can enter (blk, ln, pos, lat, rd, unit)
  stub 021b87d8 ret=pick:0x2E81000/0x2E81070/0x2E810E0/0x2E81150/0x2E811C0/0x2E81230/0x2E812A0 args=v,v  # rail: point (rd, i)
  stub 021b8514 ret=pick:0/0x2E81400/0x2E81448/0x2E81490/0x2E814D8/0x2E81520/0x2E81568/0x2E815B0 args=v,v,v,v,v  # rail: line of a point in a direction (pt, d, rd, blk, open)
  stub 021b8658 ret=none args=v,v,v,v,v,v,v  # rail: set the line state (w, ln, dir, pos, lat, len, wid)
  stub 021b963c force ret=pick:0/1 args=v,v  # rail: corner check (ln, rd)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b93d4 force ret=pick:0/1/2/3/4 args=v  # rail: direction back (d)
  stub 021b9510 force ret=pick:0/0x2E81400/0x2E81448/0x2E81490/0x2E814D8/0x2E81520/0x2E81568/0x2E815B0 args=v,v,v  # rail: line i unless blocked (blk, rd, i)
  stub 021b87fc ret=pick:0x2E81400/0x2E81448/0x2E81490/0x2E814D8/0x2E81520/0x2E81568/0x2E815B0 args=v,v  # rail: line (rd, i)

# FUN_021B8658(w, ln, dir, pos, lat, len, wid): set the line state.
fn 021b8658 ret=none inputs=20
  nowork
  arg u32 const 0x2E80100
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x3000
  fix arg7 0x2 u16 alt const 0 || const 1 || any
  fix arg7 0x4 u16 alt const 0 || const 1
  fix arg7 0x8 u32 addrof arg7 0x2000
  fix arg7 0xC u32 range 3 4
  fix arg7 0x14 u32 alt const 0 || const 0x2E80100 || oneof 0x2E82000 0x2E8207C
  fix arg7 0x18 u32 alt const 0 || const 1
  fix arg7 0x1C u32 addrof arg7 0x100
  fix arg7 0x20 u16 alt const 7 || range 0 7
  fix arg7 0x22 u16 alt const 7 || range 0 7
  fix arg7 0x24 u16 alt const 7 || range 0 7
  fix arg7 0x26 u16 alt const 7 || range 0 7
  fix arg7 0x28 u16 alt const 7 || range 0 7
  fix arg7 0x2A u16 alt const 7 || range 0 7
  fix arg7 0x2C u16 alt const 7 || range 0 7
  fix arg7 0x30 u32 addrof arg7 0x1000
  fix arg7 0x34 u32 addrof arg7 0x1400
  fix arg7 0x38 u32 addrof arg7 0x1600
  fix arg7 0x3C u32 addrof arg7 0x1700
  fix arg7 0x40 u32 addrof arg7 0x1800
  fix arg7 0x44 u32 addrof arg7 0x1840
  fix arg7 0x48 u32 addrof arg7 0x1880
  fix arg7 0x4C u32 alt range 0 16 || any
  fix arg7 0x50 u16 alt const 0xFFFF || range 0 5 rep=18,0x2
  fix arg7 0x100 u16 alt const 0 || const 1 || const 2
  fix arg7 0x102 u16 alt const 0 || const 1
  fix arg7 0x104 u32 oneof 0x2E81000 0x2E81070 0x2E810E0 0x2E81150 0x2E811C0 0x2E81230 0x2E812A0 0x2E81400 0x2E81448 0x2E81490 0x2E814D8 0x2E81520 0x2E81568 0x2E815B0
  fix arg7 0x108 u16 alt const 0 || const 1
  fix arg7 0x10C u32 alt range 0 0x80 || any
  fix arg7 0x110 u32 alt range 0 0x80 || any
  fix arg7 0x114 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg7 0x118 u32 alt range 0 0x40 || any
  fix arg7 0x124 u32 addrof arg7 0x20
  fix arg7 0x128 u32 addrof arg7 0x50
  fix arg7 0x12C u8 alt const 0 || const 1
  fix arg7 0x12D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg7 0x12E u16 range 0 5
  fix arg7 0x130 u16 range 0 4
  fix arg7 0x134 u8 alt const 0 || const 1
  fix arg7 0x174 u8 alt const 0 || const 1
  fix arg7 0x178 u32 addrof arg7 0x0
  fix arg7 0x2000 u16 alt const 0 || const 1 || const 2
  fix arg7 0x2002 u16 alt const 0 || const 1
  fix arg7 0x2004 u32 oneof 0x2E81000 0x2E81070 0x2E810E0 0x2E81150 0x2E811C0 0x2E81230 0x2E812A0 0x2E81400 0x2E81448 0x2E81490 0x2E814D8 0x2E81520 0x2E81568 0x2E815B0
  fix arg7 0x2008 u16 alt const 0 || const 1
  fix arg7 0x200C u32 alt range 0 0x80 || any
  fix arg7 0x2010 u32 alt range 0 0x80 || any
  fix arg7 0x2014 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg7 0x2018 u32 alt range 0 0x40 || any
  fix arg7 0x2024 u32 addrof arg7 0x20
  fix arg7 0x2028 u32 addrof arg7 0x50
  fix arg7 0x202C u8 alt const 0 || const 1
  fix arg7 0x202D u8 alt range 0 16 || range 0xF0 0xFF
  fix arg7 0x202E u16 range 0 5
  fix arg7 0x2030 u16 range 0 4
  fix arg7 0x2034 u8 alt const 0 || const 1
  fix arg7 0x2074 u8 alt const 0 || const 1
  fix arg7 0x2078 u32 addrof arg7 0x0
  fix arg7 0x207C u16 alt const 0 || const 1 || const 2
  fix arg7 0x207E u16 alt const 0 || const 1
  fix arg7 0x2080 u32 oneof 0x2E81000 0x2E81070 0x2E810E0 0x2E81150 0x2E811C0 0x2E81230 0x2E812A0 0x2E81400 0x2E81448 0x2E81490 0x2E814D8 0x2E81520 0x2E81568 0x2E815B0
  fix arg7 0x2084 u16 alt const 0 || const 1
  fix arg7 0x2088 u32 alt range 0 0x80 || any
  fix arg7 0x208C u32 alt range 0 0x80 || any
  fix arg7 0x2090 u32 alt range 0 0x40 || range 0xFFFFFFC0 0xFFFFFFFF || any
  fix arg7 0x2094 u32 alt range 0 0x40 || any
  fix arg7 0x20A0 u32 addrof arg7 0x20
  fix arg7 0x20A4 u32 addrof arg7 0x50
  fix arg7 0x20A8 u8 alt const 0 || const 1
  fix arg7 0x20A9 u8 alt range 0 16 || range 0xF0 0xFF
  fix arg7 0x20AA u16 range 0 5
  fix arg7 0x20AC u16 range 0 4
  fix arg7 0x20B0 u8 alt const 0 || const 1
  fix arg7 0x20F0 u8 alt const 0 || const 1
  fix arg7 0x20F4 u32 addrof arg7 0x0
  fix arg7 0x20FA u16 const 0
  fix arg7 0x25C u16 range 0 3
  fix arg7 0x25E u8 alt const 0 || const 1
  fix arg7 0x274 u8 alt const 0 || const 1
  fix arg7 0x276 u8 range 0 4
  fix arg7 0x278 u32 addrof arg7 0x0
  fix arg7 0x1000 u32 alt const 0 || const 6
  fix arg7 0x1004 u32 alt const 6 || const 0xFFFF || const 0
  fix arg7 0x1008 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x1010 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x1020 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x103C u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1070 u32 alt const 1 || const 0
  fix arg7 0x1074 u32 alt const 0 || const 0xFFFF || const 1
  fix arg7 0x1078 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x1080 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x1090 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x10AC u32 alt const 0xFFFF || range 0 5
  fix arg7 0x10E0 u32 alt const 2 || const 1
  fix arg7 0x10E4 u32 alt const 1 || const 0xFFFF || const 2
  fix arg7 0x10E8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x10F0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x1100 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x111C u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1150 u32 alt const 3 || const 2
  fix arg7 0x1154 u32 alt const 2 || const 0xFFFF || const 3
  fix arg7 0x1158 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x1160 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x1170 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x118C u32 alt const 0xFFFF || range 0 5
  fix arg7 0x11C0 u32 alt const 4 || const 3
  fix arg7 0x11C4 u32 alt const 3 || const 0xFFFF || const 4
  fix arg7 0x11C8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x11D0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x11E0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x11FC u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1230 u32 alt const 5 || const 4
  fix arg7 0x1234 u32 alt const 4 || const 0xFFFF || const 5
  fix arg7 0x1238 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x1240 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x1250 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x126C u32 alt const 0xFFFF || range 0 5
  fix arg7 0x12A0 u32 alt const 6 || const 5
  fix arg7 0x12A4 u32 alt const 5 || const 0xFFFF || const 6
  fix arg7 0x12A8 u32 alt const 0xFFFF || range 0 6 rep=2,0x4
  fix arg7 0x12B0 u32 alt range 1 4 || range 0 5 rep=4,0x4
  fix arg7 0x12C0 u32 alt range 0 0x40 || any rep=4,0x4
  fix arg7 0x12DC u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1400 u32 const 0
  fix arg7 0x1404 u32 const 1
  fix arg7 0x1408 u32 alt range 1 4 || range 0 5
  fix arg7 0x140C u32 alt range 0 6 || const 9
  fix arg7 0x1410 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1414 u32 alt range 0 8 || any
  fix arg7 0x1448 u32 const 1
  fix arg7 0x144C u32 const 2
  fix arg7 0x1450 u32 alt range 1 4 || range 0 5
  fix arg7 0x1454 u32 alt range 0 6 || const 9
  fix arg7 0x1458 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x145C u32 alt range 0 8 || any
  fix arg7 0x1490 u32 const 2
  fix arg7 0x1494 u32 const 3
  fix arg7 0x1498 u32 alt range 1 4 || range 0 5
  fix arg7 0x149C u32 alt range 0 6 || const 9
  fix arg7 0x14A0 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x14A4 u32 alt range 0 8 || any
  fix arg7 0x14D8 u32 const 3
  fix arg7 0x14DC u32 const 4
  fix arg7 0x14E0 u32 alt range 1 4 || range 0 5
  fix arg7 0x14E4 u32 alt range 0 6 || const 9
  fix arg7 0x14E8 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x14EC u32 alt range 0 8 || any
  fix arg7 0x1520 u32 const 4
  fix arg7 0x1524 u32 const 5
  fix arg7 0x1528 u32 alt range 1 4 || range 0 5
  fix arg7 0x152C u32 alt range 0 6 || const 9
  fix arg7 0x1530 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x1534 u32 alt range 0 8 || any
  fix arg7 0x1568 u32 const 5
  fix arg7 0x156C u32 const 6
  fix arg7 0x1570 u32 alt range 1 4 || range 0 5
  fix arg7 0x1574 u32 alt range 0 6 || const 9
  fix arg7 0x1578 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x157C u32 alt range 0 8 || any
  fix arg7 0x15B0 u32 const 6
  fix arg7 0x15B4 u32 const 0
  fix arg7 0x15B8 u32 alt range 1 4 || range 0 5
  fix arg7 0x15BC u32 alt range 0 6 || const 9
  fix arg7 0x15C0 u32 alt const 0xFFFF || range 0 5
  fix arg7 0x15C4 u32 alt range 0 8 || any
  fix arg7 0x1600 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg7 0x1700 u32 alt const 0xFFFF || range 0 5 rep=7,0x24
  fix arg7 0x1800 u32 alt const 0x02EE0011 || const 0 rep=7,0x4
  fix arg7 0x1840 u32 alt const 0x02EE0021 || const 0 rep=7,0x4
  fix arg7 0x1880 u32 alt const 0x02EE0031 || const 0 rep=7,0x4
  fix arg7 0x340 u16 alt range 0 3 || range 0 6
  fix arg7 0x342 u8 alt const 0 || const 1
  fix arg7 0x343 u8 range 0 4
  fix arg7 0x344 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg7 0x346 u16 alt range 0 8 || any
  fix arg7 0x25C u16 alt range 0 3 || range 0 6
  fix arg7 0x25E u8 alt const 0 || const 1
  fix arg7 0x25F u8 range 0 4
  fix arg7 0x260 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg7 0x262 u16 alt range 0 8 || any
  fix arg7 0x15C u16 alt range 0 3 || range 0 6
  fix arg7 0x15E u8 alt const 0 || const 1
  fix arg7 0x15F u8 range 0 4
  fix arg7 0x160 u16 alt range 0xFFF8 0xFFFF || range 0 8 || any
  fix arg7 0x162 u16 alt range 0 8 || any

# FUN_021B8674(rd, ln, p): ln starts at p.
fn 021b8674 ret=u32 inputs=100
  nowork
  arg u32 const 0x2E40020
  arg u32 oneof 0x2E41400 0x2E41448 0x2E41490 0x2E414D8
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
  stub 021b87d8 ret=pick:0x2E41000/0x2E41070/0x2E410E0/0x2E41150/0x2E411C0/0x2E41230/0x2E412A0 args=v,v  # rail: point (rd, i)

# FUN_021B868C(rd, ln, p): ln ends at p.
fn 021b868c ret=u32 inputs=100
  nowork
  arg u32 const 0x2E40020
  arg u32 oneof 0x2E41400 0x2E41448 0x2E41490 0x2E414D8
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
  stub 021b87d8 ret=pick:0x2E41000/0x2E41070/0x2E410E0/0x2E41150/0x2E411C0/0x2E41230/0x2E412A0 args=v,v  # rail: point (rd, i)

# FUN_021B86A4(rd, ln, side, start, blk, unit): widths around the start of ln.
fn 021b86a4 ret=none inputs=400
  nowork
  arg u32 const 0x2E70020
  arg u32 oneof 0x2E71400 0x2E71448 0x2E71490 0x2E714D8
  arg u32 const 0x2E703F0
  arg u32 const 0x2E703F4
  arg u32 const 0x2E70050
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
  stub 021b87d8 ret=pick:0x2E71000/0x2E71070/0x2E710E0/0x2E71150/0x2E711C0/0x2E71230/0x2E712A0 args=v,v  # rail: point (rd, i)
  stub 021b93d4 force ret=pick:0/1/2/3/4 args=v  # rail: direction back (d)
  stub 021b8514 ret=pick:0/0x2E71400/0x2E71448/0x2E71490/0x2E714D8/0x2E71520/0x2E71568/0x2E715B0 args=v,v,v,v,v  # rail: line of a point in a direction (pt, d, rd, blk, open)
  stub 021b9434 force ret=pick:0/1/2/3/4 args=v  # rail: direction left (d)
  stub 021b9404 force ret=pick:0/1/2/3/4 args=v  # rail: direction right (d)
  stub 021b9278 force ret=pick+u32:0/0x10/0x40/0x80 args=v  # rail: line length (ln)
  stub 021b9280 force ret=pick+u32:0/0x10/0x20/0x40 args=v,v,v,v  # rail: line half width at a position (ln, pos, len, rd)
  stub 021b8674 ret=pick:0/1 args=v,v,v  # rail: line starts at the point (rd, ln, p)
  stub 021b9510 force ret=pick:0/0x2E71400/0x2E71448/0x2E71490/0x2E714D8/0x2E71520/0x2E71568/0x2E715B0 args=v,v,v  # rail: line i unless blocked (blk, rd, i)
  stub 021b87fc ret=pick:0x2E71400/0x2E71448/0x2E71490/0x2E714D8/0x2E71520/0x2E71568/0x2E715B0 args=v,v  # rail: line (rd, i)

# FUN_021B87BC(rd): zero 0x30 bytes.
fn 021b87bc ret=none inputs=10
  nowork
  arg u32 const 0x2E20020
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

# FUN_021B87C8(dst, src): copy 0x30 bytes.
fn 021b87c8 ret=none inputs=10
  nowork
  arg u32 const 0x2E30020
  arg u32 const 0x2E31000
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

# FUN_021B87D8(rd, i): point i.
fn 021b87d8 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30020
  arg u32 alt range 0 7 || const 0xFFFF || any
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

# FUN_021B87FC(rd, i): line i.
fn 021b87fc ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30020
  arg u32 alt range 0 7 || const 0xFFFF || any
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

# FUN_021B8820(rd, i): camera record i.
fn 021b8820 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30020
  arg u32 alt range 0 7 || const 0xFFFF || any
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

# FUN_021B8848(rd, i): line record i.
fn 021b8848 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30020
  arg u32 alt range 0 7 || const 0xFFFF || any
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

# FUN_021B8870(rd, i): camera proc i.
fn 021b8870 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30020
  arg u32 alt range 0 7 || const 0xFFFF || any
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

# FUN_021B8894(rd, i): position proc i.
fn 021b8894 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30020
  arg u32 alt range 0 7 || const 0xFFFF || any
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

# FUN_021B88B8(rd, i): line-hit proc i.
fn 021b88b8 ret=u32 inputs=60
  nowork
  arg u32 const 0x2E30020
  arg u32 alt range 0 7 || const 0xFFFF || any
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
