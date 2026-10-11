//! C: src/battle_record.c, include/battle_record.h
//!
//! Deviations from the C (which does unchecked pointer arithmetic): reads past the end of the
//! record yield 0 instead of garbage; copies that would overflow a fixed buffer are clamped;
//! out-of-range client IDs panic.

use crate::battle_common::{BattleEntry, EntryHooks, BTL_CLIENT_MAX};
use crate::heap::{Heap, HeapId};
use crate::mi_memory::cpu_copy8;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heap::HostHeap;

    // Expected vectors were generated by compiling the original battle_record.c on the host
    // (with shims for the undecompiled SDK/heap/overlay calls and the asm-only Load).

    struct Hooks;
    impl EntryHooks for Hooks {
        fn init_unk3(&self, e: &mut BattleEntry) {
            e.unk_00 = 0xAAAA0003;
        }
        fn init_end(&self, e: &mut BattleEntry) {
            e.unk_00 = 0xBBBB0004;
        }
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    fn en(v: &[u32]) -> Vec<BattleEntry> {
        v.iter().map(|&unk_00| BattleEntry { unk_00 }).collect()
    }

    fn builder_a() -> BattleRecordBuilder {
        let mut b = BattleRecordBuilder::new(true);
        b.add_client_entries(0, &en(&[0x11223344, 0x55667788]), 2);
        b.add_client_entries(2, &en(&[0xA1A2A3A4]), 1);
        b.add_client_entries(0, &en(&[0xA1A2A3A4]), 1); // duplicate client: ignored
        b
    }

    #[test]
    fn tag_packing() {
        assert_eq!(pack_header_tag(1, 3, true), 0x93);
        assert_eq!(pack_header_tag(0xF, 0x1F, false), 0x7F);
        assert_eq!(unpack_header_tag(0x93), (1, 3, true));
        assert_eq!(pack_client_tag(7, 31), 0xFF);
        assert_eq!(pack_client_tag(8, 0), 0x00);
        assert_eq!(unpack_client_tag(0x42), (2, 2));
        for c in 0..=255u8 {
            let (t, n, f) = unpack_header_tag(c);
            assert_eq!(pack_header_tag(t, n, f), c); // all bits used
        }
    }

    #[test]
    fn header_code() {
        assert_eq!(HeaderCode::from_u8(4), Some(HeaderCode::Code4));
        assert_eq!(HeaderCode::from_u8(0), Some(HeaderCode::Code0));
        assert_eq!(HeaderCode::from_u8(1), None);
        assert_eq!(header_code_of(&[146, 1]), Some(146));
        assert_eq!(header_code_of(&[]), None);
    }

    #[test]
    fn builder_vectors() {
        let mut b = builder_a();
        assert_eq!(
            hex(b.finish(HeaderCode::Code4).unwrap()),
            "049202443322118877665541a4a3a2a1"
        );
        assert_eq!(
            hex(&b.to_bytes()),
            "10050241049202443322118877665541a4a3a2a10000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
        );
        b.add_client_entries(3, &en(&[0xC0C1C2C3, 0xD0D1D2D3, 0xE0E1E2E3]), 3);
        assert_eq!(
            hex(b.finish(HeaderCode::Code0).unwrap()),
            "009302443322118877665541a4a3a2a163c3c2c1c0d3d2d1d0e3e2e1e0"
        );
        assert_eq!(BattleRecordBuilder::from_bytes(&b.to_bytes()), b);
    }

    #[test]
    fn builder_overflow_and_empty() {
        let mut b = BattleRecordBuilder::new(false);
        let big: Vec<u32> = (1..=14).map(|i| 0x01010101 * i).collect();
        b.add_client_entries(1, &en(&big), 14);
        assert_eq!(
            hex(&b.to_bytes()),
            "3b02010100002e0101010102020202030303030404040405050505060606060707070708080808090909090a0a0a0a0b0b0b0b0c0c0c0c0d0d0d0d0e0e0e0e00"
        );
        assert!(!b.error());
        b.add_client_entries(3, &en(&[1]), 1);
        assert!(b.error());
        assert!(b.finish(HeaderCode::Code4).is_none());
        // finish with nothing added: empty payload but header bytes written
        let mut e = BattleRecordBuilder::new(true);
        assert_eq!(e.finish(HeaderCode::Code4).unwrap().len(), 0);
        assert_eq!(&e.buffer[..2], &[4, 0x80]);
        // num_entries = 0 still registers the client
        let mut z = BattleRecordBuilder::new(false);
        z.add_client_entries(5, &[], 0);
        assert_eq!((z.write_ptr, z.num_clients, z.client_bits), (3, 1, 0x20));
        // client ids >= 8 never match the bitmask
        let mut w = BattleRecordBuilder::new(false);
        w.add_client_entries(9, &[], 0);
        w.add_client_entries(9, &[], 0);
        assert_eq!((w.num_clients, w.client_bits), (2, 0));
    }

    #[test]
    fn header_only_records() {
        let mut b = BattleRecordBuilder::new(false);
        assert_eq!(hex(b.header_only_flag_record(true)), "04a0");
        assert_eq!(hex(b.header_only_flag_record(false)), "0420");
        assert_eq!(hex(b.header_only_unk3_record()), "0030");
    }

    fn stream() -> (BattleRecord, Vec<u8>) {
        let mut rec = BattleRecord::new(&mut HostHeap::default(), HeapId::Unknown).unwrap();
        let q = en(&[0x01020304, 0x05060708]);
        let mut s = BattleRecordBuilder::new(true);
        s.add_client_entries(0, &q, 2);
        s.add_client_entries(1, &q, 1);
        rec.append(s.finish(HeaderCode::Code4).unwrap());
        rec.append(s.header_only_flag_record(true));
        let mut s = BattleRecordBuilder::new(false);
        s.add_client_entries(0, &q, 1);
        rec.append(s.finish(HeaderCode::Code4).unwrap());
        rec.append(s.header_only_unk3_record());
        let (d, n) = rec.data();
        let v = d[..n as usize].to_vec();
        (rec, v)
    }

    #[test]
    fn record_stream_and_reader() {
        let (_, d) = stream();
        assert_eq!(hex(&d), "920204030201080706052104030201a011010403020130");
        let exp0 = [
            (2u8, true, 15u32, false, false, "0403020108070605"),
            (1, false, 22, false, false, "04030201"),
            (1, false, 23, false, true, "0300aaaa"),
            (1, false, 23, true, true, "0400bbbb"),
            (1, false, 23, true, true, "0400bbbb"),
        ];
        let mut rd = BattleRecordReader::new(&d, d.len() as u32);
        assert_eq!(rd.count_flagged_tags(), 2);
        assert!(!rd.consume_flag_tag(0));
        for (n, t, rp, ended, atend, e) in exp0 {
            let r = rd.read_next(0, &Hooks);
            assert_eq!(
                (r.num_entries, r.tag_flag, hex(r.entries)),
                (n, t, e.to_string())
            );
            assert_eq!(rd.read_ptr(0), rp);
            assert_eq!((rd.read_ended(), rd.is_at_end(0)), (ended, atend));
        }
        // client 1
        let mut rd = BattleRecordReader::new(&d, d.len() as u32);
        let r = rd.read_next(1, &Hooks);
        assert_eq!(
            (r.num_entries, r.tag_flag, hex(r.entries)),
            (1, true, "04030201".into())
        );
        assert_eq!(r.entry(0), Some(BattleEntry { unk_00: 0x01020304 }));
        assert_eq!(r.entry(1), None);
        assert_eq!(rd.read_ptr(1), 15);
        let r = rd.read_next(1, &Hooks);
        assert_eq!(hex(r.entries), "0300aaaa");
        assert_eq!(rd.read_ptr(1), 23);
        // reset keeps read_ended
        rd.read_next(1, &Hooks);
        assert!(rd.read_ended());
        rd.reset();
        assert_eq!(rd.read_ptr(1), 0);
        assert!(rd.is_at_end(1));
    }

    #[test]
    fn consume_flag_tag_works() {
        let d = [0xA0u8, 0x11, 0x00];
        let mut rd = BattleRecordReader::new(&d, 3);
        assert!(rd.consume_flag_tag(2));
        assert_eq!(rd.read_ptr(2), 1);
        assert!(!rd.consume_flag_tag(2)); // type 1 tag, not consumed
        assert_eq!(rd.read_ptr(2), 1);
        let d = [0x20u8];
        let mut rd = BattleRecordReader::new(&d, 1);
        assert!(!rd.consume_flag_tag(0));
        assert_eq!(rd.read_ptr(0), 1);
    }

    #[test]
    fn reader_init_truncates_size() {
        let rd = BattleRecordReader::new(&[], 0xFFFF_FFFF);
        assert_eq!(rd.data_size(), 0x7FFF_FFFF);
        let rd = BattleRecordReader::new(&[], 0);
        assert!(rd.is_at_end(3));
    }

    #[test]
    fn record_append_overflow() {
        let mut rec = BattleRecord::new(&mut HostHeap::default(), HeapId::Unknown).unwrap();
        let big = [0x5Au8; 0x401];
        for (i, wp) in [1024u16, 2048, 3072].into_iter().enumerate() {
            rec.append(&big);
            assert_eq!((rec.write_ptr, rec.overflowed), (wp, 0), "{i}");
        }
        // buffer is exactly full (0xC00); one more payload byte overflows
        rec.append(&[0, 1]);
        assert_eq!((rec.write_ptr, rec.overflowed), (3072, 1));
        let mut rec = BattleRecord::from_bytes(&[0u8; 0xC08]).unwrap();
        rec.write_ptr = 0xBFE;
        rec.append(&[0, 1, 2]); // exactly fits
        assert_eq!((rec.write_ptr, rec.overflowed), (0xC00, 0));
        assert_eq!(rec.buf[0xBFF], 2);
        rec.append(&[0, 1, 2]);
        assert_eq!((rec.write_ptr, rec.overflowed), (0xC00, 1));
        assert_eq!(rec.data().1, 0xC00);
    }

    #[test]
    fn record_append_overflow_c_vector() {
        // C run: 3 x 0x401 bytes, then 0x401 -> (3072, overflowed) as printed by the C harness.
        let mut rec = BattleRecord::new(&mut HostHeap::default(), HeapId::Unknown).unwrap();
        let big = [0x5Au8; 0x401];
        for _ in 0..3 {
            rec.append(&big);
        }
        rec.append(&big);
        assert_eq!((rec.write_ptr, rec.overflowed), (3072, 1));
        rec.append(&[0; 3]);
        assert_eq!((rec.write_ptr, rec.overflowed), (3072, 1));
    }

    #[test]
    fn record_append_edge_cases() {
        let mut rec = BattleRecord::new(&mut HostHeap::default(), HeapId::Unknown).unwrap();
        rec.append(&[]);
        rec.append(&[9]);
        assert_eq!((rec.write_ptr, rec.overflowed), (0, 0));
        rec.append(&[9, 1, 2]);
        assert_eq!(&rec.buf[..3], &[1, 2, 0]);
        assert_eq!(rec.write_ptr, 2);
    }

    #[test]
    fn record_heap_and_serialization() {
        let mut heap = HostHeap::default();
        let rec = BattleRecord::new(&mut heap, HeapId::Unknown).unwrap();
        assert_eq!(heap.last_site, Some(("btl_rec.c", 107)));
        assert_eq!(heap.live_blocks, 1);
        assert_eq!(rec.to_bytes().len(), 0xC08);
        rec.free(&mut heap);
        assert_eq!(heap.live_blocks, 0);
        heap.fail_next = true;
        assert!(BattleRecord::new(&mut heap, HeapId::Unknown).is_none());

        let mut r = BattleRecord::from_bytes(&[0u8; 0xC08]).unwrap();
        r.write_ptr = 0x1234;
        r.overflowed = 1;
        r.unk_04 = 0xDEADBEEF;
        r.buf[0] = 7;
        let b = r.to_bytes();
        assert_eq!(&b[..8], &[0x34, 0x12, 1, 0, 0xEF, 0xBE, 0xAD, 0xDE]);
        assert_eq!(BattleRecord::from_bytes(&b), Some(r));
        assert!(BattleRecord::from_bytes(&[0; 5]).is_none());
    }

    #[test]
    fn builder_load_and_read_next() {
        let mut src = builder_a();
        src.add_client_entries(3, &en(&[0xC0C1C2C3, 0xD0D1D2D3, 0xE0E1E2E3]), 3);
        let rec = src.finish(HeaderCode::Code0).unwrap().to_vec();
        let mut b = BattleRecordBuilder::new(true);
        b.error_for_test();
        b.load(&rec).unwrap();
        assert_eq!(
            (b.write_ptr, b.client_bits, b.num_clients, b.tag_type()),
            (29, 0, 0, 0)
        );
        assert!(!b.tag_flag() && !b.error());
        let mut rp = 0;
        let mut out = vec![BattleEntry::default(); 32];
        assert_eq!(b.read_next(&mut rp, &mut out), Some((0, 2)));
        assert_eq!(rp, 11);
        assert_eq!(out[1].unk_00, 0x55667788);
        assert_eq!(b.read_next(&mut rp, &mut out), Some((2, 1)));
        assert_eq!(out[0].unk_00, 0xA1A2A3A4);
        assert_eq!(b.read_next(&mut rp, &mut out), Some((3, 3)));
        assert_eq!(rp, 29);
        assert_eq!(out[2].unk_00, 0xE0E1E2E3);
        assert_eq!(b.read_next(&mut rp, &mut out), None);
        // oversize load rejected, builder untouched
        let before = b.clone();
        assert_eq!(b.load(&[0; 0x3D]), Err(LoadTooLarge));
        assert_eq!(b, before);
        assert!(b.load(&[1; 0x3C]).is_ok());
        assert_eq!(b.write_ptr, 0x3C);
        // load of empty data: nothing to read
        b.load(&[]).unwrap();
        let mut rp = 0;
        assert_eq!(b.read_next(&mut rp, &mut out), None);
        assert_eq!(rp, 2);
    }

    impl BattleRecordBuilder {
        fn error_for_test(&mut self) {
            self.set_error();
            self.client_bits = 0xFF;
            self.num_clients = 9;
        }
    }

    // ---- differential fuzz against the C implementation --------------------------------

    struct Fnv(u64);
    impl Fnv {
        fn b(&mut self, v: u8) {
            self.0 ^= v as u64;
            self.0 = self.0.wrapping_mul(1099511628211);
        }
        fn w(&mut self, v: u32) {
            for x in v.to_le_bytes() {
                self.b(x);
            }
        }
    }

    struct Rng(u32);
    impl Rng {
        fn new(seed: u32) -> Self {
            let s = seed.wrapping_mul(2654435761).wrapping_add(1);
            Rng(if s == 0 { 1 } else { s })
        }
        fn next(&mut self) -> u32 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            self.0 = x;
            x
        }
    }

    fn gen(rng: &mut Rng, buf: &mut [u8; 1024]) -> u32 {
        let mut pos = 0usize;
        let nrec = 1 + rng.next() % 6;
        let mut k = 0;
        while k < nrec && pos < 400 {
            k += 1;
            const TYS: [u8; 6] = [1, 1, 1, 2, 3, 5];
            let ty = TYS[(rng.next() % 6) as usize];
            let f = rng.next() & 1 != 0;
            if ty == 1 {
                let nc = (rng.next() % 6) as u8;
                buf[pos] = pack_header_tag(ty, nc, f);
                pos += 1;
                for _ in 0..nc {
                    let cid = (rng.next() % 8) as u8;
                    let n = (rng.next() % 17) as u8;
                    buf[pos] = pack_client_tag(cid, n);
                    pos += 1;
                    for _ in 0..4 * n as usize {
                        buf[pos] = (rng.next() & 255) as u8;
                        pos += 1;
                    }
                }
            } else {
                let nc = (rng.next() % 3) as u8;
                buf[pos] = pack_header_tag(ty, nc, f);
                pos += 1;
                for _ in 0..nc {
                    buf[pos] = (rng.next() & 255) as u8;
                    pos += 1;
                }
            }
        }
        let r = rng.next();
        if r.is_multiple_of(4) {
            let x = rng.next();
            return x % (pos as u32 + 1);
        }
        pos as u32
    }

    #[test]
    fn reader_matches_c_fuzz() {
        let mut h = Fnv(14695981039346656037);
        for s in 0..3000u32 {
            let mut buf = [0u8; 1024];
            let mut rng = Rng::new(s);
            let size = gen(&mut rng, &mut buf);
            // The C overflows its 0x40-byte read buffer on malformed streams (UB); the
            // harness skipped those seeds, and so do we (Rust clamps instead).
            let save = h.0;
            let mut overflow = false;
            for c in 0..4u8 {
                let mut rd = BattleRecordReader::new(&buf, size);
                h.w(rd.count_flagged_tags());
                h.w(rd.consume_flag_tag(c) as u32);
                h.w(rd.read_ptr(c));
                for i in 0..8 {
                    let (n, t, e) = {
                        let r = rd.read_next(c, &Hooks);
                        (r.num_entries, r.tag_flag, r.entries.to_vec())
                    };
                    overflow |= n as usize * 4 > BTLREC_READBUF_SIZE;
                    h.b(n);
                    h.b(t as u8);
                    h.b(rd.read_ended() as u8);
                    h.w(rd.read_ptr(c));
                    h.w(rd.is_at_end(c) as u32);
                    e.iter().take(4 * n as usize).for_each(|&x| h.b(x));
                    if i == 3 {
                        rd.reset();
                        h.w(rd.consume_flag_tag(c) as u32);
                    }
                }
            }
            if overflow {
                h.0 = save;
            }
        }
        assert_eq!(format!("{:016x}", h.0), "ffcad299f27f9b41");
    }

    #[test]
    fn builder_matches_c_fuzz() {
        let mut h = Fnv(14695981039346656037);
        for s in 0..3000u32 {
            let mut rng = Rng::new(s);
            let ops = 1 + rng.next() % 5;
            let mut b = BattleRecordBuilder::new(rng.next() & 1 != 0);
            for _ in 0..ops {
                let cid = (rng.next() % 8) as u8;
                let n = (rng.next() % 17) as u8;
                let e: Vec<BattleEntry> =
                    (0..n).map(|_| BattleEntry { unk_00: rng.next() }).collect();
                b.add_client_entries(cid, &e, n);
            }
            let code = if rng.next() & 1 != 0 {
                HeaderCode::Code4
            } else {
                HeaderCode::Code0
            };
            let out = b.finish(code).map(|p| p.to_vec());
            h.w(out.is_some() as u32);
            h.w(out.as_ref().map_or(0, |p| p.len() as u32));
            if let Some(p) = &out {
                p.iter().for_each(|&x| h.b(x));
            }
            b.to_bytes().iter().for_each(|&x| h.b(x));
            if let Some(p) = out {
                let mut b2 = BattleRecordBuilder::from_bytes(&[0xEE; 64]);
                b2.load(&p).unwrap();
                b2.to_bytes().iter().for_each(|&x| h.b(x));
                let mut rp = 0;
                let mut o = [BattleEntry::default(); 32];
                while let Some((c, n)) = b2.read_next(&mut rp, &mut o) {
                    h.b(c);
                    h.b(n);
                    h.w(rp);
                    for e in &o[..n as usize] {
                        h.w(e.unk_00);
                    }
                }
                h.w(rp);
            }
        }
        assert_eq!(format!("{:016x}", h.0), "d279a27af8489c13");
    }
}
