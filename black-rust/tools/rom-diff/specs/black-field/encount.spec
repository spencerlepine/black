# fndiff spec: field_encount.c (overlay 21 Thumb code 0x021A9ED8..0x021AA77C, F1 row 21.29; F5a-spec): the FIELD ENCOUNT object, the
# wild encounter check of a step (type from the map attribute, rate check, slot roll, roamers, battle setup + battle event), the fixed
# (scripted) wild battle, the fishing check, the WFBC encounter, the battle field situation, the trainer battle setup and the
# walk counter. File bounds: `Heap_AllocDebug(heap, 0x14, 1, "field_encount.c", 0x62)` in 021A9ED8 (string 0x021DDFA4, also used by
# 021AA490 / 021AA548); effect_encount.c (string 0x021DDFB4 in 021AA77C) starts at 0x021AA77C.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field encount --per-function
# Set `field` (main 10 11 18 20 21).
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry; 19
# entries, no veneers or fragments: 19 specced. Render-only functions: none.
# Closure stubs (as core4 / F4a / F6a): a block lists the stubs of its callees and of the callees of its in-area descendants, so the area
# can be ported in any order (checked with every function treated as ported: no fault, stray write or unstubbed callee). Every
# out-of-area stub is `force` (also those into enc_pokeset / effect_encount / map_attr, the other F5a areas).
#
# The encounter roll of a step (021A9F40(enc, kind), kind 0 walk / 1 effect encounter / 2 forced):
#  1. kind 0 counts the step (021AA728: ENCOUNT WORK +8 = steps since the last reset, +6 = moved since then);
#  2. table loaded (enc +0xC != 0 and byte 7 bit 7), else none;
#  3. type = 021AA35C(enc, kind, &bonus) from the attribute under the player (map_attr): not an encounter attribute (flags bit 2) and
#     kind != 1: none; water attribute (flags bit 1): 3 surf (4 rippling water with kind 1); else 1 dark grass when the value is
#     6 / 7 / 9 / 0x22 (021AB29C), 0 grass (2 shaking grass with kind 1); bonus = +10 on attribute values 8 / 9 (021AB140);
#  4. rate = table[type] (+ bonus when nonzero, min 0; 021AA3E0); 0 or type 0xFF: none;
#  5. EFP of the lead (enc_pokeset 021A9350, weather from 021A1BF4);
#  6. kind 0 only: rate' = 021A94B0 (ability / incense / pass power modifiers, min 100); 021AA3FC: steps 0 -> none, steps 1 -> rate 1;
#     encounter when 021AA6E0() (0..99) <= rate';
#  7. kind 1 counts a record (02008DF0(rec, 0x70)); kind != 1: a roamer in this zone (enc_pokeset 021A957C, 50 %) replaces the roll;
#  8. else 021A954C: the slot group of the type (enc_pokeset 021A9A5C, or the swarm slot), the slot pick (land 20/20/10/10/10/10/5/5/
#     4/4/1/1, water 60/30/5/4/1, fishing 40/40/15/4/1, Magnet Pull / Static preference), level min + roll % (max - min + 1), form
#     (0x1F = random), the level filter (repel: below the lead's level; Intimidate / Keen Eye: 5 below, half the time) per Pokemon;
#     none accepted: none; a double with one accepted becomes a single;
#  9. the battle setup (btl_setup alloc / setup_with_party with the field situation 021AA304), the reset of the walk counter
#     (021AA6F8) and the battle event 0216E7C8(gsys, fieldmap, setup, 0, kind).
#
# FIELD ENCOUNT (0x14, 021A9ED8): `+0` FIELDMAP, `+4` GAMESYS, `+8` GAMEDATA, `+0xC` -> ENCOUNT TABLE (EVENTDATA_SYSTEM + 0x38, see
# encount_st.spec), `+0x10` -> EFFECT ENCOUNT work (0x5C0, effect_encount.spec). ENCOUNT WORK (0x20 bytes, `02012A38(GAMEDATA)`):
# `+0 / +2 / +4 u16` grid x / y / z at the last reset, `+6 u16` moved, `+8 u32` steps (saturating at 0xA000), `+0xC..` effect
# encounter state (effect_encount.spec). FIELD SITUATION (0x10, 021AA304): `+0 u32` battle background (02013ED8(zone)), `+4 u32` ground
# kind (021AB580 of the attribute value), `+8 u8` weather class (021AA69C), `+9 u8` season, `+0xA u16` zone, `+0xC / +0xD u8` RTC
# hour / minute.
# Arena: the FIELD ENCOUNT object is `arg buf 0x800`: object at +0, ENCOUNT TABLE at +0x20 (0xE8 bytes; `+0xC` fixed to it or null),
# EFFECT ENCOUNT work at +0x200, ENCOUNT WORK (the `02012A38` stub's answer) at +0x7C0. EFP / MON / FIELD SITUATION locals reach
# their callees as `in:` / `out:` buffers (the in-area enc_pokeset stubs fill them).
area encount
set field

# 021A9ED8(fieldmap) -> enc: Heap_AllocDebug(02188C84(fieldmap), 0x14, 1, "field_encount.c", 0x62); +0 fieldmap, +4 gsys =
# 02188C80(fieldmap), +8 = 020120F4(gsys), +0xC = 0216255C(02012844(gd)), +0x10 = 021AA77C(heap) (effect_encount alloc);
# 021AA7AC(enc, +0x10). Arena: argument 1 (the block the allocator returns)
fn 021a9ed8 ret=ofs:arg1 inputs=100
  nowork
  arg u32 any
  arg buf 0x20
  stub 02030734 force ret=ptr:arg1:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, 0x14, 1, file, 0x62)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012844 force ret=u32 args=v  # GAMEDATA -> EVENTDATA_SYSTEM
  stub 0216255c force ret=u32 args=v  # eventdata: EVENTDATA_SYSTEM -> encount table (sys + 0x38)
  stub 02188c80 force ret=u32 args=v  # fieldmap -> GAMESYS
  stub 02188c84 force ret=u16 args=v  # fieldmap heap id
  stub 021aa77c force ret=u32 args=v  # effect_encount
  stub 021aa7ac force ret=none args=v,v  # effect_encount

# 021A9F28(enc): 021AA7CC(enc, enc +0x10); 021AA794(enc +0x10); Heap_Free(enc)
fn 021a9f28 ret=none inputs=100
  nowork
  arg buf 0x20
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 021aa794 force ret=none args=v  # effect_encount
  stub 021aa7cc force ret=none args=v,v  # effect_encount

# 021A9F40(enc, kind) -> battle event or 0: the step check (header, steps 1..9); kind 1 / 2 skip the walk counter and the rate
# roll; kind 1 counts record 0x70
fn 021a9f40 ret=u32 inputs=300
  nowork
  arg buf 0x800
  arg u32 alt range 0 3 || any
  fix arg0 0xC u32 alt addrof arg0 0x20 || const 0
  fix arg0 0x27 u8 alt oneof 0x80 0x81 0x82 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 02008df0 force ret=none args=v,v  # record counter add (rec, id)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02012f2c force ret=u32 args=v  # GAMEDATA -> record counters
  stub 02013090 force ret=u32 args=v  # btl_setup alloc (heap 4)
  stub 02013144 force ret=none args=v,v  # btl_setup or_flags (setup, mask)
  stub 0201347c force ret=none args=v,v,v,in:16,v,v  # btl_setup setup_with_party (setup, gamedata, party, &field, double, heap)
  stub 02013ed8 force ret=u32 args=v  # zonedata: zone -> battle background
  stub 0201a920 force ret=u32 args=v  # POKEPARTY alloc (heap)
  stub 0201a98c force ret=none args=v,v  # POKEPARTY add (party, pp)
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 0203f4cc force ret=none args=out:12  # RTC time (hour, minute, second)
  stub 0216d83c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFF args=v  # MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gz
  stub 0216e7c8 force ret=u32 args=v,v,v,v,v  # event_battle: wild battle event (gsys, fieldmap, setup, kind, encount kind)
  stub 02188c6c force ret=u32 args=v  # fieldmap -> weather system
  stub 02188c98 force ret=u32 args=v  # fieldmap -> map mapper
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a1bf4 force ret=pick:0/1/2/3/4/5/6/7/0xB/0xC/0xD args=v  # weather id (weather system)
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute word
  stub 021a9350 force ret=none args=outm:0x2C:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0x1/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF,v,v,v,v  # enc_pokeset
  stub 021a94b0 force ret=pick:0/1/10/50/100 args=in:0x2C,v,v  # enc_pokeset
  stub 021a954c force ret=pick:0/1/2 args=v,in:0x2C,v,out:0x28  # enc_pokeset
  stub 021a957c force ret=nullable:2:u32 args=v,in:0x2C,v  # enc_pokeset
  stub 021a96b8 force ret=none args=v,v,in:0x14  # enc_pokeset
  stub 021a9780 force ret=none args=v,v,v  # enc_pokeset
  stub 021aa304 ret=none args=out:0x10,v,v  # in-area
  stub 021aa35c ret=pick:0/1/2/3/4/5/6/0xFF args=v,v,out:1  # in-area
  stub 021aa3e0 ret=pick:0/1/10/25/100 args=v,v,v  # in-area
  stub 021aa3fc ret=pick:0/1 args=v,v,v  # in-area
  stub 021aa490 ret=none args=v,in:0x2C,v,v,in:0x28  # in-area
  stub 021aa548 ret=none args=v,in:0x2C,v,v,v  # in-area
  stub 021aa69c ret=pick:0/2/3/4 args=v  # in-area
  stub 021aa6e0 ret=pick:0/1/9/10/49/50/98/99 args=  # in-area
  stub 021aa6f8 ret=none args=v,v  # in-area
  stub 021aa728 ret=none args=v,v  # in-area
  stub 021aa778 ret=pick:0/1/2/5/100 args=v  # in-area
  stub 021ab0f0 force ret=u16 args=v  # map_attr
  stub 021ab0f8 force ret=pick:0/1/2/4/6/5/7 args=v  # map_attr
  stub 021ab100 force ret=pick:0/1 args=v  # map_attr
  stub 021ab140 force ret=pick:0/1 args=v  # map_attr
  stub 021ab29c force ret=pick:0/1 args=v  # map_attr
  stub 021ab580 force ret=pick:0/1/4/5/8 args=v  # map_attr

# 021AA0A0(enc, species, level, flags): fixed wild battle: EFP(enc gsys, type 0xFF, kind 0); MON species, level; flags 4: item 0xFFFF
# (none), 2: shiny mode 1, else 0x10: 2, 8: +7 = 1, 0x20: gender 1, else 0x40: 2; setup = 02013090(4); 021AA490(enc, efp, setup, 4,
# &mon); flags 1: or_flags 4, else 0x80: or_flags 0x80; 0x100: or_flags 0x200; 0216E7C8(gsys, fieldmap, setup, 1, 0)
fn 021aa0a0 ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 any
  arg u32 alt mask 0x1FF || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02013090 force ret=u32 args=v  # btl_setup alloc (heap 4)
  stub 02013144 force ret=none args=v,v  # btl_setup or_flags (setup, mask)
  stub 0201347c force ret=none args=v,v,v,in:16,v,v  # btl_setup setup_with_party (setup, gamedata, party, &field, double, heap)
  stub 02013ed8 force ret=u32 args=v  # zonedata: zone -> battle background
  stub 0201a920 force ret=u32 args=v  # POKEPARTY alloc (heap)
  stub 0201a98c force ret=none args=v,v  # POKEPARTY add (party, pp)
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 0203f4cc force ret=none args=out:12  # RTC time (hour, minute, second)
  stub 0216e7c8 force ret=u32 args=v,v,v,v,v  # event_battle: wild battle event (gsys, fieldmap, setup, kind, encount kind)
  stub 02188c6c force ret=u32 args=v  # fieldmap -> weather system
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a1bf4 force ret=pick:0/1/2/3/4/5/6/7/0xB/0xC/0xD args=v  # weather id (weather system)
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute word
  stub 021a9350 force ret=none args=outm:0x2C:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0x1/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF,v,v,v,v  # enc_pokeset
  stub 021a96b8 force ret=none args=v,v,in:0x14  # enc_pokeset
  stub 021aa304 ret=none args=out:0x10,v,v  # in-area
  stub 021aa490 ret=none args=v,in:0x2C,v,v,in:0x28  # in-area
  stub 021aa69c ret=pick:0/2/3/4 args=v  # in-area
  stub 021ab0f0 force ret=u16 args=v  # map_attr
  stub 021ab580 force ret=pick:0/1/4/5/8 args=v  # map_attr

# 021AA180(enc, kind) -> battle setup or 0: fishing: table loaded; type 6 (kind 1: on a ripple) or 5; rate = 021AA3E0(enc, type,
# 0) (0: none); EFP; kind != 1: 021A94B0 rate below 021AA6E0(): none; 021A954C(table, efp, zone, mons) below efp +0x15: none; setup =
# 02013090(4); 021AA490(..); or_flags(setup, 1); setup +0xC = 6. Arena: setup inside argument 0 (+0x1C0)
fn 021aa180 ret=ofs:arg0 inputs=100
  nowork
  arg buf 0x800
  arg u32 alt range 0 2 || any
  fix arg0 0xC u32 alt addrof arg0 0x20 || const 0
  fix arg0 0x27 u8 alt oneof 0x80 0x81 0x82 || any
  fix arg0 0x25 u16 alt const 0 || any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 02013090 force ret=ptr:arg0:0x1C0:1 args=v  # btl_setup alloc (heap 4)
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02013144 force ret=none args=v,v  # btl_setup or_flags (setup, mask)
  stub 0201347c force ret=none args=v,v,v,in:16,v,v  # btl_setup setup_with_party (setup, gamedata, party, &field, double, heap)
  stub 02013ed8 force ret=u32 args=v  # zonedata: zone -> battle background
  stub 0201a920 force ret=u32 args=v  # POKEPARTY alloc (heap)
  stub 0201a98c force ret=none args=v,v  # POKEPARTY add (party, pp)
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 0203f4cc force ret=none args=out:12  # RTC time (hour, minute, second)
  stub 02188c6c force ret=u32 args=v  # fieldmap -> weather system
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a1bf4 force ret=pick:0/1/2/3/4/5/6/7/0xB/0xC/0xD args=v  # weather id (weather system)
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute word
  stub 021a9350 force ret=none args=outm:0x2C:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0x1/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF,v,v,v,v  # enc_pokeset
  stub 021a94b0 force ret=pick:0/1/10/50/100 args=in:0x2C,v,v  # enc_pokeset
  stub 021a954c force ret=pick:0/1/2 args=v,in:0x2C,v,out:0x28  # enc_pokeset
  stub 021a96b8 force ret=none args=v,v,in:0x14  # enc_pokeset
  stub 021aa304 ret=none args=out:0x10,v,v  # in-area
  stub 021aa3e0 ret=pick:0/1/10/25/100 args=v,v,v  # in-area
  stub 021aa490 ret=none args=v,in:0x2C,v,v,in:0x28  # in-area
  stub 021aa69c ret=pick:0/2/3/4 args=v  # in-area
  stub 021aa6e0 ret=pick:0/1/9/10/49/50/98/99 args=  # in-area
  stub 021ab0f0 force ret=u16 args=v  # map_attr
  stub 021ab580 force ret=pick:0/1/4/5/8 args=v  # map_attr

# 021AA248(enc, wfbc) -> battle event or 0: WFBC encounter: count the step (021AA728); 02193920(wfbc) == 1: none; 021AA42C(ewk,
# wfbc) == 0: none; type = 021AA35C(enc, 3, &b); EFP(type, kind 3); 021A9604(wfbc, efp, mon) == 0: none; setup; 021AA490; reset the walk
# counter; 0216E7C8(gsys, fieldmap, setup, 0, 3)
fn 021aa248 ret=u32 inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  stub 02012a38 force ret=ptr:arg0:0x7C0:1 args=v  # GAMEDATA -> ENCOUNT WORK (0x20)
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 02010fb0 force ret=u32 args=v  # encounter rate modifier of the pass powers (rate -> rate)
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02013090 force ret=u32 args=v  # btl_setup alloc (heap 4)
  stub 02013144 force ret=none args=v,v  # btl_setup or_flags (setup, mask)
  stub 0201347c force ret=none args=v,v,v,in:16,v,v  # btl_setup setup_with_party (setup, gamedata, party, &field, double, heap)
  stub 02013ed8 force ret=u32 args=v  # zonedata: zone -> battle background
  stub 0201a920 force ret=u32 args=v  # POKEPARTY alloc (heap)
  stub 0201a98c force ret=none args=v,v  # POKEPARTY add (party, pp)
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 0203f4cc force ret=none args=out:12  # RTC time (hour, minute, second)
  stub 0207c7b4 force ret=u32 args=v  # FX sqrt / root (fx32)
  stub 0216d83c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFF args=v  # MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gz
  stub 0216e7c8 force ret=u32 args=v,v,v,v,v  # event_battle: wild battle event (gsys, fieldmap, setup, kind, encount kind)
  stub 02188c6c force ret=u32 args=v  # fieldmap -> weather system
  stub 02188c98 force ret=u32 args=v  # fieldmap -> map mapper
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021938f4 force ret=u32 args=v  # field_wfbc: encounter level factor (wfbc)
  stub 02193920 force ret=pick:0/1 args=v  # field_wfbc: encounters off (wfbc)
  stub 021a1bf4 force ret=pick:0/1/2/3/4/5/6/7/0xB/0xC/0xD args=v  # weather id (weather system)
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute word
  stub 021a9350 force ret=none args=outm:0x2C:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0x1/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF,v,v,v,v  # enc_pokeset
  stub 021a9604 force ret=pick:0/1 args=v,in:0x2C,out:0x14  # enc_pokeset
  stub 021a96b8 force ret=none args=v,v,in:0x14  # enc_pokeset
  stub 021aa304 ret=none args=out:0x10,v,v  # in-area
  stub 021aa35c ret=pick:0/1/2/3/4/5/6/0xFF args=v,v,out:1  # in-area
  stub 021aa42c ret=pick:0/1 args=v,v  # in-area
  stub 021aa490 ret=none args=v,in:0x2C,v,v,in:0x28  # in-area
  stub 021aa69c ret=pick:0/2/3/4 args=v  # in-area
  stub 021aa6f8 ret=none args=v,v  # in-area
  stub 021aa728 ret=none args=v,v  # in-area
  stub 021aa778 ret=pick:0/1/2/5/100 args=v  # in-area
  stub 021ab0f0 force ret=u16 args=v  # map_attr
  stub 021ab0f8 force ret=pick:0/1/2/4/6/5/7 args=v  # map_attr
  stub 021ab100 force ret=pick:0/1 args=v  # map_attr
  stub 021ab140 force ret=pick:0/1 args=v  # map_attr
  stub 021ab29c force ret=pick:0/1 args=v  # map_attr
  stub 021ab580 force ret=pick:0/1/4/5/8 args=v  # map_attr

# 021AA304(sit, gd, fieldmap): FIELD SITUATION (header): zone = 02188C9C(fieldmap); +0 = 02013ED8(zone); +4 = 021AB580(021AB0F0(
# 021A30C8(player))); +0xA = zone; 0203F4CC(&time): +0xC hour, +0xD minute; +8 = 021AA69C(fieldmap); +9 = 02012984(gd)
fn 021aa304 ret=none inputs=100
  nowork
  arg buf 0x10
  arg u32 any
  arg u32 any
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02013ed8 force ret=u32 args=v  # zonedata: zone -> battle background
  stub 0203f4cc force ret=none args=out:12  # RTC time (hour, minute, second)
  stub 02188c6c force ret=u32 args=v  # fieldmap -> weather system
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a1bf4 force ret=pick:0/1/2/3/4/5/6/7/0xB/0xC/0xD args=v  # weather id (weather system)
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute word
  stub 021aa69c ret=pick:0/2/3/4 args=v  # in-area
  stub 021ab0f0 force ret=u16 args=v  # map_attr
  stub 021ab580 force ret=pick:0/1/4/5/8 args=v  # map_attr

# 021AA35C(enc, kind, &bonus) -> encounter type or 0xFF: *bonus = 0; a = 021A30C8(player); 021AB100(a) == 0: 0xFF; f = 021AB0F8(a)
# (flags); kind != 1 and !(f & 4): 0xFF; v = 021AB0F0(a); 021AB140(v): *bonus += 10; f & 2 (water): kind 1 -> 4, else 3; kind 1 -> 2;
# 021AB29C(v) -> 1; else 0
fn 021aa35c ret=u32 inputs=100
  nowork
  arg buf 0x800
  arg u32 alt range 0 3 || any
  arg buf 4
  stub 02188c98 force ret=u32 args=v  # fieldmap -> map mapper
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute word
  stub 021ab0f0 force ret=u16 args=v  # map_attr
  stub 021ab0f8 force ret=pick:0/1/2/4/6/5/7 args=v  # map_attr
  stub 021ab100 force ret=pick:0/1 args=v  # map_attr
  stub 021ab140 force ret=pick:0/1 args=v  # map_attr
  stub 021ab29c force ret=pick:0/1 args=v  # map_attr

# 021AA3E0(enc, type, bonus) -> rate: type >= 7 (signed): 0; r = table[type] (u8); r > 0: r + bonus; negative -> 0
fn 021aa3e0 ret=u32 inputs=200
  nowork
  arg buf 0x800
  arg u32 alt range 0 8 || oneof 0xFF 0xFFFFFFFF 0xFFFFFFE0
  arg u32 alt range 0 10 || any
  fix arg0 0xC u32 addrof arg0 0x20

# 021AA3FC(enc, ewk, rate) -> encounter: rate = min(rate, 100); s = 021AA778(ewk) (steps); s == 0: 0; s == 1: rate = 1;
# 021AA6E0() <= rate
fn 021aa3fc ret=u32 inputs=200
  nowork
  arg u32 any
  arg buf 0x10
  arg u32 alt range 0 110 || any
  fix arg1 8 u32 alt range 0 3 || any
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 021aa6e0 ret=pick:0/1/9/10/49/50/98/99 args=  # in-area
  stub 021aa778 ret=pick:0/1/2/5/100 args=v  # in-area

# 021AA42C(ewk, wfbc) -> encounter: s = 021AA778(ewk); f = 021938F4(wfbc); r = 020056EC(100); p = FX: ((s - 1) << 12) * FX_Sqrt(f
# << 12) (64-bit, rounded twice) >> 12; 02010FB0(p) > r
fn 021aa42c ret=u32 inputs=100
  nowork
  arg buf 0x10
  arg u32 any
  fix arg0 8 u32 alt range 0 40 || any
  stub 020056ec force ret=lt:0 args=v  # RNG next(100): below 100
  stub 02010fb0 force ret=u32 args=v  # encounter rate modifier of the pass powers (rate -> rate)
  stub 0207c7b4 force ret=u32 args=v  # FX sqrt / root (fx32)
  stub 021938f4 force ret=u32 args=v  # field_wfbc: encounter level factor (wfbc)
  stub 021aa778 ret=pick:0/1/2/5/100 args=v  # in-area

# 021AA490(enc, efp, setup, heap, mons) (fifth argument on the stack): party = 0201A920(heap); pp = Heap_AllocDebug(heap, 0xDC, 1,
# "field_encount.c", 0x285); for i < 1 + (flag 0x20): 021A96B8(pp, efp, mons + 0x14 i); 0201A98C(party, pp); 021AA304(&sit, gd,
# fieldmap); 0201347C(setup, gd, party, &sit, flag 0x20, heap); type 1: or_flags(setup, 0x20); Heap_Free(pp); Heap_Free(party)
fn 021aa490 ret=none inputs=100
  nowork
  arg buf 0x800
  arg buf 0x30
  arg u32 any
  arg u32 any
  arg buf 0x30
  fix arg1 8 u32 alt range 0 2 || any
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02013144 force ret=none args=v,v  # btl_setup or_flags (setup, mask)
  stub 0201347c force ret=none args=v,v,v,in:16,v,v  # btl_setup setup_with_party (setup, gamedata, party, &field, double, heap)
  stub 02013ed8 force ret=u32 args=v  # zonedata: zone -> battle background
  stub 0201a920 force ret=u32 args=v  # POKEPARTY alloc (heap)
  stub 0201a98c force ret=none args=v,v  # POKEPARTY add (party, pp)
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 0203f4cc force ret=none args=out:12  # RTC time (hour, minute, second)
  stub 02188c6c force ret=u32 args=v  # fieldmap -> weather system
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a1bf4 force ret=pick:0/1/2/3/4/5/6/7/0xB/0xC/0xD args=v  # weather id (weather system)
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute word
  stub 021a96b8 force ret=none args=v,v,in:0x14  # enc_pokeset
  stub 021aa304 ret=none args=out:0x10,v,v  # in-area
  stub 021aa69c ret=pick:0/2/3/4 args=v  # in-area
  stub 021ab0f0 force ret=u16 args=v  # map_attr
  stub 021ab580 force ret=pick:0/1/4/5/8 args=v  # map_attr

# 021AA548(enc, efp, setup, heap, rec) (rec on the stack): party = 0201A920(heap); pp = Heap_AllocDebug((heap & 0x7FFF) | 0x8000,
# 0xDC, 1, "field_encount.c", 0x2A4); 021A9780(pp, efp, rec); 0201A98C(party, pp); 021AA304(&sit, gd, fieldmap); 0201347C(setup, gd,
# party, &sit, 0, heap); or_flags(setup, 8); Heap_Free(pp); Heap_Free(party)
fn 021aa548 ret=none inputs=100
  nowork
  arg buf 0x800
  arg buf 0x30
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 02013144 force ret=none args=v,v  # btl_setup or_flags (setup, mask)
  stub 0201347c force ret=none args=v,v,v,in:16,v,v  # btl_setup setup_with_party (setup, gamedata, party, &field, double, heap)
  stub 02013ed8 force ret=u32 args=v  # zonedata: zone -> battle background
  stub 0201a920 force ret=u32 args=v  # POKEPARTY alloc (heap)
  stub 0201a98c force ret=none args=v,v  # POKEPARTY add (party, pp)
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 020307b0 force ret=none args=v  # Heap_Free(block)
  stub 0203f4cc force ret=none args=out:12  # RTC time (hour, minute, second)
  stub 02188c6c force ret=u32 args=v  # fieldmap -> weather system
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a1bf4 force ret=pick:0/1/2/3/4/5/6/7/0xB/0xC/0xD args=v  # weather id (weather system)
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute word
  stub 021a9780 force ret=none args=v,v,v  # enc_pokeset
  stub 021aa304 ret=none args=out:0x10,v,v  # in-area
  stub 021aa69c ret=pick:0/2/3/4 args=v  # in-area
  stub 021ab0f0 force ret=u16 args=v  # map_attr
  stub 021ab580 force ret=pick:0/1/4/5/8 args=v  # map_attr

# 021AA5CC(enc, setup, kind, partner, tr, tr2, heap) (last three on the stack): trainer battle setup: 020130C4(setup);
# 021AA304(&sit, gd, fieldmap); k = 0202A420(0202A230(tr, 1)); k != 0x11: sit +4 = k; kind 0: tr2 ? 020136E0(setup, gd, &sit, tr, tr2,
# heap) : 02013610(setup, gd, &sit, tr, heap); 1: partner ? 02013734(setup, gd, &sit, partner, tr, tr2, heap) : 02013644(..); 2:
# 02013678; 3: 020136AC; else nothing
fn 021aa5cc ret=none inputs=100
  nowork
  arg buf 0x800
  arg u32 any
  arg u32 alt range 0 4 || any
  arg u32 alt const 0 || any
  arg u32 any
  arg u32 alt const 0 || any
  arg u32 any
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte getter (season)
  stub 020130c4 force ret=none args=v  # btl_setup init (setup)
  stub 02013610 force ret=none args=v,v,in:16,v,v  # btl_setup setup_single_0 (setup, gamedata, &field, trainer, heap)
  stub 02013644 force ret=none args=v,v,in:16,v,v  # btl_setup setup_single_1
  stub 02013678 force ret=none args=v,v,in:16,v,v  # btl_setup setup_single_2
  stub 020136ac force ret=none args=v,v,in:16,v,v  # btl_setup setup_single_3
  stub 020136e0 force ret=none args=v,v,in:16,v,v,v  # btl_setup setup_double (setup, gamedata, &field, trainer, trainer 2, heap)
  stub 02013734 force ret=none args=v,v,in:16,v,v,v,v  # btl_setup setup_triple (setup, gamedata, &field, partner, trainer, trainer 2, heap)
  stub 02013ed8 force ret=u32 args=v  # zonedata: zone -> battle background
  stub 0202a230 force ret=u32 args=v,v  # (trainer data, 1) -> trainer record
  stub 0202a420 force ret=pick:0/2/0x11/0x20 args=v  # trainer record: class / kind (0x11 = none)
  stub 0203f4cc force ret=none args=out:12  # RTC time (hour, minute, second)
  stub 02188c6c force ret=u32 args=v  # fieldmap -> weather system
  stub 02188c9c force ret=pick:0/1/0x3f/0xFFFF args=v  # fieldmap zone id
  stub 02188cb4 force ret=u32 args=v  # fieldmap +0x8C (field player)
  stub 021a1bf4 force ret=pick:0/1/2/3/4/5/6/7/0xB/0xC/0xD args=v  # weather id (weather system)
  stub 021a30c8 force ret=u32 args=v  # field player -> attribute word
  stub 021aa304 ret=none args=out:0x10,v,v  # in-area
  stub 021aa69c ret=pick:0/2/3/4 args=v  # in-area
  stub 021ab0f0 force ret=u16 args=v  # map_attr
  stub 021ab580 force ret=pick:0/1/4/5/8 args=v  # map_attr

# 021AA69C(fieldmap) -> weather class of 021A1BF4(02188C6C(fieldmap)): 2 / 6 / 7 -> 2, 3 / 0xC -> 4, 4 / 5 -> 3, else 0
fn 021aa69c ret=u32 inputs=100
  nowork
  arg u32 any
  stub 02188c6c force ret=u32 args=v  # fieldmap -> weather system
  stub 021a1bf4 force ret=pick:0/1/2/3/4/5/6/7/0xB/0xC/0xD args=v  # weather id (weather system)

# 021AA6E0() -> 020056EC(0xFFFF) / 0x290 (0..99, the rate roll)
fn 021aa6e0 ret=u32 inputs=100
  nowork
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max

# 021AA6F8(ewk, player): reset: m = 021A3074(player); MI_CpuFill8(ewk, 0, 0xC); +0 / +2 / +4 = MMDL gx / gy / gz
fn 021aa6f8 ret=none inputs=100
  nowork
  arg buf 0x10
  arg u32 any
  stub 0216d83c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFF args=v  # MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gz
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL

# 021AA728(ewk, player): count a step: moved (+6): steps +8 < 0xA000: +1; else the grid position differs from +0 / +2 / +4: moved =
# 1, steps + 1
fn 021aa728 ret=none inputs=100
  nowork
  arg buf 0x10
  arg u32 any
  fix arg0 6 u16 range 0 1
  fix arg0 8 u32 alt near 0xA000 2 || range 0 5
  fix arg0 0 u16 alt range 0 3 || const 0xFFFF rep=3,2
  stub 0216d83c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gx
  stub 0216d854 force ret=pick:0/1/2/3/0xFFFF args=v  # MMDL gy
  stub 0216d86c force ret=pick:0/1/2/3/0x1F/0x20/0xFFFF args=v  # MMDL gz
  stub 021a3074 force ret=u32 args=v  # field player -> MMDL

# 021AA778(ewk) -> ewk +8 (steps)
fn 021aa778 ret=u32 inputs=100
  nowork
  arg buf 0x10

