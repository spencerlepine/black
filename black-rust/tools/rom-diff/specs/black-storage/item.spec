# fndiff spec (S4): item data accessors and item classification, item.c of the main ARM9
# (0x02020F34..0x02021414). Run: tools/check-fn.sh black-storage item --per-function
#
# Item record = 36 bytes (ARC 024). Usage block = record bytes 0x10..0x24; its kind byte is record
# byte 0xC (0: one byte at 0x10, 1: parameter block read by FUN_020210C8, else 0). The rodata tables
# the originals read (TM/HM move table 0x0209EAA0, mail list 0x0209E8EC, pair table 0x0209E904,
# berry list 0x0209E968, graphics index table 0x0209EB6C) are `global`s: random on the ROM side, the
# same bytes handed to the port. List/pair entries are kept small (fix ... rep) so the item and
# move arguments, drawn from 0..9 half of the time, hit them.
# The archive loader FUN_02048F60 is stubbed (its result is a fixed buffer address or a random
# word), as are Heap_Free (FUN_020307B0); FUN_02020F34, FUN_02021014, FUN_020210C8 and FUN_02021288
# run as real code on both sides.
area item
set field

# FUN_02020F34(item, kind, heap): archive 0x18 (kind 0) or 0x19 (kinds 1, 2; member from the graphics
# index table), 0 for any other kind.
fn 02020f34 ret=u32 inputs=800
  nowork
  arg u32 alt range 0 0x280 || near 0x272 4 || any
  arg u32 alt range 0 3 || any
  arg u32 any
  global 0209eb6c 0x9d8
  stub 02048f60 ret=u32 args=v,v,v,v

# FUN_02021014(record, param): record fields 0..0x11, usage block fields above (its tail 0x020210AA
# is the usage-kind switch; not a function of its own: it has no prologue).
fn 02021014 ret=u32 inputs=2000
  nowork
  arg buf 36
  arg u32 alt range 0 0x41 || any
  fix arg0 0xc u8 alt const 0 || const 1 || const 1 || const 1 || any

# FUN_020210C8(block, param): usage block field 0x12..0x3E, else 0.
fn 020210c8 ret=u32 inputs=2000
  nowork
  arg buf 20
  arg u32 alt range 0x10 0x41 || any

# FUN_02020FE4(item, param, heap): load the record (stubbed to buffer arg3), read the parameter, free it.
fn 02020fe4 ret=u32 inputs=1500
  nowork
  arg u32 alt range 0 0x280 || any
  arg u32 alt range 0 0x41 || any
  arg u32 any
  arg buf 36
  fix arg3 0xc u8 alt const 0 || const 1 || const 1 || any
  stub 02048f60 ret=pick:0x02E40000 args=v,v,v,v
  stub 020307b0 ret=none args=v

# FUN_020212B0(item, tm_moves): the move of a TM/HM item.
fn 020212b0 ret=u32 inputs=1500
  nowork
  arg u32 alt range 0x140 0x1B0 || range 0x265 0x270 || any
  global 0209eaa0 0xcc

# FUN_020212E4(move, tm_moves): is the move one of the six HM moves (slots 92..97).
fn 020212e4 ret=bool inputs=800
  nowork
  arg u32 alt range 0 9 || any
  global 0209eaa0 0xcc
  fix global0 0xb8 u16 range 0 8 rep=6,2

# FUN_02021350(item): HM number of an HM item, 0xFF otherwise.
fn 02021350 ret=u32 inputs=500
  nowork
  arg u32 alt range 0x19e 0x1b0 || any

# FUN_0202136C(item, mails): is the item in the 12-entry mail list.
fn 0202136c ret=bool inputs=800
  nowork
  arg u32 alt range 0 9 || any
  global 0209e8ec 0x18
  fix global0 0 u16 range 0 8 rep=12,2

# FUN_0202138C(item, mails): index in the mail list, 0 if absent.
fn 0202138c ret=u32 inputs=800
  nowork
  arg u32 alt range 0 9 || any
  global 0209e8ec 0x18
  fix global0 0 u16 range 0 8 rep=12,2

# FUN_020213AC(index, mails): the mail item of an index below 12, else 0.
fn 020213ac ret=u32 inputs=500
  nowork
  arg u32 alt range 0 14 || any
  global 0209e8ec 0x18

# FUN_020213C0(item, berries): is the item in the 64-entry berry list.
fn 020213c0 ret=bool inputs=800
  nowork
  arg u32 alt range 0 9 || any
  global 0209e968 0x80
  fix global0 0 u16 range 0 8 rep=64,2

# FUN_020213E0(item): 0 for items 0x1EC..=0x1F4 (16-bit wrap), else 1.
fn 020213e0 ret=u32 inputs=500
  nowork
  arg u32 alt range 0x1e8 0x1f8 || near 0x1ec 0x10000 || any

# FUN_020213F8(value, col, pairs): the other column of the row holding value in column col, else 0.
fn 020213f8 ret=u32 inputs=800
  nowork
  arg u32 alt range 0 9 || any
  arg u32 alt const 0 || const 1
  global 0209e904 0x64
  fix global0 0 u16 range 0 8 rep=50,2
