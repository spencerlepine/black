# fndiff spec: strbuf.c (F9): the string-buffer functions of main ARM9 0x02045730..0x02045AFC (ARM
# mode) plus the msgdata.c get-string path (FUN_02045CAC / FUN_02045E04 / FUN_02045E3C) restricted
# to the preloaded-table case (MsgData ARC loading, file reads and the language reload are stubbed
# or fixed away).  Grammar: svflow_specs/README.md (generic mode, `mode=arm`).
# Run: tools/check-fn.sh black-text strbuf --per-function
#
# STRBUF layout: +0 u16 capacity (in characters, incl. the terminator), +2 u16 length, +4 u32 magic
# 0xB6F8D2EC (written by FUN_020457B0), +8 u16 chars[], terminated by EOS = the u16 at 0x020A7E7C
# (0xFFFF in the ROM image; FUN_020457A0 overwrites it).  Buffers are 0x60 bytes (44 chars); the
# fixes keep capacity and length <= 20 and the characters in a tiny alphabet plus EOS, so the
# copy/compare loops terminate inside the buffer.
# MSGDATA (FUN_02045B38, not specified): +0 -> header {?, u16 count@2, ..}, +4 -> preloaded table
# {u32 ?, then 8-byte entries {u32 offset, u16 length, u16 ?} from +4; strings at table + offset},
# +8 entry cache (non-preloaded path), +0x10/+0x14 file offset/handle, +0x18 u16 heap, +0x1a u8 bank,
# +0x1b u8 preloaded flag.  FUN_02045CAC is only specified with the flag set (fix below).
# The text-parameter accessors at 0x02045730..0x02045798 belong to the same file (a small byte-field
# record; struct name UNVERIFIED).
area strbuf
set field

# ---- record accessors (FUN_02045730 ..)
fn 02045730 mode=arm ret=u8
  nowork
  arg buf 0x10

fn 02045738 mode=arm ret=u8
  nowork
  arg buf 0x10

fn 02045740 mode=arm ret=u8
  nowork
  arg buf 0x10

fn 02045748 mode=arm ret=u8
  nowork
  arg buf 0x10

fn 02045750 mode=arm ret=u8
  nowork
  arg buf 0x10

fn 02045758 mode=arm ret=u8
  nowork
  arg buf 0x10

fn 02045760 mode=arm ret=u8
  nowork
  arg buf 0x10

fn 02045768 mode=arm ret=u16
  nowork
  arg buf 0x10

fn 02045770 mode=arm ret=u32
  nowork
  arg buf 0x10

fn 02045778 mode=arm ret=u8
  nowork
  arg buf 0x10

# setters: byte at +3 / +4 / +8 / +9
fn 02045780 mode=arm ret=none
  nowork
  arg buf 0x10
  arg u8 any

fn 02045788 mode=arm ret=none
  nowork
  arg buf 0x10
  arg u8 any

fn 02045790 mode=arm ret=none
  nowork
  arg buf 0x10
  arg u8 any

fn 02045798 mode=arm ret=none
  nowork
  arg buf 0x10
  arg u8 any

# FUN_020457A0(c): *(u16 *)0x020A7E7C = c  (the end-of-string marker)
fn 020457a0 mode=arm ret=none
  nowork
  arg u16 any
  global 020a7e7c 4

# FUN_020457B0(len, heap): p = alloc(heap, len * 2 + 10, 0, "strbuf.c", 100); p->capacity = len; p->length = 0;
# p->magic = 0xB6F8D2EC; p->chars[0] = EOS; return p (the allocator returns the second buffer).
fn 020457b0 mode=arm ret=ofs:arg2
  nowork
  arg u32 alt range 0 20 || any
  arg u32 any
  arg buf 0x60
  stub 02030734 ret=ptr:arg2:0:1 args=v,v,v,v,v

# FUN_02045808(sb): sb->magic = 0; Heap_Free(sb)  (tail call)
fn 02045808 mode=arm ret=none
  nowork
  arg buf 0x60
  stub 020307b0 ret=none args=v

# FUN_0204581C(sb): length = 0; chars[0] = EOS
fn 0204581c mode=arm ret=none
  nowork
  arg buf 0x60
  fix arg0 2 u16 range 0 20

# FUN_02045838(dst, src): if dst.capacity > src.length: copy (src.length + 1) chars, dst.length = src.length
fn 02045838 mode=arm ret=none
  nowork
  arg buf 0x60
  arg buf 0x60
  fix arg0 0 u16 range 0 20
  fix arg1 2 u16 range 0 20
  fix arg0 8 u16 alt range 0 3 || const 0xffff rep=20,2
  fix arg1 8 u16 alt range 0 3 || const 0xffff rep=20,2

# FUN_02045874(src, heap): copy = FUN_020457B0(src.length + 1, heap) then FUN_02045838(copy, src); return copy
fn 02045874 mode=arm ret=ofs:arg2
  nowork
  arg buf 0x60
  arg u32 any
  arg buf 0x60
  fix arg0 2 u16 range 0 20
  fix arg0 8 u16 alt range 0 3 || const 0xffff rep=21,2
  stub 02030734 ret=ptr:arg2:0:1 args=v,v,v,v,v

# FUN_0204589C(a, b): 1 if the strings are equal up to the terminator (compares chars from index 0)
fn 0204589c mode=arm ret=bool
  nowork
  arg buf 0x60
  arg buf 0x60
  fix arg0 8 u16 alt range 0 2 || const 0xffff rep=40,2
  fix arg1 8 u16 alt range 0 2 || const 0xffff rep=40,2

# FUN_020458F8(sb): length
fn 020458f8 mode=arm ret=u16
  nowork
  arg buf 0x60

# FUN_02045900(sb, i): if i < length: chars[i] = EOS, length = i
fn 02045900 mode=arm ret=none
  nowork
  arg buf 0x60
  arg u32 alt range 0 24 || any
  fix arg0 2 u16 range 0 20

# FUN_02045924(sb, src): copy u16 chars from src until EOS or capacity - 1 (signed), terminate; length = count
fn 02045924 mode=arm ret=none
  nowork
  arg buf 0x60
  arg buf 0x60
  fix arg0 0 u16 range 0 20
  fix arg1 0 u16 alt range 0 3 || const 0xffff rep=40,2

# FUN_02045A00(sb, src, n): if n != 0 and n <= capacity: copy n chars from src, length = n - 1
fn 02045a00 mode=arm ret=none
  nowork
  arg buf 0x60
  arg buf 0x60
  arg u32 alt range 0 24 || any
  fix arg0 0 u16 range 0 20

# FUN_02045A38(sb, dst, n): copy chars until EOS (inclusive) or n; if n chars were copied, dst[n-1] = EOS
fn 02045a38 mode=arm ret=none
  nowork
  arg buf 0x60
  arg buf 0x60
  arg u32 alt range 1 20 || range 1 40
  fix arg0 8 u16 alt range 0 3 || const 0xffff rep=40,2

# FUN_02045A90(sb): &sb->chars  (as an offset)
fn 02045a90 mode=arm ret=ofs:arg0
  nowork
  arg buf 0x60

# FUN_02045A98(): the end-of-string marker (u16 at 0x020A7E7C)
fn 02045a98 mode=arm ret=u16
  nowork

# FUN_02045AA8(dst, src): if dst.length + src.length + 1 <= dst.capacity: append src (with terminator), dst.length += src.length
fn 02045aa8 mode=arm ret=none
  nowork
  arg buf 0x60
  arg buf 0x60
  fix arg0 0 u16 range 0 20
  fix arg0 2 u16 range 0 12
  fix arg1 2 u16 range 0 12
  fix arg0 8 u16 alt range 0 3 || const 0xffff rep=20,2
  fix arg1 8 u16 alt range 0 3 || const 0xffff rep=20,2

# FUN_02045AFC(sb, c): if length + 1 < capacity (signed): chars[length++] = c; chars[length] = EOS
fn 02045afc mode=arm ret=none
  nowork
  arg buf 0x60
  arg u16 alt range 0 3 || any
  fix arg0 0 u16 range 0 20
  fix arg0 2 u16 range 0 20

# FUN_02045E3C(chars, n, id): decrypt n chars in place: key = ((id + 3) * 0x2983) & 0xFFFF, xor each char with
# the key (key rotates by 3 bits) and stop at EOS; returns the index of the EOS, else n (0 for n = 0).
# id 0xFFFD makes the initial key 0, so a stored 0xFFFF decrypts to EOS at once.
fn 02045e3c mode=arm ret=u32
  nowork
  arg buf 0x60
  arg u32 alt range 0 40 || range 0 3
  arg u32 alt any || const 0xfffd
  fix arg0 0 u16 alt const 0xffff || any rep=40,2

# FUN_02045E04(sb, id): n = FUN_02045E3C(sb.chars, sb.length + 1, id); FUN_02045900(sb, n)
fn 02045e04 mode=arm ret=none
  nowork
  arg buf 0x60
  arg u32 alt any || const 0xfffd
  fix arg0 2 u16 range 0 20
  fix arg0 8 u16 alt const 0xffff || any rep=40,2

# FUN_02045CAC(msgdata, id, sb), preloaded case only: if id >= header.count: FUN_0204581C(sb); else the entry
# {offset, length} at table + 4 + id * 8: FUN_02045A00(sb, table + offset, length) then FUN_02045E04(sb, id).
# FUN_02046180 (reload the bank if the language changed) is stubbed.
fn 02045cac mode=arm ret=none
  nowork
  arg buf 0x40
  arg u32 alt range 0 3 || range 0 5
  arg buf 0x60
  arg buf 0x10
  arg buf 0x100
  fix arg0 0 u32 addrof arg3
  fix arg0 4 u32 addrof arg4
  fix arg0 0x1b u8 const 1
  fix arg3 2 u16 range 0 4
  fix arg4 4 u32 range 0x30 0x80 rep=4,8
  fix arg4 8 u16 alt range 0 12 || range 0 20 rep=4,8
  fix arg2 0 u16 alt range 0 16 || range 0 24
  fix arg2 2 u16 range 0 20
  stub 02046180 ret=none args=p
