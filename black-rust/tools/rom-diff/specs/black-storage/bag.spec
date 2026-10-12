# fndiff spec: bag / items (S3): the myitem_savedata.c pocket functions 0x02007D00..0x02008216, the
# TM/HM item tests 0x02021288 / 0x0202130C / 0x0200823C, and bag_param.c FUN_0202E474.
# Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage bag --per-function
#
# BAG layout (0x9C0 bytes, FUN_02007CB0): five pockets of 4-byte slots {u16 item, u16 count}, in
# FUN_02007D00's order (pocket id -> offset, slot count): 0 -> 0x000, 310 | 4 -> 0x4D8, 83 |
# 2 -> 0x624, 109 | 1 -> 0x7D8, 48 | 3 -> 0x898, 64.  A slot with item 0 is empty (count 0).
# Pocket ids of an item come from FUN_02020FE4(item, 5, heap) (item data from a NARC; stubbed with
# ret=pick:0/1/2/3/4, mixing item and heap id).  The heap id is a 16-bit stack argument of the
# FUN_02007D6C family (the original reads it with ldrh, the stub is passed the low 16 bits).
# FUN_02007D00 leaves *count untouched for a pocket id > 4 and sets *base = 0.
# The bag fixes plant small item ids (0..6) so the "same item" paths run, and counts around 0..3
# or 16-bit noise; 'n' arguments are kept 1..N for the sorts (n = 0 would loop ~2^32 times in the
# original: n - 1 is an unsigned loop bound).
area bag
set field

# FUN_02007D00(bag, pocket, u32 *base, u32 *count): *base = bag + offset, *count = slot count; returns pocket.
fn 02007d00 ret=u32
  nowork
  arg buf 0xA00
  arg u32 alt range 0 4 || range 0 7 || any
  arg buf 4
  arg buf 4

# FUN_02007D6C(bag, item, &base, &count, heap16): pocket = FUN_02020FE4(item, 5, heap16); FUN_02007D00(bag, pocket, ..).
fn 02007d6c ret=u32
  nowork
  arg buf 0xA00
  arg u32 alt range 0 6 || any
  arg buf 4
  arg buf 4
  arg u32 any
  stub 02020fe4 ret=pick:0/1/2/3/4 args=v,v,v

# FUN_02007D94(base, n, item, add, max16): slot with that item: itself if count + add <= max (signed 16-bit
# max), else null; otherwise the first empty slot (item 0 and count 0), else null.
fn 02007d94 ret=ofs:arg5
  nowork
  arg u32 const 0x02E60000
  arg u32 range 0 0x20
  arg u32 alt range 0 6 || any
  arg u32 alt range 0 3 || any
  arg u32 alt oneof 1 0x3e7 || range 0 6 || any
  arg buf 0x80
  fix arg5 0 u16 range 0 6 rep=32,4
  fix arg5 2 u16 alt range 0 3 || range 0 0x3e7 || any rep=32,4

# FUN_02007DF8(bag, item, add, heap): FUN_02007D94 on the item's pocket, max 1 for pocket 2 else 999; the slot or null.
fn 02007df8 ret=ofs:arg0
  nowork
  arg buf 0xA00
  arg u32 alt range 0 6 || any
  arg u32 alt range 0 3 || any
  arg u32 any
  fix arg0 0 u16 range 0 6 rep=310,4
  fix arg0 2 u16 alt range 0 3 || range 0 0x3e7 || any rep=310,4
  fix arg0 0x4d8 u16 range 0 6 rep=83,4
  fix arg0 0x4da u16 alt range 0 3 || any rep=83,4
  fix arg0 0x624 u16 range 0 6 rep=109,4
  fix arg0 0x626 u16 alt range 0 3 || any rep=109,4
  fix arg0 0x7d8 u16 range 0 6 rep=48,4
  fix arg0 0x7da u16 alt range 0 3 || any rep=48,4
  fix arg0 0x898 u16 range 0 6 rep=64,4
  fix arg0 0x89a u16 alt range 0 3 || any rep=64,4
  stub 02020fe4 ret=pick:0/1/2/3/4 args=v,v,v

# FUN_02007E3C(bag, item, add, heap): FUN_02007DF8 != null  (can the item be added)
fn 02007e3c ret=bool
  nowork
  arg buf 0xA00
  arg u32 alt range 0 6 || any
  arg u32 alt range 0 3 || any
  arg u32 any
  fix arg0 0 u16 range 0 6 rep=310,4
  fix arg0 2 u16 alt range 0 3 || range 0 0x3e7 || any rep=310,4
  fix arg0 0x4d8 u16 range 0 6 rep=83,4
  fix arg0 0x4da u16 alt range 0 3 || any rep=83,4
  fix arg0 0x624 u16 range 0 6 rep=109,4
  fix arg0 0x626 u16 alt range 0 3 || any rep=109,4
  fix arg0 0x7d8 u16 range 0 6 rep=48,4
  fix arg0 0x7da u16 alt range 0 3 || any rep=48,4
  fix arg0 0x898 u16 range 0 6 rep=64,4
  fix arg0 0x89a u16 alt range 0 3 || any rep=64,4
  stub 02020fe4 ret=pick:0/1/2/3/4 args=v,v,v

# FUN_02007E50(bag, item, add, heap): add items: slot = FUN_02007DF8; slot.item = item, slot.count += add; then
# re-sort the pocket (3: FUN_020080A0 by id, 2: FUN_020080FC by TM index) or, if the slot was empty and the
# pocket is not 2/3, move it to the front (FUN_02007F7C count, FUN_02007FF4). Returns 1, or 0 if no slot.
fn 02007e50 ret=bool
  nowork
  arg buf 0xA00
  arg u32 alt range 0 6 || any
  arg u32 alt range 0 3 || any
  arg u32 any
  fix arg0 0 u16 range 0 6 rep=310,4
  fix arg0 2 u16 alt range 0 3 || range 0 0x3e7 || any rep=310,4
  fix arg0 0x4d8 u16 range 0 6 rep=83,4
  fix arg0 0x4da u16 alt range 0 3 || any rep=83,4
  fix arg0 0x624 u16 alt range 0 6 || range 0x148 0x1a9 || range 0x26a 0x26c rep=109,4
  fix arg0 0x626 u16 alt range 0 3 || any rep=109,4
  fix arg0 0x7d8 u16 range 0 6 rep=48,4
  fix arg0 0x7da u16 alt range 0 3 || any rep=48,4
  fix arg0 0x898 u16 alt range 0 6 || range 0x148 0x1a9 rep=64,4
  fix arg0 0x89a u16 alt range 0 3 || any rep=64,4
  stub 02020fe4 ret=pick:0/1/2/3/4 args=v,v,v

# FUN_02007ECC(base, n, item, min): slot holding the item with count >= min, else null.
fn 02007ecc ret=ofs:arg4
  nowork
  arg u32 const 0x02E50000
  arg u32 range 0 0x20
  arg u32 alt range 0 6 || any
  arg u32 alt range 0 3 || any
  arg buf 0x80
  fix arg4 0 u16 range 0 6 rep=32,4
  fix arg4 2 u16 alt range 0 3 || range 0 0x3e7 || any rep=32,4

# FUN_02007EF8(bag, item, min, heap): FUN_02007ECC on the item's pocket; slot or null.
fn 02007ef8 ret=ofs:arg0
  nowork
  arg buf 0xA00
  arg u32 alt range 0 6 || any
  arg u32 alt range 0 3 || any
  arg u32 any
  fix arg0 0 u16 range 0 6 rep=310,4
  fix arg0 2 u16 alt range 0 3 || range 0 0x3e7 || any rep=310,4
  fix arg0 0x4d8 u16 range 0 6 rep=83,4
  fix arg0 0x4da u16 alt range 0 3 || any rep=83,4
  fix arg0 0x624 u16 range 0 6 rep=109,4
  fix arg0 0x626 u16 alt range 0 3 || any rep=109,4
  fix arg0 0x7d8 u16 range 0 6 rep=48,4
  fix arg0 0x7da u16 alt range 0 3 || any rep=48,4
  fix arg0 0x898 u16 range 0 6 rep=64,4
  fix arg0 0x89a u16 alt range 0 3 || any rep=64,4
  stub 02020fe4 ret=pick:0/1/2/3/4 args=v,v,v

# FUN_02007F1C(bag, item, sub, heap): remove: slot = FUN_02007EF8(bag, item, 1, heap) (null: return 0);
# slot.count -= sub, slot.item = 0 if the count is 0; compact the pocket (FUN_02008058); returns 1.
fn 02007f1c ret=bool
  nowork
  arg buf 0xA00
  arg u32 alt range 0 6 || any
  arg u32 alt range 0 3 || any
  arg u32 any
  fix arg0 0 u16 range 0 6 rep=310,4
  fix arg0 2 u16 alt range 0 3 || range 0 0x3e7 || any rep=310,4
  fix arg0 0x4d8 u16 range 0 6 rep=83,4
  fix arg0 0x4da u16 alt range 0 3 || any rep=83,4
  fix arg0 0x624 u16 range 0 6 rep=109,4
  fix arg0 0x626 u16 alt range 0 3 || any rep=109,4
  fix arg0 0x7d8 u16 range 0 6 rep=48,4
  fix arg0 0x7da u16 alt range 0 3 || any rep=48,4
  fix arg0 0x898 u16 range 0 6 rep=64,4
  fix arg0 0x89a u16 alt range 0 3 || any rep=64,4
  stub 02020fe4 ret=pick:0/1/2/3/4 args=v,v,v

# FUN_02007F68(bag, item, min, heap): FUN_02007EF8 != null (has at least `min` of the item)
fn 02007f68 ret=bool
  nowork
  arg buf 0xA00
  arg u32 alt range 0 6 || any
  arg u32 alt range 0 3 || any
  arg u32 any
  fix arg0 0 u16 range 0 6 rep=310,4
  fix arg0 2 u16 alt range 0 3 || range 0 0x3e7 || any rep=310,4
  fix arg0 0x4d8 u16 range 0 6 rep=83,4
  fix arg0 0x4da u16 alt range 0 3 || any rep=83,4
  fix arg0 0x624 u16 range 0 6 rep=109,4
  fix arg0 0x626 u16 alt range 0 3 || any rep=109,4
  fix arg0 0x7d8 u16 range 0 6 rep=48,4
  fix arg0 0x7da u16 alt range 0 3 || any rep=48,4
  fix arg0 0x898 u16 range 0 6 rep=64,4
  fix arg0 0x89a u16 alt range 0 3 || any rep=64,4
  stub 02020fe4 ret=pick:0/1/2/3/4 args=v,v,v

# FUN_02007F7C(base, n): number of slots with a nonzero item id (0 for a null base)
fn 02007f7c ret=u32
  nowork
  arg u32 alt const 0 || const 0x02E30000
  arg u32 range 0 0x20
  arg buf 0x80
  fix arg2 0 u16 alt range 0 3 || any rep=32,4

# FUN_02007FA0(bag, pocket): FUN_02007F7C over the pocket (an invalid pocket gives base 0 -> 0)
fn 02007fa0 ret=u32
  nowork
  arg buf 0xA00
  arg u32 alt range 0 4 || any
  fix arg0 0 u16 alt range 0 3 || any rep=310,4
  fix arg0 0x4d8 u16 alt range 0 3 || any rep=83,4
  fix arg0 0x624 u16 alt range 0 3 || any rep=109,4
  fix arg0 0x7d8 u16 alt range 0 3 || any rep=48,4
  fix arg0 0x898 u16 alt range 0 3 || any rep=64,4

# FUN_02007FB8(bag, item, heap): count of the item = FUN_02007EF8(bag, item, 1, heap)->count, 0 if none
fn 02007fb8 ret=u16
  nowork
  arg buf 0xA00
  arg u32 alt range 0 6 || any
  arg u32 any
  fix arg0 0 u16 range 0 6 rep=310,4
  fix arg0 2 u16 alt range 0 3 || range 0 0x3e7 || any rep=310,4
  fix arg0 0x4d8 u16 range 0 6 rep=83,4
  fix arg0 0x4da u16 alt range 0 3 || any rep=83,4
  fix arg0 0x624 u16 range 0 6 rep=109,4
  fix arg0 0x626 u16 alt range 0 3 || any rep=109,4
  fix arg0 0x7d8 u16 range 0 6 rep=48,4
  fix arg0 0x7da u16 alt range 0 3 || any rep=48,4
  fix arg0 0x898 u16 range 0 6 rep=64,4
  fix arg0 0x89a u16 alt range 0 3 || any rep=64,4
  stub 02020fe4 ret=pick:0/1/2/3/4 args=v,v,v

# FUN_02007FD0(a, b): swap two {u16, u16} slots
fn 02007fd0 ret=none
  nowork
  arg buf 8
  arg buf 8

# FUN_02007FF4(base, n): n >= 2: temp = slot[n-1]; walking down, slot[i+1] = slot[i] once temp holds an
# item (else temp = slot[i]); slot[0] = temp  (n < 2: nothing). Writes slot[n].
fn 02007ff4 ret=none
  nowork
  arg buf 0x110
  arg u32 alt range 0 0x40 || range 0 3
  fix arg0 0 u16 alt range 0 3 || any rep=65,4

# FUN_02008038(base, n): zero the count of every slot whose item id is 0
fn 02008038 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 0x20
  fix arg0 0 u16 alt range 0 3 || any rep=32,4

# FUN_02008058(base, n): FUN_02008038, then move empty-count slots (count 0) behind the used ones (selection
# swaps FUN_02007FD0); n >= 1
fn 02008058 ret=none
  nowork
  arg buf 0x80
  arg u32 range 1 0x20
  fix arg0 0 u16 alt range 0 3 || any rep=32,4
  fix arg0 2 u16 alt range 0 3 || any rep=32,4

# FUN_020080A0(base, n): FUN_02008038, then sort used slots by ascending item id (empty ones last); n >= 1
fn 020080a0 ret=none
  nowork
  arg buf 0x80
  arg u32 range 1 0x20
  fix arg0 0 u16 alt range 0 8 || any rep=32,4
  fix arg0 2 u16 alt range 0 3 || any rep=32,4

# FUN_020080FC(base, n): like FUN_020080A0 but ordered by FUN_0202130C(item) (TM/HM number); n >= 1
fn 020080fc ret=none
  nowork
  arg buf 0x80
  arg u32 range 1 0x20
  fix arg0 0 u16 alt range 0 3 || range 0x140 0x1b0 || range 0x268 0x26d || any rep=32,4
  fix arg0 2 u16 alt range 0 3 || any rep=32,4

# FUN_0200816C(bag): FUN_02008058 over the pockets in the order 4, 0, 3, 1, 2
fn 0200816c ret=none inputs=12
  nowork
  arg buf 0xA00
  fix arg0 0 u16 range 0 6 rep=310,4
  fix arg0 2 u16 alt range 0 3 || any rep=310,4
  fix arg0 0x4d8 u16 range 0 6 rep=83,4
  fix arg0 0x4da u16 alt range 0 3 || any rep=83,4
  fix arg0 0x624 u16 range 0 6 rep=109,4
  fix arg0 0x626 u16 alt range 0 3 || any rep=109,4
  fix arg0 0x7d8 u16 range 0 6 rep=48,4
  fix arg0 0x7da u16 alt range 0 3 || any rep=48,4
  fix arg0 0x898 u16 range 0 6 rep=64,4
  fix arg0 0x89a u16 alt range 0 3 || any rep=64,4

# FUN_020081DC(bag, pocket, idx, size): &slot[idx] of the pocket, null if idx >= the pocket's slot count;
# an invalid pocket id (> 4) uses the whole bag with the incoming r3 as the slot count.
fn 020081dc ret=ofs:arg0
  nowork
  arg buf 0xA00
  arg u32 alt range 0 4 || range 0 6 || any
  arg u32 alt range 0 0x140 || any
  arg u32 alt range 0 0x20 || any

# FUN_02008216(base, ofs, idx, n): base + ofs + idx * 4 if idx < n, else null
fn 02008216 ret=ofs:arg0
  nowork
  arg buf 0x40
  arg u32 alt range 0 0x30 || any
  arg u32 alt range 0 0x10 || any
  arg u32 alt range 0 0x10 || any

# FUN_0200823C(item): 1 if the item is a TM/HM (FUN_02021288), else 999 (max stack)
fn 0200823c ret=u32
  nowork
  arg u32 alt near 0x148 3 || near 0x1a9 3 || near 0x26a 4 || any

# FUN_02021288(item): TM/HM test: 0x148..=0x1A9 or 0x26A..=0x26C
fn 02021288 ret=bool
  nowork
  arg u32 alt near 0x148 3 || near 0x1a9 3 || near 0x26a 4 || any

# FUN_0202130C(item): TM/HM index byte: 0xFF if not a TM/HM; item - 0x20E for >= 0x26A, item - 0x145 for
# 0x1A4.. (0x26A - 0xC6), else item - 0x148
fn 0202130c ret=u32
  nowork
  arg u32 alt near 0x148 3 || near 0x1a4 6 || near 0x26a 4 || any

# FUN_0202E474(gamedata, src, v, heap): bag_param.c constructor. alloc(heap, 0x3c, 1, "bag_param.c", 0x37);
# p[0] = gamedata, p[1] = FUN_0200873C(FUN_02012EBC(gamedata)), p[2] = FUN_02012944(gamedata),
# p[3] = FUN_02012924, p[4] = FUN_0201292C, if src != 0: 0x18 bytes at p+0x14 = *src, p[0xB] = v,
# FUN_020127A4(gamedata), p[0xC] = (FUN_02012B28(..) == 1). The callees are accessors of the game
# data root; stubbed (their meaning is F0's).  Returns the new struct (ofs into arg4).
fn 0202e474 ret=ofs:arg4
  nowork
  arg u32 any
  arg u32 alt const 0 || const 0x02E60000
  arg u32 any
  arg u32 any
  arg buf 0x40
  arg buf 0x18
  stub 02012ebc ret=u32 args=v
  stub 020071cc ret=u32 args=v,v
  stub 02012944 ret=u32 args=v
  stub 02012924 ret=u32 args=v
  stub 0201292c ret=u32 args=v
  stub 020127a4 ret=u32 args=v
  stub 02012b28 ret=pick:0/1 args=v
  stub 02030734 ret=ptr:arg4:0:1 args=v,v,v,v,v
