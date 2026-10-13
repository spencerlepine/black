//! The tail of F1 row 10.23 (overlay 10, `0x021643AC..0x02164768`): zone-change hooks (field_encount_st, flag
//! 0x964, the daily hidden-item respawn), the field-menu entry checks and event constructors (ROM tables
//! `0x021714DC` / `0x02171504`, called from `event_shortcut_menu.c`), two GMEVENT procs and the hidden-item flags.
//!
//! F6a-spec specced all 19 functions (`tools/rom-diff/specs/black-field/eventdata_tail.spec`); F6a ported them.
//! Every callee outside the area (game data / field map getters, the event-work flag and marker accessors,
//! field_encount_st, the party predicates, the GMEVENT system, the overlay-20 / -21 event constructors and field
//! objects, the random source) is reached through [`FieldEnv::call`] by address; the ROM tables' in-area entries
//! are dispatched to the Rust functions. HIDDEN ITEMS (ROM `0x02171534`, 0xB1 records of 10 bytes, embedded as
//! [`HIDDEN_ITEMS`]): `u16 +0` = event flag - 900, `u8 +3` nonzero = respawns; a background event of kind 2 with
//! script id `8000 + i` is hidden item `i`.

use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported and verified (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_43AC,
    0x0216_43E8,
    0x0216_447C,
    0x0216_4480,
    0x0216_44C4,
    0x0216_44F4,
    0x0216_4508,
    0x0216_4564,
    0x0216_458C,
    0x0216_459C,
    0x0216_45C0,
    0x0216_45E0,
    0x0216_4654,
    0x0216_4678,
    0x0216_46A0,
    0x0216_46C4,
    0x0216_46E8,
    0x0216_4708,
    0x0216_4728,
];

/// First script id of the hidden items.
pub const HIDDEN_ITEM_SCRIPT_BASE: u32 = 8000;
/// Last script id that [`hidden_item_index`] accepts (`0x225F`, 8799).
pub const HIDDEN_ITEM_SCRIPT_LAST: u32 = 0x225F;
/// Offset of the hidden-item event flags (`flag = record +0 + 900`).
pub const HIDDEN_ITEM_FLAG_BASE: u16 = 900;
/// Event flag cleared on every zone change (`FUN_021643AC` / `FUN_021643E8`).
pub const ZONE_CHANGE_FLAG: u32 = 0x964;
/// Number of field-menu entries checked by [`menu_checks`].
pub const MENU_ENTRIES: u32 = 10;

const GET_GAMEDATA: u32 = 0x0201_2EBC; // GAMESYS -> GAMEDATA
const ENCOUNT_WORK: u32 = 0x0200_D418; // (gamedata) -> encount work
const FIELDMAP_ZONE: u32 = 0x0218_8C9C; // fieldmap zone id
const ENCOUNT_ZONE: u32 = 0x0215_E428; // field_encount_st (work, zone / loc)
const ENCOUNT_3A0: u32 = 0x0215_E3A0; // field_encount_st (gsys)
const ENCOUNT_364: u32 = 0x0215_E364; // field_encount_st (gsys)
const ENCOUNT_33C: u32 = 0x0215_E33C; // field_encount_st (fieldmap)
const EVENT_WORK: u32 = 0x0201_296C; // GAMESYS / GAMEDATA -> EVENT WORK
const CLEAR_FLAG: u32 = 0x0201_433C; // event_work clear_flag (ew, flag)
const FIELD_STATUS: u32 = 0x0201_2B38; // GAMEDATA -> field_status
const LOC_FLAGS: u32 = 0x0201_3F6C; // LOCATION / zone flags
const FIELD_STATUS_SET: u32 = 0x0201_4464; // field_status set (fs, 0)
const ZONE_PRED: u32 = 0x0201_3FFC; // zone predicate
const SOUND_OBJ: u32 = 0x0200_8854; // (gamedata) -> object
const SOUND_SET: u32 = 0x0200_8988; // (object, 0)
const TAKE_DAY_MARKER: u32 = 0x0201_440C; // event_work: read and clear the marker byte +0x3E8
const RANDOM_BELOW: u32 = 0x0200_56EC; // random below n
const GSYS_GAMEDATA: u32 = 0x0201_20F4; // GAMESYS -> GAMEDATA
const GAMEDATA_7A4: u32 = 0x0201_27A4; // GAMEDATA getter
const MENU_ENTRY_SET: u32 = 0x0202_8748; // menu entry set (menu, index, enabled)
const GAMEDATA_BYTE: u32 = 0x0201_2B28; // GAMEDATA byte getter
const GAMEDATA_PARTY: u32 = 0x0201_2AC8; // GAMEDATA -> party
const FIELDMAP_8C: u32 = 0x0218_8CB4; // fieldmap +0x8C
const PLAYER_ATTR: u32 = 0x021A_30C8; // field player -> attribute
const ATTR_VALUE: u32 = 0x021A_B0F0; // attribute value of an attribute
const ATTR_PRED: u32 = 0x021A_B4B8; // map attribute predicate
const PARTY_C20: u32 = 0x0201_3C20; // party predicates
const PARTY_C50: u32 = 0x0201_3C50;
const PARTY_D20: u32 = 0x0201_3D20;
const PARTY_E04: u32 = 0x0201_3E04;
const CHECK_5C64: u32 = 0x021C_5C64; // (a, b, 0)
const GMEVENT_CREATE: u32 = 0x0201_22C0; // GMEVENT create (gsys, parent, proc, work size)
const GMEVENT_WORK: u32 = 0x0201_24E8; // GMEVENT work (event)
const MAPCHANGE_EVENT: u32 = 0x0218_5140; // event_mapchange (ov20): event (gsys, zone, &pos, 0)
const GSYS_FIELDMAP: u32 = 0x0201_210C; // GAMESYS -> FIELDMAP
const GSYS_2138: u32 = 0x0201_2138; // GAMESYS getter
const SCRIPT_PRED: u32 = 0x0215_870C; // script_sys predicate
const FIELDMAP_C7C: u32 = 0x0218_8C7C; // fieldmap getter
const FIELDMAP_BF8: u32 = 0x0218_8BF8; // fieldmap (fieldmap)
const STEP_2F54: u32 = 0x021A_2F54; // (object) step
const STEP_34F8: u32 = 0x021A_34F8; // (object) step
const FIELDMAP_CCC: u32 = 0x0218_8CCC; // fieldmap getter
const OBJ_MODE_SET: u32 = 0x021A_0F30; // (object, mode)
const OBJ_MODE: u32 = 0x021A_0F90; // (object) -> mode

/// The GMEVENT proc of [`event_a_create`] (`FUN_021645E0`, Thumb).
pub const EVENT_A_PROC: u32 = 0x0216_45E1;
/// The GMEVENT proc of [`event_b_create`] (`FUN_021646A0`, Thumb).
pub const EVENT_B_PROC: u32 = 0x0216_46A1;

/// ROM `0x021714DC`: the field-menu entry checks of [`menu_check`] (0 = always enabled; Thumb addresses).
pub const MENU_CHECKS: [u32; 10] = [
    0x0216_4509,
    0,
    0,
    0x0216_4565,
    0x0216_458D,
    0x0216_459D,
    0x0216_45C1,
    0,
    0,
    0x0216_45C1,
];

/// ROM `0x02171504`: the field-menu event constructors of [`menu_event`] (Thumb address, second word; the second
/// word is not read here).
pub const MENU_EVENTS: [(u32, u32); 6] = [
    (0x0216_4655, 0),
    (0x0216_4679, 0xFF),
    (0x0218_4E95, 3),
    (0x021C_6069, 0xFF),
    (0x021C_5D11, 5),
    (0x0216_46C5, 9),
];

/// One record of the hidden-item table (ROM `0x02171534`, 10 bytes). Only `flag` and `respawn` are read by this
/// area; the other fields are kept as they are in the ROM (meaning not checked).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HiddenItem {
    /// `u16 +0`: event flag - 900.
    pub flag: u16,
    /// `u8 +2` (unread here).
    pub unk_02: u8,
    /// `u8 +3`: nonzero = the item respawns ([`hidden_items_respawn`]).
    pub respawn: u8,
    /// `u16 +4` (unread here).
    pub unk_04: u16,
    /// `u16 +6` (unread here).
    pub unk_06: u16,
    /// `u16 +8` (unread here).
    pub unk_08: u16,
}

/// ROM `0x02171534`: the 0xB1 hidden-item records.
pub const HIDDEN_ITEMS: [HiddenItem; 0xB1] = [
    HiddenItem {
        flag: 0,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x6,
        unk_06: 0x2E8,
        unk_08: 0x243,
    },
    HiddenItem {
        flag: 1,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x6,
        unk_06: 0x2FA,
        unk_08: 0x246,
    },
    HiddenItem {
        flag: 2,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x98,
        unk_06: 0x13,
        unk_08: 0x2D,
    },
    HiddenItem {
        flag: 3,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x98,
        unk_06: 0x29,
        unk_08: 0x10,
    },
    HiddenItem {
        flag: 4,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x141,
        unk_06: 0x2B2,
        unk_08: 0x242,
    },
    HiddenItem {
        flag: 5,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x10,
        unk_06: 0x261,
        unk_08: 0x259,
    },
    HiddenItem {
        flag: 6,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x99,
        unk_06: 0x18,
        unk_08: 0x8,
    },
    HiddenItem {
        flag: 7,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x99,
        unk_06: 0x8,
        unk_08: 0x17,
    },
    HiddenItem {
        flag: 8,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x9A,
        unk_06: 0x274,
        unk_08: 0x26C,
    },
    HiddenItem {
        flag: 9,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x121,
        unk_06: 0x196,
        unk_08: 0x1D4,
    },
    HiddenItem {
        flag: 10,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x122,
        unk_06: 0xBB,
        unk_08: 0x19D,
    },
    HiddenItem {
        flag: 11,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x123,
        unk_06: 0x6C,
        unk_08: 0x131,
    },
    HiddenItem {
        flag: 12,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x124,
        unk_06: 0xB8,
        unk_08: 0xC4,
    },
    HiddenItem {
        flag: 13,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x125,
        unk_06: 0x1A9,
        unk_08: 0xAD,
    },
    HiddenItem {
        flag: 14,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x13B,
        unk_06: 0x294,
        unk_08: 0xB9,
    },
    HiddenItem {
        flag: 15,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x13C,
        unk_06: 0x2F8,
        unk_08: 0x12C,
    },
    HiddenItem {
        flag: 16,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x126,
        unk_06: 0x2B,
        unk_08: 0x25,
    },
    HiddenItem {
        flag: 17,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x127,
        unk_06: 0x2B,
        unk_08: 0x25,
    },
    HiddenItem {
        flag: 18,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x121,
        unk_06: 0x1AD,
        unk_08: 0x1D0,
    },
    HiddenItem {
        flag: 19,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x122,
        unk_06: 0xB2,
        unk_08: 0x1AE,
    },
    HiddenItem {
        flag: 20,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x123,
        unk_06: 0x64,
        unk_08: 0x128,
    },
    HiddenItem {
        flag: 21,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x124,
        unk_06: 0xAE,
        unk_08: 0xCE,
    },
    HiddenItem {
        flag: 22,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x125,
        unk_06: 0x188,
        unk_08: 0xAD,
    },
    HiddenItem {
        flag: 23,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x125,
        unk_06: 0x1B7,
        unk_08: 0xAD,
    },
    HiddenItem {
        flag: 24,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x13B,
        unk_06: 0x28E,
        unk_08: 0xBD,
    },
    HiddenItem {
        flag: 25,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x13C,
        unk_06: 0x2EC,
        unk_08: 0x12C,
    },
    HiddenItem {
        flag: 26,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x126,
        unk_06: 0x30,
        unk_08: 0x24,
    },
    HiddenItem {
        flag: 27,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x127,
        unk_06: 0x30,
        unk_08: 0x24,
    },
    HiddenItem {
        flag: 28,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x130,
        unk_06: 0x172,
        unk_08: 0x1B5,
    },
    HiddenItem {
        flag: 29,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x131,
        unk_06: 0x9D,
        unk_08: 0x194,
    },
    HiddenItem {
        flag: 30,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x132,
        unk_06: 0x78,
        unk_08: 0x119,
    },
    HiddenItem {
        flag: 31,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x133,
        unk_06: 0xD2,
        unk_08: 0xBB,
    },
    HiddenItem {
        flag: 32,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x134,
        unk_06: 0x15D,
        unk_08: 0xAD,
    },
    HiddenItem {
        flag: 33,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x135,
        unk_06: 0x1D2,
        unk_08: 0xAD,
    },
    HiddenItem {
        flag: 34,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x136,
        unk_06: 0x250,
        unk_08: 0xAE,
    },
    HiddenItem {
        flag: 35,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x137,
        unk_06: 0x2AA,
        unk_08: 0xAF,
    },
    HiddenItem {
        flag: 36,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x138,
        unk_06: 0x2E8,
        unk_08: 0x14A,
    },
    HiddenItem {
        flag: 37,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x139,
        unk_06: 0x275,
        unk_08: 0x1AA,
    },
    HiddenItem {
        flag: 38,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x139,
        unk_06: 0x251,
        unk_08: 0x1B2,
    },
    HiddenItem {
        flag: 39,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x13A,
        unk_06: 0x1CA,
        unk_08: 0x1B4,
    },
    HiddenItem {
        flag: 40,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x10,
        unk_06: 0x278,
        unk_08: 0x248,
    },
    HiddenItem {
        flag: 41,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x9A,
        unk_06: 0x25B,
        unk_08: 0x246,
    },
    HiddenItem {
        flag: 42,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x9A,
        unk_06: 0x255,
        unk_08: 0x27B,
    },
    HiddenItem {
        flag: 43,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x9A,
        unk_06: 0x24E,
        unk_08: 0x26B,
    },
    HiddenItem {
        flag: 44,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x9B,
        unk_06: 0x58,
        unk_08: 0x37,
    },
    HiddenItem {
        flag: 45,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x9B,
        unk_06: 0xE,
        unk_08: 0x38,
    },
    HiddenItem {
        flag: 46,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x146,
        unk_06: 0x19E,
        unk_08: 0x239,
    },
    HiddenItem {
        flag: 47,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x146,
        unk_06: 0x1B3,
        unk_08: 0x216,
    },
    HiddenItem {
        flag: 48,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x9E,
        unk_06: 0x23,
        unk_08: 0x1C,
    },
    HiddenItem {
        flag: 49,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x9E,
        unk_06: 0x18,
        unk_08: 0x38,
    },
    HiddenItem {
        flag: 50,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x9E,
        unk_06: 0x51,
        unk_08: 0x1A,
    },
    HiddenItem {
        flag: 51,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xBB,
        unk_06: 0xE,
        unk_08: 0x11,
    },
    HiddenItem {
        flag: 52,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x40,
        unk_06: 0x3C,
        unk_08: 0x19,
    },
    HiddenItem {
        flag: 53,
        unk_02: 1,
        respawn: 0,
        unk_04: 0xBF,
        unk_06: 0xCA,
        unk_08: 0x1D5,
    },
    HiddenItem {
        flag: 54,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x14B,
        unk_06: 0x80,
        unk_08: 0x16A,
    },
    HiddenItem {
        flag: 55,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x14B,
        unk_06: 0x83,
        unk_08: 0x16A,
    },
    HiddenItem {
        flag: 56,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x14B,
        unk_06: 0x96,
        unk_08: 0x169,
    },
    HiddenItem {
        flag: 57,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x163,
        unk_06: 0x1B1,
        unk_08: 0x70,
    },
    HiddenItem {
        flag: 58,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x14E,
        unk_06: 0x13,
        unk_08: 0x1B,
    },
    HiddenItem {
        flag: 59,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x14F,
        unk_06: 0x1C,
        unk_08: 0x7,
    },
    HiddenItem {
        flag: 60,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC3,
        unk_06: 0x19,
        unk_08: 0x36,
    },
    HiddenItem {
        flag: 61,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC3,
        unk_06: 0x28,
        unk_08: 0xB,
    },
    HiddenItem {
        flag: 62,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC4,
        unk_06: 0x38,
        unk_08: 0x2A,
    },
    HiddenItem {
        flag: 63,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC5,
        unk_06: 0xA,
        unk_08: 0x21,
    },
    HiddenItem {
        flag: 64,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x6B,
        unk_06: 0x48,
        unk_08: 0x151,
    },
    HiddenItem {
        flag: 65,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x6B,
        unk_06: 0x56,
        unk_08: 0x153,
    },
    HiddenItem {
        flag: 66,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x6B,
        unk_06: 0x52,
        unk_08: 0x159,
    },
    HiddenItem {
        flag: 67,
        unk_02: 1,
        respawn: 0,
        unk_04: 0xC6,
        unk_06: 0x96,
        unk_08: 0xD5,
    },
    HiddenItem {
        flag: 68,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC7,
        unk_06: 0x1B,
        unk_08: 0xB,
    },
    HiddenItem {
        flag: 69,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC7,
        unk_06: 0x27,
        unk_08: 0xA,
    },
    HiddenItem {
        flag: 70,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC7,
        unk_06: 0x2F,
        unk_08: 0x21,
    },
    HiddenItem {
        flag: 71,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC8,
        unk_06: 0x2B,
        unk_08: 0x37,
    },
    HiddenItem {
        flag: 72,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC8,
        unk_06: 0x2C,
        unk_08: 0xA,
    },
    HiddenItem {
        flag: 73,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC9,
        unk_06: 0x3C,
        unk_08: 0x3B,
    },
    HiddenItem {
        flag: 74,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC9,
        unk_06: 0xD,
        unk_08: 0x2F,
    },
    HiddenItem {
        flag: 75,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC9,
        unk_06: 0x3,
        unk_08: 0x3B,
    },
    HiddenItem {
        flag: 76,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xCA,
        unk_06: 0x10,
        unk_08: 0x12,
    },
    HiddenItem {
        flag: 77,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x71,
        unk_06: 0xBF,
        unk_08: 0xCE,
    },
    HiddenItem {
        flag: 78,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x71,
        unk_06: 0xAD,
        unk_08: 0xBE,
    },
    HiddenItem {
        flag: 79,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xCE,
        unk_06: 0x21,
        unk_08: 0x18,
    },
    HiddenItem {
        flag: 80,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x159,
        unk_06: 0xCB,
        unk_08: 0xB7,
    },
    HiddenItem {
        flag: 81,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x15C,
        unk_06: 0x156,
        unk_08: 0xB5,
    },
    HiddenItem {
        flag: 82,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x15C,
        unk_06: 0x167,
        unk_08: 0xAE,
    },
    HiddenItem {
        flag: 83,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x78,
        unk_06: 0x1AC,
        unk_08: 0xB3,
    },
    HiddenItem {
        flag: 84,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x78,
        unk_06: 0x1A3,
        unk_08: 0x94,
    },
    HiddenItem {
        flag: 85,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x161,
        unk_06: 0xA,
        unk_08: 0x19,
    },
    HiddenItem {
        flag: 86,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x161,
        unk_06: 0x5,
        unk_08: 0xF,
    },
    HiddenItem {
        flag: 87,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x162,
        unk_06: 0x3A,
        unk_08: 0x11,
    },
    HiddenItem {
        flag: 88,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x162,
        unk_06: 0x6,
        unk_08: 0x29,
    },
    HiddenItem {
        flag: 89,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xD9,
        unk_06: 0x7,
        unk_08: 0xB,
    },
    HiddenItem {
        flag: 90,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xDB,
        unk_06: 0x3,
        unk_08: 0x9,
    },
    HiddenItem {
        flag: 91,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xDE,
        unk_06: 0x10,
        unk_08: 0xA,
    },
    HiddenItem {
        flag: 92,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xDE,
        unk_06: 0x19,
        unk_08: 0x12,
    },
    HiddenItem {
        flag: 93,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xE0,
        unk_06: 0x6,
        unk_08: 0x9,
    },
    HiddenItem {
        flag: 94,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xE3,
        unk_06: 0x14,
        unk_08: 0x18,
    },
    HiddenItem {
        flag: 95,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xE3,
        unk_06: 0x10,
        unk_08: 0xC,
    },
    HiddenItem {
        flag: 96,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x16D,
        unk_06: 0x1CF,
        unk_08: 0xB3,
    },
    HiddenItem {
        flag: 97,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x170,
        unk_06: 0x26E,
        unk_08: 0xAE,
    },
    HiddenItem {
        flag: 98,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x170,
        unk_06: 0x25B,
        unk_08: 0xA5,
    },
    HiddenItem {
        flag: 99,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x170,
        unk_06: 0x26D,
        unk_08: 0xA8,
    },
    HiddenItem {
        flag: 100,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x170,
        unk_06: 0x274,
        unk_08: 0xAE,
    },
    HiddenItem {
        flag: 101,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x196,
        unk_06: 0x286,
        unk_08: 0xAC,
    },
    HiddenItem {
        flag: 102,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x196,
        unk_06: 0x298,
        unk_08: 0xA9,
    },
    HiddenItem {
        flag: 103,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x172,
        unk_06: 0x2E5,
        unk_08: 0x107,
    },
    HiddenItem {
        flag: 104,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x172,
        unk_06: 0x2E9,
        unk_08: 0x10B,
    },
    HiddenItem {
        flag: 105,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x172,
        unk_06: 0x2E1,
        unk_08: 0x109,
    },
    HiddenItem {
        flag: 106,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x172,
        unk_06: 0x2CC,
        unk_08: 0xF1,
    },
    HiddenItem {
        flag: 107,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x172,
        unk_06: 0x2CC,
        unk_08: 0xF8,
    },
    HiddenItem {
        flag: 108,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x172,
        unk_06: 0x2AC,
        unk_08: 0xE0,
    },
    HiddenItem {
        flag: 109,
        unk_02: 1,
        respawn: 1,
        unk_04: 0xF0,
        unk_06: 0x304,
        unk_08: 0x126,
    },
    HiddenItem {
        flag: 110,
        unk_02: 1,
        respawn: 1,
        unk_04: 0xF0,
        unk_06: 0x305,
        unk_08: 0x12B,
    },
    HiddenItem {
        flag: 111,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x178,
        unk_06: 0xE,
        unk_08: 0xC,
    },
    HiddenItem {
        flag: 112,
        unk_02: 1,
        respawn: 0,
        unk_04: 0xF0,
        unk_06: 0x318,
        unk_08: 0x110,
    },
    HiddenItem {
        flag: 113,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x178,
        unk_06: 0x1B,
        unk_08: 0x2B,
    },
    HiddenItem {
        flag: 114,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x178,
        unk_06: 0x20,
        unk_08: 0xB,
    },
    HiddenItem {
        flag: 115,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x17A,
        unk_06: 0x257,
        unk_08: 0x1A4,
    },
    HiddenItem {
        flag: 116,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x1A7,
        unk_06: 0x2BF,
        unk_08: 0x2C5,
    },
    HiddenItem {
        flag: 117,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x1A7,
        unk_06: 0x2AB,
        unk_08: 0x2D4,
    },
    HiddenItem {
        flag: 118,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x183,
        unk_06: 0x282,
        unk_08: 0x2E7,
    },
    HiddenItem {
        flag: 119,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x183,
        unk_06: 0x28B,
        unk_08: 0x2D5,
    },
    HiddenItem {
        flag: 120,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x183,
        unk_06: 0x28C,
        unk_08: 0x2D9,
    },
    HiddenItem {
        flag: 121,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x1A7,
        unk_06: 0x2A1,
        unk_08: 0x2D6,
    },
    HiddenItem {
        flag: 122,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x145,
        unk_06: 0x16,
        unk_08: 0x4,
    },
    HiddenItem {
        flag: 123,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x13D,
        unk_06: 0x2F8,
        unk_08: 0x2F4,
    },
    HiddenItem {
        flag: 124,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x71,
        unk_06: 0xC2,
        unk_08: 0xDF,
    },
    HiddenItem {
        flag: 125,
        unk_02: 1,
        respawn: 1,
        unk_04: 0x71,
        unk_06: 0xC5,
        unk_08: 0xDF,
    },
    HiddenItem {
        flag: 126,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x71,
        unk_06: 0xC7,
        unk_08: 0xDF,
    },
    HiddenItem {
        flag: 127,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x99,
        unk_06: 0x8,
        unk_08: 0xC,
    },
    HiddenItem {
        flag: 128,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xFF,
        unk_06: 0x20,
        unk_08: 0x2D,
    },
    HiddenItem {
        flag: 129,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x9E,
        unk_06: 0x2C,
        unk_08: 0x56,
    },
    HiddenItem {
        flag: 130,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x162,
        unk_06: 0x6,
        unk_08: 0x36,
    },
    HiddenItem {
        flag: 131,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x162,
        unk_06: 0x33,
        unk_08: 0x37,
    },
    HiddenItem {
        flag: 132,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x162,
        unk_06: 0x1A,
        unk_08: 0x29,
    },
    HiddenItem {
        flag: 133,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x178,
        unk_06: 0x20,
        unk_08: 0x20,
    },
    HiddenItem {
        flag: 134,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x17A,
        unk_06: 0x25F,
        unk_08: 0x1B9,
    },
    HiddenItem {
        flag: 135,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x1A7,
        unk_06: 0x2C1,
        unk_08: 0x2E2,
    },
    HiddenItem {
        flag: 136,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x145,
        unk_06: 0xC,
        unk_08: 0x1A,
    },
    HiddenItem {
        flag: 137,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x15A,
        unk_06: 0x11,
        unk_08: 0x33,
    },
    HiddenItem {
        flag: 138,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x15A,
        unk_06: 0x2A,
        unk_08: 0x28,
    },
    HiddenItem {
        flag: 139,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x15A,
        unk_06: 0x26,
        unk_08: 0x1C,
    },
    HiddenItem {
        flag: 140,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x15A,
        unk_06: 0x14,
        unk_08: 0xC,
    },
    HiddenItem {
        flag: 141,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x1A2,
        unk_06: 0x2F,
        unk_08: 0x10,
    },
    HiddenItem {
        flag: 142,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xEB,
        unk_06: 0x122,
        unk_08: 0x2F6,
    },
    HiddenItem {
        flag: 143,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xE7,
        unk_06: 0xF,
        unk_08: 0x3A,
    },
    HiddenItem {
        flag: 144,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xE7,
        unk_06: 0x2,
        unk_08: 0x7,
    },
    HiddenItem {
        flag: 145,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xE7,
        unk_06: 0x18,
        unk_08: 0x5,
    },
    HiddenItem {
        flag: 146,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x16D,
        unk_06: 0x1D1,
        unk_08: 0xA8,
    },
    HiddenItem {
        flag: 147,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x176,
        unk_06: 0x2F5,
        unk_08: 0x150,
    },
    HiddenItem {
        flag: 148,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC3,
        unk_06: 0x1C,
        unk_08: 0xA,
    },
    HiddenItem {
        flag: 149,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC4,
        unk_06: 0x1F,
        unk_08: 0x2C,
    },
    HiddenItem {
        flag: 150,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xC5,
        unk_06: 0x19,
        unk_08: 0x1E,
    },
    HiddenItem {
        flag: 151,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xA1,
        unk_06: 0xA,
        unk_08: 0xA,
    },
    HiddenItem {
        flag: 152,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xA4,
        unk_06: 0x1A,
        unk_08: 0x1A,
    },
    HiddenItem {
        flag: 153,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x14D,
        unk_06: 0x3,
        unk_08: 0x11,
    },
    HiddenItem {
        flag: 154,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x14D,
        unk_06: 0x14,
        unk_08: 0x3,
    },
    HiddenItem {
        flag: 155,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x14F,
        unk_06: 0x7,
        unk_08: 0x9,
    },
    HiddenItem {
        flag: 156,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x172,
        unk_06: 0x284,
        unk_08: 0xD7,
    },
    HiddenItem {
        flag: 157,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x69,
        unk_06: 0x7,
        unk_08: 0xD,
    },
    HiddenItem {
        flag: 158,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x146,
        unk_06: 0x18E,
        unk_08: 0x228,
    },
    HiddenItem {
        flag: 159,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x146,
        unk_06: 0x192,
        unk_08: 0x256,
    },
    HiddenItem {
        flag: 160,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x178,
        unk_06: 0x2B,
        unk_08: 0x1A,
    },
    HiddenItem {
        flag: 161,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x144,
        unk_06: 0x14,
        unk_08: 0xB,
    },
    HiddenItem {
        flag: 162,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x144,
        unk_06: 0x17,
        unk_08: 0xC,
    },
    HiddenItem {
        flag: 163,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x145,
        unk_06: 0x19,
        unk_08: 0x29,
    },
    HiddenItem {
        flag: 164,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x145,
        unk_06: 0x17,
        unk_08: 0x37,
    },
    HiddenItem {
        flag: 165,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x14E,
        unk_06: 0x7,
        unk_08: 0x1C,
    },
    HiddenItem {
        flag: 166,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x14E,
        unk_06: 0x5,
        unk_08: 0x9,
    },
    HiddenItem {
        flag: 167,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x159,
        unk_06: 0xE0,
        unk_08: 0xA6,
    },
    HiddenItem {
        flag: 168,
        unk_02: 1,
        respawn: 0,
        unk_04: 0xBF,
        unk_06: 0xB0,
        unk_08: 0x1CF,
    },
    HiddenItem {
        flag: 169,
        unk_02: 1,
        respawn: 0,
        unk_04: 0xBF,
        unk_06: 0xCA,
        unk_08: 0x1C8,
    },
    HiddenItem {
        flag: 170,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x181,
        unk_06: 0x17,
        unk_08: 0x21,
    },
    HiddenItem {
        flag: 171,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x71,
        unk_06: 0xDC,
        unk_08: 0xC3,
    },
    HiddenItem {
        flag: 172,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x71,
        unk_06: 0xDE,
        unk_08: 0xC4,
    },
    HiddenItem {
        flag: 173,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xFF,
        unk_06: 0x53,
        unk_08: 0xD,
    },
    HiddenItem {
        flag: 174,
        unk_02: 0,
        respawn: 0,
        unk_04: 0xFF,
        unk_06: 0x55,
        unk_08: 0x8,
    },
    HiddenItem {
        flag: 175,
        unk_02: 0,
        respawn: 0,
        unk_04: 0x9B,
        unk_06: 0x23,
        unk_08: 0x50,
    },
    HiddenItem {
        flag: 176,
        unk_02: 1,
        respawn: 0,
        unk_04: 0x146,
        unk_06: 0x196,
        unk_08: 0x220,
    },
];

/// C: `FUN_021643AC` (unlabeled routine at `0x021643AC`): zone-change hook: `FUN_0215E428(FUN_0200D418(game data),
/// fieldmap zone FUN_02188C9C(fieldmap))`, `FUN_0215E3A0(gsys)` (field_encount_st), clear event flag 0x964,
/// `FUN_0215E33C(fieldmap)`.
pub fn zone_enter<E: FieldEnv>(env: &mut E, gsys: u32, fieldmap: u32) {
    let gd = env.callv(GET_GAMEDATA, &[gsys]);
    let w = env.callv(ENCOUNT_WORK, &[gd]);
    let z = env.callv(FIELDMAP_ZONE, &[fieldmap]);
    env.callv(ENCOUNT_ZONE, &[w, z]);
    env.callv(ENCOUNT_3A0, &[gsys]);
    let ew = env.callv(EVENT_WORK, &[gsys]);
    env.callv(CLEAR_FLAG, &[ew, ZONE_CHANGE_FLAG]);
    env.callv(ENCOUNT_33C, &[fieldmap]);
}

/// C: `FUN_021643E8` (unlabeled routine at `0x021643E8`): zone-change hook for the LOCATION `loc`: when
/// `FUN_02013F6C(loc) & 3 == 0`, `FUN_02014464(field status, 0)`; `FUN_0215E428(encount work, loc)`,
/// `FUN_0215E364(gsys)`, clear event flag 0x964, [`hidden_items_respawn`]; when `FUN_02013FFC(loc) == 0`,
/// `FUN_02008988(FUN_02008854(game data), 0)`.
pub fn zone_change<E: FieldEnv>(env: &mut E, gsys: u32, loc: u32) {
    let fs = env.callv(FIELD_STATUS, &[gsys]);
    if env.callv(LOC_FLAGS, &[loc]) & 3 == 0 {
        env.callv(FIELD_STATUS_SET, &[fs, 0]);
    }
    let gd = env.callv(GET_GAMEDATA, &[gsys]);
    let w = env.callv(ENCOUNT_WORK, &[gd]);
    env.callv(ENCOUNT_ZONE, &[w, loc]);
    env.callv(ENCOUNT_364, &[gsys]);
    let ew = env.callv(EVENT_WORK, &[gsys]);
    env.callv(CLEAR_FLAG, &[ew, ZONE_CHANGE_FLAG]);
    hidden_items_respawn(env, ew);
    if env.callv(ZONE_PRED, &[loc]) == 0 {
        let gd = env.callv(GET_GAMEDATA, &[gsys]);
        let o = env.callv(SOUND_OBJ, &[gd]);
        env.callv(SOUND_SET, &[o, 0]);
    }
}

/// C: `FUN_0216447C` (unlabeled routine at `0x0216447C`): does nothing.
pub fn nothing() {}

/// C: `FUN_02164480` (unlabeled routine at `0x02164480`): fill the field-menu record `menu` (0x18 bytes, cleared
/// first): entry `i` of 10 enabled by [`menu_check`] (`FUN_02028748(menu, i, enabled)`), `+4` =
/// `FUN_02012B28(FUN_020127A4(game data))`.
pub fn menu_checks<E: FieldEnv>(env: &mut E, menu: u32, gsys: u32, x: u32) {
    let gd = env.callv(GSYS_GAMEDATA, &[gsys]);
    let p = env.callv(GAMEDATA_7A4, &[gd]);
    env.mem().fill(menu, 0x18, 0);
    for i in 0..MENU_ENTRIES {
        let e = menu_check(env, i, gsys, x);
        env.callv(MENU_ENTRY_SET, &[menu, i, e]);
    }
    let b = env.callv(GAMEDATA_BYTE, &[p]);
    env.w32(menu.wrapping_add(4), b);
}

/// C: `FUN_021644C4` (unlabeled routine at `0x021644C4`): is field-menu entry `i` enabled: the check `MENU_CHECKS[i]`
/// called as `f(game data, x, FUN_020127A4(game data))`, or 1 when the entry has none. UNVERIFIED: `i >= 10` reads
/// past the ROM table in the original; the port panics.
pub fn menu_check<E: FieldEnv>(env: &mut E, i: u32, gsys: u32, x: u32) -> u32 {
    let gd = env.callv(GSYS_GAMEDATA, &[gsys]);
    let p = env.callv(GAMEDATA_7A4, &[gd]);
    let f = MENU_CHECKS[i as usize];
    if f == 0 {
        return 1;
    }
    match f & !1 {
        0x0216_4508 => menu_check_0(env, gd, x, p),
        0x0216_4564 => menu_check_3(env, gd, x, p),
        0x0216_458C => menu_check_4(env, gd, x, p),
        0x0216_459C => menu_check_5(env, gd, x, p),
        0x0216_45C0 => menu_check_6(env, gd, x, p),
        a => env.callv(a, &[gd, x, p]),
    }
}

/// C: `FUN_021644F4` (unlabeled routine at `0x021644F4`): the field-menu event `i`: `MENU_EVENTS[i]`'s constructor
/// called as `f(b, a)`; returns its result (a GMEVENT). The in-area constructors are [`event_a_create`],
/// [`mapchange_event`] and [`event_b_create`]; the others live in overlays 20 / 21. UNVERIFIED: `i >= 6` reads
/// past the ROM table in the original; the port panics.
pub fn menu_event<E: FieldEnv>(env: &mut E, i: u32, a: u32, b: u32) -> u32 {
    let (f, _) = MENU_EVENTS[i as usize];
    match f & !1 {
        0x0216_4654 => event_a_create(env, b, a),
        0x0216_4678 => mapchange_event(env, b, a),
        0x0216_46C4 => event_b_create(env, b, a),
        t => env.callv(t, &[b, a]),
    }
}

/// C: `FUN_02164508` (unlabeled routine at `0x02164508`): menu entry 0: with `q` the party (`FUN_02012AC8(p)`),
/// `b` the byte `FUN_02012B28(p)` and `r` the attribute predicate `FUN_021AB4B8(FUN_021AB0F0(FUN_021A30C8(
/// FUN_02188CB4(fieldmap))))`: `b == 1`: `r == 0`; else `FUN_02013C20(q) != 0 && b == 0 && r == 0`.
pub fn menu_check_0<E: FieldEnv>(env: &mut E, _gd: u32, fieldmap: u32, p: u32) -> u32 {
    let q = env.callv(GAMEDATA_PARTY, &[p]);
    let b = env.callv(GAMEDATA_BYTE, &[p]) & 0xFF;
    let o = env.callv(FIELDMAP_8C, &[fieldmap]);
    let a = env.callv(PLAYER_ATTR, &[o]);
    let v = env.callv(ATTR_VALUE, &[a]);
    let r = env.callv(ATTR_PRED, &[v]);
    if b == 1 {
        return (r == 0) as u32;
    }
    if env.callv(PARTY_C20, &[q]) == 0 || b != 0 {
        return 0;
    }
    (r == 0) as u32
}

/// C: `FUN_02164564` (unlabeled routine at `0x02164564`): menu entry 3: `FUN_02013E04(party) != 0 &&
/// FUN_02013C50(party) != 0`.
pub fn menu_check_3<E: FieldEnv>(env: &mut E, _gd: u32, _x: u32, p: u32) -> u32 {
    let q = env.callv(GAMEDATA_PARTY, &[p]);
    if env.callv(PARTY_E04, &[q]) == 0 {
        return 0;
    }
    (env.callv(PARTY_C50, &[q]) != 0) as u32
}

/// C: `FUN_0216458C` (unlabeled routine at `0x0216458C`): menu entry 4: `FUN_02013E04(party)`.
pub fn menu_check_4<E: FieldEnv>(env: &mut E, _gd: u32, _x: u32, p: u32) -> u32 {
    let q = env.callv(GAMEDATA_PARTY, &[p]);
    env.callv(PARTY_E04, &[q])
}

/// C: `FUN_0216459C` (unlabeled routine at `0x0216459C`): menu entry 5: 0 when `FUN_02013E04(party) == 0`, else
/// `FUN_021C5C64(gd, x, 0)`.
pub fn menu_check_5<E: FieldEnv>(env: &mut E, gd: u32, x: u32, p: u32) -> u32 {
    let q = env.callv(GAMEDATA_PARTY, &[p]);
    if env.callv(PARTY_E04, &[q]) == 0 {
        return 0;
    }
    env.callv(CHECK_5C64, &[gd, x, 0])
}

/// C: `FUN_021645C0` (unlabeled routine at `0x021645C0`): menu entries 6 and 9: 0 when `FUN_02013D20(party) != 0`,
/// else `FUN_02013E04(party)`.
pub fn menu_check_6<E: FieldEnv>(env: &mut E, _gd: u32, _x: u32, p: u32) -> u32 {
    let q = env.callv(GAMEDATA_PARTY, &[p]);
    if env.callv(PARTY_D20, &[q]) != 0 {
        return 0;
    }
    env.callv(PARTY_E04, &[q])
}

/// C: `FUN_021645E0` (unlabeled routine at `0x021645E0`, GMEVENT proc [`EVENT_A_PROC`]): with `f` the field map of
/// `work[0]` (`FUN_0201210C`; `FUN_02188C7C(f)` first): step 0: `work[1] += 1`; while
/// `FUN_0215870C(FUN_02012138(work[0])) == 1` and `work[1] < 30` it waits (0), else `FUN_02188BF8(f)` and step 1
/// at once; step 1: `FUN_021A2F54(FUN_02188CB4(f)) == 1` -> step 2; step 2: done (1) when
/// `FUN_021A34F8(FUN_02188CB4(f)) == 1`. Returns 0 while not done.
pub fn event_a_proc<E: FieldEnv>(env: &mut E, _ev: u32, seq: u32, work: u32) -> u32 {
    let g = env.r32(work);
    let f = env.callv(GSYS_FIELDMAP, &[g]);
    env.callv(FIELDMAP_C7C, &[f]);
    match env.r32(seq) {
        0 => {
            let g = env.r32(work);
            let s = env.callv(GSYS_2138, &[g]);
            let n = env.r32(work.wrapping_add(4)).wrapping_add(1);
            env.w32(work.wrapping_add(4), n);
            if env.callv(SCRIPT_PRED, &[s]) == 1 && env.r32(work.wrapping_add(4)) < 30 {
                return 0;
            }
            env.callv(FIELDMAP_BF8, &[f]);
            let s = env.r32(seq).wrapping_add(1);
            env.w32(seq, s);
        }
        1 => {}
        2 => {
            let o = env.callv(FIELDMAP_8C, &[f]);
            return (env.callv(STEP_34F8, &[o]) == 1) as u32;
        }
        _ => return 0,
    }
    let o = env.callv(FIELDMAP_8C, &[f]);
    if env.callv(STEP_2F54, &[o]) == 1 {
        let s = env.r32(seq).wrapping_add(1);
        env.w32(seq, s);
    }
    0
}

/// C: `FUN_02164654` (unlabeled routine at `0x02164654`): create the GMEVENT of [`event_a_proc`]
/// (`FUN_020122C0(gsys, 0, proc, 8)`), work `[gsys, 0]`; returns it.
pub fn event_a_create<E: FieldEnv>(env: &mut E, _x: u32, gsys: u32) -> u32 {
    let ev = env.callv(GMEVENT_CREATE, &[gsys, 0, EVENT_A_PROC, 8]);
    let w = env.callv(GMEVENT_WORK, &[ev]);
    env.w32(w, gsys);
    env.w32(w.wrapping_add(4), 0);
    ev
}

/// C: `FUN_02164678` (unlabeled routine at `0x02164678`): the map-change event (overlay 20 `FUN_02185140(gsys, zone
/// 0x117, &(0x200000, 0x20, 0x1E8000), 0)`); returns it.
pub fn mapchange_event<E: FieldEnv>(env: &mut E, _x: u32, gsys: u32) -> u32 {
    let mut pos = [0u8; 12];
    for (i, v) in [0x0020_0000u32, 0x20, 0x001E_8000].into_iter().enumerate() {
        pos[4 * i..4 * i + 4].copy_from_slice(&v.to_le_bytes());
    }
    env.call(
        MAPCHANGE_EVENT,
        &mut [CArg::V(gsys), CArg::V(0x117), CArg::InBuf(&pos), CArg::V(0)],
    )
}

/// C: `FUN_021646A0` (unlabeled routine at `0x021646A0`, GMEVENT proc [`EVENT_B_PROC`]): toggle the mode of the
/// field object `FUN_02188CCC(work[1])` between 6 and 0 (`FUN_021A0F90` / `FUN_021A0F30`); done at once (1).
pub fn event_b_proc<E: FieldEnv>(env: &mut E, _ev: u32, _seq: u32, work: u32) -> u32 {
    let a = env.r32(work.wrapping_add(4));
    let o = env.callv(FIELDMAP_CCC, &[a]);
    let m = if env.callv(OBJ_MODE, &[o]) == 6 { 0 } else { 6 };
    env.callv(OBJ_MODE_SET, &[o, m]);
    1
}

/// C: `FUN_021646C4` (unlabeled routine at `0x021646C4`): create the GMEVENT of [`event_b_proc`]
/// (`FUN_020122C0(gsys, 0, proc, 8)`), work `[gsys, a]`; returns it.
pub fn event_b_create<E: FieldEnv>(env: &mut E, a: u32, gsys: u32) -> u32 {
    let ev = env.callv(GMEVENT_CREATE, &[gsys, 0, EVENT_B_PROC, 8]);
    let w = env.callv(GMEVENT_WORK, &[ev]);
    env.w32(w, gsys);
    env.w32(w.wrapping_add(4), a);
    ev
}

/// C: `FUN_021646E8` (unlabeled routine at `0x021646E8`): the hidden-item index of a script id, `id - 8000` for
/// `8000 <= id <= 0x225F` (unsigned), else 0 (the same as item 0).
pub fn hidden_item_index(id: u32) -> u16 {
    if !(HIDDEN_ITEM_SCRIPT_BASE..=HIDDEN_ITEM_SCRIPT_LAST).contains(&id) {
        return 0;
    }
    (id - HIDDEN_ITEM_SCRIPT_BASE) as u16
}

/// C: `FUN_02164708` (unlabeled routine at `0x02164708`): the event flag of the hidden item of a script id,
/// `HIDDEN_ITEMS[hidden_item_index(id)].flag + 900` (halfword). [`hidden_item_index`] accepts ids up to 8799 but the
/// table has 0xB1 records (scripts 8000..=8176, the only hidden-item scripts in the ROM): for ids 8177..8799 the
/// original reads the ROM past the table (overlay 10 data and overlay 11 code) and the port panics; the spec (Z22)
/// draws no such id.
pub fn hidden_item_flag(id: u32) -> u16 {
    let i = hidden_item_index(id) as usize;
    HIDDEN_ITEMS[i].flag.wrapping_add(HIDDEN_ITEM_FLAG_BASE)
}

/// C: `FUN_02164728` (unlabeled routine at `0x02164728`): once a day (`FUN_0201440C(ew)` reads and clears the
/// marker): every hidden item that respawns (`+3 != 0`) has its flag cleared when `FUN_020056EC(100) < 20`.
pub fn hidden_items_respawn<E: FieldEnv>(env: &mut E, ew: u32) {
    if env.callv(TAKE_DAY_MARKER, &[ew]) == 0 {
        return;
    }
    for r in HIDDEN_ITEMS.iter() {
        if r.respawn != 0 && env.callv(RANDOM_BELOW, &[100]) < 20 {
            let f = r.flag.wrapping_add(HIDDEN_ITEM_FLAG_BASE) as u32;
            env.callv(CLEAR_FLAG, &[ew, f]);
        }
    }
}

#[cfg(test)]
mod tests;
