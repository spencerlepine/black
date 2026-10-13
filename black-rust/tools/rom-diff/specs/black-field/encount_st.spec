# fndiff spec: field_encount_st.c (overlay 10 Thumb code 0x0215E2B8..0x0215E440, F1 row 10.18; F5a-spec): the zone's wild
# encounter table load (ARC 126), the FIELD ENCOUNT ST work alloc, the roamer (move_pokemon) hooks on zone change and the
# swarm zone table. File bounds: `Heap_AllocDebug(heap, 0x20, 1, "field_encount_st.c", 0x2D)` in 0215E31C (string 0x02172928);
# the next file (move_pokemon.c) starts at 0x0215E440.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field encount_st --per-function
# Set `field` (main 10 11 18 20 21).
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry; 11 entries,
# minus the veneer 0215E334 (`ldr r3, =Heap_Free; bx r3`; callers stub 020307B0) = 10 specced. Render-only functions: none.
# Closure stubs (as core4 / F4a / F6a): a block lists the stubs of its callees and of the callees of its in-area descendants, so the area
# can be ported in any order (checked with every function treated as ported: no fault, stray write or unstubbed callee). Every
# out-of-area stub is `force` (overlay addresses collide with battle PORTED entries; ARM9 / other-area callees are ported elsewhere).
#
# ENCOUNT TABLE (one ARC 126 member = 1 or 4 seasonal tables of 0xE8 bytes, black-data `encounters`): `+0..+6 u8` rates of the seven
# slot groups (encounter type t = 0 grass, 1 dark grass (doubles), 2 shaking grass, 3 surf, 4 rippling water, 5 fishing, 6 fishing
# on a ripple; read by field_encount 021AA3E0), `+7 u8` = effect encounter kind 0..2 (low 7 bits, effect_encount 021AA7DC) | 0x80 =
# loaded (set by 0215E2B8 in the RAM copy; the ROM byte has bit 7 clear), `+8` 56 slots of 4 bytes (u16 species | form << 11, u8
# min, u8 max level): groups 12 / 12 / 12 / 5 / 5 / 5 / 5 at +0x08 / +0x38 / +0x68 / +0x98 / +0xAC / +0xC0 / +0xD4 (enc_pokeset
# 021A9A5C). A member of 0xE8 bytes is used in every season; one of 0x3A0 bytes holds the four seasons in order; any other size is
# rejected (the table stays zero, not loaded).
# ENC SAVE (save block 0x38, `0200D418(GAMEDATA)`): `+0 u16` current location, `+2 u16` previous, `+4` two roamer records of 0x14
# (field getter 0200D4B8(rec, id)), `+0x16 + 0x14 i u8` roamer i active, `+0x2C[i] u8`, `+0x30 u8` swarm index, `+0x31 u8` repel steps.
# All blocks are `nowork`; objects are `arg buf` arenas or stub answers.
area encount_st
set field

# 0215E2B8(tab, handle, zone, season) -> loaded: MI_CpuFill8(tab, 0, 0xE8); m = 02013EC0(zone) (ARC 126 member); m == 0xFFFF: 0;
# n = 020493D4(handle, m) / 0xE8; n != 1 and n != 4: 0; n == 1: season = 0; 0204944C(handle, m, season * 0xE8, 0xE8, tab);
# tab[7] |= 0x80; 1
fn 0215e2b8 ret=u32 inputs=100
  nowork
  arg buf 0xE8
  arg u32 any
  arg u32 any
  arg u32 alt range 0 3 || any
  stub 02013ec0 force ret=pick:0/1/2/0x6F/0xFFFF args=v  # zonedata: zone -> ARC 126 member (encount id; 0xFFFF none)
  stub 020493d4 force ret=pick:0xE8/0x3A0/0x1D0/0 args=v,v  # archive member size (handle, member): 1 / 4 / 2 tables or empty
  stub 0204944c force ret=none args=v,v,v,v,out:0xE8  # archive read (handle, member, offset, size, dst)

# 0215E31C(heap) -> Heap_AllocDebug(heap, 0x20, 1, "field_encount_st.c", 0x2D) (the FIELD ENCOUNT ST work)
fn 0215e31c ret=u32 inputs=100
  nowork
  arg u32 any
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)

# 0215E33C(fieldmap): enc = 02188CE8(fieldmap) (FIELD ENCOUNT); 021AA7DC(enc, enc +0x10) (effect_encount: interval and
# probability from the table's kind)
fn 0215e33c ret=none inputs=100
  arg u32 any
  stub 02188ce8 force ret=ptr:work:0x100:1 args=v  # fieldmap -> FIELD ENCOUNT
  stub 021aa7dc force ret=none args=v,v  # effect_encount

# 0215E34C(gamedata): 02012A38(gamedata) +0xC = 0 (the effect encounter step counter; the zone-change hook, also reached
# through the veneers 02164454 / 5C / 64 / 6C)
fn 0215e34c ret=none inputs=100
  arg u32 any
  stub 02012a38 force ret=ptr:work:0x200:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)

# 0215E358(sv, season, i) -> 0215E740(sv, season, i, 0xFF) (move_pokemon: move roamer i)
fn 0215e358 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0215e740 force ret=u32 args=v,v,v,v  # move_pokemon: roamer location update (sv, season, i, 0xFF)

# 0215E364(gsys): sv = 0200D418(02012EBC(gsys)); s = 02012984(gsys); i = 0, 1: 0200D488(sv, i) != 0: 0215E358(sv, s, i)
# (every active roamer moves; the zone-change hook)
fn 0215e364 ret=none inputs=100
  nowork
  arg u32 any
  stub 0200d418 force ret=u32 args=v  # (GAMEDATA) -> ENC SAVE (save block 0x38)
  stub 0200d488 force ret=pick:0/1 args=v,v  # ENC SAVE roamer i active (+0x16 + 0x14 i)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02012ebc force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0215e358 ret=u32 args=v,v,v  # in-area
  stub 0215e740 force ret=u32 args=v,v,v,v  # move_pokemon: roamer location update (sv, season, i, 0xFF)

# 0215E3A0(gsys): as 0215E364, with a 0200D4AC(sv, i) call (result unused) before each test
fn 0215e3a0 ret=none inputs=100
  nowork
  arg u32 any
  stub 0200d418 force ret=u32 args=v  # (GAMEDATA) -> ENC SAVE (save block 0x38)
  stub 0200d488 force ret=pick:0/1 args=v,v  # ENC SAVE roamer i active (+0x16 + 0x14 i)
  stub 0200d4ac force ret=u32 args=v,v  # ENC SAVE roamer record i (sv + 4 + 0x14 i)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02012ebc force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 0215e358 ret=u32 args=v,v,v  # in-area
  stub 0215e740 force ret=u32 args=v,v,v,v  # move_pokemon: roamer location update (sv, season, i, 0xFF)

# 0215E3E4(sv, i) -> zone: k = 0200D478(sv, i) (r1 passed through); k > 16: 319; ROM 0x021711C6[k] (u16 x 17: the swarm
# zones 319, 321, 326, .., 387, 0xFFFF)
fn 0215e3e4 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 range 0 3
  stub 0200d478 force ret=pick:0/1/5/0x10/0x11/0xFF args=v,v  # ENC SAVE +0x2C[i] (byte)

# 0215E404(sv) -> any roamer active: i = 0, 1: 0200D488(sv, i) != 0: 1; 0
fn 0215e404 ret=u32 inputs=100
  nowork
  arg u32 any
  stub 0200d488 force ret=pick:0/1 args=v,v  # ENC SAVE roamer i active (+0x16 + 0x14 i)

# 0215E428(sv, loc): 0215E404(sv) != 0: 0200D45C(sv, loc) (push the location for the roamers)
fn 0215e428 ret=none inputs=100
  nowork
  arg u32 any
  arg u32 any
  stub 0200d45c force ret=none args=v,v  # ENC SAVE location push (sv, loc)
  stub 0200d488 force ret=pick:0/1 args=v,v  # ENC SAVE roamer i active (+0x16 + 0x14 i)
  stub 0215e404 ret=pick:0/1 args=v  # in-area

