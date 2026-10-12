# fndiff spec: event flags and vars + the 0x14-byte "field_status" record (F0-spec), main ARM9 Thumb
# code 0x020142D0..0x02014488 (the unit T1 calls `unk_02014128`; its first half, 0x02014128..0x020142C8, is the LOCATION
# record and is specced in zonedata.spec).  Grammar: svflow_specs/README.md
# (generic mode).  Run: tools/check-fn.sh black-field event_work --per-function
#
# EVENT WORK layout (the object GAMEDATA+0x1AC points to, see game_data.spec; every function takes it as arg0):
#   +0x000 .. +0x27C   vars: u16 each; var id V >= 0x4000 is the halfword at (V - 0x4000) * 2     (FUN_020143CC)
#                      (the address is not range checked; 0x27C / 2 = 318 vars fit before the flags)
#   +0x27C .. +0x3E8   flags: bit (id & 7) of the byte at 0x27C + (id >> 3); ids 1..0xB5F are valid
#                      (0x16C bytes = 2912 flags; id 0 and id >> 3 >= 0x16C give no address)  (FUN_02014388)
#   +0x3E8             u8 "temp flags were cleared" marker (FUN_020143F0 sets it, FUN_0201440C reads and clears it)
# Flag ids 0x4000..0x403F (idx = (id - 0x4000) / 8, 0..7, signed round-towards-zero shift) are not in the
# object: they are bits of the 8-byte RAM table at 0x02146268 (system flags that are not saved).  The original
# does not range check against the other end, so an id >= 0x80004000 (negative idx) addresses memory below the
# table; the specs stay below 0x80000000 for the functions that dereference.
# Temp flag range (FUN_020143F0): ids 0xAA0..0xB5F (192 flags, the last 24 bytes of the flag block).
# Flag 0x96D is tested by the battle setup init (btl_setup.spec FUN_02013360).
#
# FIELD_STATUS (0x14 bytes, "field_status.c", FUN_02014428 allocates it): +0 u32, +4 u8, +5 u8, +6 u8, +8 u16,
# +0x11 u8 (FUN_0201447C swaps it), +0x12 u16 (cleared by the allocator); the accessors below are 1:1 get/set.
#
# Not specced (callbacks of the script/task queue that only pass pointers between unported callees, UI side):
# 0x02014488 .. 0x020147B0 (sub_02014488, FUN_02014494, FUN_020144DC, sub_0201450C, FUN_02014554, sub_02014584,
# FUN_020145CC, sub_02014628, FUN_0201464C, FUN_02014678, FUN_020146EA, FUN_02014706, FUN_02014728, FUN_0201472C,
# sub_02014758, FUN_020147A6, sub_020147B0).
area event_work
set field

# FUN_020142D0(): size of the event work object, 0x3EA
fn 020142d0 ret=u32 inputs=100
  nowork

# FUN_020142D8(ew): MI_CpuFill8(ew, 0, 0x3EA): zero the whole object
fn 020142d8 ret=none inputs=100
  nowork
  arg buf 0x400

# FUN_02014388(ew, id): address of the flag byte (see layout); 0 for id 0 or an out-of-range flag id.
# Returned as an address: the table address 0x02146268 + idx for ids >= 0x4000 (idx < 8, signed).
fn 02014388 ret=u32 inputs=600
  nowork
  arg buf 0x400
  arg u32 alt range 0 0x20 || near 0xB60 0x20 || range 0 0xB5F || near 0x4000 0x50 || range 0x4000 0x4040 || oneof 0x7FFF 0x8000 0xFFFF 0x80004000 0x80004008 0xFFFFFFFF 0xFFFF4000 0xC0000000 || any

# FUN_020142E8(ew, id): 1 if the flag byte exists and its bit is set
fn 020142e8 ret=bool inputs=600
  nowork
  arg buf 0x400
  arg u32 alt range 0 0x20 || near 0xB60 0x20 || range 0 0xB5F || near 0x4000 0x50 || range 0x4000 0x4040 || oneof 0x7FFF 0x8000 0xFFFF 0x7FFFFFFF
  global 02146268 0x10

# FUN_02014314(ew, id): set the flag bit (no-op when the flag has no address)
fn 02014314 ret=none inputs=600
  nowork
  arg buf 0x400
  arg u32 alt range 0 0x20 || near 0xB60 0x20 || range 0 0xB5F || near 0x4000 0x50 || range 0x4000 0x4040 || oneof 0x7FFF 0x8000 0xFFFF 0x7FFFFFFF
  global 02146268 0x10

# FUN_0201433C(ew, id): clear the flag bit (no-op when the flag has no address)
fn 0201433c ret=none inputs=600
  nowork
  arg buf 0x400
  arg u32 alt range 0 0x20 || near 0xB60 0x20 || range 0 0xB5F || near 0x4000 0x50 || range 0x4000 0x4040 || oneof 0x7FFF 0x8000 0xFFFF 0x7FFFFFFF
  global 02146268 0x10

# FUN_02014368(ew, lo, hi): FUN_0201433C(ew, id & 0xFFFF) for id = lo..=hi (signed compare; empty when lo > hi)
fn 02014368 ret=none inputs=300
  nowork
  arg buf 0x400
  arg s32 alt range 0 0x30 || near 0xAA0 0x30 || near 0x4000 0x10 || range 0 0xB60
  arg s32 alt range 0 0x30 || near 0xB5F 0x30 || near 0x4000 0x10 || range 0 0xB60
  global 02146268 0x10

# FUN_020143CC(ew, id): address of var `id` = ew + (id - 0x4000) * 2 (no range check)
fn 020143cc ret=u32 inputs=300
  nowork
  arg buf 0x400
  arg u32 alt range 0x4000 0x4140 || any

# FUN_020143D8(ew, first, end): MI_CpuFill8(var address of `first`, 0, (end - first) * 2): clears vars first..end-1
fn 020143d8 ret=none inputs=300
  nowork
  arg buf 0x400
  arg u32 range 0x4000 0x413E
  arg u32 rel 1 range 0 0x30

# FUN_020143F0(ew): clear flags 0xAA0..0xB5F (FUN_02014368), then marker byte at +0x3E8 = 1
fn 020143f0 ret=none inputs=100
  nowork
  arg buf 0x400
  global 02146268 0x10

# FUN_0201440C(ew): returns the marker byte at +0x3E8 and clears it
fn 0201440c ret=u8 inputs=100
  nowork
  arg buf 0x400

# FUN_0201441C(i): byte i of the ROM table at 0x0209E050 (read from the image)
fn 0201441c ret=u8 inputs=100
  nowork
  arg u32 range 0 0x40

# FUN_02014428(heap): alloc(heap, 0x14, 1, "field_status.c", 0x33) then +0x12 = 0 (u16); returns the block
fn 02014428 ret=ofs:arg1 inputs=100
  nowork
  arg u32 any
  arg buf 0x40
  stub 02030734 ret=ptr:arg1:0:1 args=v,v,v,v,v

# FUN_02014444(block): tail call heap free(block)
fn 02014444 ret=none inputs=100
  nowork
  arg u32 any
  stub 020307b0 ret=none args=v

# FUN_0201444C(fs): u32 at +0
fn 0201444c ret=u32 inputs=100
  nowork
  arg buf 0x20

# FUN_02014450(fs, v): u32 at +0 = v
fn 02014450 ret=none inputs=100
  nowork
  arg buf 0x20
  arg u32 any

# FUN_02014454(fs, v): u8 at +4 = v
fn 02014454 ret=none inputs=100
  nowork
  arg buf 0x20
  arg u8 any

# FUN_02014458(fs): u8 at +4
fn 02014458 ret=u8 inputs=100
  nowork
  arg buf 0x20

# FUN_0201445C(fs, v): u8 at +5 = v
fn 0201445c ret=none inputs=100
  nowork
  arg buf 0x20
  arg u8 any

# FUN_02014460(fs): u8 at +5
fn 02014460 ret=u8 inputs=100
  nowork
  arg buf 0x20

# FUN_02014464(fs, v): u8 at +6 = v
fn 02014464 ret=none inputs=100
  nowork
  arg buf 0x20
  arg u8 any

# FUN_02014468(fs): u8 at +6
fn 02014468 ret=u8 inputs=100
  nowork
  arg buf 0x20

# FUN_0201446C(fs, v): u16 at +8 = v
fn 0201446c ret=none inputs=100
  nowork
  arg buf 0x20
  arg u16 any

# FUN_02014470(fs): u16 at +8
fn 02014470 ret=u16 inputs=100
  nowork
  arg buf 0x20

# FUN_02014474(fs, v): u16 at +0x12 = v
fn 02014474 ret=none inputs=100
  nowork
  arg buf 0x20
  arg u16 any

# FUN_02014478(fs): u16 at +0x12
fn 02014478 ret=u16 inputs=100
  nowork
  arg buf 0x20

# FUN_0201447C(fs, v): returns the old u8 at +0x11 and stores v there
fn 0201447c ret=u8 inputs=100
  nowork
  arg buf 0x20
  arg u8 any

# FUN_02014484(fs): u8 at +0x11
fn 02014484 ret=u8 inputs=100
  nowork
  arg buf 0x20
