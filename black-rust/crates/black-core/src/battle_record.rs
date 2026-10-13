//! C: src/battle_record.c, include/battle_record.h
//!
//! Deviations from the C (which does unchecked pointer arithmetic): reads past the end of the
//! record yield 0 instead of garbage; copies that would overflow a fixed buffer are clamped;
//! out-of-range client IDs panic.
//!
//! The overlay-93 functions `021ee5e4`..`021ee8b4` are also ported one to one on memory images
//! (`record_new` .. `builder_init`, at the end of this file) and verified against the ROM with
//! `tools/check-fn.sh black-core battle_record`; those have none of the deviations above.

use crate::battle_common::{BattleEntry, EntryHooks, BTL_CLIENT_MAX};
use crate::heap::{Heap, HeapId};
use crate::mi_memory::{cpu_copy8, cpu_fill8};

pub const BTLREC_BUFFER_SIZE: usize = 0xC00;
pub const BTLREC_READBUF_SIZE: usize = 0x40;
pub const BTLREC_TOOL_BUFFER_SIZE: usize = 0x3C;

const BTLREC_SRC_FILE: &str = "btl_rec.c";
const BTLREC_SRC_LINE: u32 = 107;

const HEADER_CODE: usize = 0;
const HEADER_TAG: usize = 1;
/// C: `HEADER_SIZE`
pub const HEADER_SIZE: u32 = 2;

/// C: `TagType` (raw 3-bit values; other values are possible on the wire).
pub const TAG_TYPE_NONE: u8 = 0;
pub const TAG_TYPE_ENTRIES: u8 = 1;
pub const TAG_TYPE_UNK_2: u8 = 2;
pub const TAG_TYPE_UNK_3: u8 = 3;

/// C: `HeaderCode`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum HeaderCode {
    Code0 = 0,
    Code4 = 4,
}

impl HeaderCode {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Code0),
            4 => Some(Self::Code4),
            _ => None,
        }
    }
}

/// C: `PackHeaderTag`
pub fn pack_header_tag(tag_type: u8, num_client: u8, tag_flag: bool) -> u8 {
    ((tag_flag as u8) << 7) | ((tag_type & 0x07) << 4) | (num_client & 0x0f)
}

/// C: `UnpackHeaderTag`. Returns `(type, num_client, tag_flag)`.
pub fn unpack_header_tag(code: u8) -> (u8, u8, bool) {
    ((code >> 4) & 0x07, code & 0x0f, (code >> 7) & 1 != 0)
}

/// C: `PackClientTag` (result truncated to u8 like the C).
pub fn pack_client_tag(client_id: u8, num_entries: u8) -> u8 {
    (((client_id as u32) << 5) | num_entries as u32) as u8
}

/// C: `UnpackClientTag`. Returns `(client_id, num_entries)`.
pub fn unpack_client_tag(code: u8) -> (u8, u8) {
    ((code >> 5) & 0x07, code & 0x1f)
}

// ---------------------------------------------------------------------------------------------
// BattleRecord
// ---------------------------------------------------------------------------------------------

/// C: `struct _BattleRecord`
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct BattleRecord {
    pub write_ptr: u16,
    pub overflowed: u8,
    pub padding: u8,
    pub unk_04: u32,
    pub buf: [u8; BTLREC_BUFFER_SIZE],
}

const _: () = assert!(core::mem::size_of::<BattleRecord>() == 0xC08);

impl BattleRecord {
    pub const SIZE: usize = 0xC08;

    /// Serialise to the in-memory little-endian layout.
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut o = [0u8; Self::SIZE];
        o[0..2].copy_from_slice(&self.write_ptr.to_le_bytes());
        o[2] = self.overflowed;
        o[3] = self.padding;
        o[4..8].copy_from_slice(&self.unk_04.to_le_bytes());
        o[8..].copy_from_slice(&self.buf);
        o
    }

    /// Parse from the in-memory layout. Returns `None` if `b` is not exactly 0xC08 bytes.
    pub fn from_bytes(b: &[u8]) -> Option<Self> {
        if b.len() != Self::SIZE {
            return None;
        }
        let mut buf = [0u8; BTLREC_BUFFER_SIZE];
        buf.copy_from_slice(&b[8..]);
        Some(Self {
            write_ptr: u16::from_le_bytes([b[0], b[1]]),
            overflowed: b[2],
            padding: b[3],
            unk_04: u32::from_le_bytes([b[4], b[5], b[6], b[7]]),
            buf,
        })
    }

    /// C: `BattleRecord_New @ battle_record.c`. Contents are whatever the heap returns
    /// (the C does not initialise them). `None` on allocation failure.
    pub fn new(heap: &mut dyn Heap, heap_id: HeapId) -> Option<Self> {
        let block = heap.alloc_debug(
            heap_id,
            Self::SIZE as u32,
            true,
            BTLREC_SRC_FILE,
            BTLREC_SRC_LINE,
        )?;
        // The typed record takes over the block's contents; `free` hands a block back.
        Self::from_bytes(&block)
    }

    /// C: `BattleRecord_Free`
    pub fn free(self, heap: &mut dyn Heap) {
        heap.free(self.to_bytes().to_vec().into_boxed_slice());
    }

    /// C: `BattleRecord_Append`. Appends `data[1..]` (the first byte is skipped, as in the C).
    /// Sets `overflowed` instead if it does not fit. Empty `data` is a no-op (UB in the C).
    pub fn append(&mut self, data: &[u8]) {
        if self.overflowed != 0 || data.is_empty() {
            return;
        }
        let payload = &data[1..];
        let end_ptr = self.write_ptr as usize + payload.len();
        if end_ptr <= BTLREC_BUFFER_SIZE {
            let w = self.write_ptr as usize;
            cpu_copy8(payload, &mut self.buf[w..], payload.len() as u32);
            self.write_ptr = end_ptr as u16;
        } else {
            self.overflowed = 1;
        }
    }

    /// C: `BattleRecord_GetData`. Returns the whole buffer and the used size.
    pub fn data(&self) -> (&[u8], u32) {
        (&self.buf, self.write_ptr as u32)
    }
}

/// C: `FUN_overlay_d_93__021ee638 @ 0x021ee638` - first byte of a record is its header code.
pub fn header_code_of(data: &[u8]) -> Option<u8> {
    data.first().copied()
}

// ---------------------------------------------------------------------------------------------
// BattleRecordReader
// ---------------------------------------------------------------------------------------------

/// C: `BattleRecordReader`. `data_size` (31 bits) and `read_ended` (1 bit) are separate fields here.
#[derive(Debug, Clone)]
pub struct BattleRecordReader<'a> {
    record_data: &'a [u8],
    data_size: u32,
    read_ended: bool,
    read_ptr: [u32; BTL_CLIENT_MAX],
    read_buf: [[u8; BTLREC_READBUF_SIZE]; BTL_CLIENT_MAX],
}

/// Result of [`BattleRecordReader::read_next`].
#[derive(Debug, PartialEq, Eq)]
pub struct ReadEntries<'r> {
    /// Entry bytes (`num_entries * 4`, clamped to the 0x40-byte read buffer).
    pub entries: &'r [u8],
    pub num_entries: u8,
    pub tag_flag: bool,
}

impl ReadEntries<'_> {
    /// The `i`th entry, if present in `entries`.
    pub fn entry(&self, i: usize) -> Option<BattleEntry> {
        let c = self.entries.get(i * 4..i * 4 + 4)?;
        Some(BattleEntry::from_bytes(c.try_into().ok()?))
    }
}

enum Scan {
    Entries { len: usize, num: u8, flag: bool },
    Unk3,
    End,
}

impl<'a> BattleRecordReader<'a> {
    /// C: `BattleRecordReader_Init`. `data_size` is truncated to 31 bits.
    pub fn new(record_data: &'a [u8], data_size: u32) -> Self {
        Self {
            record_data,
            data_size: data_size & 0x7FFF_FFFF,
            read_ended: false,
            read_ptr: [0; BTL_CLIENT_MAX],
            read_buf: [[0; BTLREC_READBUF_SIZE]; BTL_CLIENT_MAX],
        }
    }

    /// C: `BattleRecordReader_Reset` (read pointers only; `read_ended` is kept).
    pub fn reset(&mut self) {
        self.read_ptr = [0; BTL_CLIENT_MAX];
    }

    pub fn read_ptr(&self, client_id: u8) -> u32 {
        self.read_ptr[client_id as usize]
    }

    pub fn read_ended(&self) -> bool {
        self.read_ended
    }

    pub fn data_size(&self) -> u32 {
        self.data_size
    }

    fn byte(&self, i: u32) -> u8 {
        self.record_data.get(i as usize).copied().unwrap_or(0)
    }

    /// C: `FUN_overlay_d_93__021ee688 @ 0x021ee688` - if the next tag for `client_id` is
    /// `TAG_TYPE_UNK_2`, consume it and return its flag; otherwise false.
    pub fn consume_flag_tag(&mut self, client_id: u8) -> bool {
        let c = client_id as usize;
        let (ty, _, flag) = unpack_header_tag(self.byte(self.read_ptr[c]));
        if ty == TAG_TYPE_UNK_2 {
            self.read_ptr[c] += 1;
            return flag;
        }
        false
    }

    fn end_entry(&mut self, c: usize, hooks: &dyn EntryHooks, unk3: bool) -> ReadEntries<'_> {
        let mut e = BattleEntry::default();
        if unk3 {
            hooks.init_unk3(&mut e);
        } else {
            hooks.init_end(&mut e);
        }
        self.read_buf[c][..4].copy_from_slice(&e.to_bytes());
        ReadEntries {
            entries: &self.read_buf[c][..4],
            num_entries: 1,
            tag_flag: false,
        }
    }

    fn scan(&mut self, c: usize, client_id: u8) -> Scan {
        while self.read_ptr[c] < self.data_size {
            let (ty, num_client, flag) = unpack_header_tag(self.byte(self.read_ptr[c]));
            self.read_ptr[c] += 1;
            if self.read_ptr[c] > self.data_size {
                break;
            }
            if ty == TAG_TYPE_ENTRIES {
                if num_client == 0 || num_client as usize > BTL_CLIENT_MAX {
                    break;
                }
                let mut found: Option<(usize, u8)> = None;
                for _ in 0..num_client {
                    let (rid, rn) = unpack_client_tag(self.byte(self.read_ptr[c]));
                    self.read_ptr[c] += 1;
                    if self.read_ptr[c] >= self.data_size {
                        break;
                    }
                    let nbytes = BattleEntry::SIZE * rn as usize;
                    if rid != client_id {
                        self.read_ptr[c] = self.read_ptr[c].wrapping_add(nbytes as u32);
                    } else {
                        let start = self.read_ptr[c] as usize;
                        let len = nbytes.min(BTLREC_READBUF_SIZE);
                        let avail = self.record_data.len().saturating_sub(start).min(len);
                        if avail > 0 {
                            self.read_buf[c][..avail]
                                .copy_from_slice(&self.record_data[start..start + avail]);
                        }
                        self.read_ptr[c] = self.read_ptr[c].wrapping_add(nbytes as u32);
                        found = Some((len, rn));
                    }
                }
                if let Some((len, num)) = found {
                    return Scan::Entries { len, num, flag };
                }
            } else if ty == TAG_TYPE_UNK_3 {
                return Scan::Unk3;
            } else if ty == TAG_TYPE_UNK_2 {
                // header only; flag bit is handled via consume_flag_tag
            } else {
                break;
            }
            if self.read_ptr[c] >= self.data_size {
                break;
            }
        }
        Scan::End
    }

    /// C: `FUN_overlay_d_93__021ee6b8 @ 0x021ee6b8` - next entries for `client_id`, or a
    /// synthesized end entry (and `read_ended` set) once the record is exhausted.
    pub fn read_next(&mut self, client_id: u8, hooks: &dyn EntryHooks) -> ReadEntries<'_> {
        let c = client_id as usize;
        assert!(c < BTL_CLIENT_MAX, "client id out of range");
        if self.read_ended {
            return self.end_entry(c, hooks, false);
        }
        match self.scan(c, client_id) {
            Scan::Entries { len, num, flag } => ReadEntries {
                entries: &self.read_buf[c][..len],
                num_entries: num,
                tag_flag: flag,
            },
            Scan::Unk3 => self.end_entry(c, hooks, true),
            Scan::End => {
                self.read_ended = true;
                self.end_entry(c, hooks, false)
            }
        }
    }

    /// C: `FUN_overlay_d_93__021ee830 @ 0x021ee830` - number of tags with the flag bit set.
    pub fn count_flagged_tags(&self) -> u32 {
        let mut p: u32 = 0;
        let mut count = 0;
        while p < self.data_size {
            let (ty, num_client, flag) = unpack_header_tag(self.byte(p));
            p += 1;
            if flag {
                count += 1;
            }
            if p >= self.data_size {
                break;
            }
            if ty != TAG_TYPE_ENTRIES {
                p += num_client as u32;
            } else {
                for _ in 0..num_client {
                    let (_, rn) = unpack_client_tag(self.byte(p));
                    p += 1;
                    p = p.wrapping_add(BattleEntry::SIZE as u32 * rn as u32);
                    if p >= self.data_size {
                        break;
                    }
                }
            }
        }
        count
    }

    /// C: `BattleRecordReader_IsAtEnd`
    pub fn is_at_end(&self, client_id: u8) -> bool {
        if !self.read_ended {
            return self.read_ptr[client_id as usize] == self.data_size;
        }
        true
    }
}

// ---------------------------------------------------------------------------------------------
// BattleRecordBuilder
// ---------------------------------------------------------------------------------------------

/// C: `BattleRecordBuilder`. Bitfields live in `flags`: bits 0-5 type, bit 6 tagFlag, bit 7 error
/// (matches the mwcc layout seen in `BattleRecordBuilder_Load`'s asm).
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct BattleRecordBuilder {
    pub write_ptr: u8,
    pub client_bits: u8,
    pub num_clients: u8,
    flags: u8,
    pub buffer: [u8; BTLREC_TOOL_BUFFER_SIZE],
}

const _: () = assert!(core::mem::size_of::<BattleRecordBuilder>() == 0x40);

/// Error from [`BattleRecordBuilder::load`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadTooLarge;

impl BattleRecordBuilder {
    pub const SIZE: usize = 0x40;

    pub fn tag_type(&self) -> u8 {
        self.flags & 0x3f
    }
    pub fn tag_flag(&self) -> bool {
        self.flags & 0x40 != 0
    }
    pub fn error(&self) -> bool {
        self.flags & 0x80 != 0
    }
    fn set_type(&mut self, t: u8) {
        self.flags = (self.flags & !0x3f) | (t & 0x3f);
    }
    fn set_error(&mut self) {
        self.flags |= 0x80;
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut o = [0u8; Self::SIZE];
        o[0] = self.write_ptr;
        o[1] = self.client_bits;
        o[2] = self.num_clients;
        o[3] = self.flags;
        o[4..].copy_from_slice(&self.buffer);
        o
    }

    pub fn from_bytes(b: &[u8; Self::SIZE]) -> Self {
        let mut buffer = [0u8; BTLREC_TOOL_BUFFER_SIZE];
        buffer.copy_from_slice(&b[4..]);
        Self {
            write_ptr: b[0],
            client_bits: b[1],
            num_clients: b[2],
            flags: b[3],
            buffer,
        }
    }

    /// C: `BattleRecordBuilder_Init` - zero everything, then set the tag flag.
    pub fn new(tag_flag: bool) -> Self {
        Self {
            write_ptr: 0,
            client_bits: 0,
            num_clients: 0,
            flags: (tag_flag as u8) << 6,
            buffer: [0; BTLREC_TOOL_BUFFER_SIZE],
        }
    }

    /// C: `FUN_overlay_d_93__021ee8d8 @ 0x021ee8d8` - 2-byte header-only record (code 4, UNK_2).
    pub fn header_only_flag_record(&mut self, tag_flag: bool) -> &[u8] {
        self.buffer[HEADER_CODE] = HeaderCode::Code4 as u8;
        self.buffer[HEADER_TAG] = pack_header_tag(TAG_TYPE_UNK_2, 0, tag_flag);
        &self.buffer[..HEADER_SIZE as usize]
    }

    /// C: `BattleRecordBuilder_AddClientEntries`. Uses the first `num_entries` of `entries`
    /// (panics if fewer are supplied). A client already added is ignored; overflow sets `error`.
    pub fn add_client_entries(&mut self, client_id: u8, entries: &[BattleEntry], num_entries: u8) {
        assert!(entries.len() >= num_entries as usize);
        if self.write_ptr == 0 {
            self.set_type(TAG_TYPE_ENTRIES);
            self.write_ptr = HEADER_SIZE as u8;
        }
        let bit = 1u32.checked_shl(client_id as u32).unwrap_or(0);
        if self.client_bits as u32 & bit == 0 {
            let nbytes = BattleEntry::SIZE * num_entries as usize;
            let end_ptr = self.write_ptr as usize + nbytes + 1;
            if end_ptr <= BTLREC_TOOL_BUFFER_SIZE {
                self.client_bits |= bit as u8;
                self.num_clients = self.num_clients.wrapping_add(1);
                let w = self.write_ptr as usize;
                self.buffer[w] = pack_client_tag(client_id, num_entries);
                for (i, e) in entries[..num_entries as usize].iter().enumerate() {
                    let o = w + 1 + i * 4;
                    self.buffer[o..o + 4].copy_from_slice(&e.to_bytes());
                }
                self.write_ptr = end_ptr as u8;
            } else {
                self.set_error();
            }
        }
    }

    /// C: `BattleRecordBuilder_Finish`. `None` if an error occurred; otherwise the record bytes.
    pub fn finish(&mut self, header_code: HeaderCode) -> Option<&[u8]> {
        if self.error() {
            return None;
        }
        self.buffer[HEADER_CODE] = header_code as u8;
        self.buffer[HEADER_TAG] =
            pack_header_tag(self.tag_type(), self.num_clients, self.tag_flag());
        Some(&self.buffer[..self.write_ptr as usize])
    }

    /// C: `FUN_overlay_d_93__021ee9a4 @ 0x021ee9a4` - 2-byte header-only record (code 0, UNK_3).
    pub fn header_only_unk3_record(&mut self) -> &[u8] {
        self.buffer[HEADER_CODE] = HeaderCode::Code0 as u8;
        self.buffer[HEADER_TAG] = pack_header_tag(TAG_TYPE_UNK_3, 0, false);
        &self.buffer[..HEADER_SIZE as usize]
    }

    /// C: `BattleRecordBuilder_Load` (asm-only, `battle_record_restorestart.s`). Replaces the
    /// buffer with `data`, clears client bits/count/type/tag flag/error.
    /// The C overflows for `data.len() > 0x3C`; here that is rejected and nothing changes.
    pub fn load(&mut self, data: &[u8]) -> Result<(), LoadTooLarge> {
        if data.len() > BTLREC_TOOL_BUFFER_SIZE {
            return Err(LoadTooLarge);
        }
        cpu_copy8(data, &mut self.buffer, data.len() as u32);
        self.write_ptr = data.len() as u8;
        self.client_bits = 0;
        self.num_clients = 0;
        self.flags = 0;
        Ok(())
    }

    /// C: `BattleRecordBuilder_ReadNext`. Reads one client block at `*rp` (0 means start of
    /// payload). Returns `(client_id, num_entries)`. At most `entries.len()` entries (and only
    /// bytes inside the 0x3C buffer) are copied; `*rp` always advances by the full block.
    pub fn read_next(&self, rp: &mut u32, entries: &mut [BattleEntry]) -> Option<(u8, u8)> {
        if *rp == 0 {
            *rp = HEADER_SIZE;
        }
        if *rp < self.write_ptr as u32 {
            let (client_id, num_entries) = unpack_client_tag(self.buffer[*rp as usize]);
            let nbytes = BattleEntry::SIZE * num_entries as usize;
            *rp += 1;
            let start = *rp as usize;
            for (i, e) in entries.iter_mut().enumerate().take(num_entries as usize) {
                let o = start + i * 4;
                match self.buffer.get(o..o + 4) {
                    Some(c) => *e = BattleEntry::from_bytes(c.try_into().ok()?),
                    None => break,
                }
            }
            *rp += nbytes as u32;
            return Some((client_id, num_entries));
        }
        None
    }
}

// ---------------------------------------------------------------------------------------------
// Overlay 93 ports on memory images (ROM-verified, `tools/check-fn.sh black-core battle_record`)
// ---------------------------------------------------------------------------------------------
//
// The typed API above follows `src/battle_record.c` (with its documented clamps). The functions
// below are the overlay-93 machine code ported one to one on the raw little-endian images, for
// callers that keep these objects in emulated memory (the battle client): no bounds are added, so
// a slice must cover every byte the original touches (see each function).
//
// Image layouts (overlay 93):
// * `BattleRecord` (0xC08): +0 u16 write_ptr, +2 u8 overflowed, +8 buf[0xC00].
// * `BattleRecordReader` (0x118): +0 record data address, +4 bits 0-30 data size / bit 31
//   read ended, +8 read_ptr[4] (u32), +0x18 read_buf[4][0x40].
// * `BattleRecordBuilder` (0x40): +0 write_ptr, +1 client bits, +2 client count, +3 flags
//   (bits 0-5 type, bit 6 tag flag, bit 7 error), +4 buffer[0x3C].

/// Functions of this area verified against the ROM (`tools/check-fn.sh black-core battle_record`).
pub const PORTED: &[u32] = &[
    0x021E_E5E4, // BattleRecord_New
    0x021E_E600, // BattleRecord_Free
    0x021E_E63C, // BattleRecord_GetData
    0x021E_E644, // BattleRecordReader_Init
    0x021E_E674, // BattleRecordReader_Reset
    0x021E_E6B8, // reader read next
    0x021E_E830, // reader count flagged tags
    0x021E_E894, // BattleRecordReader_IsAtEnd
    0x021E_E8B4, // BattleRecordBuilder_Init
];

/// Address of the string `"btl_rec.c"` (the file argument of the allocation in `BattleRecord_New`).
pub const BTLREC_SRC_FILE_ADDR: u32 = 0x021F_3ADC;

/// Size of the `BattleRecordReader` image (overlay 93).
pub const READER_SIZE: usize = 0x118;
/// Offset of `read_ptr[0]` in the reader image.
pub const READER_READ_PTR: usize = 0x08;
/// Offset of `read_buf[0]` in the reader image.
pub const READER_READ_BUF: usize = 0x18;
/// Offset of the buffer in the `BattleRecord` image (what `BattleRecord_GetData` returns).
pub const RECORD_BUF: usize = 0x08;

/// Address-level heap of the record functions (unported ARM9 routines).
pub trait RecordHeap {
    /// C: `Heap_AllocDebug` (`FUN_02030734(heap, size, flag, file, line)`): returns the block address.
    fn heap_alloc(&mut self, heap_id: u32, size: u32, flag: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free` (`FUN_020307B0(block)`).
    fn heap_free(&mut self, block: u32);
}

fn rd32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

fn wr32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

/// Runs an [`EntryHooks`] initialiser on the 4-byte entry at `o` of `img`.
fn init_entry(img: &mut [u8], o: usize, f: impl FnOnce(&mut BattleEntry)) {
    let mut e = BattleEntry::from_bytes(img[o..o + 4].try_into().unwrap());
    f(&mut e);
    img[o..o + 4].copy_from_slice(&e.to_bytes());
}

/// C: `BattleRecord_New` at `0x021EE5E4`: `Heap_AllocDebug(heap_id, 0xC08, 1, "btl_rec.c", 107)`;
/// returns the block address (contents not initialised).
pub fn record_new(heap: &mut dyn RecordHeap, heap_id: u32) -> u32 {
    heap.heap_alloc(
        heap_id,
        BattleRecord::SIZE as u32,
        1,
        BTLREC_SRC_FILE_ADDR,
        BTLREC_SRC_LINE,
    )
}

/// C: `BattleRecord_Free` at `0x021EE600`: a tail call to `Heap_Free`.
pub fn record_free(heap: &mut dyn RecordHeap, record: u32) {
    heap.heap_free(record);
}

/// C: `BattleRecord_GetData` at `0x021EE63C`: `*size = write_ptr` (zero-extended u16); returns the
/// offset of the buffer in the record image ([`RECORD_BUF`]).
pub fn record_get_data(record: &[u8], size: &mut u32) -> usize {
    *size = u16::from_le_bytes([record[0], record[1]]) as u32;
    RECORD_BUF
}

/// C: `BattleRecordReader_Init` at `0x021EE644`: stores `record_data` (an address) and
/// `data_size & 0x7FFFFFFF` with the read-ended bit clear, and zeroes the 4 read pointers.
/// `reader` must hold at least 0x18 bytes.
pub fn reader_init(reader: &mut [u8], record_data: u32, data_size: u32) {
    wr32(reader, 0, record_data);
    let old = rd32(reader, 4);
    let w = ((data_size & 0x7FFF_FFFF) | (old & 0x8000_0000)) & 0x7FFF_FFFF;
    wr32(reader, 4, w);
    reader_reset(reader);
}

/// C: `BattleRecordReader_Reset` at `0x021EE674`: zeroes the 4 read pointers (read ended is kept).
pub fn reader_reset(reader: &mut [u8]) {
    for i in 0..BTL_CLIENT_MAX {
        wr32(reader, READER_READ_PTR + 4 * i, 0);
    }
}

/// C: `FUN_overlay_d_93__021ee6b8` at `0x021EE6B8`: the next entries of `client_id` from the record.
/// `data` is the memory at the reader's record-data address; it must cover every byte read (the
/// original reads up to ~0x7D bytes past the data size on malformed streams). Writes
/// `*num_entries` / `*tag_flag` and returns the offset in `reader` of the entries
/// (`read_buf[client_id]`). Found entries (n per client block, up to 31) are copied to
/// `read_buf[client_id]` without a bound, as in the original: `reader` must hold
/// `0x18 + 0x40 * client_id + 4 * n` bytes. At the end of the record the read-ended bit is set and
/// the end entry (`init_end`) is returned; a type-3 tag returns `init_unk3`'s entry.
pub fn reader_read_next(
    reader: &mut [u8],
    data: &[u8],
    client_id: u32,
    num_entries: &mut u8,
    tag_flag: &mut u8,
    hooks: &dyn EntryHooks,
) -> usize {
    let buf = READER_READ_BUF + (client_id << 6) as usize;
    let rpo = READER_READ_PTR + (client_id << 2) as usize;
    if rd32(reader, 4) >> 31 == 0 {
        while rd32(reader, rpo) < rd32(reader, 4) & 0x7FFF_FFFF {
            let b = data[rd32(reader, rpo) as usize];
            let nc = b & 0xF;
            let ty = (b >> 4) & 7;
            *tag_flag = (b >> 7) & 1;
            let rp = rd32(reader, rpo).wrapping_add(1);
            wr32(reader, rpo, rp);
            if rp > rd32(reader, 4) & 0x7FFF_FFFF {
                break;
            }
            if ty == TAG_TYPE_ENTRIES {
                let mut found = 0;
                if nc == 0 || nc > 4 {
                    break;
                }
                for _ in 0..nc {
                    let rp0 = rd32(reader, rpo);
                    let rp = rp0.wrapping_add(1);
                    wr32(reader, rpo, rp);
                    let t = data[rp0 as usize];
                    let id = (t >> 5) & 7;
                    let n = t & 0x1F;
                    if rp >= rd32(reader, 4) & 0x7FFF_FFFF {
                        break;
                    }
                    if id as u32 != client_id {
                        wr32(reader, rpo, rp.wrapping_add(4 * n as u32));
                    } else {
                        let len = 4 * n as u32;
                        cpu_copy8(&data[rp as usize..], &mut reader[buf..], len);
                        found = buf;
                        let p = rd32(reader, rpo).wrapping_add(len);
                        wr32(reader, rpo, p);
                        *num_entries = n;
                    }
                }
                if found != 0 {
                    return found;
                }
            } else if ty == TAG_TYPE_UNK_3 {
                init_entry(reader, buf, |e| hooks.init_unk3(e));
                *num_entries = 1;
                *tag_flag = 0;
                return buf;
            } else if ty != TAG_TYPE_UNK_2 {
                break;
            }
        }
        let w = rd32(reader, 4);
        wr32(reader, 4, (w & 0x7FFF_FFFF) | 0x8000_0000);
    }
    init_entry(reader, buf, |e| hooks.init_end(e));
    *num_entries = 1;
    *tag_flag = 0;
    buf
}

/// C: `FUN_overlay_d_93__021ee830` at `0x021EE830`: number of tags with the flag bit set, walking
/// the whole record from 0 (read pointers untouched). `data` is the memory at the reader's
/// record-data address; only bytes below the data size are read.
pub fn reader_count_flagged_tags(reader: &[u8], data: &[u8]) -> u32 {
    let size = rd32(reader, 4) & 0x7FFF_FFFF;
    let mut p: u32 = 0;
    let mut count: u32 = 0;
    while p < size {
        let b = data[p as usize];
        let nc = (b & 0xF) as u32;
        let ty = (b >> 4) & 7;
        p += 1;
        if (b >> 7) & 1 != 0 {
            count += 1;
        }
        if p >= size {
            break;
        }
        if ty != TAG_TYPE_ENTRIES {
            p += nc;
        } else {
            for _ in 0..nc {
                let n = (data[p as usize] & 0x1F) as u32;
                p = p + 1 + 4 * n;
                if p >= size {
                    break;
                }
            }
        }
    }
    count
}

/// C: `BattleRecordReader_IsAtEnd` at `0x021EE894`: read ended, or `read_ptr[client_id]` equals the
/// data size.
pub fn reader_is_at_end(reader: &[u8], client_id: u32) -> bool {
    let w = rd32(reader, 4);
    if w >> 31 != 0 {
        return true;
    }
    rd32(reader, READER_READ_PTR + (client_id << 2) as usize) == w & 0x7FFF_FFFF
}

/// C: `BattleRecordBuilder_Init` at `0x021EE8B4`: `MI_CpuFill8(builder, 0, 0x40)`, then flags
/// bit 6 = bit 0 of `tag_flag`.
pub fn builder_init(builder: &mut [u8], tag_flag: u32) {
    cpu_fill8(builder, 0, BattleRecordBuilder::SIZE as u32);
    builder[3] = (builder[3] & !0x40) | (((tag_flag as u8) & 1) << 6);
}

#[cfg(test)]
mod tests;
