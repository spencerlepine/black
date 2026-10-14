# fndiff spec: effect_encount.c (overlay 21 Thumb code 0x021AA77C..0x021AB0D4, the logic part of F1 row 21.30; F5a-spec): the
# effect encounters (shaking grass / dust clouds / rippling water / bird shadows): the step trigger, the candidate search around the
# player, the placement, the check when the player steps on it, the items of dust clouds (021AB018) and bird shadows (021AB070), the
# effect object hooks. The draw side (the shaking grass / dust / ripple effect objects) is fldeff_encount.c (F1 row 21.38, U2), reached
# only through the force stubs 021ADAAC / 021ADB4C / 021ADB5C and the field effect delete 021AC154 (D3).
# File bounds: `Heap_AllocDebug(heap, 0x5C0, 1, "effect_encount.c", 0x6E)` in 021AA77C (string 0x021DDFB4); the rest of F1 row 21.30
# (0x021AB0D4..0x021AB5F2, the map attribute predicates, no __FILE__ string) is the area map_attr: from 021AB0D4 on nothing refers to the
# EFFECT ENCOUNT work any more and every function is a MAPATTR accessor (attribute word -> value / flags, value predicates).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field effect_encount --per-function
# Set `field` (main 10 11 18 20 21).
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry; 30 entries,
# minus the veneer 021AA8F8 (`ldr r3, =021AAF55; bx r3`; callers stub 021AAF54) = 29 specced (021AA8EC is not a veneer: it loads enc
# +0x10 into r1 before its tail jump to 021AAFA0). Render-only functions: none in this range.
# Closure stubs (as core4 / F4a / F6a): a block lists the stubs of its callees and of the callees of its in-area descendants, so the area
# can be ported in any order (checked with every function treated as ported: no fault, stray write or unstubbed callee). Every
# out-of-area stub is `force` (also field_encount 021A9F40 and map_attr 021AB504).
#
# Trigger (021AA80C(enc), each step): table loaded; map matrix present and the player's block (02159C48 / 02159BD4 at the player's
# position) is in the current zone; 0200C234(02012F08(gd), 0) (effect encounters enabled) and 02188CFC(fieldmap) == 0; no effect
# active (ENCOUNT WORK +0x1D low nibble 0); the effect step counter (+0xC) reaches the interval (021AABC4); then 02005728(1000) <
# 10 * probability: search (021AACD4) and place one candidate at random (021AAE34); else the counter restarts (021AABEC). Interval /
# probability: ROM 0x021D8E70 by the table's effect kind (byte 7 & 0x7F, > 2 -> 0): kind 0 / 1: 20 steps, 10 %; kind 2: 5 steps, 15 %.
# Search window (021AABF4): the player's tile within its 32 x 32 block, offsets ROM 0x021D8E76 (kind 2: x / z -5..+5; else one of four
# half windows, 02005728(4) + 1), clamped to the block; candidates: every tile whose attribute (0218DBAC) has a kind (map_attr 021AB504:
# 0 grass / 1 long grass / 2 / 3 / 4 water / 5 / 6 cave dust / 7, 8 = none). Stepping on it (021AA944, kind 1 encounter): kind 7 (bird
# shadow) with 02005728(1000) >= 200, kind 4 (dust) with >= 400: an item (021AB070 feathers 565..571, 021AB018 evolution stones /
# gems 548..564 / Everstone 229) given by the script 0x2795 (021AB0AC); else field_encount 021A9F40(enc, 1).
#
# EFFECT ENCOUNT (0x5C0, 021AA77C): `+0` field effect control (02188CD0), `+4 u16` candidate count, `+8` CANDIDATE[121] (0xC: `+0 u8`
# kind, `+2 s16` x, `+4 s16` y (tiles, from the height), `+6 s16` z, `+8 s32` height fx32), `+0x5B4` the effect object, `+0x5B8 u16`
# interval, `+0x5BA u16` probability (per mille / 10), `+0x5BC u16` item of the last dust / shadow. ENCOUNT WORK (0x20, `02012A38(gd)`)
# `+0xC u32` effect step counter (saturating at 0xFFFFFFFF), `+0x10` STATE: `+0x10 u16` zone, `+0x12 / +0x14 / +0x16 s16` x / y / z,
# `+0x18 s32` height, `+0x1C u8` kind, `+0x1D u8` bits 0..3 active, 4..5 needs the effect object again (021AAB54 after a map change),
# 6..7 stepped on (0x40).
# Arena: the FIELD ENCOUNT object is `arg buf 0x800` (as encount.spec): object +0, ENCOUNT TABLE +0x20, EFFECT ENCOUNT +0x200 (enc
# +0x10 fixed to it), ENCOUNT WORK +0x7C0 (the `02012A38` answer). The blocks that iterate MMDLs (021AAEC0 and its callers) also use
# the `work` image (MMDL pointers of the `0216DD1C` iterator).
area effect_encount
set field

# 021AA77C(heap) -> Heap_AllocDebug(heap, 0x5C0, 1, "effect_encount.c", 0x6E)
fn 021aa77c ret=u32 inputs=100
  nowork
  arg u32 any
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)

# 021AA794(eff): MI_CpuFill8(eff, 0, 0x5C0); Heap_Free(eff)
fn 021aa794 ret=none inputs=100
  nowork
  arg buf 0x5C0
  stub 020307b0 force ret=none args=v  # Heap_Free(block)

# 021AA7AC(enc, eff): eff +0 = 02188CD0(enc fieldmap); 021AA7DC(enc, eff); 021AAB74(enc, eff)
fn 021aa7ac ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 addrof arg0 0x200
  fix arg0 0xC u32 addrof arg0 0x20
  fix arg0 0x10 u32 addrof arg0 0x200
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 02188cd0 force ret=u32 args=v  # fieldmap -> field effect control
  stub 021aa7dc ret=none args=v,v  # in-area
  stub 021aab74 ret=none args=v,v  # in-area
  stub 021aafc0 ret=none args=v,v,v  # in-area
  stub 021adaac force ret=pick:0/1/0x02EE0000 args=v,v,v,v,v,v  # fldeff_encount create (eff, ctrl, x, z, y, kind)

# 021AA7CC(enc, eff): 021AAB54(enc); 021AAFFC(eff)
fn 021aa7cc ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 addrof arg0 0x200
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 021aab54 ret=none args=v  # in-area
  stub 021aaffc ret=none args=v  # in-area
  stub 021ac154 force ret=none args=v  # field effect delete (obj)

# 021AA7DC(enc, eff): k = table byte 7 & 0x7F (> 2 -> 0); eff +0x5B8 = ROM 0x021D8E70[2k] (interval), +0x5BA = [2k + 1]
# (probability)
fn 021aa7dc ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 addrof arg0 0x200
  fix arg0 0xC u32 addrof arg0 0x20
  fix arg0 0x27 u8 alt range 0x80 0x83 || range 0 3 || any

# 021AA80C(enc): the step trigger (header)
fn 021aa80c ret=none inputs=600
  arg buf 0x800
  fix arg0 0xC u32 alt addrof arg0 0x20 || addrof arg0 0x20 || const 0
  fix arg0 0x27 u8 alt oneof 0x80 0x81 0x82 || oneof 0x80 0x81 || any
  fix arg0 0x10 u32 addrof arg0 0x200
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  fix arg0 0x204 u16 range 0 3
  fix arg0 0x7BA u16 alt range 50 100 || any
  fix arg0 0x7CC u32 alt range 20 40 || any
  fix arg0 0x7B8 u16 alt range 0 30 || any
  fix arg0 0x7DD u8 alt const 0 || const 0 || oneof 1 0x11 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 02005728 force ret=lt:0 args=v  # RNG next_scaled(max): below max
  stub 02188c9c force ret=pick:1/1/1/0x3f args=v  # fieldmap zone id
  stub 02159bd4 force ret=pick:1/1/0x3f/0xFFFF args=v,v,v  # map_matrix (zone id at a position)
  stub 0200c234 force ret=pick:1/1/1/0 args=v,v  # (GAMEDATA getter result, 0) predicate: effect encounters on
  stub 02188cfc force ret=pick:0/0/0/1 args=v  # fieldmap getter (map mode)
  stub 02159c48 force ret=pick:1/1/1/0 args=v,v,v  # map_matrix (in bounds)
  stub 02012974 force ret=u32 args=v  # GAMEDATA -> map matrix
  stub 02012f08 force ret=u32 args=v  # GAMEDATA getter
  stub 0216d454 force ret=pick:0/1 args=v  # MMDL in use
  stub 0216d828 force ret=none args=v,outm:6:0x3/0/0/0/0x3/0  # MMDL grid x / y / z (s16 x3) -> out
  stub 0216d83c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gz
  stub 0216d88c force ret=none args=v,out:12  # MMDL position VecFx32 -> out
  stub 0216dd1c force seq ret=pick:0/1 args=v,outp:work:0x100:2,ptr  # MMDL iterator (sys, &m, &i): MMDL work+0x100 or +0x104
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188c7c force ret=u32 args=v  # fieldmap -> MMDLSYS
  stub 02188c98 force ret=u32 args=v  # fieldmap -> map mapper
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 0218d168 force ret=u32 args=v  # mapper: block / grid handle
  stub 0218dbac force ret=none args=v,v,in:12,out:0x10  # mapper attribute at a position (mapper, h, &pos, &out)
  stub 0218dbd4 force ret=pick:0/1 args=v,v  # mapper: grid ready (mapper, h)
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL
  stub 021aabc4 ret=pick:0/1 args=v,v  # in-area
  stub 021aabec ret=none args=v  # in-area
  stub 021aabf4 ret=none args=v,v,outm:24:0/0/0/0/0/0/0/0/0/0/0/0/0/0/0/0/3/0/3/0/3/0/3/0  # in-area
  stub 021aacd4 ret=none args=v,v  # in-area
  stub 021aae34 ret=none args=v,v  # in-area
  stub 021aaec0 ret=pick:0/1 args=v,v,v,v,v  # in-area
  stub 021aafc0 ret=none args=v,v,v  # in-area
  stub 021ab504 force ret=pick:0/1/2/3/4/5/6/7/8 args=v  # map_attr
  stub 021adaac force ret=pick:0/1/0x02EE0000 args=v,v,v,v,v,v  # fldeff_encount create (eff, ctrl, x, z, y, kind)

# 021AA8EC(enc) -> 021AAFA0(enc, enc +0x10) (tail jump; the thunk 021AA8F8 -> 021AAF54 follows it)
fn 021aa8ec ret=none inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x10 u32 addrof arg0 0x200
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 021aafa0 ret=none args=v,v  # in-area
  stub 021aaffc ret=none args=v  # in-area
  stub 021ac154 force ret=none args=v  # field effect delete (obj)

# 021AA900(enc, pos) -> on the effect: s = ENCOUNT WORK +0x10; active and (x, y, z) at +0x12 / +0x14 / +0x16 == pos s16
# [0] / [1] / [2]
fn 021aa900 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg buf 8
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  fix arg0 0x7D2 u16 range 0 1
  fix arg0 0x7D4 u16 range 0 1
  fix arg0 0x7D6 u16 range 0 1
  fix arg1 0 u16 range 0 1 rep=3,2
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)

# 021AA944(enc) -> battle event or 0: active effect under the player (0216D828 grid x / y / z == state x / y / z); r =
# 02005728(1000); kind 7 and r >= 200: item 021AB070, 021AB0AC(enc, item) (script); kind 4 and r >= 400: item 021AB018; else event =
# 021A9F40(enc, 1); event: item path -> 021AAFA0(enc, eff); else kind 5 / 6 -> 021AAB1C(enc, 1); state bits 6..7 = 1; 021A3624(player)
# -> 021A3644(player); no event: 021AAFA0(enc, eff), 0
fn 021aa944 ret=u32 inputs=300
  nowork
  arg buf 0x800
  fix arg0 0x10 u32 addrof arg0 0x200
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  fix arg0 0x7D2 u16 const 0
  fix arg0 0x7D4 u16 const 0
  fix arg0 0x7D6 u16 const 0
  fix arg0 0x7DC u8 alt oneof 4 5 6 7 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 0216d828 force ret=none args=v,outm:6:1/0/0/0/0/0  # MMDL grid x / y / z (s16 x3) -> out
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10 script call (gsys, script 0x2795, 0, heap)
  stub 02188c84 force ret=u16 args=v  # fieldmap heap id
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL
  stub 021a3624 force ret=pick:0/1 args=v  # field player predicate
  stub 021a3644 force ret=none args=v  # field player (player)
  stub 021a9f40 force ret=nullable:2:u32 args=v,v  # encount
  stub 021aab1c ret=none args=v,v  # in-area
  stub 021aafa0 ret=none args=v,v  # in-area
  stub 021aaffc ret=none args=v  # in-area
  stub 021ab018 ret=pick:80/229/548/564 args=  # in-area
  stub 021ab070 ret=pick:565/571 args=  # in-area
  stub 021ab0ac ret=nullable:2:u32 args=v,v  # in-area
  stub 021ac154 force ret=none args=v  # field effect delete (obj)
  stub 021adb5c force ret=none args=v,v  # fldeff_encount (obj, v)

# 021AAA4C(enc, &d) -> active: d = max(|state x - player gx|, |state z - player gz|) (s16), else d = 0
fn 021aaa4c ret=u32 inputs=100
  nowork
  arg buf 0x800
  arg buf 4
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 0216d828 force ret=none args=v,outm:6:0x3/0/0/0/0x3/0  # MMDL grid x / y / z (s16 x3) -> out
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL

# 021AAABC(enc): active: state bits 6..7 = 1; eff +0x5B4: 021ADB4C(obj, 1)
fn 021aaabc ret=none inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x10 u32 addrof arg0 0x200
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  fix arg0 0x7B4 u32 alt const 0 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 021adb4c force ret=none args=v,v  # fldeff_encount anim (obj, v)

# 021AAAF0(enc, v): active and eff +0x5B4: 021ADB4C(obj, v)
fn 021aaaf0 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x10 u32 addrof arg0 0x200
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  fix arg0 0x7B4 u32 alt const 0 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 021adb4c force ret=none args=v,v  # fldeff_encount anim (obj, v)

# 021AAB1C(enc, v): active and eff +0x5B4: 021ADB5C(obj, v)
fn 021aab1c ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x10 u32 addrof arg0 0x200
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  fix arg0 0x7B4 u32 alt const 0 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 021adb5c force ret=none args=v,v  # fldeff_encount (obj, v)

# 021AAB48(enc) -> eff +0x5BC (the item, u16)
fn 021aab48 ret=u32 inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x10 u32 addrof arg0 0x200

# 021AAB54(enc): active: state bits 4..5 = 1
fn 021aab54 ret=none inputs=100
  nowork
  arg buf 0x800
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)

# 021AAB74(enc, eff): active and bits 4..5: clear them; zone 02188C9C(fieldmap) != state zone or stepped on (bits 6..7):
# MI_CpuFill8(state, 0, 0x10); else 021AAFC0(enc, eff, state) (recreate the effect object)
fn 021aab74 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 addrof arg0 0x200
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  fix arg0 0x7D0 u16 oneof 0 1 0x3f
  fix arg0 0x7B4 u32 alt const 0 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 021aafc0 ret=none args=v,v,v  # in-area
  stub 021adaac force ret=pick:0/1/0x02EE0000 args=v,v,v,v,v,v  # fldeff_encount create (eff, ctrl, x, z, y, kind)

# 021AABC4(ewk, eff) -> due: ewk +0xC != 0xFFFFFFFF: +1; ewk +0xC >= eff +0x5B8 (u16)
fn 021aabc4 ret=u32 inputs=200
  nowork
  arg buf 0x20
  arg buf 0x5C0
  fix arg0 0xC u32 alt range 0 30 || oneof 0xFFFFFFFF 0xFFFFFFFE || any
  fix arg1 0x5B8 u16 alt range 0 30 || any

# 021AABEC(ewk): ewk +0xC = 0
fn 021aabec ret=none inputs=100
  nowork
  arg buf 0x20

# 021AABF4(enc, eff, w): the search window (s16 x 12): w[0] / w[1] = player gx / gz, w[2] / w[3] = their tile in the 32-tile
# block, w[4] / w[5] = block, w[6] / w[7] = block origin; o = ROM 0x021D8E76 (kind 2) or + 4 (02005728(4) + 1); x0 = max(w[2] + o[0], 0),
# x1 = min(w[2] + o[1], 31), z0 = max(w[3] + o[2], 0), z1 = min(w[3] + o[3], 31); w[8] / w[9] / w[10] / w[11] = origin + x0 / z0 / x1 /
# z1
fn 021aabf4 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg buf 0x18
  fix arg0 0xC u32 addrof arg0 0x20
  fix arg0 0x27 u8 alt oneof 0x82 0x80 0x81 || any
  stub 0216d83c force ret=pick:0/1/3/4/0x1B/0x1F/0x20/0x23/0x3B/0x40/0xFFFF/0xFFE0 args=v  # MMDL gx (0x1B / 0x3B: tile 27, x1 = 32)
  stub 0216d86c force ret=pick:0/1/3/4/0x1B/0x1F/0x20/0x23/0x3B/0x40/0xFFFF/0xFFE0 args=v  # MMDL gz (0x1B / 0x3B: tile 27, z1 = 32)
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL

# 021AACD4(enc, eff): candidates: MI_CpuFill8(eff +4, 0, 0x5B0); h = 0218D168(mapper); 0218DBD4(mapper, h) == 0: none; window
# 021AABF4(enc, eff, &w); for z = w[9] .. w[11], x = w[8] .. w[10]: pos = (x, 0, z) tile centres in fx32 (soft float: (t << 16) +-
# 0.5, round, + 0x8000); 0218DBAC(mapper, h, &pos, &out); k = 021AB504(out attribute) != 8: candidate (k, x, z, out height)
fn 021aacd4 ret=none inputs=200
  nowork
  arg buf 0x800
  arg u32 addrof arg0 0x200
  fix arg0 0xC u32 addrof arg0 0x20
  fix arg0 0x27 u8 alt oneof 0x82 0x80 0x81 || any
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max
  stub 0216d83c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gx
  stub 0216d86c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gz
  stub 02188c98 force ret=u32 args=v  # fieldmap -> map mapper
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 0218d168 force ret=u32 args=v  # mapper: block / grid handle
  stub 0218dbac force ret=none args=v,v,in:12,out:0x10  # mapper attribute at a position (mapper, h, &pos, &out)
  stub 0218dbd4 force ret=pick:0/1 args=v,v  # mapper: grid ready (mapper, h)
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL
  stub 021aabf4 ret=none args=v,v,outm:24:0/0/0/0/0/0/0/0/0/0/0/0/0/0/0/0/3/0/3/0/3/0/3/0  # in-area
  stub 021ab504 force ret=pick:0/1/2/3/4/5/6/7/8 args=v  # map_attr

# 021AAE34(enc, eff): c = candidate[02005728(count)]; c.y = (c.height >> 4 (rounded toward zero)) >> 12 ...; 021AAEC0(enc, c.x,
# c.y, c.z, 0) == 0 (no MMDL there): state = (zone 02188C9C, c.x, c.y, c.z, c.height, c.kind), active = 1; 021AABEC(ewk);
# 021AAFC0(enc, eff, state)
fn 021aae34 ret=none inputs=100
  arg buf 0x800
  arg u32 addrof arg0 0x200
  fix arg0 0x204 u16 range 1 3
  fix arg0 0x7B4 u32 alt const 0 || any
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 02005728 force ret=lt:0 args=v  # RNG next_scaled(count): below count
  stub 0216d454 force ret=pick:0/1 args=v  # MMDL in use
  stub 0216d828 force ret=none args=v,outm:6:0x3/0/0/0/0x3/0  # MMDL grid x / y / z (s16 x3) -> out
  stub 0216dd1c force seq ret=pick:0/1 args=v,outp:work:0x100:2,ptr  # MMDL iterator (sys, &m, &i): MMDL work+0x100 or +0x104
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188c7c force ret=u32 args=v  # fieldmap -> MMDLSYS
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL
  stub 021aabec ret=none args=v  # in-area
  stub 021aaec0 ret=pick:0/1 args=v,v,v,v,v  # in-area
  stub 021aafc0 ret=none args=v,v,v  # in-area
  stub 021adaac force ret=pick:0/1/0x02EE0000 args=v,v,v,v,v,v  # fldeff_encount create (eff, ctrl, x, z, y, kind)

# 021AAEC0(enc, x, y, z, skip_player) -> an MMDL in use covers (x, y, z): for every MMDL (0216DD1C) in use (0216D454), not the
# player when skip_player: d = OBJ data (0216E3C4); (gx, gy, gz) = 0216D828(m); y == gy, gx <= x < gx + d +0xB, gz - d +0xC < z <= gz
fn 021aaec0 ret=u32 inputs=200
  arg buf 0x800
  arg u32 range 0 3
  arg u32 range 0 1
  arg u32 range 0 3
  arg u32 alt range 0 1 || any
  stub 0216d828 force ret=none args=v,outm:6:3/0/1/0/3/0  # MMDL grid x / y / z -> out
  stub 021a3074 force ret=pick:0x02E00100/0x02E00104/0 args=v  # field player -> MMDL
  stub 0216d454 force ret=pick:0/1 args=v  # MMDL in use
  stub 0216dd1c force seq ret=pick:0/1 args=v,outp:work:0x100:2,ptr  # MMDL iterator (sys, &m, &i): MMDL work+0x100 or +0x104
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188c7c force ret=u32 args=v  # fieldmap -> MMDLSYS
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)

# 021AAF54(enc) -> an MMDL (not the player) on the active effect: 021AAEC0(enc, state x, y, z, 1): 021AAFA0(enc, enc +0x10), 1;
# else 0 (also reached through the thunk 021AA8F8)
fn 021aaf54 ret=u32 inputs=100
  arg buf 0x800
  fix arg0 0x10 u32 addrof arg0 0x200
  fix arg0 0x7DD u8 alt oneof 0 1 0x11 0x41 0x21 0x81 || any
  fix arg0 0x7D2 u16 range 0 3
  fix arg0 0x7D4 u16 range 0 1
  fix arg0 0x7D6 u16 range 0 3
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 0216d828 force ret=none args=v,outm:6:3/0/1/0/3/0  # MMDL grid x / y / z -> out
  stub 0216d454 force ret=pick:0/1 args=v  # MMDL in use
  stub 0216dd1c force seq ret=pick:0/1 args=v,outp:work:0x100:2,ptr  # MMDL iterator (sys, &m, &i): MMDL work+0x100 or +0x104
  stub 0216e3c4 force ret=ptr:work:0x900:1 args=v  # MMDL OBJ data (m + 0xE4)
  stub 02188c7c force ret=u32 args=v  # fieldmap -> MMDLSYS
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL
  stub 021aaec0 ret=pick:0/1 args=v,v,v,v,v  # in-area
  stub 021aafa0 ret=none args=v,v  # in-area
  stub 021aaffc ret=none args=v  # in-area
  stub 021ac154 force ret=none args=v  # field effect delete (obj)

# 021AAFA0(enc, eff): end the effect: 021AAFFC(eff); MI_CpuFill8(state, 0, 0x10)
fn 021aafa0 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 addrof arg0 0x200
  fix arg0 0x7B4 u32 alt const 0 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 021aaffc ret=none args=v  # in-area
  stub 021ac154 force ret=none args=v  # field effect delete (obj)

# 021AAFC0(enc, eff, rec): eff +0x5B4 == 0: obj = 021ADAAC(enc, eff +0 (control), rec x +2, z +6, height +8, kind +0xC);
# +0x5B4 = obj; obj == 0: MI_CpuFill8(rec, 0, 0x10)
fn 021aafc0 ret=none inputs=100
  nowork
  arg u32 any
  arg buf 0x5C0
  arg buf 0x10
  fix arg1 0x5B4 u32 alt const 0 || any
  stub 021adaac force ret=pick:0/1/0x02EE0000 args=v,v,v,v,v,v  # fldeff_encount create (eff, ctrl, x, z, y, kind)

# 021AAFFC(eff): eff +0x5B4 != 0: 021AC154(obj) (field effect delete); +0x5B4 = 0
fn 021aaffc ret=none inputs=100
  nowork
  arg buf 0x5C0
  fix arg0 0x5B4 u32 alt const 0 || any
  stub 021ac154 force ret=none args=v  # field effect delete (obj)

# 021AB018() -> dust cloud item: r = 02005728(1000) (u16): < 100: ROM 0x021D8E8A[02005728(1000) / 100] (10 evolution stones
# 80..85, 107..110); < 950: 548 + 02005728(1700) / 100 (gems 548..564); else 229 (Everstone)
fn 021ab018 ret=u32 inputs=200
  nowork
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max

# 021AB070() -> bird shadow item: 02005728(1000) < 900: 565 + 02005728(600) / 100 (wings 565..570); else 571
fn 021ab070 ret=u32 inputs=100
  nowork
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max

# 021AB0AC(enc, item) -> eff +0x5BC = item; 02158768(gsys, 0x2795, 0, 02188C84(fieldmap) (heap)) (the item script)
fn 021ab0ac ret=u32 inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  fix arg0 0x10 u32 addrof arg0 0x200
  stub 02158768 force ret=u32 args=v,v,v,v  # ov10 script call (gsys, script 0x2795, 0, heap)
  stub 02188c84 force ret=u16 args=v  # fieldmap heap id

