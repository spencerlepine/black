# fndiff spec (S5b): the Entralink save block 53 of the main ARM9 (0x0200BDBC..0x0200C054): 0x1B0 bytes
# (+0 0x10-byte record, +0x10 20 x 0x14-byte entries, +0x1A0 u8, +0x1A1 flag bytes, +0x1A4 u32, +0x1A8 u64).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage intrude_save --per-function
# FUN_02045A98 (the EOS, black-text) and MI_CpuFill8 / _ll_mul run as real code; FUN_021763BC (overlay 11) and the
# play-time getters FUN_0200864C / 50 / 54 are stubbed. The session start 0x02146170 is a random global.
area intrude_save
set field

fn 0200bdbc ret=u32 inputs=10
  nowork

# sub_0200BDC4(block): zero, record init, +0x1A0 = 0x30
fn 0200bdc4 ret=none inputs=50
  nowork
  arg buf 0x1b0
  stub 021763bc ret=none args=in:16

# FUN_0200BDE4(block): zero the record at +0, FUN_021763BC on it
fn 0200bde4 ret=none inputs=50
  nowork
  arg buf 0x1b0
  stub 021763bc ret=none args=in:16

fn 0200bdf8 ret=u16 inputs=50
  nowork
  arg buf 0x1b0

fn 0200bdfc ret=u16 inputs=50
  nowork
  arg buf 0x1b0

# FUN_0200BE00 / FUN_0200BE24(block, n): add, capped at 999 (signed sum)
fn 0200be00 ret=none inputs=400
  nowork
  arg buf 0x1b0
  arg u32 alt range 0 1100 || any
  fix arg0 0xc u16 alt range 0 1100 || any

fn 0200be24 ret=none inputs=400
  nowork
  arg buf 0x1b0
  arg u32 alt range 0 1100 || any
  fix arg0 0xe u16 alt range 0 1100 || any

# FUN_0200BE54(sav, src): record at +0 = src
fn 0200be54 ret=none inputs=50
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x1b0
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# FUN_0200BE6C(sav, dst): copy the record out
fn 0200be6c ret=none inputs=50
  nowork
  arg u32 any
  arg buf 0x10
  arg buf 0x1b0
  stub 020071cc ret=ptr:arg2:0:1 args=v,v

# 0x0200BE84(block, rec): add an entry (rec: 8-halfword name with an EOS, +0x10 used, +0x12 kind < 18)
fn 0200be84 ret=none inputs=1500
  nowork
  arg buf 0x1b0
  arg buf 0x14
  fix arg1 0 u16 alt oneof 0xffff || any rep=8,2
  fix arg1 0x12 u8 alt range 0 20 || any
  fix arg0 0x20 u16 alt const 0 || const 0 || any rep=20,0x14

# 0x0200BF08 / FUN_0200BF10: +0x1A0 set / get
fn 0200bf08 ret=none inputs=50
  nowork
  arg buf 0x1b0
  arg u8 any

fn 0200bf10 ret=u8 inputs=50
  nowork
  arg buf 0x1b0

# FUN_0200bf18(block, id): set the flag of id 33..38
fn 0200bf18 ret=none inputs=300
  nowork
  arg buf 0x1b0
  arg u32 alt range 0 50 || oneof 0xffffffff 0xffffffe0

# FUN_0200BF54(block, out): the two flag bytes
fn 0200bf54 ret=none inputs=50
  nowork
  arg buf 0x1b0
  arg buf 0x4

fn 0200bf70 ret=u32 inputs=50
  nowork
  arg buf 0x1b0

# 0x0200BF78(block): counter + 1 below 99999
fn 0200bf78 ret=u32 inputs=200
  nowork
  arg buf 0x1b0
  fix arg0 0x1a4 u32 alt near 99999 3 || range 0 1000 || any

fn 0200bf94 ret=u64 inputs=50
  nowork
  arg buf 0x1b0

# FUN_0200BFA4(block, playtime, mode): play seconds; session start in RAM 0x02146170
fn 0200bfa4 ret=u64 inputs=400
  nowork
  arg buf 0x1b0
  arg u32 any
  arg u32 alt range 0 3 || any
  global 02146170 8
  stub 0200864c ret=u16 args=v
  stub 02008650 ret=u8 args=v
  stub 02008654 ret=u8 args=v
