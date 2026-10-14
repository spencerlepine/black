# fndiff spec (S5b): the field-object save block 41 of the main ARM9 (0x0200EE48..0x0200F128): 64 records of
# 0x50 bytes, the save form of a 0x100-byte MMDL.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage mmdl_save --per-function
# MI_CpuFill8 / MI_CpuCopy8 run as real code; the overlay-10 OBJ-data load FUN_0216E2E8 is stubbed (writes the
# 16 bytes at MMDL + 0xE4). The status word's bit 1 is drawn both ways.
area mmdl_save
set field

fn 0200ee48 ret=u32 inputs=10
  nowork

fn 0200ee50 ret=none inputs=20
  nowork
  arg buf 0x1400

# FUN_0200EED8(mmdl, rec): MMDL -> record
fn 0200eed8 ret=none inputs=200
  nowork
  arg buf 0x100
  arg buf 0x50

# FUN_0200EF78(mmdl, rec, sys): record -> MMDL
fn 0200ef78 ret=none inputs=300
  nowork
  arg buf 0x100
  arg buf 0x50
  arg u32 any
  stub 0216e2e8 force ret=none args=v,v,out:16
