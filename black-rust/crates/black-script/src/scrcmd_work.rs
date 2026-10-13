//! F8: `scrcmd_work.c`, overlay 10 0x0215A514-0x0215C63A: the SCRCMD WORK / CTX objects and their accessors, the
//! movement-object slots, the choice-menu builder, and 86 commands (system / Pokedex / time / party and Pokemon queries,
//! give Pokemon / egg).
//!
//! Every function of the area (130) is specced in `tools/rom-diff/specs/black-script/scrcmd_work.spec`; its header has
//! the command-id table and the CTX / WORK / MENU layouts. The functions are ported here, listed in [`PORTED`] and
//! checked with `tools/check-fn.sh black-script scrcmd_work`. The 8 ctx getters 0x0215A5CC-0x0215A5F4 of this file are
//! ported in [`crate::script_sys`] (`ctx_*`). [`COMMANDS`] maps the command ids to the handler addresses and [`dispatch`]
//! runs a handler by its table address (the F7 `CommandDispatch` seam).
//!
//! Pointers are `u32` addresses in the [`Mem`](crate::hooks::Mem) window; the callees are [`WorkHooks`] methods named by
//! address (the ones shared with the scrcmd core, vm.c and script_sys operand readers among them, come from
//! [`CoreHooks`]). Stack records the original passes to its callees are byte arrays here.

use crate::hooks::MemEnv;
use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_A514,
    0x0215_A55C,
    0x0215_A5A4,
    0x0215_A5FC,
    0x0215_A60C,
    0x0215_A610,
    0x0215_A614,
    0x0215_A61C,
    0x0215_A624,
    0x0215_A630,
    0x0215_A63C,
    0x0215_A644,
    0x0215_A64C,
    0x0215_A654,
    0x0215_A660,
    0x0215_A66C,
    0x0215_A678,
    0x0215_A684,
    0x0215_A690,
    0x0215_A6AC,
    0x0215_A6B8,
    0x0215_A6C4,
    0x0215_A6D0,
    0x0215_A6DC,
    0x0215_A6FC,
    0x0215_A738,
    0x0215_A760,
    0x0215_A77C,
    0x0215_A7F0,
    0x0215_A864,
    0x0215_A920,
    0x0215_A954,
    0x0215_A990,
    0x0215_A9C0,
    0x0215_AA24,
    0x0215_AA34,
    0x0215_AA48,
    0x0215_AA4C,
    0x0215_AA50,
    0x0215_AA60,
    0x0215_AA90,
    0x0215_AABC,
    0x0215_AADC,
    0x0215_AAF4,
    0x0215_AB18,
    0x0215_AB3C,
    0x0215_AB60,
    0x0215_AB98,
    0x0215_ABD0,
    0x0215_ABEC,
    0x0215_AC1C,
    0x0215_AC3C,
    0x0215_AC68,
    0x0215_ACA8,
    0x0215_ACC8,
    0x0215_AD38,
    0x0215_AD50,
    0x0215_AD7C,
    0x0215_ADCC,
    0x0215_AE34,
    0x0215_AEB8,
    0x0215_AEE8,
    0x0215_AF40,
    0x0215_AF98,
    0x0215_AFE0,
    0x0215_AFF4,
    0x0215_B014,
    0x0215_B028,
    0x0215_B068,
    0x0215_B084,
    0x0215_B0B4,
    0x0215_B104,
    0x0215_B138,
    0x0215_B158,
    0x0215_B190,
    0x0215_B214,
    0x0215_B26C,
    0x0215_B2C4,
    0x0215_B328,
    0x0215_B350,
    0x0215_B378,
    0x0215_B3AC,
    0x0215_B3DC,
    0x0215_B408,
    0x0215_B438,
    0x0215_B464,
    0x0215_B488,
    0x0215_B4AC,
    0x0215_B4DC,
    0x0215_B500,
    0x0215_B558,
    0x0215_B5A8,
    0x0215_B5BC,
    0x0215_B660,
    0x0215_B6C0,
    0x0215_B704,
    0x0215_B74C,
    0x0215_B7E8,
    0x0215_B824,
    0x0215_B848,
    0x0215_B888,
    0x0215_B904,
    0x0215_B994,
    0x0215_BA18,
    0x0215_BAB0,
    0x0215_BAF4,
    0x0215_BB50,
    0x0215_BBD0,
    0x0215_BCA0,
    0x0215_BCE8,
    0x0215_BD2C,
    0x0215_BE24,
    0x0215_BF30,
    0x0215_BF7C,
    0x0215_BFD4,
    0x0215_C01C,
    0x0215_C064,
    0x0215_C0D0,
    0x0215_C138,
    0x0215_C180,
    0x0215_C200,
    0x0215_C2D0,
    0x0215_C40C,
    0x0215_C4C8,
    0x0215_C510,
    0x0215_C558,
    0x0215_C584,
    0x0215_C5B0,
    0x0215_C5EC,
    0x0215_C610,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests). 140 and 378 have identical bodies at two addresses.
pub const COMMANDS: &[(u16, u32)] = &[
    (140, 0x0215_B3AC),
    (141, 0x0215_B3DC),
    (149, 0x0215_B328),
    (150, 0x0215_B350),
    (151, 0x0215_B378),
    (201, 0x0215_AA60),
    (202, 0x0215_AA90),
    (203, 0x0215_AABC),
    (204, 0x0215_AADC),
    (205, 0x0215_AB18),
    (206, 0x0215_AAF4),
    (207, 0x0215_AB3C),
    (208, 0x0215_AB60),
    (209, 0x0215_AB98),
    (210, 0x0215_ABD0),
    (211, 0x0215_AD38),
    (212, 0x0215_ABEC),
    (213, 0x0215_AC3C),
    (214, 0x0215_AC68),
    (215, 0x0215_ACA8),
    (216, 0x0215_AEB8),
    (217, 0x0215_AD50),
    (218, 0x0215_AE34),
    (219, 0x0215_AD7C),
    (220, 0x0215_ADCC),
    (221, 0x0215_AF98),
    (222, 0x0215_AEE8),
    (223, 0x0215_AF40),
    (224, 0x0215_AA50),
    (225, 0x0215_AC1C),
    (226, 0x0215_B028),
    (227, 0x0215_B068),
    (228, 0x0215_B084),
    (229, 0x0215_B0B4),
    (230, 0x0215_B104),
    (231, 0x0215_B138),
    (232, 0x0215_B158),
    (234, 0x0215_B190),
    (249, 0x0215_C558),
    (250, 0x0215_C584),
    (251, 0x0215_C5B0),
    (252, 0x0215_B704),
    (253, 0x0215_B74C),
    (254, 0x0215_B7E8),
    (255, 0x0215_B824),
    (257, 0x0215_B660),
    (258, 0x0215_B6C0),
    (259, 0x0215_B888),
    (260, 0x0215_B5A8),
    (264, 0x0215_BCA0),
    (266, 0x0215_BCE8),
    (267, 0x0215_BD2C),
    (268, 0x0215_BB50),
    (269, 0x0215_B848),
    (270, 0x0215_BBD0),
    (271, 0x0215_C2D0),
    (272, 0x0215_C138),
    (273, 0x0215_C0D0),
    (274, 0x0215_B5BC),
    (275, 0x0215_BE24),
    (276, 0x0215_B904),
    (277, 0x0215_BAB0),
    (278, 0x0215_BAF4),
    (279, 0x0215_BF30),
    (280, 0x0215_BA18),
    (281, 0x0215_C01C),
    (282, 0x0215_C064),
    (283, 0x0215_B558),
    (284, 0x0215_BF7C),
    (289, 0x0215_B994),
    (290, 0x0215_C180),
    (291, 0x0215_C200),
    (312, 0x0215_ACC8),
    (330, 0x0215_C5EC),
    (331, 0x0215_C610),
    (378, 0x0215_B408),
    (379, 0x0215_B438),
    (464, 0x0215_AFE0),
    (465, 0x0215_AFF4),
    (466, 0x0215_B014),
    (488, 0x0215_BFD4),
    (490, 0x0215_B214),
    (491, 0x0215_B26C),
    (492, 0x0215_B2C4),
    (562, 0x0215_B464),
    (563, 0x0215_B488),
];

// ---- layouts (spec header)

/// SCRCMD CTX size (`0x0215A55C`).
pub const CTX_SIZE: u32 = 0x24;
/// SCRCMD CTX field: the message data (`0x0215A760`).
pub const CTX_MSGDATA: u32 = 0x14;
/// SCRCMD CTX field: u16 message bank.
pub const CTX_MSG_BANK: u32 = 0x18;
/// SCRCMD CTX field: u8 VM slot command 28 waits for.
pub const CTX_WAIT_SLOT: u32 = 0x1A;
/// SCRCMD CTX field: the saved script vars (`script_sys::save_vars`).
pub const CTX_SAVED_VARS: u32 = 0x1C;
/// SCRCMD CTX field: -> SCRCMD WORK.
pub const CTX_WORK: u32 = 0x20;
/// SCRCMD WORK size (`0x0215A514`).
pub const WORK_SIZE: u32 = 0x110;
/// SCRCMD WORK field: the 8 movement-object slots (u32 each).
pub const WORK_SLOTS: u32 = 0x14;
/// Number of movement-object slots.
pub const SLOT_COUNT: u32 = 8;
/// SCRCMD WORK field: the choice menu (MENU, 0xA0 bytes).
pub const WORK_MENU: u32 = 0x34;
/// SCRCMD WORK field: the window / message object of the overlay-21 message commands.
pub const WORK_WINDOW: u32 = 0xFC;
/// SCRCMD WORK field: u16 frame countdown of command 3.
pub const WORK_COUNTDOWN: u32 = 0x106;
/// MENU field: -> u16 result var.
pub const MENU_RESULT_VAR: u32 = 8;
/// MENU field: the WORDSET.
pub const MENU_WORDSET: u32 = 0xC;
/// MENU field: 1 = the menu created its own message data.
pub const MENU_OWN_MSGDATA: u32 = 0x10;
/// MENU field: the message data.
pub const MENU_MSGDATA: u32 = 0x14;
/// MENU field: the item list (`0x02190A20`).
pub const MENU_LIST: u32 = 0x18;
/// MENU field: the list-menu handle (`0x02190988`).
pub const MENU_HANDLE: u32 = 0x1C;
/// MENU field: 32 help-text STRBUF slots.
pub const MENU_HELP: u32 = 0x20;
/// Number of help-text slots (and of items with help text).
pub const MENU_HELP_SLOTS: u32 = 0x20;

/// `"scrcmd_work.c"` (overlay 10 rodata), the file argument of the two allocations.
pub const FILE_SCRCMD_WORK: u32 = 0x0217_28E0;
/// The cursor-move callback the menu installs (`0x0215A9C1` = [`menu_cursor_cb`]).
pub const MENU_CURSOR_CB: u32 = 0x0215_A9C1;
/// The RAM word with the message data command 271 takes the egg nickname from.
pub const EGG_NAME_MSGDATA: u32 = 0x020A_7370;
/// The version byte at main 0x0209DF08 (0x15) that command 275 compares a mon's version with.
pub const VERSION: u8 = 0x15;
/// ROM address of [`VERSION`].
pub const VERSION_ADDR: u32 = 0x0209_DF08;

// ---- ROM tables (overlay 10 data)

/// The 0x20-byte list-menu header template at 0x02171140 that [`menu_open`] copies.
pub const MENU_HEADER: [u8; 0x20] = [
    0x01, 0x00, 0x06, 0x00, 0x00, 0x0D, 0x00, 0x10, 0x2F, 0x00, 0x08, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x0C, 0x00, 0x0D, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
/// ROM address of [`MENU_HEADER`].
pub const MENU_HEADER_ADDR: u32 = 0x0217_1140;
/// The u16 table of command 562 (overlay 10 data at 0x02171160, 8 entries; meaning unknown).
pub const TABLE_562: [u16; 8] = [0x0F, 0x37, 0x40, 0x1E, 0x35, 0x2B, 0x9A, 0x0D];
/// ROM address of [`TABLE_562`].
pub const TABLE_562_ADDR: u32 = 0x0217_1160;
/// The u16 table of command 563 (0x02171170, 8 entries; meaning unknown).
pub const TABLE_563: [u16; 8] = [0x11D, 0x219, 0x217, 0x119, 0x60, 0x21A, 0x218, 0x11C];
/// ROM address of [`TABLE_563`].
pub const TABLE_563_ADDR: u32 = 0x0217_1170;
/// The u16 table of command 281 (0x02171180, 2 entries before the [`PARAM_WHITELIST`] bytes).
pub const TABLE_281: [u16; 2] = [0x33, 0x605];
/// ROM address of [`TABLE_281`].
pub const TABLE_281_ADDR: u32 = 0x0217_1180;
/// The PokeParam ids command 272 may read (0x02171182, 12 bytes).
pub const PARAM_WHITELIST: [u8; 12] = [
    0x05, 0x06, 0x0C, 0x6D, 0x6E, 0x6F, 0x70, 0x75, 0x77, 0x99, 0x9A, 0x9E,
];
/// ROM address of [`PARAM_WHITELIST`].
pub const PARAM_WHITELIST_ADDR: u32 = 0x0217_1182;
/// `{u16 param, u16 max}` x 6 at 0x0217118E (command 273: the IV params 0x46..0x4B, max 31).
pub const IV_TABLE: [(u16, u16); 6] = [
    (0x46, 31),
    (0x47, 31),
    (0x48, 31),
    (0x49, 31),
    (0x4A, 31),
    (0x4B, 31),
];
/// ROM address of [`IV_TABLE`].
pub const IV_TABLE_ADDR: u32 = 0x0217_118E;

/// The 0x24-byte "give a Pokemon" record of commands 268 / 270 / 290 / 291 (a stack record in the original), read by
/// [`build_mon`]: `+0 u16 heap, +4 species, +8 form, +0xC level, +0x10 param 6, +0x14, +0x18, +0x1C (020186C8
/// arguments), +0x20 item / ball` (field names past species UNVERIFIED).
pub type GiveMonRec = [u8; 0x24];

fn rd32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

fn rd16(b: &[u8], o: usize) -> u32 {
    u16::from_le_bytes([b[o], b[o + 1]]) as u32
}

fn wr32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

/// Callees of the scrcmd_work functions that this area does not port, named by address (meaning per the spec; names
/// of main / overlay-21 routines are inferred, UNVERIFIED). Callees shared with the scrcmd core come from [`CoreHooks`].
pub trait WorkHooks: CoreHooks {
    // ---- scrcmd ctx getters (ported in crate::script_sys)
    /// C: `FUN_0215A5CC` (`script_sys::ctx_heap`): the ctx's heap id.
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::ctx_heap(self, ctx)
    }
    /// C: `FUN_0215A5D0` (`script_sys::ctx_zone`): the ctx's zone id.
    fn fun_0215a5d0(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::ctx_zone(self, ctx)
    }

    // ---- overlay 10 outside the area
    /// C: `FUN_0215898C` (`script_sys::sw_set_end_callback`): end callback argument of the script's SYS.
    fn fun_0215898c(&mut self, sw: u32, arg: u32);
    /// C: `FUN_02158F54` (`script_sys::sw_gamesys`): the GAMESYS of a SCRIPT_WORK.
    fn fun_02158f54(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F58` (`script_sys::sw_field_block`): address of the SW +0x18 field block.
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F6C` (`script_sys::sw_scrcmd_work`): the SCRCMD WORK of a SCRIPT_WORK.
    fn fun_02158f6c(&mut self, sw: u32) -> u32;
    /// C: `FUN_overlay_d_10__0215909c` (`script_sys::save_vars`): a heap copy of the script vars.
    fn fun_0215909c(&mut self, sw: u32) -> u32;
    /// C: `FUN_overlay_d_10__021590d4` (`script_sys::restore_vars`): restore them from `buf` and free it.
    fn fun_021590d4(&mut self, sw: u32, buf: u32);
    /// C: `FUN_overlay_d_10__02159198` (`script_sys::proc_mask_test`, reached through the thunk `0x02159190`): bit
    /// `bit` of the init-proc mask.
    fn fun_02159198(&mut self, bit: u32) -> u32;
    /// C: `FUN_02159B94(x, matrix, zone, heap)` (map_matrix, black-field F2a): load the zone's map matrix.
    fn fun_02159b94(&mut self, x: u32, matrix: u32, zone: u32, heap: u32);
    /// C: `FUN_02159CC4(x, gamesys, heap)` (map_matrix): apply it.
    fn fun_02159cc4(&mut self, x: u32, gamesys: u32, heap: u32);
    /// C: `FUN_02159F28(gamedata, a, b)` (command 218; UNVERIFIED meaning).
    fn fun_02159f28(&mut self, gamedata: u32, a: u32, b: u32);
    /// C: `FUN_02159F54(gamedata, v)` (command 216).
    fn fun_02159f54(&mut self, gamedata: u32, v: u32) -> u32;
    /// C: `FUN_02159F84(v)` (command 217: 1 = valid).
    fn fun_02159f84(&mut self, v: u32) -> u32;
    /// C: `FUN_0215A4D8(ew, v)` (command 151).
    fn fun_0215a4d8(&mut self, ew: u32, v: u32) -> u32;
    /// C: `FUN_0215A4EC(ew, v)` (command 149).
    fn fun_0215a4ec(&mut self, ew: u32, v: u32);
    /// C: `FUN_0215A500(ew, v)` (command 150).
    fn fun_0215a500(&mut self, ew: u32, v: u32);
    /// C: `FUN_0215F6E8(gamesys)` (commands 140 / 378: the end-callback argument).
    fn fun_0215f6e8(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02162708(x, a, b, c, d)` (command 491).
    fn fun_02162708(&mut self, x: u32, a: u32, b: u32, c: u32, d: u32);
    /// C: `FUN_02162748(x, a, b, c, d, e)` (command 492).
    fn fun_02162748(&mut self, x: u32, a: u32, b: u32, c: u32, d: u32, e: u32);
    /// C: `FUN_02162A74(x, a, b, c, d)` (command 490).
    fn fun_02162a74(&mut self, x: u32, a: u32, b: u32, c: u32, d: u32);
    /// C: `FUN_021690B4(gamedata)` (command 205).
    fn fun_021690b4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_021690D0(gamedata)` (command 208).
    fn fun_021690d0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_021690E0(gamedata)` (command 208).
    fn fun_021690e0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_021690F0(gamedata)` (command 207).
    fn fun_021690f0(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02169100(gamedata)` (command 209).
    fn fun_02169100(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02169110(gamedata)` (command 209).
    fn fun_02169110(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0216EFA8(x, v)` (commands 141 / 379).
    fn fun_0216efa8(&mut self, x: u32, v: u32) -> u32;
    /// C: `FUN_0216F830(gamedata)` (command 206).
    fn fun_0216f830(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0216F948(mon, heap)` (command 488: a heap object made from the mon).
    fn fun_0216f948(&mut self, mon: u32, heap: u32) -> u32;
    /// C: `FUN_0216FAA0(x)` (command 488).
    fn fun_0216faa0(&mut self, x: u32) -> u32;

    // ---- overlay 21 (list menu, message window, movement objects)
    /// C: `FUN_02190988(fieldmap, header, list, callback, ctx, 0, cursor, flag)`: open the list menu; returns its handle.
    #[allow(clippy::too_many_arguments)]
    fn fun_02190988(
        &mut self,
        fm: u32,
        header: &[u8; 0x20],
        list: u32,
        cb: u32,
        ctx: u32,
        a: u32,
        cursor: u32,
        flag: u32,
    ) -> u32;
    /// C: `FUN_021909BC(handle)`: close the list menu.
    fn fun_021909bc(&mut self, handle: u32);
    /// C: `FUN_021909E4(handle)`: poll it (-1 open, -2 cancelled, else the result).
    fn fun_021909e4(&mut self, handle: u32) -> u32;
    /// C: `FUN_02190A20(n, heap)`: create an item list of `n` entries.
    fn fun_02190a20(&mut self, n: u32, heap: u32) -> u32;
    /// C: `FUN_02190A64(list, text, param, heap)`: add an item.
    fn fun_02190a64(&mut self, list: u32, text: u32, param: u32, heap: u32);
    /// C: `FUN_02190AC4(list)`: the item count.
    fn fun_02190ac4(&mut self, list: u32) -> u32;
    /// C: `FUN_02190ACC(fieldmap, list, a, b)`: the menu width.
    fn fun_02190acc(&mut self, fm: u32, list: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_02190AEC(rows, a, b, scroll)`: the menu height.
    fn fun_02190aec(&mut self, rows: u32, a: u32, b: u32, scroll: u32) -> u32;
    /// C: `FUN_02190B08(header, n, x, y, w, h)`: lay out the header (a stack copy; the original callee may write it).
    fn fun_02190b08(&mut self, header: &mut [u8; 0x20], n: u32, x: u32, y: u32, w: u32, h: u32);
    /// C: `FUN_02191164(window)`: clear the message window.
    fn fun_02191164(&mut self, window: u32);
    /// C: `FUN_02191184(window, x, y, text)`: print `text` in it.
    fn fun_02191184(&mut self, window: u32, x: u32, y: u32, text: u32);
    /// C: `FUN_02197DA0(obj)`: the movement command object finished (1).
    fn fun_02197da0(&mut self, obj: u32) -> u32;
    /// C: `FUN_02197DAC(obj)`: delete it.
    fn fun_02197dac(&mut self, obj: u32);

    // ---- main: heap / text
    /// C: `FUN_02030734(heap, size, flag, file, line)` (`Heap_AllocDebug`).
    fn fun_02030734(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32;
    /// C: `FUN_020307B0(block)` (`Heap_Free`; also the veneer 0x0215A554).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_0201F250(wordset, dst, src)`: expand the `{..}` tags of `src` into `dst`.
    fn fun_0201f250(&mut self, wordset: u32, dst: u32, src: u32);
    /// C: `FUN_0201F91C(list)`: the item count of a list (help-text slot index).
    fn fun_0201f91c(&mut self, list: u32) -> u32;
    /// C: `FUN_020202EC(menu, &idx)`: the cursor index (u16 out).
    fn fun_020202ec(&mut self, menu: u32, idx: &mut [u8; 2]);
    /// C: `FUN_02020D0C(menu, 0)`: the menu's callback argument (the ctx).
    fn fun_02020d0c(&mut self, menu: u32, a: u32) -> u32;
    /// C: `FUN_020457B0(len, heap)`: create a STRBUF.
    fn fun_020457b0(&mut self, len: u32, heap: u32) -> u32;
    /// C: `FUN_02045808(strbuf)`: free a STRBUF.
    fn fun_02045808(&mut self, strbuf: u32);
    /// C: `FUN_02045874(strbuf, heap)`: a heap copy of a STRBUF.
    fn fun_02045874(&mut self, strbuf: u32, heap: u32) -> u32;
    /// C: `FUN_0204589C(a, b)`: compare two STRBUFs (nonzero = equal, as command 275 uses it).
    fn fun_0204589c(&mut self, a: u32, b: u32) -> u32;
    /// C: `FUN_02045B38(0, type, msg, heap)` (`MsgData_Create`).
    fn fun_02045b38(&mut self, a: u32, ty: u32, msg: u32, heap: u32) -> u32;
    /// C: `FUN_02045C04(msgdata)`: free message data.
    fn fun_02045c04(&mut self, msgdata: u32);
    /// C: `FUN_02045CAC(msgdata, id, strbuf)`: load message `id` into a STRBUF.
    fn fun_02045cac(&mut self, msgdata: u32, id: u32, strbuf: u32);
    /// C: `FUN_02045EC0(msgdata, id)`: message `id` as a new STRBUF.
    fn fun_02045ec0(&mut self, msgdata: u32, id: u32) -> u32;

    // ---- main: game data
    /// C: `FUN_020056EC(v)` (command 203).
    fn fun_020056ec(&mut self, v: u32) -> u32;
    /// C: `FUN_020071E4(x)` (command 312).
    fn fun_020071e4(&mut self, x: u32) -> u32;
    /// C: `FUN_020071EC(x)` (command 312).
    fn fun_020071ec(&mut self, x: u32) -> u32;
    /// C: `FUN_0200723C(x, &a, &b)` (command 312: two u32 outs).
    fn fun_0200723c(&mut self, x: u32, a: &mut [u8; 4], b: &mut [u8; 4]);
    /// C: `FUN_0200725C(x, 8, 4)` (command 234).
    fn fun_0200725c(&mut self, x: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_020072A4(x, 8)` (command 234).
    fn fun_020072a4(&mut self, x: u32, a: u32);
    /// C: `FUN_020073D8(x, 8, 0)` (command 234).
    fn fun_020073d8(&mut self, x: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_020074D8(box, mon)`: deposit a mon in the PC boxes.
    fn fun_020074d8(&mut self, boxes: u32, mon: u32);
    /// C: `FUN_02007984(box)`: number of mons in the PC boxes.
    fn fun_02007984(&mut self, boxes: u32) -> u32;
    /// C: `FUN_020079A0(box)` (command 289).
    fn fun_020079a0(&mut self, boxes: u32) -> u32;
    /// C: `FUN_02007A68(box, v)` (command 228).
    fn fun_02007a68(&mut self, boxes: u32, v: u32);
    /// C: `FUN_020083E4(x)`: an option value (commands 201 / 202).
    fn fun_020083e4(&mut self, x: u32) -> u32;
    /// C: `FUN_020083EC(x, v)`: set it.
    fn fun_020083ec(&mut self, x: u32, v: u32);
    /// C: `FUN_02008428(x)` (command 226).
    fn fun_02008428(&mut self, x: u32) -> u32;
    /// C: `FUN_02008500(player, strbuf)`: the player name into a STRBUF.
    fn fun_02008500(&mut self, player: u32, strbuf: u32);
    /// C: `FUN_02008510(player, heap)`: the player name as a new STRBUF.
    fn fun_02008510(&mut self, player: u32, heap: u32) -> u32;
    /// C: `FUN_02008530(player)`: the player's trainer id.
    fn fun_02008530(&mut self, player: u32) -> u32;
    /// C: `FUN_02008550(player)`: the player's gender (also command 225).
    fn fun_02008550(&mut self, player: u32) -> u32;
    /// C: `FUN_02008554(x)` (command 229).
    fn fun_02008554(&mut self, x: u32) -> u32;
    /// C: `FUN_0200873C(x)`: the options record (commands 201 / 202 / 226).
    fn fun_0200873c(&mut self, x: u32) -> u32;
    /// C: `FUN_020089C0(x)` (command 229).
    fn fun_020089c0(&mut self, x: u32) -> u32;
    /// C: `FUN_020089F4(x)` (command 229).
    fn fun_020089f4(&mut self, x: u32) -> u32;
    /// C: `FUN_02008A00(x)` (command 229).
    fn fun_02008a00(&mut self, x: u32);
    /// C: `FUN_02008A7C(x, v, t0, t1, t2)` (command 214).
    fn fun_02008a7c(&mut self, x: u32, v: u32, t0: u32, t1: u32, t2: u32);
    /// C: `FUN_0200C234(x, v)` (command 213).
    fn fun_0200c234(&mut self, x: u32, v: u32) -> u32;
    /// C: `FUN_0200C248(x, v)` (command 214).
    fn fun_0200c248(&mut self, x: u32, v: u32);
    /// C: `FUN_0200C258(x)` (command 215).
    fn fun_0200c258(&mut self, x: u32) -> u32;
    /// C: `FUN_0200C274(x)` (command 251).
    fn fun_0200c274(&mut self, x: u32) -> u32;
    /// C: `FUN_0200C278(x, v)` (command 249).
    fn fun_0200c278(&mut self, x: u32, v: u32);
    /// C: `FUN_0200C29C(x, v)` (command 250).
    fn fun_0200c29c(&mut self, x: u32, v: u32);
    /// C: `FUN_0200C2F0(x, v, 1)` (command 231).
    fn fun_0200c2f0(&mut self, x: u32, v: u32, a: u32);
    /// C: `FUN_0200C308(x, v)` (commands 230 / 232).
    fn fun_0200c308(&mut self, x: u32, v: u32) -> u32;
    /// C: `FUN_0200CA1C(gamedata)`: the Pokedex.
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200CA28(dex)` (command 464).
    fn fun_0200ca28(&mut self, dex: u32);
    /// C: `FUN_0200CA38(dex)` (command 465).
    fn fun_0200ca38(&mut self, dex: u32) -> u32;
    /// C: `FUN_0200CA40(dex)` (command 466).
    fn fun_0200ca40(&mut self, dex: u32);
    /// C: `FUN_0200CDE0(dex, mon)`: register a caught mon.
    fn fun_0200cde0(&mut self, dex: u32, mon: u32);
    /// C: `FUN_0200CED4(dex, v)` (command 223 mode 1).
    fn fun_0200ced4(&mut self, dex: u32, v: u32) -> u32;
    /// C: `FUN_0200CF28(dex)` (command 221 mode 1).
    fn fun_0200cf28(&mut self, dex: u32) -> u32;
    /// C: `FUN_0200CFA0(dex, mon)`: register a seen mon.
    fn fun_0200cfa0(&mut self, dex: u32, mon: u32);
    /// C: `FUN_0200D064(dex, v)` (command 223 mode 0).
    fn fun_0200d064(&mut self, dex: u32, v: u32) -> u32;
    /// C: `FUN_0200D0CC(dex)` (command 221 mode 0).
    fn fun_0200d0cc(&mut self, dex: u32) -> u32;
    /// C: `FUN_0200ECB4(x)` (command 234).
    fn fun_0200ecb4(&mut self, x: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)`: the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020127A4(gamedata)`: the location block (GAMEDATA + 0x114).
    fn fun_020127a4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012844(gamedata)` (commands 490..492).
    fn fun_02012844(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201289C(gamedata, &rec)`: store a 0x1C-byte location record.
    fn fun_0201289c(&mut self, gamedata: u32, rec: &[u8; 0x1C]);
    /// C: `FUN_02012900(gamedata, v)` (command 217).
    fn fun_02012900(&mut self, gamedata: u32, v: u32);
    /// C: `FUN_02012934(gamedata)`: the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201293C(gamedata)`: the PC boxes.
    fn fun_0201293c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)`: the player record.
    fn fun_02012944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012964(gamedata)`: SCRCMD WORK +0xC.
    fn fun_02012964(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012974(gamedata)` (command 218: the map matrix object).
    fn fun_02012974(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012984(gamedata)` (command 210).
    fn fun_02012984(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012A44(gamedata)` (commands 141 / 379).
    fn fun_02012a44(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012AA0(loc)`: the 3-word position of a location block.
    fn fun_02012aa0(&mut self, loc: u32) -> u32;
    /// C: `FUN_02012AC8(loc)`: its zone id.
    fn fun_02012ac8(&mut self, loc: u32) -> u32;
    /// C: `FUN_02012AE4(loc)`: its u16 direction (command 219).
    fn fun_02012ae4(&mut self, loc: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)` (config / misc record).
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F08(gamedata)` (commands 213..215, 230..232, 249..251).
    fn fun_02012f08(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F5C(gamedata)` (command 214).
    fn fun_02012f5c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02013B44(zone)`: the zone's map matrix id.
    fn fun_02013b44(&mut self, zone: u32) -> u32;
    /// C: `FUN_02013E4C(zone)`: the zone's location-name id.
    fn fun_02013e4c(&mut self, zone: u32) -> u32;
    /// C: `FUN_02014184(&rec, zone, dir, x, y, z)`: build a 0x1C-byte location record.
    fn fun_02014184(&mut self, rec: &mut [u8; 0x1C], zone: u32, dir: u32, x: u32, y: u32, z: u32);
    /// C: `FUN_020144DC(gamesys, fieldmap)`: the child event of command 331.
    fn fun_020144dc(&mut self, gamesys: u32, fm: u32) -> u32;
    /// C: `FUN_020145CC(gamesys)`: the child event of command 330.
    fn fun_020145cc(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020173F0(species, level, a, b, heap)`: create a PokeParam.
    fn fun_020173f0(&mut self, species: u32, level: u32, a: u32, b: u32, heap: u32) -> u32;
    /// C: `FUN_02017430(species, level, ot id, a, b, c, d, heap)`: create a PokeParam with more fields.
    #[allow(clippy::too_many_arguments)]
    fn fun_02017430(
        &mut self,
        species: u32,
        level: u32,
        ot: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        heap: u32,
    ) -> u32;
    /// C: `FUN_02017988(mon, v)` (command 279).
    fn fun_02017988(&mut self, mon: u32, v: u32);
    /// C: `FUN_02017A20(mon, a, b)` (command 284).
    fn fun_02017a20(&mut self, mon: u32, a: u32, b: u32);
    /// C: `FUN_02017D30(mon)`: begin fast access (returns the old state).
    fn fun_02017d30(&mut self, mon: u32) -> u32;
    /// C: `FUN_02017D70(mon, k)`: end it.
    fn fun_02017d70(&mut self, mon: u32, k: u32);
    /// C: `FUN_02017E1C(mon, param, out)`: PokeParam get.
    fn fun_02017e1c(&mut self, mon: u32, param: u32, out: u32) -> u32;
    /// C: `FUN_02017E40(mon, param, v)`: PokeParam set.
    fn fun_02017e40(&mut self, mon: u32, param: u32, v: u32);
    /// C: `FUN_020180F4(mon, move)`: learn a move (push out the oldest).
    fn fun_020180f4(&mut self, mon: u32, mv: u32);
    /// C: `FUN_020181F4(mon, move, slot)`: put a move in a slot.
    fn fun_020181f4(&mut self, mon: u32, mv: u32, slot: u32);
    /// C: `FUN_020185C4(mon)`: recalculate the stats.
    fn fun_020185c4(&mut self, mon: u32);
    /// C: `FUN_020185F4(mon)`: the box part of a party mon.
    fn fun_020185f4(&mut self, mon: u32) -> u32;
    /// C: `FUN_020186C8(ot id, species, form, a, b, c)`: a personality value.
    fn fun_020186c8(&mut self, ot: u32, species: u32, form: u32, a: u32, b: u32, c: u32) -> u32;
    /// C: `FUN_0201A98C(party, mon)`: add a mon to the party.
    fn fun_0201a98c(&mut self, party: u32, mon: u32);
    /// C: `FUN_0201AA14(party)`: the party capacity.
    fn fun_0201aa14(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AA18(party)`: the party count.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AA1C(party)` (command 259 case 1).
    fn fun_0201aa1c(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AA50(party)` (command 259 case 2).
    fn fun_0201aa50(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AA88(party)` (command 259 case 3).
    fn fun_0201aa88(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AAD4(party)` (command 259 case 4).
    fn fun_0201aad4(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB08(party)` (command 269).
    fn fun_0201ab08(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB18(party)` (command 269).
    fn fun_0201ab18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)`: party mon `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_0201ADE4(species, form, heap)`: load a personal record.
    fn fun_0201ade4(&mut self, species: u32, form: u32, heap: u32) -> u32;
    /// C: `FUN_0201AE14(pd)`: free it.
    fn fun_0201ae14(&mut self, pd: u32);
    /// C: `FUN_0201AE1C(pd, field)`: a personal field (0x15 = hatch cycles).
    fn fun_0201ae1c(&mut self, pd: u32, field: u32) -> u32;
    /// C: `FUN_0201BA34(party)` (command 260: heal the party, UNVERIFIED).
    fn fun_0201ba34(&mut self, party: u32);
    /// C: `FUN_02020FE4(item, field, heap)`: an item parameter.
    fn fun_02020fe4(&mut self, item: u32, field: u32, heap: u32) -> u32;
    /// C: `FUN_0202142C(item)`: the ball id of an item.
    fn fun_0202142c(&mut self, item: u32) -> u32;
    /// C: `FUN_02025C88(x)` (command 229).
    fn fun_02025c88(&mut self, x: u32) -> u32;
    /// C: `FUN_0202F190(mon, a, player, b, heap)`: set the met data.
    fn fun_0202f190(&mut self, mon: u32, a: u32, player: u32, b: u32, heap: u32);
    /// C: `FUN_020362DC()`: the held pad bits (`ScriptHost::pad_held`).
    fn fun_020362dc(&mut self) -> u32;
    /// C: `FUN_0203F4E4(&t)`: a 12-byte time record (command 214).
    fn fun_0203f4e4(&mut self, t: &mut [u8; 12]);
    /// C: `FUN_02087D28(&t)`: the RTC time (command 212; 8 bytes read of the original 0x58-byte stack record).
    fn fun_02087d28(&mut self, t: &mut [u8; 8]);
}

fn work<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    h.r32(ctx + CTX_WORK)
}

fn menu<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    work(h, ctx) + WORK_MENU
}

// ---- SCRCMD WORK / CTX

/// C: `FUN_overlay_d_10__0215a514`: create the SCRCMD WORK of `sw`: `Heap_AllocDebug(heap, 0x110, 1, "scrcmd_work.c",
/// 0x78)`, +0 SW, +4 GAMESYS, +8 GAMEDATA, +0xC `FUN_02012964(GAMEDATA)`, u8 +0x104 = 7; returns it.
pub fn work_new<H: WorkHooks>(h: &mut H, sw: u32, heap: u32) -> u32 {
    let w = h.fun_02030734(heap, WORK_SIZE, 1, FILE_SCRCMD_WORK, 0x78);
    h.w32(w, sw);
    let gs = h.fun_02158f54(sw);
    h.w32(w + 4, gs);
    let gd = h.fun_020120f4(gs);
    h.w32(w + 8, gd);
    let x = h.fun_02012964(gd);
    h.w32(w + 0xC, x);
    h.w8(w + 0x104, 7);
    w
}

/// C: `FUN_overlay_d_10__0215a55c`: create a SCRCMD CTX: `Heap_AllocDebug((heap & 0x7FFF) | 0x8000, 0x24, 1,
/// "scrcmd_work.c", 0x94)`, u16 +0 heap, +4..+0x14 the init record `{u16 zone, u16 script id, mode, mode != 0, SW}`, +0x20
/// = `FUN_02158F6C(SW)` (the SCRCMD WORK); returns it.
pub fn ctx_new<H: WorkHooks>(h: &mut H, init: &[u8; 16], heap: u32) -> u32 {
    let hp = ((heap & 0x7FFF) | 0x8000) & 0xFFFF;
    let c = h.fun_02030734(hp, CTX_SIZE, 1, FILE_SCRCMD_WORK, 0x94);
    h.w16(c, heap as u16);
    for (i, &b) in init.iter().enumerate() {
        h.w8(c + 4 + i as u32, b);
    }
    let w = h.fun_02158f6c(rd32(init, 0xC));
    h.w32(c + CTX_WORK, w);
    c
}

/// C: `FUN_overlay_d_10__0215a5a4`: free a ctx: the saved vars (`Heap_Free`) and the message data (`FUN_02045C04`) when
/// set, the movement objects ([`slots_clear`]), the ctx.
pub fn ctx_free<H: WorkHooks>(h: &mut H, ctx: u32) {
    let v = h.r32(ctx + CTX_SAVED_VARS);
    if v != 0 {
        h.fun_020307b0(v);
    }
    let m = h.r32(ctx + CTX_MSGDATA);
    if m != 0 {
        h.fun_02045c04(m);
    }
    slots_clear(h, ctx);
    h.fun_020307b0(ctx);
}

/// C: `FUN_overlay_d_10__0215a5fc`: the first word of the SW's field block (`*FUN_02158F58(SW)` = the
/// `FUN_02188C54(fieldmap)` value).
pub fn field_block_0<H: WorkHooks>(h: &mut H, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let p = h.fun_02158f58(sw);
    h.r32(p)
}

/// C: `FUN_0215A60C`: the ctx's message data (+0x14).
pub fn ctx_msgdata<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    h.r32(ctx + CTX_MSGDATA)
}

/// C: `FUN_0215A610`: the ctx's message bank (u16 +0x18).
pub fn ctx_msg_bank<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    h.r16(ctx + CTX_MSG_BANK) as u32
}

/// C: `FUN_0215A614`: SCRCMD WORK +0xFC (the message window object) = `v`.
pub fn set_window<H: MemEnv>(h: &mut H, ctx: u32, v: u32) {
    let w = work(h, ctx);
    h.w32(w + WORK_WINDOW, v);
}

/// C: `FUN_0215A61C`: SCRCMD WORK +0xFC.
pub fn window<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let w = work(h, ctx);
    h.r32(w + WORK_WINDOW)
}

/// C: `FUN_0215A624`: SCRCMD WORK +0x100 = `v`.
pub fn set_work_100<H: MemEnv>(h: &mut H, ctx: u32, v: u32) {
    let w = work(h, ctx);
    h.w32(w + 0x100, v);
}

/// C: `FUN_0215A630`: SCRCMD WORK +0x100.
pub fn work_100<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let w = work(h, ctx);
    h.r32(w + 0x100)
}

/// C: `FUN_0215A63C`: address of SCRCMD WORK +0xD4.
pub fn work_d4_addr<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    work(h, ctx) + 0xD4
}

/// C: `FUN_0215A644`: SCRCMD WORK +0x10 = `v`.
pub fn set_work_10<H: MemEnv>(h: &mut H, ctx: u32, v: u32) {
    let w = work(h, ctx);
    h.w32(w + 0x10, v);
}

/// C: `FUN_0215A64C`: SCRCMD WORK +0x10.
pub fn work_10<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let w = work(h, ctx);
    h.r32(w + 0x10)
}

/// C: `FUN_0215A654`: the u8 at SCRCMD WORK +0x104 (7 after [`work_new`]).
pub fn work_104<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let w = work(h, ctx);
    h.r8(w + 0x104) as u32
}

/// C: `FUN_0215A660`: u8 SCRCMD WORK +0x104 = `v`.
pub fn set_work_104<H: MemEnv>(h: &mut H, ctx: u32, v: u32) {
    let w = work(h, ctx);
    h.w8(w + 0x104, v as u8);
}

/// C: `FUN_0215A66C`: the u8 at SCRCMD WORK +0x105.
pub fn work_105<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let w = work(h, ctx);
    h.r8(w + 0x105) as u32
}

/// C: `FUN_0215A678`: u8 SCRCMD WORK +0x105 = `v`.
pub fn set_work_105<H: MemEnv>(h: &mut H, ctx: u32, v: u32) {
    let w = work(h, ctx);
    h.w8(w + 0x105, v as u8);
}

/// C: `FUN_0215A684`: the frame countdown of command 3 (u16 SCRCMD WORK +0x106) = `n`.
pub fn set_countdown<H: MemEnv>(h: &mut H, ctx: u32, n: u32) {
    let w = work(h, ctx);
    h.w16(w + WORK_COUNTDOWN, n as u16);
}

/// C: `FUN_0215A690`: one frame of the countdown: 0 left -> 1 (done); else count down, 0.
pub fn countdown<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let w = work(h, ctx);
    let n = h.r16(w + WORK_COUNTDOWN);
    if n == 0 {
        return 1;
    }
    h.w16(w + WORK_COUNTDOWN, n - 1);
    0
}

/// C: `FUN_0215A6AC`: SCRCMD WORK +0x108.
pub fn work_108<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let w = work(h, ctx);
    h.r32(w + 0x108)
}

/// C: `FUN_0215A6B8`: SCRCMD WORK +0x108 = `v`.
pub fn set_work_108<H: MemEnv>(h: &mut H, ctx: u32, v: u32) {
    let w = work(h, ctx);
    h.w32(w + 0x108, v);
}

/// C: `FUN_0215A6C4`: SCRCMD WORK +0x10C.
pub fn work_10c<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let w = work(h, ctx);
    h.r32(w + 0x10C)
}

/// C: `FUN_0215A6D0`: SCRCMD WORK +0x10C = `v`.
pub fn set_work_10c<H: MemEnv>(h: &mut H, ctx: u32, v: u32) {
    let w = work(h, ctx);
    h.w32(w + 0x10C, v);
}

/// C: `FUN_0215A6DC`: store `obj` (an overlay-21 movement command object) in the first free of the 8 slots; with none free
/// it is dropped.
pub fn add_slot_object<H: MemEnv>(h: &mut H, ctx: u32, obj: u32) {
    let w = work(h, ctx);
    for i in 0..SLOT_COUNT {
        let a = w + WORK_SLOTS + 4 * i;
        if h.r32(a) == 0 {
            h.w32(a, obj);
            return;
        }
    }
}

/// C: `FUN_overlay_d_10__0215a6fc`: the movement wait: every slot object that finished (`FUN_02197DA0 == 1`) is deleted
/// (`FUN_02197DAC`) and its slot cleared; returns 1 while any is still running.
pub fn slots_wait<H: WorkHooks>(h: &mut H, ctx: u32) -> u32 {
    let mut running = 0;
    for i in 0..SLOT_COUNT {
        let a = work(h, ctx) + WORK_SLOTS + 4 * i;
        let obj = h.r32(a);
        if obj == 0 {
            continue;
        }
        if h.fun_02197da0(obj) == 1 {
            let a = work(h, ctx) + WORK_SLOTS + 4 * i;
            let obj = h.r32(a);
            h.fun_02197dac(obj);
            let a = work(h, ctx) + WORK_SLOTS + 4 * i;
            h.w32(a, 0);
        } else {
            running = 1;
        }
    }
    running
}

/// C: `FUN_overlay_d_10__0215a738`: delete (`FUN_02197DAC`) every slot object and clear the slots.
pub fn slots_clear<H: WorkHooks>(h: &mut H, ctx: u32) {
    for i in 0..SLOT_COUNT {
        let a = work(h, ctx) + WORK_SLOTS + 4 * i;
        let obj = h.r32(a);
        if obj != 0 {
            h.fun_02197dac(obj);
            let a = work(h, ctx) + WORK_SLOTS + 4 * i;
            h.w32(a, 0);
        }
    }
}

/// C: `FUN_0215A760`: open the script's message bank: ctx +0x14 = `FUN_02045B38(0, type, msg, heap)` (`MsgData_Create`),
/// u16 +0x18 = `msg` (`ScriptHost::message_bank_open`).
pub fn message_bank_open<H: WorkHooks>(h: &mut H, ctx: u32, ty: u32, msg: u32) {
    let heap = h.r16(ctx) as u32;
    let m = h.fun_02045b38(0, ty & 0xFFFF, msg & 0xFFFF, heap);
    h.w32(ctx + CTX_MSGDATA, m);
    h.w16(ctx + CTX_MSG_BANK, msg as u16);
}

// ---- choice menu (overlay-21 menu commands; ScriptHost::menu_start / menu_result)

/// C: `FUN_overlay_d_10__0215a77c`: begin a choice menu. MENU = SCRCMD WORK +0x34 (0xA0 bytes, zeroed): +0 x, +2 y,
/// +4 cursor (u16), +6 / +7 the low bytes of `s0` / `s1`, +8 the result var address, +0xC WORDSET, +0x14 message data (0 ->
/// its own, `FUN_02045B38(0, 3, 0x106, heap)` with +0x10 = 1), +0x18 = `FUN_02190A20(0x20, heap)` (the item list).
#[allow(clippy::too_many_arguments)]
pub fn menu_begin<H: WorkHooks>(
    h: &mut H,
    ctx: u32,
    x: u32,
    y: u32,
    cursor: u32,
    s0: u32,
    s1: u32,
    var: u32,
    wordset: u32,
    msgdata: u32,
) {
    let w = work(h, ctx);
    let m = w + WORK_MENU;
    for i in 0..0xA0 {
        h.w8(m + i, 0);
    }
    h.w16(m, x as u16);
    h.w16(m + 2, y as u16);
    h.w16(m + 4, cursor as u16);
    h.w8(m + 6, s0 as u8);
    h.w8(m + 7, s1 as u8);
    h.w32(m + MENU_RESULT_VAR, var);
    h.w32(m + MENU_WORDSET, wordset);
    h.w32(m + MENU_MSGDATA, msgdata);
    for i in 0..MENU_HELP_SLOTS {
        h.w32(m + MENU_HELP + 4 * i, 0);
    }
    if h.r32(m + MENU_MSGDATA) == 0 {
        h.w32(m + MENU_OWN_MSGDATA, 1);
        let heap = h.r16(ctx) as u32;
        let md = h.fun_02045b38(0, 3, 0x106, heap);
        h.w32(m + MENU_MSGDATA, md);
    }
    let heap = h.r16(ctx) as u32;
    let list = h.fun_02190a20(0x20, heap);
    h.w32(m + MENU_LIST, list);
}

/// C: `FUN_overlay_d_10__0215a7f0`: add a menu item: with a help text (`help != 0xFFFF`) and fewer than 0x20 items, the
/// help text (`FUN_02045CAC`, expanded with the WORDSET by `FUN_0201F250` into `s0`) is copied into help slot n
/// (`FUN_02045874`); with 0x20 or more items nothing is added. Then the item text `msg` the same way and
/// `FUN_02190A64(list, s0, param, heap)`. `s0` / `s1` are the script's two STRBUFs.
pub fn menu_add<H: WorkHooks>(
    h: &mut H,
    ctx: u32,
    msg: u32,
    help: u32,
    param: u32,
    s0: u32,
    s1: u32,
) {
    let m = menu(h, ctx);
    if help != 0xFFFF {
        let list = h.r32(m + MENU_LIST);
        let n = h.fun_0201f91c(list);
        if n >= MENU_HELP_SLOTS {
            return;
        }
        let md = h.r32(m + MENU_MSGDATA);
        h.fun_02045cac(md, help, s1);
        let ws = h.r32(m + MENU_WORDSET);
        h.fun_0201f250(ws, s0, s1);
        let heap = h.r16(ctx) as u32;
        let copy = h.fun_02045874(s0, heap);
        h.w32(m + MENU_HELP + n * 4, copy);
    }
    let md = h.r32(m + MENU_MSGDATA);
    h.fun_02045cac(md, msg, s1);
    let ws = h.r32(m + MENU_WORDSET);
    h.fun_0201f250(ws, s0, s1);
    let heap = h.r16(ctx) as u32;
    let list = h.r32(m + MENU_LIST);
    h.fun_02190a64(list, s0, param, heap);
}

/// C: `FUN_overlay_d_10__0215a864`: open the menu: a stack copy of [`MENU_HEADER`], width `FUN_02190ACC(fieldmap, list,
/// hdr u16 +0x10, hdr u16 +0xA & 7)` (right-aligned (+7 == 1): x -= width), rows = min(items, 6), height
/// `FUN_02190AEC(rows, hdr u16 +0x12, (hdr u16 +0xA >> 3) & 0xF, items > 6)`, layout `FUN_02190B08`; MENU +0x1C =
/// `FUN_02190988(fieldmap, hdr, list, 0x0215A9C1, ctx, 0, cursor, +6)` (the overlay-21 list menu).
pub fn menu_open<H: WorkHooks>(h: &mut H, ctx: u32) {
    let m = menu(h, ctx);
    let mut hdr = MENU_HEADER;
    let fm = field_block_0(h, ctx);
    let h10 = rd16(&hdr, 0x10);
    let h0a = rd16(&hdr, 0xA);
    let list = h.r32(m + MENU_LIST);
    let width = h.fun_02190acc(fm, list, h10, h0a & 7);
    if h.r8(m + 7) == 1 {
        let x = h.r16(m) as u32;
        h.w16(m, x.wrapping_sub(width & 0xFFFF) as u16);
    }
    let list = h.r32(m + MENU_LIST);
    let n = h.fun_02190ac4(list);
    let rows = if n > 6 { 6 } else { n };
    let scroll = (n > 6) as u32;
    let h12 = rd16(&hdr, 0x12);
    let h0a = rd16(&hdr, 0xA);
    let height = h.fun_02190aec(rows, h12, (h0a << 25) >> 28, scroll);
    let list = h.r32(m + MENU_LIST);
    let n2 = h.fun_02190ac4(list);
    let x = h.r16(m) as u32;
    let y = h.r16(m + 2) as u32;
    h.fun_02190b08(&mut hdr, n2 & 0xFFFF, x, y, width & 0xFFFF, height & 0xFFFF);
    let cursor = h.r16(m + 4) as u32;
    let flag = h.r8(m + 6) as u32;
    let list = h.r32(m + MENU_LIST);
    let handle = h.fun_02190988(fm, &hdr, list, MENU_CURSOR_CB, ctx, 0, cursor, flag);
    h.w32(m + MENU_HANDLE, handle);
}

/// C: `FUN_overlay_d_10__0215a920`: close the menu `m` (the MENU address): `FUN_021909BC(handle)`, its own message data
/// (+0x10 == 1) freed (`FUN_02045C04`), the 32 help-text STRBUFs freed (`FUN_02045808`) and cleared.
pub fn menu_close<H: WorkHooks>(h: &mut H, m: u32) {
    let handle = h.r32(m + MENU_HANDLE);
    h.fun_021909bc(handle);
    if h.r32(m + MENU_OWN_MSGDATA) == 1 {
        let md = h.r32(m + MENU_MSGDATA);
        h.fun_02045c04(md);
    }
    for i in 0..MENU_HELP_SLOTS {
        let a = m + MENU_HELP + 4 * i;
        let s = h.r32(a);
        if s != 0 {
            h.fun_02045808(s);
            h.w32(a, 0);
        }
    }
}

/// C: `FUN_overlay_d_10__0215a954`: poll the menu: `r = FUN_021909E4(handle)`; -1 -> 0 (still open); else close it
/// ([`menu_close`]), the result var = 0xFFFE for -2 (cancelled) or `r`, and 1.
pub fn menu_poll<H: WorkHooks>(h: &mut H, ctx: u32) -> u32 {
    let m = menu(h, ctx);
    let handle = h.r32(m + MENU_HANDLE);
    let r = h.fun_021909e4(handle);
    if r == 0xFFFF_FFFF {
        return 0;
    }
    menu_close(h, m);
    let var = h.r32(m + MENU_RESULT_VAR);
    if r == 0xFFFF_FFFE {
        h.w16(var, crate::host::MENU_CANCEL);
    } else {
        h.w16(var, r as u16);
    }
    1
}

/// C: `FUN_overlay_d_10__0215a990`: [`menu_poll`], and while it is open X (`FUN_020362DC() & 0x400`) closes it with the
/// result 0xFFFD.
pub fn menu_poll_x<H: WorkHooks>(h: &mut H, ctx: u32) -> u32 {
    let m = menu(h, ctx);
    let mut r = menu_poll(h, ctx);
    if r == 0 && h.fun_020362dc() & 0x400 != 0 {
        menu_close(h, m);
        let var = h.r32(m + MENU_RESULT_VAR);
        r = 1;
        h.w16(var, crate::host::MENU_X);
    }
    r
}

/// C: `sub_0215a9c0`: the list menu's cursor-move callback (`0x0215A9C1`): ctx = `FUN_02020D0C(menu, 0)`, `idx` =
/// `FUN_020202EC`; when the item has a help text: init-proc bit 1 set -> print it in the message window
/// (`FUN_02191164`, `FUN_02191184(window, 0, 0, text)`); else test bit 2, and bit 3 when bit 2 is clear (no effect;
/// UNVERIFIED purpose).
pub fn menu_cursor_cb<H: WorkHooks>(h: &mut H, menu_obj: u32) {
    let ctx = h.fun_02020d0c(menu_obj, 0);
    let m = work(h, ctx) + WORK_MENU;
    let mut idx = [0u8; 2];
    h.fun_020202ec(menu_obj, &mut idx);
    let i = u16::from_le_bytes(idx) as u32;
    if h.r32(m + MENU_HELP + i * 4) == 0 {
        return;
    }
    if h.fun_02159198(1) != 0 {
        let win = window(h, ctx);
        h.fun_02191164(win);
        let i = u16::from_le_bytes(idx) as u32;
        let text = h.r32(m + MENU_HELP + i * 4);
        h.fun_02191184(win, 0, 0, text);
        return;
    }
    if h.fun_02159198(2) == 0 {
        h.fun_02159198(3);
    }
}

/// C: `FUN_0215AA24`: save the script vars: ctx +0x1C = `FUN_0215909C(SW)`.
pub fn save_vars<H: WorkHooks>(h: &mut H, ctx: u32) {
    let sw = h.fun_0215a5f4(ctx);
    let b = h.fun_0215909c(sw);
    h.w32(ctx + CTX_SAVED_VARS, b);
}

/// C: `FUN_0215AA34`: restore them (`FUN_021590D4(SW, ctx +0x1C)`, which frees the copy); +0x1C = 0.
pub fn restore_vars<H: WorkHooks>(h: &mut H, ctx: u32) {
    let sw = h.fun_0215a5f4(ctx);
    let b = h.r32(ctx + CTX_SAVED_VARS);
    h.fun_021590d4(sw, b);
    h.w32(ctx + CTX_SAVED_VARS, 0);
}

/// C: `FUN_0215AA48`: u8 ctx +0x1A (the VM slot command 28 waits for) = `slot`.
pub fn set_wait_slot<H: MemEnv>(h: &mut H, ctx: u32, slot: u32) {
    h.w8(ctx + CTX_WAIT_SLOT, slot as u8);
}

/// C: `FUN_0215AA4C`: u8 ctx +0x1A.
pub fn wait_slot<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    h.r8(ctx + CTX_WAIT_SLOT) as u32
}

// ---- commands: system / options / time

/// C: `sub_0215aa50`: command 224: var = 0x15 (UNVERIFIED meaning: a constant, perhaps the game version id).
pub fn cmd_var_set_15<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    h.w16(a, 0x15);
    0
}

/// The options record `FUN_0200873C(FUN_02012EBC(gamedata))`.
fn options<H: WorkHooks>(h: &mut H, gd: u32) -> u32 {
    let x = h.fun_02012ebc(gd);
    h.fun_0200873c(x)
}

/// C: `sub_0215aa60`: command 201: flip the option `FUN_020083E4` (0 <-> 1, other values kept; UNVERIFIED meaning).
pub fn cmd_201<H: WorkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let x = options(h, gd);
    let v = h.fun_020083e4(x);
    let nv = match v {
        0 => 1,
        1 => 0,
        v => v,
    };
    h.fun_020083ec(x, nv);
    0
}

/// C: `sub_0215aa90`: command 202: var = that option.
pub fn cmd_202<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let x = options(h, gd);
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_020083e4(x);
    h.w16(a, v as u16);
    0
}

/// C: `sub_0215aabc`: command 203: var = `FUN_020056EC(value)`.
pub fn cmd_203<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let r = h.fun_020056ec(v);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215aadc`: command 204: var = the ctx's message bank ([`ctx_msg_bank`]).
pub fn cmd_204<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let r = ctx_msg_bank(h, ctx);
    h.w16(a, r as u16);
    0
}

/// var = `f(GAMEDATA)` (commands 205..207; the GAMEDATA is read before the var operand).
fn var_from_gd<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32, f: fn(&mut H, u32) -> u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let r = f(h, gd);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215aaf4`: command 206: var = `FUN_0216F830(GAMEDATA)`.
pub fn cmd_206<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    var_from_gd(h, vm, ctx, |h, gd| h.fun_0216f830(gd))
}

/// C: `sub_0215ab18`: command 205: var = `FUN_021690B4(GAMEDATA)`.
pub fn cmd_205<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    var_from_gd(h, vm, ctx, |h, gd| h.fun_021690b4(gd))
}

/// C: `sub_0215ab3c`: command 207: var = `FUN_021690F0(GAMEDATA)`.
pub fn cmd_207<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    var_from_gd(h, vm, ctx, |h, gd| h.fun_021690f0(gd))
}

/// var1 = `f(GAMEDATA)`, var2 = `g(GAMEDATA)` (commands 208 / 209).
fn vars2_from_gd<H: WorkHooks>(
    h: &mut H,
    vm: u32,
    ctx: u32,
    f: fn(&mut H, u32) -> u32,
    g: fn(&mut H, u32) -> u32,
) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let a1 = h.fun_02159ae8(vm, ctx);
    let a2 = h.fun_02159ae8(vm, ctx);
    let r = f(h, gd);
    h.w16(a1, r as u16);
    let r = g(h, gd);
    h.w16(a2, r as u16);
    0
}

/// C: `sub_0215ab60`: command 208: var1 = `FUN_021690D0(GAMEDATA)`, var2 = `FUN_021690E0(GAMEDATA)`.
pub fn cmd_208<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    vars2_from_gd(
        h,
        vm,
        ctx,
        |h, gd| h.fun_021690d0(gd),
        |h, gd| h.fun_021690e0(gd),
    )
}

/// C: `sub_0215ab98`: command 209: var1 = `FUN_02169100(GAMEDATA)`, var2 = `FUN_02169110(GAMEDATA)`.
pub fn cmd_209<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    vars2_from_gd(
        h,
        vm,
        ctx,
        |h, gd| h.fun_02169100(gd),
        |h, gd| h.fun_02169110(gd),
    )
}

/// C: `sub_0215abd0`: command 210: var = `FUN_02012984(GAMEDATA)`.
pub fn cmd_210<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let r = h.fun_02012984(gd);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215abec`: command 212: the RTC time (`FUN_02087D28`, UNVERIFIED): var1 = byte 2, var2 = byte 3 (hour / minute?).
pub fn cmd_212<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a1 = h.fun_02159ae8(vm, ctx);
    let a2 = h.fun_02159ae8(vm, ctx);
    let mut t = [0u8; 8];
    h.fun_02087d28(&mut t);
    h.w16(a1, t[2] as u16);
    h.w16(a2, t[3] as u16);
    0
}

/// C: `sub_0215ac1c`: command 225: var = `FUN_02008550(FUN_020127A4(GAMEDATA) + 0x20)`.
pub fn cmd_225<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let l = h.fun_020127a4(gd);
    let r = h.fun_02008550(l.wrapping_add(0x20));
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215ac3c`: command 213: var = `FUN_0200C234(FUN_02012F08(GAMEDATA), value)`.
pub fn cmd_213<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012f08(gd);
    let r = h.fun_0200c234(x, v);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215ac68`: command 214: `FUN_0200C248(FUN_02012F08(GAMEDATA), v)`, then `FUN_02008A7C(FUN_02012F5C(GAMEDATA),
/// v, t0, t1, t2)` with the 3-word time record of `FUN_0203F4E4` (UNVERIFIED: record `v` with a timestamp).
pub fn cmd_214<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012f08(gd);
    h.fun_0200c248(x, v);
    let mut t = [0u8; 12];
    h.fun_0203f4e4(&mut t);
    let p = h.fun_02012f5c(gd);
    h.fun_02008a7c(p, v, rd32(&t, 0), rd32(&t, 4), rd32(&t, 8));
    0
}

/// C: `sub_0215aca8`: command 215: var = `FUN_0200C258(FUN_02012F08(GAMEDATA))`.
pub fn cmd_215<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012f08(gd);
    let r = h.fun_0200c258(x);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215acc8`: command 312: x = `FUN_02012EBC(GAMEDATA)`; var1 = `FUN_020071EC(x)`, var2 = `FUN_020071E4(x)`,
/// `FUN_0200723C(x, &a, &b)`, var3 = `a * 3 >= b` (unsigned, wrapping).
pub fn cmd_312<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012ebc(gd);
    let a1 = h.fun_02159ae8(vm, ctx);
    let a2 = h.fun_02159ae8(vm, ctx);
    let a3 = h.fun_02159ae8(vm, ctx);
    let r = h.fun_020071ec(x);
    h.w16(a1, r as u16);
    let r = h.fun_020071e4(x);
    h.w16(a2, r as u16);
    let mut a = [0u8; 4];
    let mut b = [0u8; 4];
    h.fun_0200723c(x, &mut a, &mut b);
    let a = u32::from_le_bytes(a);
    let b = u32::from_le_bytes(b);
    h.w16(a3, (a.wrapping_mul(3) >= b) as u16);
    0
}

/// C: `sub_0215ad38`: command 211: var = the ctx's zone id.
pub fn cmd_211<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let r = h.fun_0215a5d0(ctx);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215ad50`: command 217: `FUN_02159F84(v) == 1` -> `FUN_02012900(GAMEDATA, v)`.
pub fn cmd_217<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let v = h.fun_02159b10(vm, ctx);
    if h.fun_02159f84(v) == 1 {
        h.fun_02012900(gd, v);
    }
    0
}

/// `(s16)v` as the register value (`lsls 16; asrs 16`).
fn sext16(v: u32) -> u32 {
    v as u16 as i16 as i32 as u32
}

/// C: `sub_0215ad7c`: command 219: the current location as a record: `FUN_02014184(&rec, zone, (s16)dir, pos[0..3])` from
/// the location block `FUN_020127A4(GAMEDATA)`, then `FUN_0201289C(GAMEDATA, &rec)` (UNVERIFIED: save it as a warp point).
pub fn cmd_219<H: WorkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let l = h.fun_020127a4(gd);
    let p = h.fun_02012aa0(l);
    let zone = h.fun_02012ac8(l);
    let dir = h.fun_02012ae4(l);
    let (x, y, z) = (h.r32(p), h.r32(p + 4), h.r32(p + 8));
    let mut rec = [0u8; 0x1C];
    h.fun_02014184(&mut rec, zone, sext16(dir), x, y, z);
    h.fun_0201289c(gd, &rec);
    0
}

/// C: `sub_0215adcc`: command 220: the record from five values: `FUN_02014184(&rec, v0, (s16)v1, v2 << 16, v3 << 16,
/// v4 << 16)`, `FUN_0201289C(GAMEDATA, &rec)` (UNVERIFIED: set that warp point).
pub fn cmd_220<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let v0 = h.fun_02159b10(vm, ctx);
    let v1 = h.fun_02159b10(vm, ctx);
    let v2 = h.fun_02159b10(vm, ctx);
    let v3 = h.fun_02159b10(vm, ctx);
    let v4 = h.fun_02159b10(vm, ctx);
    let mut rec = [0u8; 0x1C];
    h.fun_02014184(&mut rec, v0, sext16(v1), v2 << 16, v3 << 16, v4 << 16);
    h.fun_0201289c(gd, &rec);
    0
}

/// C: `sub_0215ae34`: command 218: `FUN_02159F28(GAMEDATA, a, b)`; with `c != 0` the map matrix of the current zone is
/// reloaded (`FUN_02159B94` / `FUN_02159CC4` on `FUN_02012974(GAMEDATA)`, black-field map_matrix).
pub fn cmd_218<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    h.fun_02159f28(gd, a, b);
    if c != 0 {
        let x = h.fun_02012974(gd);
        let l = h.fun_020127a4(gd);
        let zone = h.fun_02012ac8(l);
        let mtx = h.fun_02013b44(zone);
        let heap = h.fun_0215a5cc(ctx);
        h.fun_02159b94(x, mtx, zone, heap);
        h.fun_02159cc4(x, gs, heap);
    }
    0
}

/// C: `sub_0215aeb8`: command 216: var = `FUN_02159F54(GAMEDATA, value)`.
pub fn cmd_216<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let r = h.fun_02159f54(gd, v);
    h.w16(a, r as u16);
    0
}

// ---- commands: Pokedex

/// C: `sub_0215aee8`: command 222: `mon = FUN_020173F0(species, 1, 0, -1, heap)`; mode 0 -> `FUN_0200CFA0(dex, mon)`;
/// the mon is freed (UNVERIFIED: mark species seen). Operands: mode, species.
pub fn cmd_222<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let dex = h.fun_0200ca1c(gd);
    let heap = h.fun_0215a5cc(ctx);
    let mode = h.fun_02159b10(vm, ctx);
    let species = h.fun_02159b10(vm, ctx);
    let mon = h.fun_020173f0(species, 1, 0, 0xFFFF_FFFF, heap);
    if mode == 0 {
        h.fun_0200cfa0(dex, mon);
    }
    h.fun_020307b0(mon);
    0
}

/// C: `sub_0215af40`: command 223: var = mode 0: `FUN_0200D064(dex, v)`, 1: `FUN_0200CED4(dex, v)`, else 0
/// (UNVERIFIED: seen / caught check).
pub fn cmd_223<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let dex = h.fun_0200ca1c(gd);
    let mode = h.fun_02159b10(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let r = match mode {
        0 => h.fun_0200d064(dex, v),
        1 => h.fun_0200ced4(dex, v),
        _ => 0,
    };
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215af98`: command 221: var = mode 0: `FUN_0200D0CC(dex)`, 1: `FUN_0200CF28(dex)`, else 0 (counts?).
pub fn cmd_221<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let dex = h.fun_0200ca1c(gd);
    let mode = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let r = match mode {
        0 => h.fun_0200d0cc(dex),
        1 => h.fun_0200cf28(dex),
        _ => 0,
    };
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215afe0`: command 464: `FUN_0200CA28(dex)`.
pub fn cmd_464<H: WorkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let dex = h.fun_0200ca1c(gd);
    h.fun_0200ca28(dex);
    0
}

/// C: `sub_0215aff4`: command 465: var = `FUN_0200CA38(dex)`.
pub fn cmd_465<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let dex = h.fun_0200ca1c(gd);
    let r = h.fun_0200ca38(dex);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215b014`: command 466: `FUN_0200CA40(dex)`.
pub fn cmd_466<H: WorkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let dex = h.fun_0200ca1c(gd);
    h.fun_0200ca40(dex);
    0
}

/// C: `sub_0215b028`: command 226: `r = FUN_02008428(options of FUN_020120F4(GAMESYS))`; r 0 / 1 -> var = r (other values
/// leave the var unchanged).
pub fn cmd_226<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let x = options(h, gd);
    let a = h.fun_02159ae8(vm, ctx);
    let r = h.fun_02008428(x);
    if r <= 1 {
        h.w16(a, r as u16);
    }
    0
}

/// C: `sub_0215b068`: command 227: set event flag 0x963.
pub fn cmd_227<H: WorkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let ew = h.fun_0201296c(gd);
    h.fun_02014314(ew, 0x963);
    0
}

/// C: `sub_0215b084`: command 228: `v` in 1..=2 (u16) -> `FUN_02007A68(boxes, v)`.
pub fn cmd_228<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let bx = h.fun_0201293c(gd);
    if (v.wrapping_add(0xFFFF) & 0xFFFF) <= 1 {
        h.fun_02007a68(bx, v);
    }
    0
}

/// C: `sub_0215b0b4`: command 229: x = `FUN_020089C0(FUN_02012EBC(GAMEDATA))`; var2 =
/// `FUN_02025C88(FUN_02008554(location block + 0x20))`, var1 = `FUN_020089F4(x)`, `FUN_02008A00(x)`.
pub fn cmd_229<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a1 = h.fun_02159ae8(vm, ctx);
    let a2 = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let l = h.fun_020127a4(gd);
    let y = h.fun_02012ebc(gd);
    let x = h.fun_020089c0(y);
    let p = h.fun_02008554(l.wrapping_add(0x20));
    let r = h.fun_02025c88(p);
    h.w16(a2, r as u16);
    let r = h.fun_020089f4(x);
    h.w16(a1, r as u16);
    h.fun_02008a00(x);
    0
}

/// C: `sub_0215b104`: command 230: var = `FUN_0200C308(FUN_02012F08(GAMEDATA), value) & 1`.
pub fn cmd_230<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012f08(gd);
    let r = h.fun_0200c308(x, v);
    h.w16(a, (r & 1) as u16);
    0
}

/// C: `sub_0215b138`: command 231: `FUN_0200C2F0(FUN_02012F08(GAMEDATA), value, 1)`.
pub fn cmd_231<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012f08(gd);
    h.fun_0200c2f0(x, v, 1);
    0
}

/// C: `sub_0215b158`: command 232: var = `FUN_0200C308(FUN_02012F08(GAMEDATA), value) == 3`.
pub fn cmd_232<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012f08(gd);
    let r = h.fun_0200c308(x, v);
    h.w16(a, (r == 3) as u16);
    0
}

/// C: `sub_0215b190`: command 234: event flag 0x97B clear -> var = 0; else x = `FUN_02012EBC(GAMEDATA)`, by
/// `FUN_0200725C(x, 8, 4)`: 0 -> var 0, 1 / 2 -> var = `FUN_0200ECB4(FUN_020073D8(x, 8, 0)) != 0`, other -> var 2; then
/// `FUN_020072A4(x, 8)`.
pub fn cmd_234<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let ew = h.fun_0201296c(gd);
    let a = h.fun_02159ae8(vm, ctx);
    if h.fun_020142e8(ew, 0x97B) == 0 {
        h.w16(a, 0);
        return 0;
    }
    let x = h.fun_02012ebc(gd);
    match h.fun_0200725c(x, 8, 4) {
        0 => h.w16(a, 0),
        1 | 2 => {
            let y = h.fun_020073d8(x, 8, 0);
            let r = h.fun_0200ecb4(y);
            h.w16(a, (r != 0) as u16);
        }
        _ => h.w16(a, 2),
    }
    h.fun_020072a4(x, 8);
    0
}

/// The four value operands of commands 490 / 491 (`v0`, then three u16).
fn values4<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> [u32; 4] {
    let v0 = h.fun_02159b10(vm, ctx);
    let v1 = h.fun_02159b10(vm, ctx);
    let v2 = h.fun_02159b10(vm, ctx);
    let v3 = h.fun_02159b10(vm, ctx);
    [v0, v1 & 0xFFFF, v2 & 0xFFFF, v3 & 0xFFFF]
}

/// C: `sub_0215b214`: command 490: `FUN_02162A74(FUN_02012844(GAMEDATA), v0, (u16)v1, (u16)v2, (u16)v3)`.
pub fn cmd_490<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = values4(h, vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012844(gd);
    h.fun_02162a74(x, v[0], v[1], v[2], v[3]);
    0
}

/// C: `sub_0215b26c`: command 491: `FUN_02162708(FUN_02012844(GAMEDATA), v0, (u16)v1, (u16)v2, (u16)v3)`.
pub fn cmd_491<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = values4(h, vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012844(gd);
    h.fun_02162708(x, v[0], v[1], v[2], v[3]);
    0
}

/// C: `sub_0215b2c4`: command 492: `FUN_02162748(FUN_02012844(GAMEDATA), v0, v1, (u16)v2, v3 << 16, (u16)v4)`.
pub fn cmd_492<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v0 = h.fun_02159b10(vm, ctx);
    let v1 = h.fun_02159b10(vm, ctx);
    let v2 = h.fun_02159b10(vm, ctx);
    let v3 = h.fun_02159b10(vm, ctx);
    let v4 = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012844(gd);
    h.fun_02162748(x, v0, v1, v2 & 0xFFFF, v3 << 16, v4 & 0xFFFF);
    0
}

/// C: `sub_0215b328`: command 149: `FUN_0215A4EC(event work, value)`.
pub fn cmd_149<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let ew = h.fun_0201296c(gd);
    let v = h.fun_02159b10(vm, ctx);
    h.fun_0215a4ec(ew, v);
    0
}

/// C: `sub_0215b350`: command 150: `FUN_0215A500(event work, value)`.
pub fn cmd_150<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let ew = h.fun_0201296c(gd);
    let v = h.fun_02159b10(vm, ctx);
    h.fun_0215a500(ew, v);
    0
}

/// C: `sub_0215b378`: command 151: var = `FUN_0215A4D8(event work, value)`.
pub fn cmd_151<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let ew = h.fun_0201296c(gd);
    let v = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let r = h.fun_0215a4d8(ew, v);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215b3ac` (command 140) / `sub_0215b408` (command 378, the same code): set the script event's end callback
/// argument (`FUN_0215898C(SW, FUN_0215F6E8(GAMESYS))`), stop the VM and yield.
pub fn cmd_140<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let x = h.fun_0215f6e8(gs);
    h.fun_0215898c(sw, x);
    h.fun_02011290(vm);
    1
}

/// var = `FUN_0216EFA8(FUN_02012A44(GAMEDATA), k) == 1 ? 0 : 1` (commands 141 / 379).
fn cmd_efa8<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32, k: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012a44(gd);
    let r = h.fun_0216efa8(x, k);
    h.w16(a, (r != 1) as u16);
    0
}

/// C: `sub_0215b3dc`: command 141: var = `FUN_0216EFA8(FUN_02012A44(GAMEDATA), 1) == 1 ? 0 : 1`.
pub fn cmd_141<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    cmd_efa8(h, vm, ctx, 1)
}

/// C: `sub_0215b438`: command 379: as command 141 with argument 0.
pub fn cmd_379<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    cmd_efa8(h, vm, ctx, 0)
}

/// C: `FUN_overlay_d_10__0215b464`: command 562: var = [`TABLE_562`]`[value]`. An index past the table (the original
/// reads the following ROM bytes) panics.
pub fn cmd_table_562<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let i = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    h.w16(a, TABLE_562[i as usize]);
    0
}

/// C: `sub_0215b488`: command 563: var = [`TABLE_563`]`[value]` (an index past the table panics).
pub fn cmd_table_563<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let i = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    h.w16(a, TABLE_563[i as usize]);
    0
}

// ---- party / Pokemon

/// C: `FUN_overlay_d_10__0215b4ac` (the value part): party mon `idx`: `(idx < count, FUN_0201AB28(party, idx or 0))`
/// (an index past the party count reads mon 0 and reports 0). The original writes the mon through its third argument;
/// see [`party_mon_to`].
pub fn party_mon<H: WorkHooks>(h: &mut H, ctx: u32, idx: u32) -> (u32, u32) {
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let n = h.fun_0201aa18(party);
    let (ok, i) = if idx < n { (1, idx) } else { (0, 0) };
    let mon = h.fun_0201ab28(party, i);
    (ok, mon)
}

/// C: `FUN_overlay_d_10__0215b4ac`: [`party_mon`] with the mon stored at `out`; returns the flag.
pub fn party_mon_to<H: WorkHooks>(h: &mut H, ctx: u32, idx: u32, out: u32) -> u32 {
    let (ok, mon) = party_mon(h, ctx, idx);
    h.w32(out, mon);
    ok
}

/// C: `FUN_overlay_d_10__0215b4dc`: PokeParam `param` of party mon `idx` (`FUN_02017E1C(mon, param, 0)`), 0 without one.
pub fn party_mon_param<H: WorkHooks>(h: &mut H, ctx: u32, idx: u32, param: u32) -> u32 {
    let (ok, mon) = party_mon(h, ctx, idx);
    if ok == 1 {
        h.fun_02017e1c(mon, param, 0)
    } else {
        0
    }
}

/// C: `FUN_overlay_d_10__0215b500`: 1 if `mon` is not an egg (param 0x4C) and knows `mv` (params 0x36..0x39).
pub fn mon_knows_move<H: WorkHooks>(h: &mut H, mon: u32, mv: u32) -> u32 {
    if h.fun_02017e1c(mon, 0x4C, 0) != 0 {
        return 0;
    }
    for p in 0x36..=0x39 {
        if mv == h.fun_02017e1c(mon, p, 0) {
            return 1;
        }
    }
    0
}

/// C: `sub_0215b558`: command 283: party mon `idx`: var1 = param 0xAE, var2 = param 0xAF (both 0 without a mon).
pub fn cmd_283<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a1 = h.fun_02159ae8(vm, ctx);
    let a2 = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let (ok, mon) = party_mon(h, ctx, idx);
    let r = if ok == 1 {
        let r = h.fun_02017e1c(mon, 0xAE, 0);
        h.w16(a1, r as u16);
        h.fun_02017e1c(mon, 0xAF, 0)
    } else {
        h.w16(a1, 0);
        0
    };
    h.w16(a2, r as u16);
    0
}

/// C: `sub_0215b5a8`: command 260: `FUN_0201BA34(party)` (UNVERIFIED: heal the party).
pub fn cmd_260<H: WorkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    h.fun_0201ba34(party);
    0
}

/// C: `sub_0215b5bc`: command 274: var = the u16 sum of params 0x0D..0x12 (the EVs) of party mon `idx` (0 without one).
pub fn cmd_274<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let (ok, mon) = party_mon(h, ctx, idx);
    let mut sum: u16 = 0;
    if ok == 1 {
        for p in 0x0D..=0x12 {
            let v = h.fun_02017e1c(mon, p, 0) as u16;
            sum = sum.wrapping_add(v);
        }
    }
    h.w16(a, sum);
    0
}

/// C: `sub_0215b660`: command 257: var = 1 when party mon `idx` has full HP (param 0xA0 == 0xA1, as u16) or is an egg
/// (0x4C == 1), else 0.
pub fn cmd_257<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    h.w16(a, 0);
    let (ok, mon) = party_mon(h, ctx, idx);
    if ok == 1 {
        let hp = h.fun_02017e1c(mon, 0xA0, 0) & 0xFFFF;
        let max = h.fun_02017e1c(mon, 0xA1, 0) & 0xFFFF;
        if hp == max || h.fun_02017e1c(mon, 0x4C, 0) == 1 {
            h.w16(a, 1);
        }
    }
    0
}

/// C: `sub_0215b6c0`: command 258: var = party mon `idx` is an egg (param 0x4C != 0).
pub fn cmd_258<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    h.w16(a, 0);
    let (ok, mon) = party_mon(h, ctx, idx);
    if ok == 1 {
        let egg = h.fun_02017e1c(mon, 0x4C, 0);
        h.w16(a, (egg != 0) as u16);
    }
    0
}

/// C: `sub_0215b704`: command 252: var = the friendship (param 9) of party mon `idx` when it is not an egg, else 0.
pub fn cmd_252<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    h.w16(a, 0);
    let (ok, mon) = party_mon(h, ctx, idx);
    if ok == 1 && h.fun_02017e1c(mon, 0x4C, 0) == 0 {
        let r = h.fun_02017e1c(mon, 9, 0);
        h.w16(a, r as u16);
    }
    0
}

/// C: `sub_0215b74c`: command 253: change the friendship of party mon `idx` (not an egg): `f` = param 9 (u8); op 0: `f =
/// min(v, 255)`, 1: `f = min(f + v, 255)`, 2: `f = f >= v ? f - v : 0` (signed compares), other ops keep `f`;
/// `FUN_02017E40(mon, 9, f)`. Operands: idx, v (values), op (u16 literal).
pub fn cmd_253<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let idx = h.fun_02159b10(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let op = h.fun_02011330(vm);
    let (ok, mon) = party_mon(h, ctx, idx);
    if ok == 0 {
        return 0;
    }
    if h.fun_02017e1c(mon, 0x4C, 0) == 1 {
        return 0;
    }
    let f = h.fun_02017e1c(mon, 9, 0) & 0xFF;
    let f = match op {
        0 => {
            if v > 0xFF {
                0xFF
            } else {
                v & 0xFF
            }
        }
        1 => {
            if (f.wrapping_add(v) as i32) > 0xFF {
                0xFF
            } else {
                f.wrapping_add(v & 0xFF) & 0xFF
            }
        }
        2 => {
            if (f as i32) < (v as i32) {
                0
            } else {
                f.wrapping_sub(v & 0xFF) & 0xFF
            }
        }
        _ => f,
    };
    h.fun_02017e40(mon, 9, f);
    0
}

/// C: `sub_0215b7e8`: command 254: var = 0x28A when party mon `idx` is an egg, else its species ([`party_mon_param`]).
pub fn cmd_254<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let r = if party_mon_param(h, ctx, idx, 0x4C) != 0 {
        0x28A
    } else {
        party_mon_param(h, ctx, idx, 5)
    };
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215b824`: command 255: var = the form (param 0x6F) of party mon `idx`.
pub fn cmd_255<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let r = party_mon_param(h, ctx, idx, 0x6F);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215b848`: command 269: var = v 1: `FUN_0201AB18(party)`, 2: `FUN_0201AB08(party)`, else 0.
pub fn cmd_269<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let r = match v {
        1 => h.fun_0201ab18(party) & 0xFFFF,
        2 => h.fun_0201ab08(party) & 0xFFFF,
        _ => 0,
    };
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215b888`: command 259: party counts by `v`: 0 `FUN_0201AA18` (all), 1 `FUN_0201AA1C`, 2 `FUN_0201AA50`, 3
/// `FUN_0201AA88`, 4 `FUN_0201AAD4`, 5 free slots `max(0, capacity - count)` (signed), other 0.
pub fn cmd_259<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let r = match v {
        0 => h.fun_0201aa18(party),
        1 => h.fun_0201aa1c(party),
        2 => h.fun_0201aa50(party),
        3 => h.fun_0201aa88(party),
        4 => h.fun_0201aad4(party),
        5 => {
            let cap = h.fun_0201aa14(party);
            let n = h.fun_0201aa18(party);
            if cap as i32 >= n as i32 {
                cap.wrapping_sub(n)
            } else {
                0
            }
        }
        _ => 0,
    };
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215b904`: command 276: var = the number of party mons (not eggs) of species `v` (`0 < v <= 0x289`; 0 for other
/// values).
pub fn cmd_276<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let n = h.fun_0201aa18(party) & 0xFFFF;
    if v == 0 || v > 0x289 {
        h.w16(a, 0);
        return 0;
    }
    let mut count: u16 = 0;
    let mut i = 0u32;
    while (i as i32) < (n as i32) {
        let mon = h.fun_0201ab28(party, i);
        if mon != 0 && h.fun_02017e1c(mon, 0x4C, 0) == 0 && h.fun_02017e1c(mon, 5, 0) & 0xFFFF == v
        {
            count = count.wrapping_add(1);
        }
        i += 1;
    }
    h.w16(a, count);
    0
}

/// C: `sub_0215b994`: command 289: PC box counts by `v`: 0 `FUN_02007984` (mons stored), 1 and 2 `FUN_020079A0`, 3
/// `FUN_02007984 - FUN_020079A0`, 5 free places `0x2D0 - FUN_02007984` (0 when above, signed), other 0 (jump table
/// 0x0215B9CE; the spec header's case list is off by one from case 2 on).
pub fn cmd_289<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let bx = h.fun_0201293c(gd);
    h.fun_02012934(gd);
    let r = match v {
        0 => h.fun_02007984(bx),
        1 | 2 => h.fun_020079a0(bx),
        3 => {
            let n = h.fun_02007984(bx);
            let m = h.fun_020079a0(bx);
            n.wrapping_sub(m)
        }
        5 => {
            let n = h.fun_02007984(bx);
            if n as i32 <= 0x2D0 {
                0x2D0u32.wrapping_sub(n)
            } else {
                0
            }
        }
        _ => 0,
    };
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215ba18`: command 280: the first party mon (not an egg) of species `v` (`0 < v <= 0x289`): var1 = 1, var2 =
/// its index; none -> var1 = 0.
pub fn cmd_280<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let a1 = h.fun_02159ae8(vm, ctx);
    let a2 = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let n = h.fun_0201aa18(party) & 0xFFFF;
    if v == 0 || v > 0x289 {
        h.w16(a1, 0);
        return 0;
    }
    let mut i = 0u32;
    while (i as i32) < (n as i32) {
        let mon = h.fun_0201ab28(party, i);
        if mon != 0 && h.fun_02017e1c(mon, 0x4C, 0) == 0 && h.fun_02017e1c(mon, 5, 0) & 0xFFFF == v
        {
            break;
        }
        i += 1;
    }
    if (i as i32) < (n as i32) {
        h.w16(a1, 1);
        h.w16(a2, i as u16);
    } else {
        h.w16(a1, 0);
    }
    0
}

/// C: `sub_0215bab0`: command 277: var = party mon `idx` knows move `mv` ([`mon_knows_move`]). Operands: var, move, idx.
pub fn cmd_277<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let mv = h.fun_02159b10(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    h.w16(a, 0);
    let (ok, mon) = party_mon(h, ctx, idx);
    if ok == 1 && mon_knows_move(h, mon, mv) == 1 {
        h.w16(a, 1);
    }
    0
}

/// C: `sub_0215baf4`: command 278: var = the first party slot whose mon knows move `mv`, 6 when none.
pub fn cmd_278<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let a = h.fun_02159ae8(vm, ctx);
    let mv = h.fun_02159b10(vm, ctx);
    let n = h.fun_0201aa18(party);
    h.w16(a, 6);
    let mut i = 0u32;
    while (i as i32) < (n as i32) {
        let mon = h.fun_0201ab28(party, i);
        if mon_knows_move(h, mon, mv) == 1 {
            h.w16(a, i as u16);
            break;
        }
        i += 1;
    }
    0
}

/// The give-Pokemon record of commands 268 / 290: `{heap, species, form, level, 0, 2, 2, 2, 4}` (UNVERIFIED names).
fn give_rec_3(heap: u32, v: [u32; 3]) -> GiveMonRec {
    let mut r = [0u8; 0x24];
    r[0..2].copy_from_slice(&(heap as u16).to_le_bytes());
    wr32(&mut r, 4, v[0]);
    wr32(&mut r, 8, v[1]);
    wr32(&mut r, 0xC, v[2]);
    wr32(&mut r, 0x10, 0);
    wr32(&mut r, 0x14, 2);
    wr32(&mut r, 0x18, 2);
    wr32(&mut r, 0x1C, 2);
    wr32(&mut r, 0x20, 4);
    r
}

/// `x >= 3 ? 2 : x` (signed; commands 270 / 291).
fn min3(x: u32) -> u32 {
    if x as i32 >= 3 {
        2
    } else {
        x
    }
}

/// The give-Pokemon record of commands 270 / 291 from eight values: `{heap, v1, v2, v3, v7, min3(v4), min3(v5), min3(v6),
/// v8}`.
fn give_rec_8(heap: u32, v: [u32; 8]) -> GiveMonRec {
    let mut r = [0u8; 0x24];
    r[0..2].copy_from_slice(&(heap as u16).to_le_bytes());
    wr32(&mut r, 4, v[0]);
    wr32(&mut r, 8, v[1]);
    wr32(&mut r, 0xC, v[2]);
    wr32(&mut r, 0x10, v[6]);
    wr32(&mut r, 0x14, min3(v[3]));
    wr32(&mut r, 0x18, min3(v[4]));
    wr32(&mut r, 0x1C, min3(v[5]));
    wr32(&mut r, 0x20, v[7]);
    r
}

/// The common head of commands 268 / 270 / 290 / 291: (GAMEDATA, heap, result var).
fn give_head<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> (u32, u32, u32) {
    h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let heap = h.fun_0215a5cc(ctx);
    h.fun_02012934(gd);
    h.fun_02012944(gd);
    let a = h.fun_02159ae8(vm, ctx);
    (gd, heap, a)
}

fn values<H: WorkHooks, const N: usize>(h: &mut H, vm: u32, ctx: u32) -> [u32; N] {
    let mut v = [0u32; N];
    for x in v.iter_mut() {
        *x = h.fun_02159b10(vm, ctx);
    }
    v
}

/// C: `sub_0215bb50`: command 268: give a Pokemon (species, form, level values) to the party: var = [`give_party`]
/// (0 = party full).
pub fn cmd_268<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gd, heap, a) = give_head(h, vm, ctx);
    let v = values::<H, 3>(h, vm, ctx);
    let rec = give_rec_3(heap, v);
    let r = give_party(h, gd, &rec);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215bbd0`: command 270: give a Pokemon with more fields (eight values, see the record) to the party.
pub fn cmd_270<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gd, heap, a) = give_head(h, vm, ctx);
    let v = values::<H, 8>(h, vm, ctx);
    let rec = give_rec_8(heap, v);
    let r = give_party(h, gd, &rec);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215bca0`: command 264: var = the number of nonzero moves (params 0x36..0x39) of party mon `idx`.
pub fn cmd_264<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let (ok, mon) = party_mon(h, ctx, idx);
    let mut n: u16 = 0;
    if ok == 1 {
        for i in 0..4 {
            if h.fun_02017e1c(mon, 0x36 + i, 0) != 0 {
                n += 1;
            }
        }
    }
    h.w16(a, n);
    0
}

/// C: `sub_0215bce8`: command 266: var = the move in slot `s` (param 0x36 + s) of party mon `idx`, 0 without a mon.
pub fn cmd_266<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let s = h.fun_02159b10(vm, ctx);
    let (ok, mon) = party_mon(h, ctx, idx);
    let r = if ok == 1 {
        h.fun_02017e1c(mon, s.wrapping_add(0x36), 0)
    } else {
        0
    };
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215bd2c`: command 267: set a move of party mon `idx` (inside `FUN_02017D30` / `FUN_02017D70`): slot > 4 ->
/// learn it (`FUN_020180F4`); move 0 -> delete slot `slot`: the moves after it move up (move 0x36.., PP 0x3E.., PP-ups
/// 0x3A.. from the next slot while it holds a move, up to slot 3) and the last one is cleared; else
/// `FUN_020181F4(mon, move, slot)`.
pub fn cmd_267<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let idx = h.fun_02159b10(vm, ctx);
    let slot = h.fun_02159b10(vm, ctx);
    let mv = h.fun_02159b10(vm, ctx);
    let (ok, mon) = party_mon(h, ctx, idx);
    if ok == 0 {
        return 0;
    }
    let k = h.fun_02017d30(mon);
    if slot > 4 {
        h.fun_020180f4(mon, mv);
    } else if mv == 0 {
        let mut s = slot & 0xFF;
        if slot < 3 && s < 3 {
            loop {
                let m = h.fun_02017e1c(mon, s + 0x37, 0);
                if m == 0 {
                    break;
                }
                let pp = h.fun_02017e1c(mon, s + 0x3F, 0);
                let ppu = h.fun_02017e1c(mon, s + 0x3B, 0);
                h.fun_02017e40(mon, s + 0x36, m);
                h.fun_02017e40(mon, s + 0x3E, pp);
                h.fun_02017e40(mon, s + 0x3A, ppu);
                s = (s + 1) & 0xFF;
                if s >= 3 {
                    break;
                }
            }
        }
        h.fun_02017e40(mon, s + 0x36, 0);
        h.fun_02017e40(mon, s + 0x3E, 0);
        h.fun_02017e40(mon, s + 0x3A, 0);
    } else {
        h.fun_020181f4(mon, mv, slot & 0xFF);
    }
    h.fun_02017d70(mon, k);
    0
}

/// C: `sub_0215be24`: command 275: var = party mon `idx` is the player's own: OT id (param 7) == `FUN_02008530(player)`,
/// OT name (0x8D, into a new STRBUF) equal to the player name (`FUN_02008500`, compared with `FUN_0204589C`), OT gender
/// (0x9A) == `FUN_02008550(player)` and version (0x77) == [`VERSION`]. A failed STRBUF allocation gives 0 without freeing
/// the other one (as the original).
pub fn cmd_275<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let pl = h.fun_02012944(gd);
    let (ok, mon) = party_mon(h, ctx, idx);
    if ok == 0 {
        h.w16(a, 0);
        return 0;
    }
    let id = h.fun_02017e1c(mon, 7, 0);
    if id != h.fun_02008530(pl) {
        h.w16(a, 0);
        return 0;
    }
    let heap = h.fun_0215a5cc(ctx);
    let s1 = h.fun_020457b0(0x10, heap);
    let s2 = h.fun_020457b0(0x10, heap);
    if s1 == 0 || s2 == 0 {
        h.w16(a, 0);
        return 0;
    }
    h.fun_02017e1c(mon, 0x8D, s1);
    h.fun_02008500(pl, s2);
    let same = h.fun_0204589c(s1, s2);
    h.fun_02045808(s1);
    h.fun_02045808(s2);
    if same == 0 {
        h.w16(a, 0);
        return 0;
    }
    let g = h.fun_02017e1c(mon, 0x9A, 0);
    if g != h.fun_02008550(pl) {
        h.w16(a, 0);
        return 0;
    }
    if h.fun_02017e1c(mon, 0x77, 0) != VERSION as u32 {
        h.w16(a, 0);
        return 0;
    }
    h.w16(a, 1);
    0
}

/// C: `sub_0215bf30`: command 279: `FUN_02017988(party mon idx, v)`, then the mon is registered in the Pokedex
/// (`FUN_0200CFA0`) (UNVERIFIED: change its form).
pub fn cmd_279<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let idx = h.fun_02159b10(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let party = h.fun_02012934(gd);
    let mon = h.fun_0201ab28(party, idx);
    h.fun_02017988(mon, v);
    let dex = h.fun_0200ca1c(gd);
    h.fun_0200cfa0(dex, mon);
    0
}

/// C: `sub_0215bf7c`: command 284: `FUN_02017A20(party mon v0, v2, v1)`, then `FUN_0200CFA0(dex, mon)`.
pub fn cmd_284<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v0 = h.fun_02159b10(vm, ctx);
    let v1 = h.fun_02159b10(vm, ctx);
    let v2 = h.fun_02159b10(vm, ctx);
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let party = h.fun_02012934(gd);
    let mon = h.fun_0201ab28(party, v0);
    h.fun_02017a20(mon, v2, v1);
    let dex = h.fun_0200ca1c(gd);
    h.fun_0200cfa0(dex, mon);
    0
}

/// C: `sub_0215bfd4`: command 488: party mon `idx`: `x = FUN_0216F948(mon, heap)`, var = `FUN_0216FAA0(x)`, x freed.
pub fn cmd_488<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let heap = h.fun_0215a5cc(ctx);
    let (ok, mon) = party_mon(h, ctx, idx);
    if ok == 1 {
        let x = h.fun_0216f948(mon, heap);
        let r = h.fun_0216faa0(x);
        h.w16(a, r as u16);
        h.fun_020307b0(x);
    }
    0
}

/// C: `sub_0215c01c`: command 281: var = `(u16) param 0x95 of party mon idx == TABLE_281[t]` (an index past the table
/// panics; the original reads the following ROM bytes).
pub fn cmd_281<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let t = h.fun_02159b10(vm, ctx);
    let r = party_mon_param(h, ctx, idx, 0x95) & 0xFFFF;
    h.w16(a, (r == TABLE_281[t as usize] as u32) as u16);
    0
}

/// C: `sub_0215c064`: command 282: var1..3 = params 0x92 / 0x93 / 0x94 of party mon `idx` (0 without one).
pub fn cmd_282<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a1 = h.fun_02159ae8(vm, ctx);
    let a2 = h.fun_02159ae8(vm, ctx);
    let a3 = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let (ok, mon) = party_mon(h, ctx, idx);
    let r = if ok == 1 {
        let r = h.fun_02017e1c(mon, 0x92, 0);
        h.w16(a1, r as u16);
        let r = h.fun_02017e1c(mon, 0x93, 0);
        h.w16(a2, r as u16);
        h.fun_02017e1c(mon, 0x94, 0)
    } else {
        h.w16(a1, 0);
        h.w16(a2, 0);
        0
    };
    h.w16(a3, r as u16);
    0
}

/// C: `sub_0215c0d0`: command 273: set an IV: when `p` is one of [`IV_TABLE`]'s params and `v <= max`,
/// `FUN_02017E40(party mon idx, p, v)` and `FUN_020185C4(mon)` (recalculate the stats). Operands: idx, p, v.
pub fn cmd_273<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let idx = h.fun_02159b10(vm, ctx);
    let p = h.fun_02159b10(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    for &(param, max) in IV_TABLE.iter() {
        if p == param as u32 {
            if v <= max as u32 {
                let (ok, mon) = party_mon(h, ctx, idx);
                if ok == 1 {
                    h.fun_02017e40(mon, p, v);
                    h.fun_020185c4(mon);
                }
            }
            return 0;
        }
    }
    0
}

/// C: `sub_0215c138`: command 272: var = PokeParam `v` of party mon `idx` when `v` is in [`PARAM_WHITELIST`], else 0.
pub fn cmd_272<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let idx = h.fun_02159b10(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    if PARAM_WHITELIST.iter().any(|&p| v == p as u32) {
        let r = party_mon_param(h, ctx, idx, v);
        h.w16(a, r as u16);
        return 0;
    }
    h.w16(a, 0);
    0
}

/// C: `sub_0215c180`: command 290: as command 268, but to the PC boxes ([`give_box`]; 0 = boxes full).
pub fn cmd_290<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gd, heap, a) = give_head(h, vm, ctx);
    let v = values::<H, 3>(h, vm, ctx);
    let rec = give_rec_3(heap, v);
    let r = give_box(h, gd, &rec);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215c200`: command 291: as command 270, but to the PC boxes ([`give_box`]).
pub fn cmd_291<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let (gd, heap, a) = give_head(h, vm, ctx);
    let v = values::<H, 8>(h, vm, ctx);
    let rec = give_rec_8(heap, v);
    let r = give_box(h, gd, &rec);
    h.w16(a, r as u16);
    0
}

/// C: `sub_0215c2d0`: command 271: give an egg of `species` / `form`: party full (capacity <= count, signed) -> var 0; else
/// `mon = FUN_020173F0(species, 1, -1, -1, heap)`, form (0x6F), OT name (0x8D, the player's), hatch cycles
/// (friendship 9 = personal field 0x15 of `FUN_0201ADE4(species, form, heap)`), egg flag (0x4C = 1), nickname (0x73 =
/// message 0x28A of the message data at [`EGG_NAME_MSGDATA`]), stats (`FUN_020185C4`), met data
/// (`FUN_0202F190(mon, 5, player, 0xEA63, heap)`), added to the party (`FUN_0201A98C`) and freed; var 1.
pub fn cmd_271<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let heap = h.fun_0215a5cc(ctx);
    let party = h.fun_02012934(gd);
    let pl = h.fun_02012944(gd);
    let a = h.fun_02159ae8(vm, ctx);
    let species = h.fun_02159b10(vm, ctx);
    let form = h.fun_02159b10(vm, ctx);
    let cap = h.fun_0201aa14(party);
    let n = h.fun_0201aa18(party);
    if cap as i32 <= n as i32 {
        h.w16(a, 0);
        return 0;
    }
    let mon = h.fun_020173f0(species, 1, 0xFFFF_FFFF, 0xFFFF_FFFF, heap);
    h.fun_02017e40(mon, 0x6F, form);
    let pl2 = h.fun_02012944(gd);
    let name = h.fun_02008510(pl2, heap);
    h.fun_02017e40(mon, 0x8D, name);
    h.fun_02045808(name);
    let sp = h.fun_02017e1c(mon, 5, 0);
    let fm = h.fun_02017e1c(mon, 0x6F, 0);
    let pd = h.fun_0201ade4(sp & 0xFFFF, fm & 0xFFFF, heap);
    let cycles = h.fun_0201ae1c(pd, 0x15);
    h.fun_0201ae14(pd);
    h.fun_02017e40(mon, 9, cycles);
    h.fun_02017e40(mon, 0x4C, 1);
    let md = h.r32(EGG_NAME_MSGDATA);
    let nick = h.fun_02045ec0(md, 0x28A);
    h.fun_02017e40(mon, 0x73, nick);
    h.fun_02045808(nick);
    h.fun_020185c4(mon);
    h.fun_0202f190(mon, 5, pl, 0xEA63, heap);
    h.fun_0201a98c(party, mon);
    h.fun_020307b0(mon);
    h.w16(a, 1);
    0
}

/// C: `FUN_overlay_d_10__0215c40c`: build a Pokemon from a [`GiveMonRec`]: `FUN_02017430(species, level, OT id, 0, -1,
/// FUN_020186C8(OT id, species, form, +0x18, +0x14, +0x1C), 0, heap)`, form (0x6F) = +8, param 6 = +0x10, the ball (0x98 =
/// `FUN_0202142C(item)`) when `FUN_02020FE4(item, 0xF, heap) == 4`, met data (`FUN_0202F190(mon, 0, player, location
/// name of the current zone, heap)`), stats; returns the mon.
pub fn build_mon<H: WorkHooks>(h: &mut H, gd: u32, rec: &GiveMonRec) -> u32 {
    let pl = h.fun_02012944(gd);
    let pl2 = h.fun_02012944(gd);
    let ot = h.fun_02008530(pl2);
    let heap = rd16(rec, 0);
    let pid = h.fun_020186c8(
        ot,
        rd32(rec, 4) & 0xFFFF,
        rd32(rec, 8) & 0xFFFF,
        rd32(rec, 0x18),
        rd32(rec, 0x14),
        rd32(rec, 0x1C),
    );
    let mon = h.fun_02017430(
        rd32(rec, 4) & 0xFFFF,
        rd32(rec, 0xC) & 0xFFFF,
        ot,
        0,
        0xFFFF_FFFF,
        pid,
        0,
        heap,
    );
    h.fun_02017e40(mon, 0x6F, rd32(rec, 8));
    h.fun_02017e40(mon, 6, rd32(rec, 0x10));
    let item = rd32(rec, 0x20) & 0xFFFF;
    if h.fun_02020fe4(item, 0xF, heap) == 4 {
        let ball = h.fun_0202142c(item);
        h.fun_02017e40(mon, 0x98, ball);
    }
    let l = h.fun_020127a4(gd);
    let zone = h.fun_02012ac8(l);
    let place = h.fun_02013e4c(zone);
    h.fun_0202f190(mon, 0, pl, place, heap);
    h.fun_020185c4(mon);
    mon
}

/// C: `FUN_overlay_d_10__0215c4c8`: give the mon of `rec` to the PC boxes: 0 when 0x2D0 or more are stored (signed); else
/// [`build_mon`], deposit (`FUN_020074D8(boxes, FUN_020185F4(mon))`), Pokedex (`FUN_0200CDE0`), free; 1.
pub fn give_box<H: WorkHooks>(h: &mut H, gd: u32, rec: &GiveMonRec) -> u32 {
    let bx = h.fun_0201293c(gd);
    if h.fun_02007984(bx) as i32 >= 0x2D0 {
        return 0;
    }
    let mon = build_mon(h, gd, rec);
    let part = h.fun_020185f4(mon);
    h.fun_020074d8(bx, part);
    let dex = h.fun_0200ca1c(gd);
    h.fun_0200cde0(dex, mon);
    h.fun_020307b0(mon);
    1
}

/// C: `FUN_overlay_d_10__0215c510`: give the mon of `rec` to the party: 0 when full (capacity <= count, signed); else
/// [`build_mon`], `FUN_0201A98C(party, mon)`, Pokedex (`FUN_0200CDE0`), free; 1.
pub fn give_party<H: WorkHooks>(h: &mut H, gd: u32, rec: &GiveMonRec) -> u32 {
    let party = h.fun_02012934(gd);
    let cap = h.fun_0201aa14(party);
    let n = h.fun_0201aa18(party);
    if cap as i32 <= n as i32 {
        return 0;
    }
    let mon = build_mon(h, gd, rec);
    h.fun_0201a98c(party, mon);
    let dex = h.fun_0200ca1c(gd);
    h.fun_0200cde0(dex, mon);
    h.fun_020307b0(mon);
    1
}

/// C: `sub_0215c558`: command 249: `FUN_0200C278(FUN_02012F08(GAMEDATA), value)` (the `FUN_020127A4` result is unused).
pub fn cmd_249<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    h.fun_020127a4(gd);
    let v = h.fun_02159b10(vm, ctx);
    let x = h.fun_02012f08(gd);
    h.fun_0200c278(x, v);
    0
}

/// C: `sub_0215c584`: command 250: `FUN_0200C29C(FUN_02012F08(GAMEDATA), value)`.
pub fn cmd_250<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    h.fun_020127a4(gd);
    let v = h.fun_02159b10(vm, ctx);
    let x = h.fun_02012f08(gd);
    h.fun_0200c29c(x, v);
    0
}

/// C: `sub_0215c5b0`: command 251: var = `value <= FUN_0200C274(FUN_02012F08(GAMEDATA))` (unsigned).
pub fn cmd_251<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    h.fun_020127a4(gd);
    let x = h.fun_02012f08(gd);
    let r = h.fun_0200c274(x);
    h.w16(a, (v <= r) as u16);
    0
}

/// C: `sub_0215c5ec`: command 330: call the child event `FUN_020145CC(GAMESYS)`; yields.
pub fn cmd_330<H: WorkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = h.fun_020145cc(gs);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0215c610`: command 331: call the child event `FUN_020144DC(GAMESYS, field map)`; yields.
pub fn cmd_331<H: WorkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let ev = h.fun_020144dc(gs, fm);
    h.fun_0215897c(sw, ev);
    1
}

/// Run the command handler at `handler` (the command table word; the Thumb bit may be set) when it belongs to this
/// area; `None` for any other address (the `scrcmd_work` part of [`crate::scrcmd::dispatch`]).
pub fn dispatch<H: WorkHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    let f: fn(&mut H, u32, u32) -> u32 = match handler & !1 {
        0x0215_B3AC | 0x0215_B408 => cmd_140,
        0x0215_B3DC => cmd_141,
        0x0215_B328 => cmd_149,
        0x0215_B350 => cmd_150,
        0x0215_B378 => cmd_151,
        0x0215_AA60 => cmd_201,
        0x0215_AA90 => cmd_202,
        0x0215_AABC => cmd_203,
        0x0215_AADC => cmd_204,
        0x0215_AB18 => cmd_205,
        0x0215_AAF4 => cmd_206,
        0x0215_AB3C => cmd_207,
        0x0215_AB60 => cmd_208,
        0x0215_AB98 => cmd_209,
        0x0215_ABD0 => cmd_210,
        0x0215_AD38 => cmd_211,
        0x0215_ABEC => cmd_212,
        0x0215_AC3C => cmd_213,
        0x0215_AC68 => cmd_214,
        0x0215_ACA8 => cmd_215,
        0x0215_AEB8 => cmd_216,
        0x0215_AD50 => cmd_217,
        0x0215_AE34 => cmd_218,
        0x0215_AD7C => cmd_219,
        0x0215_ADCC => cmd_220,
        0x0215_AF98 => cmd_221,
        0x0215_AEE8 => cmd_222,
        0x0215_AF40 => cmd_223,
        0x0215_AA50 => cmd_var_set_15,
        0x0215_AC1C => cmd_225,
        0x0215_B028 => cmd_226,
        0x0215_B068 => cmd_227,
        0x0215_B084 => cmd_228,
        0x0215_B0B4 => cmd_229,
        0x0215_B104 => cmd_230,
        0x0215_B138 => cmd_231,
        0x0215_B158 => cmd_232,
        0x0215_B190 => cmd_234,
        0x0215_C558 => cmd_249,
        0x0215_C584 => cmd_250,
        0x0215_C5B0 => cmd_251,
        0x0215_B704 => cmd_252,
        0x0215_B74C => cmd_253,
        0x0215_B7E8 => cmd_254,
        0x0215_B824 => cmd_255,
        0x0215_B660 => cmd_257,
        0x0215_B6C0 => cmd_258,
        0x0215_B888 => cmd_259,
        0x0215_B5A8 => cmd_260,
        0x0215_BCA0 => cmd_264,
        0x0215_BCE8 => cmd_266,
        0x0215_BD2C => cmd_267,
        0x0215_BB50 => cmd_268,
        0x0215_B848 => cmd_269,
        0x0215_BBD0 => cmd_270,
        0x0215_C2D0 => cmd_271,
        0x0215_C138 => cmd_272,
        0x0215_C0D0 => cmd_273,
        0x0215_B5BC => cmd_274,
        0x0215_BE24 => cmd_275,
        0x0215_B904 => cmd_276,
        0x0215_BAB0 => cmd_277,
        0x0215_BAF4 => cmd_278,
        0x0215_BF30 => cmd_279,
        0x0215_BA18 => cmd_280,
        0x0215_C01C => cmd_281,
        0x0215_C064 => cmd_282,
        0x0215_B558 => cmd_283,
        0x0215_BF7C => cmd_284,
        0x0215_B994 => cmd_289,
        0x0215_C180 => cmd_290,
        0x0215_C200 => cmd_291,
        0x0215_ACC8 => cmd_312,
        0x0215_C5EC => cmd_330,
        0x0215_C610 => cmd_331,
        0x0215_B438 => cmd_379,
        0x0215_AFE0 => cmd_464,
        0x0215_AFF4 => cmd_465,
        0x0215_B014 => cmd_466,
        0x0215_BFD4 => cmd_488,
        0x0215_B214 => cmd_490,
        0x0215_B26C => cmd_491,
        0x0215_B2C4 => cmd_492,
        0x0215_B464 => cmd_table_562,
        0x0215_B488 => cmd_table_563,
        _ => return None,
    };
    Some(f(h, vm, ctx))
}

#[cfg(test)]
mod tests;
