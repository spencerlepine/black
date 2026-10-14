# fndiff spec (S5b): the shortcut save block 54 of the main ARM9 (0x0200C3F4..0x0200C520): 0x1A id bytes,
# 25 = free. Grammar: svflow_specs/README.md (generic mode). Run: tools/check-fn.sh black-storage shortcut --per-function
# The list bytes are drawn from 0..25 (mostly ids that repeat, often free) so the searches hit and miss.
# MI_CpuFill8 runs as real code.
area shortcut
set field

fn 0200c3f4 ret=u32 inputs=10
  nowork

# sub_0200C3F8(list): fill 25
fn 0200c3f8 ret=none inputs=50
  nowork
  arg buf 0x20

# FUN_0200C418(list, id, on): add / remove an id, then pack
fn 0200c418 ret=none inputs=1500
  nowork
  arg buf 0x20
  arg u8 alt range 0 26 || any
  arg u32 alt oneof 0 1 || any
  fix arg0 0 u8 alt range 0 8 || oneof 25 || any rep=26,1

# FUN_0200C44C(list, id): listed?
fn 0200c44c ret=bool inputs=400
  nowork
  arg buf 0x20
  arg u8 alt range 0 26 || any
  fix arg0 0 u8 alt range 0 8 || oneof 25 || any rep=26,1

# FUN_0200C464(list, i): entry i
fn 0200c464 ret=u8 inputs=100
  nowork
  arg buf 0x20
  arg u32 range 0 25

# FUN_0200C468(list): used entries
fn 0200c468 ret=u16 inputs=400
  nowork
  arg buf 0x20
  fix arg0 0 u8 alt range 0 8 || oneof 25 || any rep=26,1

# FUN_0200c480(list, id, pos): move / add id to pos (0xFF = append); pos 0..=25 or 0xFF only
fn 0200c480 ret=u32 inputs=2000
  nowork
  arg buf 0x20
  arg u8 alt range 0 26 || any
  arg u32 alt range 0 25 || oneof 0xff
  fix arg0 0 u8 alt range 0 8 || oneof 25 || range 0 30 rep=26,1

# FUN_0200C4E8(list): bubble the free entries back
fn 0200c4e8 ret=none inputs=600
  nowork
  arg buf 0x20
  fix arg0 0 u8 alt range 0 8 || oneof 25 || any rep=26,1
