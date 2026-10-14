//! Item data accessors and item classification (S4; `item.c` of the main ARM9, 0x02020F34..0x02021414).
//!
//! The item records are 36-byte entries of ARC 024 (`black_data::items`). The main ARM9 reads them
//! through [`record_param`] (`FUN_02021014`) and, for the "usage" block at record offset 0x10 that
//! holds the out-of-battle effect of an item (status cures, HP/PP restore, EV and level changes, the
//! friendship deltas), through [`usage_param`] (`FUN_020210C8`). Parameter numbers 0x00-0x11 are the
//! record fields, 0x12-0x3E the usage block fields; they are the arguments the party and bag
//! overlays pass (see `docs/migration/OPEN.md`: those overlays are not in a decompiled set).
//!
//! Tables that live in the main ARM9's rodata (the TM/HM move table, the mail and berry item lists,
//! the pair table, the graphics index table) are passed in as slices; the ROM differential test
//! fills the original's tables with random data and gives the port the same bytes.

use crate::bag::{is_tm, Heap};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0202_0F34,
    0x0202_0FE4,
    0x0202_1014,
    0x0202_10C8,
    0x0202_12B0,
    0x0202_12E4,
    0x0202_1350,
    0x0202_136C,
    0x0202_138C,
    0x0202_13AC,
    0x0202_13C0,
    0x0202_13E0,
    0x0202_13F8,
];

/// Bytes per item record.
pub const RECORD_SIZE: usize = 36;
/// Offset of the usage block inside a record.
pub const USAGE_OFS: usize = 0x10;
/// Number of item ids the loader accepts (ids above this load item 0).
pub const ITEM_ID_MAX: u32 = 0x272;
/// Number of TM/HM slots in the move table (`black_data::tmhm`).
pub const TM_SLOTS: usize = 101;
/// First HM slot of the move table (`HM01`).
const HM_FIRST_SLOT: usize = 92;
/// Number of mails (`FUN_0202136C`'s list).
pub const MAIL_COUNT: usize = 12;
/// Number of berries (`FUN_020213C0`'s list).
pub const BERRY_COUNT: usize = 64;
/// Number of `(a, b)` pairs of `FUN_020213F8`'s table.
pub const PAIR_COUNT: usize = 25;

/// Undecompiled routines the item loader calls.
pub trait ItemHooks {
    /// Callee `FUN_02048F60(archive, member, heap, r3)`: load a member of an archive into a new
    /// block; returns the block (an opaque handle for [`ItemHooks::block`] and [`ItemHooks::free`]).
    /// `r3` is whatever the caller left in the fourth argument register.
    fn archive_load(&mut self, archive: u32, member: u32, heap: u32, r3: u32) -> u32;

    /// The bytes of a loaded item record block (not a call of the original: the original reads
    /// the record through the returned pointer).
    fn block(&mut self, handle: u32) -> [u8; RECORD_SIZE];

    /// Callee `FUN_020307B0(block)`: `Heap_Free`.
    fn free(&mut self, handle: u32);
}

fn rd16(b: &[u8], ofs: usize) -> u32 {
    u16::from_le_bytes([b[ofs], b[ofs + 1]]) as u32
}

/// C: `FUN_02020F34(item, kind, heap)`, the item-data path (`kind` 0): archive 0x18, member = the
/// item id (ids above [`ITEM_ID_MAX`] read item 0). Returns the loaded block.
pub fn load_item_data<H: ItemHooks>(h: &mut H, item: u32, heap: u32) -> u32 {
    let item = if item > ITEM_ID_MAX { 0 } else { item };
    h.archive_load(0x18, item, heap, item)
}

/// C: `FUN_02020F34(item, kind, heap)`: load an item asset: `kind` 0 the item data (archive 0x18),
/// 1 and 2 the two graphics members of the item (archive 0x19, member = the u16 at
/// `gfx_table[item * 4]` / `gfx_table[item * 4 + 2]`, `gfx_table` being the table at 0x0209EB6C).
/// Any other kind loads nothing and returns 0.
pub fn load_item_asset<H: ItemHooks>(
    h: &mut H,
    gfx_table: &[u8],
    item: u32,
    kind: u32,
    heap: u32,
) -> u32 {
    let id = if item > ITEM_ID_MAX { 0 } else { item };
    match kind {
        0 => h.archive_load(0x18, id, heap, id),
        1 | 2 => {
            let member = rd16(gfx_table, id as usize * 4 + (kind as usize - 1) * 2);
            h.archive_load(0x19, member, heap, id << 2)
        }
        _ => 0,
    }
}

/// C: `FUN_02020FE4(item, param, heap)`: parameter `param` of an item: load its record (the heap id
/// gets bit 15 set), read the parameter with [`record_param`], free the record.
pub fn item_param<H: ItemHooks>(h: &mut H, item: u32, param: u32, heap: Heap) -> u32 {
    let heap = (heap as u32 & 0x7FFF) | 0x8000;
    let handle = load_item_data(h, item, heap);
    let rec = h.block(handle);
    let v = record_param(&rec, param);
    h.free(handle);
    v
}

/// C: `FUN_02021014(record, param)`: parameter `param` of an item record. 0x00-0x11 are the record's
/// own fields (0: price, `u16 * 10`); anything above reads the usage block (see [`usage_byte`]).
pub fn record_param(rec: &[u8], param: u32) -> u32 {
    let w8 = rd16(rec, 8);
    match param {
        0 => rd16(rec, 0).wrapping_mul(10),
        1 => rec[2] as u32,
        2 => rec[3] as u32,
        3 => (w8 & 0x3F) >> 5,
        4 => (w8 & 0x7F) >> 6,
        5 => (w8 & 0x7FF) >> 7,
        6 => rec[0xA] as u32,
        7 => rec[0xB] as u32,
        8 => rec[4] as u32,
        9 => rec[5] as u32,
        10 => rec[6] as u32,
        11 => rec[7] as u32,
        0xC => w8 & 0x1F,
        0xD => w8 >> 11,
        0xE => rec[0xC] as u32,
        0xF => rec[0xD] as u32,
        0x10 => rec[0xE] as u32,
        0x11 => rec[0xF] as u32,
        _ => usage_byte(rec, param),
    }
}

/// C: `FUN_020210AA(record, param)`: the usage block field `param` of a record: the usage kind
/// (record byte 0xC) 0 means a single byte at 0x10 (whatever the parameter), 1 a parameter block
/// read by [`usage_param`], anything else 0.
pub fn usage_byte(rec: &[u8], param: u32) -> u32 {
    match rec[0xC] {
        0 => rec[USAGE_OFS] as u32,
        1 => usage_param(&rec[USAGE_OFS..], param),
        _ => 0,
    }
}

/// How `FUN_020210C8` reads one parameter of the usage block.
#[derive(Clone, Copy)]
enum Field {
    /// `ldrb` at the offset, then `lsl a`, `lsr b` on the register: `(byte << a) >> b`.
    Bits(u8, u8, u8),
    /// `ldrb` at the offset.
    U8(u8),
    /// `ldrsb` at the offset (sign-extended to 32 bits).
    S8(u8),
}

use Field::{Bits as F, S8 as S, U8 as U};

/// C: the jump table of `FUN_020210C8`, for parameters 0x12-0x3E.
const USAGE_FIELDS: [Field; 45] = [
    // 0x12-0x19: the bits of byte 0 (status cures, revive flags ...)
    F(0, 31, 31),
    F(0, 30, 31),
    F(0, 29, 31),
    F(0, 28, 31),
    F(0, 27, 31),
    F(0, 26, 31),
    F(0, 25, 31),
    F(0, 24, 31),
    // 0x1A-0x1D: bits 0-3 of byte 1, 0x1E its high nibble
    F(1, 31, 31),
    F(1, 30, 31),
    F(1, 29, 31),
    F(1, 28, 31),
    F(1, 24, 28),
    // 0x1F-0x22: the nibbles of bytes 2 and 3
    F(2, 28, 28),
    F(2, 24, 28),
    F(3, 28, 28),
    F(3, 24, 28),
    // 0x23-0x26: byte 4 (low nibble, bits 4-5, bit 6, bit 7)
    F(4, 28, 28),
    F(4, 26, 30),
    F(4, 25, 31),
    F(4, 24, 31),
    // 0x27-0x2E: the bits of byte 5
    F(5, 31, 31),
    F(5, 30, 31),
    F(5, 29, 31),
    F(5, 28, 31),
    F(5, 27, 31),
    F(5, 26, 31),
    F(5, 25, 31),
    F(5, 24, 31),
    // 0x2F-0x33: bits 0-4 of byte 6
    F(6, 31, 31),
    F(6, 30, 31),
    F(6, 29, 31),
    F(6, 28, 31),
    F(6, 27, 31),
    // 0x34-0x39: signed bytes 7-12 (the EV deltas and the HP/PP amounts)
    S(7),
    S(8),
    S(9),
    S(10),
    S(11),
    S(12),
    // 0x3A, 0x3B: bytes 13, 14
    U(13),
    U(14),
    // 0x3C-0x3E: signed bytes 15-17 (the friendship deltas)
    S(15),
    S(16),
    S(17),
];

/// C: `FUN_020210C8(block, param)`: field `param` (0x12..=0x3E) of the usage block (record bytes
/// 0x10..0x24). Any other parameter gives 0 (`FUN_02021284`). Signed fields come back sign-extended.
pub fn usage_param(block: &[u8], param: u32) -> u32 {
    let Some(f) = param
        .checked_sub(0x12)
        .and_then(|i| USAGE_FIELDS.get(i as usize))
    else {
        return 0;
    };
    match *f {
        F(o, a, b) => ((block[o as usize] as u32) << a) >> b,
        U(o) => block[o as usize] as u32,
        S(o) => block[o as usize] as i8 as i32 as u32,
    }
}

/// C: `FUN_020212B0(item, tm_moves)`: the move a TM/HM item teaches (0 if it is not one). The first
/// 98 slots of the move table are TM01-92 and HM01-06 (items 0x148..=0x1A9), the last three TM93-95
/// (items 0x26A..=0x26C).
pub fn tm_move(item: u32, tm_moves: &[u16]) -> u32 {
    if !is_tm(item) {
        return 0;
    }
    let base = if item < 0x26A { 0x148 } else { 0x26A - 0x62 };
    tm_moves[(item.wrapping_sub(base) & 0xFFFF) as usize] as u32
}

/// C: `sub_020212E4(move, tm_moves)`: whether a move is one of the six HM moves (slots 92..98 of the
/// TM/HM move table).
pub fn is_hm_move(mv: u32, tm_moves: &[u16]) -> bool {
    tm_moves[HM_FIRST_SLOT..HM_FIRST_SLOT + 6]
        .iter()
        .any(|&m| mv == m as u32)
}

/// C: `FUN_02021350(item)`: the HM number (0-5) of an HM item (0x1A4..=0x1A9), else 0xFF.
pub fn hm_index(item: u32) -> u32 {
    if (0x1A4..=0x1A9).contains(&item) {
        (item - 0x1A4) & 0xFF
    } else {
        0xFF
    }
}

/// C: `FUN_0202136C(item, mails)`: whether an item is a mail (the item list at 0x0209E8EC, 12 ids).
pub fn is_mail(item: u32, mails: &[u16]) -> bool {
    mails[..MAIL_COUNT].iter().any(|&m| item == m as u32)
}

/// C: `FUN_0202138C(item, mails)`: the index of a mail item in the mail list; 0 if it is not one.
pub fn mail_index(item: u32, mails: &[u16]) -> u32 {
    mails[..MAIL_COUNT]
        .iter()
        .position(|&m| item == m as u32)
        .unwrap_or(0) as u32
}

/// C: `FUN_020213AC(index, mails)`: the item of a mail index; 0 for an index past the list.
pub fn mail_item(index: u32, mails: &[u16]) -> u32 {
    if index < MAIL_COUNT as u32 {
        mails[index as usize] as u32
    } else {
        0
    }
}

/// C: `FUN_020213C0(item, berries)`: whether an item is a berry (the item list at 0x0209E968, 64 ids).
pub fn is_berry(item: u32, berries: &[u16]) -> bool {
    berries[..BERRY_COUNT].iter().any(|&m| item == m as u32)
}

/// C: `FUN_020213E0(item)`: 0 for the nine items 0x1EC..=0x1F4 (16-bit wrap), else 1.
pub fn outside_1ec_range(item: u32) -> u32 {
    (item.wrapping_add(0xFE14) & 0xFFFF > 8) as u32
}

/// C: `sub_020213F8(value, col, pairs)`: look `value` up in column `col` (0 or 1) of the 25-pair
/// table at 0x0209E904 (`u16` pairs) and return the other column of that row; 0 if absent.
pub fn pair_lookup(value: u32, col: u32, pairs: &[u8]) -> u32 {
    for i in 0..PAIR_COUNT {
        if value == rd16(pairs, i * 4 + col as usize * 2) {
            return rd16(pairs, i * 4 + (col ^ 1) as usize * 2);
        }
    }
    0
}

#[cfg(test)]
mod tests;
