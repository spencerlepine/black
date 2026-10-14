# fndiff spec: the encounter Pokemon set-up of overlay 21 (`enc_pokeset`, Thumb code 0x021A9350..0x021A9ED8; F5a-spec): the
# encounter context (EFP) of the lead Pokemon, the rate modifiers, the slot-group selection, the slot pick, the level / form roll,
# the ability effects (repel / Intimidate level filter, Magnet Pull / Static type preference, Pressure max level, Synchronize,
# Cute Charm, Compound Eyes), the swarm and WFBC encounters and the POKEPARAM set-up of a wild or roaming Pokemon.
# No __FILE__ string: the file allocates nothing. F1 counted the range in row 21.28 (field_menu.c, anchor 0x021A84A0); the last
# field_menu function is 021A92A4 (BG load of the menu graphics, ends 0x021A934C), field_encount.c (string at 0x021A9F24) starts
# at 0x021A9ED8 and calls every entry point here, so the range is its own file (the original source: enc_pokeset.c of the Gen 5 field).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field enc_pokeset --per-function
# Set `field` (main 10 11 18 20 21).
# Function list: bl/blx targets of the whole field set, odd code pointers (the slot-pick table ROM 0x021D8DD8), the Ghidra index and a
# Thumb walk from every entry; 27 entries minus 2 false ones (021A939C inside 021A9350 and 021A9C56, the last `pop` of 021A9BE8: data
# words in main 0x020A3EE2 / 0x020A4790 decode as bl) = 25 specced. Ghidra splits 021A9DE0 / 021A9E64 / 021A9E98 into LAB_ pieces and misses 021A9810
# (swarm species; no caller in the field set) and 021A9E64 / 021A9E98. Render-only functions: none.
# Closure stubs (as core4 / F4a / F6a): a block lists the stubs of its callees and of the callees of its in-area descendants, so the area
# can be ported in any order (checked with every function treated as ported: no fault, stray write or unstubbed callee). Every
# out-of-area stub is `force`.
#
# RNG: 021A9DC8 = `020056EC(0xFFFF) / 0x290` (0..99; the global 64-bit LCG MathRandContext32 of black-core rng, `next`), the per-mille
# draws are `02005728(1000)` (`next_scaled`). Both are force stubs here (ported in black-core rng; the ports call them through the seam).
#
# EFP (encounter context, 0x2C bytes on the caller's stack, built by 021A9350): `+0x00` GAMESYS, `+0x04` ENC SAVE (0200D418), `+0x08`
# encounter type (0 grass, 1 dark grass, 2 shaking grass, 3 surf, 4 rippling water, 5 fishing, 6 fishing on a ripple, 0xFF fixed),
# `+0x0C` kind (0 walk, 1 effect encounter, 2 forced (no repel), 3 WFBC; 1 and 2 clear to 0 and set flag 8), `+0x10` slot-pick mode
# (0 land, 1 water, 2 fishing, 3 uniform; index of ROM 0x021D8DD8), `+0x14 u8` slot count (12 / 5 / 1), `+0x15 u8` Pokemon count (1, or
# 2 for a dark-grass double), `+0x16 u16` lead species, `+0x18 u16` lead item, `+0x1A u8` lead is an egg (then the lead fields stay
# 0), `+0x1B u8` lead ability, `+0x1C u8` lead gender, `+0x1D u8` lead nature, `+0x1E u8` lead level, `+0x20` trainer id, `+0x24`
# MYSTATUS, `+0x28` FLAGS: 1 event flag 0x960 (swarms on), 2 fishing type (5 / 6), 4 repel active (ENC SAVE +0x31 != 0), 8 kind 1 / 2
# (no level filter), 0x20 double battle (type 1, >= 2 able Pokemon, roll < 40), 0x40 rate x2 (Illuminate 0x23, Arena Trap 0x47,
# No Guard 0x63), 0x80 rate / 2 (Stench 1, White Smoke 0x49, Quick Feet 0x5F; Sand Veil 8 in weather 3 / 0xC; Snow Cloak 0x51 in
# weather 1 / 4 / 5), 0x100 fishing rate x2 (Suction Cups 0x15, Sticky Hold 0x3C), 0x200 held-item odds (Compound Eyes 0xE), 0x400
# Synchronize 0x1C, 0x800 Cute Charm 0x38 (roll < 67), and with a roll >= 50: 0x1000 Magnet Pull 0x2A (steel 8), 0x2000 Static 9
# (electric 0xC), 0x4000 max level (Pressure 0x2E, Hustle 0x37, Vital Spirit 0x48), 0x8000 Intimidate 0x16 / Keen Eye 0x33.
# MON (0x14 per Pokemon, an array of 1 or 2 on the caller's stack): `+0 u16` species, `+2 u16` item (0 = roll from the personal
# items, 1..0x272 = that item, else none), `+4 u8` level, `+5 u8` form, `+6 u8` shiny mode (2 = no rare check), `+7 u8` special
# (02018A00), `+0x10 u8` gender request (1 male, 2 female).
area enc_pokeset
set field

# 021A9350(efp, gsys, type, kind, weather) (fifth argument on the stack): MI_CpuFill8(efp, 0, 0x2C); efp +0 gsys, +4 0200D418(gd),
# +8 type, +0xC kind; type 5 / 6: flag 2; +0x24 = 02012944(gsys), +0x20 = 02008530(+0x24); lead = 0201AB28(party, 0); +0x1A = egg (0x4C);
# not an egg: +0x16 species (5), +0x18 item (6), +0x1B ability (0xA), +0x1C gender (0x6E), +0x1D nature (02018598), 021A9828(efp,
# weather); flag 1 = check_flag(ew, 0x960); flag 4 = 0200D58C(sv) == 0; kind 2 / 1: flag 8 (kind 2 -> 0); flags 0x10 / 0x20 cleared;
# type 1 with 0201AA50(party) >= 2 and 021A9DC8() < 40: flag 0x20; flags 0x8000 / 4: +0x1E = level (0x9E) of 0201AB28(party,
# 0201AB08(party)); +0x15 = 1 + (flag 0x20)
fn 021a9350 ret=none inputs=300
  nowork
  arg buf 0x30
  arg u32 any
  arg u32 alt const 1 || range 0 6 || oneof 0xFF 7
  arg u32 alt range 0 3 || any
  arg u32 alt range 0 0xD || any
  stub 021a9dc8 ret=pick:0/39/40/99 args=  # in-area
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 02008530 force ret=u32 args=v  # MYSTATUS -> trainer id
  stub 0200d418 force ret=u32 args=v  # (GAMEDATA) -> ENC SAVE (save block 0x38)
  stub 0200d58c force ret=pick:0/1 args=v  # ENC SAVE: repel steps == 0
  stub 02012934 force ret=u32 args=v  # GAMEDATA -> party
  stub 02012944 force ret=u32 args=v  # GAMEDATA -> MYSTATUS (+0x20 of a save block)
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02012ebc force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 02017e1c force ret=pick:0/1/3/8/9/0xE/0x15/0x16/0x1C/0x23/0x2A/0x2E/0x33/0x37/0x38/0x3C/0x47/0x48/0x49/0x51/0x5F/0x63/0xE0/0x140/30/60 args=v,v,v  # POKEPARAM get (pp, id, 0)
  stub 02018598 force ret=u8 args=v  # POKEPARAM nature (pp)
  stub 0201aa50 force ret=pick:0/1/2/3 args=v  # POKEPARTY: count of mons able to battle
  stub 0201ab08 force ret=pick:0/1/5 args=v  # POKEPARTY: first mon able to battle (index)
  stub 0201ab28 force ret=u32 args=v,v  # POKEPARTY mon i (party + 8 + 0xDC i)
  stub 021a9828 ret=none args=v,v  # in-area

# 021A94B0(efp, gsys, rate) -> rate: fishing (flag 2): flag 0x100 doubles it, min(100); else flag 0x40 doubles, flag 0x80 halves;
# lead item 0xE0 (Cleanse Tag) / 0x140 (Pure Incense): 2 * rate / 3 (of the unmodified rate); min(100); 02010FB0(rate) (pass powers)
fn 021a94b0 ret=u32 inputs=100
  nowork
  arg buf 0x30
  arg u32 any
  arg u32 alt range 0 110 || any
  fix arg0 0x18 u16 oneof 0 0xE0 0x140 1
  stub 02010fb0 force ret=u32 args=v  # encounter rate modifier of the pass powers (rate -> rate)

# 021A9508(efp, slots, mons) -> accepted: i < efp +0x15: 021A9C58(efp, slots, mons + 0x14 i); 021A996C(efp, mons[i] level) == 0:
# accepted++
fn 021a9508 ret=u32 inputs=100
  nowork
  arg buf 0x30
  arg buf 0x30
  arg buf 0x30
  fix arg0 0x15 u8 range 0 2
  fix arg0 0x10 u32 range 0 3
  fix arg0 0x14 u8 range 0 12
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max
  stub 02019c90 force ret=pick:0/1/2/3/8/0xC/0x1F args=v,v,v  # personal field (species, form, id): form count 0x20, types 6 / 7
  stub 021a996c ret=pick:0/1 args=v,v  # in-area
  stub 021a9b00 ret=none args=v,v  # in-area
  stub 021a9b68 ret=pick:0/1/4/0xFF args=v,v,v  # in-area
  stub 021a9be8 ret=none args=v,v,v  # in-area
  stub 021a9c58 ret=none args=v,v,v  # in-area
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area
  stub 021a9de0 ret=pick:0/1/2/3/4/5/6/7/8/9/10/11 args=v  # in-area
  stub 021a9e64 ret=pick:0/1/2/3/4 args=v  # in-area
  stub 021a9e98 ret=pick:0/1/2/3/4 args=v  # in-area
  stub 021a9ec4 ret=pick:0/1/2 args=v  # in-area

# 021A954C(table, efp, zone, mons) -> accepted: MI_CpuFill32(0, slots, 0x30) (local); 021A9A5C(table, efp, zone, slots);
# 021A9508(efp, slots, mons)
fn 021a954c ret=u32 inputs=100
  nowork
  arg buf 0x100
  arg buf 0x30
  arg u32 alt oneof 0x13D 0x163 0x183 || any
  arg buf 0x30
  fix arg1 8 u32 alt range 0 6 || oneof 7 0xFF
  fix arg1 0x15 u8 range 0 2
  fix arg1 0x10 u32 range 0 3
  fix arg1 0x14 u8 range 0 12
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max
  stub 0200d418 force ret=u32 args=v  # (GAMEDATA) -> ENC SAVE (save block 0x38)
  stub 0200d454 force ret=pick:0/1/5/0x10/0x11/0xFF args=v  # ENC SAVE +0x30 (swarm index)
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02012ebc force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 02019c90 force ret=pick:0/1/2/3/8/0xC/0x1F args=v,v,v  # personal field (species, form, id): form count 0x20, types 6 / 7
  stub 021a9508 ret=pick:0/1/2 args=in:0x2C,in:0x30,v  # in-area
  stub 021a996c ret=pick:0/1 args=v,v  # in-area
  stub 021a99a8 ret=pick:0/0x021D8DE8/0x021D8E30/0x021D8E68 args=v  # in-area
  stub 021a99e4 ret=pick:0/1 args=v,v,v,v  # in-area
  stub 021a9a5c ret=pick:0/1/5/12 args=v,v,v,out:0x30  # in-area
  stub 021a9b00 ret=none args=v,v  # in-area
  stub 021a9b68 ret=pick:0/1/4/0xFF args=v,v,v  # in-area
  stub 021a9be8 ret=none args=v,v,v  # in-area
  stub 021a9c58 ret=none args=v,v,v  # in-area
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area
  stub 021a9de0 ret=pick:0/1/2/3/4/5/6/7/8/9/10/11 args=v  # in-area
  stub 021a9e64 ret=pick:0/1/2/3/4 args=v  # in-area
  stub 021a9e98 ret=pick:0/1/2/3/4 args=v  # in-area
  stub 021a9ec4 ret=pick:0/1/2 args=v  # in-area

# 021A957C(table, efp, zone) -> roamer record or 0: for i = 0, 1: 0200D488(sv, i) and 0200D4B8(rec = 0200D4AC(sv, i), 1) ==
# zone and 021A996C(efp, 0200D4B8(rec, 6) (level)) == 0: candidate; candidates and 02005728(1000) >= 500: candidate[02005728(n)]
# (table unused)
fn 021a957c ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x30
  arg u32 alt oneof 0 1 2 0x3D 0x13F || any
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max
  stub 0200d488 force ret=pick:0/1 args=v,v  # ENC SAVE roamer i active (+0x16 + 0x14 i)
  stub 0200d4ac force ret=u32 args=v,v  # ENC SAVE roamer record i (sv + 4 + 0x14 i)
  stub 0200d4b8 force ret=pick:0/1/2/5/0x10/0x3D/0x13F/0xFFFF args=v,v  # roamer record field (rec, id 1..10)
  stub 021a996c ret=pick:0/1 args=v,v  # in-area

# 021A9604(wfbc, efp, mon) -> found: column c of the WFBC person record by efp +8: type 0 -> 0, 1 -> 1, 3 -> 2, 0xFF -> 0 (no
# encounter); any other type leaves c = the caller's r5 (0 under the Z19 entry state; kept in the inputs); total = sum of the weights
# (rec +0x14 + c) of the present people (0219390C(wfbc, i), i < 30; rec = 02193C00(person)); r = 020056EC(total); the first person
# whose running sum exceeds r: MI_CpuFill8(mon, 0, 0x14), mon +0 species (rec +8 + 2c), +4 level (rec +0x10 + c), 1; else 0
fn 021a9604 ret=u32 inputs=100
  arg u32 any
  arg buf 0x30
  arg buf 0x20
  fix arg1 8 u32 alt range 0 6 || oneof 0xFF 0xFFFFFFFF
  fix work 0x1014 u32 alt const 0 || any
  stub 020056ec force ret=lt:0 args=v  # RNG next(total): below total
  stub 0219390c force ret=pick:0/1 args=v,v  # field_wfbc: person i present (wfbc, i)
  stub 02193c00 force ret=ptr:work:0x1000:1 args=  # field_wfbc: person record (uses the caller state)

# 021A96B8(pp, efp, mon): per = 0201ADE4(species, form, 4); pid = 021A9D30(efp, per, mon); 02017588(pp, species, level, tid, 0, -1,
# pid, 0); put form (0x6F); mon +7: 02018A00(pp, species, form); item mon +2: 1..0x272 put item (6), 0 -> 021A9CAC(pp, per, type == 1,
# flag 0x200), above 0x272 nothing; put 0x9A = 02008550(efp +0x24), 0x8E = 020084F4(efp +0x24); flag 0x400: nature (0x70) = efp +0x1D;
# 020185C4(pp); 0201AE14(per)
fn 021a96b8 ret=none inputs=100
  nowork
  arg u32 any
  arg buf 0x30
  arg buf 0x20
  fix arg2 2 u16 alt range 0 3 || oneof 0x272 0x273 || any
  fix arg2 7 u8 range 0 1
  fix arg1 8 u32 alt range 0 2 || any
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 020084f4 force ret=u32 args=v  # MYSTATUS getter (copied to POKEPARAM 0x8E)
  stub 02008550 force ret=u32 args=v  # MYSTATUS getter (copied to POKEPARAM 0x9A)
  stub 02017588 force ret=none args=v,v,v,v,v,v,v,v  # POKEPARAM init (pp, species, level, tid, 0, iv -1 / roamer word, pid, 0)
  stub 02017e40 force ret=none args=v,v,v  # POKEPARAM put (pp, id, value)
  stub 020185c4 force ret=none args=v  # POKEPARAM recalc (pp)
  stub 020186c8 force ret=u32 args=v,v,v,v,v,v  # PID for (tid, species, form, gender, 2, shiny mode)
  stub 02018a00 force ret=none args=v,v,v  # POKEPARAM (pp, species, form): hidden ability / special
  stub 0201ade4 force ret=u32 args=v,v,v  # personal open (species, form, heap)
  stub 0201ae14 force ret=none args=v  # personal close
  stub 0201ae1c force ret=pick:0/1/0x7F/0xFE/0xFF/0x101/0x102 args=v,v  # personal field (handle, id): items 0x11..0x13, gender ratio 0x14
  stub 0201af6c force ret=pick:0/1 args=v  # gender ratio fixed (ratio)
  stub 021a9cac ret=none args=v,v,v,v  # in-area
  stub 021a9d30 ret=u32 args=v,v,v  # in-area
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area

# 021A9780(pp, efp, rec): roamer -> POKEPARAM: 02017588(pp, field 4 (u16), field 6 (u8), tid, 0, field 2, field 3, 0); put 0x70 =
# field 10, 0x9D = field 7, 0xA0 = field 5 (0200D4B8(rec, id)); 020185C4(pp)
fn 021a9780 ret=none inputs=100
  nowork
  arg u32 any
  arg buf 0x30
  arg u32 any
  stub 0200d4b8 force ret=pick:0/1/2/5/0x10/0x3D/0x13F/0xFFFF args=v,v  # roamer record field (rec, id 1..10)
  stub 02017588 force ret=none args=v,v,v,v,v,v,v,v  # POKEPARAM init (pp, species, level, tid, 0, iv -1 / roamer word, pid, 0)
  stub 02017e40 force ret=none args=v,v,v  # POKEPARAM put (pp, id, value)
  stub 020185c4 force ret=none args=v  # POKEPARAM recalc (pp)

# 021A9810(gsys) -> swarm species: s = 021A99A8(gsys); s == 0: 0xFFFF; *(u16 *)s (no caller in the field set)
fn 021a9810 ret=u32 inputs=100
  nowork
  arg u32 any
  stub 0200d418 force ret=u32 args=v  # (GAMEDATA) -> ENC SAVE (save block 0x38)
  stub 0200d454 force ret=pick:0/1/5/0x10/0x11/0xFF args=v  # ENC SAVE +0x30 (swarm index)
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02012ebc force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 021a99a8 ret=pick:0/0x021D8DE8/0x021D8E30/0x021D8E68 args=v  # in-area

# 021A9828(efp, weather): the lead ability +0x1B -> flags +0x28 (header table); Cute Charm needs 021A9DC8() < 67, the 0x1000 /
# 0x2000 / 0x4000 / 0x8000 group 021A9DC8() >= 50; Suction Cups / Sticky Hold only with flag 2
fn 021a9828 ret=none inputs=400
  nowork
  arg buf 0x30
  arg u32 alt oneof 1 3 4 5 0xC || range 0 0xD || any
  fix arg0 0x1B u8 alt oneof 1 8 9 0xE 0x15 0x16 0x1C 0x23 0x2A 0x2E 0x33 0x37 0x38 0x3C 0x47 0x48 0x49 0x51 0x5F 0x63 || oneof 0x2B 0x34 0x17 0x10 0 2 0x51 8 || any
  # V-0030: `pick+u16` = 1 in 4 answers any u16 (the whole next(0xFFFF) range 0..0xFFFE as the old `lt:0`, plus 0xFFFF, which
  # / 0x290 is roll 99 like 65534), the rest the boundary list
  stub 020056ec force ret=pick+u16:0/20000/32799/32800/43295/43296/43951/43952/65534 args=v  # RNG next(0xFFFF), only through 021A9DC8 (/ 0x290): rolls 0 / 30 / 49 / 50 / 65 / 66 / 66 / 67 / 99 (the Cute Charm < 67 and >= 50 thresholds)
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area

# 021A996C(efp, level) -> rejected: flag 8: 0; flag 0x8000 and level + 5 <= lead level: 1; flag 4 without flag 2 and level < lead
# level: 1 (repel); 0
fn 021a996c ret=u32 inputs=200
  nowork
  arg buf 0x30
  arg u32 alt range 0 100 || any
  fix arg0 0x1E u8 range 0 100

# 021A99A8(gsys) -> swarm record or 0: check_flag(ew, 0x960) == 0: 0; k = 0200D454(sv) (k >= 17 -> 0); ROM 0x021D8DE8 + 8k
# (17 records: u16 zone, u16 species, u8 form, u8 min, u8 max, u8 0)
fn 021a99a8 ret=u32 inputs=100
  nowork
  arg u32 any
  stub 0200d418 force ret=u32 args=v  # (GAMEDATA) -> ENC SAVE (save block 0x38)
  stub 0200d454 force ret=pick:0/1/5/0x10/0x11/0xFF args=v  # ENC SAVE +0x30 (swarm index)
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02012ebc force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)

# 021A99E4(table, efp, zone, slot) -> swarm: s = 021A99A8(efp +0 (gsys)); s == 0 or s.zone != zone: 0; efp +8 != 0: 0;
# 02005728(1000) > 400: 0; slot = (s.species | s.form << 11, s.min, s.max); efp +0x10 = 3, +0x14 = 1; 1 (table unused)
fn 021a99e4 ret=u32 inputs=100
  nowork
  arg u32 any
  arg buf 0x30
  arg u32 alt oneof 0x13D 0x163 0x183 || any
  arg buf 0x10
  fix arg1 8 u32 alt oneof 0 0 1 || any
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max
  stub 0200d418 force ret=u32 args=v  # (GAMEDATA) -> ENC SAVE (save block 0x38)
  stub 0200d454 force ret=pick:0/1/5/0x10/0x11/0xFF args=v  # ENC SAVE +0x30 (swarm index)
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02012ebc force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 021a99a8 ret=pick:0/0x021D8DE8/0x021D8E30/0x021D8E68 args=v  # in-area

# 021A9A5C(table, efp, zone, slots) -> slot count: 021A99E4(table, efp, zone, slots): efp +0x14; type +8 >= 7 (signed) -> 0;
# type 0 / 1 / 2: 12 slots at table +8 / +0x38 / +0x68, mode 0; 3 / 4: 5 at +0x98 / +0xAC, mode 1; 5 / 6: 5 at +0xC0 / +0xD4, mode
# 2; efp +0x10 = mode, +0x14 = n; MI_CpuCopy8(group, slots, 4n); n (a negative type copies 0 bytes from the caller's r4)
fn 021a9a5c ret=u32 inputs=300
  nowork
  arg buf 0x100
  arg buf 0x30
  arg u32 alt oneof 0x13D 0x183 || any
  arg buf 0x30
  fix arg1 8 u32 alt const 0 || range 0 6 || oneof 7 0xFF 0xFFFFFFFF
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max
  stub 0200d418 force ret=u32 args=v  # (GAMEDATA) -> ENC SAVE (save block 0x38)
  stub 0200d454 force ret=pick:0/1/5/0x10/0x11/0xFF args=v  # ENC SAVE +0x30 (swarm index)
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02012ebc force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 020142e8 force ret=pick:0/1 args=v,v  # event_work check_flag (ew, flag)
  stub 021a99a8 ret=pick:0/0x021D8DE8/0x021D8E30/0x021D8E68 args=v  # in-area
  stub 021a99e4 ret=pick:0/1 args=v,v,v,v  # in-area

# 021A9B00(slot, mon): MI_CpuFill8(mon, 0, 0x14); level = min + 021A9DC8() % (max(max - min, 0) + 1); species = slot & 0x7FF;
# n = 02019C90(species, 0, 0x20) (form count); form = slot >> 11: 0x1F -> 02005728(n), >= n -> 0, else form
fn 021a9b00 ret=none inputs=100
  nowork
  arg buf 0x10
  arg buf 0x20
  fix arg0 0 u16 alt oneof 0xF801 0x0801 0x1001 0x2001 || any
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max
  stub 02019c90 force ret=pick:0/1/2/3/8/0xC/0x1F args=v,v,v  # personal field (species, form, id): form count 0x20, types 6 / 7
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area

# 021A9B68(efp, slots, type) -> slot index or 0xFF: the slots i < efp +0x14 (not form 0x1F) whose personal type 1 or 2
# (02019C90(species, form, 6 / 7)) is `type`; none: 0xFF; else one of them by 021A9DC8() % n
fn 021a9b68 ret=u32 inputs=100
  nowork
  arg buf 0x30
  arg buf 0x30
  arg u32 alt oneof 8 0xC || any
  fix arg0 0x14 u8 range 0 12
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 02019c90 force ret=pick:0/1/2/3/8/0xC/0x1F args=v,v,v  # personal field (species, form, id): form count 0x20, types 6 / 7
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area

# 021A9BE8(efp, slots, mon): flag 0x4000: m = max(mon level, max level of the slots i < efp +0x14 with mon's species); the
# slots' min / max differ somewhere and mon level + 5 <= m: level += 5; else level = m
fn 021a9be8 ret=none inputs=100
  nowork
  arg buf 0x30
  arg buf 0x30
  arg buf 0x20
  fix arg0 0x14 u8 range 0 12
  fix arg1 0 u16 range 1 2 rep=12,4
  fix arg1 2 u8 range 1 20 rep=12,4
  fix arg1 3 u8 range 1 30 rep=12,4
  fix arg2 0 u16 range 1 2
  fix arg2 4 u8 range 0 30

# 021A9C58(efp, slots, mon): flag 0x1000 (Magnet Pull): i = 021A9B68(efp, slots, 8); else flag 0x2000 (Static): 021A9B68(efp,
# slots, 0xC); none (0xFF): i = ROM 0x021D8DD8[efp +0x10](efp +0x14) (021A9DE0 land / 021A9E64 water / 021A9E98 fishing / 021A9EC4
# uniform); 021A9B00(slots + 4i, mon); 021A9BE8(efp, slots, mon)
fn 021a9c58 ret=none inputs=100
  nowork
  arg buf 0x30
  arg buf 0x30
  arg buf 0x20
  fix arg0 0x10 u32 range 0 3
  fix arg0 0x14 u8 range 1 12
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 02005728 force ret=lt:0 args=v  # RNG: global 64-bit LCG next_scaled(max) (black-core rng 02005728): below max
  stub 02019c90 force ret=pick:0/1/2/3/8/0xC/0x1F args=v,v,v  # personal field (species, form, id): form count 0x20, types 6 / 7
  stub 021a9b00 ret=none args=v,v  # in-area
  stub 021a9b68 ret=pick:0/1/4/0xFF args=v,v,v  # in-area
  stub 021a9be8 ret=none args=v,v,v  # in-area
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area
  stub 021a9de0 ret=pick:0/1/2/3/4/5/6/7/8/9/10/11 args=v  # in-area
  stub 021a9e64 ret=pick:0/1/2/3/4 args=v  # in-area
  stub 021a9e98 ret=pick:0/1/2/3/4 args=v  # in-area
  stub 021a9ec4 ret=pick:0/1/2 args=v  # in-area

# 021A9CAC(pp, per, wide, eyes): items a, b, c = personal 0x11 / 0x12 / 0x13; a == b: put item (6) a; else odds ROM 0x021D8DCC[wide
# + 2 eyes] (3 x 3 bytes: 50 / 5 / 0, 60 / 20 / 0 with Compound Eyes; wide = type 1): the first item whose running sum exceeds
# 021A9DC8()
fn 021a9cac ret=none inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 range 0 1
  arg u32 range 0 1
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 02017e40 force ret=none args=v,v,v  # POKEPARAM put (pp, id, value)
  stub 0201ae1c force ret=pick:0/1/0x7F/0xFE/0xFF/0x101/0x102 args=v,v  # personal field (handle, id): items 0x11..0x13, gender ratio 0x14
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area

# 021A9D30(efp, per, mon) -> PID: g = 2 (random); 0201AF6C(personal 0x14) != 1 (not fixed): flag 0x800 (Cute Charm): lead gender 0 -> 1, 1
# -> 0; mon +0x10 1 -> 0, 2 -> 1; pid = 020186C8(tid, species, form, g, 2, mon +6 == 2 ? 0 : 2); mon +6 != 2: bit 31 = parity of
# (pid ^ tid ^ tid >> 16) & 1 (16-bit halves)
fn 021a9d30 ret=u32 inputs=100
  nowork
  arg buf 0x30
  arg u32 any
  arg buf 0x20
  fix arg2 0x10 u8 range 0 3
  fix arg2 6 u8 range 1 3
  fix arg0 0x1C u8 range 0 2
  stub 020186c8 force ret=u32 args=v,v,v,v,v,v  # PID for (tid, species, form, gender, 2, shiny mode)
  stub 0201ae1c force ret=pick:0/1/0x7F/0xFE/0xFF/0x101/0x102 args=v,v  # personal field (handle, id): items 0x11..0x13, gender ratio 0x14
  stub 0201af6c force ret=pick:0/1 args=v  # gender ratio fixed (ratio)

# 021A9DC8() -> 020056EC(0xFFFF) / 0x290 (0..99)
fn 021a9dc8 ret=u32 inputs=100
  nowork
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max

# 021A9DE0(n) -> land slot by 021A9DC8(): < 20: 0, < 40: 1, < 50: 2, < 60: 3, < 70: 4, < 80: 5, < 85: 6, < 90: 7, < 94: 8,
# < 98: 9, 98: 10, else 11 (20 / 20 / 10 / 10 / 10 / 10 / 5 / 5 / 4 / 4 / 1 / 1 percent)
fn 021a9de0 ret=u32 inputs=800
  nowork
  arg u32 any
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area

# 021A9E64(n) -> water slot: < 60: 0, < 90: 1, < 95: 2, < 99: 3, else 4 (60 / 30 / 5 / 4 / 1)
fn 021a9e64 ret=u32 inputs=800
  nowork
  arg u32 any
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area

# 021A9E98(n) -> fishing slot: < 40: 0, < 80: 1, < 95: 2, < 99: 3, else 4 (40 / 40 / 15 / 4 / 1)
fn 021a9e98 ret=u32 inputs=800
  nowork
  arg u32 any
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area

# 021A9EC4(n) -> 021A9DC8() % n (uniform; mode 3, the swarm's single slot)
fn 021a9ec4 ret=u32 inputs=100
  nowork
  arg u32 range 1 12
  stub 020056ec force ret=lt:0 args=v  # RNG: global 64-bit LCG next(max) (black-core rng 020056EC): below max
  stub 021a9dc8 ret=pick:0/19/20/39/40/49/50/59/60/66/67/69/70/79/80/84/85/89/90/93/94/97/98/99 args=  # in-area

