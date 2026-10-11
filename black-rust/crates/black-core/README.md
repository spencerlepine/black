# black-core

Safe Rust port of `src/battle_record.c` and `src/unk_02008574.c` (Pokemon Black decomp).
Undecompiled calls are traits: `heap::Heap` (`Heap_AllocDebug`/`Heap_Free`, host impl `HostHeap`) and
`battle_common::EntryHooks` (`FUN_overlay_d_93__021d845c` / `021d8470`). SDK `MI_CpuCopy8/Fill8` are slice ops.

| C | Rust |
|---|------|
| `BattleRecord_New` | `battle_record::BattleRecord::new` |
| `BattleRecord_Free` | `BattleRecord::free` |
| `BattleRecord_Append` | `BattleRecord::append` |
| `BattleRecord_GetData` | `BattleRecord::data` |
| `FUN_overlay_d_93__021ee638` | `battle_record::header_code_of` |
| `BattleRecordReader_Init` / `_Reset` / `_IsAtEnd` | `BattleRecordReader::new` / `reset` / `is_at_end` |
| `FUN_overlay_d_93__021ee688` | `BattleRecordReader::consume_flag_tag` |
| `FUN_overlay_d_93__021ee6b8` | `BattleRecordReader::read_next` |
| `FUN_overlay_d_93__021ee830` | `BattleRecordReader::count_flagged_tags` |
| `BattleRecordBuilder_Init` | `BattleRecordBuilder::new` |
| `FUN_overlay_d_93__021ee8d8` | `BattleRecordBuilder::header_only_flag_record` |
| `BattleRecordBuilder_AddClientEntries` | `BattleRecordBuilder::add_client_entries` |
| `BattleRecordBuilder_Finish` | `BattleRecordBuilder::finish` |
| `FUN_overlay_d_93__021ee9a4` | `BattleRecordBuilder::header_only_unk3_record` |
| `BattleRecordBuilder_Load` (asm-only) | `BattleRecordBuilder::load` |
| `BattleRecordBuilder_ReadNext` | `BattleRecordBuilder::read_next` |
| `Pack/UnpackHeaderTag`, `Pack/UnpackClientTag` | `pack_header_tag`, `unpack_header_tag`, `pack_client_tag`, `unpack_client_tag` |
| `FUN_02008574` | `unk_02008574::set_bytes_18_19` |
| `MI_CpuCopy8` / `MI_CpuFill8` | `mi_memory::cpu_copy8` / `cpu_fill8` |

Serialization: `BattleRecord::{to,from}_bytes` (0xC08 bytes), `BattleRecordBuilder::{to,from}_bytes`
(0x40 bytes; bitfield byte at +3: type bits 0-5, tagFlag bit 6, error bit 7), `BattleEntry::{to,from}_bytes` (LE).

Deliberate deviations (C does unchecked pointer math): out-of-range reads give 0, copies into the
0x40-byte reader buffer are clamped, `Load` rejects data over 0x3C bytes, empty `append` is a no-op.

Tests include differential fuzz hashes produced by compiling the original C on the host with shims.
