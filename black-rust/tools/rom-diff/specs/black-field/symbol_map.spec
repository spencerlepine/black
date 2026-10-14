# fndiff spec: symbol_map.c (overlay 10 Thumb code 0x021664CC..0x02166F9C, F1 row 10.27; F5b): the functions of the file that are
# not script-command handlers. The handlers and their in-row helpers (17 functions: 02166658, 0216666C, 021666B8, 0216670C, 02166714,
# 02166734, 021667E0, 02166824, 02166834, 02166854, 02166908, 02166980, 021669C0, 02166A00, 02166D30, 02166E50, 02166F1C) are
# black-script `scrcmd_misc10b` (F8-spec5). File bound: `Heap_AllocDebug(heap, 0x54, 1, "symbol_map.c", 0x1C9 / 0x1DC)` in 021664CC
# (string 0x021729EC); game_beacon_accessor.c (X1) ends at 0x021664C6, event_research_radar.c starts at 0x02166F9C.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-field symbol_map --per-function
#
# Function list: bl/blx targets of the whole field set, odd code pointers, the Ghidra index and a Thumb walk from every entry; 31
# entries, minus the 17 above = 14, minus 3 render-only functions (U-phase, D3; not specced): 02166A6C (V-blank callback: a soft-float
# fade of the global 0x02172BA4 written into REG_BG2PA area 0x04000020 via 0207AD58 / 0207D59C), 02166B0C (starts it: 0x02172BA4 =
# {1.0f, 0}, 02005654(0x02166A6D) = V-blank callback set), 02166B2C (resets the register (0207AD40 / 0207D59C) and the callback
# 0200566C) = 11 specced. Fragments (not functions): 0x02166DFE (a label inside 02166D30), 0x021666C8 / 0x021666D0 (inside 021666B8).
# D9: the Entralink / Entree Forest side (the intrude state 021767A0, the comm record 02158344 / 02176880 of overlay 11, game_beacon
# 0216649C / 02166324 / 021663F4 / 0216559C) is reached only through force stubs; the local logic (record bit packing, the three
# symbol-kind predicates, the zone remap, the special flags) is ported.
# Closure stubs (as core4 / F4a / F5a): a block lists the stubs of its callees and of the callees of its in-area descendants (checked with
# every function treated as ported: no fault, stray write or unstubbed callee). Every out-of-area stub is `force`.
#
# SYMBOL RECORD (0x54, 021664CC): filled by 0216749C (own game) or copied from the comm record 02176880 (another player's forest);
# `+0x50 u32` bits 0..5 = 0216749C's kind byte, 6..9 = 021673C4(Y) & 0xF, 10..13 = 021673A8(Y) & 0xF, 14..15 = 0, 16..17 = 1,
# 18..23 = 02012FB0(GD) & 0x3F, 24..31 kept (Y = 0200E0B0(save), the symbol save block).
# HATCH EVENT work (0x18, 02166C4C): +0 GAMESYS, +4 GAMEDATA, +8 FIELDMAP, +0xC the Pokemon, +0x10 / +0x14 the app parameter
# {GAMEDATA, Pokemon} of app 0xD7 (0x021F7118).
# All blocks are `nowork`; objects are `arg buf` arenas or stub answers.
area symbol_map
set field

# 021664CC(heap, gsys, out) -> record or 0: I = 02012138(gsys); 021767A0(I, gd) == 4: rec = Heap_AllocDebug(heap, 0x54, 1,
# "symbol_map.c", 0x1C9), *out = 0216749C(Y, rec, 20, 02012FB0(gd), &k), rec +0x50 bit fields (header); else p = 02158344(I): 0 -> 0;
# rec = Heap_AllocDebug(heap, 0x54, 1, "symbol_map.c", 0x1DC), MI_CpuCopy8(02176880(p), rec, 0x54), *out = 0x212. Arena: arg 3 (rec at
# 0, the comm record at 0x80).
fn 021664cc ret=u32 inputs=300
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  arg buf 0x100
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save
  stub 0200e0b0 force ret=u32 args=v  # save -> symbol save Y
  stub 02012138 force ret=u32 args=v  # GAMESYS -> intrude / comm system I
  stub 021767a0 force ret=pick+u32:4/0/3 args=v,v  # ov11 (X1): intrude state (I, gd)
  stub 02012fb0 force ret=pick+u8:0/1/0x3F/0x40/0x41 args=v  # GAMEDATA u8 +0x1CF
  stub 02030734 force ret=ptr:arg3:0:1 args=v,v,v,v,v  # Heap_AllocDebug(heap, size, flag, file, line)
  stub 0216749c force ret=u32 args=v,v,v,v,outm:1:0xFF  # research radar (F10): fill the record (Y, rec, 20, x, &k)
  stub 021673c4 force ret=pick+u32:0/1/0xF/0x10/0x1F args=v  # (Y) symbol field
  stub 021673a8 force ret=pick+u32:0/1/0xF/0x10/0x1F args=v  # (Y) symbol field
  stub 02158344 force ret=nullable:3:u32 args=v  # comm record (I) (X1)
  stub 02176880 force ret=ptr:arg3:0x80:1 args=v  # ov11 (X1): the record's data (p)

# 021665D0(x, a) -> p = 0216649C(x); p ? 02166324(p, (u8)(a + 1)) : 0x118
fn 021665d0 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 alt range 0 0x100 || any
  stub 0216649c force ret=nullable:3:u32 args=v  # game_beacon_accessor (X1)
  stub 02166324 force ret=u32 args=v,v  # game_beacon_accessor (X1)

# 021665F0(x, a, b) -> p = 0216649C(x); p ? (u8) 021663F4(p, (u8)(a + 1), (u8) b) - 1 : 4
fn 021665f0 ret=u32 inputs=100
  nowork
  arg u32 any
  arg u32 alt range 0 0x100 || any
  arg u32 alt range 0 0x100 || any
  stub 0216649c force ret=nullable:3:u32 args=v  # game_beacon_accessor (X1)
  stub 021663f4 force ret=pick+u32:0/1/2/0x100 args=v,v,v  # game_beacon_accessor (X1)

# 02166618(x) -> (u8)(x + 1) == 1; 0216662C(x) -> (u8)(x + 1) == 5; 02166640(x) -> (u8)((u8)(x + 1) - 2) <= 2 (symbol kinds)
fn 02166618 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 8 || oneof 0xFF 0x100 0x1FF || any

fn 0216662c ret=u32 inputs=100
  nowork
  arg u32 alt range 0 8 || oneof 0xFF 0x104 0x1FF || any

fn 02166640 ret=u32 inputs=100
  nowork
  arg u32 alt range 0 8 || oneof 0xFF 0x101 0x103 0x104 || any

# 021666A4(p): *p != 0 -> Heap_Free(*p), *p = 0
fn 021666a4 ret=none inputs=60
  nowork
  arg buf 0x10
  fix arg0 0 u32 alt const 0 || any
  stub 020307b0 force ret=none args=v  # Heap_Free

# 02166B58(ev, seq, W) GMEVENT proc 0x02166B59 (W in the header): 0: not an egg (get 0x4C == 0) or get 3 == 1 -> seq 7, else seq 1;
# 1: W+0x10 = gd, W+0x14 = mon, seq 2; 2: call 021BE8D4(gsys, fm, 0, 0), seq 3; 3: call 020144DC(gsys, fm), seq 4; 4: 020120B4(gsys,
# 0xD7, 0x021F7118, W+0x10) (app), seq 5; 5: call 020145CC(gsys), seq 6; 6: call 021BE92C(gsys, fm, 0, 0, 1, 0, 0), 0216559C((u16) get
# 5), seq 7; 7: -> 1; other -> 0 (every step returns 0)
fn 02166b58 ret=u32 inputs=300
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x40
  fix arg1 0 u32 alt range 0 7 || range 0 8 || any
  stub 02017e1c force ret=pick+u32:0/1/2 args=v,v,v  # PokeParam get (mon, id, 0)
  stub 021be8d4 force ret=u32 args=v,v,v,v  # (gsys, fm, 0, 0) -> event (fade out)
  stub 02012374 force ret=none args=v,v  # GMEVENT call (event, child)
  stub 020144dc force ret=u32 args=v,v  # (gsys, fm) -> event
  stub 020120b4 force ret=none args=v,v,v,v  # app proc request (gsys, overlay, proc data, param)
  stub 020145cc force ret=u32 args=v  # (gsys) -> event
  stub 021be92c force ret=u32 args=v,v,v,v,v,v,v  # (gsys, fm, 0, 0, 1, 0, 0) -> event (fade in)
  stub 0216559c force ret=none args=v  # game_beacon (X1): hatched (species)

# 02166C4C(gsys, fm, mon) -> ev = 020122C0(gsys, 0, 0x02166B59, 0x18); W +0 gsys, +8 fm, +4 020120F4(gsys), +0xC mon
fn 02166c4c ret=u32 inputs=60
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg buf 0x40
  stub 020122c0 force ret=u32 args=v,v,v,v  # GMEVENT create (gsys, parent, proc, work size)
  stub 020124e8 force ret=ptr:arg3:0:1 args=v  # GMEVENT work (event)
  stub 020120f4 force ret=u32 args=v  # GAMESYS -> GAMEDATA

# 02166C7C(gd, zone) -> z = 02013E40(zone); z == 0x1A6: 02013E40(u16 *(02012894(gd) + 4)); else ROM 0x02171F2C {0x88 <- 0x108,
# 0xF0 <- 0xF1}: the mapped value (u16), else z. Arena: arg 2 (the 02012894 record).
fn 02166c7c ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 any
  arg buf 0x10
  stub 02013e40 force ret=pick+u32:0x1A6/0x108/0xF1/0x88/0xF0/0x10108 args=v  # zonedata (zone)
  stub 02012894 force ret=ptr:arg2:0:1 args=v  # GAMEDATA -> LOCATION

# 02166CD0(gd, flag) -> E = 0201296C(gd), x = 02012F08(gd), y = 02012944(gd), s = 02009470(02012EBC(gd)); flag 0xF000 -> 0216BF9C(x,
# 1, y); 0xF001 -> 02009590(s, y, 0) != 0; else 020142E8(E, flag)
fn 02166cd0 ret=u32 inputs=200
  nowork
  arg u32 any
  arg u32 alt oneof 0xF000 0xF001 0xEFFF 0xF002 || any
  stub 0201296c force ret=u32 args=v  # GAMEDATA -> EVENT WORK
  stub 02012f08 force ret=u32 args=v  # GAMEDATA -> save block
  stub 02012944 force ret=u32 args=v  # GAMEDATA -> MYSTATUS
  stub 02012ebc force ret=u32 args=v  # GAMEDATA -> save
  stub 02009470 force ret=u32 args=v  # save -> record block
  stub 0216bf9c force ret=u32 args=v,v,v  # namein_setup (F8): (x, 1, y)
  stub 02009590 force ret=pick+u32:0/1/2 args=v,v,v  # (s, y, 0) count
  stub 020142e8 force ret=pick:0/1 args=v,v  # event flag check (ew, flag)
