# fndiff spec: move_pokemon.c (overlay 10 Thumb code 0x0215E440..0x0215E840, first part of F1 row 10.19; F5b): the roaming
# Pokemon (Tornadus / Thundurus): creation of a roamer record, the respawn of roamers in state 2, the post-battle update, the
# location move on a zone change and the zone / active queries. File bounds: `Heap_AllocDebug(0x8003, 0xDC, 1, "move_pokemon.c",
# 0x12E)` in 0215E440 (string 0x0217293C); field_encount_st.c ends at 0x0215E440 (F5a, area encount_st, which also holds the roamer
# zone-change hooks 0215E358 / 0215E364 / 0215E3A0 and the location -> zone lookup 0215E3E4). The rest of row 10.19
# (0x0215E840..0x0215F168, no __FILE__ string) is the field-skill (HM) table code: area `field_skill`.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field move_pokemon --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry: 9
# entries (0x0215E4C0 is a data word at 0x020A3E4E that decodes as `bl`). Render-only functions: none.
# Closure stubs (as core4 / F4a / F5a): a block lists the stubs of its callees and of the callees of its in-area descendants, so the area
# can be ported in any order (checked with every function treated as ported: no fault, stray write or unstubbed callee). Every
# out-of-area stub is `force`.
#
# ENC SAVE sv (save block 0x38, `0200D418(save)`, save = 02012EBC(GAMEDATA)): roamer i (0, 1) state 0200D468(sv, i) (0 none, 1 roaming,
# 2 defeated: respawn, 3 caught; 0200D470(sv, i, state) sets it), active 0200D488(sv, i), location index 0200D478(sv, i) (16 = hidden),
# 0200D480(sv, i, loc) sets it, record 0200D4AC(sv, i) (0x14 bytes: field getter 0200D4B8(rec, id), setter 0200D50C(rec, id, v): 1 zone,
# 2, 3 PID, 4 species, 5 HP, 6 level, 7 status, 8, 9, 10), 0200D494(rec) clears it.
# Roamers: i 0 = species 0x282 (Thundurus), i 1 = 0x281 (Tornadus), level 40. Moves on every zone change (encount_st): 20 % (cur !=
# 16) hide (loc 16, zone 0xFFFF); otherwise a uniform pick among the group's locations other than the current one and `exclude`,
# group = ROM 0x021711A8[020113D8(season)] (5 x {u16 count, u8 loc[4]}); zone = ROM 0x021711C6[loc] (u16 x 17), loc > 16 -> 0x13F.
# All blocks are `nowork`; objects are `arg buf` arenas or stub answers.
area move_pokemon
set field

# 0215E440(gd, i): create roamer i (i 0: 0x282, 1: 0x281, else nothing): sv = 0200D418(save = 02012EBC(gd)), rec = 0200D4AC(sv, i);
# rec 4 = species, 6 = 40, 9 = 1; mon = Heap_AllocDebug(0x8003, 0xDC, 1, "move_pokemon.c", 0x12E); 02017588(mon, species, 40,
# 02008530(02012944(gd)), 0, -1, 0, -1); rec 7 = 0, 8 = 1, 2 = get(0xAC), 3 = get(0) (PID), 10 = get(0x70), 5 = get(0xA1) (max HP);
# 0200CFA0(0200CA10(save), mon) (pokedex seen); free mon; 0215E740(sv, 02012984(gd), i, 5); 0200D470(sv, i, 1)
fn 0215e440 ret=none inputs=200
  nowork
  arg u32 any
  arg u32 alt range 0 1 || range 0 1 || any
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save
  stub 0200d418 force ret=u32 args=v  # save -> ENC SAVE (block 0x38)
  stub 0200d4ac force ret=u32 args=v,v  # ENC SAVE roamer record i
  stub 0200d50c force ret=none args=v,v,v  # roamer record set (rec, id, v)
  stub 02012944 force ret=u32 args=v  # GAMEDATA -> MYSTATUS
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 02008530 force ret=u32 args=v  # MYSTATUS trainer id
  stub 02017588 force ret=none args=v,v,v,v,v,v,v,v  # PokeParam init (mon, species, level, id, 0, -1, 0, -1)
  stub 02017e1c force ret=u32 args=v,v,v  # PokeParam get (mon, id, 0)
  stub 0200ca10 force ret=u32 args=v  # save -> pokedex
  stub 0200cfa0 force ret=none args=v,v  # pokedex register (dex, mon)
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte (season)
  stub 0215e740 ret=none args=v,v,v,v  # in-area: move roamer (sv, season, i, exclude)
  stub 0200d470 force ret=none args=v,v,v  # ENC SAVE roamer state set (sv, i, state)
  # descendants (0215E740 -> 0215E7DC)
  stub 0200d478 force ret=pick+u8:0/1/2/4/7/10/14/15/16 args=v,v  # ENC SAVE roamer location index
  stub 020113d8 force ret=pick:0/1/2/3/4 args=v  # (season) -> roamer group (RTC)
  stub 02005728 force ret=lt:0 args=v  # RNG next_scaled(n) (black-core rng)
  stub 0215e7dc ret=none args=v,v,v,v  # in-area
  stub 0200d480 force ret=none args=v,v,v  # ENC SAVE roamer location set (sv, i, loc)

# 0215E580(gd): sv = ENC SAVE; i = 0, 1: 0200D468(sv, i) == 2: 0215E440(gd, i) (respawn the defeated roamers)
fn 0215e580 ret=none inputs=200
  nowork
  arg u32 any
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save
  stub 0200d418 force ret=u32 args=v  # save -> ENC SAVE (block 0x38)
  stub 0200d468 force ret=pick+u32:0/1/2/3 args=v,v  # ENC SAVE roamer state (sv, i)
  stub 0215e440 ret=none args=v,v  # in-area: create roamer (gd, i)
  # descendants (0215E440 -> 0215E740 -> 0215E7DC)
  stub 0200d4ac force ret=u32 args=v,v  # ENC SAVE roamer record i
  stub 0200d50c force ret=none args=v,v,v  # roamer record set (rec, id, v)
  stub 02012944 force ret=u32 args=v  # GAMEDATA -> MYSTATUS
  stub 02030734 force ret=u32 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 02008530 force ret=u32 args=v  # MYSTATUS trainer id
  stub 02017588 force ret=none args=v,v,v,v,v,v,v,v  # PokeParam init
  stub 02017e1c force ret=u32 args=v,v,v  # PokeParam get (mon, id, 0)
  stub 0200ca10 force ret=u32 args=v  # save -> pokedex
  stub 0200cfa0 force ret=none args=v,v  # pokedex register (dex, mon)
  stub 020307b0 force ret=none args=v  # Heap_Free
  stub 02012984 force ret=u8 args=v  # GAMEDATA byte (season)
  stub 0215e740 ret=none args=v,v,v,v  # in-area
  stub 0200d470 force ret=none args=v,v,v  # ENC SAVE roamer state set (sv, i, state)
  stub 0200d478 force ret=pick+u8:0/1/2/4/7/10/14/15/16 args=v,v  # ENC SAVE roamer location index
  stub 020113d8 force ret=pick:0/1/2/3/4 args=v  # (season) -> roamer group (RTC)
  stub 02005728 force ret=lt:0 args=v  # RNG next_scaled(n) (black-core rng)
  stub 0215e7dc ret=none args=v,v,v,v  # in-area
  stub 0200d480 force ret=none args=v,v,v  # ENC SAVE roamer location set (sv, i, loc)

# 0215E5B0(gd, setup): wild battle only (setup +4 == 0 and +0 == 0): 02012AC8(020127A4(gd)) (unused); mon = party member 0 of
# 0201313C(setup, 1): species (u16 get 5), hp (u16 get 0xA0), status (u8 get 0x9D); rec = 0215E804(sv, species): none -> 30 %
# (02005728(100) < 30) 0215E364(gd) (every roamer moves); else i = 0215E724(species); result r = setup +0x8C: r 1 / 2 with hp 0 ->
# 0200D494(rec), loc 16, state 2; r 5 -> 0200D494(rec), loc 16, state 3; else rec 5 = hp, 7 = status, 0215E364(gd)
fn 0215e5b0 ret=none inputs=600
  nowork
  arg u32 any
  arg buf 0x100
  fix arg1 0 u32 alt const 0 || const 0 || any
  fix arg1 4 u32 alt const 0 || const 0 || any
  fix arg1 0x8C u32 alt range 0 6 || oneof 1 2 5 || any
  stub 020127a4 force ret=u32 args=v  # GAMEDATA -> PLAYER
  stub 02012ac8 force ret=u16 args=v  # PLAYER u16 +0
  stub 0201313c force ret=u32 args=v,v  # BATTLE_SETUP party (setup, 1)
  stub 0201ab28 force ret=u32 args=v,v  # party member (party, 0)
  stub 02017e1c force ret=pick+u32:0/0x281/0x282/1/0x10000/0x10281 args=v,v,v  # PokeParam get (mon, id, 0)
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save
  stub 0200d418 force ret=u32 args=v  # save -> ENC SAVE (block 0x38)
  stub 0215e804 ret=nullable:2:u32 args=v,v  # in-area: roamer record of species
  stub 02005728 force ret=lt:0 args=v  # RNG next_scaled(100)
  stub 0215e364 force ret=none args=v  # encount_st: every active roamer moves (gd)
  stub 0215e724 ret=pick:0/1/2 args=v  # in-area: roamer index of species
  stub 0200d494 force ret=none args=v  # roamer record clear
  stub 0200d480 force ret=none args=v,v,v  # ENC SAVE roamer location set (sv, i, loc)
  stub 0200d470 force ret=none args=v,v,v  # ENC SAVE roamer state set (sv, i, state)
  stub 0200d50c force ret=none args=v,v,v  # roamer record set (rec, id, v)
  # descendants (0215E804)
  stub 0200d488 force ret=pick:0/1 args=v,v  # ENC SAVE roamer i active
  stub 0200d4ac force ret=u32 args=v,v  # ENC SAVE roamer record i
  stub 0200d4b8 force ret=pick+u32:0x281/0x282/0 args=v,v  # roamer record get (rec, 4 = species)

# 0215E6A8(gd, zone) -> 6 when roamer 0 is active in `zone`, 7 for roamer 1, else 0xFFFF (zone of roamer i: encount_st 0215E3E4)
fn 0215e6a8 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 alt oneof 0x13F 0x141 || any
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save
  stub 0200d418 force ret=u32 args=v  # save -> ENC SAVE (block 0x38)
  stub 0200d488 force ret=pick:0/1 args=v,v  # ENC SAVE roamer i active
  stub 0215e3e4 force ret=pick+u32:0x13F/0x141 args=v,v  # encount_st: zone of roamer i (sv, i)

# 0215E6F8(gd) -> 1 when a roamer is active
fn 0215e6f8 ret=u32 inputs=100
  nowork
  arg u32 any
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save
  stub 0200d418 force ret=u32 args=v  # save -> ENC SAVE (block 0x38)
  stub 0200d488 force ret=pick:0/1 args=v,v  # ENC SAVE roamer i active

# 0215E724(species) -> roamer index: 0x281 -> 1, 0x282 -> 0, else 2
fn 0215e724 ret=u32 inputs=100
  nowork
  arg u32 alt oneof 0x281 0x282 0x280 0x283 || any

# 0215E740(sv, season, i, exclude): cur = 0200D478(sv, i); group = ROM 0x021711A8 + 6 * (u16) 020113D8(season); cur != 16 and
# 02005728(100) < 20: 0215E7DC(sv, i, 16, 0xFFFF); else the group's locations other than cur / exclude, k = 02005728(n), loc = l[k],
# 0215E7DC(sv, i, loc, loc > 16 ? 0x13F : ROM 0x021711C6[loc])
fn 0215e740 ret=none inputs=400
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt range 0 16 || oneof 5 0xFF || any
  stub 0200d478 force ret=pick+u8:0/1/2/4/7/10/14/15/16 args=v,v  # ENC SAVE roamer location index
  stub 020113d8 force ret=pick:0/1/2/3/4 args=v  # (season) -> roamer group (RTC)
  stub 02005728 force ret=lt:0 args=v  # RNG next_scaled(n) (black-core rng)
  stub 0215e7dc ret=none args=v,v,v,v  # in-area: set location (sv, i, loc, zone)
  stub 0200d4ac force ret=u32 args=v,v  # ENC SAVE roamer record i
  stub 0200d480 force ret=none args=v,v,v  # ENC SAVE roamer location set (sv, i, loc)
  stub 0200d50c force ret=none args=v,v,v  # roamer record set (rec, id, v)

# 0215E7DC(sv, i, loc, zone): rec = 0200D4AC(sv, i); 0200D480(sv, i, loc); rec 1 = zone
fn 0215e7dc ret=none inputs=100
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0200d4ac force ret=u32 args=v,v  # ENC SAVE roamer record i
  stub 0200d480 force ret=none args=v,v,v  # ENC SAVE roamer location set (sv, i, loc)
  stub 0200d50c force ret=none args=v,v,v  # roamer record set (rec, id, v)

# 0215E804(sv, species) -> the record of the active roamer i (0, 1) whose species (get 4) is `species`, else 0
fn 0215e804 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 alt oneof 0x281 0x282 || any
  stub 0200d488 force ret=pick:0/1 args=v,v  # ENC SAVE roamer i active
  stub 0200d4ac force ret=u32 args=v,v  # ENC SAVE roamer record i
  stub 0200d4b8 force ret=pick+u32:0x281/0x282/0 args=v,v  # roamer record get (rec, 4 = species)
