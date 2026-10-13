# fndiff spec: the Pokemon icon helpers (P4), main ARM9 0x0201BA68-0x0201BD28, between shinka_check.c and waza_tool.c
# (T1 counted them as waza_tool: Ghidra merged 0x0201BC40.. with FUN_0201BD28, hence "FUN_0201BC40 loads ARC 21").
# Run: tools/check-fn.sh black-pokemon icon --per-function
#
# ARC 7 (Pokemon icons, 1431 members): member 0 = the palette file (0201BC3C), 1..6 = cell / animation files picked by the
# OBJ VRAM mapping mode in DISPCNT (0201BC40 / BC78 / BCB4 / BCEC), 7.. = the icon graphics (0201BABC): species * 2 + 7
# (+1 = the female variant if that member is non-empty), eggs at (0x28A + (species == 490 Manaphy)) * 2 + 7, alternate forms
# at 0x51F + (first-form index + form - 1) * 2. The palette of an icon (0201BB5C) is a nibble of the 712-byte table at main
# 0x0209E474 (index species / 0x28A + Manaphy / 0x28B + first-form index + form; low nibble, high nibble when r2 != 0).
# The personal fields read through FUN_02019C90(species, 0, field) (area personal, a `force` stub here): 0x1F first-form
# index, 0x23 (nonzero = forms share the base icon), 0x20 form count. The stub's answers are small (pick) so the table
# index stays inside the 712 bytes.
area icon
set field

# FUN_0201BC3C(): the palette member of ARC 7 (0).
fn 0201bc3c ret=u32 inputs=10
  nowork

# FUN_0201BC40(): cell member for the MAIN engine's OBJ mapping (REG_DISPCNT 0x04000000 & 0x00300010): 0x10 (1D 32K) -> 2,
# 0x100010 (64K) -> 4, 0x200010 (128K) -> 6, anything else -> 6. The register is a `global` (the harness maps the page).
fn 0201bc40 ret=u32 inputs=300
  nowork
  global 04000000 4
  fix global0 0 u32 alt const 1 || const 1 || const 0 bits=4:1
  fix global0 0 u32 range 0 3 bits=20:2

# FUN_0201BC78(): the same for the SUB engine (REG_DISPCNT_SUB 0x04001000).
fn 0201bc78 ret=u32 inputs=300
  nowork
  global 04001000 4
  fix global0 0 u32 alt const 1 || const 1 || const 0 bits=4:1
  fix global0 0 u32 range 0 3 bits=20:2

# FUN_0201BCB4(): animation member, main engine: 1 / 3 / 5 / 5.
fn 0201bcb4 ret=u32 inputs=300
  nowork
  global 04000000 4
  fix global0 0 u32 alt const 1 || const 1 || const 0 bits=4:1
  fix global0 0 u32 range 0 3 bits=20:2

# FUN_0201BCEC(): animation member, sub engine: 1 / 3 / 5 / 5.
fn 0201bcec ret=u32 inputs=300
  nowork
  global 04001000 4
  fix global0 0 u32 alt const 1 || const 1 || const 0 bits=4:1
  fix global0 0 u32 range 0 3 bits=20:2

# FUN_0201BABC(species, form, female, egg): the icon graphics member (see the header). `female` 1 adds 1 when
# ArcMemberSize(7, member + 1) (FUN_020490A8) is nonzero, 0 and 2 add nothing, any other value is added as it is.
fn 0201babc ret=u32 inputs=1500
  nowork
  arg u32 alt range 0 711 || oneof 490 489 650 || any
  arg u32 alt const 0 || range 0 31 || range 1 3 || any
  arg u32 alt const 0 || const 1 || const 2 || any
  arg u32 alt const 0 || const 0 || const 1 || any
  stub 02019c90 force ret=pick:0/0/1/2/3/5/10/30 args=v,v,v
  stub 020490a8 ret=pick:0/1 args=v,v

# FUN_0201BB5C(species, form, high, egg): the icon palette (the table nibble; see the header). The table is a `global`
# (random contents); species and the stub answers keep the index below 712.
fn 0201bb5c ret=u32 inputs=1500
  nowork
  global 0209e474 0x2c8
  arg u32 alt range 0 711 || oneof 490 489 650 || range 0 649
  arg u32 alt const 0 || range 0 31 || range 1 3 || any
  arg u32 alt const 0 || const 1 || any
  arg u32 alt const 0 || const 0 || const 1 || any
  stub 02019c90 force ret=pick:0/0/1/2/3/5/10/30 args=v,v,v

# FUN_0201BA68(mon): FUN_0201BABC(species (5), form (0x6F), gender (0x6E), egg (0x4C)) of a BoxPokemon, read between
# decrypt-on-access (FUN_02017DBC) and FUN_02017DE4 (poke_tool, real code). The species is planted at the start of all
# four blocks, so block A has one whatever the PID's shuffle; a bad egg reads as 0x28A.
fn 0201ba68 ret=u32 inputs=1000
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0x0 u32 alt const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 711 || oneof 490 650 || range 0 711 rep=4,0x20
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  stub 02019c90 force ret=pick:0/0/1/2/3/5/10/30 args=v,v,v
  stub 020490a8 ret=pick:0/1 args=v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v

# FUN_0201BBE8(mon): FUN_0201BB5C(species, form, gender, egg) of a BoxPokemon (as 0201BA68, palette table as a global).
fn 0201bbe8 ret=u32 inputs=1000
  nowork
  global 0209e474 0x2c8
  region mon 0xDC
  arg region mon
  fix region:mon 0x0 u32 alt const 0 || any bits=13:5
  fix region:mon 0x4 u16 alt const 0 || const 0 || const 0 || const 3 || const 1 || const 4 || any
  fix region:mon 0x8 u16 alt range 1 711 || oneof 490 650 || range 0 711 rep=4,0x20
  fix region:mon 0x6 u16 alt sum16 0x8 0x80 || sum16 0x8 0x80 || any
  crypt region:mon 0x8 0x80 u16@0x6 skipif=u16@0x4&2
  stub 02019c90 force ret=pick:0/0/1/2/3/5/10/30 args=v,v,v
  stub 02017f50 ret=pick:0/1/2 args=v,v,v
