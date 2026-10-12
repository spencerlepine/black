//! `poke_tool.c` (0x02016BA0-0x0201A26C): the Pokémon record (`BoxPokemon` 0x88 bytes, `Pokemon`
//! 0xDC bytes), its block cipher and the field-ID `get`/`put` switches (P1a/P1b).
//!
//! Layout (offsets into the record; verified by the ROM differential test, see below):
//!
//! | offset | size | field |
//! |---|---|---|
//! | 0x00 | 4 | PID (`personality`); seeds the shuffle and the party-extension cipher |
//! | 0x04 | 2 | flags: bit 0 [`FLAG_PARTY_PLAIN`], bit 1 [`FLAG_BOX_PLAIN`], bit 2 [`FLAG_BAD_EGG`] |
//! | 0x06 | 2 | block checksum, also the block cipher's seed |
//! | 0x08 | 0x80 | four 0x20-byte blocks A-D, stored in the order [`SHUFFLE`] gives for `(PID >> 13) & 31`, XOR-encrypted as one stream |
//! | 0x88 | 0x54 | party extension (status, level, HP and stats, ...), encrypted with the PID as seed |
//!
//! Block A (id 0) holds species, held item, OT ID, EXP, friendship, ability, markings, language, EVs,
//! contest stats and a ribbon word; B (1) the moves, PP, PP ups, IVs/egg/nickname flags, a second
//! ribbon word, gender/form; C (2) the nickname (11 `u16`) and a 64-bit flag word; D (3) the OT
//! name (8 `u16`), dates, locations, Poké Ball.
//!
//! Verified per function with `tools/check-fn.sh black-pokemon poke_tool --per-function`
//! (spec `tools/rom-diff/specs/black-pokemon/poke_tool.spec`).

use crate::hooks::PokeToolHooks;
use black_core::poke_crypt::{checksum, lcg32_step};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_7DBC,
    0x0201_7DE4,
    0x0201_7E1C,
    0x0201_7E40,
    0x0201_7E64,
    0x0201_7E88,
    0x0201_8D10,
    0x0201_8D34,
    0x0201_8D6C,
    0x0201_8D8C,
    0x0201_8DB4,
    0x0201_8E34,
    0x0201_9308,
    0x0201_9388,
    0x0201_9A50,
    0x0201_9ABC,
    0x0201_9BBC,
    0x0201_9C38,
    0x0201_9CE0,
];

/// Size of a `BoxPokemon` (the part the PC boxes store).
pub const BOX_MON_SIZE: usize = 0x88;
/// Size of a `Pokemon` (`FUN_02017430` allocates 0xDC): the box part plus the party extension.
pub const PARTY_MON_SIZE: usize = 0xDC;
/// Offset of the four encrypted blocks.
pub const BLOCKS_OFS: usize = 8;
/// Size of the four blocks together (the cipher and checksum span).
pub const BLOCKS_SIZE: usize = 0x80;
/// Size of one block.
pub const BLOCK_SIZE: usize = 0x20;
/// Offset and size of the party extension.
pub const PARTY_EXT_OFS: usize = 0x88;
pub const PARTY_EXT_SIZE: usize = 0x54;

/// Flags word (offset 4) bit 0: the party extension (and with it the whole record) is in plain
/// state; the party-level enter/exit routines skip all work. Name inferred.
pub const FLAG_PARTY_PLAIN: u16 = 1;
/// Flags bit 1: the four blocks are currently plain (a decrypt-on-access lock is held); the box-level
/// enter/exit routines skip all work. Name inferred.
pub const FLAG_BOX_PLAIN: u16 = 2;
/// Flags bit 2: the checksum did not match when the blocks were last decrypted ("bad egg"). Set by
/// [`box_enter`]; makes species/name reads report a bad egg. Name inferred.
pub const FLAG_BAD_EGG: u16 = 4;

/// Species ID reported for a bad egg (0x28A, `DAT_020192F4`).
pub const BAD_EGG_SPECIES: u32 = 0x28A;
/// Highest valid held-item ID (0x272); larger values read as 0 and are rejected by put.
pub const MAX_ITEM: u32 = 0x272;

/// Block shuffle table at 0x0209E3A4: row `(PID >> 13) & 31`, entry = byte offset (from
/// [`BLOCKS_OFS`]) of logical block A, B, C, D. Rows 24-31 repeat rows 0-7.
pub const SHUFFLE: [u8; 128] = [
    0, 32, 64, 96, 0, 32, 96, 64, 0, 64, 32, 96, 0, 96, 32, 64, 0, 64, 96, 32, 0, 96, 64, 32, 32,
    0, 64, 96, 32, 0, 96, 64, 64, 0, 32, 96, 96, 0, 32, 64, 64, 0, 96, 32, 96, 0, 64, 32, 32, 64,
    0, 96, 32, 96, 0, 64, 64, 32, 0, 96, 96, 32, 0, 64, 64, 96, 0, 32, 96, 64, 0, 32, 32, 64, 96,
    0, 32, 96, 64, 0, 64, 32, 96, 0, 96, 32, 64, 0, 64, 96, 32, 0, 96, 64, 32, 0, 0, 32, 64, 96, 0,
    32, 96, 64, 0, 64, 32, 96, 0, 96, 32, 64, 0, 64, 96, 32, 0, 96, 64, 32, 32, 0, 64, 96, 32, 0,
    96, 64,
];

/// Field IDs of [`box_get`], [`box_put`] and their party-level variants. Names follow the Gen-4
/// `MON_DATA_*` layout (INFERRED from the field offsets, not from symbols); the numbers and offsets
/// are verified against the ROM.
pub mod id {
    pub const PERSONALITY: u32 = 0;
    pub const PARTY_PLAIN_FLAG: u32 = 1;
    pub const BOX_PLAIN_FLAG: u32 = 2;
    pub const BAD_EGG_FLAG: u32 = 3;
    pub const CHECKSUM: u32 = 4;
    /// 0x28A when the bad-egg flag is set.
    pub const SPECIES: u32 = 5;
    /// Get returns 0 above [`super::MAX_ITEM`]; put ignores larger values.
    pub const HELD_ITEM: u32 = 6;
    pub const OT_ID: u32 = 7;
    pub const EXP: u32 = 8;
    pub const FRIENDSHIP: u32 = 9;
    pub const ABILITY: u32 = 10;
    pub const MARKINGS: u32 = 11;
    pub const LANGUAGE: u32 = 12;
    /// EV IDs 13-18 in the order HP, Atk, Def, Speed, SpAtk, SpDef; put keeps the total <= 510.
    pub const EV_HP: u32 = 13;
    pub const EV_SPD: u32 = 18;
    /// Contest stats 19-24: cool, beauty, cute, smart, tough, sheen.
    pub const COOL: u32 = 19;
    pub const SHEEN: u32 = 24;
    /// 29 ribbon flags (bits of the word at block A + 0x1C).
    pub const RIBBON_FIRST: u32 = 25;
    pub const RIBBON_LAST: u32 = 53;
    pub const MOVE1: u32 = 54;
    pub const MOVE4: u32 = 57;
    pub const PP1: u32 = 58;
    pub const PP4: u32 = 61;
    pub const PP_UPS1: u32 = 62;
    pub const PP_UPS4: u32 = 65;
    /// Get only: computed by `FUN_0201C158(move, pp_ups)`, 0 for an empty slot.
    pub const MAX_PP1: u32 = 66;
    pub const MAX_PP4: u32 = 69;
    /// IVs 70-75 in the order HP, Atk, Def, Speed, SpAtk, SpDef (5 bits each, from bit 0).
    pub const IV_HP: u32 = 70;
    pub const IV_SPD: u32 = 75;
    /// Bit 30 of the IV word, or 1 if the bad-egg flag is set.
    pub const IS_EGG: u32 = 76;
    /// 32 flags (bits of the word at block B + 0x14).
    pub const FLAGS2_FIRST: u32 = 77;
    pub const FLAGS2_LAST: u32 = 108;
    pub const FATEFUL: u32 = 109;
    /// Get recomputes the gender (hook), stores it in block B + 0x18 bits 1-2 and the checksum.
    pub const GENDER: u32 = 110;
    pub const FORM: u32 = 111;
    pub const BLOCK_B_19: u32 = 112;
    pub const BLOCK_B_1A_BIT0: u32 = 113;
    pub const BLOCK_B_1C: u32 = 114;
    /// String-buffer nickname (value = buffer handle); the bad-egg name for a bad egg.
    pub const NICKNAME_STRBUF: u32 = 115;
    /// `u16`-array nickname, 11 characters.
    pub const NICKNAME_U16: u32 = 116;
    /// Bit 31 of the IV word.
    pub const IS_NICKNAMED: u32 = 117;
    pub const BLOCK_C_16: u32 = 118;
    pub const BLOCK_C_17: u32 = 119;
    /// 21 flags (bits of the word at block C + 0x18).
    pub const FLAGS3_FIRST: u32 = 120;
    pub const FLAGS3_LAST: u32 = 140;
    pub const OT_NAME_STRBUF: u32 = 141;
    pub const OT_NAME_U16: u32 = 142;
    /// Bytes at block D + 0x10..0x15 (egg/met date).
    pub const DATE1: u32 = 143;
    pub const DATE6: u32 = 148;
    pub const BLOCK_D_16: u32 = 149;
    pub const BLOCK_D_18: u32 = 150;
    pub const POKERUS: u32 = 151;
    pub const BALL: u32 = 152;
    pub const BLOCK_D_1C_LOW7: u32 = 153;
    pub const BLOCK_D_1C_BIT7: u32 = 154;
    pub const BLOCK_D_1D: u32 = 155;
    pub const BLOCK_D_1E: u32 = 156;
    // Party-level only (offsets into the party extension), handled by `party_get`/`party_put`.
    /// u32 at 0x88.
    pub const STATUS: u32 = 157;
    /// Party get: byte at 0x8C. Box get: level computed from EXP by `FUN_0201850C`.
    pub const LEVEL: u32 = 158;
    /// byte at 0x8D.
    pub const PARTY_8D: u32 = 159;
    /// u16 at 0x8E, 0x90, ... 0x9A.
    pub const HP: u32 = 160;
    pub const MAX_HP: u32 = 161;
    pub const ATK: u32 = 162;
    pub const DEF: u32 = 163;
    pub const SPEED: u32 = 164;
    pub const SPATK: u32 = 165;
    pub const SPDEF: u32 = 166;
    /// 0x38 bytes at 0x9C copied to/from the value pointer.
    pub const PARTY_EXT_BYTES: u32 = 167;
    /// Party get returns 1, put does nothing.
    pub const PARTY_A8: u32 = 168;
    /// Get: species != 0.
    pub const IS_VALID: u32 = 169;
    /// Get: 1 for a bad egg, else the egg bit.
    pub const IS_EGG_OR_BAD: u32 = 170;
    /// Get: species, 0x28A for an egg or bad egg, 0 for an empty slot.
    pub const SPECIES_OR_EGG: u32 = 171;
    /// Get: IV word & 0x3FFFFFFF. Put: unpacks the 6 IVs from a word in the order HP, Atk, Def,
    /// SpAtk, SpDef, Speed (bits 0, 5, 10, 15, 20, 25).
    pub const IVS_PACKED: u32 = 172;
    /// Get: 0 for a Nidoran line (species 29/32) with the nickname flag clear, else 1.
    pub const NICKNAME_DISPLAY: u32 = 173;
    /// Get: personal-data type 1 / type 2 (Arceus with ability 121 uses the plate: [`plate_type`]).
    pub const TYPE1: u32 = 174;
    pub const TYPE2: u32 = 175;
    /// Put only: like 115/116 without updating the nickname flag.
    pub const NICKNAME_STRBUF_RAW: u32 = 176;
    pub const NICKNAME_U16_RAW: u32 = 177;
}

fn rd8(b: &[u8], o: usize) -> u32 {
    b[o] as u32
}

fn rd16(b: &[u8], o: usize) -> u32 {
    u16::from_le_bytes([b[o], b[o + 1]]) as u32
}

fn rd32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

fn wr8(b: &mut [u8], o: usize, v: u32) {
    b[o] = v as u8;
}

fn wr16(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 2].copy_from_slice(&(v as u16).to_le_bytes());
}

fn wr32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

/// C: `FUN_02019A50`: XOR the first `size / 2` halfwords of `data` with the keystream of the 32-bit
/// LCG (`FUN_02019A7C`, `black_core::poke_crypt::lcg32_step`) started at `seed`. Applying it twice
/// restores the data. The blocks use the 16-bit checksum as seed, the party extension the PID.
pub fn crypt(data: &mut [u8], size: u32, seed: u32) {
    let mut state = seed;
    for i in 0..(size >> 1) as usize {
        let key = lcg32_step(&mut state);
        let v = rd16(data, 2 * i) ^ key;
        wr16(data, 2 * i, v);
    }
}

/// C: `FUN_02019C38`: byte offset (from the record start) of logical block `blk` (0-3 = A-D) for a
/// record with this PID: `8 + SHUFFLE[((pid & 0x3E000) >> 13) * 4 + blk]`. The original returns
/// `mon + 8 + entry`.
pub fn block_offset(pid: u32, blk: u32) -> usize {
    BLOCKS_OFS + SHUFFLE[(((pid & 0x3E000) >> 13) * 4 + blk) as usize] as usize
}

/// C: `FUN_02017DBC`: decrypt-on-access. If the record is not yet marked plain, mark it
/// ([`FLAG_BOX_PLAIN`]) and decrypt the blocks with the stored checksum as seed. Returns whether it
/// did (the checksum is not verified here).
pub fn decrypt_on_access(mon: &mut [u8]) -> bool {
    let flags = rd16(mon, 4);
    let was_encrypted = flags & FLAG_BOX_PLAIN as u32 == 0;
    if was_encrypted {
        wr16(mon, 4, flags | FLAG_BOX_PLAIN as u32);
        let seed = rd16(mon, 6);
        crypt(&mut mon[BLOCKS_OFS..], BLOCKS_SIZE as u32, seed);
    }
    was_encrypted
}

/// C: `FUN_02017DE4`: inverse of [`decrypt_on_access`]. If the record is marked plain and
/// `commit == 1`: clear the mark, store the blocks' checksum and encrypt them with it. Returns 1 if
/// it did, else 0.
pub fn reencrypt(mon: &mut [u8], commit: u32) -> u32 {
    let flags = rd16(mon, 4);
    if (flags & 3) >> 1 == 1 && commit == 1 {
        wr16(mon, 4, flags & 0xFFFD);
        let sum = checksum(&mon[BLOCKS_OFS..], BLOCKS_SIZE as u32) as u32;
        wr16(mon, 6, sum);
        let seed = rd16(mon, 6);
        crypt(&mut mon[BLOCKS_OFS..], BLOCKS_SIZE as u32, seed);
    }
    ((flags & 3) >> 1 == 1 && commit == 1) as u32
}

/// C: `FUN_02018D34`: enter a box-level access. Unless [`FLAG_BOX_PLAIN`] is set, decrypt the blocks
/// and verify the checksum; on a mismatch set [`FLAG_BAD_EGG`] and return 0 (the blocks stay
/// decrypted). Returns 1 otherwise.
pub fn box_enter(mon: &mut [u8]) -> u32 {
    let mut ok = 1;
    if rd16(mon, 4) & FLAG_BOX_PLAIN as u32 == 0 {
        let seed = rd16(mon, 6);
        crypt(&mut mon[BLOCKS_OFS..], BLOCKS_SIZE as u32, seed);
        let sum = checksum(&mon[BLOCKS_OFS..], BLOCKS_SIZE as u32) as u32;
        if sum != rd16(mon, 6) {
            ok = 0;
            let f = rd16(mon, 4) | FLAG_BAD_EGG as u32;
            wr16(mon, 4, f);
        }
    }
    ok
}

/// C: `FUN_02018D8C`: leave a box-level access. Unless [`FLAG_BOX_PLAIN`] is set, store the
/// checksum of the (decrypted) blocks and encrypt them with it.
pub fn box_exit(mon: &mut [u8]) {
    if rd16(mon, 4) & FLAG_BOX_PLAIN as u32 == 0 {
        let sum = checksum(&mon[BLOCKS_OFS..], BLOCKS_SIZE as u32) as u32;
        wr16(mon, 6, sum);
        let seed = rd16(mon, 6);
        crypt(&mut mon[BLOCKS_OFS..], BLOCKS_SIZE as u32, seed);
    }
}

/// C: `FUN_02018D10`: enter a party-level access. Unless [`FLAG_PARTY_PLAIN`] is set, decrypt the
/// party extension with the PID and then [`box_enter`] (returns its result); returns 1 otherwise.
pub fn party_enter(mon: &mut [u8]) -> u32 {
    let mut ok = 1;
    if rd16(mon, 4) & FLAG_PARTY_PLAIN as u32 == 0 {
        let pid = rd32(mon, 0);
        crypt(&mut mon[PARTY_EXT_OFS..], PARTY_EXT_SIZE as u32, pid);
        ok = box_enter(mon);
    }
    ok
}

/// C: `FUN_02018D6C`: leave a party-level access: unless [`FLAG_PARTY_PLAIN`] is set, re-encrypt
/// the party extension and [`box_exit`].
pub fn party_exit(mon: &mut [u8]) {
    if rd16(mon, 4) & FLAG_PARTY_PLAIN as u32 == 0 {
        let pid = rd32(mon, 0);
        crypt(&mut mon[PARTY_EXT_OFS..], PARTY_EXT_SIZE as u32, pid);
        box_exit(mon);
    }
}

/// C: `FUN_02019BBC`: the type an Arceus plate (item `0x12A..=0x139`) gives, in the Gen-5 type
/// numbering (Fire 9, Water 10, ... Dark 16; checked against the personal data of Bulbasaur, 11/3);
/// 0 (Normal) for any other item. Plate order by item ID: Flame, Splash, Zap, Meadow, Icicle, Fist,
/// Toxic, Earth, Sky, Mind, Insect, Stone, Spooky, Draco, Dread, Iron (plate names INFERRED from the
/// types).
pub fn plate_type(item: u32) -> u32 {
    match item.wrapping_sub(0x12A) {
        0 => 9,
        1 => 10,
        2 => 12,
        3 => 11,
        4 => 14,
        5 => 1,
        6 => 3,
        7 => 4,
        8 => 2,
        9 => 13,
        10 => 6,
        11 => 5,
        12 => 7,
        13 => 15,
        14 => 16,
        15 => 8,
        _ => 0,
    }
}

/// C: `FUN_02019ABC`: whether the nickname (block C) differs from the species' default name.
/// Returns 1 or 0.
pub fn is_nicknamed(mon: &[u8], h: &mut dyn PokeToolHooks) -> u32 {
    let pid = rd32(mon, 0);
    let a = block_offset(pid, 0);
    let c = block_offset(pid, 2);
    h.species_name_to_scratch(rd16(mon, a));
    (h.scratch_equals_mon_string(c) == 0) as u32
}

/// C: `FUN_02019CE0`: copy a whole `Pokemon` (0xDC bytes) from `src` to `dst`.
pub fn copy_pokemon(src: &[u8], dst: &mut [u8]) {
    dst[..PARTY_MON_SIZE].copy_from_slice(&src[..PARTY_MON_SIZE]);
}

/// C: `FUN_02018E34`: read field `id` of the record's (already decrypted) blocks. `val` is only
/// used as a destination handle by the string ids (115, 116, 141, 142). Unknown ids give 0. Id 110
/// also writes the gender and the checksum back into `mon`.
pub fn box_get(mon: &mut [u8], id: u32, val: u32, h: &mut dyn PokeToolHooks) -> u32 {
    let pid = rd32(mon, 0);
    let flags = rd16(mon, 4);
    let bad_egg = flags & FLAG_BAD_EGG as u32 != 0;
    let a = block_offset(pid, 0);
    let b = block_offset(pid, 1);
    let c = block_offset(pid, 2);
    let d = block_offset(pid, 3);
    match id {
        0 => pid,
        1 => flags & 1,
        2 => (flags >> 1) & 1,
        3 => (flags >> 2) & 1,
        4 => rd16(mon, 6),
        5 => {
            if bad_egg {
                BAD_EGG_SPECIES
            } else {
                rd16(mon, a)
            }
        }
        6 => {
            let item = rd16(mon, a + 2);
            if item > MAX_ITEM {
                0
            } else {
                item
            }
        }
        7 => rd32(mon, a + 4),
        8 => rd32(mon, a + 8),
        9..=0x18 => rd8(mon, a + 0xC + (id - 9) as usize),
        0x19..=0x35 => (rd32(mon, a + 0x1C) >> (id - 0x19)) & 1,
        0x36..=0x39 => rd16(mon, b + (id - 0x36) as usize * 2),
        0x3A..=0x3D => rd8(mon, b + 8 + (id - 0x3A) as usize),
        0x3E..=0x41 => rd8(mon, b + 0xC + (id - 0x3E) as usize),
        0x42..=0x45 => {
            let i = (id - 0x42) as usize;
            let mv = rd16(mon, b + i * 2);
            if mv != 0 {
                h.move_max_pp(mv, rd8(mon, b + 0xC + i))
            } else {
                0
            }
        }
        0x46..=0x4B => (rd32(mon, b + 0x10) >> ((id - 0x46) * 5)) & 0x1F,
        0x4C | 0xAA => {
            if bad_egg {
                1
            } else {
                (rd32(mon, b + 0x10) >> 30) & 1
            }
        }
        0x4D..=0x6C => (rd32(mon, b + 0x14) >> (id - 0x4D)) & 1,
        0x6D => rd8(mon, b + 0x18) & 1,
        0x6E => {
            let g = h.gender(rd16(mon, a), rd8(mon, b + 0x18) >> 3, pid);
            let cur = rd8(mon, b + 0x18);
            wr8(mon, b + 0x18, (cur & 0xF9) | ((g & 3) << 1));
            let sum = checksum(&mon[BLOCKS_OFS..], BLOCKS_SIZE as u32) as u32;
            wr16(mon, 6, sum);
            g
        }
        0x6F => rd8(mon, b + 0x18) >> 3,
        0x70 => rd8(mon, b + 0x19),
        0x71 => rd16(mon, b + 0x1A) & 1,
        0x72 => rd32(mon, b + 0x1C),
        0x73 => {
            if bad_egg {
                h.bad_egg_name_strbuf(val);
            } else {
                h.strbuf_from_mon(val, c);
            }
            0
        }
        0x74 => {
            if bad_egg {
                h.bad_egg_name_u16(val, 0xB);
            } else {
                h.u16str_from_mon(c, val, 0xB);
            }
            0
        }
        0x75 => rd32(mon, b + 0x10) >> 31,
        0x76 => rd8(mon, c + 0x16),
        0x77 => rd8(mon, c + 0x17),
        0x78..=0x8C => ((rd32(mon, c + 0x18) >> (id - 0x78)) & 1 != 0) as u32,
        0x8D => {
            h.strbuf_from_mon(val, d);
            0
        }
        0x8E => {
            h.u16str_from_mon(d, val, 8);
            0
        }
        0x8F..=0x94 => rd8(mon, d + 0x10 + (id - 0x8F) as usize),
        0x95 => rd16(mon, d + 0x16),
        0x96 => rd16(mon, d + 0x18),
        0x97 => rd8(mon, d + 0x1A),
        0x98 => rd8(mon, d + 0x1B),
        0x99 => rd8(mon, d + 0x1C) & 0x7F,
        0x9A => rd8(mon, d + 0x1C) >> 7,
        0x9B => rd8(mon, d + 0x1D),
        0x9C => rd16(mon, d + 0x1E),
        0x9E => h.level_from_exp(rd16(mon, a), rd8(mon, b + 0x18) >> 3, rd32(mon, a + 8)),
        0xA9 => (rd16(mon, a) != 0) as u32,
        0xAB => {
            let sp = rd16(mon, a);
            if sp == 0 {
                0
            } else if (rd32(mon, b + 0x10) >> 30) & 1 != 0 || bad_egg {
                BAD_EGG_SPECIES
            } else {
                sp
            }
        }
        0xAC => rd32(mon, b + 0x10) & 0x3FFF_FFFF,
        0xAD => {
            let sp = rd16(mon, a);
            if (sp == 0x1D || sp == 0x20) && rd32(mon, b + 0x10) >> 31 == 0 {
                0
            } else {
                1
            }
        }
        0xAE | 0xAF => {
            let sp = rd16(mon, a);
            if sp == 0x1ED && rd8(mon, a + 0xD) == 0x79 {
                plate_type(rd16(mon, a + 2))
            } else {
                h.personal_param(sp, rd8(mon, b + 0x18) >> 3, id - 0xA8)
            }
        }
        _ => 0,
    }
}

/// Sets or clears bit `n` of the word at `ofs` depending on `val != 0` (the ribbon/flag puts).
fn put_bit(mon: &mut [u8], ofs: usize, n: u32, val: u32) {
    let mask = 1u32 << n;
    let w = rd32(mon, ofs);
    wr32(mon, ofs, if val != 0 { w | mask } else { w & !mask });
}

/// The EV puts: `cur + min(requested - cur, 510 - total)` clamped to 255, where the signed
/// difference is compared like the original (`int`), and the sum wraps like `uint`.
fn put_ev(mon: &mut [u8], a: usize, ofs: usize, val: u32) {
    let cur = rd8(mon, a + ofs);
    let total: u32 = (0x10..=0x15).map(|o| rd8(mon, a + o)).sum();
    let room = 0x1FEi32.wrapping_sub(total as i32);
    let mut v = val;
    if room < v.wrapping_sub(cur) as i32 {
        v = cur.wrapping_add(room as u32);
    }
    if v > 0xFF {
        v = 0xFF;
    }
    wr8(mon, a + ofs, v);
}

/// C: `FUN_02019388`: write field `id` of the record's (already decrypted) blocks. `val` is the
/// new value, or a source handle for the string ids (115, 116, 141, 142, 176, 177). Ids without a
/// writer (66-69, 117, 157-175, > 177) do nothing.
pub fn box_put(mon: &mut [u8], id: u32, val: u32, h: &mut dyn PokeToolHooks) {
    let pid = rd32(mon, 0);
    let a = block_offset(pid, 0);
    let b = block_offset(pid, 1);
    let c = block_offset(pid, 2);
    let d = block_offset(pid, 3);
    match id {
        0 => wr32(mon, 0, val),
        1 => {
            let f = rd16(mon, 4);
            wr16(mon, 4, (val & 1) | (f & 0xFFFE));
        }
        2 => {
            let f = rd16(mon, 4);
            wr16(mon, 4, ((val & 1) << 1) | (f & 0xFFFD));
        }
        3 => {
            let f = rd16(mon, 4);
            wr16(mon, 4, ((val & 1) << 2) | (f & 0xFFFB));
        }
        4 => wr16(mon, 6, val),
        5 => wr16(mon, a, val),
        6 => {
            if val <= MAX_ITEM {
                wr16(mon, a + 2, val);
            }
        }
        7 => wr32(mon, a + 4, val),
        8 => wr32(mon, a + 8, val),
        9..=0xC => wr8(mon, a + 0xC + (id - 9) as usize, val),
        0xD..=0x12 => put_ev(mon, a, 0x10 + (id - 0xD) as usize, val),
        0x13..=0x18 => wr8(mon, a + 0x16 + (id - 0x13) as usize, val),
        0x19..=0x35 => put_bit(mon, a + 0x1C, id - 0x19, val),
        0x36..=0x39 => wr16(mon, b + (id - 0x36) as usize * 2, val),
        0x3A..=0x3D => wr8(mon, b + 8 + (id - 0x3A) as usize, val),
        0x3E..=0x41 => wr8(mon, b + 0xC + (id - 0x3E) as usize, val),
        0x46..=0x4B => {
            let sh = (id - 0x46) * 5;
            let w = rd32(mon, b + 0x10);
            wr32(mon, b + 0x10, ((val & 0x1F) << sh) | (w & !(0x1F << sh)));
        }
        0x4C => {
            let w = rd32(mon, b + 0x10);
            wr32(mon, b + 0x10, ((val & 1) << 30) | (w & 0xBFFF_FFFF));
        }
        0x4D..=0x6C => put_bit(mon, b + 0x14, id - 0x4D, val),
        0x6D => {
            let cur = rd8(mon, b + 0x18);
            wr8(mon, b + 0x18, (val & 1) | (cur & 0xFE));
        }
        0x6E => {
            let cur = rd8(mon, b + 0x18);
            let g = h.gender(rd16(mon, a), cur >> 3, pid);
            wr8(mon, b + 0x18, ((g & 3) << 1) | (cur & 0xF9));
        }
        0x6F => {
            let cur = rd8(mon, b + 0x18);
            wr8(mon, b + 0x18, ((val & 0x1F) << 3) | (cur & 7));
        }
        0x70 => wr8(mon, b + 0x19, val),
        0x71 => {
            let cur = rd16(mon, b + 0x1A);
            wr16(mon, b + 0x1A, (val & 1) | (cur & 0xFFFE));
        }
        0x72 => wr32(mon, b + 0x1C, val),
        0x73 | 0x74 => {
            if id == 0x73 {
                h.strbuf_to_mon(val, c, 0xB);
            } else {
                h.u16str_to_mon(val, c, 0xB);
            }
            let named = is_nicknamed(mon, h);
            let w = rd32(mon, b + 0x10);
            wr32(mon, b + 0x10, (named << 31) | (w & 0x7FFF_FFFF));
        }
        0x76 => wr8(mon, c + 0x16, val),
        0x77 => wr8(mon, c + 0x17, val),
        0x78..=0x8C => put_bit(mon, c + 0x18, id - 0x78, val),
        0x8D => h.strbuf_to_mon(val, d, 8),
        0x8E => h.u16str_to_mon(val, d, 8),
        0x8F..=0x94 => wr8(mon, d + 0x10 + (id - 0x8F) as usize, val),
        0x95 => wr16(mon, d + 0x16, val),
        0x96 => wr16(mon, d + 0x18, val),
        0x97 => wr8(mon, d + 0x1A, val),
        0x98 => wr8(mon, d + 0x1B, val),
        0x99 => {
            let cur = rd8(mon, d + 0x1C);
            wr8(mon, d + 0x1C, (val & 0x7F) | (cur & 0x80));
        }
        0x9A => {
            let cur = rd8(mon, d + 0x1C);
            wr8(mon, d + 0x1C, ((val & 1) << 7) | (cur & 0x7F));
        }
        0x9B => wr8(mon, d + 0x1D, val),
        0x9C => wr16(mon, d + 0x1E, val),
        0xAC => {
            let mut w = (rd32(mon, b + 0x10) & !0x1F) | (val & 0x1F);
            w = (w & 0xFFFF_FC1F) | (((val >> 5) & 0x1F) << 5);
            w = (w & 0xFFFF_83FF) | (((val >> 10) & 0x1F) << 10);
            w = (w & 0xFE0F_FFFF) | (((val >> 15) & 0x1F) << 20);
            w = (w & 0xC1FF_FFFF) | (((val >> 20) & 0x1F) << 25);
            w = (w & 0xFFF0_7FFF) | (((val >> 25) & 0x1F) << 15);
            wr32(mon, b + 0x10, w);
        }
        0xB0 => h.strbuf_to_mon(val, c, 0xB),
        0xB1 => h.u16str_to_mon(val, c, 0xB),
        _ => {}
    }
}

/// C: `FUN_02018DB4`: party-level get: ids 157-168 read the party extension, everything else is
/// [`box_get`].
pub fn party_get(mon: &mut [u8], id: u32, val: u32, h: &mut dyn PokeToolHooks) -> u32 {
    match id {
        0x9D => rd32(mon, 0x88),
        0x9E => rd8(mon, 0x8C),
        0x9F => rd8(mon, 0x8D),
        0xA0 => rd16(mon, 0x8E),
        0xA1 => rd16(mon, 0x90),
        0xA2 => rd16(mon, 0x92),
        0xA3 => rd16(mon, 0x94),
        0xA4 => rd16(mon, 0x96),
        0xA5 => rd16(mon, 0x98),
        0xA6 => rd16(mon, 0x9A),
        0xA7 => {
            h.copy_ext_out(0x9C, val);
            1
        }
        0xA8 => 1,
        _ => box_get(mon, id, val, h),
    }
}

/// C: `FUN_02019308`: party-level put: ids 157-167 write the party extension (168 does nothing),
/// everything else is [`box_put`].
pub fn party_put(mon: &mut [u8], id: u32, val: u32, h: &mut dyn PokeToolHooks) {
    match id {
        0x9D => wr32(mon, 0x88, val),
        0x9E => wr8(mon, 0x8C, val),
        0x9F => wr8(mon, 0x8D, val),
        0xA0 => wr16(mon, 0x8E, val),
        0xA1 => wr16(mon, 0x90, val),
        0xA2 => wr16(mon, 0x92, val),
        0xA3 => wr16(mon, 0x94, val),
        0xA4 => wr16(mon, 0x96, val),
        0xA5 => wr16(mon, 0x98, val),
        0xA6 => wr16(mon, 0x9A, val),
        0xA7 => h.copy_ext_in(val, 0x9C),
        0xA8 => {}
        _ => box_put(mon, id, val, h),
    }
}

/// C: `FUN_02017E88`: get a field of a `BoxPokemon`: [`box_enter`], [`box_get`], [`box_exit`].
pub fn box_get_value(mon: &mut [u8], id: u32, val: u32, h: &mut dyn PokeToolHooks) -> u32 {
    box_enter(mon);
    let r = box_get(mon, id, val, h);
    box_exit(mon);
    r
}

/// C: `FUN_02017E64`: set a field of a `BoxPokemon`; skipped if the checksum check in [`box_enter`]
/// failed.
pub fn box_set_value(mon: &mut [u8], id: u32, val: u32, h: &mut dyn PokeToolHooks) {
    if box_enter(mon) == 1 {
        box_put(mon, id, val, h);
    }
    box_exit(mon);
}

/// C: `FUN_02017E1C`: get a field of a `Pokemon`: [`party_enter`], [`party_get`], [`party_exit`].
pub fn party_get_value(mon: &mut [u8], id: u32, val: u32, h: &mut dyn PokeToolHooks) -> u32 {
    party_enter(mon);
    let r = party_get(mon, id, val, h);
    party_exit(mon);
    r
}

/// C: `FUN_02017E40`: set a field of a `Pokemon`; skipped if [`party_enter`] failed.
pub fn party_set_value(mon: &mut [u8], id: u32, val: u32, h: &mut dyn PokeToolHooks) {
    if party_enter(mon) == 1 {
        party_put(mon, id, val, h);
    }
    party_exit(mon);
}

#[cfg(test)]
pub(crate) mod tests;
