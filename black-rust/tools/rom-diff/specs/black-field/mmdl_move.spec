# fndiff spec: MMDL movement helpers (overlay 10 Thumb code 0x02163010..0x021643AC, F1 row 10.23; F6a-spec): the grid move procs
# (02163010 / 02163028), height and attribute tracking (02164010 / 021640B0), attribute-driven field effects (021631C0 ATTR INFO and the
# 0216xxxx effect handlers), hit checks (021638CC: move limit, attribute blocks, height, other MMDLs, map bounds), direction tables and the
# rail-header glue (02164304). fldmmdl.spec stubs 02163010 / 02163028 / 02163C04 / 02163F10 / 02164010 / 021642F8 / 02164304.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field mmdl_move --per-function
# Set `field` (main 10 11 18 20 21). F1 row 10.23 ("eventdata_system.c", 0x02161D98..0x02164768) holds four files; F6a-spec splits it by
# address into the areas wfbc_data (0x02161D98..0x0216237C), eventdata (0x0216237C..0x02163010), mmdl_move (0x02163010..0x021643AC) and
# eventdata_tail (0x021643AC..0x02164768); calls between them are `force` stubs (one: eventdata 02162970 -> eventdata_tail 02164708).
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry; 197 entries in
# the row, minus 12 veneers (`ldr r3,=T; bx r3`, callers stub T: 02162024 -> 021622AC, 02162038 -> 02162330, 02162040 -> 02162358,
# 02162A64 -> 02162CE8, 02162A6C -> 02162ECC, 021626A8 -> 02162B68, 021626B0 -> 02162BCC, 02164454 / 5C / 64 / 6C -> 0215E34C,
# 02164474 -> 0215E364) and 3 false entries (02161E82, 021634A8 inside 02163464, 02163E72 inside 02163E58: data words in main
# 0x020A3E5A..0x020A481C decode as bl), = 185 specced (wfbc_data 40, eventdata 56, mmdl_move 70, eventdata_tail 19). Ghidra misses 02162370,
# 0216237C, 02162458, 02162478, 021624B0, 02162560, 021625E8, 021621C0, 02164240, 0216458C, 021645C0 (and splits 9 switch cases / tails
# into LAB_ functions). Render-only functions: none (the draw side is reached only through overlay-21 field-effect callees, all stubs).
# Closure stubs (as core4 / F4a): a block lists the stubs of its callees and of the callees of its in-area descendants and the shared fix
# lines, so the area can be ported in any order (checked with every function treated as ported: no fault, stray write or unstubbed
# callee). Out-of-area callees are `force` stubs (overlay addresses collide with battle PORTED entries; ARM9 / fldmmdl callees are ported
# in other black-field areas). Indirect calls through ROM tables reach in-area stubs (plain) or force stubs at the table targets.
#
# MMDL fields are read only through fldmmdl getters (all `force` stubs here: grid x/y/z +0x3C..+0x40 = 0216D83C / D854 / D86C, old
# grid +0x36.. = 0216D804 / D810 / D81C, position VecFx32 +0x44 = 0216D88C / D89C, y = 0216D8AC, attributes +0x74 / +0x78, move bits +4).
# ATTR INFO (0x1C, on the caller's stack, 021631C0): +0 u32 previous attribute (-1 = none), +4 u32 attribute, +8 / +0xA u16 values
# (021AB0F0), +0xC / +0xE u16 flags (021AB0F8: 2, 8, 0x20 effect, 0x40, 0x80), +0x10 -> OBJ DATA (0216E3C4; +4, +5 (2 = ..), +6 read),
# +0x14 field-effect control, +0x18 u8 season (021ABE28).
# Direction 0..3 = up, down, left, right (dx ROM 0x02171480 = 0, 0, -1, 1; dz 0x02171470 = -1, 1, 0, 0; opposite 0x021714B0 = 1, 0, 3, 2).
# Grid <-> fx32: tile = (fx >> 4) / 4096 (toward zero), centre = tile << 16 + 0x8000.
# 02163D04 calls attribute predicates through ROM 0x021714A0 / 0x02171490 [dir] (overlay-21 021AB458 / 474 / 490 / 4A4): dir must be 0..3.
#
# Arena (the `work` image): MMDLSYS work+0 (0216D67C answer), MMDL work+0x100 (value only; another MMDL work+0x200 / the 0216DD1C
# iterator answers work+0x100 / +0x104), MMDL move work work+0x194 (0216D69C), ATTR INFO work+0x800, OBJ DATA work+0x900, VecFx32
# work+0xA00, rail location work+0xA20, rectangles work+0xA40 / 0xA48, map query out work+0xA60, MMDL header work+0xA80.
area mmdl_move
set field

# 02163010(m): 0216D6F4(m) (move init proc); 0219B414(m); 0216D548(m, 2) (status |= move proc ready)
fn 02163010 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d548 force ret=none args=v,v  # MMDL +0 |= v
  stub 0216d6f4 force ret=none args=v  # MMDL move proc init
  stub 0219b414 force ret=none args=v  # fldmmdl_acmd: move init (m)

# 02163028(m): 0216DA20(m, 2) (MMDLSYS paused?) != 0: return; 021630F4, 0216310C, 02163130 (m); move bit 0x200 (acmd set):
# 02197EB4(m); else not paused (0216DB10 == 0) and 02163090(m) == 1 and 0219B41C(m) == 0: 0216D708(m) (move proc);
# 02163154(m); 02163188(m)
fn 02163028 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163090 ret=pick:0/1 args=v  # in-area
  stub 021630f4 ret=none args=v  # in-area
  stub 0216310c ret=none args=v  # in-area
  stub 02163130 ret=none args=v  # in-area
  stub 02163154 ret=none args=v  # in-area
  stub 02163188 ret=none args=v  # in-area
  stub 021631c0 ret=none args=v,out:0x1C  # in-area
  stub 02163230 ret=none args=v  # in-area
  stub 02163288 ret=none args=v  # in-area
  stub 021632f0 ret=none args=v  # in-area
  stub 02163340 ret=none args=v  # in-area
  stub 021633a8 ret=none args=v  # in-area
  stub 02163418 ret=none args=v,v  # in-area
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 021634d8 ret=none args=v,v  # in-area
  stub 021634fc ret=none args=v,v  # in-area
  stub 0216352c ret=none args=v,v  # in-area
  stub 0216355c ret=none args=v,v  # in-area
  stub 02163580 ret=none args=v,v  # in-area
  stub 02163618 ret=none args=v,v  # in-area
  stub 02163650 ret=none args=v,v  # in-area
  stub 0216369c ret=none args=v,v  # in-area
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 021636f0 ret=none args=v,v  # in-area
  stub 0216372c ret=none args=v,v  # in-area
  stub 0216375c ret=none args=v,v  # in-area
  stub 0216377c ret=none args=v,v  # in-area
  stub 021637b0 ret=none args=v,v  # in-area
  stub 021637e4 ret=none args=v,v  # in-area
  stub 02163868 ret=none args=v,v  # in-area
  stub 021638ac ret=none args=v,v  # in-area
  stub 021638c8 ret=none args=v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163e58 ret=pick:0/1 args=v,in:12,out:4  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 02164010 ret=pick:0/1 args=v  # in-area
  stub 021640b0 ret=pick:0/1 args=v  # in-area
  stub 021641c8 ret=none args=v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d534 force ret=u32 args=v  # MMDLSYS +0x3C
  stub 0216d55c force ret=pick:0/1 args=v,v  # MMDL +0 & v
  stub 0216d564 force ret=pick:0/0x800/0x1000/0x1800/0xFFFFFFFF args=v  # MMDL +4
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d5a0 force ret=pick:0x30/0x32/0x33/0x36/0x37/0/1 args=v  # MMDL move code
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # MMDL move dir
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d708 force ret=none args=v  # MMDL move proc
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d7e4 force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x78 (previous attribute)
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d818 force ret=none args=v,v  # MMDL old gy = v
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gy
  stub 0216d85c force ret=none args=v,v  # MMDL gy = v
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # MMDL position VecFx32 = *v
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216d900 force ret=none args=v,in:12  # MMDL +0x68 VecFx32 = *v
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 0216da20 force ret=pick:0/1 args=v,v  # MMDLSYS flag of an MMDL
  stub 0216da5c force ret=pick:0/1 args=v  # MMDL +4 & 0x10
  stub 0216da70 force ret=none args=v  # MMDL +4 |= 0x20
  stub 0216db10 force ret=pick:0/1 args=v  # MMDL +4 & 8 (paused)
  stub 0216db24 force ret=pick:0/1 args=v  # MMDL draw ready
  stub 0216db68 force ret=pick:0/1 args=v  # MMDL +0 & 0x200
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dbe8 force ret=pick:0/1 args=v  # MMDL +4 & 0x40000
  stub 0216dc00 force ret=pick:0/1 args=v  # MMDL +0 & 0x1000
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216dd04 force ret=pick:0/1 args=v  # MMDL +4 & 0x200000
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188cd0 force ret=u32 args=v  # field effect control of a field system
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 02197eb4 force ret=none args=v  # fldmmdl_acmd: acmd proc (m)
  stub 0219b41c force ret=pick:0/1 args=v  # fldmmdl_acmd: move busy (m)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab2f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab304 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab334 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab358 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab364 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3e0 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab440 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021abe28 force ret=pick:0/1/2/3 args=v  # season / weather byte
  stub 021ac2c0 force ret=none args=v,v  # field effect (m, ctrl)
  stub 021ac528 force ret=none args=v,v  # field effect (m, ctrl)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)
  stub 021bbdd4 force ret=none args=v,v,v  # field effect (m, ctrl, kind)
  stub 021bbfb8 force ret=none args=v,v,v,v  # field effect (sys, m, ctrl, kind)
  stub 021c484c force ret=none args=v,v,v,v  # field effect (ctrl, x, z, y)
  stub 021c4a10 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c4be0 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c8f7c force ret=none args=v,v  # field effect (m, ctrl)

# 02163090(m) -> bool: 0216DA5C(m): 1; move bits & 0x1800 == 0: 1; move code 0x32: 1; bits = 0216D564(m): 0x1000 and
# 0216DB68(m) == 0: 0; 0x800 and 0216DCB4(m) == 0: 0; 1
fn 02163090 ret=u32 inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d564 force ret=pick:0/0x800/0x1000/0x1800/0xFFFFFFFF args=v  # MMDL +4
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d5a0 force ret=pick:0x30/0x32/0x33/0x36/0x37/0/1 args=v  # MMDL move code
  stub 0216da5c force ret=pick:0/1 args=v  # MMDL +4 & 0x10
  stub 0216db68 force ret=pick:0/1 args=v  # MMDL +0 & 0x200
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400

# 021630F4(m): move bit 0x1000: 02164010(m) (height)
fn 021630f4 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e58 ret=pick:0/1 args=v,in:12,out:4  # in-area
  stub 02164010 ret=pick:0/1 args=v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d55c force ret=pick:0/1 args=v,v  # MMDL +0 & v
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d818 force ret=none args=v,v  # MMDL old gy = v
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gy
  stub 0216d85c force ret=none args=v,v  # MMDL gy = v
  stub 0216d88c force ret=none args=v,out:12  # MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # MMDL position VecFx32 = *v
  stub 0216db68 force ret=pick:0/1 args=v  # MMDL +0 & 0x200
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)

# 0216310C(m): move bit 0x800 and 021640B0(m) == 1: 0216DA70(m)
fn 0216310c ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 021640b0 ret=pick:0/1 args=v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216da70 force ret=none args=v  # MMDL +4 |= 0x20
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)

# 02163130(m): move bit 0x20: 021641C8(m); 02163230(m); move bits &= ~0xA0
fn 02163130 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021631c0 ret=none args=v,out:0x1C  # in-area
  stub 02163230 ret=none args=v  # in-area
  stub 02163418 ret=none args=v,v  # in-area
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 021634d8 ret=none args=v,v  # in-area
  stub 02163618 ret=none args=v,v  # in-area
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 021636f0 ret=none args=v,v  # in-area
  stub 021637e4 ret=none args=v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 021640b0 ret=pick:0/1 args=v  # in-area
  stub 021641c8 ret=none args=v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d534 force ret=u32 args=v  # MMDLSYS +0x3C
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d5a0 force ret=pick:0x30/0x32/0x33/0x36/0x37/0/1 args=v  # MMDL move code
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d708 force ret=none args=v  # MMDL move proc
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d7e4 force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x78 (previous attribute)
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216d900 force ret=none args=v,in:12  # MMDL +0x68 VecFx32 = *v
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 0216db24 force ret=pick:0/1 args=v  # MMDL draw ready
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dbe8 force ret=pick:0/1 args=v  # MMDL +4 & 0x40000
  stub 0216dc00 force ret=pick:0/1 args=v  # MMDL +0 & 0x1000
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188cd0 force ret=u32 args=v  # field effect control of a field system
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab304 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab358 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab440 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021abe28 force ret=pick:0/1/2/3 args=v  # season / weather byte
  stub 021ac2c0 force ret=none args=v,v  # field effect (m, ctrl)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)
  stub 021bbfb8 force ret=none args=v,v,v,v  # field effect (sys, m, ctrl, kind)
  stub 021c4be0 force ret=none args=v,v  # field effect (ctrl, m)

# 02163154(m): move bit 0x80: 021632F0(m); else bit 0x20: 02163288(m); move bits &= ~0xA0
fn 02163154 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021631c0 ret=none args=v,out:0x1C  # in-area
  stub 02163288 ret=none args=v  # in-area
  stub 021632f0 ret=none args=v  # in-area
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 021634fc ret=none args=v,v  # in-area
  stub 02163580 ret=none args=v,v  # in-area
  stub 02163650 ret=none args=v,v  # in-area
  stub 0216369c ret=none args=v,v  # in-area
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 021636f0 ret=none args=v,v  # in-area
  stub 0216377c ret=none args=v,v  # in-area
  stub 021637e4 ret=none args=v,v  # in-area
  stub 021638c8 ret=none args=v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 021640b0 ret=pick:0/1 args=v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d534 force ret=u32 args=v  # MMDLSYS +0x3C
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # MMDL move dir
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d7e4 force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x78 (previous attribute)
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 0216db24 force ret=pick:0/1 args=v  # MMDL draw ready
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dbe8 force ret=pick:0/1 args=v  # MMDL +4 & 0x40000
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188cd0 force ret=u32 args=v  # field effect control of a field system
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab2f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab304 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab334 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab358 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab364 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3e0 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab440 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021abe28 force ret=pick:0/1/2/3 args=v  # season / weather byte
  stub 021ac2c0 force ret=none args=v,v  # field effect (m, ctrl)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)
  stub 021bbdd4 force ret=none args=v,v,v  # field effect (m, ctrl, kind)
  stub 021bbfb8 force ret=none args=v,v,v,v  # field effect (sys, m, ctrl, kind)
  stub 021c484c force ret=none args=v,v,v,v  # field effect (ctrl, x, z, y)
  stub 021c4a10 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c4be0 force ret=none args=v,v  # field effect (ctrl, m)

# 02163188(m): move bit 0x100: 021633A8(m); else bit 0x40: 02163340(m); move bits &= ~0x140
fn 02163188 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021631c0 ret=none args=v,out:0x1C  # in-area
  stub 02163340 ret=none args=v  # in-area
  stub 021633a8 ret=none args=v  # in-area
  stub 02163418 ret=none args=v,v  # in-area
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 0216352c ret=none args=v,v  # in-area
  stub 0216355c ret=none args=v,v  # in-area
  stub 02163650 ret=none args=v,v  # in-area
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 0216372c ret=none args=v,v  # in-area
  stub 0216375c ret=none args=v,v  # in-area
  stub 021637b0 ret=none args=v,v  # in-area
  stub 02163868 ret=none args=v,v  # in-area
  stub 021638ac ret=none args=v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 021640b0 ret=pick:0/1 args=v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d534 force ret=u32 args=v  # MMDLSYS +0x3C
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # MMDL move dir
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d7e4 force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x78 (previous attribute)
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216d900 force ret=none args=v,in:12  # MMDL +0x68 VecFx32 = *v
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 0216db24 force ret=pick:0/1 args=v  # MMDL draw ready
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dbe8 force ret=pick:0/1 args=v  # MMDL +4 & 0x40000
  stub 0216dc00 force ret=pick:0/1 args=v  # MMDL +0 & 0x1000
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216dd04 force ret=pick:0/1 args=v  # MMDL +4 & 0x200000
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188cd0 force ret=u32 args=v  # field effect control of a field system
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab2f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab304 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab364 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021abe28 force ret=pick:0/1/2/3 args=v  # season / weather byte
  stub 021ac528 force ret=none args=v,v  # field effect (m, ctrl)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)
  stub 021c484c force ret=none args=v,v,v,v  # field effect (ctrl, x, z, y)
  stub 021c4a10 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c4be0 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c8f7c force ret=none args=v,v  # field effect (m, ctrl)

# 021631C0(m, info): MI_CpuFill (memset) info 0x1C; +0 = 0216D7E4(m) (previous attribute), +4 = 0216D7DC(m) (attribute);
# +4 != -1: +0xA = 021AB0F0(+4) (value), +0xE = 021AB0F8(+4) (flags); +0 != -1: +8, +0xC likewise; +0x10 = 0216E3C4(m) (OBJ data);
# +0x14 = 02188CD0(0216D534(0216D67C(m))) (field effect control); +0x18 = 021ABE28(..) (u8)
fn 021631c0 ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d534 force ret=u32 args=v  # MMDLSYS +0x3C
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e4 force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x78 (previous attribute)
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188cd0 force ret=u32 args=v  # field effect control of a field system
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021abe28 force ret=pick:0/1/2/3 args=v  # season / weather byte

# 02163230(m): attribute effects on a step start: 021640B0(m); move bit 0x800 == 0 and 0216DB24(m) == 1 (draw ready): info on the stack, then 021631C0, 021634D8, 02163618, 021636F0, 02163418, 021637E4 (m, &info)
fn 02163230 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021631c0 ret=none args=v,out:0x1C  # in-area
  stub 02163418 ret=none args=v,v  # in-area
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 021634d8 ret=none args=v,v  # in-area
  stub 02163618 ret=none args=v,v  # in-area
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 021636f0 ret=none args=v,v  # in-area
  stub 021637e4 ret=none args=v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 021640b0 ret=pick:0/1 args=v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d534 force ret=u32 args=v  # MMDLSYS +0x3C
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d7e4 force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x78 (previous attribute)
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216d900 force ret=none args=v,in:12  # MMDL +0x68 VecFx32 = *v
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 0216db24 force ret=pick:0/1 args=v  # MMDL draw ready
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dbe8 force ret=pick:0/1 args=v  # MMDL +4 & 0x40000
  stub 0216dc00 force ret=pick:0/1 args=v  # MMDL +0 & 0x1000
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188cd0 force ret=u32 args=v  # field effect control of a field system
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab304 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab358 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab440 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021abe28 force ret=pick:0/1/2/3 args=v  # season / weather byte
  stub 021ac2c0 force ret=none args=v,v  # field effect (m, ctrl)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)
  stub 021bbfb8 force ret=none args=v,v,v,v  # field effect (sys, m, ctrl, kind)
  stub 021c4be0 force ret=none args=v,v  # field effect (ctrl, m)

# 02163288(m): attribute effects on a step end: 021640B0(m); move bit 0x800 == 0 and 0216DB24(m) == 1 (draw ready): info on the stack, then 021631C0, 021634FC, 02163580, 02163650, 021636F0, 0216377C, 021637E4, 021638C8 (m, &info)
fn 02163288 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021631c0 ret=none args=v,out:0x1C  # in-area
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 021634fc ret=none args=v,v  # in-area
  stub 02163580 ret=none args=v,v  # in-area
  stub 02163650 ret=none args=v,v  # in-area
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 021636f0 ret=none args=v,v  # in-area
  stub 0216377c ret=none args=v,v  # in-area
  stub 021637e4 ret=none args=v,v  # in-area
  stub 021638c8 ret=none args=v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 021640b0 ret=pick:0/1 args=v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d534 force ret=u32 args=v  # MMDLSYS +0x3C
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # MMDL move dir
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d7e4 force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x78 (previous attribute)
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 0216db24 force ret=pick:0/1 args=v  # MMDL draw ready
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dbe8 force ret=pick:0/1 args=v  # MMDL +4 & 0x40000
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188cd0 force ret=u32 args=v  # field effect control of a field system
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab2f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab304 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab328 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab334 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab358 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab364 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3e0 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab440 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021abe28 force ret=pick:0/1/2/3 args=v  # season / weather byte
  stub 021ac2c0 force ret=none args=v,v  # field effect (m, ctrl)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)
  stub 021bbdd4 force ret=none args=v,v,v  # field effect (m, ctrl, kind)
  stub 021bbfb8 force ret=none args=v,v,v,v  # field effect (sys, m, ctrl, kind)
  stub 021c484c force ret=none args=v,v,v,v  # field effect (ctrl, x, z, y)
  stub 021c4a10 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c4be0 force ret=none args=v,v  # field effect (ctrl, m)

# 021632F0(m): attribute effects on a jump end: 021640B0(m); move bit 0x800 == 0 and 0216DB24(m) == 1 (draw ready): info on the stack, then 021631C0, 021636F0, 021637E4, 0216369C, 021638C8 (m, &info)
fn 021632f0 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021631c0 ret=none args=v,out:0x1C  # in-area
  stub 0216369c ret=none args=v,v  # in-area
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 021636f0 ret=none args=v,v  # in-area
  stub 021637e4 ret=none args=v,v  # in-area
  stub 021638c8 ret=none args=v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 021640b0 ret=pick:0/1 args=v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d534 force ret=u32 args=v  # MMDLSYS +0x3C
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d7e4 force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x78 (previous attribute)
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 0216db24 force ret=pick:0/1 args=v  # MMDL draw ready
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188cd0 force ret=u32 args=v  # field effect control of a field system
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021ab358 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab440 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021abe28 force ret=pick:0/1/2/3 args=v  # season / weather byte
  stub 021ac2c0 force ret=none args=v,v  # field effect (m, ctrl)
  stub 021bbfb8 force ret=none args=v,v,v,v  # field effect (sys, m, ctrl, kind)

# 02163340(m): attribute effects on a step start (2): 021640B0(m); move bit 0x800 == 0 and 0216DB24(m) == 1 (draw ready): info on the stack, then 021631C0, 02163418, 0216352C, 021637B0, 02163650, 02163868, 0216372C, 021638AC (m, &info)
fn 02163340 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021631c0 ret=none args=v,out:0x1C  # in-area
  stub 02163418 ret=none args=v,v  # in-area
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 0216352c ret=none args=v,v  # in-area
  stub 02163650 ret=none args=v,v  # in-area
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 0216372c ret=none args=v,v  # in-area
  stub 021637b0 ret=none args=v,v  # in-area
  stub 02163868 ret=none args=v,v  # in-area
  stub 021638ac ret=none args=v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 021640b0 ret=pick:0/1 args=v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d534 force ret=u32 args=v  # MMDLSYS +0x3C
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # MMDL move dir
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d7e4 force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x78 (previous attribute)
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216d900 force ret=none args=v,in:12  # MMDL +0x68 VecFx32 = *v
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 0216db24 force ret=pick:0/1 args=v  # MMDL draw ready
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dbe8 force ret=pick:0/1 args=v  # MMDL +4 & 0x40000
  stub 0216dc00 force ret=pick:0/1 args=v  # MMDL +0 & 0x1000
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188cd0 force ret=u32 args=v  # field effect control of a field system
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab2f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab304 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab364 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021abe28 force ret=pick:0/1/2/3 args=v  # season / weather byte
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)
  stub 021c484c force ret=none args=v,v,v,v  # field effect (ctrl, x, z, y)
  stub 021c4a10 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c4be0 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c8f7c force ret=none args=v,v  # field effect (m, ctrl)

# 021633A8(m): attribute effects on a step end (2): 021640B0(m); move bit 0x800 == 0 and 0216DB24(m) == 1 (draw ready): info on the stack, then 021631C0, 02163418, 021637B0, 02163650, 02163868, 0216372C, 0216355C, 0216375C, 021638AC (m, &info)
fn 021633a8 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021631c0 ret=none args=v,out:0x1C  # in-area
  stub 02163418 ret=none args=v,v  # in-area
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 0216355c ret=none args=v,v  # in-area
  stub 02163650 ret=none args=v,v  # in-area
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 0216372c ret=none args=v,v  # in-area
  stub 0216375c ret=none args=v,v  # in-area
  stub 021637b0 ret=none args=v,v  # in-area
  stub 02163868 ret=none args=v,v  # in-area
  stub 021638ac ret=none args=v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 021640b0 ret=pick:0/1 args=v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d534 force ret=u32 args=v  # MMDLSYS +0x3C
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d7e4 force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x78 (previous attribute)
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216d900 force ret=none args=v,in:12  # MMDL +0x68 VecFx32 = *v
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 0216db24 force ret=pick:0/1 args=v  # MMDL draw ready
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dbe8 force ret=pick:0/1 args=v  # MMDL +4 & 0x40000
  stub 0216dc00 force ret=pick:0/1 args=v  # MMDL +0 & 0x1000
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216dd04 force ret=pick:0/1 args=v  # MMDL +4 & 0x200000
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188cd0 force ret=u32 args=v  # field effect control of a field system
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab2f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab304 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab364 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021abe28 force ret=pick:0/1/2/3 args=v  # season / weather byte
  stub 021ac528 force ret=none args=v,v  # field effect (m, ctrl)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)
  stub 021c484c force ret=none args=v,v,v,v  # field effect (ctrl, x, z, y)
  stub 021c4a10 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c4be0 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c8f7c force ret=none args=v,v  # field effect (m, ctrl)

# 02163418(m, info): 0216DC00(m) == 0 and 021AB41C(info +0xA) != 0: 0216D900(m, ROM 0x021714D0 = (0, -0x7000, 0)); else
# 0216D900(m, (0, 0, 0)) (draw offset)
fn 02163418 ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d900 force ret=none args=v,in:12  # MMDL +0x68 VecFx32 = *v
  stub 0216dc00 force ret=pick:0/1 args=v  # MMDL +0 & 0x1000
  stub 021ab41c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)

# 02163464(attr, s) -> kind: 021AB4EC: 6; 021AB4F8: 7; 021AB434: s == 3 ? 5 : 1; 021AB428: s == 3 ? 4 : 0; 021AB250: 1;
# 021AB264: 2; 021AB270: 3; else 0
fn 02163464 ret=u32 inputs=300
  nowork
  arg u32 any
  arg u32 alt range 0 3 || any
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)

# 021634D8(m, info): info +0xE & 0x20: 021AC760(info +0x14, m, 0, 02163464(info +0xA, info +0x18))
fn 021634d8 ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)

# 021634FC(m, info): info +0xE & 0x20: d = 0216D608(m); k = 02163464(info +0xA, info +0x18); d != 0: 021AC760(info +0x14, m, 1, k)
fn 021634fc ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # MMDL move dir
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)

# 0216352C(m, info): as 021634FC with d == 0
fn 0216352c ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 0216d608 force ret=pick:0/1/2/3 args=v  # MMDL move dir
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)

# 0216355C(m, info): info +0xE & 0x20: 021AC760(info +0x14, m, 1, 02163464(info +0xA, info +0x18))
fn 0216355c ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163464 ret=pick:0/1/3/7 args=v,v  # in-area
  stub 021ab250 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab264 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab270 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab428 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab434 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ac760 force ret=none args=v,v,v,v  # field effect (ctrl, m, flag, kind)

# 02163580(m, info): info +0xC & 8: k from the predicates of info +8 (021AB3D4, 021AB3E0, 021AB334, 021AB328, 021AB41C), the
# season +0x18 and OBJ data +5; k != 5: 021BBDD4(m, info +0x14, k)
fn 02163580 ret=none inputs=300
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021ab328 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab334 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab3e0 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab41c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021bbdd4 force ret=none args=v,v,v  # field effect (m, ctrl, kind)

# 02163618(m, info): 021AB304(info +0xA) == 1: 0216DBE8(m) == 0: 021C4BE0(info +0x14, m); 0216DBCC(m, 1); else 0216DBCC(m, 0)
fn 02163618 ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dbe8 force ret=pick:0/1 args=v  # MMDL +4 & 0x40000
  stub 021ab304 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021c4be0 force ret=none args=v,v  # field effect (ctrl, m)

# 02163650(m, info): as 02163618, and 021AB364(info +0xA) == 1: 021C4A10(info +0x14, m) instead of clearing
fn 02163650 ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on
  stub 0216dbe8 force ret=pick:0/1 args=v  # MMDL +4 & 0x40000
  stub 021ab304 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab364 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021c4a10 force ret=none args=v,v  # field effect (ctrl, m)
  stub 021c4be0 force ret=none args=v,v  # field effect (ctrl, m)

# 0216369C(m, info): tail 0216DBCC(m, 0)
fn 0216369c ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216dbcc force ret=none args=v,v  # MMDL +4 bit 0x40000 = on

# 021636A8(m, info) -> bool: 0216D98C(0216D67C(m)) != 0 and OBJ data (info +0x10) +4 != 0
fn 021636a8 ret=u32 inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag

# 021636C8(info) -> bool: info +0xE & 0x80 == 0: 0; info +0x18 == 3 and 021AB3D4(info +0xA) != 0: 0; 1
fn 021636c8 ret=u32 inputs=100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)

# 021636F0(m, info): 021636A8 == 1 and 021636C8 == 1 and move bit 0x4000 clear: 021AC2C0(m, info +0x14); bit 0x4000 set
fn 021636f0 ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d57c force ret=pick:0/1 args=v,v  # MMDL +4 & v
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ac2c0 force ret=none args=v,v  # field effect (m, ctrl)

# 0216372C(m, info): 021636A8 == 1: 021636C8(info) == 0 ? set : clear move bit 0x8000
fn 0216372c ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021636a8 ret=pick:0/1 args=v,v  # in-area
  stub 021636c8 ret=pick:0/1 args=v  # in-area
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d98c force ret=pick:0/1 args=v  # MMDLSYS draw flag
  stub 021ab3d4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)

# 0216375C(m, info): info +0xE & 2 == 0 and 0216DD04(m) == 0: 021AC528(m, info +0x14)
fn 0216375c ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216dd04 force ret=pick:0/1 args=v  # MMDL +4 & 0x200000
  stub 021ac528 force ret=none args=v,v  # field effect (m, ctrl)

# 0216377C(m, info): 021AB2F8(info +8) == 1: 021C484C(info +0x14, old gx, old gz, pos y)
fn 0216377c ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 021ab2f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021c484c force ret=none args=v,v,v,v  # field effect (ctrl, x, z, y)

# 021637B0(m, info): 021AB2F8(info +0xA) == 1: 021C484C(info +0x14, gx, gz, pos y)
fn 021637b0 ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 021ab2f8 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021c484c force ret=none args=v,v,v,v  # field effect (ctrl, x, z, y)

# 021637E4(m, info): OBJ data +6 != 0 and 0216DC34(m) == 0: a = info +0xE & 0x40 ? info +4 : (b = 02163F40(m, 1),
# 021AB0F8(b) & 0x40 ? b : -1); a != -1: v = 021AB0F0(a); k = 021AB440(v) == 1 or 021AB358(v) == 1 ? 2 : 1;
# 021BBFB8(0216D67C(m), m, info +0x14, k); 0216DC18(m, 1)
fn 021637e4 ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute
  stub 021ab358 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab440 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021bbfb8 force ret=none args=v,v,v,v  # field effect (sys, m, ctrl, kind)

# 02163868(m, info): OBJ data +6 != 0 and 0216DC34(m): b = 02163F40(m, 1); b == -1 or 021AB0F8(b) & 0x40 == 0: 0216DC18(m, 0)
fn 02163868 ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163f40 ret=pick:0xFFFFFFFF/0/1/0x40 args=v,v  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216dc18 force ret=none args=v,v  # MMDL +4 bit 0x20000 = on
  stub 0216dc34 force ret=pick:0/1 args=v  # MMDL +4 & 0x20000
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f8 force ret=u8 args=v  # attribute flags of an attribute

# 021638AC(m, info): 021AB3EC(info +0xA) == 1: 021C8F7C(m, info +0x14)
fn 021638ac ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021ab3ec force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021c8f7c force ret=none args=v,v  # field effect (m, ctrl)

# 021638C8(m, info): empty
fn 021638c8 ret=none inputs=100
  arg work 0x100
  arg work 0x800
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2

# 021638CC(m, pos, x, y, z, dir) -> hit bits (z: stack s16, dir: stack u16): v = (x << 16 + 0x8000, pos +4, z << 16 + 0x8000);
# 1: 02163C9C(m, x, z) (outside the move limit); 2: 02163D04(m, dir, v) (attribute blocks); height h = 02163E58(m, &v, &h):
# none or |pos.y - h| >= 0x14000: 8, else y = h >> 16; 4: 02163BAC(m, x, y, z) (another MMDL); 0x10: 0218DC04(+0x34 of MMDLSYS, &v)
fn 021638cc ret=u32 inputs=400
  arg work 0x100
  arg work 0xA00
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg s16 alt range 0 4 || any
  arg u16 range 0 3
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any || range 0x13000 0x14FFF
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163a08 ret=none args=v,v,v,outm:8:3/0/3/0/7/0/7/0  # in-area
  stub 02163a40 ret=pick:0/1 args=in:8,in:8  # in-area
  stub 02163af0 ret=pick:0/1 args=v,v,v,v,v,v  # in-area
  stub 02163bac ret=pick:0/1 args=v,v,v,v  # in-area
  stub 02163c9c ret=pick:0/1 args=v,v,v  # in-area
  stub 02163d04 ret=pick:0/1 args=v,v,v,v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163e58 ret=pick:0/1 args=v,in:12,out:4  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d55c force ret=pick:0/1 args=v,v  # MMDL +0 & v
  stub 0216d660 force ret=pick:0xFFFFFFFF/0/1/2 args=v  # MMDL move limit x
  stub 0216d66c force ret=pick:0xFFFFFFFF/0/1/2 args=v  # MMDL move limit z
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e8 force ret=pick:0/1/2/3 args=v  # MMDL initial gx
  stub 0216d7f8 force ret=pick:0/1/2/3 args=v  # MMDL initial gz
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # MMDL size z (u8)
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216dd1c force seq ret=pick:0/1 args=v,outp:work:0x100:2,ptr  # MMDL iterator (sys, &m, &i): MMDL work+0x100 or +0x104
  stub 0218daec force ret=pick:0/1 args=v,in:12,outm:16:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0x1F/0/0x80  # map height / attribute query (fsys, &pos, &out); Z22: height +0xC = 0..0x1FFF or 0x80000000.. (|pos.y - h| meets the 0x14000 threshold)
  stub 0218dc04 force ret=pick:0/1 args=v,in:12  # map bound / hit test (fsys, &pos)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab458 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab474 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab490 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4a4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)

# 02163994(m, x, y, z, dir) (dir: stack u16): 021638CC(m, &position (0216D88C), x, y, z, dir)
fn 02163994 ret=u32 inputs=100
  arg work 0x100
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg u16 range 0 3
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021638cc ret=u8 args=v,in:12,v,v,v,v  # in-area
  stub 02163a08 ret=none args=v,v,v,outm:8:3/0/3/0/7/0/7/0  # in-area
  stub 02163a40 ret=pick:0/1 args=in:8,in:8  # in-area
  stub 02163af0 ret=pick:0/1 args=v,v,v,v,v,v  # in-area
  stub 02163bac ret=pick:0/1 args=v,v,v,v  # in-area
  stub 02163c9c ret=pick:0/1 args=v,v,v  # in-area
  stub 02163d04 ret=pick:0/1 args=v,v,v,v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163e58 ret=pick:0/1 args=v,in:12,out:4  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d55c force ret=pick:0/1 args=v,v  # MMDL +0 & v
  stub 0216d660 force ret=pick:0xFFFFFFFF/0/1/2 args=v  # MMDL move limit x
  stub 0216d66c force ret=pick:0xFFFFFFFF/0/1/2 args=v  # MMDL move limit z
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e8 force ret=pick:0/1/2/3 args=v  # MMDL initial gx
  stub 0216d7f8 force ret=pick:0/1/2/3 args=v  # MMDL initial gz
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # MMDL position VecFx32 -> out
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # MMDL size z (u8)
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216dd1c force seq ret=pick:0/1 args=v,outp:work:0x100:2,ptr  # MMDL iterator (sys, &m, &i): MMDL work+0x100 or +0x104
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 0218dc04 force ret=pick:0/1 args=v,in:12  # map bound / hit test (fsys, &pos)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab458 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab474 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab490 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4a4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)

# 021639C0(m, dir) -> 02163994(m, gx + dx(dir), gy, gz + dz(dir), dir) (s16)
fn 021639c0 ret=u32 inputs=100
  arg work 0x100
  arg u32 range 0 3
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 021638cc ret=u8 args=v,in:12,v,v,v,v  # in-area
  stub 02163994 ret=u8 args=v,v,v,v,v  # in-area
  stub 02163a08 ret=none args=v,v,v,outm:8:3/0/3/0/7/0/7/0  # in-area
  stub 02163a40 ret=pick:0/1 args=in:8,in:8  # in-area
  stub 02163af0 ret=pick:0/1 args=v,v,v,v,v,v  # in-area
  stub 02163bac ret=pick:0/1 args=v,v,v,v  # in-area
  stub 02163c9c ret=pick:0/1 args=v,v,v  # in-area
  stub 02163d04 ret=pick:0/1 args=v,v,v,v,v  # in-area
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02163e58 ret=pick:0/1 args=v,in:12,out:4  # in-area
  stub 021641e8 ret=pick:0/1/0xFFFFFFFF args=v  # in-area
  stub 021641f8 ret=pick:0/1/0xFFFFFFFF args=v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d55c force ret=pick:0/1 args=v,v  # MMDL +0 & v
  stub 0216d660 force ret=pick:0xFFFFFFFF/0/1/2 args=v  # MMDL move limit x
  stub 0216d66c force ret=pick:0xFFFFFFFF/0/1/2 args=v  # MMDL move limit z
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d7e8 force ret=pick:0/1/2/3 args=v  # MMDL initial gx
  stub 0216d7f8 force ret=pick:0/1/2/3 args=v  # MMDL initial gz
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # MMDL position VecFx32 -> out
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # MMDL size z (u8)
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0216dd1c force seq ret=pick:0/1 args=v,outp:work:0x100:2,ptr  # MMDL iterator (sys, &m, &i): MMDL work+0x100 or +0x104
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 0218dc04 force ret=pick:0/1 args=v,in:12  # map bound / hit test (fsys, &pos)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab458 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab474 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab490 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4a4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)

# 02163A08(m, x, z, out): s = max(0216D91C(m) - 1, 0), t = max(0216D924(m) - 1, 0) (s16): out = (x, z - t, x + s, z) (s16)
fn 02163a08 ret=none inputs=100
  arg work 0x100
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg work 0xA40
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # MMDL size z (u8)

# 02163A40(r, s) -> bool: the s16 rectangles (x0, z0, x1, z1) overlap (each axis: one end of the narrower span inside the wider)
fn 02163a40 ret=u32 inputs=100
  arg work 0xA40
  arg work 0xA48
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2

# 02163AF0(m, o, x, y, z, all) -> bool (z: stack s16, all: stack): o +0 & 0x80 (0216D55C) == 0 and the footprint of m at (x, z)
# overlaps o at its grid (02163A08 / 02163A40) with |y - o gy| < 1; all == 1: likewise against o's old grid
fn 02163af0 ret=u32 inputs=100
  arg work 0x100
  arg work 0x200
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg s16 alt range 0 4 || any
  arg u32 alt oneof 0 1 || any
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163a08 ret=none args=v,v,v,outm:8:3/0/3/0/7/0/7/0  # in-area
  stub 02163a40 ret=pick:0/1 args=in:8,in:8  # in-area
  stub 0216d55c force ret=pick:0/1 args=v,v  # MMDL +0 & v
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # MMDL size z (u8)

# 02163BAC(m, x, y, z) -> bool: any other MMDL in use (0216DD1C over 0216D67C(m)) with 02163AF0(m, o, x, y, z, 1) == 1
fn 02163bac ret=u32 inputs=100
  arg work 0x100
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163a08 ret=none args=v,v,v,outm:8:3/0/3/0/7/0/7/0  # in-area
  stub 02163a40 ret=pick:0/1 args=in:8,in:8  # in-area
  stub 02163af0 ret=pick:0/1 args=v,v,v,v,v,v  # in-area
  stub 0216d55c force ret=pick:0/1 args=v,v  # MMDL +0 & v
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # MMDL size z (u8)
  stub 0216dd1c force seq ret=pick:0/1 args=v,outp:work:0x100:2,ptr  # MMDL iterator (sys, &m, &i): MMDL work+0x100 or +0x104

# 02163C04(m, x, z, old) -> bool: (x, z) inside the footprint at the grid, or (old == 1) at the old grid
fn 02163c04 ret=u32 inputs=300
  arg work 0x100
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg u32 alt oneof 0 1 || any
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163a08 ret=none args=v,v,v,outm:8:3/0/3/0/7/0/7/0  # in-area
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # MMDL size z (u8)

# 02163C9C(m, x, z) -> bool: limit 0216D660 != -1 and x outside initial gx +- limit, or likewise z (0216D7F8 / 0216D66C)
fn 02163c9c ret=u32 inputs=100
  arg work 0x100
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d660 force ret=pick:0xFFFFFFFF/0/1/2 args=v  # MMDL move limit x
  stub 0216d66c force ret=pick:0xFFFFFFFF/0/1/2 args=v  # MMDL move limit z
  stub 0216d7e8 force ret=pick:0/1/2/3 args=v  # MMDL initial gx
  stub 0216d7f8 force ret=pick:0/1/2/3 args=v  # MMDL initial gz

# 02163D04(m, dir, v) -> bool (v VecFx32 by value: r2, r3, stack): 0216DCB4(m): 1; for each footprint cell (0216D91C x 0216D924,
# x += 0x10000, z -= 0x10000): no attribute (02163E34 == 0): 1; 021AB11C(a) == 1: 1; first a kept; then ROM 0x021714A0[dir](021AB0F0(
# 0216D7DC(m))) == 1: 1; ROM 0x02171490[dir](021AB0F0(first a)) == 1: 1; 0. The first a is uninitialised when the footprint is empty
fn 02163d04 ret=u32 inputs=100
  arg work 0x100
  arg u32 range 0 3
  arg u32 alt range 0 0x4FFFF || any
  arg u32 any
  arg u32 alt range 0 0x4FFFF || any
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7dc force ret=pick:0xFFFFFFFF/0/1/0x40 args=v  # MMDL +0x74 (attribute)
  stub 0216d91c force ret=pick:0/1/2/3 args=v  # MMDL size x (u8)
  stub 0216d924 force ret=pick:0/1/2/3 args=v  # MMDL size z (u8)
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)
  stub 021ab0f0 force ret=u32 args=v  # attribute value of an attribute
  stub 021ab11c force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab458 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab474 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab490 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)
  stub 021ab4a4 force ret=pick:0/1 args=v  # map attribute predicate (attribute value)

# 02163E1C(m, pos, out) -> 0218DAEC(0216D52C(0216D67C(m)), pos, out) (out: 16 bytes: +8 attribute, +0xC height)
fn 02163e1c ret=u32 inputs=100
  arg work 0x100
  arg work 0xA00
  arg work 0xA60
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)

# 02163E34(m, pos, out) -> bool: *out = 0; 02163E1C(m, pos, &b) == 1: *out = b +8 (attribute), 1; else 0
fn 02163e34 ret=u32 inputs=100
  arg work 0x100
  arg work 0xA00
  arg work 0xA60
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)

# 02163E58(m, pos, out) -> bool: as 02163E34 with b +0xC (height)
fn 02163e58 ret=u32 inputs=100
  arg work 0x100
  arg work 0xA00
  arg work 0xA60
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)

# 02163E7C(m, dir): old grid = grid; gx += dx(dir), gz += dz(dir); h = 02163E58(m, (gx, gz) centre with y = pos y, &h)
# (h = 0 if none); gy = h >> 16 (toward zero, s16)
fn 02163e7c ret=none inputs=100
  arg work 0x100
  arg u32 alt range 0 3 || range 0 15
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e58 ret=pick:0/1 args=v,in:12,out:4  # in-area
  stub 021641e8 ret=pick:0/1/0xFFFFFFFF args=v  # in-area
  stub 021641f8 ret=pick:0/1/0xFFFFFFFF args=v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d80c force ret=none args=v,v  # MMDL old gx = v
  stub 0216d818 force ret=none args=v,v  # MMDL old gy = v
  stub 0216d824 force ret=none args=v,v  # MMDL old gz = v
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d848 force ret=none args=v,v  # MMDL gx += d
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gy
  stub 0216d85c force ret=none args=v,v  # MMDL gy = v
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d87c force ret=none args=v,v  # MMDL gz += d
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)

# 02163F10(m): old grid (+0x36..) = grid (+0x3C..)
fn 02163f10 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d80c force ret=none args=v,v  # MMDL old gx = v
  stub 0216d818 force ret=none args=v,v  # MMDL old gy = v
  stub 0216d824 force ret=none args=v,v  # MMDL old gz = v
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz

# 02163F40(m, dir) -> attribute of the cell in front: v = grid centre (02164280) with y = pos y; 02164208(dir, &v, 0x10000);
# a = -1; 02163E34(m, &v, &a); a
fn 02163f40 ret=u32 inputs=100
  arg work 0x100
  arg u32 alt range 0 3 || any
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02164208 ret=none args=v,v,v  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)

# 02163F8C(m, d): position (0216D88C) += *d (VecFx32); 0216D89C
fn 02163f8c ret=none inputs=100
  arg work 0x100
  arg work 0xA00
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d88c force ret=none args=v,out:12  # MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # MMDL position VecFx32 = *v

# 02163FC0(m, dir, n): position: dir 0: z -= n, 1: z += n, 2: x -= n, 3: x += n; 0216D89C
fn 02163fc0 ret=none inputs=100
  arg work 0x100
  arg u32 alt range 0 3 || any
  arg u32 any
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d88c force ret=none args=v,out:12  # MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # MMDL position VecFx32 = *v

# 02164010(m) -> bool: status 0x2000 (rail): 0; 0216DB68(m) == 1: clear move bit 0x1000, 0; h = 02163E58(m, &position, &h):
# found: position y = h (0216D89C), old gy = gy, gy = h >> 16 (toward zero), clear 0x1000; else set 0x1000; found
fn 02164010 ret=u32 inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e58 ret=pick:0/1 args=v,in:12,out:4  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d55c force ret=pick:0/1 args=v,v  # MMDL +0 & v
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d818 force ret=none args=v,v  # MMDL old gy = v
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gy
  stub 0216d85c force ret=none args=v,v  # MMDL gy = v
  stub 0216d88c force ret=none args=v,out:12  # MMDL position VecFx32 -> out
  stub 0216d89c force ret=none args=v,in:12  # MMDL position VecFx32 = *v
  stub 0216db68 force ret=pick:0/1 args=v  # MMDL +0 & 0x200
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)

# 021640B0(m) -> bool: 0216DCB4(m) == 0: a = attribute at the old grid (02163E34), b = at the grid with y = pos y; a found:
# 0216D7E0(m, a); b found: 0216D7D8(m, b), clear move bit 0x800, 1; else set 0x800, 0
fn 021640b0 ret=u32 inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 02163e1c ret=pick:0/1 args=v,in:12,out:16  # in-area
  stub 02163e34 ret=pick:0/1 args=v,in:12,outm:4:0xFF/0xFF/0xFF/0xFF  # in-area
  stub 02164280 ret=none args=v,v,v  # in-area
  stub 0216d52c force ret=u32 args=v  # MMDLSYS +0x34
  stub 0216d568 force ret=none args=v,v  # MMDL +4 |= v
  stub 0216d570 force ret=none args=v,v  # MMDL +4 &= ~v
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d7d8 force ret=none args=v,v  # MMDL +0x74 = v
  stub 0216d7e0 force ret=none args=v,v  # MMDL +0x78 = v
  stub 0216d804 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gx
  stub 0216d810 force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gy
  stub 0216d81c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL old gz
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216d8ac force ret=pick:0/0x10000/0xFFFF0000/0x30000/0x8000 args=v  # MMDL position y (fx32)
  stub 0216dcb4 force ret=pick:0/1 args=v  # MMDL +0 & 0x400
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)

# 02164168(m) -> bool: player p = 0216DEE0(0216D67C(m)); p and |p gz - gz| / |p gx - gx| within -0x12..+0x11
fn 02164168 ret=u32 inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d83c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0xFFFFFFFF args=v  # MMDL gz
  stub 0216dee0 force ret=nullable:4:ptr:work:0x200:1 args=v  # MMDLSYS player MMDL

# 021641C8(m): move code - 0x33 (u16) < 4: 0216D708(m)
fn 021641c8 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d5a0 force ret=pick:0x30/0x32/0x33/0x36/0x37/0/1 args=v  # MMDL move code
  stub 0216d708 force ret=none args=v  # MMDL move proc

# 021641E8(dir) -> ROM 0x02171480[dir] (s16: 0, 0, -1, 1 = dx)
fn 021641e8 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 3 || range 0 15

# 021641F8(dir) -> ROM 0x02171470[dir] (s16: -1, 1, 0, 0 = dz)
fn 021641f8 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 3 || range 0 15

# 02164208(dir, v, n): dir 0: v.z -= n, 1: v.z += n, 2: v.x -= n, 3: v.x += n (VecFx32)
fn 02164208 ret=none inputs=100
  arg u32 alt range 0 3 || any
  arg work 0xA00
  arg u32 any
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2

# 02164240(dir, x, z, n) (s16 pointers): dir 0: *z -= n, 1: *z += n, 2: *x -= n, 3: *x += n
fn 02164240 ret=none inputs=100
  nowork
  arg u32 alt range 0 3 || any
  arg buf 4
  arg buf 4
  arg u32 any

# 02164280(gx, gz, v): v.x = gx << 16 + 0x8000, v.z = gz << 16 + 0x8000 (v.y untouched)
fn 02164280 ret=none inputs=100
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg work 0xA00
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2

# 02164294(x, y, z, v): v = (x, y, z) << 16
fn 02164294 ret=none inputs=100
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg work 0xA00
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2

# 021642A4(dir) -> ROM 0x021714B0[dir] (u16: 1, 0, 3, 2 = the opposite direction)
fn 021642a4 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 3 || range 0 15

# 021642B4(x0, z0, x1, z1) -> direction from (x0, z0) to (x1, z1): x0 > x1: 2; x0 < x1: 3; z0 > z1: 0; else 1
fn 021642b4 ret=u32 inputs=100
  nowork
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any
  arg s32 alt range 0 4 || any

# 021642D4(dir, angle) -> ROM 0x021714C1[4 ROM 0x02171460[((angle + ROM 0x02171458[dir]) & 0xFFFF) >> 12]] (u8: the
# direction of a rotated angle)
fn 021642d4 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 3
  arg u32 any

# 021642F8(m): tail 0216D548(m, 0x2000) (status |= rail)
fn 021642f8 ret=none inputs=100
  arg work 0x100
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d548 force ret=none args=v,v  # MMDL +0 |= v

# 02164304(m, hdr): RAIL LOCATION r = (hdr +0x1C (line), 1, 0201441C(hdr +0xC (s16 dir)), hdr +0x20 (s16 side), hdr +0x1E
# (front)); MMDLSYS flag 0x10 (0216D4D4): 0219E2C0(m, &r); else 0216D684(m, 0xC); 0216435C(m, &r)
fn 02164304 ret=none inputs=100
  arg work 0x100
  arg work 0xA80
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0201441c force ret=u8 args=v  # dir -> rail key
  stub 0216435c ret=none args=v,in:8  # in-area
  stub 0216d4d4 force ret=pick:0/1 args=v,v  # MMDLSYS +0 & mask
  stub 0216d67c force ret=ptr:work:0:1 args=v  # MMDL -> MMDLSYS
  stub 0216d684 force ret=u32 args=v,v  # MMDL move work clear (m, n)
  stub 0216d69c force ret=ptr:work:0x194:1 args=v  # MMDL move work (m + 0x94)
  stub 0219e2c0 force ret=none args=v,in:8  # rail: set MMDL rail location (m, &loc)

# 0216435C(m, r): w = 0216D69C(m) (move work): w +4..+0xA = r (4 x u16); w +0 = 0xEFEFEFEF (marker)
fn 0216435c ret=none inputs=100
  arg work 0x100
  arg work 0xA20
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  stub 0216d69c force ret=ptr:work:0x194:1 args=v  # MMDL move work (m + 0x94)

# 02164380(m, out) -> bool: w = 0216D69C(m); w +0 == 0xEFEFEFEF: out = w +4..+0xA, 1; else 0
fn 02164380 ret=u32 inputs=100
  arg work 0x100
  arg work 0xA20
  fix work 0x800 u32 alt const 0xFFFFFFFF || any
  fix work 0x804 u32 alt const 0xFFFFFFFF || any
  fix work 0x810 u32 addrof work 0x900
  fix work 0x818 u8 alt range 0 3 || any
  fix work 0x904 u8 alt const 0 || any
  fix work 0x905 u8 alt const 2 || any
  fix work 0x906 u8 alt const 0 || any
  fix work 0xA00 u32 alt range 0 0x4FFFF || any
  fix work 0xA04 u32 alt range 0 0x4FFFF || any
  fix work 0xA08 u32 alt range 0 0x4FFFF || any
  fix work 0xA40 u16 alt range 0 6 || any rep=8,2
  fix work 0x194 u32 alt const 0xEFEFEFEF || any
  stub 0216d69c force ret=ptr:work:0x194:1 args=v  # MMDL move work (m + 0x94)

