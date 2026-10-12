# fndiff spec: PokeParam allocation and initialisation, recalculation and learnset scan (P1c),
# poke_tool.c of main ARM9 (0x02016BA0-0x0201A26C).
# Run: tools/check-fn.sh black-pokemon create --per-function
#
# Boundaries (stubbed, each verified by its own area): the heap allocator FUN_02030734(heap, size, align,
# file, line) returns a block that the spec supplies as a `buf` argument (`ptr:argK:..`; `nullable:N:..`
# makes an allocation failure happen 1 time in N); the RNG FUN_020056EC (LCG64 random(max)) and
# FUN_0203F0A8 (MT19937 next); the personal-data routines FUN_02019C54 / FUN_0201AE1C /
# FUN_02019C90 / FUN_02017F50 / FUN_0201850C / FUN_02018584 (area personal / growth); the
# string, learnset and ext-copy callees that poke_tool.spec stubs. The Pokemon get/put routines
# (FUN_02017E1C/40/64/88, ported in poke_tool.rs) and the cipher run as real code.
# Records come from the same planting recipe as poke_tool.spec (valid checksum, really encrypted
# blocks, party extension with a level, HP fields and the PID cipher).
# The PID/IV arguments of the init family are (species, level, a3, a4, ivs, a6, a7): a4:a3 = -1:-1
# picks a random PID (-1:0 re-rolls until FUN_02017ED8 is false), ivs = 0xFFFFFFFF rolls 6 random IVs, (a7:a6) = (0xFFFFFFFF:0) picks a random
# trainer ID and (0xFFFFFFFF:1) copies the PID pair.
area create
set field

# FUN_02017528(mon): zero the 0xDC-byte Pokemon, then run the cipher over blocks (seed halfword at 6)
# and party extension (seed PID at 0), i.e. an encrypted all-zero record.
fn 02017528 ret=none inputs=100
  nowork
  region mon 0xDC
  arg region mon

# FUN_02017550(mon): the same for a 0x88-byte BoxPokemon.
fn 02017550 ret=none inputs=100
  nowork
  region mon 0xDC
  arg region mon

# FUN_02017D30(mon): set the plain flags (bit 0, then bit 1) and decrypt the party extension (PID seed)
# and the blocks (checksum seed) in place; returns whether bit 0 was clear (no work otherwise).
fn 02017d30 ret=u32 inputs=200
  nowork
  region mon 0xDC
  arg region mon

# FUN_02017D70(mon, flag): the inverse: only if bit 0 is set and flag == 1, clear both flags, encrypt the
# extension, store the block checksum (FUN_02019A9C) and encrypt the blocks; returns 1 if it ran.
fn 02017d70 ret=u32 inputs=300
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt const 1 || const 1 || const 0 || any
  fix region:mon 4 u16 alt const 1 || const 3 || const 1 || const 0 || any
  fix region:mon 6 u16 alt sum16 8 0x80 || any

# FUN_020173F0(species, level, a2, a3, heap): allocate 0xDC bytes (poke_tool.c line 0xA0, align flag 1)
# and, if that worked, initialise the Pokemon with FUN_0201756C(mon, species, level, a2, a3);
# returns the block or 0. The block is the `buf` argument 5, which the original ignores.
fn 020173f0 ret=ofs:arg5 inputs=300
  nowork
  arg u32 alt range 0 649 || any
  arg u32 alt range 1 100 || any
  arg u32 any
  arg u32 any
  arg u16 alt range 0 80 || any
  arg buf 0xE0
  stub 02030734 ret=nullable:6:ptr:arg5:4:1 args=v,v,v,v,v
  stub 0201756c force ret=none args=v,v,v,v,v

# FUN_02017430(a0..a6, heap): the same with the 8-argument initialiser FUN_02017588 (line 0xBA);
# the block is the `buf` argument 8.
fn 02017430 ret=ofs:arg8 inputs=300
  nowork
  arg u32 alt range 0 649 || any
  arg u32 alt range 1 100 || any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u16 alt range 0 80 || any
  arg buf 0xE0
  stub 02030734 ret=nullable:6:ptr:arg8:4:1 args=v,v,v,v,v
  stub 02017588 force ret=none args=v,v,v,v,v,v,v,v

# FUN_0201756C(mon, species, level, a3, a4): FUN_02017588 with the last three arguments (-1, 0, -1).
fn 0201756c ret=none
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02017588 force ret=none args=v,v,v,v,v,v,v,v

# FUN_02017588(mon, species, level, a3, a4, a5, a6, a7): clear the record (FUN_02017528), run the core
# initialiser FUN_0201761C, re-encrypt the extension with a zero then the PID seed, store the level,
# put a fresh 0x38-byte ext block (allocated by FUN_02008F08, line 0x87) with id 0xA7, free it and
# recalculate the stats (FUN_020185C4).
fn 02017588 ret=none inputs=300
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 1 649 || range 1 649 || oneof 0 0x124 0x28a || any
  arg u32 alt range 1 100 || range 1 100 || oneof 0 1 100 || any
  arg u32 alt const 0 || const 0xFFFFFFFF || const 0xFFFFFFFF || any
  arg u32 alt const 0xFFFFFFFF || const 0xFFFFFFFF || const 0 || any
  arg u32 alt const 0xFFFFFFFF || any
  arg u32 alt const 0 || const 1 || any
  arg u32 alt const 0xFFFFFFFF || const 0xFFFFFFFF || const 0 || any
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || any
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || range 1 649 || range 1 649 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x8c u8 alt range 1 100 || range 1 100 || range 1 100 || any
  fix region:mon 0x8e u16 alt range 0 400 || any
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || const 0 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
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
  stub 0201761c force ret=none args=v,v,v,v,v,v,v,v
  stub 02008f08 ret=u32 args=v
  stub 020307b0 ret=none args=v
  stub 020185c4 force ret=none args=v

# FUN_02017600(mon, species, level, a3, a4): FUN_0201761C with (-1, 0, -1) for the last three.
fn 02017600 ret=none
  nowork
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  stub 0201761c force ret=none args=v,v,v,v,v,v,v,v

# FUN_0201761C(mon, species, level, pid_lo, pid_hi, ivs, otid_lo, otid_hi): the core initialiser: it
# starts from a zeroed box record, rolls or copies the PID and trainer ID, stores species, name,
# EXP for the level, base friendship, ability slot, the IVs (random 5-bit values or the packed argument),
# gender, nature, and learns the starting moves (FUN_02017FB0).
fn 0201761c ret=none inputs=600
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 1 649 || range 1 649 || oneof 0 0x124 0x28a || any
  arg u32 alt range 1 100 || range 1 100 || oneof 0 1 100 || any
  arg u32 alt const 0 || const 0xFFFFFFFF || const 0xFFFFFFFF || any
  arg u32 alt const 0xFFFFFFFF || const 0xFFFFFFFF || const 0 || any
  arg u32 alt const 0xFFFFFFFF || any
  arg u32 alt const 0 || const 1 || any
  arg u32 alt const 0xFFFFFFFF || const 0xFFFFFFFF || const 0 || any
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || any
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || range 1 649 || range 1 649 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x8c u8 alt range 1 100 || range 1 100 || range 1 100 || any
  fix region:mon 0x8e u16 alt range 0 400 || any
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || const 0 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
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
  stub 02019c54 ret=u32 args=v,v
  stub 0201ae1c ret=u32 args=v,v
  stub 020056ec ret=u32 args=v
  stub 0203f0a8 ret=u32 args=
  stub 02018584 ret=u32 args=v,v,v
  stub 02017fb0 ret=none args=v

# FUN_0201747C(mon, heap): a copy of a Pokemon: allocate and initialise a new one with its species, level
# and id 7 (FUN_020173F0), copy the first 0x88 bytes over it, zero ids 0x9D/0xA0/0xA1, put a fresh
# ext block (FUN_02008F08) and recalculate (FUN_020185C4). The new block is the `buf` argument 2.
fn 0201747c ret=ofs:arg2 inputs=300
  nowork
  region mon 0xDC
  arg region mon
  arg u32 alt range 0 80 || any
  arg buf 0xE0
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || any
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || range 1 649 || range 1 649 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x8c u8 alt range 1 100 || range 1 100 || range 1 100 || any
  fix region:mon 0x8e u16 alt range 0 400 || any
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || const 0 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
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
  stub 02030734 ret=ptr:arg2:0:1 args=v,v,v,v,v
  stub 0201756c force ret=none args=v,v,v,v,v
  stub 02008f08 ret=u32 args=v
  stub 020307b0 ret=none args=v
  stub 020185c4 force ret=none args=v

# FUN_020185C4(mon): after a change of EXP: enter (FUN_02017D30), level from EXP (FUN_02018438 ->
# FUN_0201850C), store the level (id 0x9E), recalculate stats (FUN_02018A7C), exit (FUN_02017D70).
fn 020185c4 ret=none inputs=400
  nowork
  region mon 0xDC
  arg region mon
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || any
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || range 1 649 || range 1 649 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x8c u8 alt range 1 100 || range 1 100 || range 1 100 || any
  fix region:mon 0x8e u16 alt range 0 400 || any
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || const 0 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
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
  stub 02018a7c ret=none args=v

# FUN_02018328(mon, cursor, heap): the next level-up move: allocate 0x68 bytes (line 0x644), load the
# learnset of the mon's species/form into it (FUN_0201ADD0, stubbed: the block holds the `fix`ed
# (move, level) pairs ending in 0xFFFF, 0xFFFF), and walk it from *cursor. The block is the `buf`
# argument 3. Pairs whose level equals the mon's level are looked up
# with FUN_0201808C (a move-slot search, real code); *cursor advances; the result is a move or slot
# value built from that, 0xFFFF, or 0.
fn 02018328 ret=u32 inputs=600
  nowork
  region mon 0xDC
  region cursor 4
  arg region mon
  arg region cursor
  arg u32 alt range 0 80 || any
  arg buf 0x68
  fix region:mon 0 u32 alt const 0 || const 0 || any bits=13:5
  fix region:mon 4 u16 alt const 0 || const 0 || const 0 || any
  fix region:mon 0x08 u16 alt const 0 || const 0x1d || const 0x20 || const 0x1ed || range 1 649 || range 1 649 || any
  fix region:mon 0x0a u16 alt range 0 0x290 || near 0x272 4 || any
  fix region:mon 0x15 u8 alt const 0x79 || range 0 255 || any
  fix region:mon 0x18 u8 alt range 0 90 || range 0 40 || any rep=6,1
  fix region:mon 0x28 u16 alt const 0 || range 1 560 || any rep=4,2
  fix region:mon 0x34 u8 alt range 0 3 || any rep=4,1
  fix region:mon 6 u16 alt sum16 8 0x80 || sum16 8 0x80 || sum16 8 0x80 || any
  crypt region:mon 8 0x80 u16@6 skipif=u16@4&2
  fix region:mon 0x8c u8 alt range 1 6 || range 1 6 || range 1 6 || any
  fix region:mon 0x8e u16 alt range 0 400 || any
  fix region:mon 0x90 u16 alt range 0 400 || range 0 400 || const 0 || any
  crypt region:mon 0x88 0x54 u32@0 skipif=u16@4&1
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
  fix region:cursor 0 u32 alt const 0 || range 0 12 || range 0 24
  fix arg3 0 u16 alt range 1 560 || range 1 560 || range 1 560 || const 0xFFFF rep=25,4
  fix arg3 2 u16 alt range 1 6 || range 1 6 || range 1 6 || const 0xFFFF rep=25,4
  fix arg3 0x64 u32 const 0xFFFFFFFF
  stub 02030734 ret=ptr:arg3:0:1 args=v,v,v,v,v
  stub 0201add0 ret=none args=v,v,p
  stub 020307b0 ret=none args=v
