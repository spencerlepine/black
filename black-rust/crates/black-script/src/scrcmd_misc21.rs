//! F8e: the overlay-21 script commands without a `__FILE__` anchor: F1-SPLIT rows 21.91 (Thumb 0x021CF7DC-0x021D0140) and
//! 21.97 (0x021D1670-0x021D30B8): distinct-OT counting, the 20-record save block 0x02009470, an OBJ animation proc, field
//! events, lottery / reward numbers, the camera shake of command 328 and the overlay-0x021F3xxx calls (35 command ids, 81
//! functions; 11 render / UI-flow functions are left out, see the spec header).
//!
//! Every logic function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_misc21.spec`, ported here, listed
//! in [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_misc21`. [`COMMANDS`] maps the command ids to their
//! handlers, [`dispatch`] runs one by address, [`call_event`] the GMEVENT functions, [`call_init_proc`] the init proc of
//! `script_sys::INIT_PROCS` slot 8 and [`call_field_proc`] the OBJ-animation field proc.
//!
//! Callees outside the area and the render / UI functions left out are [`Misc21Hooks`] methods named by address (vm.c, the
//! operand readers, the ctx getters and the GMEVENT / child-event calls come from [`CoreHooks`]). Roles are UNVERIFIED (named by
//! address). ROM data the functions read is embedded and compared with the build by the tests.

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_F7DC,
    0x021C_F844,
    0x021C_F8E4,
    0x021C_F940,
    0x021C_F9AC,
    0x021C_FA04,
    0x021C_FA1C,
    0x021C_FA58,
    0x021C_FA88,
    0x021C_FAC0,
    0x021C_FB1C,
    0x021C_FB68,
    0x021C_FBA8,
    0x021C_FC14,
    0x021C_FC4C,
    0x021C_FC80,
    0x021C_FCF4,
    0x021C_FD08,
    0x021C_FD88,
    0x021C_FDBC,
    0x021C_FEA0,
    0x021C_FEF0,
    0x021C_FF28,
    0x021C_FF78,
    0x021C_FFDC,
    0x021D_0014,
    0x021D_004C,
    0x021D_0084,
    0x021D_00B8,
    0x021D_0114,
    0x021D_1670,
    0x021D_16D8,
    0x021D_179C,
    0x021D_182C,
    0x021D_1884,
    0x021D_1894,
    0x021D_18E0,
    0x021D_18F4,
    0x021D_1984,
    0x021D_1988,
    0x021D_1A04,
    0x021D_1A54,
    0x021D_1A74,
    0x021D_1A9C,
    0x021D_1AB0,
    0x021D_1AEC,
    0x021D_1B2C,
    0x021D_1B8C,
    0x021D_1BD8,
    0x021D_1C24,
    0x021D_1F48,
    0x021D_1F50,
    0x021D_1F64,
    0x021D_213C,
    0x021D_21C0,
    0x021D_21E0,
    0x021D_220C,
    0x021D_2278,
    0x021D_2358,
    0x021D_2418,
    0x021D_2440,
    0x021D_2468,
    0x021D_2524,
    0x021D_26D0,
    0x021D_2710,
    0x021D_2750,
    0x021D_297C,
    0x021D_29A8,
    0x021D_2ADC,
    0x021D_2B18,
    0x021D_2B54,
    0x021D_2CE8,
    0x021D_2D24,
    0x021D_2D50,
    0x021D_2D7C,
    0x021D_2EDC,
    0x021D_2F44,
    0x021D_2F6C,
    0x021D_3018,
    0x021D_3068,
    0x021D_3090,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (233, 0x021C_FA88),
    (309, 0x021C_FA58),
    (347, 0x021D_1A04),
    (348, 0x021D_1A54),
    (349, 0x021D_1A74),
    (350, 0x021D_1AB0),
    (412, 0x021D_1B2C),
    (413, 0x021D_1B8C),
    (472, 0x021C_FEA0),
    (473, 0x021C_FF28),
    (474, 0x021C_FF78),
    (475, 0x021C_FFDC),
    (476, 0x021D_0084),
    (479, 0x021D_00B8),
    (481, 0x021C_FEF0),
    (482, 0x021D_0014),
    (483, 0x021D_004C),
    (521, 0x021D_1670),
    (522, 0x021D_16D8),
    (523, 0x021D_179C),
    (524, 0x021D_213C),
    (525, 0x021D_2468),
    (551, 0x021D_2524),
    (553, 0x021D_2710),
    (569, 0x021C_F7DC),
    (570, 0x021D_2ADC),
    (571, 0x021D_2B18),
    (573, 0x021D_2B54),
    (574, 0x021D_2EDC),
    (575, 0x021D_2F44),
    (582, 0x021D_0114),
    (597, 0x021D_2F6C),
    (598, 0x021D_3018),
    (599, 0x021D_3068),
    (600, 0x021D_3090),
];

/// GMEVENT functions of this area (Thumb): command 569, the slot event, command 350, commands 412 / 413, the show-mon event
/// (UI flow, not ported), the two warp events, command 551, the 0x021D29A8 event and the camera shake.
pub const EVENT_569: u32 = 0x021C_F845;
pub const EVENT_SLOT: u32 = 0x021C_FBA9;
pub const EVENT_350: u32 = 0x021D_1A9D;
pub const EVENT_412: u32 = 0x021D_1AED;
pub const EVENT_SHOW_MON: u32 = 0x021D_1C8D;
pub const EVENT_2278: u32 = 0x021D_2279;
pub const EVENT_2358: u32 = 0x021D_2359;
pub const EVENT_551: u32 = 0x021D_26D1;
pub const EVENT_29A8: u32 = 0x021D_29A9;
pub const EVENT_SHAKE: u32 = 0x021D_2D7D;
/// Init proc of `script_sys::INIT_PROCS` slot 8 ([`init_proc`], Thumb).
pub const INIT_PROC: u32 = 0x021D_1BD9;
/// The field-proc table of the OBJ animation (0x021DD5F0, overlay 21 ROM: 4 words, then the step / end callbacks).
pub const ANIM_PROC_TABLE: u32 = 0x021D_D5F0;
pub const ANIM_PROC_STEP: u32 = 0x021D_18F5;
pub const ANIM_PROC_NOP: u32 = 0x021D_1985;

/// Capacity of the id list of command 569 ({u32 count, u32 ids[0x32]}).
pub const OT_LIST_MAX: u32 = 0x32;
/// Matches kept by `FUN_021CFC80` (the record indices at R+8..).
pub const MATCH_MAX: u32 = 5;
/// The record list of `FUN_021CFAC0` / `FUN_021CFB1C` / `FUN_021CFB68`: 20 slots of 0x14 bytes from L + 0x10.
pub const SLOT_COUNT: u32 = 20;
pub const SLOT_SIZE: u32 = 0x14;
/// Script of the slot event (`FUN_021587B0(event, 0xAFC, 0, 0x15)`).
pub const SCRIPT_SLOT: u32 = 0xAFC;
/// PokeParam / box-data ids read here.
pub const PP_SPECIES: u32 = 5;
pub const PP_OT_ID: u32 = 7;
pub const PP_EGG: u32 = 0x4C;
pub const BOX_PRESENT: u32 = 0xA9;
/// Proc id / table of command 309 (`FUN_021BB990`).
pub const PROC_309: (u32, u32) = (0x39, 0x021F_3900);
/// The tag of an empty record of `FUN_02009390` (commands 522 / 523).
pub const RECORD_EMPTY: u16 = 0x7E;
/// The flag-set pairs of command 524 (k = 1 / 2).
pub const FLAGS_524: [(u32, u32); 2] = [(0x5C0, 0x67B), (0x664, 0x6B7)];
/// The script of `FUN_021D220C` for counts above 500.
pub const SCRIPT_OVER_500: u32 = 0x28E1;
/// SE of the warp events.
pub const SE_2278: u32 = 0x63C;
/// Zone of the grotto check's second table set, its heap id.
pub const ZONE_1A8: u32 = 0x1A8;
pub const HEAP_GROTTO: u32 = 0x8015;

/// Animation rotation (fx32 degrees) -> OBJ direction (`FUN_0216D60C`), `FUN_021D1988`.
pub const ROT_DIRS: [(u32, u32); 4] = [(0, 1), (0x5A000, 3), (0xB4000, 0), (0x10E000, 2)];
/// The ROM position records `{s16 x, s16 y, s16 z, u16 zone}` the slot list refers to (0x021DB834, overlay 21 ROM; 18
/// records).
pub const POSITIONS_ADDR: u32 = 0x021D_B834;
pub const POSITIONS: [(i16, i16, i16, u16); 18] = [
    (0x1A1, 1, 0x1D5, 0x3E),
    (0xAD, 0, 0x19C, 0x60),
    (0x4E, 0, 0x11B, 0x6B),
    (0xB0, 3, 0xBB, 0x71),
    (0x1AF, 0, 0x88, 0x78),
    (0x286, 0, 0xAA, 0x196),
    (0x2C7, -3, 0x138, 0x19C),
    (0x160, 0, 0x1B0, 0x149),
    (0x67, 0, 0x16E, 0x14B),
    (0x6B, 0, 0xE9, 0x151),
    (0xD2, 0, 0xBB, 0x159),
    (0x172, 0, 0xA9, 0x15C),
    (0x1E7, 0, 0xAD, 0x16D),
    (0x264, 0, 0xB7, 0x170),
    (0x2B0, 0, 0xDC, 0x172),
    (0x2EB, -4, 0x148, 0x176),
    (0x259, 5, 0x1AA, 0x17A),
    (0x1E2, 0, 0x1B8, 0x17F),
];
/// Per animation kind (W+0x199, 0..7): a nonzero word sets the OBJ flag 0x8000 when the animation ends (0x021DD5E0, overlay
/// 21 ROM; words 4..7 are the start of the proc table 0x021DD5F0).
pub const ANIM_FLAGS_ADDR: u32 = 0x021D_D5E0;
pub const ANIM_FLAGS: [u32; 8] = [0, 0, 0, 1, 0, 0x1A0, 0, 0];
/// The modes of command 412 by operand (0x021DD608, overlay 21 ROM; the 4th word is the first key of [`TABLE_DD614`]).
pub const TABLE_412_ADDR: u32 = 0x021D_D608;
pub const TABLE_412: [u32; 4] = [7, 0x0048_0080, 0, 0x68A];
/// The 21 `{u32 key, u32 flag}` records at 0x021DD614 (overlay 21 ROM).
pub const TABLE_DD614_ADDR: u32 = 0x021D_D614;
pub const TABLE_DD614: [(u32, u32); 21] = [
    (0x68A, 1),
    (0x690, 1),
    (0x691, 1),
    (0x692, 0),
    (0x693, 0),
    (0x6D5, 0),
    (0x6D6, 0),
    (0x6D7, 0),
    (0x6D8, 0),
    (0x6E3, 1),
    (0x6E4, 1),
    (0x71E, 0),
    (0x71F, 0),
    (0x720, 1),
    (0x743, 0),
    (0x746, 1),
    (0x756, 1),
    (0x76E, 0),
    (0x7EF, 1),
    (0x77E, 0),
    (0x77F, 0),
];
/// The keys of [`TABLE_DD614`] (kept for [`in_table_a`]'s callers).
pub const TABLE_DD614_KEYS: [u32; 21] = [
    0x68A, 0x690, 0x691, 0x692, 0x693, 0x6D5, 0x6D6, 0x6D7, 0x6D8, 0x6E3, 0x6E4, 0x71E, 0x71F,
    0x720, 0x743, 0x746, 0x756, 0x76E, 0x7EF, 0x77E, 0x77F,
];
/// The `{u16 count, u16 script}` pairs of `FUN_021D220C` (0x021DD6BC, overlay 21 ROM).
pub const COUNT_SCRIPTS_ADDR: u32 = 0x021D_D6BC;
pub const COUNT_SCRIPTS: [(u16, u16); 5] = [
    (0, 0x28DD),
    (0x64, 0x28DE),
    (0x12C, 0x28DF),
    (0x1C2, 0x28E0),
    (0x1F4, 0x28E1),
];
/// The 31 u16 lottery values of command 525 (0x021DD6D0, overlay 21 ROM).
pub const LOTTERY_ADDR: u32 = 0x021D_D6D0;
pub const LOTTERY: [u16; 31] = [
    0x6A1, 0x5D6, 0x699, 0x5FE, 0x687, 0x5A3, 0x624, 0x6A3, 0x61A, 0x6B5, 0x60B, 0x602, 0x6A0,
    0x57D, 0x249, 0x20A, 0x257, 0x24B, 0x253, 0x208, 0x1FF, 0x213, 0x233, 0x210, 0x22B, 0x252,
    0x206, 0x245, 0x26A, 0x2C5, 0x62B,
];
/// The 8 numbers [`adjust_a`] re-rolls (0x021DD718, overlay 21 ROM).
pub const ADJUST_A_ADDR: u32 = 0x021D_D718;
pub const ADJUST_A: [u32; 8] = [0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D];
/// The 15 numbers [`adjust_b`] re-rolls (0x021DD738, overlay 21 ROM).
pub const ADJUST_B_ADDR: u32 = 0x021D_D738;
pub const ADJUST_B: [u32; 15] = [
    0x87, 0x88, 0x89, 0x8A, 0x8B, 0x8C, 0x8D, 0x8E, 0x8F, 0x90, 0x91, 0x92, 0x93, 0x94, 0x95,
];
/// The 29 reward rows {a, b, na, nb} of command 573 (0x021DD774, overlay 21 ROM; -1 = no counter).
pub const REWARD_ROWS_ADDR: u32 = 0x021D_D774;
pub const REWARD_ROWS: [[u32; 4]; 29] = [
    [0xFFFF_FFFF, 0xFFFF_FFFF, 0, 0],
    [0x1E, 0xFFFF_FFFF, 6, 0],
    [1, 0xFFFF_FFFF, 9, 0],
    [0xB, 0xFFFF_FFFF, 9, 0],
    [0x15, 0x16, 9, 9],
    [0x17, 0xFFFF_FFFF, 9, 0],
    [7, 0xFFFF_FFFF, 9, 0],
    [5, 6, 9, 9],
    [0xA, 0xFFFF_FFFF, 9, 0],
    [8, 0x50, 6, 4],
    [4, 0x47, 9, 4],
    [3, 0xFFFF_FFFF, 9, 0],
    [9, 0xFFFF_FFFF, 6, 0],
    [0x18, 0xFFFF_FFFF, 9, 0],
    [0x1C, 0xFFFF_FFFF, 6, 0],
    [0x1D, 0xFFFF_FFFF, 6, 0],
    [0x1F, 0xFFFF_FFFF, 6, 0],
    [0x48, 0xFFFF_FFFF, 4, 0],
    [0x49, 0xFFFF_FFFF, 4, 0],
    [0x4A, 0xFFFF_FFFF, 4, 0],
    [0x4B, 0xFFFF_FFFF, 4, 0],
    [0x4E, 0xFFFF_FFFF, 4, 0],
    [0x4F, 0xFFFF_FFFF, 4, 0],
    [0x59, 0xFFFF_FFFF, 4, 0],
    [0x5A, 0xFFFF_FFFF, 4, 0],
    [0x51, 0xFFFF_FFFF, 4, 0],
    [0x53, 0xFFFF_FFFF, 4, 0],
    [0x55, 0xFFFF_FFFF, 4, 0],
    [0x57, 0xFFFF_FFFF, 4, 0],
];

/// Callees of the scrcmd_misc21 functions outside the area and the render / UI functions of the area that are left out (D3),
/// named by address (meaning per the spec; UNVERIFIED unless stated). vm.c, the operand readers, the ctx getters and the
/// GMEVENT / child-event calls come from [`CoreHooks`].
#[allow(clippy::too_many_arguments)]
pub trait Misc21Hooks: CoreHooks {
    // ---- script_sys / scrcmd
    /// C: `FUN_0215A5CC(ctx)` (script_sys): the heap id of the script.
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A5EC(ctx)` (script_sys): the field OBJ system.
    fn fun_0215a5ec(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02158768(gamesys, script, x, heap)` (script_sys): a script event.
    fn fun_02158768(&mut self, gamesys: u32, script: u32, x: u32, heap: u32) -> u32;
    /// C: `FUN_021587B0(event, script, x, heap)` (script_sys): start a script in the event; its SW.
    fn fun_021587b0(&mut self, event: u32, script: u32, x: u32, heap: u32) -> u32;
    /// C: `FUN_02158F58(sw)` (script_sys): a record whose word +4 is the field map.
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F74(sw)` (script_sys): the word set.
    fn fun_02158f74(&mut self, sw: u32) -> u32;
    /// C: `FUN_02159058(sw, a, b, c, d)` (script_sys).
    fn fun_02159058(&mut self, sw: u32, a: u32, b: u32, c: u32, d: u32);
    /// C: `FUN_02159104(sw, w)` (script_sys): set the SW's OBJ-animation slot.
    fn fun_02159104(&mut self, sw: u32, w: u32);
    /// C: `FUN_0215910C(sw)` (script_sys): the SW's OBJ-animation slot.
    fn fun_0215910c(&mut self, sw: u32) -> u32;
    /// C: `FUN_02159144(flags)` (overlay 10).
    fn fun_02159144(&mut self, flags: u32);
    /// C: `FUN_02159168(flags)` (overlay 10).
    fn fun_02159168(&mut self, flags: u32);
    // ---- main
    /// C: `FUN_020124E4(event)`: the event's GAMESYS.
    fn fun_020124e4(&mut self, event: u32) -> u32;
    /// C: `FUN_02012374(event, child)`: call a child event.
    fn fun_02012374(&mut self, event: u32, child: u32);
    /// C: `FUN_0201210C(gamesys)`: the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012844(gamedata)`: the event data.
    fn fun_02012844(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201293C(gamedata)`: the box data.
    fn fun_0201293c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012934(gamedata)`: the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)`: the player status.
    fn fun_02012944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)`: the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F08(gamedata)`.
    fn fun_02012f08(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F2C(gamedata)`: the record counters.
    fn fun_02012f2c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F38(gamedata)`: the match record R (byte 0 = the language).
    fn fun_02012f38(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F8C(gamedata)`: the 5-record object Z.
    fn fun_02012f8c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02013C60(zone)`: a zone flag test.
    fn fun_02013c60(&mut self, zone: u32) -> u32;
    /// C: `FUN_0201AA18(party)`: the party count.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)`: party mon `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_02017E1C(mon, id, x)`: a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_02017E88(boxmon, id, x)`: a box-mon value.
    fn fun_02017e88(&mut self, boxmon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_020079BC(box, b, s, id, x)`: a value of the box mon in box `b`, slot `s`.
    fn fun_020079bc(&mut self, bx: u32, b: u32, s: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_02007B44(save)`.
    fn fun_02007b44(&mut self, save: u32) -> u32;
    /// C: `FUN_0200BAC8(save)`: the daycare save block (UNVERIFIED).
    fn fun_0200bac8(&mut self, save: u32) -> u32;
    /// C: `FUN_0200BBEC(block, k, i)`: daycare record `i` of set `k`.
    fn fun_0200bbec(&mut self, block: u32, k: u32, i: u32) -> u32;
    /// C: `FUN_0200BE48(save)`: the slot-list save block L.
    fn fun_0200be48(&mut self, save: u32) -> u32;
    /// C: `FUN_0200C0F0(save)`.
    fn fun_0200c0f0(&mut self, save: u32) -> u32;
    /// C: `FUN_0200C234(x)`.
    fn fun_0200c234(&mut self, x: u32) -> u32;
    /// C: `FUN_0200E878()` (called with r0 = the 02012F38 record left over).
    fn fun_0200e878(&mut self, r0: u32);
    /// C: `FUN_02008530(status)`: the player's OT id.
    fn fun_02008530(&mut self, status: u32) -> u32;
    /// C: `FUN_02008550(status)`.
    fn fun_02008550(&mut self, status: u32) -> u32;
    /// C: `FUN_02008554(record)`.
    fn fun_02008554(&mut self, rec: u32) -> u32;
    /// C: `FUN_0200856C(status)`.
    fn fun_0200856c(&mut self, status: u32) -> u32;
    /// C: `FUN_02008570(record)`.
    fn fun_02008570(&mut self, rec: u32) -> u32;
    /// C: `FUN_02008854(save)`.
    fn fun_02008854(&mut self, save: u32) -> u32;
    /// C: `FUN_02008990(x)`: a save counter.
    fn fun_02008990(&mut self, x: u32) -> u32;
    /// C: `FUN_02008B08(save)`.
    fn fun_02008b08(&mut self, save: u32) -> u32;
    /// C: `FUN_02008B94(x)`: the saved month.
    fn fun_02008b94(&mut self, x: u32) -> u32;
    /// C: `FUN_02008B98(x)`: the saved day.
    fn fun_02008b98(&mut self, x: u32) -> u32;
    /// C: `FUN_02008C58(save)`: the record counters.
    fn fun_02008c58(&mut self, save: u32) -> u32;
    /// C: `FUN_02008DAC(counters, id, n)`: add `n` to a counter.
    fn fun_02008dac(&mut self, counters: u32, id: u32, n: u32);
    /// C: `FUN_02008DFC(counters, id)`: a counter.
    fn fun_02008dfc(&mut self, counters: u32, id: u32) -> u32;
    /// C: `FUN_020092D8(z)`.
    fn fun_020092d8(&mut self, z: u32) -> u32;
    /// C: `FUN_020092EC(z)`.
    fn fun_020092ec(&mut self, z: u32) -> u32;
    /// C: `FUN_02009390(z, i)`: record `i` (u16 tag, then a name) or null.
    fn fun_02009390(&mut self, z: u32, i: u32) -> u32;
    /// C: `FUN_020093D8(z)`.
    fn fun_020093d8(&mut self, z: u32) -> u32;
    /// C: `FUN_020093F0(z, a)`.
    fn fun_020093f0(&mut self, z: u32, a: u32);
    /// C: `FUN_02009418(z)`.
    fn fun_02009418(&mut self, z: u32) -> u32;
    /// C: `FUN_02009428(z, x)`.
    fn fun_02009428(&mut self, z: u32, x: u32);
    /// C: `FUN_02009470(save)`: the 20-record save block S.
    fn fun_02009470(&mut self, save: u32) -> u32;
    /// C: `FUN_02009590(s, status, v)`.
    fn fun_02009590(&mut self, s: u32, status: u32, v: u32) -> u32;
    /// C: `FUN_020095C8(s)`.
    fn fun_020095c8(&mut self, s: u32) -> u32;
    /// C: `FUN_020095FC(s, i)`.
    fn fun_020095fc(&mut self, s: u32, i: u32);
    /// C: `FUN_02009618(s, i)`.
    fn fun_02009618(&mut self, s: u32, i: u32);
    /// C: `FUN_02009628(s)`.
    fn fun_02009628(&mut self, s: u32) -> u32;
    /// C: `FUN_02009634(s, i, field)`: a field of record `i`.
    fn fun_02009634(&mut self, s: u32, i: u32, field: u32) -> u32;
    /// C: `FUN_02009698(s, i, field, v)`: set a field of record `i`.
    fn fun_02009698(&mut self, s: u32, i: u32, field: u32, v: u32);
    /// C: `FUN_0200973C(s, i)`: record `i`'s data (or null).
    fn fun_0200973c(&mut self, s: u32, i: u32) -> u32;
    /// C: `FUN_020249B0(flags, i)`: flag `i` of a flag set.
    fn fun_020249b0(&mut self, flags: u32, i: u32) -> u32;
    /// C: `FUN_02025B64(lang)`.
    fn fun_02025b64(&mut self, lang: u32) -> u32;
    /// C: `FUN_02025C74(t)`.
    fn fun_02025c74(&mut self, t: u32) -> u32;
    /// C: `FUN_0201147C(heap, arc, k, x, work, size)`: an OBJ animation.
    fn fun_0201147c(&mut self, heap: u32, arc: u32, k: u32, x: u32, work: u32, size: u32) -> u32;
    /// C: `FUN_02011558(anime)`: free the animation.
    fn fun_02011558(&mut self, anime: u32);
    /// C: `FUN_02011590(anime, step)`: step the animation; nonzero when it has ended.
    fn fun_02011590(&mut self, anime: u32, step: u32) -> u32;
    /// C: `FUN_02011610(anime, &t)`: the animation's translation (VecFx32).
    fn fun_02011610(&mut self, anime: u32, t: &mut [u8; 12]);
    /// C: `FUN_02011638(anime, &r)`: the animation's rotation (VecFx32; r.y = the angle).
    fn fun_02011638(&mut self, anime: u32, r: &mut [u8; 12]);
    /// C: `FUN_0207C9D0(a, b, out)` (VEC_Add): `out = a + b`.
    fn fun_0207c9d0(&mut self, a: &[u8; 12], b: &[u8; 12], out: &mut [u8; 12]);
    /// C: `FUN_0201EE0C(wordset, i, strbuf, a, b, c)`: put a string into the word set.
    fn fun_0201ee0c(&mut self, ws: u32, i: u32, buf: u32, a: u32, b: u32, c: u32);
    /// C: `FUN_0201EE60(wordset, i, v)`: a species name.
    fn fun_0201ee60(&mut self, ws: u32, i: u32, v: u32);
    /// C: `FUN_0201EFA4(wordset, i, n, digits, a, b)`: a number.
    fn fun_0201efa4(&mut self, ws: u32, i: u32, n: u32, digits: u32, a: u32, b: u32);
    /// C: `FUN_0201F02C(wordset, i, status)`: a player / record name.
    fn fun_0201f02c(&mut self, ws: u32, i: u32, status: u32);
    /// C: `FUN_0201F048(wordset, i, lang)`.
    fn fun_0201f048(&mut self, ws: u32, i: u32, lang: u32);
    /// C: `FUN_0201F094(wordset, i, lang, v)`.
    fn fun_0201f094(&mut self, ws: u32, i: u32, lang: u32, v: u32);
    /// C: `FUN_0201F198(wordset, i, v)`.
    fn fun_0201f198(&mut self, ws: u32, i: u32, v: u32) -> u32;
    /// C: `FUN_020457B0(n, heap)`: a new string buffer.
    fn fun_020457b0(&mut self, n: u32, heap: u32) -> u32;
    /// C: `FUN_02045808(strbuf)`: free the string buffer.
    fn fun_02045808(&mut self, buf: u32);
    /// C: `FUN_02045994(strbuf, &s, n)`: set the buffer from the stack string `s` (16 bytes logged).
    fn fun_02045994(&mut self, buf: u32, s: &[u8; 16], n: u32);
    /// C: `FUN_02045994(strbuf, src, n)` with a string in memory at `src`.
    fn fun_02045994_at(&mut self, buf: u32, src: u32, n: u32);
    /// C: `FUN_020452E8(window)`: delete the window.
    fn fun_020452e8(&mut self, window: u32) -> u32;
    /// C: `FUN_020056EC(n)` (RNG): a random number below `n`.
    fn fun_020056ec(&mut self, n: u32) -> u32;
    /// C: `FUN_020061E4(se)`: play an SE.
    fn fun_020061e4(&mut self, se: u32);
    /// C: `FUN_02006148(se)`: the SE's player.
    fn fun_02006148(&mut self, se: u32) -> u32;
    /// C: `FUN_02006224(player)`: nonzero while it plays.
    fn fun_02006224(&mut self, player: u32) -> u32;
    /// C: `FUN_020362DC()`: the held keys (A = 1, B = 2).
    fn fun_020362dc(&mut self) -> u32;
    /// C: `FUN_0203F4CC(&t)` (RTC): the time record (16 bytes; t[1] = the minute word).
    fn fun_0203f4cc(&mut self, t: &mut [u8; 16]);
    // ---- overlay 21 and others
    /// C: `FUN_02188C54(fm)`.
    fn fun_02188c54(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C58(fm)`: the field camera object E.
    fn fun_02188c58(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C64(fm)`.
    fn fun_02188c64(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C7C(fm)`: the field OBJ system.
    fn fun_02188c7c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C84(fm)`: the field heap id.
    fn fun_02188c84(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C9C(fm)`: the zone.
    fn fun_02188c9c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CB4(fm)`: the player.
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CCC(fm)`.
    fn fun_02188ccc(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CFC(fm)`: the field-map mode.
    fn fun_02188cfc(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D24(fm)`.
    fn fun_02188d24(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D38(fm)`: the camera-move system T.
    fn fun_02188d38(&mut self, fm: u32) -> u32;
    /// C: `FUN_021BB990(proc, x, table)`: load a field proc.
    fn fun_021bb990(&mut self, proc_id: u32, x: u32, table: u32) -> u32;
    /// C: `FUN_021BBA1C(proc)`: delete a field proc.
    fn fun_021bba1c(&mut self, x: u32);
    /// C: `FUN_021BBA6C(proc)`: the field proc's work.
    fn fun_021bba6c(&mut self, x: u32) -> u32;
    /// C: `FUN_0216D410(obj, n)` (fldmmdl).
    fn fun_0216d410(&mut self, obj: u32, n: u32);
    /// C: `FUN_0216D454(obj)` (fldmmdl): nonzero when done.
    fn fun_0216d454(&mut self, obj: u32) -> u32;
    /// C: `FUN_0216D568(obj, flag)` (fldmmdl): set an OBJ flag.
    fn fun_0216d568(&mut self, obj: u32, flag: u32);
    /// C: `FUN_0216D570(obj, flag)` (fldmmdl): clear an OBJ flag.
    fn fun_0216d570(&mut self, obj: u32, flag: u32);
    /// C: `FUN_0216D5F8(obj)` (fldmmdl): the OBJ's direction.
    fn fun_0216d5f8(&mut self, obj: u32) -> u32;
    /// C: `FUN_0216D60C(mmdl, dir)` (fldmmdl): set the object's direction.
    fn fun_0216d60c(&mut self, mmdl: u32, dir: u32);
    /// C: `FUN_0216D758(obj)` (fldmmdl).
    fn fun_0216d758(&mut self, obj: u32);
    /// C: `FUN_0216D88C(obj, &pos)` (fldmmdl): the object's position.
    fn fun_0216d88c(&mut self, obj: u32, pos: &mut [u8; 12]);
    /// C: `FUN_0216D89C(mmdl, &pos)` (fldmmdl): set the object's position.
    fn fun_0216d89c(&mut self, mmdl: u32, pos: &[u8; 12]);
    /// C: `FUN_0216D9A0(objsys)` (fldmmdl): pause all OBJs.
    fn fun_0216d9a0(&mut self, objsys: u32);
    /// C: `FUN_0216D9E4(objsys)` (fldmmdl): resume all OBJs.
    fn fun_0216d9e4(&mut self, objsys: u32);
    /// C: `FUN_0216DE94(objsys, id)` (fldmmdl): the OBJ with `id` (or null).
    fn fun_0216de94(&mut self, objsys: u32, id: u32) -> u32;
    /// C: `FUN_0216E014(obj, &pos, dir)` (fldmmdl).
    fn fun_0216e014(&mut self, obj: u32, pos: &[u8; 12], dir: u32);
    /// C: `FUN_0216E2E8(objsys, code, &o)` (fldmmdl): the OBJ code's header (16 bytes; o[0xB] / o[0xC] its size).
    fn fun_0216e2e8(&mut self, objsys: u32, code: u32, o: &mut [u8; 0x10]);
    /// C: `FUN_02162740(evdata)`: the event-data OBJ records (0x24 bytes each, or null).
    fn fun_02162740(&mut self, evdata: u32) -> u32;
    /// C: `FUN_02162744(evdata)`: their count.
    fn fun_02162744(&mut self, evdata: u32) -> u32;
    /// C: `FUN_02162890(evdata, zone, &pos)`: nonzero when the spot is taken.
    fn fun_02162890(&mut self, evdata: u32, zone: u32, pos: &[u8; 12]) -> u32;
    /// C: `FUN_021A0ED4(x)`: nonzero when ready.
    fn fun_021a0ed4(&mut self, x: u32) -> u32;
    /// C: `FUN_021A0F30(x, mode)`.
    fn fun_021a0f30(&mut self, x: u32, mode: u32);
    /// C: `FUN_021A0F94(x)`: the state.
    fn fun_021a0f94(&mut self, x: u32) -> u32;
    /// C: `FUN_021A0FA8(x)`.
    fn fun_021a0fa8(&mut self, x: u32);
    /// C: `FUN_021A01D4(s)`.
    fn fun_021a01d4(&mut self, s: u32) -> u32;
    /// C: `FUN_021A01FC(s, a, b, c)`.
    fn fun_021a01fc(&mut self, s: u32, a: u32, b: u32, c: u32);
    /// C: `FUN_021A3038(player, &pos)`: the player position (VecFx32).
    fn fun_021a3038(&mut self, player: u32, pos: &mut [u8; 12]);
    /// C: `FUN_021A3074(player)`: the player's OBJ.
    fn fun_021a3074(&mut self, player: u32) -> u32;
    /// C: `FUN_021A3098(player)`: the player state.
    fn fun_021a3098(&mut self, player: u32) -> u32;
    /// C: `FUN_021A30A4(player, x)`.
    fn fun_021a30a4(&mut self, player: u32, x: u32);
    /// C: `FUN_021A35FC(player, &x, &y, &z)`: the player's grid position (s16 each).
    fn fun_021a35fc(&mut self, player: u32, x: &mut [u8; 2], y: &mut [u8; 2], z: &mut [u8; 2]);
    /// C: `FUN_021A36BC(player, &p)`: the player's position record (8 bytes).
    fn fun_021a36bc(&mut self, player: u32, p: &mut [u8; 8]);
    /// C: `FUN_021C6DE0(t, move, after)`: queue a camera move.
    fn fun_021c6de0(&mut self, t: u32, mv: u32, after: u32);
    /// C: `FUN_021C71D4(fm, a, b)`: a camera move.
    fn fun_021c71d4(&mut self, fm: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_021C7260(fm, a, b)`: a camera move.
    fn fun_021c7260(&mut self, fm: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_021C7858(fm, n, &vec)`: a camera move by the VecFx32 `vec`.
    fn fun_021c7858(&mut self, fm: u32, n: u32, vec: &[u8; 12]) -> u32;
    /// C: `FUN_021CD440(fm, &zone)`: the warp target zone (u16).
    fn fun_021cd440(&mut self, fm: u32, zone: &mut [u8; 2]);
    /// C: `FUN_02184F88(gamesys)`: a warp event.
    fn fun_02184f88(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02184FD4(gamesys, zone)`: a warp event.
    fn fun_02184fd4(&mut self, gamesys: u32, zone: u32) -> u32;
    /// C: `FUN_021867C4(x, heap)`: the grotto table set.
    fn fun_021867c4(&mut self, x: u32, heap: u32) -> u32;
    /// C: `FUN_02186804(t)`: free it.
    fn fun_02186804(&mut self, t: u32);
    /// C: `FUN_02186830(t, k, heap)`: table k of the set.
    fn fun_02186830(&mut self, t: u32, k: u32, heap: u32) -> u32;
    /// C: `FUN_0218683C(u)`: free the table (`r1` / `r2`: the caller's leftover registers, which the spec stub logs).
    fn fun_0218683c(&mut self, u: u32, r1: u32, r2: u32);
    /// C: `FUN_02186844(u, i)`: entry i of the first list ({u8 x, u8 z}).
    fn fun_02186844(&mut self, u: u32, i: u32) -> u32;
    /// C: `FUN_0218684C(u, i)`: entry i of the second list.
    fn fun_0218684c(&mut self, u: u32, i: u32) -> u32;
    /// C: `FUN_0218EC30(E, pos)`: set the camera target.
    fn fun_0218ec30(&mut self, e: u32, pos: u32);
    /// C: `FUN_0218EC54(E)`.
    fn fun_0218ec54(&mut self, e: u32);
    /// C: `FUN_0218EC5C(E)`: the camera target (VecFx32 *).
    fn fun_0218ec5c(&mut self, e: u32) -> u32;
    /// C: `FUN_0218EC90(E, &pos)`: set the camera target.
    fn fun_0218ec90(&mut self, e: u32, pos: &[u8; 12]);
    /// C: `FUN_0218ED04(E, &off)`: set the camera offset.
    fn fun_0218ed04(&mut self, e: u32, off: &[u8; 12]);
    /// C: `FUN_0218F05C(E)`.
    fn fun_0218f05c(&mut self, e: u32);
    /// C: `FUN_0218F0A8(E)`.
    fn fun_0218f0a8(&mut self, e: u32);
    /// C: `FUN_0218F0E8(E)`: nonzero while busy.
    fn fun_0218f0e8(&mut self, e: u32) -> u32;
    /// C: `FUN_0216BF9C(x, v, status)`.
    fn fun_0216bf9c(&mut self, x: u32, v: u32, status: u32) -> u32;
    /// C: `FUN_021690D0(gamedata)`: the month.
    fn fun_021690d0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_021690E0(gamedata)`: the day.
    fn fun_021690e0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_021D25A4(W)` (render, D3): the BMP window of command 551.
    fn fun_021d25a4(&mut self, w: u32);
    /// C: `FUN_021D25E8(W)` (render, D3): clear the window.
    fn fun_021d25e8(&mut self, w: u32);
    /// C: `FUN_021D25F4(W, a, b, c, d, e)` (render, D3): the 3D model of command 551.
    fn fun_021d25f4(&mut self, w: u32, a: u32, b: u32, c: u32, d: u32, e: u32);
    /// C: `FUN_021D26AC(W)` (render, D3): free the model.
    fn fun_021d26ac(&mut self, w: u32);
    /// C: `FUN_021F37B8(gamesys)` (overlay at 0x021F3xxx): an event (or null).
    fn fun_021f37b8(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_021F3970(gamesys, &pos, &vec, a, b)` (overlay at 0x021F3xxx): an event (or null).
    fn fun_021f3970(&mut self, gamesys: u32, pos: &[u8; 12], vec: &[u8; 12], a: u32, b: u32)
        -> u32;
    /// C: `FUN_021F3A44(gamesys, &vec)` (overlay at 0x021F3xxx).
    fn fun_021f3a44(&mut self, gamesys: u32, vec: &[u8; 12]);
    /// C: `FUN_021F3AC0(gamesys)` (overlay at 0x021F3xxx): an event (or null).
    fn fun_021f3ac0(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_021F3ADC(gamesys)` (overlay at 0x021F3xxx): an event (or null).
    fn fun_021f3adc(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_021F3CFC(gamesys, &pos, a, b)` (overlay at 0x021F3xxx): an event (or null).
    fn fun_021f3cfc(&mut self, gamesys: u32, pos: &[u8; 12], a: u32, b: u32) -> u32;
}
/// C: `sub_021cf7dc`: command 569: var; the GMEVENT [`event_569`] with the work W = {own OT id (`FUN_02008530(FUN_02012944(
/// GAMEDATA))`), box 0, var, the id list {count 0, ids[0x32]}} (0xD8 bytes) runs as a child event; 1.
pub fn cmd_569<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let ev = h.fun_020122c0(gs, 0, EVENT_569, 0xD8);
    let st = h.fun_02012944(gd);
    let w = h.fun_020124e8(ev);
    let id = h.fun_02008530(st);
    h.w32(w, id);
    h.w32(w + 0xC, 0);
    h.w32(w + 4, 0);
    h.w32(w + 8, var);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021cf844`: the GMEVENT function of command 569 (event, seq, W): seq 0 counts the party ([`count_party`]), seq 1
/// two boxes per call ([`count_box`]; past box 23 -> seq 2), seq 2 the daycare ([`count_daycare`]) and ends; a full list
/// (0x32 ids) ends too; at the end `*var` = the count, 1; else 0.
pub fn event_569<H: Misc21Hooks>(h: &mut H, event: u32, seq: u32, w: u32) -> u32 {
    let mut done = false;
    let gs = h.fun_020124e4(event);
    let gd = h.fun_020120f4(gs);
    match h.r32(seq) {
        0 => {
            let id = h.r32(w);
            count_party(h, id, gd, w + 0xC);
            let s = h.r32(seq);
            h.w32(seq, s.wrapping_add(1));
        }
        1 => {
            let first = h.r32(w + 4);
            for j in 0..2u32 {
                let id = h.r32(w);
                count_box(h, id, gd, first.wrapping_add(j), w + 0xC);
                let b = h.r32(w + 4).wrapping_add(1);
                h.w32(w + 4, b);
                if b as i32 >= 24 {
                    let s = h.r32(seq);
                    h.w32(seq, s.wrapping_add(1));
                    break;
                }
            }
        }
        2 => {
            let id = h.r32(w);
            count_daycare(h, id, gd, 0, w + 0xC);
            done = true;
        }
        _ => {}
    }
    let n = h.r32(w + 0xC);
    if n as i32 >= OT_LIST_MAX as i32 {
        done = true;
    }
    if done {
        let var = h.r32(w + 8);
        h.w16(var, n as u16);
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_21__021cf8e4`: count_party(id, GD, list): the party mons that are not eggs whose OT id [`add_other`]
/// adds; their number.
pub fn count_party<H: Misc21Hooks>(h: &mut H, id: u32, gd: u32, list: u32) -> u32 {
    let party = h.fun_02012934(gd);
    let n = h.fun_0201aa18(party) as i32;
    let mut c = 0;
    let mut i: i32 = 0;
    while i < n {
        let mon = h.fun_0201ab28(party, i as u32);
        if h.fun_02017e1c(mon, PP_EGG, 0) == 0 {
            let ot = h.fun_02017e1c(mon, PP_OT_ID, 0);
            if add_other(h, id, ot, list) != 0 {
                c += 1;
            }
        }
        i += 1;
    }
    c
}

/// C: `FUN_overlay_d_21__021cf940`: count_box(id, GD, box, list): the 30 slots of `box` that hold a mon (0xA9) that is not an
/// egg and whose OT id [`add_other`] adds; their number.
pub fn count_box<H: Misc21Hooks>(h: &mut H, id: u32, gd: u32, bx: u32, list: u32) -> u32 {
    let boxes = h.fun_0201293c(gd);
    let mut c = 0;
    for s in 0..30 {
        if h.fun_020079bc(boxes, bx, s, BOX_PRESENT, 0) == 0 {
            continue;
        }
        if h.fun_020079bc(boxes, bx, s, PP_EGG, 0) != 0 {
            continue;
        }
        let ot = h.fun_020079bc(boxes, bx, s, PP_OT_ID, 0);
        if add_other(h, id, ot, list) != 0 {
            c += 1;
        }
    }
    c
}

/// C: `FUN_overlay_d_21__021cf9ac`: count_daycare(id, GD, k, list): the 6 daycare records `FUN_0200BBEC(S, k, i)` that hold a
/// mon (`FUN_02017E88(P, 0xA9)`) whose OT id [`add_other`] adds; their number.
pub fn count_daycare<H: Misc21Hooks>(h: &mut H, id: u32, gd: u32, k: u32, list: u32) -> u32 {
    let save = h.fun_02012ebc(gd);
    let s = h.fun_0200bac8(save);
    let mut c = 0;
    for i in 0..6 {
        let p = h.fun_0200bbec(s, k, i);
        if h.fun_02017e88(p, BOX_PRESENT, 0) == 0 {
            continue;
        }
        let ot = h.fun_02017e88(p, PP_OT_ID, 0);
        if add_other(h, id, ot, list) != 0 {
            c += 1;
        }
    }
    c
}

/// C: `FUN_overlay_d_21__021cfa04`: add_other(id, ot, list): another trainer's OT id goes into the list ([`ot_list_add`]);
/// 1 when added, 0 for the own id or when not added.
pub fn add_other<H: crate::hooks::MemEnv>(h: &mut H, id: u32, ot: u32, list: u32) -> u32 {
    if id == ot {
        return 0;
    }
    ot_list_add(h, ot, list) as u32
}

/// C: `FUN_overlay_d_21__021cfa1c`: add `v` to the list {count, ids[0x32]} unless it is full or holds `v` already; 1 when added.
pub fn ot_list_add<H: crate::hooks::MemEnv>(h: &mut H, v: u32, list: u32) -> bool {
    let n = h.r32(list) as i32;
    if n >= OT_LIST_MAX as i32 {
        return false;
    }
    let mut i = 0;
    while i < n {
        if v == h.r32(list + 4 + 4 * i as u32) {
            return false;
        }
        i += 1;
    }
    h.w32(list.wrapping_add(4 + 4 * i as u32), v);
    let n = h.r32(list);
    h.w32(list, n.wrapping_add(1));
    true
}

/// C: `sub_021cfa58`: command 309: `FUN_021BB990(0x39, FUN_02188D24(field map), 0x021F3900)`; 0.
pub fn cmd_309<H: Misc21Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let x = h.fun_02188d24(fm);
    h.fun_021bb990(PROC_309.0, x, PROC_309.1);
    0
}

/// C: `sub_021cfa88`: command 233: `*var = FUN_02008990(FUN_02008854(SAVE))`; 0.
pub fn cmd_233<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let save = h.fun_02012ebc(gd);
    let x = h.fun_02008854(save);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02008990(x);
    h.w16(var, v as u16);
    0
}

/// C: `FUN_overlay_d_21__021cfac0`: find_slot(L, zone, x, y, z): the first of the 20 slots of L (0x14 bytes from L + 0x10) whose
/// position record [`POSITIONS`]`[slot +0x12]` has the zone, x and z and a y within 1 of `y`; an empty slot (u16 +0x10 == 0) or
/// none: 0xFF. Rust panics for a record index past [`POSITIONS`] (the original reads the following bytes).
pub fn find_slot<H: crate::hooks::MemEnv>(
    h: &mut H,
    l: u32,
    zone: u32,
    x: u32,
    y: u32,
    z: u32,
) -> u32 {
    for i in 0..SLOT_COUNT {
        let e = l + 0x10 + SLOT_SIZE * i;
        if h.r16(e + 0x10) == 0 {
            return 0xFF;
        }
        let (rx, ry, rz, rzone) = POSITIONS[h.r8(e + 0x12) as usize];
        if u32::from(rzone) != zone || i32::from(rx) != x as i32 || i32::from(rz) != z as i32 {
            continue;
        }
        let ry = i32::from(ry);
        if ry - 1 > y as i32 || y as i32 > ry + 1 {
            continue;
        }
        return i;
    }
    0xFF
}

/// C: `FUN_overlay_d_21__021cfb1c`: take_slot(L, i, out): the 10 halfwords of slot i go to `out`; the slots past i move down
/// one; the last slot's u16 +0x10 (L + 0x19C) = 0.
pub fn take_slot<H: crate::hooks::MemEnv>(h: &mut H, l: u32, i: u32, out: u32) {
    let src = l.wrapping_add(SLOT_SIZE.wrapping_mul(i)) + 0x10;
    for k in 0..10 {
        let v = h.r16(src + 2 * k);
        h.w16(out + 2 * k, v);
    }
    let mut j = i.wrapping_add(1);
    while (j as i32) < SLOT_COUNT as i32 {
        let s = l + SLOT_SIZE * j + 0x10;
        for k in 0..10 {
            let v = h.r16(s + 2 * k);
            h.w16(s - SLOT_SIZE + 2 * k, v);
        }
        j += 1;
    }
    h.w16(l + 0x19C, 0);
}

/// C: `FUN_overlay_d_21__021cfb68`: next_slot(L, u32 *i): from `*i` (< 20) the next filled slot -> `*i` past it and the address
/// of its [`POSITIONS`] record (ROM 0x021DB834 + 8 idx); none -> 0 (`*i` = 20; unchanged when it was 20 already).
pub fn next_slot<H: crate::hooks::MemEnv>(h: &mut H, l: u32, i: u32) -> u32 {
    if h.r32(i) as i32 >= SLOT_COUNT as i32 {
        return 0;
    }
    loop {
        let k = h.r32(i);
        let e = l.wrapping_add(SLOT_SIZE.wrapping_mul(k));
        if h.r16(e + 0x20) != 0 {
            let idx = u32::from(h.r8(e + 0x22));
            h.w32(i, k.wrapping_add(1));
            return POSITIONS_ADDR + 8 * idx;
        }
        let k = h.r32(i).wrapping_add(1);
        h.w32(i, k);
        if k as i32 >= SLOT_COUNT as i32 {
            return 0;
        }
    }
}

/// C: `sub_021cfba8`: the GMEVENT function of [`take_slot_event`] (event, seq, W = the slot's 10 halfwords): the script 0xAFC
/// (`FUN_021587B0`) runs with `FUN_02159058(X, W.u16[8], 0, 0, 0)` and word-set slot 3 = the 8-character string at W
/// (`FUN_02045994`, `FUN_0201EE0C(L, 3, B, 1, 1, 2)`); 0.
pub fn event_slot<H: Misc21Hooks>(h: &mut H, event: u32, _seq: u32, w: u32) -> u32 {
    let mut t = [0u8; 0x14];
    for k in 0..10 {
        let v = h.r16(w + 2 * k as u32);
        t[2 * k..2 * k + 2].copy_from_slice(&v.to_le_bytes());
    }
    let x = h.fun_021587b0(event, SCRIPT_SLOT, 0, 0x15);
    let id = u32::from(u16::from_le_bytes([t[0x10], t[0x11]]));
    h.fun_02159058(x, id, 0, 0, 0);
    let ws = h.fun_02158f74(x);
    let buf = h.fun_020457b0(8, 0x15);
    let s: [u8; 16] = t[..16].try_into().unwrap();
    h.fun_02045994(buf, &s, 8);
    h.fun_0201ee0c(ws, 3, buf, 1, 1, 2);
    h.fun_02045808(buf);
    0
}

/// C: `FUN_overlay_d_21__021cfc14`: take_slot_event(GAMESYS, -, i): the GMEVENT [`event_slot`] whose work gets slot i of the
/// save block `FUN_0200BE48(SAVE)` ([`take_slot`]); returns the event.
pub fn take_slot_event<H: Misc21Hooks>(h: &mut H, gs: u32, _x: u32, i: u32) -> u32 {
    let ev = h.fun_020122c0(gs, 0, EVENT_SLOT, 0x14);
    let w = h.fun_020124e8(ev);
    let gd = h.fun_020120f4(gs);
    let save = h.fun_02012ebc(gd);
    let l = h.fun_0200be48(save);
    take_slot(h, l, i, w);
    ev
}

/// C: `FUN_overlay_d_21__021cfc4c`: fill_matches_top(GAMESYS, a, b): R = `FUN_02012F38(GAMEDATA)`; a < 0xE9 -> [`fill_matches`]`(
/// FUN_02009470(SAVE), R, a, b)`; else `FUN_0200E878()` (r0 = R left over).
pub fn fill_matches_top<H: Misc21Hooks>(h: &mut H, gs: u32, a: u32, b: u32) {
    let gd = h.fun_020120f4(gs);
    let r = h.fun_02012f38(gd);
    if (a as i32) < 0xE9 {
        let save = h.fun_02012ebc(gd);
        let s = h.fun_02009470(save);
        fill_matches(h, s, r, a, b);
    } else {
        h.fun_0200e878(r);
    }
}

/// C: `FUN_overlay_d_21__021cfc80`: fill_matches(S, R, a, b): R[0] = a, R[2] = b, R[1] = 0; the records i < 20 of S with field
/// 6 set (`FUN_02009634`) whose `FUN_0200856C` is `a` go to the match list (R[8 + n] = i, R[3 + n] = `FUN_02025C74(t)` with t =
/// `FUN_02008554` capped to 0xF / 0xB by `FUN_02008550`), at most 5.
pub fn fill_matches<H: Misc21Hooks>(h: &mut H, s: u32, r: u32, a: u32, b: u32) {
    h.w8(r, a as u8);
    h.w8(r + 2, b as u8);
    h.w8(r + 1, 0);
    for i in 0..20 {
        if h.fun_02009634(s, i, 6) & 0xFF == 0 {
            continue;
        }
        let p = h.fun_0200973c(s, i);
        if p == 0 || h.fun_0200856c(p) != a {
            continue;
        }
        let mut t = h.fun_02008554(p);
        if t > 0xF {
            t = if h.fun_02008550(p) != 0 { 0xF } else { 0xB };
        }
        let n = u32::from(h.r8(r + 1));
        h.w8(r + n + 8, i as u8);
        let v = h.fun_02025c74(t);
        let n = u32::from(h.r8(r + 1));
        h.w8(r + n + 3, v as u8);
        let n = h.r8(r + 1).wrapping_add(1);
        h.w8(r + 1, n);
        if h.r8(r + 1) >= MATCH_MAX as u8 {
            break;
        }
    }
}

/// C: `FUN_overlay_d_21__021cfcf4`: the record index of match `i` (`R[8 + i]` for `i < 5` and `i < R[1]`), else 0xFF.
pub fn match_index<H: crate::hooks::MemEnv>(h: &mut H, r: u32, i: u32) -> u8 {
    if i < MATCH_MAX && i < u32::from(h.r8(r + 1)) {
        h.r8(r + 8 + i)
    } else {
        0xFF
    }
}

/// C: `FUN_overlay_d_21__021cfd08`: match_message(L, GAMESYS, R, i, x): i >= 5 -> 4; else the words of match i
/// ([`match_words`]) and its [`message_id`]`(FUN_02008550(P), FUN_02008530(P), FUN_02009634(S, k, 4), x, the language test
/// FUN_02025B64)`.
pub fn match_message<H: Misc21Hooks>(h: &mut H, l: u32, gs: u32, r: u32, i: u32, x: u32) -> u32 {
    let gd = h.fun_020120f4(gs);
    let save = h.fun_02012ebc(gd);
    let s = h.fun_02009470(save);
    if i as i32 >= MATCH_MAX as i32 {
        return 4;
    }
    let k = u32::from(h.r8(r.wrapping_add(i) + 8));
    match_words(h, l, gd, k);
    let p = h.fun_0200973c(s, k);
    let f = h.fun_02008550(p);
    let m = h.fun_02009634(s, k, 4);
    let o = h.fun_02008530(p);
    let lp = h.fun_02012f38(gd);
    let lang = u32::from(h.r8(lp));
    let g = (h.fun_02025b64(lang) != 0) as u32;
    message_id(f, o, m, x, g)
}

/// C: `FUN_overlay_d_21__021cfd88`: the message id of a record: base 4 (`g` set) or 0x44, + 0xC m for m < 5, + 6 when `f`,
/// and + 1 + o % 5 when `x` is 0.
pub fn message_id(f: u32, o: u32, m: u32, x: u32, g: u32) -> u32 {
    let mut id: u32 = if g != 0 { 4 } else { 0x44 };
    if m < 5 {
        id += 0xC * m;
    }
    if f != 0 {
        id += 6;
    }
    if x == 0 {
        id += 1 + o % 5;
    }
    id
}

/// C: `FUN_overlay_d_21__021cfdbc`: match_words(L, GD, k): the word set of record k: 0 the language, 1 (when
/// `FUN_02025B64(lang)`) `FUN_0201F094(L, 1, lang, FUN_02008570(P))`, 3 the player name, 5 field 0, 6 `FUN_02009628(S)`, 7 the
/// record's name, 8 the number field 3 (1 when 0; 3 digits), 9 field 1, 0xA field 2 (u8).
pub fn match_words<H: Misc21Hooks>(h: &mut H, l: u32, gd: u32, k: u32) {
    let save = h.fun_02012ebc(gd);
    let s = h.fun_02009470(save);
    let lp = h.fun_02012f38(gd);
    let lang = u32::from(h.r8(lp));
    let p = h.fun_0200973c(s, k);
    h.fun_0201f048(l, 0, lang);
    if h.fun_02025b64(lang) != 0 {
        let v = h.fun_02008570(p);
        h.fun_0201f094(l, 1, lang, v);
    }
    let st = h.fun_02012944(gd);
    h.fun_0201f02c(l, 3, st);
    let v = h.fun_02009634(s, k, 0) & 0xFFFF;
    h.fun_0201ee60(l, 5, v);
    let v = h.fun_02009628(s);
    h.fun_0201f198(l, 6, v);
    h.fun_0201f02c(l, 7, p);
    let mut n = h.fun_02009634(s, k, 3);
    if n == 0 {
        n = 1;
    }
    h.fun_0201efa4(l, 8, n, 3, 0, 1);
    let v = h.fun_02009634(s, k, 1) & 0xFFFF;
    h.fun_0201ee60(l, 9, v);
    let v = h.fun_02009634(s, k, 2) & 0xFF;
    h.fun_0201f198(l, 0xA, v);
}

/// The 20-record save block (`FUN_02009470(SAVE)`) of the script's GAMEDATA.
fn record_block<H: Misc21Hooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.fun_02012ebc(gd);
    h.fun_02009470(save)
}

/// C: `sub_021cfea0`: command 472: v, var; `*var = FUN_02009590(S, FUN_02012944(GAMEDATA), v)`; 0.
pub fn cmd_472<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_020120f4(gs);
    let s = record_block(h, gd);
    let st = h.fun_02012944(gd);
    let r = h.fun_02009590(s, st, v);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021cfef0`: command 481: `*var = FUN_020095C8(S)`; 0.
pub fn cmd_481<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_020120f4(gs);
    let s = record_block(h, gd);
    let r = h.fun_020095c8(s);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021cff28`: command 473: k = [`match_index`]`(R, (u8)value)`; k != 0xFF -> `FUN_02009698(S, k, 5, 1)`; 0.
pub fn cmd_473<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let r = h.fun_02012f38(gd);
    let i = h.fun_02159b10(vm, ctx) & 0xFF;
    let k = match_index(h, r, i);
    if k != 0xFF {
        let s = record_block(h, gd);
        h.fun_02009698(s, u32::from(k), 5, 1);
    }
    0
}

/// C: `sub_021cff78`: command 474: i, var; `*var` = 1 when match i exists ([`match_index`]) and its field 5 is set, else 0;
/// 0.
pub fn cmd_474<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let r = h.fun_02012f38(gd);
    let i = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    h.w16(var, 0);
    let k = match_index(h, r, i);
    if k != 0xFF {
        let s = record_block(h, gd);
        if h.fun_02009634(s, u32::from(k), 5) & 0xFF != 0 {
            h.w16(var, 1);
        }
    }
    0
}

/// C: `sub_021cffdc`: command 475: `FUN_020095FC(S, (u8)value)`; 0.
pub fn cmd_475<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let v = h.fun_02159b10(vm, ctx) & 0xFF;
    let gd = h.fun_020120f4(gs);
    let s = record_block(h, gd);
    h.fun_020095fc(s, v);
    0
}

/// C: `sub_021d0014`: command 482: `FUN_02009618(S, (u8)value)`; 0.
pub fn cmd_482<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let v = h.fun_02159b10(vm, ctx) & 0xFF;
    let gd = h.fun_020120f4(gs);
    let s = record_block(h, gd);
    h.fun_02009618(s, v);
    0
}

/// C: `sub_021d004c`: command 483: `*var = FUN_02009628(S)`; 0.
pub fn cmd_483<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_020120f4(gs);
    let s = record_block(h, gd);
    let r = h.fun_02009628(s);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021d0084`: command 476: a, b; [`fill_matches_top`]`(GAMESYS, a, b)`; 0.
pub fn cmd_476<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    fill_matches_top(h, gs, a, b);
    0
}

/// C: `sub_021d00b8`: command 479: i, x (u16 operand), var; `*var = `[`match_message`]`(word set, GAMESYS, R, i, x)`; 0.
pub fn cmd_479<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let r = h.fun_02012f38(gd);
    let ws = h.fun_02158f74(sw);
    let i = h.fun_02159b10(vm, ctx);
    let x = h.fun_02011330(vm);
    let var = h.fun_02159ae8(vm, ctx);
    let v = match_message(h, ws, gs, r, i, x);
    h.w16(var, v as u16);
    0
}

/// C: `sub_021d0114`: command 582: `*var = FUN_0200856C(FUN_02012944(GAMEDATA)) != 0`; 0.
pub fn cmd_582<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let st = h.fun_02012944(gd);
    let v = (h.fun_0200856c(st) != 0) as u16;
    h.w16(var, v);
    0
}

/// C: `sub_021d1670`: command 521: k, _, _, var; k == 0 -> `*var` = `FUN_020092EC(Z) == 1 && FUN_020092D8(Z) == 1` (Z =
/// `FUN_02012F8C(GAMEDATA)`); other k nothing; 0.
pub fn cmd_521<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let k = h.fun_02159b10(vm, ctx);
    h.fun_02159b10(vm, ctx);
    h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let z = h.fun_02012f8c(gd);
    if k == 0 {
        let v = h.fun_020092ec(z) == 1 && h.fun_020092d8(z) == 1;
        h.w16(var, v as u16);
    }
    0
}

/// C: `sub_021d16d8`: command 522: k, a, _, var on the 5 records of Z = `FUN_02012F8C(GAMEDATA)`: k 0 -> `*var` = 1, then 0 if
/// any record `FUN_02009390(Z, i)` is null or tagged 0x7E / 0; 1 -> `*var = FUN_020093D8(Z) != 0x7F`; 2 -> `*var =
/// FUN_02009418(Z) == 1`; 3 -> `FUN_02009428(Z, 0)`; 4 -> `FUN_020093F0(Z, a)`; 0.
pub fn cmd_522<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let k = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159b10(vm, ctx);
    h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let z = h.fun_02012f8c(gd);
    match k {
        0 => {
            h.w16(var, 1);
            for i in 0..5 {
                let p = h.fun_02009390(z, i);
                if p == 0 {
                    h.w16(var, 0);
                    continue;
                }
                let tag = h.r16(p);
                if tag == RECORD_EMPTY || tag == 0 {
                    h.w16(var, 0);
                }
            }
        }
        1 => {
            let v = h.fun_020093d8(z) != 0x7F;
            h.w16(var, v as u16);
        }
        2 => {
            let v = h.fun_02009418(z) == 1;
            h.w16(var, v as u16);
        }
        3 => h.fun_02009428(z, 0),
        4 => h.fun_020093f0(z, a),
        _ => {}
    }
    0
}

/// C: `sub_021d179c`: command 523: k, i; the name of record i of Z (when set: not 0x7E / 0) goes into word-set slot k
/// (`FUN_02045994(B, p + 2, 0xD)`, `FUN_0201EE0C(L, k, B, 0, 0, 2)`) through a temporary string buffer; 0.
pub fn cmd_523<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let heap = h.fun_0215a5cc(ctx);
    let ws = h.fun_02158f74(sw);
    let z = h.fun_02012f8c(gd);
    let k = h.fun_02159b10(vm, ctx);
    let i = h.fun_02159b10(vm, ctx);
    let p = h.fun_02009390(z, i);
    if p == 0 {
        return 0;
    }
    let tag = h.r16(p);
    if tag == RECORD_EMPTY || tag == 0 {
        return 0;
    }
    let buf = h.fun_020457b0(0xE, heap);
    h.fun_02045994_at(buf, p + 2, 0xD);
    h.fun_0201ee0c(ws, k, buf, 0, 0, 2);
    h.fun_02045808(buf);
    0
}

/// C: `FUN_overlay_d_21__021d182c`: obj_anim_create(fm, id, vec, heap): the field proc `FUN_021BB990(-1, FUN_02188D24(fm),
/// 0x021DD5F0)` (its work W = `FUN_021BBA6C(X)`) = {+0 the OBJ `FUN_0216DE94(objsys, id)`, +0x188 X, +0x18C the VecFx32
/// `vec`, +0x19C u16 heap}; returns W.
pub fn obj_anim_create<H: Misc21Hooks>(h: &mut H, fm: u32, id: u32, vec: u32, heap: u32) -> u32 {
    let mut v = [0u8; 12];
    for (k, b) in v.iter_mut().enumerate() {
        *b = h.r8(vec + k as u32);
    }
    obj_anim_create_v(h, fm, id, &v, heap)
}

/// [`obj_anim_create`] with the VecFx32 as bytes (command 347 passes a stack record).
fn obj_anim_create_v<H: Misc21Hooks>(
    h: &mut H,
    fm: u32,
    id: u32,
    vec: &[u8; 12],
    heap: u32,
) -> u32 {
    let objsys = h.fun_02188c7c(fm);
    let x0 = h.fun_02188d24(fm);
    let x = h.fun_021bb990(0xFFFF_FFFF, x0, ANIM_PROC_TABLE);
    let w = h.fun_021bba6c(x);
    h.w32(w + 0x188, x);
    h.w16(w + 0x19C, heap as u16);
    for (k, &b) in vec.iter().enumerate() {
        h.w8(w + 0x18C + k as u32, b);
    }
    let o = h.fun_0216de94(objsys, id);
    h.w32(w, o);
    w
}

/// C: `FUN_overlay_d_21__021d1884`: obj_anim_delete(W): `FUN_021BBA1C(W + 0x188 word)` (thunk tail call: delete the proc).
pub fn obj_anim_delete<H: Misc21Hooks>(h: &mut H, w: u32) {
    let x = h.r32(w + 0x188);
    h.fun_021bba1c(x);
}

/// C: `FUN_overlay_d_21__021d1894`: obj_anim_start(W, k): the OBJ animation `FUN_0201147C(W+0x19C, 0xCB, k, 0xA, W + 4,
/// 0x180)` -> W+0x184; W+0x19A = 1 (running), W+0x199 = k; [`obj_anim_apply`] once; `FUN_0216D568(OBJ, 0x8000)`.
pub fn obj_anim_start<H: Misc21Hooks>(h: &mut H, w: u32, k: u32) {
    let heap = u32::from(h.r16(w + 0x19C));
    let a = h.fun_0201147c(heap, 0xCB, k, 0xA, w + 4, 0x180);
    h.w32(w + 0x184, a);
    h.w8(w + 0x19A, 1);
    h.w8(w + 0x199, k as u8);
    let o = h.r32(w);
    obj_anim_apply(h, a, o, w + 0x18C);
    let o = h.r32(w);
    h.fun_0216d568(o, 0x8000);
}

/// C: `FUN_overlay_d_21__021d18e0`: obj_anim_done(W): the animation is not running (W+0x19A == 0).
pub fn obj_anim_done<H: crate::hooks::MemEnv>(h: &mut H, w: u32) -> bool {
    h.r8(w + 0x19A) == 0
}

/// C: `FUN_overlay_d_21__021d18f4`: the field-proc step of the OBJ animation (proc table 0x021DD5F0; `(-, -, W)`): while
/// running (W+0x19A == 1) step the animation (`FUN_02011590(A, 0x1000)`): not finished -> [`obj_anim_apply`]; finished ->
/// `FUN_02011558(A)`, W+0x184 = 0, W+0x19A = 0, the OBJ's position goes through `FUN_0216E014(OBJ, &pos, FUN_0216D5F8(OBJ))`,
/// [`ANIM_FLAGS`]`[W+0x199]` -> `FUN_0216D570(OBJ, 0x8000)`; then `FUN_0216D758(OBJ)`.
pub fn obj_anim_proc<H: Misc21Hooks>(h: &mut H, _a: u32, _b: u32, w: u32) {
    if h.r8(w + 0x19A) != 1 {
        return;
    }
    let a = h.r32(w + 0x184);
    if h.fun_02011590(a, 0x1000) == 0 {
        let a = h.r32(w + 0x184);
        let o = h.r32(w);
        obj_anim_apply(h, a, o, w + 0x18C);
    } else {
        let a = h.r32(w + 0x184);
        h.fun_02011558(a);
        h.w32(w + 0x184, 0);
        h.w8(w + 0x19A, 0);
        let o = h.r32(w);
        let mut p = [0u8; 12];
        h.fun_0216d88c(o, &mut p);
        let o = h.r32(w);
        let d = h.fun_0216d5f8(o);
        let o = h.r32(w);
        h.fun_0216e014(o, &p, d);
        let k = h.r8(w + 0x199) as usize;
        if ANIM_FLAGS[k] != 0 {
            let o = h.r32(w);
            h.fun_0216d570(o, 0x8000);
        }
    }
    let o = h.r32(w);
    h.fun_0216d758(o);
}

/// C: `FUN_overlay_d_21__021d1984`: the empty field-proc callback of the OBJ animation (`bx lr`).
pub fn obj_anim_nop() {}

/// C: `FUN_overlay_d_21__021d1988`: move the OBJ `mmdl` to the animation's translation + `vec` (VecFx32 at `vec`) and turn it
/// to the direction of the animation's rotation (0 / 90 / 180 / 270 degrees -> 1 / 3 / 0 / 2; other angles: no turn).
pub fn obj_anim_apply<H: Misc21Hooks>(h: &mut H, anime: u32, mmdl: u32, vec: u32) {
    let mut t = [0u8; 12];
    h.fun_02011610(anime, &mut t);
    let mut r = [0u8; 12];
    h.fun_02011638(anime, &mut r);
    let mut v = [0u8; 12];
    for (k, b) in v.iter_mut().enumerate() {
        *b = h.r8(vec + k as u32);
    }
    let a = t;
    h.fun_0207c9d0(&a, &v, &mut t);
    h.fun_0216d89c(mmdl, &t);
    let angle = u32::from_le_bytes([r[4], r[5], r[6], r[7]]);
    if let Some(&(_, dir)) = ROT_DIRS.iter().find(|&&(a, _)| a == angle) {
        h.fun_0216d60c(mmdl, dir);
    }
}

/// C: `sub_021d1a04`: command 347: k; W = [`obj_anim_create`]`(fm, k, the player position (FUN_021A3038), field heap)` goes into
/// the SW slot (`FUN_02159104`); 0.
pub fn cmd_347<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let r = h.fun_02158f58(sw);
    let fm = h.r32(r + 4);
    let heap = h.fun_02188c84(fm);
    let k = h.fun_02159b10(vm, ctx);
    let fm = h.r32(r + 4);
    let p = h.fun_02188cb4(fm);
    let mut v = [0u8; 12];
    h.fun_021a3038(p, &mut v);
    let fm = h.r32(r + 4);
    let w = obj_anim_create_v(h, fm, k, &v, heap);
    h.fun_02159104(sw, w);
    0
}

/// C: `sub_021d1a54`: command 348: [`obj_anim_delete`] of the SW slot (`FUN_0215910C`), the slot cleared; 0.
pub fn cmd_348<H: Misc21Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let w = h.fun_0215910c(sw);
    obj_anim_delete(h, w);
    h.fun_02159104(sw, 0);
    0
}

/// C: `sub_021d1a74`: command 349: k; [`obj_anim_start`]`(SW slot, k)`; 0.
pub fn cmd_349<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let k = h.fun_02159b10(vm, ctx);
    let w = h.fun_0215910c(sw);
    obj_anim_start(h, w, k);
    0
}

/// C: `sub_021d1a9c`: the GMEVENT function of command 350 (W = {anim W}): done (1) once [`obj_anim_done`]; else 0.
pub fn event_350<H: Misc21Hooks>(h: &mut H, _event: u32, _seq: u32, w: u32) -> u32 {
    let a = h.r32(w);
    obj_anim_done(h, a) as u32
}

/// C: `sub_021d1ab0`: command 350: the GMEVENT [`event_350`] (W = {the SW slot}) runs as a child event; 1.
pub fn cmd_350<H: Misc21Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = h.fun_020122c0(gs, 0, EVENT_350, 4);
    let w = h.fun_020124e8(ev);
    let a = h.fun_0215910c(sw);
    h.w32(w, a);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021d1aec`: the GMEVENT function of commands 412 / 413 (W = {X, mode}): seq 0 waits for `FUN_021A0ED4(X)`, then
/// `FUN_021A0F30(X, mode)`, seq 1; seq 1 waits for `FUN_021A0ED4(X)`, done (1); else 0.
pub fn event_412<H: Misc21Hooks>(h: &mut H, _event: u32, seq: u32, w: u32) -> u32 {
    match h.r32(seq) {
        0 => {
            let x = h.r32(w);
            if h.fun_021a0ed4(x) != 0 {
                let (x, m) = (h.r32(w), h.r32(w + 4));
                h.fun_021a0f30(x, m);
                let s = h.r32(seq);
                h.w32(seq, s.wrapping_add(1));
            }
            0
        }
        1 => {
            let x = h.r32(w);
            (h.fun_021a0ed4(x) != 0) as u32
        }
        _ => 0,
    }
}

/// Commands 412 / 413: X = `FUN_02188CCC(fm)`; the GMEVENT [`event_412`] with W = {X, mode} runs as a child event.
fn start_412<H: Misc21Hooks>(h: &mut H, sw: u32, ctx: u32, x: u32, mode: u32) {
    let gs = h.fun_0215a5dc(ctx);
    let ev = h.fun_020122c0(gs, 0, EVENT_412, 8);
    let w = h.fun_020124e8(ev);
    h.w32(w, x);
    h.w32(w + 4, mode);
    h.fun_0215897c(sw, ev);
}

/// C: `sub_021d1b2c`: command 412: k; the event with mode [`TABLE_412`]`[k]`; `FUN_02159144(8)`; 1. Rust panics for k > 3.
pub fn cmd_412<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let r = h.fun_02158f58(sw);
    let k = h.fun_02159b10(vm, ctx);
    let fm = h.r32(r + 4);
    let x = h.fun_02188ccc(fm);
    start_412(h, sw, ctx, x, TABLE_412[k as usize]);
    h.fun_02159144(8);
    1
}

/// C: `sub_021d1b8c`: command 413: the event with mode 0; `FUN_02159168(8)`; 1.
pub fn cmd_413<H: Misc21Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let r = h.fun_02158f58(sw);
    let fm = h.r32(r + 4);
    let x = h.fun_02188ccc(fm);
    start_412(h, sw, ctx, x, 0);
    h.fun_02159168(8);
    1
}

/// C: `sub_021d1bd8`: the init proc of `script_sys::INIT_PROCS` slot 8 (`(task, w)`, task +8 = SW): X =
/// `FUN_02188CCC(fm)`; w[0] 0: `FUN_021A0ED4(X)` -> `FUN_021A0F30(X, 0)`, `FUN_02159168(8)`, w[0] = 1; w[0] 1:
/// `FUN_021A0ED4(X)` -> 1 (done); else 0.
pub fn init_proc<H: Misc21Hooks>(h: &mut H, task: u32, w: u32) -> u32 {
    let sw = h.r32(task + 8);
    let r = h.fun_02158f58(sw);
    let fm = h.r32(r + 4);
    let x = h.fun_02188ccc(fm);
    match h.r32(w) {
        0 => {
            if h.fun_021a0ed4(x) != 0 {
                h.fun_021a0f30(x, 0);
                h.fun_02159168(8);
                let s = h.r32(w);
                h.w32(w, s.wrapping_add(1));
            }
            0
        }
        1 => (h.fun_021a0ed4(x) != 0) as u32,
        _ => 0,
    }
}
/// C: `FUN_overlay_d_21__021d1c24`: show_mon_event(GAMESYS, fm, slot): the GMEVENT 0x021D1C8D (its UI flow is render side,
/// not ported) with the work W (0x58 bytes, cleared) = {`FUN_02007B44(SAVE)`, GAMESYS, GAMEDATA, fm, the player, party mon
/// `slot`, -, `FUN_02188C54(fm)`, .., +0x4E u8 slot, +0x54 0}; returns the event.
pub fn show_mon_event<H: Misc21Hooks>(h: &mut H, gs: u32, fm: u32, slot: u32) -> u32 {
    let ev = h.fun_020122c0(gs, 0, EVENT_SHOW_MON, 0x58);
    let w = h.fun_020124e8(ev);
    for k in 0..0x58 {
        h.w8(w + k, 0);
    }
    h.w32(w + 4, gs);
    let gd = h.fun_020120f4(gs);
    h.w32(w + 8, gd);
    let save = h.fun_02012ebc(gd);
    let x = h.fun_02007b44(save);
    h.w32(w, x);
    h.w32(w + 0xC, fm);
    let p = h.fun_02188cb4(fm);
    h.w32(w + 0x10, p);
    let gd = h.r32(w + 8);
    let party = h.fun_02012934(gd);
    let mon = h.fun_0201ab28(party, slot);
    h.w32(w + 0x14, mon);
    let fm = h.r32(w + 0xC);
    let x = h.fun_02188c54(fm);
    h.w32(w + 0x1C, x);
    h.w8(w + 0x4E, slot as u8);
    h.w32(w + 0x54, 0);
    ev
}

/// C: `FUN_overlay_d_21__021d1f48`: a callback that sets the flag word `*p` = 1; 1.
pub fn flag_cb<H: crate::hooks::MemEnv>(h: &mut H, _x: u32, p: u32) -> u32 {
    h.w32(p, 1);
    1
}

/// C: `FUN_overlay_d_21__021d1f50`: pause the field OBJs of W's GAMESYS (`FUN_0216D9A0(objsys)`).
pub fn pause_objs<H: Misc21Hooks>(h: &mut H, w: u32) {
    let gs = h.r32(w + 4);
    let fm = h.fun_0201210c(gs);
    let s = h.fun_02188c7c(fm);
    h.fun_0216d9a0(s);
}

/// C: `FUN_overlay_d_21__021d1f64`: resume the field OBJs of W's GAMESYS (`FUN_0216D9E4(objsys)`).
pub fn resume_objs<H: Misc21Hooks>(h: &mut H, w: u32) {
    let gs = h.r32(w + 4);
    let fm = h.fun_0201210c(gs);
    let s = h.fun_02188c7c(fm);
    h.fun_0216d9e4(s);
}

/// C: `sub_021d213c`: command 524: k, a, b, var; k 1 / 2 pick the flag-set pair (0x5C0, 0x67B) / (0x664, 0x6B7): `*var` =
/// `FUN_020249B0(f, a & 0x7FF) && FUN_020249B0(g, b & 0x7FF)`; other k: `*var = 0`; 0.
pub fn cmd_524<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let k = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let (f, g) = match k {
        1 => FLAGS_524[0],
        2 => FLAGS_524[1],
        _ => {
            h.w16(var, 0);
            return 0;
        }
    };
    let v = h.fun_020249b0(f, a & 0x7FF) != 0 && h.fun_020249b0(g, b & 0x7FF) != 0;
    h.w16(var, v as u16);
    0
}

/// C: `FUN_overlay_d_21__021d21c0`: `v` is a key of [`TABLE_DD614`].
pub fn in_table_a(v: u32) -> bool {
    TABLE_DD614.iter().any(|&(k, _)| k == v)
}

/// C: `FUN_overlay_d_21__021d21e0`: `v` is a key of [`TABLE_DD614`] whose flag is 1.
pub fn in_table_b(v: u32) -> bool {
    TABLE_DD614.iter().any(|&(k, f)| k == v && f == 1)
}

/// C: `FUN_overlay_d_21__021d220c`: count_script(GAMESYS, fm): n = `FUN_02008990(FUN_02008854(SAVE))`; n is a key of
/// [`COUNT_SCRIPTS`] -> `FUN_02158768(GAMESYS, script, 0, heap)`; n > 500 -> the same with script 0x28E1; else 0.
pub fn count_script<H: Misc21Hooks>(h: &mut H, gs: u32, fm: u32) -> u32 {
    let gd = h.fun_020120f4(gs);
    let save = h.fun_02012ebc(gd);
    let x = h.fun_02008854(save);
    let n = h.fun_02008990(x);
    for &(key, script) in &COUNT_SCRIPTS {
        if n == u32::from(key) {
            let heap = h.fun_02188c84(fm);
            return h.fun_02158768(gs, u32::from(script), 0, heap);
        }
    }
    if n > 500 {
        let heap = h.fun_02188c84(fm);
        return h.fun_02158768(gs, SCRIPT_OVER_500, 0, heap);
    }
    0
}

/// C: `sub_021d2278`: the GMEVENT function of [`start_2278`] (W = {GAMESYS, fm, -, n}): seq 0 starts the camera move
/// (`FUN_021C6DE0(FUN_02188D38(fm), FUN_021C7858(fm, 0x28, &{0, -0x40000, 0}), 0)`), n = 8; 1 counts n down, then the SE 0x63C;
/// 2 waits for the SE, turns the player (`FUN_021A30A4(P, 3)`) and runs the warp `FUN_02184FD4(W[0], zone of FUN_021CD440)` as a
/// child event; 3 `FUN_0216D410(OBJ, 8)`; 4 waits for `FUN_0216D454(OBJ)`, done (1); each finished step: seq + 1; 0.
pub fn event_2278<H: Misc21Hooks>(h: &mut H, event: u32, seq: u32, w: u32) -> u32 {
    match h.r32(seq) {
        0 => {
            let fm = h.r32(w + 4);
            let t = h.fun_02188d38(fm);
            let mut rec = [0u8; 12];
            rec[4..8].copy_from_slice(&0xFFFC_0000u32.to_le_bytes());
            let fm = h.r32(w + 4);
            let x = h.fun_021c7858(fm, 0x28, &rec);
            h.fun_021c6de0(t, x, 0);
            h.w32(w + 0xC, 8);
        }
        1 => {
            let n = h.r32(w + 0xC).wrapping_sub(1);
            h.w32(w + 0xC, n);
            if n as i32 > 0 {
                return 0;
            }
            h.fun_020061e4(SE_2278);
        }
        2 => {
            let s = h.fun_02006148(SE_2278);
            if h.fun_02006224(s) != 0 {
                return 0;
            }
            let fm = h.r32(w + 4);
            let p = h.fun_02188cb4(fm);
            h.fun_021a3074(p);
            h.fun_021a30a4(p, 3);
            let fm = h.r32(w + 4);
            let mut z = [0u8; 2];
            h.fun_021cd440(fm, &mut z);
            let gs = h.r32(w);
            let ev = h.fun_02184fd4(gs, u32::from(u16::from_le_bytes(z)));
            h.fun_02012374(event, ev);
        }
        3 => {
            let fm = h.r32(w + 4);
            let p = h.fun_02188cb4(fm);
            let o = h.fun_021a3074(p);
            h.fun_0216d410(o, 8);
        }
        4 => {
            let fm = h.r32(w + 4);
            let p = h.fun_02188cb4(fm);
            let o = h.fun_021a3074(p);
            return (h.fun_0216d454(o) != 0) as u32;
        }
        _ => return 0,
    }
    let s = h.r32(seq);
    h.w32(seq, s.wrapping_add(1));
    0
}

/// C: `sub_021d2358`: the GMEVENT function of [`start_2358`] (W = {GAMESYS, fm, flag, n}): seq 0 plays the SE 0x63C, n = 0x18,
/// and with `flag` starts two camera moves (`FUN_021C7260` / `FUN_021C71D4`, `FUN_021C6DE0`) and `FUN_021A01FC(S, 0x7F0F,
/// FUN_021A01D4(S), 0x1A)`; 1 turns the player (`FUN_021A30A4(P, 2)`); 2 counts n down, then the warp `FUN_02184F88(W[0])`
/// runs as a child event; 3 done (1); each finished step: seq + 1; 0.
pub fn event_2358<H: Misc21Hooks>(h: &mut H, event: u32, seq: u32, w: u32) -> u32 {
    let fm = h.r32(w + 4);
    let t = h.fun_02188d38(fm);
    match h.r32(seq) {
        0 => {
            h.fun_020061e4(SE_2278);
            h.w32(w + 0xC, 0x18);
            if h.r32(w + 8) != 0 {
                let fm = h.r32(w + 4);
                let s = h.fun_02188c64(fm);
                let fm = h.r32(w + 4);
                let a = h.fun_021c7260(fm, 0x14, 3);
                let fm = h.r32(w + 4);
                let b = h.fun_021c71d4(fm, 0xC, 3);
                h.fun_021c6de0(t, a, 0);
                h.fun_021c6de0(t, b, a);
                let x = h.fun_021a01d4(s);
                h.fun_021a01fc(s, 0x7F0F, x, 0x1A);
            }
        }
        1 => {
            let fm = h.r32(w + 4);
            let p = h.fun_02188cb4(fm);
            h.fun_021a30a4(p, 2);
        }
        2 => {
            let n = h.r32(w + 0xC).wrapping_sub(1);
            h.w32(w + 0xC, n);
            if n as i32 > 0 {
                return 0;
            }
            let gs = h.r32(w);
            let ev = h.fun_02184f88(gs);
            h.fun_02012374(event, ev);
        }
        3 => return 1,
        _ => return 0,
    }
    let s = h.r32(seq);
    h.w32(seq, s.wrapping_add(1));
    0
}

/// C: `FUN_overlay_d_21__021d2418`: the GMEVENT [`event_2278`] with W = {GAMESYS, fm, 0}; returns it.
pub fn start_2278<H: Misc21Hooks>(h: &mut H, gs: u32, fm: u32) -> u32 {
    let ev = h.fun_020122c0(gs, 0, EVENT_2278, 0x10);
    let w = h.fun_020124e8(ev);
    h.w32(w, gs);
    h.w32(w + 4, fm);
    h.w32(w + 8, 0);
    ev
}

/// C: `FUN_overlay_d_21__021d2440`: the GMEVENT [`event_2358`] with W = {GAMESYS, fm, flag}; returns it.
pub fn start_2358<H: Misc21Hooks>(h: &mut H, gs: u32, fm: u32, flag: u32) -> u32 {
    let ev = h.fun_020122c0(gs, 0, EVENT_2358, 0x10);
    let w = h.fun_020124e8(ev);
    h.w32(w, gs);
    h.w32(w + 4, fm);
    h.w32(w + 8, flag);
    ev
}

/// C: `sub_021d2468`: command 525 (lottery): v, A, B, C; on the saved day (`FUN_02008B98` / `FUN_02008B94` of
/// `FUN_02008B08(SAVE)` equal to today's day / month) `*A` = 0x34, `*B` = 0x53, `*C` = [`LOTTERY`]`[30]`; else r =
/// `FUN_020056EC(v ? 0x1E : 0x16)` (random below), `*A` = r + 0x16, `*B` = r + 0x35, `*C` = [`LOTTERY`]`[r]`; 0.
pub fn cmd_525<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let b = h.fun_02159ae8(vm, ctx);
    let c = h.fun_02159ae8(vm, ctx);
    let save = h.fun_02012ebc(gd);
    let s = h.fun_02008b08(save);
    let m = h.fun_021690d0(gd);
    let d = h.fun_021690e0(gd);
    let sm = h.fun_02008b94(s);
    let x = if d == h.fun_02008b98(s) && m == sm {
        h.w16(a, 0x34);
        h.w16(b, 0x53);
        LOTTERY[30]
    } else {
        let r = h.fun_020056ec(if v != 0 { 0x1E } else { 0x16 });
        h.w16(a, r.wrapping_add(0x16) as u16);
        h.w16(b, r.wrapping_add(0x35) as u16);
        LOTTERY[r as usize]
    };
    h.w16(c, x);
    0
}

/// C: `sub_021d2524`: command 551: a, b, c, d; the GMEVENT [`event_551`] (W, 0x18 bytes) whose window `FUN_021D25A4(W)` and 3D
/// model `FUN_021D25F4(W, a, b, c, d, 0)` (render, D3) are set up here runs as a child event; 1.
pub fn cmd_551<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    h.fun_0201210c(gs);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    let d = h.fun_02159b10(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let ev = h.fun_020122c0(gs, 0, EVENT_551, 0x18);
    let w = h.fun_020124e8(ev);
    h.fun_021d25a4(w);
    h.fun_021d25f4(w, a, b, c, d, 0);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021d26d0`: the GMEVENT function of command 551: seq 0 waits for A / B (`FUN_020362DC() & 3`), then frees the model
/// and the window (`FUN_021D26AC` / `FUN_021D25E8`, render), seq 1; seq 1 deletes the window `FUN_020452E8(W[0])`, done (1);
/// else 0.
pub fn event_551<H: Misc21Hooks>(h: &mut H, _event: u32, seq: u32, w: u32) -> u32 {
    match h.r32(seq) {
        0 => {
            if h.fun_020362dc() & 3 != 0 {
                h.fun_021d26ac(w);
                h.fun_021d25e8(w);
                let s = h.r32(seq);
                h.w32(seq, s.wrapping_add(1));
            }
            0
        }
        1 => {
            let x = h.r32(w);
            h.fun_020452e8(x);
            1
        }
        _ => 0,
    }
}

/// C: `sub_021d2710`: command 553: v, var; `*var = FUN_0216BF9C(FUN_02012F08(GAMEDATA), v, FUN_02012944(GAMEDATA))`; 0.
pub fn cmd_553<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let x = h.fun_02012f08(gd);
    let st = h.fun_02012944(gd);
    let r = h.fun_0216bf9c(x, v, st);
    h.w16(var, r as u16);
    0
}

/// C: `FUN_overlay_d_21__021d2750`: grotto_check(GAMESYS): false (blocked) when the zone flag test `FUN_02013C60` fails, the
/// player state (`FUN_021A3098`) is 2 / 3, the player stands in a type-0 event-data OBJ record's rectangle or on a type-1
/// record's position (in field-map mode 1), the field-map mode is 0 and `FUN_02162890` says the spot in front is taken, or (in
/// zones 0 / 0x1A8) the player is on one of the 2 x 10 (x, z) byte pairs of `FUN_02186830`'s tables; true otherwise (also,
/// early, with no records in other zones).
pub fn grotto_check<H: Misc21Hooks>(h: &mut H, gs: u32) -> bool {
    let gd = h.fun_020120f4(gs);
    let r = h.fun_02012844(gd);
    let a = h.fun_02162740(r);
    let n = h.fun_02162744(r) as i32;
    let fm = h.fun_0201210c(gs);
    let objsys = h.fun_02188c7c(fm);
    let p = h.fun_02188cb4(fm);
    let z = h.fun_02188c9c(fm);
    if h.fun_02013c60(z & 0xFFFF) == 0 {
        return false;
    }
    if h.fun_021a3098(p).wrapping_sub(2) <= 1 {
        return false;
    }
    if (a == 0 || n == 0) && z != 0 && z != ZONE_1A8 {
        return true;
    }
    let mut px = [0u8; 2];
    let mut py = [0u8; 2];
    let mut pz = [0u8; 2];
    let mut b = [0u8; 8];
    let mut i: i32 = 0;
    while i < n {
        let e = a.wrapping_add(0x24u32.wrapping_mul(i as u32));
        match h.r32(e + 0x18) {
            0 => {
                let mut o = [0u8; 0x10];
                let id = u32::from(h.r16(e + 2));
                h.fun_0216e2e8(objsys, id, &mut o);
                h.fun_021a35fc(p, &mut px, &mut py, &mut pz);
                let x = i32::from(i16::from_le_bytes(px));
                let ex = i32::from(h.r16(e + 0x1C));
                let ez = i32::from(h.r16(e + 0x1E));
                let zz = i32::from(i16::from_le_bytes(pz));
                if ex <= x && x < ex + i32::from(o[0xB]) && ez - i32::from(o[0xC]) < zz && zz <= ez
                {
                    return false;
                }
            }
            1 if h.fun_02188cfc(fm) == 1 => {
                h.fun_021a36bc(p, &mut b);
                let q = e + 0x1C;
                if h.r16(q) == u16::from_le_bytes([b[0], b[1]])
                    && h.r16(q + 2) == u16::from_le_bytes([b[6], b[7]])
                    && h.r16(q + 4) as i16 == i16::from_le_bytes([b[4], b[5]])
                {
                    return false;
                }
            }
            _ => {}
        }
        i += 1;
    }
    if h.fun_02188cfc(fm) == 0 {
        let o = h.fun_021a3074(p);
        let z2 = h.fun_0201296c(gd);
        let mut pos = [0u8; 12];
        h.fun_0216d88c(o, &mut pos);
        if h.fun_02162890(r, z2, &pos) != 0 {
            return false;
        }
    }
    if z != 0 && z != ZONE_1A8 {
        return true;
    }
    h.fun_021a35fc(p, &mut px, &mut py, &mut pz);
    let t = h.fun_021867c4(0, HEAP_GROTTO);
    let u = h.fun_02186830(t, (z != 0) as u32, HEAP_GROTTO);
    let x = i32::from(i16::from_le_bytes(px));
    let zz = i32::from(i16::from_le_bytes(pz));
    // r1 / r2 at the `FUN_0218683C(u)` calls are whatever the compare loop left there (the spec stub logs them)
    let (mut r1, mut r2) = (0u32, 0u32);
    for second in [false, true] {
        for k in 0..10 {
            let q = if second {
                h.fun_0218684c(u, k)
            } else {
                h.fun_02186844(u, k)
            };
            let (p0, p1) = (i32::from(h.r8(q)), i32::from(h.r8(q + 1)));
            r2 = p0 as u32;
            r1 = x as u32;
            if p0 != x {
                continue;
            }
            r1 = p1 as u32;
            if p1 == zz {
                h.fun_0218683c(u, r1, r2);
                h.fun_02186804(t);
                return false;
            }
        }
    }
    h.fun_0218683c(u, r1, r2);
    h.fun_02186804(t);
    true
}

/// C: `FUN_overlay_d_21__021d297c`: the GMEVENT [`event_29a8`] with W = {fm, X = `FUN_02188CCC(fm)`}; returns it.
pub fn start_29a8<H: Misc21Hooks>(h: &mut H, gs: u32) -> u32 {
    let ev = h.fun_020122c0(gs, 0, EVENT_29A8, 8);
    let w = h.fun_020124e8(ev);
    let fm = h.fun_0201210c(gs);
    h.w32(w, fm);
    let x = h.fun_02188ccc(fm);
    h.w32(w + 4, x);
    ev
}

/// C: `sub_021d29a8`: the GMEVENT function of [`start_29a8`] (W = {fm, X}): seq 0 `FUN_021A0F30(X, 9)`; 1 waits for
/// `FUN_021A0F94(X) == 0xE`, then `FUN_021A0FA8(X)`; each: seq + 1, 0; 2 `FUN_021A0F30(X, 0)`, done (1); else 0.
pub fn event_29a8<H: Misc21Hooks>(h: &mut H, _event: u32, seq: u32, w: u32) -> u32 {
    match h.r32(seq) {
        0 => {
            let x = h.r32(w + 4);
            h.fun_021a0f30(x, 9);
        }
        1 => {
            let x = h.r32(w + 4);
            if h.fun_021a0f94(x) != 0xE {
                return 0;
            }
            let x = h.r32(w + 4);
            h.fun_021a0fa8(x);
        }
        2 => {
            let x = h.r32(w + 4);
            h.fun_021a0f30(x, 0);
            return 1;
        }
        _ => return 0,
    }
    let s = h.r32(seq);
    h.w32(seq, s.wrapping_add(1));
    0
}

/// The record counters of the save data (`FUN_02008C58(SAVE)`).
fn counters<H: Misc21Hooks>(h: &mut H, gs: u32) -> u32 {
    let gd = h.fun_020120f4(gs);
    let save = h.fun_02012ebc(gd);
    h.fun_02008c58(save)
}

/// C: `sub_021d2adc`: command 570: a, b; `FUN_02008DAC(counters, a, b)` (add b to counter a); 0.
pub fn cmd_570<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = counters(h, gs);
    h.fun_02008dac(c, a, b);
    0
}

/// C: `sub_021d2b18`: command 571: a, var; `*var = FUN_02008DFC(counters, a)`; 0.
pub fn cmd_571<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let c = counters(h, gs);
    let v = h.fun_02008dfc(c, a);
    h.w16(var, v as u16);
    0
}

/// C: `sub_021d2b54`: command 573 (a numbered reward): v, var; word 0 = the player name, word 3 = the species of the first
/// party mon that is not an egg; by the RTC minute word t[1] (`FUN_0203F4CC`): t[1] < 10 -> row r = `FUN_020056EC(0x1D)` of
/// [`REWARD_ROWS`] puts its counters a / b (`FUN_02008DFC` of `FUN_02012F2C(GAMEDATA)`) into words 1 / 2 (digits na / nb); a
/// row below 11 whose counters are not all set falls back to 0; rows >= 11, or when `FUN_02008550(status) == 1`, add 11; else
/// t[1] < 0x23 -> 0x28 + `FUN_020056EC(0x32)`, other -> 0x5A + `FUN_020056EC(0x50)`; `*var` = [`adjust_b`]`(GD,
/// `[`adjust_a`]`(GD, the number), v)`; 0.
pub fn cmd_573<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let st = h.fun_02012944(gd);
    let ws = h.fun_02158f74(sw);
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    h.fun_0201f02c(ws, 0, st);
    let party = h.fun_02012934(gd);
    let n = h.fun_0201aa18(party) as i32;
    let mut i: i32 = 0;
    while i < n {
        let mon = h.fun_0201ab28(party, i as u32);
        if h.fun_02017e1c(mon, PP_EGG, 0) == 0 {
            let sp = h.fun_02017e1c(mon, PP_SPECIES, 0) & 0xFFFF;
            h.fun_0201ee60(ws, 3, sp);
            break;
        }
        i += 1;
    }
    let mut t = [0u8; 16];
    h.fun_0203f4cc(&mut t);
    let t1 = u32::from_le_bytes([t[4], t[5], t[6], t[7]]);
    let (base, r) = if t1 < 10 {
        let mut r = h.fun_020056ec(0x1D);
        let (mut s1, mut s2) = (0u32, 0u32);
        let z = h.fun_02012f2c(gd);
        let row = REWARD_ROWS[r as usize];
        if row[0] != 0xFFFF_FFFF {
            s1 = h.fun_02008dfc(z, row[0]);
            h.fun_0201efa4(ws, 1, s1, row[2], 0, 1);
        }
        let bb = row[1];
        if bb != 0xFFFF_FFFF {
            s2 = h.fun_02008dfc(z, bb);
            h.fun_0201efa4(ws, 2, s2, row[3], 0, 1);
        }
        if (r as i32) < 11 {
            let keep = if row[0] != 0xFFFF_FFFF && bb != 0xFFFF_FFFF {
                s1 != 0 && s2 != 0
            } else {
                bb != 0xFFFF_FFFF || s1 != 0
            };
            if !keep {
                r = 0;
            }
        }
        if (r as i32) >= 11 || h.fun_02008550(st) & 0xFF == 1 {
            r = r.wrapping_add(11);
        }
        (0, r)
    } else if t1 < 0x23 {
        (0x28, h.fun_020056ec(0x32))
    } else {
        (0x5A, h.fun_020056ec(0x50))
    };
    let x = adjust_a(h, gd, base + r);
    let x = adjust_b(h, gd, x, v);
    h.w16(var, x as u16);
    0
}

/// C: `FUN_overlay_d_21__021d2ce8`: adjust_a(GD, x): x one of [`ADJUST_A`] and `FUN_0200C234(FUN_0200C0F0(SAVE))` == 0 -> 0x5A
/// + `FUN_020056EC(0xC)`; else x.
pub fn adjust_a<H: Misc21Hooks>(h: &mut H, gd: u32, x: u32) -> u32 {
    let save = h.fun_02012ebc(gd);
    let y = h.fun_0200c0f0(save);
    if ADJUST_A.contains(&x) {
        if h.fun_0200c234(y) != 0 {
            return x;
        }
        return h.fun_020056ec(0xC).wrapping_add(0x5A);
    }
    x
}

/// C: `FUN_overlay_d_21__021d2d24`: adjust_b(GD, x, v): v == 0 and x one of [`ADJUST_B`] -> 0x6E + `FUN_020056EC(0x14)`; else
/// x.
pub fn adjust_b<H: Misc21Hooks>(h: &mut H, _gd: u32, x: u32, v: u32) -> u32 {
    if v == 0 && ADJUST_B.contains(&x) {
        return h.fun_020056ec(0x14).wrapping_add(0x6E);
    }
    x
}

/// C: `FUN_overlay_d_21__021d2d50`: shake_event(GAMESYS, rec): the GMEVENT [`event_shake`] whose work is the 0x18 bytes of
/// `rec` (the camera-shake record of command 328); returns it.
pub fn shake_event<H: Misc21Hooks>(h: &mut H, gs: u32, rec: u32) -> u32 {
    let ev = h.fun_020122c0(gs, 0, EVENT_SHAKE, 0x1C);
    let w = h.fun_020124e8(ev);
    for k in 0..0x18 {
        let b = h.r8(rec + k);
        h.w8(w + k, b);
    }
    ev
}

/// C: `sub_021d2d7c`: the GMEVENT function of the camera shake (W = {s16 ax, s16 ay, s16 dx, s16 dy, u16 every, u16 k, u16
/// period, u8 n0, u8 n, frames, t, pos *}): seq 0 `FUN_0218F0A8(E)`; 1 waits for `FUN_0218F0E8(E) == 0`, keeps the camera
/// target `FUN_0218EC5C(E)`; 2 one frame: t + 1, phase = (t << 16) / frames, every `every` counted swings the amplitudes drop
/// by (dx, dy) (min 1), `n` swings of `frames` frames, the target is the kept one + (ax, ay) * sin(phase) (`FUN_0218EC90`,
/// `FUN_0218ED04`); after the last swing the offset is cleared and the target restored; 3 done (1); finished steps: seq + 1; 0.
/// UNVERIFIED: frames = 0 divides by zero (`_u32_div_f`; Rust panics).
pub fn event_shake<H: Misc21Hooks>(h: &mut H, event: u32, seq: u32) -> u32 {
    let gs = h.fun_020124e4(event);
    let fm = h.fun_0201210c(gs);
    let w = h.fun_020124e8(event);
    let e = h.fun_02188c58(fm);
    match h.r32(seq) {
        0 => h.fun_0218f0a8(e),
        1 => {
            if h.fun_0218f0e8(e) != 0 {
                return 0;
            }
            let p = h.fun_0218ec5c(e);
            h.w32(w + 0x18, p);
            h.fun_0218ec54(e);
        }
        2 => {
            let t = h.r32(w + 0x14).wrapping_add(1);
            h.w32(w + 0x14, t);
            let phase = (t.wrapping_shl(16) / h.r32(w + 0x10)) & 0xFFFF;
            let every = h.r16(w + 8);
            if every != 0 && i32::from(h.r8(w + 0xE)) - i32::from(h.r8(w + 0xF)) >= i32::from(every)
            {
                let k = h.r16(w + 0xA).wrapping_add(1);
                h.w16(w + 0xA, k);
                if h.r16(w + 0xA) >= h.r16(w + 0xC) {
                    h.w16(w + 0xA, 0);
                    let ay = (h.r16(w + 2) as i16).wrapping_sub(h.r16(w + 6) as i16);
                    h.w16(w + 2, ay as u16);
                    let ax = (h.r16(w) as i16).wrapping_sub(h.r16(w + 4) as i16);
                    h.w16(w, ax as u16);
                    if h.r16(w + 2) as i16 <= 0 {
                        h.w16(w + 2, 1);
                    }
                    if h.r16(w) as i16 <= 0 {
                        h.w16(w, 1);
                    }
                }
            }
            if h.r32(w + 0x14) >= h.r32(w + 0x10) {
                let n = h.r8(w + 0xF).wrapping_sub(1);
                h.w8(w + 0xF, n);
                h.w32(w + 0x14, 0);
            }
            let mut done = false;
            if h.r8(w + 0xF) == 0 {
                h.w32(w + 0x14, 0);
                done = true;
            }
            if done {
                h.fun_0218ed04(e, &[0u8; 12]);
                let p = h.r32(w + 0x18);
                h.fun_0218ec30(e, p);
                h.fun_0218f05c(e);
            } else {
                let s = i32::from(crate::scrcmd_misc10::fx_sin(phase >> 4));
                let p = h.r32(w + 0x18);
                let dx = i32::from(h.r16(w) as i16).wrapping_mul(s);
                let dy = i32::from(h.r16(w + 2) as i16).wrapping_mul(s);
                let mut v = [0u8; 12];
                for (k, b) in v.iter_mut().enumerate() {
                    *b = h.r8(p + k as u32);
                }
                let x = i32::from_le_bytes(v[0..4].try_into().unwrap()).wrapping_add(dx);
                let y = i32::from_le_bytes(v[4..8].try_into().unwrap()).wrapping_add(dy);
                v[0..4].copy_from_slice(&x.to_le_bytes());
                v[4..8].copy_from_slice(&y.to_le_bytes());
                h.fun_0218ec90(e, &v);
                let mut off = [0u8; 12];
                off[0..4].copy_from_slice(&dx.to_le_bytes());
                off[4..8].copy_from_slice(&dy.to_le_bytes());
                h.fun_0218ed04(e, &off);
                return 0;
            }
        }
        3 => return 1,
        _ => return 0,
    }
    let s = h.r32(seq);
    h.w32(seq, s.wrapping_add(1));
    0
}

/// C: `sub_021d2edc`: command 574: a, b; the event `FUN_021F3CFC(GAMESYS, &pos of OBJ a (FUN_0216DE94(FUN_0215A5EC(ctx), a)),
/// a, b)` (overlay at 0x021F3xxx) runs as a child event when there is one; 1.
pub fn cmd_574<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    h.fun_0201210c(gs);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let s = h.fun_0215a5ec(ctx);
    let o = h.fun_0216de94(s, a);
    let mut pos = [0u8; 12];
    h.fun_0216d88c(o, &mut pos);
    let ev = h.fun_021f3cfc(gs, &pos, a, b);
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// Commands 575 / 599 / 600: the event `f(GAMESYS)` of the overlay at 0x021F3xxx runs as a child event when there is one; 1.
fn overlay_event<H: Misc21Hooks>(h: &mut H, ctx: u32, f: fn(&mut H, u32) -> u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = f(h, gs);
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// C: `sub_021d2f44`: command 575: the event `FUN_021F37B8(GAMESYS)` runs as a child event when there is one; 1.
pub fn cmd_575<H: Misc21Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    overlay_event(h, ctx, H::fun_021f37b8)
}

/// The VecFx32 {a << 16 + 0x8000, b << 15, c << 16 + 0x8000} of commands 597 / 598 (grid coordinates to fx32, centred).
fn grid_vec(a: u32, b: u32, c: u32) -> [u8; 12] {
    let mut v = [0u8; 12];
    v[0..4].copy_from_slice(&(a << 16).wrapping_add(0x8000).to_le_bytes());
    v[4..8].copy_from_slice(&(b << 15).to_le_bytes());
    v[8..12].copy_from_slice(&(c << 16).wrapping_add(0x8000).to_le_bytes());
    v
}

/// C: `sub_021d2f6c`: command 597: id, a, b, c, d, e; the event `FUN_021F3970(GAMESYS, &pos of OBJ id (if found; else the
/// unset stack record: 0 here, the Z19 test state), &grid(a, b, c), d << 15, e)` runs as a child event when there is one; 1.
pub fn cmd_597<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let id = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    let d = h.fun_02159b10(vm, ctx);
    let e = h.fun_02159b10(vm, ctx);
    let s = h.fun_02188c7c(fm);
    let o = h.fun_0216de94(s, id);
    let mut pos = [0u8; 12];
    if o != 0 {
        h.fun_0216d88c(o, &mut pos);
    }
    let v = grid_vec(a, b, c);
    let ev = h.fun_021f3970(gs, &pos, &v, d << 15, e);
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// C: `sub_021d3018`: command 598: a, b, c; `FUN_021F3A44(GAMESYS, &grid(a, b, c))`; 0.
pub fn cmd_598<H: Misc21Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    let v = grid_vec(a, b, c);
    h.fun_021f3a44(gs, &v);
    0
}

/// C: `sub_021d3068`: command 599: the event `FUN_021F3ADC(GAMESYS)` runs as a child event when there is one; 1.
pub fn cmd_599<H: Misc21Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    overlay_event(h, ctx, H::fun_021f3adc)
}

/// C: `sub_021d3090`: command 600: the event `FUN_021F3AC0(GAMESYS)` runs as a child event when there is one; 1.
pub fn cmd_600<H: Misc21Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    overlay_event(h, ctx, H::fun_021f3ac0)
}

/// Run the command handler at `handler` (bit 0 ignored) when it is one of [`COMMANDS`]; `None` otherwise.
pub fn dispatch<H: Misc21Hooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x021C_FA88 => cmd_233(h, vm, ctx),
        0x021C_FA58 => cmd_309(h, vm, ctx),
        0x021D_1A04 => cmd_347(h, vm, ctx),
        0x021D_1A54 => cmd_348(h, vm, ctx),
        0x021D_1A74 => cmd_349(h, vm, ctx),
        0x021D_1AB0 => cmd_350(h, vm, ctx),
        0x021D_1B2C => cmd_412(h, vm, ctx),
        0x021D_1B8C => cmd_413(h, vm, ctx),
        0x021C_FEA0 => cmd_472(h, vm, ctx),
        0x021C_FF28 => cmd_473(h, vm, ctx),
        0x021C_FF78 => cmd_474(h, vm, ctx),
        0x021C_FFDC => cmd_475(h, vm, ctx),
        0x021D_0084 => cmd_476(h, vm, ctx),
        0x021D_00B8 => cmd_479(h, vm, ctx),
        0x021C_FEF0 => cmd_481(h, vm, ctx),
        0x021D_0014 => cmd_482(h, vm, ctx),
        0x021D_004C => cmd_483(h, vm, ctx),
        0x021D_1670 => cmd_521(h, vm, ctx),
        0x021D_16D8 => cmd_522(h, vm, ctx),
        0x021D_179C => cmd_523(h, vm, ctx),
        0x021D_213C => cmd_524(h, vm, ctx),
        0x021D_2468 => cmd_525(h, vm, ctx),
        0x021D_2524 => cmd_551(h, vm, ctx),
        0x021D_2710 => cmd_553(h, vm, ctx),
        0x021C_F7DC => cmd_569(h, vm, ctx),
        0x021D_2ADC => cmd_570(h, vm, ctx),
        0x021D_2B18 => cmd_571(h, vm, ctx),
        0x021D_2B54 => cmd_573(h, vm, ctx),
        0x021D_2EDC => cmd_574(h, vm, ctx),
        0x021D_2F44 => cmd_575(h, vm, ctx),
        0x021D_0114 => cmd_582(h, vm, ctx),
        0x021D_2F6C => cmd_597(h, vm, ctx),
        0x021D_3018 => cmd_598(h, vm, ctx),
        0x021D_3068 => cmd_599(h, vm, ctx),
        0x021D_3090 => cmd_600(h, vm, ctx),
        _ => return None,
    })
}

/// Run the GMEVENT function `f(event, seq, work)` when it is one of this area's ported events; `None` otherwise (also for
/// the show-mon event [`EVENT_SHOW_MON`], whose UI flow is not ported).
pub fn call_event<H: Misc21Hooks>(
    h: &mut H,
    f: u32,
    event: u32,
    seq: u32,
    work: u32,
) -> Option<u32> {
    Some(match f {
        EVENT_569 => event_569(h, event, seq, work),
        EVENT_SLOT => event_slot(h, event, seq, work),
        EVENT_350 => event_350(h, event, seq, work),
        EVENT_412 => event_412(h, event, seq, work),
        EVENT_2278 => event_2278(h, event, seq, work),
        EVENT_2358 => event_2358(h, event, seq, work),
        EVENT_551 => event_551(h, event, seq, work),
        EVENT_29A8 => event_29a8(h, event, seq, work),
        EVENT_SHAKE => event_shake(h, event, seq),
        _ => return None,
    })
}

/// Run the init proc `f(task, x)` when it is [`INIT_PROC`] (`script_sys::INIT_PROCS` slot 8); `None` otherwise.
pub fn call_init_proc<H: Misc21Hooks>(h: &mut H, f: u32, task: u32, x: u32) -> Option<u32> {
    (f == INIT_PROC).then(|| init_proc(h, task, x))
}

/// Run the field-proc callback `f(a, b, work)` of the OBJ-animation proc table ([`ANIM_PROC_STEP`] / [`ANIM_PROC_NOP`]);
/// false otherwise.
pub fn call_field_proc<H: Misc21Hooks>(h: &mut H, f: u32, a: u32, b: u32, work: u32) -> bool {
    match f {
        ANIM_PROC_STEP => obj_anim_proc(h, a, b, work),
        ANIM_PROC_NOP => obj_anim_nop(),
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests;
