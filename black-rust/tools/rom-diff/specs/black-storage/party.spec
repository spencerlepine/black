# fndiff spec: pokeparty.c (S1) = main ARM9 0x0201A954..0x0201AD58, plus the permutation generator
# FUN_0201A858 that precedes it. Grammar: svflow_specs/README.md (generic mode).
# Run: tools/check-fn.sh black-storage party --per-function
#
# PARTY layout (0x534 bytes): +0 u32 capacity, +4 u32 count, +8 + i*0xDC six 0xDC-byte Pokemon
# records, +0x530 u8 bit i = per-slot flag (FUN_0201ACC4 get / FUN_0201ACDC set).
# Every function takes the party as `arg buf 0x540` (0x534 used) with the fixes below keeping the
# capacity/count in 0..6 so no record address leaves the buffer; Pokemon pointers handed to a
# stub are the fixed emulator address of the buffer + 8 + i*0xDC (the port computes the same).
# Callees: FUN_02017E1C (Pokemon get, (mon, id, 0): 0x4C = is-egg flag, 0xA0 = ?, 3/5/0xA9 = ?)
# is stubbed with ret=pick:0/1(/2/3) so "== 1" tests are both ways; FUN_02017528 (clear one record)
# is a stub that fills 0xDC bytes. FUN_02082AC0 (word fill) and the other party functions run as real code.
# Ghidra's arity/names are wrong in places (FUN_0201AB8C "no return" etc.); this follows the disassembly.
area party
set field

# FUN_0201A858(byte *p): next 6-tuple of distinct digits 1..6 in p[0..5] (odometer), p[5] pre-incremented;
# returns 1 on a new tuple, 0 when p[0] runs past 6.
fn 0201a858 ret=bool inputs=300
  nowork
  arg buf 0x10
  fix arg0 0 u8 alt range 1 6 || range 0 7 rep=6,1

# FUN_0201A920(heap): party = alloc(heap, 0x534, 0, "pokeparty.c", 0x41); init(party, 6); return party.
fn 0201a920 ret=ofs:arg1
  nowork
  arg u32 any
  arg buf 0x540
  stub 02030734 ret=ptr:arg1:0:1 args=v,v,v,v,v
  stub 02017528 ret=none args=out:0xdc

# FUN_0201A948(party): FUN_0201A954(party, 6)  (tail call)
fn 0201a948 ret=none
  nowork
  arg buf 0x540
  stub 02017528 ret=none args=out:0xdc

# FUN_0201A954(party, capacity): fill 0x534 bytes with 0, capacity, count 0, flag byte 0, clear 6 records.
fn 0201a954 ret=none
  nowork
  arg buf 0x540
  arg u32 alt range 0 6 || any
  stub 02017528 ret=none args=out:0xdc

# FUN_0201A98C(party, mon): if count < capacity (signed): copy the 0xDC-byte record to slot[count],
# set that slot's flag, count++, return 1; else 0.
fn 0201a98c ret=bool
  nowork
  arg buf 0x540
  arg buf 0xE0
  fix arg0 0 u32 alt range 0 6 || range 0 7
  fix arg0 4 u32 range 0 5

# FUN_0201A9C8(party, idx): remove slot idx: records idx+1..count-1 move down, the last slot is
# cleared (FUN_02017528), count--; returns 1.
fn 0201a9c8 ret=bool
  nowork
  arg buf 0x540
  arg u32 range 0 5
  fix arg0 4 u32 range 0 6
  stub 02017528 ret=none args=out:0xdc

# FUN_0201AA14(party): capacity
fn 0201aa14 ret=u32
  nowork
  arg buf 0x540

# FUN_0201AA18(party): count
fn 0201aa18 ret=u32
  nowork
  arg buf 0x540

# FUN_0201AA1C(party): number of slots with get(mon, 0x4C) != 1
fn 0201aa1c ret=u32
  nowork
  arg buf 0x540
  fix arg0 4 u32 range 0 6
  stub 02017e1c ret=pick:0/1 args=v,v,v

# FUN_0201AA50(party): number of slots for which FUN_0201AD18 (battle-able) is true
fn 0201aa50 ret=u32
  nowork
  arg buf 0x540
  fix arg0 4 u32 range 0 6
  stub 02017e1c ret=pick:0/1 args=v,v,v

# FUN_0201AA88(party): number of slots with get(0x4C) == 1 and get(3) == 0
fn 0201aa88 ret=u32
  nowork
  arg buf 0x540
  fix arg0 4 u32 range 0 6
  stub 02017e1c ret=pick:0/1 args=v,v,v

# FUN_0201AAD4(party): number of slots with get(3) == 1
fn 0201aad4 ret=u32
  nowork
  arg buf 0x540
  fix arg0 4 u32 range 0 6
  stub 02017e1c ret=pick:0/1 args=v,v,v

# FUN_0201AB08(party): index of the first slot for which FUN_0201AD18 is true (count if none);
# tail call of FUN_0201AD58(party, 0x0201AD19)
fn 0201ab08 ret=u32
  nowork
  arg buf 0x540
  fix arg0 4 u32 range 0 6
  stub 02017e1c ret=pick:0/1 args=v,v,v

# FUN_0201AB18(party): same with the callback FUN_0201AD40 (0x0201AD41)
fn 0201ab18 ret=u32
  nowork
  arg buf 0x540
  fix arg0 4 u32 range 0 6
  stub 02017e1c ret=pick:0/1 args=v,v,v

# FUN_0201AB28(party, idx): &party->mon[idx]  (returned as an offset into the party buffer)
fn 0201ab28 ret=ofs:arg0
  nowork
  arg buf 0x540
  arg u32 alt range 0 6 || any

# FUN_0201AB34(party, idx): FUN_0201AD18(&mon[idx]) (0 if the pointer is null, never)
fn 0201ab34 ret=bool
  nowork
  arg buf 0x540
  arg u32 range 0 5
  stub 02017e1c ret=pick:0/1 args=v,v,v

# FUN_0201AB48(party, idx, mon): d = get(mon[idx], 0xA9) - get(src, 0xA9); mon[idx] = *src; count += d
fn 0201ab48 ret=none
  nowork
  arg buf 0x540
  arg u32 range 0 5
  arg buf 0xE0
  stub 02017e1c ret=pick:0/1/2/3 args=v,v,v

# FUN_0201AB8C(party, a, b, heap): swap records a and b through a temporary alloc(heap, 0xDC, 0,
# "pokeparty.c", 382), free it, swap the two slot flags; returns 0.
fn 0201ab8c ret=u32
  nowork
  arg buf 0x540
  arg u32 range 0 5
  arg u32 range 0 5
  arg u32 any
  arg buf 0xE0
  stub 02030734 ret=ptr:arg4:0:1 args=v,v,v,v,v
  stub 020307b0 ret=none args=v

# FUN_0201AC2C(src, dst): copy 0x534 bytes
fn 0201ac2c ret=none
  nowork
  arg buf 0x540
  arg buf 0x540

# FUN_0201AC44(party, v): 1 if some slot has get(mon, 5) == v, else 0
fn 0201ac44 ret=bool
  nowork
  arg buf 0x540
  arg u32 alt range 0 3 || any
  fix arg0 4 u32 range 0 6
  stub 02017e1c ret=pick:0/1/2/3 args=v,v,v

# FUN_0201AC80(party): number of slots with get(0xA0) != 0 and get(0x4C) == 0
fn 0201ac80 ret=u32
  nowork
  arg buf 0x540
  fix arg0 4 u32 range 0 6
  stub 02017e1c ret=pick:0/1 args=v,v,v

# FUN_0201ACC4(party, idx): flag byte (+0x530) bit idx (LSL by register: idx >= 32 gives 0)
fn 0201acc4 ret=bool
  nowork
  arg buf 0x540
  arg u32 alt range 0 7 || near 32 3 || any

# FUN_0201ACDC(party, idx, on): flag byte |= 1 << idx if on == 1, else &= ~(1 << idx)
fn 0201acdc ret=none
  nowork
  arg buf 0x540
  arg u32 alt range 0 7 || near 32 3 || any
  arg u32 oneof 0 1 1 2

# FUN_0201AD18(mon): get(mon, 0x4C) != 1 && get(mon, 0xA0) != 0
fn 0201ad18 ret=bool
  nowork
  arg buf 0xE0
  stub 02017e1c ret=pick:0/1 args=v,v,v

# FUN_0201AD40(mon): get(mon, 0x4C) != 1
fn 0201ad40 ret=bool
  nowork
  arg buf 0xE0
  stub 02017e1c ret=pick:0/1 args=v,v,v

# FUN_0201AD58(party, pred): index of the first slot where pred(&mon) is true (count if none); pred is
# one of the two callbacks above (0x0201AD19 / 0x0201AD41; run as real code under the emulator).
fn 0201ad58 ret=u32
  nowork
  arg buf 0x540
  arg u32 oneof 0x0201ad19 0x0201ad41
  fix arg0 4 u32 range 0 6
  stub 02017e1c ret=pick:0/1 args=v,v,v
