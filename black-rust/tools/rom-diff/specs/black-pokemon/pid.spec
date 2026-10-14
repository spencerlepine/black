# fndiff spec (P6): PID / gender / shininess / ability slot, poke_tool.c of main ARM9
# (0x02017C50-0x02018790, 0x0201AF6C). Run: tools/check-fn.sh black-pokemon pid --per-function
#
# Boundaries (stubbed, each verified by its own area): FUN_02019C54 (personal record handle),
# FUN_0201AE1C (personal field), FUN_02019C7C (ability count), FUN_020056EC (RNG), and the string / move /
# growth callees of poke_tool.spec (never reached by the ids used here). The Pokemon accessors
# FUN_02017DBC/DE4/E64/E88 and the cipher run as real code. In-area callees (ED8, AF6C, F50, 185F8, C50, CC8)
# are ported and run as real code on both sides.
# Records: the BoxPokemon planting recipe of poke_tool.spec (shuffle 0, valid checksum, really encrypted
# blocks). With shuffle 0 the blocks are A 0x08, B 0x28, C 0x48, D 0x68: species u16 @0x08, OT id u32 @0x0C,
# form (bits 3-7) and gender (bits 1-2) @0x40, id 0x71 flag @0x42 bit 0, origin game @0x5F.
# Personal field 0x14 (gender ratio) answers pick the fixed ratios 0 / 0xFE / 0xFF and ordinary values.
area pid
set field

# FUN_02017ED8(ot_id, pid): the four halfwords xor to less than 8.
fn 02017ed8 ret=bool inputs=600
  nowork
  arg u32 any
  arg u32 alt xrel 0 range 0 15 || rel 0 range 0 0x20000 || any

# FUN_0201AF6C(ratio): 1 for 0, 0xFE, 0xFF.
fn 0201af6c ret=bool inputs=300
  nowork
  arg u32 alt oneof 0 0xFE 0xFF 0xFD 1 0x100 0x1FE 0x1FF 0xFFFFFFFE 0xFFFFFFFF || range 0 255 || any

# FUN_02017F50(species, form, pid): 0 male, 1 female, 2 genderless from the ratio and the PID low byte.
fn 02017f50 ret=u8 inputs=600
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt range 0 255 || any
  stub 02019c54 ret=u32 args=v,v
  stub 0201ae1c ret=pick:0/0x1F/0x3F/0x7F/0xBF/0xFE/0xFF/0x80/0xFD/1 args=v,v

# FUN_020185F8(species, form, flag): the PID low byte for the ratio: 8 if fixed, else ratio (minus 1 with flag).
fn 020185f8 ret=u32 inputs=600
  nowork
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || const 1 || any
  stub 02019c54 ret=u32 args=v,v
  stub 0201ae1c ret=pick:0/0x1F/0x3F/0x7F/0xBF/0xFE/0xFF/0x80/0xFD/1 args=v,v

# FUN_0201862C(ot_id, species, form, adjust, ability_bit, flag): PID without the RNG.
fn 0201862c ret=u32 inputs=1200
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || const 1 || const 2 || any
  arg u32 alt const 0 || const 1 || const 2 || any
  arg u32 alt const 0 || const 1 || any
  stub 02019c54 ret=u32 args=v,v
  stub 0201ae1c ret=pick:0/0x1F/0x3F/0x7F/0xBF/0xFE/0xFF/0x80/0xFD/1 args=v,v

# FUN_020186C8(ot_id, species, form, kind, ability_bit, shiny): random PID.
fn 020186c8 ret=u32 inputs=1500
  nowork
  arg u32 alt const 0 || any
  arg u32 any
  arg u32 any
  arg u32 alt const 0 || const 1 || const 2 || any
  arg u32 alt const 0 || const 1 || const 2 || any
  arg u32 alt const 0 || const 1 || const 2 || const 3 || any
  stub 02019c54 ret=u32 args=v,v
  stub 0201ae1c ret=pick:0/0x1F/0x3F/0x7F/0xBF/0xFE/0xFF/0x80/0xFD/1/2/3 args=v,v
  stub 020056ec ret=pick:0/1/2/3/0x10000/0x10001/0x12345678/0xFFFFFFFF/0x80000005 args=v

# FUN_02017EB4(mon): is_shiny(ot id (id 7), pid (id 0)).
fn 02017eb4 ret=bool inputs=600
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0 u32 alt range 0 7 || any
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt range 1 649 || any
  fix region:mon 0x0c u32 alt const 0 || const 0 || any
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=p,v,v
  stub 02045f5c ret=none args=p,v,v,v

# FUN_02017F08(mon): gender from species (id 5), form (0x6F), PID (0).
fn 02017f08 ret=u8 inputs=600
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt range 1 649 || any
  fix region:mon 0x40 u8 any
  fix region:mon 4 u16 alt const 0 || const 0 || const 2 || any
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  stub 02017f50 force ret=pick:0/1/2 args=v,v,v
  stub 02019c54 ret=u32 args=v,v
  stub 0201ae1c ret=pick:0/0x1F/0x3F/0x7F/0xBF/0xFE/0xFF/0x80/0xFD/1 args=v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=p,v,v
  stub 02045f5c ret=none args=p,v,v,v

# FUN_02017C50(mon, species): ability slot (id 10) after a form change.
fn 02017c50 ret=none inputs=900
  nowork
  region mon 0xDC
  arg region mon
  arg u32 any
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt range 1 649 || any
  fix region:mon 0x40 u8 any
  fix region:mon 0x42 u16 alt const 0 || const 1 || any
  fix region:mon 0x5f u8 alt range 0 40 || any
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  stub 02019c54 ret=u32 args=v,v
  stub 0201ae1c ret=pick:0/1/2/3/0x1F/0xFE args=v,v
  stub 02019c7c ret=pick:1/2 args=v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=p,v,v
  stub 02045f5c ret=none args=p,v,v,v

# FUN_02017CC8(mon, old_species, new_species): stored gender (id 0x6E) after a form change.
fn 02017cc8 ret=none inputs=900
  nowork
  region mon 0xDC
  arg region mon
  arg u32 any
  arg u32 any
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt range 1 649 || any
  fix region:mon 0x40 u8 any
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  stub 02017f50 force ret=pick:0/1/2 args=v,v,v
  stub 02019c54 ret=u32 args=v,v
  stub 0201ae1c ret=pick:0/0x1F/0x7F/0xFE/0xFF/0xFD args=v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=p,v,v
  stub 02045f5c ret=none args=p,v,v,v

# FUN_020179B8(mon, form): set the form (id 0x6F); if it changed, refresh ability slot and gender. Returns changed.
fn 020179b8 ret=bool inputs=900
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt const 0 || const 1 || range 0 31 || any
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 0x08 u16 alt range 1 649 || any
  fix region:mon 0x40 u8 alt const 0 || const 8 || any
  fix region:mon 0x42 u16 alt const 0 || const 1 || any
  fix region:mon 0x5f u8 alt range 0 40 || any
  fix region:mon 4 u16 alt const 0 || const 0 || const 2 || any
  fix region:mon 6 u16 alt sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  stub 02017f50 force ret=pick:0/1/2 args=v,v,v
  stub 02019c54 ret=u32 args=v,v
  stub 0201ae1c ret=pick:0/1/2/3/0x1F/0xFE/0xFF args=v,v
  stub 02019c7c ret=pick:1/2 args=v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=p,v,v
  stub 02045f5c ret=none args=p,v,v,v
