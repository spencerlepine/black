# svflow spec: cmd_queue (0x021CC8D8-0x021CD800). Harness-job owned; porting workers may not edit.
# Grammar: svflow_specs/README.md
area cmd_queue

fn 021cc8d8 ret=u32
  arg buf 4

fn 021cc918 ret=u32
  arg buf 4
  arg u32 any

fn 021cc95c ret=none
  arg buf 4
  arg u32 any

fn 021cc960 ret=u32
  arg buf 4

fn 021cc96c ret=u32
  arg buf 4

fn 021cc974 ret=none
  arg buf 4
  arg u32 any

fn 021cc99c ret=u32
  arg buf 4

# byte queue: [0]=write count, [4]=read cursor, bytes from +8; capacity check count < 3000
fn 021ccaa8 ret=none
  arg buf 0xC00
  arg u8 any
  fix arg0 0 u32 near 3000 12
  stub 02034714 noreturn ret=none args=v,v,v

fn 021ccad4 ret=u8
  arg buf 0x400
  fix arg0 4 u32 range 0 0x3E0

fn 021ccae0 ret=none
  arg buf 0xC00
  arg u16 any
  fix arg0 0 u32 near 3000 12
  stub 02034714 noreturn ret=none args=v,v,v

fn 021ccb18 ret=u16
  arg buf 0x400
  fix arg0 4 u32 range 0 0x3E0

fn 021ccb78 ret=u32
  arg buf 0x400
  fix arg0 4 u32 range 0 0x3E0

# put command: copies (low nibble of table[cmd]) argument words to the global scratch at 021f3b44, then
# calls 021ccc0c. Varargs prologue (push {r0-r3}): args after cmd are r2, r3, then the caller's stack
# (arg4..arg16 are stack words), 15 words = the largest nibble in the table. Count near the cap.
fn 021cd4b0 ret=none inputs=400
  arg buf 0xC00
  arg u32 range 0 0x7F
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  fix arg0 0 u32 alt near 2992 20 || range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# 24-bit append: fatal at count >= 0xBB6
fn 021ccb34 ret=none
  arg buf 0xC00
  arg u32 any
  fix arg0 0 u32 near 2998 12
  stub 02034714 noreturn ret=none args=v,v,v

# 32-bit append: fatal at count >= 0xBB5
fn 021ccb98 ret=none
  arg buf 0xC00
  arg u32 any
  fix arg0 0 u32 near 2997 12
  stub 02034714 noreturn ret=none args=v,v,v

fn 021ccbe8 ret=u32
  arg buf 0x400
  fix arg0 4 u32 range 0 0x3E0

# bit 29 of the packed word
fn 021cc9a4 ret=u32
  arg buf 4

# record allocator: [0] bits 10-18 = used size, records from +4; ids from the table at 021efe9c
# (0x3C-0x42 unknown -> 0); the result pointer is compared as an offset from arg0
fn 021cc9ac ret=ofs:arg0 inputs=400
  arg buf 0x400
  arg u32 range 0 0x42
  arg u32 any

# frees the last record: needs (rec - (p+4)) + size == used (size = bits 13-22 of the record header).
# rec is one of three offsets (arg address 0x02E10000 + 4/8/12); used and the three sizes are small so
# that the equality (the shrink path) holds for a share of the inputs; other bits stay random
fn 021cca74 ret=none inputs=400
  arg buf 0x400
  arg u32 oneof 0x02E10004 0x02E10008 0x02E1000C
  fix arg0 0 u32 range 0 6 bits=10:9
  fix arg0 4 u32 range 0 4 bits=13:10
  fix arg0 8 u32 range 0 4 bits=13:10
  fix arg0 12 u32 range 0 4 bits=13:10

# write command: arg3 = pointer to the argument words (6-8 used); fmt = the command's table byte
# (valid formats twice as likely as invalid ones); count near the caps (u8 0xBB8, u16 0xBB7, u24 0xBB6, u32 0xBB5)
fn 021ccc0c ret=none inputs=600
  arg buf 0xC00
  arg u16 any
  arg s32 oneof 0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x8 0x11 0x12 0x13 0x14 0x15 0x16 0x22 0x23 0x24 0x25 0x26 0x32 0x33 0x34 0x42 0x43 0x44 0x53 0x54 0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x8 0x11 0x12 0x13 0x14 0x15 0x16 0x22 0x23 0x24 0x25 0x26 0x32 0x33 0x34 0x42 0x43 0x44 0x53 0x54 0x0 0x9 0x10 0x17 0x21 0x27 0x31 0x35 0x41 0x45 0x52 0x55 0x7F
  arg buf 0x40
  fix arg0 0 u32 alt near 2992 20 || range 0 2900
  stub 02034714 noreturn ret=none args=v,v,v

# read command arguments of format fmt (the write formats reversed) from the cursor of arg0
fn 021cd0f0 ret=none inputs=400
  arg buf 0x400
  arg s32 oneof 0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x8 0x11 0x12 0x13 0x14 0x15 0x16 0x22 0x23 0x24 0x25 0x26 0x32 0x33 0x34 0x42 0x43 0x44 0x53 0x54 0x0 0x9 0x10 0x17 0x21 0x27 0x31 0x35 0x41 0x45 0x52 0x55 0x7F
  arg buf 0x40
  fix arg0 4 u32 range 0 0x3E0

# placeholder command: zero arguments, re-wrapped as a 0x5F skip record. returns the start offset
fn 021cd4fc ret=u32 inputs=400
  arg buf 0xC00
  arg u32 range 0 0x7F
  fix arg0 0 u32 alt near 2990 24 || range 0 2900
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# rewrite the command at write position pos: varargs (push {r0-r3}) = cmd in r2, then r3 and the stack
# words (arg3 = r3, arg4.. stack); cmd 0x5F is a no-op write. count/cursor stay random (their 0xFFFF masks)
fn 021cd568 ret=none inputs=600
  arg buf 0xC00
  arg u32 alt near 2990 24 || range 0 2900
  arg u32 oneof 0x0 0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x8 0x9 0xA 0xB 0xC 0xD 0xE 0xF 0x10 0x11 0x12 0x13 0x14 0x15 0x16 0x17 0x18 0x19 0x1A 0x1B 0x1C 0x1D 0x1E 0x1F 0x20 0x21 0x22 0x23 0x24 0x25 0x26 0x27 0x28 0x29 0x2A 0x2B 0x2C 0x2D 0x2E 0x2F 0x30 0x31 0x32 0x33 0x34 0x35 0x36 0x37 0x38 0x39 0x3A 0x3B 0x3C 0x3D 0x3E 0x3F 0x40 0x41 0x42 0x43 0x44 0x45 0x46 0x47 0x48 0x49 0x4A 0x4B 0x4C 0x4D 0x4E 0x4F 0x50 0x51 0x52 0x53 0x54 0x55 0x56 0x57 0x58 0x59 0x5A 0x5B 0x5C 0x5D 0x5E 0x5F 0x5F 0x5F 0x5F 0x5F 0x5F 0x5F 0x5F 0x5F 0x5F 0x5F 0x5F 0x5F
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  arg u32 any
  global 021f3b44 0x40
  stub 02034714 noreturn ret=none args=v,v,v

# read next command: a chain of 3-byte slots [00 id len] (len 0 or 3 keeps slot alignment, id 0x5F = skip
# record about half the time, else any id < 0x80) so the table lookup stays inside the 0x80 known bytes;
# arg1 = 16-word argument buffer
fn 021cd5e0 ret=u32 inputs=400
  arg buf 0x400
  arg buf 0x40
  fix arg0 0 u32 alt range 0 24 || range 0 0x200
  fix arg0 4 u32 oneof 0 3 6 9
  fix arg0 8 u8 const 0 rep=80,3
  fix arg0 9 u8 alt range 0 0x7F || const 0x5F rep=80,3
  fix arg0 10 u8 oneof 0 0 0 3 rep=80,3

# variable-length put: (q, cmd, w, r3, stack words...); the loop ends after a 0xFFFF0000 word (r3 or a
# stack word) or at the buffer cap (zero words past the supplied ones)
fn 021cd64c ret=none inputs=400
  arg buf 0xC00
  arg u32 alt const 0x5C || range 0 0xFF || oneof 0x15C 0x7F5C 0xDEADBE5C || any
  arg u32 any
  arg u32 alt const 0xFFFF0000 || any || any
  arg u32 alt const 0xFFFF0000 || any || any || any
  arg u32 alt const 0xFFFF0000 || any || any || any
  arg u32 alt const 0xFFFF0000 || any || any || any
  arg u32 alt const 0xFFFF0000 || any || any || any
  arg u32 alt const 0xFFFF0000 || any || any || any
  arg u32 alt const 0xFFFF0000 || any || any || any
  arg u32 alt const 0xFFFF0000 || any || any || any
  arg u32 alt const 0xFFFF0000 || any || any || any
  fix arg0 0 u32 alt near 2990 24 || range 0 2900
  stub 02034714 noreturn ret=none args=v,v,v

# variable-length read: u16, a second u16 for cmd 0x5C, then big-endian words up to a 0xFFFF0000 word (at
# most 16 words in all); terminators (bytes FF FF 00 00) are planted at the word positions of both
# alignments (cmd 0x5C shifts the words by 2), cursor = 2 mod 4
fn 021cd6ac ret=none inputs=400
  arg buf 0x400
  arg u32 alt const 0x5C || range 0 0xFF
  arg buf 0x40
  fix arg0 4 u32 oneof 2 6 10
  fix arg0 12 u32 alt const 0x0000FFFF || any || any || any rep=8,12
  fix arg0 18 u32 alt const 0x0000FFFF || any || any || any rep=8,12

# empty function (bx lr)
fn 021cd6ec ret=none inputs=1
