# fndiff spec (S5b): the White Forest / Black City save block 43 of the main ARM9 (0x0200A840..0x0200A8A0):
# 0x2DC bytes (0x1E8 town data, ten u16 at +0x1E8, 0xE0-byte tail at +0x1FC).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage wfbc_save --per-function
# The overlay-10 init routines FUN_02161D98 / FUN_02161EEC / FUN_02161F00 (overlay 10 is in the field set),
# MI_CpuFill8 and MI_CpuCopy8 run as real code (the port writes the same bytes).
area wfbc_save
set field

fn 0200a840 ret=u32 inputs=10
  nowork

# sub_0200A848(block): init the three parts
fn 0200a848 ret=none inputs=50
  nowork
  arg buf 0x2dc

# FUN_0200A870(block, src): town data = src
fn 0200a870 ret=none inputs=50
  nowork
  arg buf 0x2dc
  arg buf 0x1e8

# FUN_0200A884(block, dst): copy the town data out
fn 0200a884 ret=none inputs=50
  nowork
  arg buf 0x2dc
  arg buf 0x1e8

# 0x0200A890 / FUN_0200a898(block): +0x1E8 / +0x1FC
fn 0200a890 ret=ofs:arg0 inputs=10
  nowork
  arg buf 0x2dc

fn 0200a898 ret=ofs:arg0 inputs=10
  nowork
  arg buf 0x2dc
