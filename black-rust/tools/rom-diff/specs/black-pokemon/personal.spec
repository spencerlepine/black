# fndiff spec: personal data access and the stat calculation (P2), main ARM9 code.
# poke_personal.c routines of 0x0201AD88-0x0201B2B0 and 0x02019C54-0x02019CA0, T3-FINDINGS.md (a).
# Run: tools/check-fn.sh black-pokemon personal --per-function
#
# Model of the personal-data archive (ARC 016, one 0x3C-byte record per species/form record index):
#  * the open handle is the word at 0x0214652C (a `global`, random);
#  * the member reads FUN_02049578(handle, index) -> offset, FUN_0204960C(handle, offset) (seek),
#    FUN_020495F4(handle, 0x3C, dst) (read) are stubbed: the read fills the 0x3C bytes with random bytes
#    (`outm`), except that the alternate-form fields (u16 @0x1C, u8 @0x20) are kept in a range where
#    both the "no alternate forms" and the "form record" answers of FUN_0201AFD4 happen;
#  * the record cache is the global 0x02146270: u16 form, u16 species, then the 0x3C-byte record at
#    0x02146274 (what FUN_02019C54 returns a pointer to).
# Record fields (FUN_0201AE1C ids 0..0x2B): 0-5 base stats HP Atk Def Spe SpA SpD, 6/7 types, 8 catch
# rate, 9 u16 @0x22, 0xA-0x10 2-bit EV yields, 0x11-0x13 items, 0x14 gender ratio, 0x15 hatch steps,
# 0x16 base friendship (the egg-cycle and friendship names are inferred), 0x17 growth curve id,
# 0x18/0x19 egg groups, 0x1A/0x1B/0x1C abilities, 0x1D flee rate, 0x1E/0x1F, 0x20-0x24 misc,
# 0x25/0x26 u16, 0x27-0x2A u32 (TM/HM bitfield words), 0x2B u32 (tutor bitfield).
area personal
set field

# FUN_0201AE1C(rec, field): the field getter (switch over 44 ids, anything above returns 0).
fn 0201ae1c ret=u32 inputs=800
  nowork
  region rec 0x3C
  arg region rec
  arg u32 alt range 0 0x2B || range 0 0x2B || range 0 0x2B || range 0 0x30 || oneof 0x2c 0x2d 0xff 0x100 0xFFFFFFFF
  fix region:rec 0x21 u8 alt const 0 || const 0xff || const 0x41 || const 0x82 || any

# FUN_02019C7C(rec): 2 if field 0x1B (second ability) is nonzero, else 1.
fn 02019c7c ret=u32
  nowork
  region rec 0x3C
  arg region rec
  fix region:rec 0x19 u8 alt const 0 || any

# FUN_0201AFD4(species, form): personal record index. 0 for species 0x28A/0x28B (low 16 bits of
# species + 0xFD76 below 2); otherwise reads the species' record through the archive handle and, if it
# has alternate forms (u16 @0x1C != 0) and 0 < form < count (u8 @0x20, signed compare), returns
# (first + form - 1) & 0xFFFF, else species unchanged.
fn 0201afd4 ret=u32 inputs=600
  nowork
  global 0214652c 4
  arg u32 alt range 0 649 || range 0 649 || oneof 0x28a 0x28b 0x289 0x28c 0xffff 0x1028a || any
  arg u32 alt range 0 4 || range 0 4 || oneof 0xFFFFFFFF 0x80000000 0x7FFFFFFF || any
  stub 02049578 ret=u32 args=v,v
  stub 0204960c ret=u32 args=v,v
  stub 020495f4 ret=none args=v,v,outm:0x3C:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xC0/0x01/0xFF/0xFF/0x03/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF

# FUN_0201AD88(arg): open the personal archive handle once (the global stays if already set).
fn 0201ad88 ret=none
  nowork
  global 0214652c 4
  arg u32 any
  fix global0 0 u32 alt const 0 || const 0 || any
  stub 020490f4 ret=u32 args=v,v

# FUN_0201ADA4(species, form, dst): read the species/form record into dst (0x3C bytes).
fn 0201ada4 ret=none inputs=300
  nowork
  global 0214652c 4
  region rec 0x3C
  arg u32 alt range 0 649 || range 0 649 || oneof 0x28a 0x28b || any
  arg u32 alt range 0 4 || range 0 4 || any
  arg region rec
  stub 02049578 ret=u32 args=v,v
  stub 0204960c ret=u32 args=v,v
  stub 020495f4 ret=none args=v,v,outm:0x3C:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xC0/0x01/0xFF/0xFF/0x03/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF

# FUN_0201ADE4(species, form, heap): allocate 0x3C bytes (FUN_02030734, the block is the `buf` argument 3,
# which the original ignores) and read the record into it; returns the block.
fn 0201ade4 ret=ofs:arg3 inputs=300
  nowork
  global 0214652c 4
  arg u32 alt range 0 649 || range 0 649 || oneof 0x28a 0x28b || any
  arg u32 alt range 0 4 || range 0 4 || any
  arg u32 alt range 0 80 || any
  arg buf 0x3C
  stub 02030734 ret=ptr:arg3:0:1 args=v,v,v,v,v
  stub 02049578 ret=u32 args=v,v
  stub 0204960c ret=u32 args=v,v
  stub 020495f4 ret=none args=v,v,outm:0x3C:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xC0/0x01/0xFF/0xFF/0x03/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF

# FUN_02019C54(species, form): the record cache. If (species, form) differs from the cached pair
# (compared as 32-bit species / form against the stored halfwords) the pair is stored and
# FUN_0201ADA4 refills the record at 0x02146274; returns that address.
fn 02019c54 ret=u32 inputs=600
  nowork
  global 02146270 0x40
  arg u32 alt range 0 5 || range 0 5 || any
  arg u32 alt range 0 3 || range 0 3 || any
  fix global0 0 u16 alt const 0 || range 0 3
  fix global0 2 u16 alt const 0 || range 0 5
  stub 0201ada4 force ret=none args=v,v,out:0x3C

# FUN_02019C90(species, form, field): personal field of a species/form = cache lookup + getter.
fn 02019c90 ret=u32 inputs=600
  nowork
  global 02146270 0x40
  arg u32 alt range 0 5 || range 0 5 || any
  arg u32 alt range 0 3 || range 0 3 || any
  arg u32 alt range 0 0x2B || range 0 0x2B || range 0 0x30
  fix global0 0 u16 alt const 0 || range 0 3
  fix global0 2 u16 alt const 0 || range 0 5
  stub 0201ada4 force ret=none args=v,v,out:0x3C

# FUN_02019B04(nature, value, stat): value * 110% / 90% (as (value * 110 or 90) & 0xFFFF / 100,
# signed division FUN_0209C0A4) when row `nature` of the nature table at 0x0209E324 marks stat
# (1 Atk, 2 Def, 3 SpA, 4 SpD, 5 Spe) as raised / lowered; value unchanged otherwise or stat > 5.
fn 02019b04 ret=u32 inputs=800
  nowork
  arg u32 alt range 0 24 || range 0 24 || range 0 24 || range 0 29
  arg u32 alt range 0 600 || range 0 40 || oneof 0 1 0xFFFF 0x10000 || any
  arg u32 alt range 0 7 || range 1 5 || range 1 5

# FUN_020185A0(mon): the nature field (id 0x70, low byte) of a BoxPokemon through FUN_02017E88.
fn 020185a0 ret=u8 inputs=400
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || any
  fix region:mon 0x08 u16 alt const 0 || const 0x124 || const 0x1d || const 0x20 || range 1 649 || range 1 649 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=p,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02045a38 ret=none args=v,v,v
  stub 0201f448 ret=bool args=v,v
  stub 02008f3c ret=none args=v,v

# FUN_02018A7C(mon): recalculate the stats of a Pokemon from level, species base stats (cache record),
# EVs/IVs and nature: HP = (2*base + iv + ev/4) * level / 100 + level + 10 (1 for species 0x124),
# other = ((2*base + iv + ev/4) * level / 100 + 5) * nature modifier (FUN_02019B04), stored in ids
# 0xA1 (max HP) and 0xA2-0xA6; current HP is shifted by the change of max HP (0 stays 0).
fn 02018a7c ret=none inputs=1200
  nowork
  global 02146270 0x40
  global 0214652c 4
  region mon 0xDC
  arg region mon
  fix global0 0 u16 alt const 0 || range 0 3 || any
  fix global0 2 u16 alt const 0 || range 0 649 || any
  fix region:mon 0 u32 const 0 bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || any
  fix region:mon 0x08 u16 alt const 0 || const 0x124 || const 0x1d || const 0x20 || range 1 649 || range 1 649 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 0x41 u8 range 0 24
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x8c u8 alt range 1 100 || range 1 100 || range 1 100 || any
  fix region:mon 0x8e u16 alt range 0 400 || any
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || const 0 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
  stub 02049578 ret=u32 args=v,v
  stub 0204960c ret=u32 args=v,v
  stub 020495f4 ret=none args=v,v,outm:0x3C:0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xC0/0x01/0xFF/0xFF/0x03/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF/0xFF
  stub 02017f50 ret=u32 args=v,v,v
  stub 0201850c ret=u32 args=v,v,v
  stub 0201c158 ret=u32 args=v,v
  stub 02019c90 ret=u32 args=v,v,v
  stub 02045924 ret=none args=v,v
  stub 0201f41c ret=none args=v,v,v
  stub 02045cac ret=none args=p,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 02045a38 ret=none args=v,v,v
  stub 0201f448 ret=bool args=v,v
  stub 02008f3c ret=none args=v,v
