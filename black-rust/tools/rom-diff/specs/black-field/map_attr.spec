# fndiff spec: the map attribute accessors and predicates of overlay 21 (`map_attr`, Thumb code 0x021AB0D4..0x021AB5F4, the tail of F1
# row 21.30; F5a-spec): MAPATTR = u32 word (low half = attribute value, high half = flags: bit 0 blocked, bit 1 water, bit 2 wild
# encounters), the value predicates (grass, long grass, water, sand, ice, ...), the attribute -> battle ground kind table and the
# effect-encounter kind of an attribute (021AB504). No __FILE__ string (nothing allocates); the range is cut off effect_encount.c at
# the first function that does not touch the EFFECT ENCOUNT work (021AB0D4, a wrapper of the map attribute query 0218DAEC); fld_scenearea.c
# (string 0x021DDFC8) starts at 0x021AB5F4. Callers: field_encount, effect_encount, eventdata (mmdl_move), field_player, scripts.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field map_attr --per-function
# Set `field` (main 10 11 18 20 21).
# Function list: bl/blx targets of the whole field set, odd code pointers (the direction predicate tables ROM 0x021714A0 / 0x02171490,
# 0x021D3CE8.., 0x021D8A2C..), the Ghidra index and a Thumb walk from every entry: 69 entries, minus the false entry 021AB394 (inside
# 021AB390: a data word in main 0x020A3EE6 decodes as a bl to it) = 68 specced. Ghidra misses 021AB1A0, 021AB1AC, 021AB2F8, 021AB358,
# 021AB428, 021AB458 and 021AB3F8 (`v == 0x32`, which nothing calls); 021AB504's jump table (values 0..10) is decoded.
# Render-only functions: none.
# Closure stubs (as core4 / F4a / F6a): a block lists the stubs of its callees and of the callees of its in-area descendants, so the area
# can be ported in any order. Out-of-area stubs are `force`. All blocks are `nowork`; values are drawn from the compared constants and
# their neighbours (or any word: the predicates compare the full 32-bit register).
area map_attr
set field

# 021AB0D4(fsys, pos) -> attribute word: 0218DAEC(fsys, pos, &out) == 1: out +8; else 0xFFFFFFFF
fn 021ab0d4 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x10
  stub 0218daec force ret=pick:0/1 args=v,in:12,out:16  # map height / attribute query (fsys, &pos, &out)

# 021AB0F0(a) -> a & 0xFFFF (attribute value)
fn 021ab0f0 ret=u32 inputs=100
  nowork
  arg u32 any

# 021AB0F8(a) -> a >> 16 (attribute flags)
fn 021ab0f8 ret=u32 inputs=100
  nowork
  arg u32 any

# 021AB100(a) -> valid: a != 0xFFFFFFFF and 021AB0F0(a) != 0xFF
fn 021ab100 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xFFFFFFFF 0xFF 0x400FF 0 4 0x40004 || any
  stub 021ab0f0 ret=u16 args=v  # in-area

# 021AB11C(a) -> blocked: 021AB100(a) == 0 or flags (021AB0F8) bit 0
fn 021ab11c ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xFFFFFFFF 0xFF 0x10000 0x10004 0 4 0x40004 0x30004 || any
  stub 021ab0f0 ret=u16 args=v  # in-area
  stub 021ab0f8 ret=pick:0/1/2/4/6/5/7 args=v  # in-area
  stub 021ab100 ret=pick:0/1 args=v  # in-area

# 021AB140(v) -> v in {0x8, 0x9}
fn 021ab140 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x7 0x8 0x9 0xA 0x10008 0x10009 || any

# 021AB158(v) -> v == 0x74
fn 021ab158 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x73 0x74 0x75 0x10074 || any

# 021AB164(v) -> v == 0x75
fn 021ab164 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x74 0x75 0x76 0x10075 || any

# 021AB170(v) -> v == 0x73
fn 021ab170 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x72 0x73 0x74 0x10073 || any

# 021AB17C(v) -> v == 0x72
fn 021ab17c ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x71 0x72 0x73 0x10072 || any

# 021AB188(v) -> v == 0x7C
fn 021ab188 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x7B 0x7C 0x7D 0x1007C || any

# 021AB194(v) -> v == 0xD4
fn 021ab194 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xD3 0xD4 0xD5 0x100D4 || any

# 021AB1A0(v) -> v == 0xD6
fn 021ab1a0 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xD5 0xD6 0xD7 0x100D6 || any

# 021AB1AC(v) -> v == 0xD7
fn 021ab1ac ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xD6 0xD7 0xD8 0x100D7 || any

# 021AB1B8(v) -> v == 0xD8
fn 021ab1b8 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xD7 0xD8 0xD9 0x100D8 || any

# 021AB1C4(v) -> v == 0xE2
fn 021ab1c4 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xE1 0xE2 0xE3 0x100E2 || any

# 021AB1D0(v) -> v == 0xD9
fn 021ab1d0 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xD8 0xD9 0xDA 0x100D9 || any

# 021AB1DC(v) -> v == 0xDA
fn 021ab1dc ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xD9 0xDA 0xDB 0x100DA || any

# 021AB1E8(v) -> v == 0xDB
fn 021ab1e8 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xDA 0xDB 0xDC 0x100DB || any

# 021AB1F4(v) -> v == 0xDC
fn 021ab1f4 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xDB 0xDC 0xDD 0x100DC || any

# 021AB200(v) -> v == 0xDD
fn 021ab200 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xDC 0xDD 0xDE 0x100DD || any

# 021AB20C(v) -> v == 0xDE
fn 021ab20c ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xDD 0xDE 0xDF 0x100DE || any

# 021AB218(v) -> v == 0xDF
fn 021ab218 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xDE 0xDF 0xE0 0x100DF || any

# 021AB224(v) -> v == 0xE0
fn 021ab224 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xDF 0xE0 0xE1 0x100E0 || any

# 021AB230(v) -> v == 0xE1
fn 021ab230 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xE0 0xE1 0xE2 0x100E1 || any

# 021AB23C(v) -> v in {0x4, 0x21, 0x5}
fn 021ab23c ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x3 0x4 0x5 0x6 0x20 0x21 0x22 0x10004 0x10005 0x10021 || any

# 021AB250(v) -> v in {0x6, 0x22, 0x7}
fn 021ab250 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x5 0x6 0x7 0x8 0x21 0x22 0x23 0x10006 0x10007 0x10022 || any

# 021AB264(v) -> v == 0x8
fn 021ab264 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x7 0x8 0x9 0x10008 || any

# 021AB270(v) -> v == 0x9
fn 021ab270 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x8 0x9 0xA 0x10009 || any

# 021AB27C(v) -> 021AB23C(v) (4 / 0x21 / 5) or 021AB264(v) (8)
fn 021ab27c ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x3 0x4 0x5 0x6 0x7 0x8 0x9 0x20 0x21 0x22 0x10004 0x10005 0x10008 0x10021 || any
  stub 021ab23c ret=pick:0/1 args=v  # in-area
  stub 021ab264 ret=pick:0/1 args=v  # in-area

# 021AB29C(v) -> long grass: 021AB250(v) (6 / 0x22 / 7) or 021AB270(v) (9)
fn 021ab29c ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x5 0x6 0x7 0x8 0x9 0xA 0x21 0x22 0x23 0x10006 0x10007 0x10009 0x10022 || any
  stub 021ab250 ret=pick:0/1 args=v  # in-area
  stub 021ab270 ret=pick:0/1 args=v  # in-area

# 021AB2BC(v) -> grass: 021AB27C(v) or 021AB29C(v)
fn 021ab2bc ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x3 0x4 0x5 0x6 0x7 0x8 0x9 0xA 0x20 0x21 0x22 0x23 0x10004 0x10005 0x10006 0x10007 0x10008 0x10009 0x10021 0x10022 || any
  stub 021ab23c ret=pick:0/1 args=v  # in-area
  stub 021ab250 ret=pick:0/1 args=v  # in-area
  stub 021ab264 ret=pick:0/1 args=v  # in-area
  stub 021ab270 ret=pick:0/1 args=v  # in-area
  stub 021ab27c ret=pick:0/1 args=v  # in-area
  stub 021ab29c ret=pick:0/1 args=v  # in-area

# 021AB2DC(v) -> v == 0x1D
fn 021ab2dc ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x1C 0x1D 0x1E 0x1001D || any

# 021AB2E8(v) -> v in {0x41, 0x44}
fn 021ab2e8 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x40 0x41 0x42 0x43 0x44 0x45 0x10041 0x10044 || any

# 021AB2F8(v) -> v == 0x14
fn 021ab2f8 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x13 0x14 0x15 0x10014 || any

# 021AB304(v) -> v == 0x17
fn 021ab304 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x16 0x17 0x18 0x10017 || any

# 021AB310(v) -> v == 0x40
fn 021ab310 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x3F 0x40 0x41 0x10040 || any

# 021AB31C(v) -> v == 0xE
fn 021ab31c ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xD 0xE 0xF 0x1000E || any

# 021AB328(v) -> v == 0xF
fn 021ab328 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xE 0xF 0x10 0x1000F || any

# 021AB334(v) -> 021AB31C(v) != 0 (0xE)
fn 021ab334 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xD 0xE 0xF 0x1000E || any
  stub 021ab31c ret=pick:0/1 args=v  # in-area

# 021AB348(v) -> v in {0x18, 0x10}
fn 021ab348 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xF 0x10 0x11 0x17 0x18 0x19 0x10010 0x10018 || any

# 021AB358(v) -> v == 0x10
fn 021ab358 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xF 0x10 0x11 0x10010 || any

# 021AB364(v) -> v == 0x1C
fn 021ab364 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x1B 0x1C 0x1D 0x1001C || any

# 021AB370(v) -> v == 0x19
fn 021ab370 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x18 0x19 0x1A 0x10019 || any

# 021AB37C(v) -> v in {0xC, 0xB, 0x23}
fn 021ab37c ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xA 0xB 0xC 0xD 0x22 0x23 0x24 0x1000B 0x1000C 0x10023 || any

# 021AB390(v) -> 021AB370(v) (0x19) or 021AB37C(v) (0xC / 0xB / 0x23) 
fn 021ab390 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xA 0xB 0xC 0xD 0x18 0x19 0x1A 0x22 0x23 0x24 0x1000B 0x1000C 0x10019 0x10023 || any
  stub 021ab370 ret=pick:0/1 args=v  # in-area
  stub 021ab37c ret=pick:0/1 args=v  # in-area

# 021AB3B0(v) -> v in {0x3D, 0x3E, 0x3F, 0x42, 0x43}
fn 021ab3b0 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x3C 0x3D 0x3E 0x3F 0x40 0x41 0x42 0x43 0x44 0x1003D 0x1003E 0x1003F 0x10042 0x10043 || any

# 021AB3D4(v) -> v == 0x2
fn 021ab3d4 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x1 0x2 0x3 0x10002 || any

# 021AB3E0(v) -> v == 0x1E
fn 021ab3e0 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x1D 0x1E 0x1F 0x1001E || any

# 021AB3EC(v) -> v == 0x30
fn 021ab3ec ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x2F 0x30 0x31 0x10030 || any

# 021AB3F8(v) -> v == 0x32
fn 021ab3f8 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x31 0x32 0x33 0x10032 || any

# 021AB404(v) -> v == 0xBE
fn 021ab404 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xBD 0xBE 0xBF 0x100BE || any

# 021AB410(v) -> v == 0xBF
fn 021ab410 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xBE 0xBF 0xC0 0x100BF || any

# 021AB41C(v) -> v == 0xC
fn 021ab41c ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0xB 0xC 0xD 0x1000C || any

# 021AB428(v) -> v == 0x5
fn 021ab428 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x4 0x5 0x6 0x10005 || any

# 021AB434(v) -> v == 0x7
fn 021ab434 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x6 0x7 0x8 0x10007 || any

# 021AB440(v) -> v == 0x1B
fn 021ab440 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x1A 0x1B 0x1C 0x1001B || any

# 021AB44C(v) -> v == 0x33
fn 021ab44c ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x32 0x33 0x34 0x10033 || any

# 021AB458(v) -> v in {0x53, 0x55, 0x56}
fn 021ab458 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x52 0x53 0x54 0x55 0x56 0x57 0x10053 0x10055 0x10056 || any

# 021AB474(v) -> v in {0x54, 0x57, 0x58}
fn 021ab474 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x53 0x54 0x55 0x56 0x57 0x58 0x59 0x10054 0x10057 0x10058 || any

# 021AB490(v) -> v in {0x52, 0x56, 0x58}
fn 021ab490 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x51 0x52 0x53 0x55 0x56 0x57 0x58 0x59 0x10052 0x10056 0x10058 || any

# 021AB4A4(v) -> v in {0x51, 0x55, 0x57}
fn 021ab4a4 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x50 0x51 0x52 0x54 0x55 0x56 0x57 0x58 0x10051 0x10055 0x10057 || any

# 021AB4B8(v) -> 021AB140(v) (8 / 9) or 021AB404 (0xBE) or 021AB410 (0xBF) or 021AB328 (0xF)
fn 021ab4b8 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x7 0x8 0x9 0xA 0xE 0xF 0x10 0xBD 0xBE 0xBF 0xC0 0x10008 0x10009 0x1000F 0x100BE 0x100BF || any
  stub 021ab140 ret=pick:0/1 args=v  # in-area
  stub 021ab328 ret=pick:0/1 args=v  # in-area
  stub 021ab404 ret=pick:0/1 args=v  # in-area
  stub 021ab410 ret=pick:0/1 args=v  # in-area

# 021AB4EC(v) -> v == 0x21
fn 021ab4ec ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x20 0x21 0x22 0x10021 || any

# 021AB4F8(v) -> v == 0x22
fn 021ab4f8 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x21 0x22 0x23 0x10022 || any

# 021AB504(a) -> effect encounter kind of the attribute word: value 0x20: 7; flags bit 2 clear: 8; value 4: 0, 5: 1, 8: 3, 10: 4 (the
# case of the jump table that returns the 4 already in r0), 0x21: 2, 0x3D: 5, 0x3F / 0x43: 6, any other: 8
fn 021ab504 ret=u32 inputs=200
  nowork
  arg u32 alt oneof 0x20 0x40020 0x40004 0x40005 0x40008 0x4000A 0x40021 0x4003D 0x4003F 0x40043 0x40000 0x40009 0x4003E 0x40042 0x40040 0x4000B 0x40001 4 0x60004 || any

# 021AB580(v) -> ground kind: ROM 0x021D8EA0 (37 pairs u8 value, u8 kind): kind of the first pair with that value (byte compare
# of the full register), else 0
fn 021ab580 ret=u32 inputs=200
  nowork
  arg u32 alt oneof 0 4 2 3 1 5 6 7 8 9 0xA 0xB 0xC 0xE 0xF 0x10 0x11 0x13 0x14 0x16 0x17 0x18 0x19 0x1B 0x1C 0x1E 0x1F 0x20 0x21 0x22 0x23 0x30 0x3D 0x3F 0x43 0x7C 0xA1 0xBE 0x12 0x15 0xFF 0x100 || any

# 021AB5AC(a, b) -> enters water: 021AB11C(a) (blocked): 0; b flags bit 1 (water) and 021AB11C(b) == 0 and 021AB2E8(b value)
# (0x41 / 0x44)
fn 021ab5ac ret=u32 inputs=200
  nowork
  arg u32 alt oneof 0xFFFFFFFF 0x10000 0 4 0x40004 || any
  arg u32 alt oneof 0x20041 0x20044 0x30041 0x20000 0x41 0x60044 || any
  stub 021ab0f0 ret=u16 args=v  # in-area
  stub 021ab0f8 ret=pick:0/1/2/4/6/5/7 args=v  # in-area
  stub 021ab100 ret=pick:0/1 args=v  # in-area
  stub 021ab11c ret=pick:0/1 args=v  # in-area
  stub 021ab2e8 ret=pick:0/1 args=v  # in-area

