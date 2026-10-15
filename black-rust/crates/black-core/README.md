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
| `FUN_02019A7C` / `FUN_02019A9C` | `poke_crypt::lcg32_step` / `checksum` (verified per function, `tools/check-fn.sh black-core poke_crypt`) |
| `FUN_0209BDBC` / `FUN_0209BE04` | `softfloat::i2f` / `u2f` (int/uint to single, nearest even) |
| `FUN_0209C494` | `softfloat::f2i` (single to int, truncating, saturating) |
| `FUN_0209C95C` / `FUN_0209CB8C` | `softfloat::fadd` / `fsub` (single add / subtract) |
| `FUN_0209B750` / `FUN_0209B790` | `softfloat::i2d` / `u2d` (int/uint to double, returns r1:r0 as `u64`) |

`softfloat` (Z17): pure-integer ports of the ARM9 soft-float helpers, verified per function
(`tools/check-fn.sh black-core softfloat`). They are **single precision** (bit patterns in r0/r1),
not double, except `i2d`/`u2d`. Quirks kept: NaN results are always `0x7FFFFFFF`, the sticky bit
is lost for exponent gaps over 32, one unreachable `fsub` path returns `0x7F8000FF`.

Serialization: `BattleRecord::{to,from}_bytes` (0xC08 bytes), `BattleRecordBuilder::{to,from}_bytes`
(0x40 bytes; bitfield byte at +3: type bits 0-5, tagFlag bit 6, error bit 7), `BattleEntry::{to,from}_bytes` (LE).

Deliberate deviations (C does unchecked pointer math): out-of-range reads give 0, copies into the
0x40-byte reader buffer are clamped, `Load` rejects data over 0x3C bytes, empty `append` is a no-op.

Tests include differential fuzz hashes produced by compiling the original C on the host with shims.

`battle_record` overlay-93 image ports (B15; verified per function with
`tools/check-fn.sh black-core battle_record`, area hash eeb8790696057445). One to one with the
machine code on the raw little-endian images (reader 0x118: data address +0, size bits 0-30 /
read-ended bit 31 at +4, `read_ptr[4]` +8, `read_buf[4][0x40]` +0x18), no clamps; the heap is the
address-level `battle_record::RecordHeap`, the entry initialisers are `EntryHooks`:

| C | Rust |
|---|------|
| `BattleRecord_New` (021ee5e4) / `BattleRecord_Free` (021ee600) | `record_new` / `record_free` |
| `BattleRecord_GetData` (021ee63c) | `record_get_data` (returns the buffer offset 8) |
| `BattleRecordReader_Init` / `_Reset` / `_IsAtEnd` (021ee644 / 674 / 894) | `reader_init` / `reader_reset` / `reader_is_at_end` |
| `FUN_overlay_d_93__021ee6b8` | `reader_read_next` (unbounded copy of up to 31 entries into `read_buf[c]`, as in the ROM) |
| `FUN_overlay_d_93__021ee830` | `reader_count_flagged_tags` |
| `BattleRecordBuilder_Init` (021ee8b4) | `builder_init` |

Shared types (Z6): `ids` (species/move/item/ability/message ID aliases; type, gender, stat and
nature constants) and `rng::MathRandContext32` (SDK 64-bit LCG, moved from `black-battle::calc`,
which re-exports it). Only the type IDs and the LCG are verified; the other orders are standard
Gen 5 values pending P1/P6.

`rng` (Z15, `gf_standard.c`; verified per function with `tools/check-fn.sh black-core rng`):

| C | Rust |
|---|------|
| `FUN_0203F034` / `FUN_0203F0A8` | `Mt19937::seed` / `next_u32` (MT19937 over the heap struct behind `*0x021469F8`: `mt[624]`, `index` @0x9C0, `mag[2]` @0x9C4) |
| `FUN_0203EF5C` (struct setup, allocation not ported) | `Mt19937::new` (not diffed) |
| `FUN_020056EC` / `FUN_02005728` | `MathRandContext32::next` / `next_scaled` (global LCG64 context at `*(0x020AA194 + 0x10)`) |
| `FUN_0203F258` (thunk `0x020056C8`) | `seed_lcg64` + `SeedHost` (SHA-1 of entropy; mul `0x5D588B656C078965`, add `0x269EC3`) |
| `FUN_0203F2FC` | `byte_sum` (sum of bytes) |
| `FUN_0203F388` / `FUN_0203F328` | `lcg32_next` / `lcg32_xor` (LCG32 `0x41C64E6D`/`0x6073`; same cipher as `poke_crypt` `FUN_02019A50`) |

Deviations: `Mt19937::next_u32` compares `index` as a signed int like the original (a negative index
is out of bounds there, a panic here). `seed_lcg64` passes 20 zero bytes for the first SHA-1 update
(the original hashes uninitialised stack), and the diff stubs hash updates by length only.

`fx` (Z24, Z25): NitroSDK fixed-point math of the ARM9 main image (0x0207AF50..0x0207D184), pure Rust, all
`fx32`/`fx16` vectors and matrices as arrays. All 24 functions are verified per function by `tools/check-fn.sh black-core
fx --per-function`: `MTX_RotAxis33` 0207AF50, `MTX_RotAxis43` 0207B65C, `MTX_MultVec43` 0207BC08, `FX_Div` 0207C768,
`FX_Inv` 0207C778, `FX_Sqrt` 0207C7B4, `FX_InvSqrt` 0207C7E8, 32-bit quotient / remainder 0207C958 / 0207C994, `VEC_Add`
0207C9D0, `VEC_Subtract` 0207CA00, `VEC_Fx16Add` 0207CA30, `VEC_DotProduct` 0207CA68, `VEC_Fx16DotProduct` 0207CAA4,
`VEC_CrossProduct` 0207CAE8, `VEC_Fx16CrossProduct` 0207CB68, `VEC_Mag` 0207CBD0, `VEC_Fx16Mag` 0207CC30, `VEC_Normalize`
0207CCA4, `VEC_Fx16Normalize` 0207CDB8, `VEC_MultAdd` 0207CEE4, `VEC_Distance` 0207CF3C, `FX_Atan2Idx` 0207CFB8,
`FX_AcosIdx` 0207D184 (area hash f705a50d28bf8858). Since Z25 the emulator models the divider / sqrt unit
(`tools/rom-diff/emu.py`, shared with battle_diff / field_diff), so the 12 hardware users run directly; `tools/rom-diff/fx_hw_diff.py`
is retired. The port mirrors the unit as `fx::div32` / `div64_32` / `div64_64` / `sqrt32` / `sqrt64` (GBATEK rules; the
registers are not part of the API). `FX_SinCosTable_` (0x020A1B38) and the arctangent table (0x020A5B38, 129 entries)
are computed (`fx::sin` / `cos` / `atan_entry`) and checked against the ROM by `tables_match_rom`. `fx_atan2_idx`
panics where the original would read outside its table (`i32::MIN` operands).
