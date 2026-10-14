# fndiff spec (S5b): the Battle Box save block 49 of the main ARM9 (0x0200BA78..0x0200BC10): 0x35C bytes
# (six 0x88-byte box Pokemon, +0x330 0x28-byte name, +0x358 u16 flags).
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage battle_box --per-function
# FUN_02017550 (clear a box Pokemon, black-pokemon create.rs) and MI_CpuFill8 run as real code; the message
# bank, Pokemon parameter, party and heap routines are stubbed. Only box 0 is addressed (the block has one).
area battle_box
set field

fn 0200ba78 ret=u32 inputs=10
  nowork

# sub_0200BA80(block): zero, clear the six mons, name from message bank 9 entry 6
fn 0200ba80 ret=none inputs=100
  nowork
  arg buf 0x35c
  stub 02045b38 ret=nullable:3:u32 args=v,v,v,v
  stub 02045f5c ret=none args=v,v,out:40,v
  stub 02045c04 ret=none args=v

# 0x0200BAD4(block, box, slot, src): copy a box Pokemon in
fn 0200bad4 ret=none inputs=100
  nowork
  arg buf 0x35c
  arg u32 const 0
  arg u32 range 0 5
  arg buf 0x88

# FUN_0200BAF4(block, heap): a party from the stored mons
fn 0200baf4 ret=u32 inputs=300
  nowork
  arg buf 0x35c
  arg u32 alt range 0 0x40 || any
  stub 0201a920 ret=u32 args=v
  stub 0201a954 ret=none args=v,v
  stub 02017e88 ret=pick+u32:0/1/1/1 args=in:136,v,v
  stub 0201747c ret=u32 args=in:136,v
  stub 0201a98c ret=none args=v,v
  stub 020307b0 ret=none args=v

# FUN_0200bb58(block, party, arena): clear, then store the box part of each party mon (arena: test-only target
# of the box-part pointers)
fn 0200bb58 ret=none inputs=300
  nowork
  arg buf 0x35c
  arg u32 any
  arg buf 0x100
  stub 0201aa18 ret=pick:0/1/2/3/4/5/6 args=v
  stub 0201ab28 ret=u32 args=v,v
  stub 020185f4 ret=ptr:arg2:0:8 args=v

# FUN_0200BBA8(block): slot 0 holds a mon
fn 0200bba8 ret=bool inputs=100
  nowork
  arg buf 0x35c
  stub 02017e88 ret=pick+u32:0/1 args=in:136,v,v

# 0x0200BBC8 / 0x0200BBD8(block, bits): flags |= bits / &= ~bits
fn 0200bbc8 ret=none inputs=100
  nowork
  arg buf 0x35c
  arg u32 any

fn 0200bbd8 ret=none inputs=100
  nowork
  arg buf 0x35c
  arg u32 any

# FUN_0200BBEC(block, box, slot): address of a slot
fn 0200bbec ret=ofs:arg0 inputs=50
  nowork
  arg buf 0x35c
  arg u32 const 0
  arg u32 range 0 5

# 0x0200BBFC(block, bits): flags & bits != 0
fn 0200bbfc ret=bool inputs=200
  nowork
  arg buf 0x35c
  arg u32 alt mask 0xffff || any
