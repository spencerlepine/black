# fndiff spec (S4b): item helpers of the bag overlay 70 (0x021B619C, 0x021BA1D0).
# Run: tools/check-fn.sh black-storage item_bag --per-function
#
# FUN_02020FE4 (parameter reader), FUN_02020F34 (record loader), FUN_02021014 and the TM helpers
# FUN_02021288 / FUN_020212B0 run as real code; the archive loader FUN_02048F60 is stubbed to hand back a
# 36-byte record buffer argument (the item.spec trick) and Heap_Free is a stub. The TM/HM move table
# (0x0209EAA0, 101 halfwords) is a `global`: random on the ROM side, the same bytes handed to the port;
# it is planted mostly with the six field HM moves and their neighbours so each comparison is hit.
area item_bag
set bag

# FUN_overlay_d_70__021b619c(item, count, heap): count * (price / 2) with price = item parameter 0.
fn 021b619c ret=u32 inputs=3000
  nowork
  arg u32 alt range 0 0x280 || any
  arg s32 alt range 0 999 || range -3 3 || any
  arg u32 any
  arg buf 36
  fix arg3 0 u16 alt range 0 300 || any
  fix arg3 0xc u8 alt const 0 || const 1 || any
  stub 02048f60 ret=pick:0x02E40000 args=v,v,v,v
  stub 020307b0 ret=none args=v

# FUN_overlay_d_70__021ba1d0(item): TM/HM item whose move is Cut, Fly, Surf, Strength, Waterfall or Dive.
fn 021ba1d0 ret=u32 inputs=3000
  nowork
  arg u16 alt range 0x140 0x1b0 || range 0x265 0x270 || any
  global 0209eaa0 0xcc
  fix global0 0 u16 alt oneof 15 19 57 70 127 291 || oneof 14 16 18 20 56 58 69 71 126 128 290 292 0x14 0x39 0x46 || range 0 300 || any rep=101,2
