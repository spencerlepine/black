# fndiff spec: battle_record (B15), overlay 93 (battle set). Grammar: svflow_specs/README.md.
# Run: tools/check-fn.sh black-core battle_record --per-function
#
# Layouts (from the overlay-93 disassembly, not from src/battle_record.c):
#   BattleRecord        0xC08: +0 u16 write_ptr, +2 u8 overflowed, +8 buf[0xC00]
#   BattleRecordReader  0x118: +0 recordData ptr, +4 bits 0-30 dataSize / bit 31 readEnded,
#                              +8 readPtr[4], +0x18 readBuf[4][0x40]
#   BattleRecordBuilder 0x40:  +3 flags (bit 6 tagFlag), +4 buffer[0x3C]
# The reader's record data is `region data`; its address is planted at reader+0. The reader is a
# larger buffer than 0x118: 021ee6b8 copies up to 31 entries (0x7C bytes) into readBuf[c] without a
# bound, so for client 3 the original writes up to reader+0x154 (into whatever follows the reader).
# MI_CpuCopy8 (02082d44) and MI_CpuFill8 (02082bcc) are real code on both sides (svflow ALLOW).
area battle_record
set battle

# BattleRecord_New(heapID): Heap_AllocDebug(heapID, 0xC08, 1, "btl_rec.c" @ 021f3adc, 107)
fn 021ee5e4 ret=u32
  nowork
  arg u32 any
  stub 02030734 ret=u32 args=v,v,v,v,v

# BattleRecord_Free(rec): `ldr r3,[pc]; bx r3` thunk to Heap_Free
fn 021ee600 ret=none
  nowork
  arg u32 any
  stub 020307b0 ret=none args=v

# BattleRecord_GetData(rec, *size): *size = (u32)rec->write_ptr; returns rec + 8 (the buffer)
fn 021ee63c ret=ofs:arg0
  nowork
  arg buf 0xC08
  arg buf 4

# BattleRecordReader_Init(reader, recordData, dataSize): stores the pointer, size & 0x7FFFFFFF with
# readEnded cleared, readPtr[0..4] = 0
fn 021ee644 ret=none
  nowork
  arg buf 0x20
  arg u32 any
  arg u32 any

# BattleRecordReader_Reset(reader): readPtr[0..4] = 0 (readEnded kept)
fn 021ee674 ret=none
  nowork
  arg buf 0x20

# FUN_overlay_d_93__021ee6b8(reader, clientID, *numEntries, *tagFlag) -> pointer to readBuf[clientID]
fn 021ee6b8 ret=ofs:arg0 inputs=2000
  nowork
  region data 0x200
  arg buf 0x160
  arg u32 range 0 3
  arg buf 4
  arg buf 4
  fix arg0 0 u32 addrof region:data
  fix arg0 4 u32 alt range 0 0x10 || range 0 0x100
  fix arg0 4 u32 oneof 0 0 0 1 bits=31:1
  fix arg0 8 u32 alt const 0 || range 0 0x108
  fix arg0 12 u32 alt const 0 || range 0 0x108
  fix arg0 16 u32 alt const 0 || range 0 0x108
  fix arg0 20 u32 alt const 0 || range 0 0x108
  # a plausible first tag at 0: type 1 (entries) mostly, 1..4 client blocks of few entries
  fix region:data 0 u8 oneof 1 1 1 1 2 3 0 5 bits=4:3
  fix region:data 0 u8 range 0 5 bits=0:4
  fix region:data 1 u8 range 0 4 bits=5:3
  fix region:data 1 u8 range 0 4 bits=0:5
  stub 021d8470 ret=none args=out:4
  stub 021d845c ret=none args=out:4

# FUN_overlay_d_93__021ee830(reader) -> number of tags with the flag bit (bit 7) set
fn 021ee830 ret=u32 inputs=1000
  nowork
  region data 0x100
  arg buf 0x20
  fix arg0 0 u32 addrof region:data
  fix arg0 4 u32 alt range 0 0x10 || range 0 0x100 bits=0:31
  fix region:data 0 u8 oneof 1 1 1 2 3 0 bits=4:3
  fix region:data 0 u8 range 0 5 bits=0:4

# BattleRecordReader_IsAtEnd(reader, clientID) -> readEnded || readPtr[clientID] == dataSize
fn 021ee894 ret=bool
  nowork
  arg buf 0x20
  arg u32 range 0 3
  fix arg0 4 u32 range 0 3 bits=0:31
  fix arg0 8 u32 range 0 3
  fix arg0 12 u32 range 0 3
  fix arg0 16 u32 range 0 3
  fix arg0 20 u32 range 0 3

# BattleRecordBuilder_Init(builder, tagFlag): MI_CpuFill8(builder, 0, 0x40); flags bit 6 = tagFlag & 1
fn 021ee8b4 ret=none
  nowork
  arg buf 0x40
  arg u32 any
