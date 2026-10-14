# fndiff spec (S5): symbol_save.c of the main ARM9 (0x0200E024..0x0200E368): save block 61
# (0x850 bytes: 530 u32 records in four zones, +0x848 / +0x849 bytes, cipher checksum / key).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage symbol_save --per-function
#
# The at-rest cipher (FUN_0200E024 seal, FUN_0200E06C open: V-count key, checksum, LCG) is DS
# save plumbing the port replaces with plaintext; both are stubbed (the port calls its
# SymbolCrypt hooks in the same places). The zone table (rodata 0x0209DE90) is read as real data.
# Records get a zero species field (free slot) half of the time so the searches find and miss.
# MI_CpuCopy8 / MI_CpuFill8 / _s32_div_f run as real code.
area symbol_save
set field

fn 0200e090 ret=u32 inputs=10
  nowork

# sub_0200E098(block): zero, seal
fn 0200e098 ret=none inputs=50
  nowork
  arg buf 0x850
  stub 0200e024 ret=none args=v

# FUN_0200E0BC(index): zone of the index
fn 0200e0bc ret=u32 inputs=400
  nowork
  arg u32 alt near 10 1 || near 20 1 || near 50 1 || near 530 1 || any

# unlabeled 0x0200E0E8(block, zone): first free record of the zone, 0xFFFF if none
fn 0200e0e8 ret=u32 inputs=600
  nowork
  arg buf 0x850
  arg u32 range 0 3
  fix arg0 0 u32 alt mask 0xfffff800 || any || any || any rep=530,4

# FUN_0200E124(block, index): drop a record of its zone
fn 0200e124 ret=none inputs=400
  nowork
  arg buf 0x850
  arg u32 alt range 0 60 || range 0 529 || range 0 700

# FUN_0200E164(block, zone): open, find free, seal
fn 0200e164 ret=u32 inputs=400
  nowork
  arg buf 0x850
  arg u32 range 0 3
  fix arg0 0 u32 alt mask 0xfffff800 || any || any || any rep=530,4
  stub 0200e024 ret=none args=v
  stub 0200e06c ret=none args=v

# FUN_0200E184(index): FUN_0200E0BC (tail call)
fn 0200e184 ret=u32 inputs=200
  nowork
  arg u32 alt near 10 1 || near 20 1 || near 50 1 || near 530 1 || any

# FUN_0200E18C(block, index): open, drop, seal
fn 0200e18c ret=none inputs=200
  nowork
  arg buf 0x850
  arg u32 alt range 0 60 || range 0 529 || range 0 700
  stub 0200e024 ret=none args=v
  stub 0200e06c ret=none args=v

# FUN_0200E1A8(block, species, a, b, c, d, zone): store a record
fn 0200e1a8 ret=none inputs=1500
  nowork
  arg buf 0x850
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 range 0 3
  fix arg0 0 u32 alt mask 0xfffff800 || any || any rep=530,4
  fix arg0 0x848 u8 alt range 0 2 || any
  stub 0200e024 ret=none args=v
  stub 0200e06c ret=none args=v

# FUN_0200E27C(block, slot): +0x849 from slots 170..530
fn 0200e27c ret=none inputs=600
  nowork
  arg buf 0x850
  arg u32 alt near 170 2 || near 0x212 2 || range 0 600 || oneof 0xffff || any
