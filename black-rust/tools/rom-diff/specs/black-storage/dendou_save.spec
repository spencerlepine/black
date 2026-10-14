# fndiff spec (S5b): save block 66 of the main ARM9 (0x0200EA14..0x0200ECB4): one Hall of Fame record (six
# 0x3C-byte entries, +0x168 u16 year, +0x16A / +0x16B u8 month / day).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage dendou_save --per-function
# The party / Pokemon / string-buffer routines are stubbed (the string copies write nothing: their destination is
# `p`); FUN_02045A98 (EOS, black-text) and MI_CpuFill8 run as real code.
area dendou_save
set field

fn 0200ea14 ret=u32 inputs=10
  nowork

fn 0200ea2c ret=none inputs=50
  nowork
  arg buf 0x16c

# FUN_0200EA3C(rec, party, date, heap): store the party's non-egg mons and the date
fn 0200ea3c ret=none inputs=400
  nowork
  arg buf 0x16c
  arg u32 any
  arg buf 0xc
  arg u32 alt range 0 0x40 || any
  stub 0201aa18 ret=pick:0/1/3/5/6 args=v
  stub 020457b0 ret=nullable:3:u32 args=v,v
  stub 0201ab28 ret=u32 args=v,v
  stub 02017d30 ret=u32 args=v
  stub 02017e1c ret=pick+u32:0/1/5/300 args=v,v,v
  stub 02045a38 ret=none args=v,ptr,v
  stub 02017d70 ret=none args=v,v
  stub 02045808 ret=none args=v

# sub_0200EC14(rec): entries before the first species 0
fn 0200ec14 ret=u32 inputs=300
  nowork
  arg buf 0x16c
  fix arg0 0 u16 alt const 0 || any rep=6,0x3c

# FUN_0200EC2C(rec, i, out): entry i into the view (out +0 / +4: string buffers)
fn 0200ec2c ret=none inputs=100
  nowork
  arg buf 0x16c
  arg u32 range 0 5
  arg buf 0x1c
  stub 02045994 ret=none args=v,in:16,v

# sub_0200EC98(rec, out): the date as 4 words
fn 0200ec98 ret=none inputs=50
  nowork
  arg buf 0x16c
  arg buf 0x10
