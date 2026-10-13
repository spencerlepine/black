//! Move data access (P4; `waza_tool.c`, `unk_0201BAE4.s` 0x0201BD28-0x0201C408, and the learnset reader
//! `FUN_0201ADD0`, T3-FINDINGS.md).
//!
//! A move record is 36 bytes (ARC 21, member = move id), read through a getter with 32 parameters
//! ([`record_param`], `FUN_0201BDB4`). The game keeps loaded records in a small cache (`FUN_0201C328`
//! builds it: `count` slots of 36 bytes plus a `u16` id per slot) or, without one, loads into a single
//! static record on every call ([`move_param`], `FUN_0201BD28`).
//!
//! Specified (ROM side, `tools/rom-diff/specs/black-pokemon/waza.spec`): the cache users (`0201BEB8`, `BF58`,
//! `C020`, `C0BC`, `C1F4`, `C284`), the wrappers over `FUN_0201BD28` (`C1BC`, `C1C8`, `C1D8`, `C1E8`, `C314`),
//! `0201C184`, the cache free `0201C3C0` and test `0201C408`, the move-slot helpers `0201B920` / `B970` /
//! `B9F0` / `BA34` and the learning helpers `02017FB0`, `02018094`, `020180FC`, `020181A0`, `020181FC`,
//! `02018240`, `020182A8`, and the cache constructor `0201C328` (P4).
//!
//! `waza_tool.c` is 0x0201BD28-0x0201C41C; 0x0201BA68-0x0201BD28 is the Pokemon icon code ([`crate::icon`]).
//!
//! Verified per function with `tools/check-fn.sh black-pokemon waza --per-function`. The data comes in
//! through [`WazaHooks`] (and [`PokeToolHooks`] for the Pokemon routines), so the crate stays free of
//! `nds-rom` / `black-data` (D6).
//!
//! Deviations from the original (all outside the verified behaviour): the cache free [`cache_free`] takes the
//! heap addresses of its blocks from the caller ([`CacheBlocks`]; there is no heap here); the learn routines
//! get the learnset buffer as a slice ([`WazaHooks::learnset_read`] fills it) and read past its end as `0xFF`
//! (the planted terminator is inside it); the sign-extended result of [`record_field16`] is returned as `u32`.

use crate::hooks::PokeToolHooks;
use crate::poke_tool::{
    box_get_value, box_set_value, decrypt_on_access, party_get_value, party_set_value, reencrypt,
};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_ADD0,
    0x0201_BD28,
    0x0201_BDB4,
    0x0201_BF44,
    0x0201_C158,
    0x0201_BEB8,
    0x0201_BF58,
    0x0201_C020,
    0x0201_C0BC,
    0x0201_C1BC,
    0x0201_C1C8,
    0x0201_C1D8,
    0x0201_C1E8,
    0x0201_C1F4,
    0x0201_C284,
    0x0201_C314,
    0x0201_C184,
    0x0201_C3C0,
    0x0201_C408,
    0x0201_C328,
    0x0201_B920,
    0x0201_B970,
    0x0201_B9F0,
    0x0201_BA34,
    0x0201_7FB0,
    0x0201_8094,
    0x0201_80FC,
    0x0201_81A0,
    0x0201_81FC,
    0x0201_8240,
    0x0201_82A8,
];

/// Global holding the move-record cache pointer (`DAT_0201C010`).
pub const CACHE_GLOBAL: u32 = 0x0214_6530;
/// The learnset buffer the learn routines fill (`DAT_02018084`, global 0x02146330).
pub const LEARNSET_BUF: u32 = 0x0214_6330;
/// `FUN_02018094` / `FUN_02018240` result when all four slots are taken (`DAT_020180EC` / `DAT_02018298`).
pub const LEARN_FULL: u32 = 0xFFFF;
/// `FUN_02018094` / `FUN_02018240` result when the move is already known (`DAT_020180F0` / `DAT_0201829C`).
pub const LEARN_KNOWN: u32 = 0xFFFE;
/// Field ids of the four move slots (moves, PP, PP ups); the slot index is added.
pub const MOVE_ID: u32 = 0x36;
pub const PP_ID: u32 = 0x3A;
pub const PP_UPS_ID: u32 = 0x3E;
/// `FUN_0201B970` argument meaning "raise to the maximum".
pub const RAISE_TO_MAX: u32 = 0x7F;

/// Size of a move record (ARC 21 member).
pub const RECORD_SIZE: usize = 36;
/// ARC id of the move records.
pub const MOVE_ARC: u32 = 0x15;
/// ARC id of the learnsets.
pub const LEARNSET_ARC: u32 = 0x12;
/// Highest PP-up count that counts.
pub const MAX_PP_UPS: u32 = 3;

/// Getter parameters (`FUN_0201BDB4`), named by the byte they read; only the ones other code uses.
pub mod param {
    /// Base PP (byte 5), the argument of the max-PP formula.
    pub const BASE_PP: u32 = 5;
}

fn rd16(b: &[u8], o: usize) -> u32 {
    u16::from_le_bytes([b[o], b[o + 1]]) as u32
}

/// C: `FUN_0201BDB4`: parameter `param` (0..=31) of a move record; anything above 31 gives 0. Signed bytes
/// come back sign-extended to 32 bits.
///
/// The parameter table (jump table at 0x0201BDC4, read with rizin; Ghidra mis-decompiles parameters 16-24, the record
/// offsets 0x15-0x1D of A7d): 0-3 bytes 0-3; 4 byte 4 (101 reads as 100); 5 byte 5; 6 signed byte 6; 7 byte 0xE (6 reads as 0);
/// 8/9 high/low nibble of byte 7; 10 byte 0xF; 11 `u16` at 8; 12-15 bytes 0xA-0xD; 16/19/22 byte
/// `0x15 + (p - 16)`; 17/20/23 signed byte `0x18 + (p - 17)`; 18/21/24 byte `0x1B + (p - 18)`; 25/26 signed
/// byte 0x12/0x13 if positive, else 0; 27 byte 0x14; 28 `u16` at 0x10; 29 always 0; 30/31 the magnitude
/// of signed byte 0x12/0x13 if negative, else 0.
pub fn record_param(rec: &[u8], param: u32) -> u32 {
    let b = |o: usize| rec[o] as u32;
    let sb = |o: usize| rec[o] as i8 as i32;
    let p = param as usize;
    match param {
        0..=3 => b(p),
        4 => {
            if b(4) == 101 {
                100
            } else {
                b(4)
            }
        }
        5 => b(5),
        6 => sb(6) as u32,
        7 => {
            if b(0xE) == 6 {
                0
            } else {
                b(0xE)
            }
        }
        8 => b(7) >> 4,
        9 => b(7) & 0xF,
        10 => b(0xF),
        11 => rd16(rec, 8),
        12..=15 => b(0xA + p - 12),
        16 | 19 | 22 => b(0x15 + p - 16),
        17 | 20 | 23 => sb(0x18 + p - 17) as u32,
        18 | 21 | 24 => b(0x1B + p - 18),
        25 | 26 => {
            let v = sb(0x12 + p - 25);
            if v > 0 {
                v as u32
            } else {
                0
            }
        }
        27 => b(0x14),
        28 => rd16(rec, 0x10),
        29 => 0,
        30 | 31 => {
            let v = sb(0x12 + p - 30);
            if v >= 0 {
                0
            } else {
                (-v) as u32
            }
        }
        _ => 0,
    }
}

/// C: `FUN_0201BF44`: bit `bit` of the flag word at record offset 0x20. `bit` is a register shift
/// amount, so only its low byte counts and 32..=255 give 0.
pub fn flag_test(rec: &[u8], bit: u32) -> u32 {
    let flags = u32::from_le_bytes([rec[0x20], rec[0x21], rec[0x22], rec[0x23]]);
    let sh = bit & 0xFF;
    if sh >= 32 {
        0
    } else {
        (flags >> sh) & 1
    }
}

/// The move-record cache (`FUN_0201C328`): per slot a record and the move id loaded into it (0 = free).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveCache {
    pub ids: Vec<u16>,
    pub bufs: Vec<[u8; RECORD_SIZE]>,
}

impl MoveCache {
    /// A cache of `count` free slots.
    pub fn new(count: usize) -> Self {
        MoveCache {
            ids: vec![0; count],
            bufs: vec![[0; RECORD_SIZE]; count],
        }
    }
}

/// Callees of the move routines that are archive reads or other areas.
///
/// The differential test stubs them identically on both sides (the spec's stub lines), so only the
/// arguments are verified here.
pub trait WazaHooks {
    /// Callee `ArcLoadMember(dst, 0x15, move)` (`FUN_02048F4C`): load ARC 21 member `move_id` into `dst`.
    fn load_move(&mut self, move_id: u32, dst: &mut [u8; RECORD_SIZE]);

    /// Callee `FUN_0201BD28(move, param)`: [`move_param`] (the caller of this one is a different function).
    fn move_param(&mut self, move_id: u32, param: u32) -> u32;

    /// Callee `FUN_0201AFD4(species, form)`: the index of the species/form in the per-form tables (area
    /// `personal`, [`crate::personal::form_record_index`]).
    fn form_record_index(&mut self, species: u32, form: u32) -> u32;

    /// Callee `ArcLoadMember(dst, 0x12, index)`: load ARC 18 member `index` (the learnset: `(u16 move, u16
    /// level)` pairs ending in `0xFFFF 0xFFFF`) into `dst`. The production hook copies the member from the
    /// extracted assets (D4); the differential test stubs the call (it logs the buffer's address).
    fn load_learnset(&mut self, index: u32, dst: &mut [u8]);

    /// Callee `FUN_0201ADD0(species, form, dst)` as a whole (the learn routines call it; the learn specs stub
    /// it). `buf` is the learnset buffer at `dst` ([`LEARNSET_BUF`]): the production hook fills it, the
    /// differential test leaves the planted bytes.
    fn learnset_read(&mut self, species: u32, form: u32, dst: u32, buf: &mut [u8]);

    /// Callee `FUN_020307B0(block)`: free a heap block (the cache free routine).
    fn free(&mut self, block: u32);

    /// Callee `FUN_02030734(heap, size, 0, "waza_tool.c", line)`: allocate a heap block (the cache
    /// constructor); returns its address. The file string (0x020A7340) is not passed.
    fn alloc(&mut self, heap: u32, size: u32, line: u32) -> u32;
}

/// The heap blocks of a move cache as the game allocated them: the cache struct, its id array, its record
/// array and each record. The production side has no heap, so the caller supplies these addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheBlocks {
    pub cache: u32,
    pub ids: u32,
    pub bufs: u32,
    pub recs: Vec<u32>,
}

/// C: `FUN_0201BD28`: parameter `param` of move `move_id`'s record. With a `cache`: the first slot whose id
/// is `move_id` is used; else the first free slot (id 0) is claimed and loaded; if all are taken by other
/// moves the last slot is reused. Without one the record is loaded into `scratch` (the static record at
/// 0x02146534) on every call, which is also left holding it.
pub fn move_param(
    move_id: u32,
    param: u32,
    cache: Option<&mut MoveCache>,
    scratch: &mut [u8; RECORD_SIZE],
    h: &mut impl WazaHooks,
) -> u32 {
    record_param(record_of(move_id, cache, scratch, h), param)
}

/// The slot search shared by the cache users (`FUN_0201BD28`, `C020`, `C1F4`, ...): the index of `move_id`'s
/// record. The first slot holding the id is used; else the first free slot (id 0) is claimed; when all are
/// taken by other moves the last slot is reused. The claimed slot is loaded.
fn cache_slot(c: &mut MoveCache, move_id: u32, h: &mut impl WazaHooks) -> usize {
    let n = c.ids.len();
    let mut i = 0;
    if n > 0 {
        loop {
            let id = c.ids[i] as u32;
            if id == 0 {
                c.ids[i] = move_id as u16;
                h.load_move(move_id, &mut c.bufs[i]);
                break;
            }
            if move_id == id {
                break;
            }
            i += 1;
            if i >= n {
                break;
            }
        }
    }
    if i == n {
        i -= 1;
        c.ids[i] = move_id as u16;
        h.load_move(move_id, &mut c.bufs[i]);
    }
    i
}

/// The record of `move_id`: through the cache ([`cache_slot`]), or loaded into `scratch` when there is none.
pub fn record_of<'a>(
    move_id: u32,
    cache: Option<&'a mut MoveCache>,
    scratch: &'a mut [u8; RECORD_SIZE],
    h: &mut impl WazaHooks,
) -> &'a [u8; RECORD_SIZE] {
    match cache {
        None => {
            h.load_move(move_id, scratch);
            scratch
        }
        Some(c) => {
            let i = cache_slot(c, move_id, h);
            &c.bufs[i]
        }
    }
}

/// C: `FUN_0201BEB8(move, bit)`: [`flag_test`] of the record of `move`.
pub fn record_flag(
    move_id: u32,
    bit: u32,
    cache: Option<&mut MoveCache>,
    scratch: &mut [u8; RECORD_SIZE],
    h: &mut impl WazaHooks,
) -> u32 {
    flag_test(record_of(move_id, cache, scratch, h), bit)
}

/// C: `FUN_0201BF58(move, a1, a2, p3)`: when byte 0xB of the record is nonzero, the 16-bit value built from its
/// bytes 0xB..0xD (`(B & 0xF) | (C & 0x3F) << 4 | D << 10`, the low 4 bits of `p3` kept where B's bits are not
/// used, masked with `0xFC0F` then `0x03FF`); else `p3` unchanged. Returned sign-extended from 16 bits. `a1` and
/// `a2` are unused (the original's extra arguments).
pub fn record_field16(
    move_id: u32,
    p3: u32,
    cache: Option<&mut MoveCache>,
    scratch: &mut [u8; RECORD_SIZE],
    h: &mut impl WazaHooks,
) -> u32 {
    let rec = record_of(move_id, cache, scratch, h);
    let mut p = p3 as u16;
    if rec[0xB] != 0 {
        let low = ((rec[0xB] as u32 & 0xF) | (p as u32 & 0xFFF0)) & 0xFC0F;
        let mid = (((rec[0xC] as u32 & 0x3F) << 4) | low) & 0x03FF;
        p = (((rec[0xD] as u32) << 10) | mid) as u16;
    }
    p as i16 as i32 as u32
}

/// C: `FUN_0201C020(move)`: how many of record bytes 0x15..=0x17 are nonzero.
pub fn record_pp_ups_count(
    move_id: u32,
    cache: Option<&mut MoveCache>,
    scratch: &mut [u8; RECORD_SIZE],
    h: &mut impl WazaHooks,
) -> u32 {
    let rec = record_of(move_id, cache, scratch, h);
    (0x15..0x18).filter(|&o| rec[o] != 0).count() as u32
}

/// C: `FUN_0201C0BC(move, idx, out)`: `*out` = signed byte `0x18 + idx` of the record if byte `0x15 + idx` is
/// nonzero, else 0. `out` is the 4-byte destination. Returns byte `0x15 + idx` (the `ldrb r0` of the test is r0 at the
/// `pop`; ov93's 021c2f70 stores it, V-0022 audit).
pub fn record_signed_if_set(
    move_id: u32,
    idx: u32,
    out: &mut [u8],
    cache: Option<&mut MoveCache>,
    scratch: &mut [u8; RECORD_SIZE],
    h: &mut impl WazaHooks,
) -> u32 {
    let rec = record_of(move_id, cache, scratch, h);
    let idx = idx as usize;
    let flag = rec[0x15 + idx];
    let v: i32 = if flag != 0 {
        rec[0x18 + idx] as i8 as i32
    } else {
        0
    };
    out[..4].copy_from_slice(&v.to_le_bytes());
    flag as u32
}

/// C: `FUN_0201C1F4(move)`: 1 if byte 4 of the record is 101 (`'e'`).
pub fn record_is_101(
    move_id: u32,
    cache: Option<&mut MoveCache>,
    scratch: &mut [u8; RECORD_SIZE],
    h: &mut impl WazaHooks,
) -> u32 {
    (record_of(move_id, cache, scratch, h)[4] == 101) as u32
}

/// C: `FUN_0201C284(move)`: 1 if byte 0xE of the record is 6.
pub fn record_is_6(
    move_id: u32,
    cache: Option<&mut MoveCache>,
    scratch: &mut [u8; RECORD_SIZE],
    h: &mut impl WazaHooks,
) -> u32 {
    (record_of(move_id, cache, scratch, h)[0xE] == 6) as u32
}

/// C: `FUN_0201C1BC(mv)` (tail call `FUN_0201BD28(mv, 1)`).
pub fn param1(mv: u32, h: &mut impl WazaHooks) -> u32 {
    h.move_param(mv, 1)
}

/// C: `FUN_0201C1C8(mv)`: `FUN_0201BD28(mv, 0)` as a byte.
pub fn param0_u8(mv: u32, h: &mut impl WazaHooks) -> u32 {
    h.move_param(mv, 0) & 0xFF
}

/// C: `FUN_0201C1D8(mv)`: `FUN_0201BD28(mv, 3)` as a halfword.
pub fn param3_u16(mv: u32, h: &mut impl WazaHooks) -> u32 {
    h.move_param(mv, 3) & 0xFFFF
}

/// C: `FUN_0201C1E8(mv)` (tail call `FUN_0201BD28(mv, 2)`).
pub fn param2(mv: u32, h: &mut impl WazaHooks) -> u32 {
    h.move_param(mv, 2)
}

/// C: `sub_0201C314(mv)`: 1 if `FUN_0201BD28(mv, 3)` is nonzero.
pub fn param3_nonzero(mv: u32, h: &mut impl WazaHooks) -> u32 {
    (h.move_param(mv, 3) != 0) as u32
}

/// C: `FUN_0201C184(mv)`: 2 for 240, 1 for 241, 4 for 201, 3 for 258 (signed compares), else 0.
pub fn class_of(mv: u32) -> u32 {
    match mv as i32 {
        240 => 2,
        241 => 1,
        201 => 4,
        258 => 3,
        _ => 0,
    }
}

/// C: `FUN_0201C3C0()`: free the cache: each record, the id array, the record array, then the struct, and
/// clear the global. `blocks` are the addresses of those blocks (the caller's heap); with no cache nothing
/// happens.
pub fn cache_free(cache: &mut Option<MoveCache>, blocks: &CacheBlocks, h: &mut impl WazaHooks) {
    let Some(c) = cache.as_ref() else {
        return;
    };
    for &r in blocks.recs.iter().take(c.ids.len()) {
        h.free(r);
    }
    h.free(blocks.ids);
    h.free(blocks.bufs);
    h.free(blocks.cache);
    *cache = None;
}

/// C: `FUN_0201C408()`: 1 if the cache exists.
pub fn cache_exists(cache: &Option<MoveCache>) -> u32 {
    cache.is_some() as u32
}

/// Pokemon-side callees of the move-slot and learning routines (`FUN_0201C158` is [`PokeToolHooks::move_max_pp`]).
pub trait MoveHooks: WazaHooks + PokeToolHooks {}
impl<T: WazaHooks + PokeToolHooks> MoveHooks for T {}

/// C: `FUN_0201B920(mon, idx)`: 1 if the PP of slot `idx` is below the maximum of its move.
pub fn slot_below_max(mon: &mut [u8], idx: u32, h: &mut impl MoveHooks) -> u32 {
    let mv = party_get_value(mon, MOVE_ID + idx, 0, h) & 0xFFFF;
    if mv == 0 {
        return 0;
    }
    let pp = party_get_value(mon, PP_ID + idx, 0, h) & 0xFF;
    let ups = party_get_value(mon, PP_UPS_ID + idx, 0, h) & 0xFF;
    let max = h.move_max_pp(mv, ups);
    (pp < max) as u32
}

/// C: `FUN_0201B970(mon, idx, add)`: raise the PP of slot `idx` by `add` up to its maximum (`add` =
/// [`RAISE_TO_MAX`]: to the maximum). 0 if the slot is empty or already full, else 1.
pub fn raise_pp(mon: &mut [u8], idx: u32, add: u32, h: &mut impl MoveHooks) -> u32 {
    let mv = party_get_value(mon, MOVE_ID + idx, 0, h) & 0xFFFF;
    if mv == 0 {
        return 0;
    }
    let pp = party_get_value(mon, PP_ID + idx, 0, h) & 0xFF;
    let ups = party_get_value(mon, PP_UPS_ID + idx, 0, h) & 0xFF;
    let max = h.move_max_pp(mv, ups);
    if pp >= max {
        return 0;
    }
    let new = if add == RAISE_TO_MAX {
        max
    } else {
        let s = (pp + (add & 0xFF)) & 0xFF;
        if s <= max {
            s
        } else {
            max
        }
    };
    party_set_value(mon, PP_ID + idx, new, h);
    1
}

/// C: `FUN_0201B9F0(mon)`: copy the current HP (id 0xA1) to id 0xA0, clear id 0x9D, then raise every slot
/// whose PP is below its maximum to the maximum.
pub fn restore_mon(mon: &mut [u8], h: &mut impl MoveHooks) {
    let cur = party_get_value(mon, 0xA1, 0, h);
    party_set_value(mon, 0xA0, cur, h);
    party_set_value(mon, 0x9D, 0, h);
    for i in 0..4 {
        if slot_below_max(mon, i, h) == 1 {
            raise_pp(mon, i, RAISE_TO_MAX, h);
        }
    }
}

/// C: `FUN_0201BA34(party)`: [`restore_mon`] on every member (count at +4, members at +8, stride 0xDC) whose
/// id 0xA9 is nonzero.
pub fn restore_party(party: &mut [u8], h: &mut impl MoveHooks) {
    let n = u32::from_le_bytes(party[4..8].try_into().unwrap()) as i32;
    for i in 0..n.max(0) as usize {
        let off = 8 + i * PARTY_MON_SIZE;
        let m = &mut party[off..off + PARTY_MON_SIZE];
        if party_get_value(m, 0xA9, 0, h) != 0 {
            restore_mon(m, h);
        }
    }
}

/// Size of a Pokemon record in the party array (`FUN_02017430` allocates 0xDC).
const PARTY_MON_SIZE: usize = 0xDC;

/// C: `FUN_02018094(mon, mv)` (the learn routine that writes its slot with [`learn_into_slot`], `FUN_020181FC`):
/// [`LEARN_FULL`] if all four slots are taken by others, [`LEARN_KNOWN`] if `mv` is known, else `mv` after
/// learning it into the first free slot.
pub fn learn_move(mon: &mut [u8], mv: u32, h: &mut impl MoveHooks) -> u32 {
    let enter = decrypt_on_access(mon) as u32;
    let mut ret = LEARN_FULL;
    for i in 0..4 {
        let m = box_get_value(mon, MOVE_ID + i, 0, h);
        if m & 0xFFFF == 0 {
            learn_into_slot(mon, mv, i, h);
            ret = mv;
            break;
        }
        if m & 0xFFFF == mv {
            ret = LEARN_KNOWN;
            break;
        }
    }
    reencrypt(mon, enter);
    ret
}

/// C: `FUN_020181FC(mon, mv, slot)`: put `mv` into `slot` with 0 PP ups and the maximum PP of `mv` (0 for move 0).
pub fn learn_into_slot(mon: &mut [u8], mv: u32, slot: u32, h: &mut impl MoveHooks) {
    box_set_value(mon, MOVE_ID + slot, mv, h);
    box_set_value(mon, PP_UPS_ID + slot, 0, h);
    let pp = if mv != 0 { h.move_max_pp(mv, 0) } else { 0 };
    box_set_value(mon, PP_ID + slot, pp, h);
}

/// C: `FUN_020180FC(mon, mv)`: forget the first move and append `mv` as the 4th, shifting the other three
/// (their PP and PP ups move with them); the new move gets `FUN_0201BD28(mv, 5)` PP and 0 PP ups.
pub fn learn_shift(mon: &mut [u8], mv: u32, h: &mut impl MoveHooks) {
    let enter = decrypt_on_access(mon) as u32;
    let mut moves = [0u32; 4];
    let mut pp = [0u32; 4];
    let mut ups = [0u32; 4];
    for i in 0..3 {
        moves[i] = box_get_value(mon, MOVE_ID + 1 + i as u32, 0, h) & 0xFFFF;
        pp[i] = box_get_value(mon, PP_ID + 1 + i as u32, 0, h) & 0xFF;
        ups[i] = box_get_value(mon, PP_UPS_ID + 1 + i as u32, 0, h) & 0xFF;
    }
    moves[3] = mv & 0xFFFF;
    pp[3] = h.move_param(mv, 5) & 0xFF;
    ups[3] = 0;
    for i in 0..4 {
        box_set_value(mon, MOVE_ID + i as u32, moves[i], h);
        box_set_value(mon, PP_ID + i as u32, pp[i], h);
        box_set_value(mon, PP_UPS_ID + i as u32, ups[i], h);
    }
    reencrypt(mon, enter);
}

/// C: `FUN_020181A0(mon)`: set the PP of every nonempty slot to its maximum (with its PP ups).
pub fn refill_pp(mon: &mut [u8], h: &mut impl MoveHooks) {
    let enter = decrypt_on_access(mon) as u32;
    for i in 0..4 {
        let mv = box_get_value(mon, MOVE_ID + i, 0, h);
        if mv != 0 {
            let ups = box_get_value(mon, PP_UPS_ID + i, 0, h) & 0xFF;
            let max = h.move_max_pp(mv & 0xFFFF, ups);
            box_set_value(mon, PP_ID + i, max, h);
        }
    }
    reencrypt(mon, enter);
}

/// C: `FUN_020182A8(mon, mv, slot)`: put `mv` into `slot` with 0 PP ups; the PP is the maximum, or the old PP
/// if that is smaller and the slot held a move.
pub fn replace_slot(mon: &mut [u8], mv: u32, slot: u32, h: &mut impl MoveHooks) {
    let old_move = box_get_value(mon, MOVE_ID + slot, 0, h);
    let old_pp = box_get_value(mon, PP_ID + slot, 0, h);
    box_set_value(mon, MOVE_ID + slot, mv, h);
    box_set_value(mon, PP_UPS_ID + slot, 0, h);
    let mut pp = 0;
    if mv != 0 {
        let max = h.move_max_pp(mv, 0);
        pp = if old_move != 0 {
            let o = old_pp & 0xFF;
            if max <= o {
                max
            } else {
                o
            }
        } else {
            max
        };
        pp &= 0xFF;
    }
    box_set_value(mon, PP_ID + slot, pp, h);
}

/// C: `sub_02018240(mon, mv)`: as [`learn_move`], with [`replace_slot`] as the slot writer.
pub fn learn_move_replacing(mon: &mut [u8], mv: u32, h: &mut impl MoveHooks) -> u32 {
    let enter = decrypt_on_access(mon) as u32;
    let mut ret = LEARN_FULL;
    for i in 0..4 {
        let m = box_get_value(mon, MOVE_ID + i, 0, h);
        if m & 0xFFFF == 0 {
            replace_slot(mon, mv, i, h);
            ret = mv;
            break;
        }
        if m & 0xFFFF == mv {
            ret = LEARN_KNOWN;
            break;
        }
    }
    reencrypt(mon, enter);
    ret
}

/// C: `FUN_02017FB0(mon)`: forget all four moves, then learn every move of the species' learnset up to the
/// level of the mon. `learnset` is the buffer at [`LEARNSET_BUF`] (`(u16 move, u16 level)` pairs ending in
/// `0xFFFF 0xFFFF`); the species and form are read from the mon, and `learnset_read` fills the buffer first.
pub fn learn_all(mon: &mut [u8], learnset: &mut [u8], h: &mut impl MoveHooks) {
    let enter = decrypt_on_access(mon) as u32;
    let species = box_get_value(mon, 5, 0, h) & 0xFFFF;
    let form = box_get_value(mon, 0x6F, 0, h) & 0xFFFF;
    let level = crate::create::level_of(mon, h) & 0xFFFF;
    h.learnset_read(species, form, LEARNSET_BUF, learnset);
    for i in 0..4 {
        box_set_value(mon, MOVE_ID + i, 0, h);
        box_set_value(mon, PP_ID + i, 0, h);
        box_set_value(mon, PP_UPS_ID + i, 0, h);
    }
    let pair = |o: usize| -> (u32, u32) {
        let g = |k: usize| learnset.get(k).copied().unwrap_or(0xFF) as u32;
        (g(o) | g(o + 1) << 8, g(o + 2) | g(o + 3) << 8)
    };
    let mut o = 0;
    loop {
        let (mv, lv) = pair(o);
        if mv == 0xFFFF && lv == 0xFFFF {
            break;
        }
        if (level & 0xFF) < lv {
            break;
        }
        if learn_move(mon, mv, h) == LEARN_FULL {
            learn_shift(mon, mv, h);
        }
        o += 4;
    }
    reencrypt(mon, enter);
}

/// C: `FUN_0201C158`: maximum PP of `move_id` with `pp_ups` PP Ups (at most 3 count): `base + base * 20 *
/// ups / 100`, in `u8` arithmetic, base being record parameter 5.
pub fn max_pp(move_id: u32, pp_ups: u32, h: &mut impl WazaHooks) -> u32 {
    let ups = pp_ups.min(MAX_PP_UPS);
    let base = h.move_param(move_id, param::BASE_PP) & 0xFF;
    let bonus = (20 * base * ups / 100) & 0xFF;
    (base + bonus) & 0xFF
}

/// C: `FUN_0201ADD0`: load the learnset of `species` / `form` into `dst`
/// (`ArcLoadMember(dst, 0x12, FUN_0201AFD4(species, form))`): ARC 18 member = the per-form record index.
pub fn read_learnset(species: u32, form: u32, dst: &mut [u8], h: &mut impl WazaHooks) {
    let index = h.form_record_index(species, form);
    h.load_learnset(index, dst);
}

/// `FUN_0201C328` allocation lines (the `line` argument of the allocator, `0x241 + k`).
pub mod alloc_line {
    pub const CACHE: u32 = 0x241;
    pub const BUFS: u32 = 0x242;
    pub const IDS: u32 = 0x243;
    pub const RECORD: u32 = 0x248;
}

/// C: `FUN_0201C328(count, heap)`: build the move-record cache unless one exists: allocate the cache struct
/// (0xC bytes), the record-pointer array (`4 * count`), the id array (`2 * count`) and `count` records of 36
/// bytes on `heap`, all ids 0 (free). Returns the blocks (what [`cache_free`] releases), `None` when a cache
/// already existed. `count <= 0` allocates no records (the arrays still get their wrapped sizes).
pub fn cache_create(
    cache: &mut Option<MoveCache>,
    count: u32,
    heap: u32,
    h: &mut impl WazaHooks,
) -> Option<CacheBlocks> {
    if cache.is_some() {
        return None;
    }
    let st = h.alloc(heap, 0xC, alloc_line::CACHE);
    let bufs = h.alloc(heap, count.wrapping_shl(2), alloc_line::BUFS);
    let ids = h.alloc(heap, count.wrapping_shl(1), alloc_line::IDS);
    let n = (count as i32).max(0) as usize;
    let recs = (0..n)
        .map(|_| h.alloc(heap, RECORD_SIZE as u32, alloc_line::RECORD))
        .collect();
    *cache = Some(MoveCache::new(n));
    Some(CacheBlocks {
        cache: st,
        ids,
        bufs,
        recs,
    })
}

#[cfg(test)]
mod tests;
