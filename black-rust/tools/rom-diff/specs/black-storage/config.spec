# fndiff spec (S5): config.c of the main ARM9 (0x020082DC..0x02008468), the options bit field.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage config --per-function
#
# CONFIG = 4 bytes: u16 bit field at +0 (bits 0-3 text speed, 4-5 button mode, 6, 7, 8 sound mode,
# 9, 10), +2..+3 unused. Every function takes the block as `arg buf 4` (random bits); setters take
# a full-width random value (the originals truncate it to 16 bits, then mask).
# FUN_020461D4 (sound system: apply the sound-mode bit) is stubbed. MI_CpuCopy8 / MI_CpuFill8 run
# as real code.
area config
set field

# FUN_02008314(src, dst): MI_CpuCopy8(src, dst, 4)
fn 02008314 ret=none
  nowork
  arg buf 4
  arg buf 4

# FUN_02008320(cfg): clear, text speed 1, bit 9 set
fn 02008320 ret=none
  nowork
  arg buf 4

fn 02008374 ret=u32
  nowork
  arg buf 4

fn 0200837c ret=none
  nowork
  arg buf 4
  arg u32 any

fn 02008390 ret=u32
  nowork
  arg buf 4

fn 02008398 ret=none
  nowork
  arg buf 4
  arg u32 any

fn 020083ac ret=u32
  nowork
  arg buf 4

fn 020083b4 ret=none
  nowork
  arg buf 4
  arg u32 any

fn 020083c8 ret=u32
  nowork
  arg buf 4

fn 020083d0 ret=none
  nowork
  arg buf 4
  arg u32 any

fn 020083e4 ret=u32
  nowork
  arg buf 4

# FUN_020083EC(cfg, v): bit 8 = v & 1, then FUN_020461D4(bit 8)
fn 020083ec ret=none
  nowork
  arg buf 4
  arg u32 any
  stub 020461d4 ret=none args=v

# FUN_02008414(cfg): FUN_020461D4(bit 8) (tail call)
fn 02008414 ret=none
  nowork
  arg buf 4
  stub 020461d4 ret=none args=v

fn 02008428 ret=u32
  nowork
  arg buf 4

fn 02008430 ret=none
  nowork
  arg buf 4
  arg u32 any

fn 02008448 ret=u32
  nowork
  arg buf 4

fn 02008450 ret=none
  nowork
  arg buf 4
  arg u32 any
