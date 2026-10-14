# fndiff spec: wordset.c (F9, `unk_0201EC64`): the WordSet message-variable buffers and the string
# expansion that consumes them.  Main ARM9 Thumb code except where noted.  Grammar: svflow_specs/README.md
# (generic mode).  Run: tools/check-fn.sh black-text wordset --per-function
#
# WORDSET layout (FUN_0201EC74 = create(maxWords, bufLen, heap), FUN_0201EC64 = create(8, 0x20, heap), 0x10 bytes,
# "wordset.c"): +0 u32 count (max words), +4 u32 heap id, +8 -> entries, +0xC -> StrBuf* scratch (capacity bufLen).
# ENTRY (12 bytes, array at +8): +0..4 five info bytes (FUN_0201ED50 zeroes them; +3 bit7 = "no plural/gender
# marker", low 7 bits a count; +4 = flag byte read by the expander), +8 -> StrBuf* of that word.  Strings of
# a word come from MsgData (FUN_02045B38 create, FUN_02045CAC get, FUN_02045C04 delete), a stub here.
# The fixes model a 4-entry set in one 0x80-byte buffer (ws header at 0, entries at 0x20, 12 bytes each)
# whose entry strings (s0..s3) and scratch (t) are separate STRBUF regions (capacity/length <= 20).
# Callees in strbuf.c (0204581C, 02045AFC, 02045AA8, 02045838, ..) and the msg-tag helpers
# 0201D5C4/D5CC/D60C/D670/D688 (a different file) run as real code.
area wordset
set field
# FUN_0201ED64(ws, i): clear entry i's five info bytes and its string (no bounds check)
fn 0201ed64 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 3
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20

# FUN_0201ED50(entry): five info bytes = 0 (MI_CpuFill8 runs as real code)
fn 0201ed50 ret=none
  nowork
  arg buf 0x10

# FUN_0201ED84(ws, i, str, info): if i < count: entry i's info = *info (5 bytes) or zeros when info is null; then copy str into its string (FUN_02045838)
fn 0201ed84 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 5
  arg buf 0x60
  arg u32 alt const 0 || const 0x02E50000
  arg buf 0x10
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  fix arg2 0 u16 range 0 24
  fix arg2 2 u16 range 0 20
  fix arg2 8 u16 alt range 0 3 || const 0xffff rep=21,2

# FUN_0201EDD0(ws, i, bank, msgid): md = MsgData_Create(0, 2, bank, ws.heap); if md: GetString(md, msgid, scratch); set entry i from scratch (info zeroed); MsgData_Delete(md)
fn 0201edd0 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 5
  arg u32 alt range 0 0xffff || any
  arg u32 alt range 0 100 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  stub 02045b38 ret=nullable:4:u32 args=v,v,v,v
  stub 02045cac ret=none args=v,v,ptr
  stub 02045c04 ret=none args=v

# FUN_0201EE0C(ws, i, str, n, plain): entry i = {info zeroed, +3 = (n & 0x7f) | ((plain == 0) << 7)}, string copied from str
fn 0201ee0c ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 5
  arg buf 0x60
  arg u32 alt range 0 0x7f || any
  arg u32 alt const 0 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  fix arg2 0 u16 range 0 24
  fix arg2 2 u16 range 0 20
  fix arg2 8 u16 alt range 0 3 || const 0xffff rep=21,2

# FUN_0201EE60(ws, i, msgid): GetString(*0x020A7370, msgid, scratch); set entry i from scratch (info zeroed)
fn 0201ee60 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 5
  arg u32 alt range 0 100 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  global 020a7370 4
  stub 02045cac ret=none args=v,v,ptr

# FUN_0201EFA4(ws, i, value, digits, mode, ascii): FUN_0201F35C(scratch, value, digits, mode, ascii); set entry i from scratch
fn 0201efa4 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 5
  arg s32 alt range 0 100000 || oneof 0xffffffff 0xfffffff9 0xffff7960 0x80000000 0x7fffffff || any
  arg u32 range 0 10
  arg u32 range 0 3
  arg u32 alt const 0 || const 1 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20

# FUN_0201F02C(ws, i, v): FUN_02008500(v, scratch) fills the scratch string (stub); set entry i from it
fn 0201f02c ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 5
  arg u32 any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  stub 02008500 ret=none args=v,p

# FUN_0201F048(ws, i, id): md = MsgData_Create(0, 2, 0xE7, heap); if md: if id < Count(md): GetString + set entry i, else clear entry i; delete md
fn 0201f048 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 3
  arg u32 alt range 0 12 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  stub 02045b38 ret=nullable:4:u32 args=v,v,v,v
  stub 02045cac ret=none args=v,v,ptr
  stub 02045c04 ret=none args=v
  stub 02045f50 ret=mod:8 args=v

# FUN_0201F184(ws, i, v): EDD0(ws, i, 0xA0, v >= 0x30 ? 0 : v)
fn 0201f184 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 5
  arg s32 alt range 0 60 || oneof 0xffffffff 0xfffffffd 0x80000000 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  stub 02045b38 ret=nullable:4:u32 args=v,v,v,v
  stub 02045cac ret=none args=v,v,ptr
  stub 02045c04 ret=none args=v

# FUN_0201F230(ws, i, v): EDD0(ws, i, 278, clamp(v, 1, 12) - 1)
fn 0201f230 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 5
  arg s32 alt range 0 60 || oneof 0xffffffff 0xfffffffd 0x80000000 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  stub 02045b38 ret=nullable:4:u32 args=v,v,v,v
  stub 02045cac ret=none args=v,v,ptr
  stub 02045c04 ret=none args=v

# FUN_0201F198(ws, i, v): v > 8 ? clear entry i : EDD0(ws, i, 0x2B, v)
fn 0201f198 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 3
  arg u32 alt range 0 10 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  stub 02045b38 ret=nullable:4:u32 args=v,v,v,v
  stub 02045cac ret=none args=v,v,ptr
  stub 02045c04 ret=none args=v

# FUN_0201F1B0(ws, i, v): v > 0x91 ? clear entry i : EDD0(ws, i, 0, v)
fn 0201f1b0 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 3
  arg u32 alt range 0 147 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  stub 02045b38 ret=nullable:4:u32 args=v,v,v,v
  stub 02045cac ret=none args=v,v,ptr
  stub 02045c04 ret=none args=v

# FUN_0201F1C8(ws, i, v): v > 8 ? clear entry i : EDD0(ws, i, 0x38, v)
fn 0201f1c8 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 3
  arg u32 alt range 0 10 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  stub 02045b38 ret=nullable:4:u32 args=v,v,v,v
  stub 02045cac ret=none args=v,v,ptr
  stub 02045c04 ret=none args=v

# unlabeled 0x0201F1E0(ws, i, v): md = MsgData_Create(0, 3, 461, heap); clear entry i; if v <= 6 and md: GetString(md, v + 0x2B) + set entry i; delete md
fn 0201f1e0 ret=none
  nowork
  arg buf 0x80
  arg u32 range 0 3
  arg u32 alt range 0 9 || any
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  stub 02045b38 ret=nullable:4:u32 args=v,v,v,v
  stub 02045cac ret=none args=v,v,ptr
  stub 02045c04 ret=none args=v

# FUN_0201F31C(ws): clear the string of every entry (info bytes untouched)
fn 0201f31c ret=none
  nowork
  arg buf 0x80
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20

# FUN_0201F35C(sb, value, digits, mode, ascii): clear sb; negative: '-' sign (0x30FC full-width) and negate; then for
# d = POW10[digits] down by /10: q = (value / d) & 0xFFFF, value -= d * q, digit = table[min(q, 10)]; mode 2 prints
# every digit, mode 1 pads leading zeros with a space (0x20 / 0x3000), mode 0 skips them; the last digit is always
# printed.  Tables (ROM data, read by the original): POW10 @0x0209E838, ascii digits @0x0209E758, full-width @0x0209E784.
fn 0201f35c ret=none
  nowork
  arg buf 0x60
  arg s32 alt range 0 100000 || oneof 0xffffffff 0xfffffff9 0xffff7960 0x80000000 0x7fffffff 0xc4653600 || range 0 1000000000 || any
  arg u32 range 0 10
  arg u32 range 0 3
  arg u32 alt const 0 || const 1 || any
  fix arg0 0 u16 range 0 20
  fix arg0 2 u16 range 0 20

# FUN_0201F41C(src, dst, n): copy u16 units up to n or until EOS (inclusive), then dst[n-1] = EOS
fn 0201f41c ret=none
  nowork
  arg buf 0x60
  arg buf 0x60
  arg u32 range 1 20
  fix arg0 8 u16 alt range 0 3 || const 0xffff rep=40,2

# FUN_0201F448(a, b): 1 if the two u16 strings are equal up to EOS
fn 0201f448 ret=bool
  nowork
  arg buf 0x60
  arg buf 0x60
  fix arg0 0 u16 alt range 0 2 || const 0xffff rep=40,2
  fix arg1 0 u16 alt range 0 2 || const 0xffff rep=40,2

# FUN_0201F47C(sb): length != 0 && chars[0] == 0xF100 (the 9-bit compressed string marker)
fn 0201f47c ret=bool
  nowork
  arg buf 0x60
  fix arg0 2 u16 alt const 0 || range 0 20
  fix arg0 8 u16 alt const 0xf100 || any

# FUN_0201F4A0(dst, src): if src is 9-bit compressed (F100 header): decode 9-bit codes (0x1FF ends; <0x100 literal;
# 0x100..0x131 -> table @0x0209E864; else '?') appending each to dst; otherwise append src to dst (FUN_02045AA8)
fn 0201f4a0 ret=none
  nowork
  arg buf 0x60
  arg buf 0x60
  fix arg0 0 u16 range 0 30
  fix arg0 2 u16 range 0 24
  fix arg1 0 u16 range 0 20
  fix arg1 2 u16 range 0 12
  fix arg1 8 u16 alt const 0xf100 || const 0xf100 || const 0xf100 || range 0 5 || any
  fix arg1 10 u16 any rep=6,2
  fix arg1 0x16 u16 const 0xffff rep=4,2
  fix arg1 0x1e u16 alt const 0xffff || any rep=10,2

# FUN_0201F250(ws, dst, src): StringExpandPlaceholders.  dst is cleared, then src is scanned to EOS: a tag
# {0xF000, group<<8|cmd, n, args[n]} (group 1 or 2 = word-set variable, arg0 = entry index; no bounds check) appends
# entry arg0's string to dst (FUN_0201F4A0); if the entry's flag byte (+4) is set, two layout tags
# {F000, BD06, 1, idx} and {F000, BD04, 1, 0x54} are first appended to the ENTRY's own string; any other tag is
# copied through verbatim; other characters are copied one by one (FUN_02045AFC).
fn 0201f250 ret=none
  nowork
  arg buf 0x80
  arg buf 0x60
  arg buf 0x60
  region t 0x60
  region s0 0x60
  region s1 0x60
  region s2 0x60
  region s3 0x60
  fix arg0 0 u32 range 0 4
  fix arg0 8 u32 addrof arg0 0x20
  fix arg0 0xc u32 addrof region:t
  fix arg0 0x28 u32 addrof region:s0
  fix arg0 0x34 u32 addrof region:s1
  fix arg0 0x40 u32 addrof region:s2
  fix arg0 0x4c u32 addrof region:s3
  fix region:t 0 u16 range 0 20
  fix region:t 2 u16 range 0 20
  fix region:s0 0 u16 range 0 20
  fix region:s0 2 u16 range 0 20
  fix region:s1 0 u16 range 0 20
  fix region:s1 2 u16 range 0 20
  fix region:s2 0 u16 range 0 20
  fix region:s2 2 u16 range 0 20
  fix region:s3 0 u16 range 0 20
  fix region:s3 2 u16 range 0 20
  fix arg1 0 u16 range 0 30
  fix arg1 2 u16 range 0 20
  fix arg2 0 u16 range 12 28
  fix arg2 2 u16 range 0 20
  fix arg2 8 u16 alt range 1 3 || range 1 3 || range 1 3 || range 1 3 || const 0xffff rep=40,2
  fix arg2 8 u16 alt const 0xf000 || const 0xf000 || range 1 3
  fix arg2 10 u16 alt const 0x0100 || const 0x0200 || const 0x0100 || const 0x0300 || const 0x0000 || const 0xbd04 || range 0 0xffff
  fix arg2 12 u16 range 0 3
  fix arg2 14 u16 range 0 3
  fix arg2 22 u16 alt const 0xf000 || const 0xf000 || range 1 3
  fix arg2 24 u16 alt const 0x0100 || const 0x0200 || const 0x0100 || const 0x0300 || const 0x0000 || const 0xbd04 || range 0 0xffff
  fix arg2 26 u16 range 0 3
  fix arg2 28 u16 range 0 3
  fix arg2 36 u16 alt const 0xf000 || const 0xf000 || range 1 3
  fix arg2 38 u16 alt const 0x0100 || const 0x0200 || const 0x0100 || const 0x0300 || const 0x0000 || const 0xbd04 || range 0 0xffff
  fix arg2 40 u16 range 0 3
  fix arg2 42 u16 range 0 3
  fix arg2 0x58 u16 const 0xffff
  fix arg0 0x24 u8 alt const 0 || any
  fix arg0 0x30 u8 alt const 0 || any
  fix arg0 0x3c u8 alt const 0 || any
  fix arg0 0x48 u8 alt const 0 || any
  fix region:s0 8 u16 alt const 0xf100 || const 0xf100 || range 1 5 || any
  fix region:s0 0xa u16 any rep=4,2
  fix region:s0 0x16 u16 const 0xffff rep=4,2
  fix region:s1 8 u16 alt const 0xf100 || const 0xf100 || range 1 5 || any
  fix region:s1 0xa u16 any rep=4,2
  fix region:s1 0x16 u16 const 0xffff rep=4,2
  fix region:s2 8 u16 alt const 0xf100 || const 0xf100 || range 1 5 || any
  fix region:s2 0xa u16 any rep=4,2
  fix region:s2 0x16 u16 const 0xffff rep=4,2
  fix region:s3 8 u16 alt const 0xf100 || const 0xf100 || range 1 5 || any
  fix region:s3 0xa u16 any rep=4,2
  fix region:s3 0x16 u16 const 0xffff rep=4,2

