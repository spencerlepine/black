# fndiff spec (P7b): egg hatching. Run: tools/check-fn.sh black-pokemon hatch --per-function
#
# The field step hook and the egg-cycle countdown (overlay 21 of the field set, 0x0218A9B0-0x0218AB9C) and
# the hatch conversion of the egg Pokemon (main ARM9 0x02017B24 / 0x02017B4C, a region of the ROM that the
# decompiled sets have no C for: ported from the Thumb disassembly).
#
# Every block lists the stubs of its callees and of the callees of its in-area descendants (closure stubs),
# so the area can be ported in any order. In-area callees run as real code on both sides.
# Boundaries (stubbed, outside the area): the Pokemon accessors FUN_02017E1C (get) / FUN_02017E40 (put) and
# the box-part accessors FUN_02017E88 / FUN_02017E64, the party accessors FUN_0201AA18 / FUN_0201AB28, the
# field system / save readers FUN_02012EBC, FUN_02008854, FUN_02012934, FUN_020120F4, the step counter
# access FUN_02008970 / FUN_0200896C, the step-rate scaler FUN_02010FF4, the step work readers FUN_02188C80
# / FUN_02188C84 / FUN_02188D50, the day care step FUN_021C3378 (F13), the event start FUN_02158768, the
# other step handlers FUN_0218AC84 / FUN_0218AB9C / FUN_0218AC08 / FUN_0218AC4C, the box / party record
# (de)crypt toggles FUN_02017D30 / FUN_02017D70 / FUN_02017DBC / FUN_02017DE4, the message copy FUN_02045F5C
# and the met-data setter FUN_0202F1B4.
#
# Records: `cfg` is the step hook's config struct (words +0x08, +0x10, +0x3C are read). Pokemon and party
# handles are few and fixed so equal stub answers repeat. The stub for the Pokemon getter answers from a
# short list of the values the functions compare with (egg flag 0 / 1, friendship 0 / small, Flame Body
# 0x31, Magma Armor 0x28), because the stub cannot tell the field ids apart.
area hatch
set field

# FUN_0218AA2C(party): 2 if a non-egg member has Flame Body or Magma Armor, else 1.
fn 0218aa2c ret=u32 inputs=600
  nowork
  arg u32 oneof 0x02300000 0x02300040
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=pick:0x02300000/0x02300100/0x02300200 args=v,v
  stub 02017e1c ret=pick:0/0/1/0x28/0x31/0x31/5 args=v,v,v

# FUN_0218AA74(party): lower the cycles of every egg by that step.
fn 0218aa74 ret=none inputs=600
  nowork
  arg u32 oneof 0x02300000 0x02300040
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=pick:0x02300000/0x02300100/0x02300200 args=v,v
  stub 02017e1c ret=pick:0/0/1/1/2/0x28/0x31/5/0xff args=v,v,v
  stub 02017e40 ret=none args=v,v,v

# FUN_0218AAD0(fsys, party, _, _): add the scaled step; above 0x10000 run the countdown.
fn 0218aad0 ret=none inputs=800
  nowork
  arg u32 oneof 0x02300000 0x02300040
  arg u32 oneof 0x02300000 0x02300040
  arg u32 any
  arg u32 any
  stub 02012ebc ret=u32 args=v
  stub 02008854 ret=u32 args=v
  stub 02008970 ret=none args=v,outm:4:0xFF/0xFF/0x1/0
  stub 02010ff4 ret=pick:0/1/0x100/0x80/0x7f00/0xffff/0x10000 args=v
  stub 0200896c ret=none args=v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=pick:0x02300000/0x02300100/0x02300200 args=v,v
  stub 02017e1c ret=pick:0/0/1/1/2/0x28/0x31/5/0xff args=v,v,v
  stub 02017e40 ret=none args=v,v,v

# FUN_0218AB0C(party): any egg with 0 cycles left.
fn 0218ab0c ret=u32 inputs=600
  nowork
  arg u32 oneof 0x02300000 0x02300040
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=pick:0x02300000/0x02300100/0x02300200 args=v,v
  stub 02017e1c ret=pick:0/0/1/1/5 args=v,v,v

# FUN_0218AB50(_, work, field, fsys): day care step, countdown, hatch event.
fn 0218ab50 ret=u32 inputs=800
  nowork
  arg u32 any
  arg u32 oneof 0x02300000 0x02300040
  arg u32 any
  arg u32 oneof 0x02300000 0x02300040
  stub 02188c84 ret=u16 args=v
  stub 02012934 ret=u32 args=v
  stub 02188d50 ret=u32 args=v
  stub 021c3378 ret=none args=v
  stub 02158768 ret=pick:0/1/2 args=v,v,v,v
  stub 02012ebc ret=u32 args=v
  stub 02008854 ret=u32 args=v
  stub 02008970 ret=none args=v,outm:4:0xFF/0xFF/0x1/0
  stub 02010ff4 ret=pick:0/1/0x100/0x80/0x7f00/0xffff/0x10000 args=v
  stub 0200896c ret=none args=v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=pick:0x02300000/0x02300100/0x02300200 args=v,v
  stub 02017e1c ret=pick:0/0/1/1/2/0x28/0x31/5/0xff args=v,v,v
  stub 02017e40 ret=none args=v,v,v

# FUN_0218A9B0(cfg, work): the field step hook. cfg flags (word +0x3C): bit 2 = step routines, bit 9 = fast handler.
fn 0218a9b0 ret=u32 inputs=1000
  nowork
  region cfg 0x40
  arg region cfg
  arg u32 any
  fix region:cfg 0x3c u32 alt const 0 || const 4 || const 0x200 || const 0x204 || any
  stub 02188c84 ret=u16 args=v
  stub 02188c80 ret=u32 args=v
  stub 020120f4 ret=u32 args=v
  stub 0218ac84 ret=pick:0/0/0/1/5 args=v,v,v
  stub 0218ab9c ret=none args=v,v
  stub 0218ac08 ret=pick:0/0/0/3 args=v,v,v
  stub 0218ac4c ret=pick:0/0/2 args=v,v,v
  stub 02012934 ret=u32 args=v
  stub 02188d50 ret=u32 args=v
  stub 021c3378 ret=none args=v
  stub 02158768 ret=pick:0/1/2 args=v,v,v,v
  stub 02012ebc ret=u32 args=v
  stub 02008854 ret=u32 args=v
  stub 02008970 ret=none args=v,outm:4:0xFF/0xFF/0x1/0
  stub 02010ff4 ret=pick:0/1/0x100/0x80/0x7f00/0xffff/0x10000 args=v
  stub 0200896c ret=none args=v,v
  stub 0201aa18 ret=pick:0/1/2/3/6 args=v
  stub 0201ab28 ret=pick:0x02300000/0x02300100/0x02300200 args=v,v
  stub 02017e1c ret=pick:0/0/1/1/2/0x28/0x31/5/0xff args=v,v,v
  stub 02017e40 ret=none args=v,v,v

# FUN_02017B4C(mon, a, b, c): the hatch conversion proper (egg flag 0, friendship 0x78, species name nickname,
# met data kind 2).
fn 02017b4c ret=none inputs=300
  nowork
  arg u32 oneof 0x02300000 0x02300100
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02017dbc ret=pick:0/1/0x101 args=v
  stub 02017e88 ret=u32 args=v,v,v
  stub 02017e64 ret=none args=v,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 0202f1b4 ret=none args=v,v,v,v,v
  stub 02017de4 ret=none args=v,v

# FUN_02017B24(mon, a, b, c): FUN_02017B4C with the box part decrypted.
fn 02017b24 ret=none inputs=300
  nowork
  arg u32 oneof 0x02300000 0x02300100
  arg u32 any
  arg u32 any
  arg u32 any
  stub 02017d30 ret=pick:0/1/0x101 args=v
  stub 02017d70 ret=none args=v,v
  stub 02017dbc ret=pick:0/1/0x101 args=v
  stub 02017e88 ret=u32 args=v,v,v
  stub 02017e64 ret=none args=v,v,v
  stub 02045f5c ret=none args=p,v,v,v
  stub 0202f1b4 ret=none args=v,v,v,v,v
  stub 02017de4 ret=none args=v,v
