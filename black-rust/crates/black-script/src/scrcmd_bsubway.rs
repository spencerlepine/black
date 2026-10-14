//! F8f (fifth batch): overlay-21 `bsubway_scr.c`, Thumb code 0x021C9934-0x021CBFB0 (F1-SPLIT row 21.84): the Battle Subway work
//! W (0x7EC bytes) and commands 343..346 (58 functions; command 346 dispatches ~170 sub-commands).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_bsubway.spec`, ported here, listed in
//! [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_bsubway`. [`COMMANDS`] maps the command ids to their
//! handlers, [`dispatch`] runs one by address, [`call_wait`] the wait callbacks command 346 installs and [`call_event`] its
//! GMEVENT functions.
//!
//! Callees outside the area are [`BsubwayHooks`] methods named by address (vm.c, the operand readers, the ctx getters and the
//! GMEVENT calls come from [`CoreHooks`]). The Wi-Fi / wireless multi parts (comm_player.c, comm_entry_menu.c, the net layer)
//! stay hooks (X1); the save side (bsubway_savedata.c: play record P = save block 0x39, score record S = 0x3A, extra X = 0x3B)
//! and fld_btl_inst_tool.c are hooks too. Roles are UNVERIFIED (named by address). A callee the spec models as returning
//! nothing whose `r0` the original uses afterwards is a hook returning `u32` (the value the original finds in `r0`): the
//! event constructors `FUN_0216C538` / `FUN_0216C640` / `FUN_0216C6E8` / `FUN_0216C860` / `FUN_0216CB48`, `FUN_0200D87C`,
//! `FUN_0200DBA0`, `FUN_0200DE5C` / `FUN_0200DE64` / `FUN_0200DE94`, `FUN_021A31A4` and `FUN_02168AEC`.

use crate::hooks::MemEnv;
use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_9934,
    0x021C_9B24,
    0x021C_9B78,
    0x021C_9BCC,
    0x021C_9C54,
    0x021C_9CAC,
    0x021C_9D30,
    0x021C_9D70,
    0x021C_9DB0,
    0x021C_9E8C,
    0x021C_9F44,
    0x021C_9FD0,
    0x021C_9FFC,
    0x021C_A048,
    0x021C_A05C,
    0x021C_A150,
    0x021C_A164,
    0x021C_A1E4,
    0x021C_A210,
    0x021C_A234,
    0x021C_A238,
    0x021C_A23C,
    0x021C_A250,
    0x021C_A258,
    0x021C_A270,
    0x021C_A2A4,
    0x021C_A2CC,
    0x021C_A2DC,
    0x021C_A338,
    0x021C_A3B4,
    0x021C_A4C0,
    0x021C_A544,
    0x021C_A57C,
    0x021C_A598,
    0x021C_A5C4,
    0x021C_B6DC,
    0x021C_B700,
    0x021C_B754,
    0x021C_B7E8,
    0x021C_B86C,
    0x021C_B8AC,
    0x021C_B914,
    0x021C_B934,
    0x021C_B9BC,
    0x021C_B9E4,
    0x021C_BBA0,
    0x021C_BBAC,
    0x021C_BBD0,
    0x021C_BC14,
    0x021C_BC94,
    0x021C_BCC4,
    0x021C_BD08,
    0x021C_BD84,
    0x021C_BD90,
    0x021C_BED8,
    0x021C_BEFC,
    0x021C_BF38,
    0x021C_BF98,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (343, 0x021C_A57C),
    (344, 0x021C_A544),
    (345, 0x021C_A598),
    (346, 0x021C_A5C4),
];

/// The wait callbacks command 346 installs (VM +0x10, Thumb).
pub const WAIT_6DC: u32 = 0x021C_B6DD;
pub const WAIT_700: u32 = 0x021C_B701;
pub const WAIT_754: u32 = 0x021C_B755;
pub const WAIT_7E8: u32 = 0x021C_B7E9;
pub const WAIT_86C: u32 = 0x021C_B86D;
pub const WAIT_C14: u32 = 0x021C_BC15;
/// GMEVENT functions of command 346 (Thumb): the wireless disconnect and the 0x2C event.
pub const EVENT_DISCONNECT: u32 = 0x021C_B8AD;
pub const EVENT_934: u32 = 0x021C_B935;
/// The GMEVENT of sub-commands 0x67..0x69 (overlay 10, `FUN_020124AC(gs, 7, 0x021542C1, &{k, var, 0, 0})`).
pub const EVENT_67: u32 = 0x0215_42C1;
/// The two app-proc tables of `FUN_02014808` (sub-commands 0x15C / 0x198) and their ids.
pub const PROC_15C: (u32, u32) = (214, 0x021B_B474);
pub const PROC_198: (u32, u32) = (100, 0x021B_B6A4);

/// The work W: its magic, size, heap and the `__FILE__` / line of its allocation.
pub const W_MAGIC: u32 = 0x1234_5678;
pub const W_SIZE: u32 = 0x7EC;
pub const W_HEAP: u32 = 4;
pub const FILE_BSUBWAY: u32 = 0x021D_E58C;
pub const LINE_W: u32 = 0x5F;
pub const LINE_RECORDS: u32 = 0x89A;
/// Offsets in W.
pub const W_COUNT: u32 = 8;
pub const W_MODE: u32 = 9;
pub const W_FLAGS: u32 = 0xC;
/// Offset of the u16 win streak in the work W.
pub const W_STREAK: u32 = 0xE;
pub const W_SLOTS: u32 = 0x1E;
pub const W_TRAINERS: u32 = 0x32;
pub const W_GD: u32 = 0x6C;
pub const W_PLAY: u32 = 0x70;
pub const W_SCORE: u32 = 0x74;
pub const W_EXTRA: u32 = 0x78;
pub const W_PARTY_7C8: u32 = 0x7C8;
pub const W_BATTLE: u32 = 0x7CC;
/// Save block ids of the play / score / extra records (`FUN_020071CC(save, id)`).
pub const SAVE_PLAY: u32 = 0x39;
pub const SAVE_SCORE: u32 = 0x3A;
pub const SAVE_EXTRA: u32 = 0x3B;
/// The streak / BP cap.
pub const CAP_9999: u32 = 9999;
/// The 0x966 flag of sub-commands 4 / 5 / 0x2D / 0x142 (`FUN_02014314` / `FUN_0201433C`).
pub const FLAG_966: u32 = 0x966;

/// ROM tables of overlay 21 used here (0x021DACCC..0x021DB404, read as one block; checked against the build by the tests):
/// 0x021DACD0 u8 level steps (`FUN_021CA164`, from 0x021DACCF for step 0), 0x021DACDC 10-byte BP rows, 0x021DAD18 BP by
/// level, 0x021DAD24 u16 by mode, 0x021DAD36 u16 by value, 0x021DAD48 / 0x021DAD5C u16 OBJ codes, 0x021DAD70 12-byte
/// records (passed by address), 0x021DAD88 u32 by value, 0x021DADAC u32 by zone kind, 0x021DADD0 5 x 8-byte NPC places,
/// 0x021DADF8 u32 levels, 0x021DAE50 12-byte records, 0x021DAEBC 0x10-byte NPC records (0xFFFF-terminated), 0x021DB3D0..
/// 0x021DB404 trainer id ranges.
pub const ROM_TABLES_ADDR: u32 = 0x021D_ACCC;
pub const T_LEVEL_STEPS: u32 = 0x021D_ACD0;
pub const T_BP_ROWS: u32 = 0x021D_ACDC;
pub const T_BP_LEVEL: u32 = 0x021D_AD18;
pub const T_BY_MODE: u32 = 0x021D_AD24;
pub const T_BY_VALUE: u32 = 0x021D_AD36;
pub const T_OBJ_A: u32 = 0x021D_AD48;
pub const T_OBJ_B: u32 = 0x021D_AD5C;
pub const T_REC_70: u32 = 0x021D_AD70;
pub const T_U32_88: u32 = 0x021D_AD88;
pub const T_U32_AC: u32 = 0x021D_ADAC;
pub const T_PLACES: u32 = 0x021D_ADD0;
pub const T_LEVELS: u32 = 0x021D_ADF8;
pub const T_REC_50: u32 = 0x021D_AE50;
pub const T_NPCS: u32 = 0x021D_AEBC;
/// Trainer id ranges `{u16 lo, u16 hi}` by level: single round < 6 / round 6, multi round < 6 / round 6.
pub const T_IDS_SINGLE: u32 = 0x021D_B3D8;
pub const T_IDS_SINGLE_BOSS: u32 = 0x021D_B3D0;
pub const T_IDS_MULTI: u32 = 0x021D_B3E4;
pub const T_IDS_MULTI_BOSS: u32 = 0x021D_B3F4;

pub const ROM_TABLES: [u8; 1848] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x04, 0x04, 0x03, 0x03, 0x02, 0x02, 0x01, 0x01, 0x00, 0x00,
    0x03, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
    0x0A, 0x0A, 0x0A, 0x0A, 0x03, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, 0x06,
    0x07, 0x08, 0x09, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x03, 0x03, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A,
    0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x0A, 0x00, 0x00, 0x14, 0x00, 0x15, 0x00, 0x16, 0x00, 0x16, 0x00,
    0x14, 0x00, 0x14, 0x00, 0x15, 0x00, 0x16, 0x00, 0x16, 0x00, 0x04, 0x00, 0x05, 0x00, 0x06, 0x00,
    0x06, 0x00, 0x0B, 0x00, 0x07, 0x00, 0x08, 0x00, 0x09, 0x00, 0x09, 0x00, 0x2C, 0x00, 0x0B, 0x00,
    0x0C, 0x00, 0x30, 0x00, 0x48, 0x00, 0x1C, 0x00, 0x0D, 0x00, 0x47, 0x00, 0x1E, 0x00, 0x20, 0x00,
    0x21, 0x00, 0x0F, 0x00, 0x10, 0x00, 0x31, 0x00, 0x35, 0x00, 0x11, 0x00, 0x1F, 0x00, 0x21, 0x00,
    0x2D, 0x00, 0x1F, 0x00, 0x00, 0x00, 0x2D, 0x00, 0x00, 0x00, 0xFE, 0xFF, 0x00, 0x60, 0x0B, 0x00,
    0x00, 0x00, 0x2F, 0x00, 0x00, 0x00, 0xFD, 0xFF, 0x00, 0x70, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x06, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
    0x43, 0x00, 0x00, 0x00, 0x45, 0x00, 0x00, 0x00, 0x47, 0x00, 0x00, 0x00, 0x47, 0x00, 0x00, 0x00,
    0x49, 0x00, 0x00, 0x00, 0x44, 0x00, 0x00, 0x00, 0x46, 0x00, 0x00, 0x00, 0x48, 0x00, 0x00, 0x00,
    0x48, 0x00, 0x00, 0x00, 0x46, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x3D, 0x00, 0x00, 0x00,
    0x0E, 0x00, 0x00, 0x00, 0x2F, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x25, 0x00, 0x00, 0x00,
    0x0E, 0x00, 0x00, 0x00, 0x1A, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00,
    0x06, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00, 0x08, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00,
    0x0A, 0x00, 0x00, 0x00, 0x0B, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x00, 0x00,
    0x0E, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x11, 0x00, 0x00, 0x00,
    0x12, 0x00, 0x00, 0x00, 0x18, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x2E, 0x00, 0x00, 0x00,
    0x92, 0x00, 0x00, 0x00, 0x00, 0x80, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x0F, 0x00,
    0x00, 0x80, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x0F, 0x00, 0x00, 0x80, 0x0B, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x0F, 0x00, 0x00, 0x80, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x80, 0x0F, 0x00, 0x00, 0x80, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x0F, 0x00,
    0x00, 0x80, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x0F, 0x00, 0x00, 0x80, 0x0B, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x0F, 0x00, 0x00, 0x80, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x80, 0x0F, 0x00, 0x00, 0x80, 0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x0F, 0x00,
    0x0B, 0x00, 0x01, 0x00, 0x47, 0x00, 0x00, 0x00, 0x11, 0x00, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0B, 0x00, 0x04, 0x00, 0x3E, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x21, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0F, 0x00, 0x01, 0x00, 0x42, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0F, 0x00, 0x04, 0x00, 0x32, 0x00, 0x00, 0x00, 0x10, 0x00, 0x23, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0C, 0x00, 0x01, 0x00, 0x17, 0x00, 0x00, 0x00, 0x10, 0x00, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0C, 0x00, 0x04, 0x00, 0x29, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x25, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x10, 0x00, 0x01, 0x00, 0x23, 0x00, 0x00, 0x00, 0x10, 0x00, 0x26, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x10, 0x00, 0x04, 0x00, 0x28, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x30, 0x00, 0x01, 0x00, 0x3D, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x28, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x30, 0x00, 0x02, 0x00, 0x1F, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x29, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x30, 0x00, 0x04, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x30, 0x00, 0x09, 0x00, 0x16, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x2B, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x30, 0x00, 0x0F, 0x00, 0x2A, 0x00, 0x00, 0x00, 0x11, 0x00, 0x2C, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x31, 0x00, 0x02, 0x00, 0x43, 0x00, 0x00, 0x00, 0x11, 0x00, 0x2D, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x31, 0x00, 0x04, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x10, 0x00, 0x2E, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x31, 0x00, 0x08, 0x00, 0x26, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x2F, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x31, 0x00, 0x0C, 0x00, 0x1C, 0x00, 0x00, 0x00, 0x10, 0x00, 0x30, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x48, 0x00, 0x02, 0x00, 0x2C, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x48, 0x00, 0x03, 0x00, 0x13, 0x00, 0x00, 0x00, 0x10, 0x00, 0x32, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x48, 0x00, 0x05, 0x00, 0x32, 0x00, 0x00, 0x00, 0x10, 0x00, 0x33, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x48, 0x00, 0x0B, 0x00, 0x44, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1C, 0x00, 0x01, 0x00, 0x2B, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x35, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1C, 0x00, 0x02, 0x00, 0x16, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x36, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1C, 0x00, 0x03, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x37, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1C, 0x00, 0x05, 0x00, 0x3D, 0x00, 0x00, 0x00, 0x10, 0x00, 0x38, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1C, 0x00, 0x06, 0x00, 0x15, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x39, 0x00, 0x3A, 0x00, 0x33, 0x00,
    0x1C, 0x00, 0x08, 0x00, 0x14, 0x00, 0x00, 0x00, 0x10, 0x00, 0x3B, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1C, 0x00, 0x0C, 0x00, 0x43, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x3C, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x35, 0x00, 0x03, 0x00, 0x43, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x3D, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x35, 0x00, 0x04, 0x00, 0x52, 0x00, 0x00, 0x00, 0x11, 0x00, 0x3E, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x35, 0x00, 0x06, 0x00, 0x3C, 0x00, 0x00, 0x00, 0x10, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x35, 0x00, 0x0B, 0x00, 0x22, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x44, 0x00, 0x03, 0x00, 0x2D, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x44, 0x00, 0x05, 0x00, 0x0C, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x42, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x44, 0x00, 0x06, 0x00, 0x4E, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x43, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x44, 0x00, 0x07, 0x00, 0x13, 0x00, 0x00, 0x00, 0x10, 0x00, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x44, 0x00, 0x08, 0x00, 0x3E, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x44, 0x00, 0x0F, 0x00, 0x3D, 0x00, 0x00, 0x00, 0x11, 0x00, 0x46, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x44, 0x00, 0x12, 0x00, 0x43, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x47, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x0D, 0x00, 0x04, 0x00, 0x20, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x11, 0x00, 0x01, 0x00, 0x37, 0x00, 0x00, 0x00, 0x11, 0x00, 0x49, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x47, 0x00, 0x11, 0x00, 0x3B, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x4A, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1E, 0x00, 0x09, 0x00, 0x45, 0x00, 0x00, 0x00, 0x11, 0x00, 0x4B, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1E, 0x00, 0x0A, 0x00, 0x51, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x4C, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1E, 0x00, 0x0C, 0x00, 0x31, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x4D, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1E, 0x00, 0x0E, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x10, 0x00, 0x4E, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1E, 0x00, 0x15, 0x00, 0x46, 0x00, 0x00, 0x00, 0x11, 0x00, 0x4F, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1F, 0x00, 0x07, 0x00, 0x43, 0x00, 0x00, 0x00, 0x11, 0x00, 0x50, 0x00, 0x51, 0x00, 0x32, 0x00,
    0x1F, 0x00, 0x09, 0x00, 0x2D, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x52, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1F, 0x00, 0x0B, 0x00, 0x17, 0x00, 0x00, 0x00, 0x10, 0x00, 0x53, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1F, 0x00, 0x10, 0x00, 0x11, 0x00, 0x00, 0x00, 0x10, 0x00, 0x54, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x1F, 0x00, 0x14, 0x00, 0x2D, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x55, 0x00, 0x56, 0x00, 0xCF, 0x00,
    0x3D, 0x00, 0x01, 0x00, 0x53, 0x00, 0x00, 0x00, 0x11, 0x00, 0x57, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x3D, 0x00, 0x02, 0x00, 0x32, 0x00, 0x00, 0x00, 0x10, 0x00, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x3D, 0x00, 0x05, 0x00, 0x2B, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x59, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x45, 0x00, 0x03, 0x00, 0x27, 0x00, 0x00, 0x00, 0x10, 0x00, 0x5A, 0x00, 0x5B, 0x00, 0x33, 0x00,
    0x45, 0x00, 0x05, 0x00, 0x23, 0x00, 0x00, 0x00, 0x10, 0x00, 0x5C, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x45, 0x00, 0x07, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x5D, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x45, 0x00, 0x16, 0x00, 0x32, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x5E, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x20, 0x00, 0x07, 0x00, 0x27, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x5F, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x20, 0x00, 0x09, 0x00, 0x3E, 0x00, 0x00, 0x00, 0x10, 0x00, 0x60, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x20, 0x00, 0x0B, 0x00, 0x35, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x61, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x20, 0x00, 0x0D, 0x00, 0x3B, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x62, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x20, 0x00, 0x0F, 0x00, 0x37, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x63, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x21, 0x00, 0x08, 0x00, 0x39, 0x00, 0x00, 0x00, 0x10, 0x00, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x21, 0x00, 0x0A, 0x00, 0x27, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x65, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x21, 0x00, 0x0D, 0x00, 0x3C, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x66, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x21, 0x00, 0x10, 0x00, 0x44, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x67, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x21, 0x00, 0x13, 0x00, 0x40, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x68, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x2D, 0x00, 0x03, 0x00, 0x19, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x69, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x2D, 0x00, 0x05, 0x00, 0x52, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x6A, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x2D, 0x00, 0x08, 0x00, 0x32, 0x00, 0x00, 0x00, 0x0F, 0x00, 0x6B, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x2C, 0x00, 0x03, 0x00, 0x1A, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x6C, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x2C, 0x00, 0x06, 0x00, 0x2C, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x6D, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x2C, 0x00, 0x07, 0x00, 0x30, 0x00, 0x00, 0x00, 0x10, 0x00, 0x6E, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x2C, 0x00, 0x0A, 0x00, 0x3A, 0x00, 0x00, 0x00, 0x10, 0x00, 0x6F, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x34, 0x00, 0x06, 0x00, 0x42, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x34, 0x00, 0x07, 0x00, 0x37, 0x00, 0x00, 0x00, 0x0D, 0x00, 0x71, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x34, 0x00, 0x0A, 0x00, 0x14, 0x00, 0x00, 0x00, 0x10, 0x00, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x34, 0x00, 0x12, 0x00, 0x1C, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x73, 0x00, 0x74, 0x00, 0xCE, 0x00,
    0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xC8, 0x00, 0x2B, 0x01, 0x32, 0x00, 0x45, 0x00, 0x46, 0x00, 0x59, 0x00, 0x00, 0x00, 0x31, 0x00,
    0x1E, 0x00, 0x45, 0x00, 0x32, 0x00, 0x6D, 0x00, 0x6E, 0x00, 0x9F, 0x00, 0x8C, 0x00, 0xB3, 0x00,
    0xA0, 0x00, 0xC7, 0x00, 0xB4, 0x00, 0xDB, 0x00, 0xA0, 0x00, 0xB3, 0x00, 0xB4, 0x00, 0xC7, 0x00,
    0xC8, 0x00, 0xDB, 0x00, 0xDC, 0x00, 0xEF, 0x00,
];

/// A byte of [`ROM_TABLES`] at ROM address `a`.
pub fn rom8(a: u32) -> u8 {
    ROM_TABLES[(a - ROM_TABLES_ADDR) as usize]
}

/// A u16 of [`ROM_TABLES`] at ROM address `a`.
pub fn rom16(a: u32) -> u16 {
    u16::from_le_bytes([rom8(a), rom8(a + 1)])
}

/// A u32 of [`ROM_TABLES`] at ROM address `a`.
pub fn rom32(a: u32) -> u32 {
    u32::from_le_bytes([rom8(a), rom8(a + 1), rom8(a + 2), rom8(a + 3)])
}

/// A pointer argument the original passes on to `FUN_02168878`: an address, or a stack local of the caller.
pub enum Buf<'a> {
    Addr(u32),
    Local(&'a [u16]),
}

/// Callees of the scrcmd_bsubway functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the ctx getters and the GMEVENT calls come from [`CoreHooks`].
#[allow(clippy::too_many_arguments)]
pub trait BsubwayHooks: CoreHooks {
    // ---- main: RNG, save blocks, bsubway_savedata.c
    /// C: `FUN_020056EC(n)` (RNG): a random number below `n`.
    fn fun_020056ec(&mut self, n: u32) -> u32;
    /// C: `FUN_020071CC(save, id)`: save block `id`.
    fn fun_020071cc(&mut self, save: u32, id: u32) -> u32;
    /// C: `FUN_02008550(x)`.
    fn fun_02008550(&mut self, x: u32) -> u32;
    /// C: `FUN_0200B424()`.
    fn fun_0200b424(&mut self);
    /// C: `FUN_0200B5CC(save, k, &out, x)` (the out word is a stack local of command 346).
    fn fun_0200b5cc(&mut self, save: u32, k: u32, out: &mut u32, x: u32) -> u32;
    /// C: `FUN_0200B6E0(gd, k, kind, v, x, &a, &b)`.
    fn fun_0200b6e0(&mut self, gd: u32, k: u32, kind: u32, v: u32, x: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_0200BA38(x)`.
    fn fun_0200ba38(&mut self, x: u32) -> u32;
    /// C: `FUN_0200BA58()`.
    fn fun_0200ba58(&mut self);
    /// C: `FUN_0200BA68(x)`.
    fn fun_0200ba68(&mut self, x: u32);
    /// C: `FUN_0200BAC8(save)`.
    fn fun_0200bac8(&mut self, save: u32) -> u32;
    /// C: `FUN_0200BAF4(x, heap)`: a party block (freed with `FUN_020307B0`).
    fn fun_0200baf4(&mut self, x: u32, heap: u32) -> u32;
    /// C: `FUN_0200BBA8(x)`.
    fn fun_0200bba8(&mut self, x: u32) -> u32;
    /// C: `FUN_0200D85C(P)`.
    fn fun_0200d85c(&mut self, p: u32);
    /// C: `FUN_0200D868(P, x)`.
    fn fun_0200d868(&mut self, p: u32, x: u32);
    /// C: `FUN_0200D87C(P)` (the spec's stub returns nothing: `r0` = P).
    fn fun_0200d87c(&mut self, p: u32) -> u32;
    /// C: `FUN_0200D884(P, id, out)`: field `id` of the play record (fields 5 / 6 / 8 fill `out`).
    fn fun_0200d884(&mut self, p: u32, id: u32, out: u32) -> u32;
    /// C: `FUN_0200D914(P, id, &data)`: store field `id` of the play record.
    fn fun_0200d914(&mut self, p: u32, id: u32, data: &[u8]);
    /// C: `FUN_0200D9E8(P, a, b, c)`.
    fn fun_0200d9e8(&mut self, p: u32, a: u32, b: u32, c: u32) -> u32;
    /// C: `FUN_0200DA14(P)`.
    fn fun_0200da14(&mut self, p: u32);
    /// C: `FUN_0200DA28(P)`.
    fn fun_0200da28(&mut self, p: u32) -> u32;
    /// C: `FUN_0200DA54(P)`: the round number.
    fn fun_0200da54(&mut self, p: u32) -> u32;
    /// C: `FUN_0200DA80(S, bp)`: add BP.
    fn fun_0200da80(&mut self, s: u32, bp: u32);
    /// C: `FUN_0200DAC4(S, mode)`: the best streak.
    fn fun_0200dac4(&mut self, s: u32, mode: u32) -> u32;
    /// C: `FUN_0200DAEC(S, mode, v)`.
    fn fun_0200daec(&mut self, s: u32, mode: u32, v: u32);
    /// C: `FUN_0200DB08(S, mode, v)`.
    fn fun_0200db08(&mut self, s: u32, mode: u32, v: u32);
    /// C: `FUN_0200DB1C(S, mode)`.
    fn fun_0200db1c(&mut self, s: u32, mode: u32);
    /// C: `FUN_0200DB30(S, mode)`.
    fn fun_0200db30(&mut self, s: u32, mode: u32);
    /// C: `FUN_0200DB44(S, mode)`: a flag.
    fn fun_0200db44(&mut self, s: u32, mode: u32) -> u32;
    /// C: `FUN_0200DB60(S, mode)`.
    fn fun_0200db60(&mut self, s: u32, mode: u32);
    /// C: `FUN_0200DB80(S, mode)`: the rounds.
    fn fun_0200db80(&mut self, s: u32, mode: u32) -> u32;
    /// C: `FUN_0200DBA0(S, a, b)` (the spec's stub returns nothing: `r0` = S).
    fn fun_0200dba0(&mut self, s: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_0200DBF0(S)`.
    fn fun_0200dbf0(&mut self, s: u32);
    /// C: `FUN_0200DBFC(S)`.
    fn fun_0200dbfc(&mut self, s: u32);
    /// C: `FUN_0200DC08(S)`: the level.
    fn fun_0200dc08(&mut self, s: u32) -> u32;
    /// C: `FUN_0200DC0C(S, k)`.
    fn fun_0200dc0c(&mut self, s: u32, k: u32) -> u32;
    /// C: `FUN_0200DC50(S, k, &records)`: store the three 0x3C-byte mon records.
    fn fun_0200dc50(&mut self, s: u32, k: u32, records: u32);
    /// C: `FUN_0200DC94(S, P)`.
    fn fun_0200dc94(&mut self, s: u32, p: u32);
    /// C: `FUN_0200DE5C(X)` (the spec's stub returns nothing: `r0` = X).
    fn fun_0200de5c(&mut self, x: u32) -> u32;
    /// C: `FUN_0200DE64(X)` (as `FUN_0200DE5C`).
    fn fun_0200de64(&mut self, x: u32) -> u32;
    /// C: `FUN_0200DE94(X)` (as `FUN_0200DE5C`).
    fn fun_0200de94(&mut self, x: u32) -> u32;
    /// C: `FUN_0200DEA8(X, rec, n, heap)`.
    fn fun_0200dea8(&mut self, x: u32, rec: u32, n: u32, heap: u32) -> u32;
    /// C: `FUN_0200DF40(X)`.
    fn fun_0200df40(&mut self, x: u32) -> u32;
    /// C: `FUN_0200DF4C(X)`.
    fn fun_0200df4c(&mut self, x: u32) -> u32;
    /// C: `FUN_0200DF58(X, heap)`: a copy of the 0x22-byte records (freed with `FUN_020307B0`).
    fn fun_0200df58(&mut self, x: u32, heap: u32) -> u32;
    /// C: `FUN_0200DF94(list)`: the record count.
    fn fun_0200df94(&mut self, list: u32) -> u32;
    /// C: `FUN_0200DFB4(rec)`.
    fn fun_0200dfb4(&mut self, rec: u32) -> u32;
    /// C: `FUN_0200ED48(save)`.
    fn fun_0200ed48(&mut self, save: u32) -> u32;
    /// C: `FUN_0200ED54(x, v)`.
    fn fun_0200ed54(&mut self, x: u32, v: u32);
    /// C: `FUN_0200FB24()` (X1 net): `r0` is the value the original leaves in `r0` (not an argument; the spec logs it under
    /// command 346).
    fn fun_0200fb24(&mut self, r0: u32) -> u32;
    /// C: `FUN_0200FB38(x)` (X1 net).
    fn fun_0200fb38(&mut self, x: u32);
    /// C: `FUN_0201210C(gamesys)`: the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012124(gamesys, x)`.
    fn fun_02012124(&mut self, gamesys: u32, x: u32);
    /// C: `FUN_02012134(gamesys)`.
    fn fun_02012134(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020124AC(gamesys, id, f, &data)`: a GMEVENT with a 16-byte work copied from `data`.
    fn fun_020124ac(&mut self, gamesys: u32, id: u32, f: u32, data: &[u32; 4]) -> u32;
    /// C: `FUN_020127A4(gamedata)`.
    fn fun_020127a4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201289C(gamedata, &rec)`: store a 0x1C-byte record.
    fn fun_0201289c(&mut self, gamedata: u32, rec: &[u8; 0x1C]);
    /// C: `FUN_02012934(gamedata)`: the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)`: the player status.
    fn fun_02012944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)`: the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012ED8(gamedata)`: the work W.
    fn fun_02012ed8(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EE4(gamedata, w)`: set the work W.
    fn fun_02012ee4(&mut self, gamedata: u32, w: u32);
    /// C: `FUN_020130B4(x)`: free the battle setup.
    fn fun_020130b4(&mut self, x: u32);
    /// C: `FUN_02014184(&rec, a, b, c, d, e)`: fill a 0x1C-byte warp record.
    fn fun_02014184(&mut self, rec: &mut [u8; 0x1C], a: u32, b: u32, c: u32, d: u32, e: u32);
    /// C: `FUN_02014808(gamesys, fm, id, procs, work, a, b)`: an app event.
    fn fun_02014808(
        &mut self,
        gamesys: u32,
        fm: u32,
        id: u32,
        procs: u32,
        work: u32,
        a: u32,
        b: u32,
    ) -> u32;
    /// C: `FUN_02017E1C(mon, id, x)`: a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_02019ED4(a, party)`.
    fn fun_02019ed4(&mut self, a: u32, party: u32) -> u32;
    /// C: `FUN_0201A354(k, heap)`: a block (freed with `FUN_020307B0`).
    fn fun_0201a354(&mut self, k: u32, heap: u32) -> u32;
    /// C: `FUN_0201A920(heap)`: a party (freed with `FUN_020307B0`).
    fn fun_0201a920(&mut self, heap: u32) -> u32;
    /// C: `FUN_0201A954(party, n)`.
    fn fun_0201a954(&mut self, party: u32, n: u32);
    /// C: `FUN_0201A98C(party, mon)`: add a mon.
    fn fun_0201a98c(&mut self, party: u32, mon: u32);
    /// C: `FUN_0201AA18(party)`: the party count.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)`: party mon `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_0202B578(x)`.
    fn fun_0202b578(&mut self, x: u32) -> u32;
    /// C: `FUN_0202C0D0(x, i)`.
    fn fun_0202c0d0(&mut self, x: u32, i: u32);
    /// C: `FUN_02030734(heap, size, flag, file, line)` (Heap_AllocDebug).
    fn fun_02030734(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32;
    /// C: `FUN_020307B0(p)`: free a block.
    fn fun_020307b0(&mut self, p: u32);
    /// C: `FUN_02039338()` (X1 net).
    fn fun_02039338(&mut self) -> u32;
    /// C: `FUN_0203CDBC()` (X1 net).
    fn fun_0203cdbc(&mut self) -> u32;
    /// C: `FUN_0203CEAC(x)` (X1 net).
    fn fun_0203ceac(&mut self, x: u32);
    /// C: `FUN_0203D1B4()` (X1 net).
    fn fun_0203d1b4(&mut self) -> u32;
    /// C: `FUN_0203D378()` (X1 net).
    fn fun_0203d378(&mut self) -> u32;
    /// C: `FUN_0203D7BC(x)` (X1 net).
    fn fun_0203d7bc(&mut self, x: u32);
    /// C: `FUN_02087BCC(x)`.
    fn fun_02087bcc(&mut self, x: u32) -> u32;
    // ---- script_sys
    /// C: `FUN_02158F54(sw)`: the GAMESYS.
    fn fun_02158f54(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F58(sw)`.
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158FE0(sw, gd, id)`: script variable `id`.
    fn fun_02158fe0(&mut self, sw: u32, gd: u32, id: u32) -> u32;
    // ---- overlay 10: comm_player.c (X1), fld_btl_inst_tool.c, the OBJ calls
    /// C: `FUN_02167AB0(w)` (X1).
    fn fun_02167ab0(&mut self, w: u32);
    /// C: `FUN_02167B00(w)` (X1).
    fn fun_02167b00(&mut self, w: u32);
    /// C: `FUN_02167B18(w, x)` (X1).
    fn fun_02167b18(&mut self, w: u32, x: u32);
    /// C: `FUN_02167B24(x)` (X1).
    fn fun_02167b24(&mut self, x: u32);
    /// C: `FUN_02167B34(x)` (X1).
    fn fun_02167b34(&mut self, x: u32) -> u32;
    /// C: `FUN_02167BFC(w, a, b)` (X1).
    fn fun_02167bfc(&mut self, w: u32, a: u32, b: u32);
    /// C: `FUN_02167CB4(w)` (X1).
    fn fun_02167cb4(&mut self, w: u32) -> u32;
    /// C: `FUN_02167CF4(w, x)` (X1).
    fn fun_02167cf4(&mut self, w: u32, x: u32);
    /// C: `FUN_02167D00(w, var)` (X1).
    fn fun_02167d00(&mut self, w: u32, var: u32) -> u32;
    /// C: `FUN_021681C8(x, party, mode, rec, trainer, n)` (fld_btl_inst_tool.c): the battle setup.
    fn fun_021681c8(
        &mut self,
        x: u32,
        party: u32,
        mode: u32,
        rec: u32,
        trainer: u32,
        n: u32,
    ) -> u32;
    /// C: `FUN_021684E8(rec, species, a, b, iv, i, x, heap)` (fld_btl_inst_tool.c).
    fn fun_021684e8(
        &mut self,
        rec: u32,
        species: u32,
        a: u32,
        b: u32,
        iv: u32,
        i: u32,
        x: u32,
        heap: u32,
    );
    /// C: `FUN_02168878(rec, id, n, &a, &b, c, heap)` (fld_btl_inst_tool.c): build a trainer.
    fn fun_02168878(&mut self, rec: u32, id: u32, n: u32, a: Buf, b: Buf, c: u32, heap: u32)
        -> u32;
    /// C: `FUN_021688BC(rec, id, heap)` (fld_btl_inst_tool.c): a block (freed with `FUN_020307B0`).
    fn fun_021688bc(&mut self, rec: u32, id: u32, heap: u32) -> u32;
    /// C: `FUN_02168AEC(x)` (fld_btl_inst_tool.c; the spec's stub returns nothing: `r0` = x).
    fn fun_02168aec(&mut self, x: u32) -> u32;
    /// C: `FUN_0216C538(w, gs, k)`: an event (the spec's stub returns nothing: `r0` = w).
    fn fun_0216c538(&mut self, w: u32, gamesys: u32, k: u32) -> u32;
    /// C: `FUN_0216C640(w, gs, fm)`: an event (as `FUN_0216C538`).
    fn fun_0216c640(&mut self, w: u32, gamesys: u32, fm: u32) -> u32;
    /// C: `FUN_0216C6E8(w, gs, a, b, k)`: an event (as `FUN_0216C538`).
    fn fun_0216c6e8(&mut self, w: u32, gamesys: u32, a: u32, b: u32, k: u32) -> u32;
    /// C: `FUN_0216C860(gs)`: an event (the spec's stub returns nothing: `r0` = gs).
    fn fun_0216c860(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_0216CB48(gs, k, id)`: an event (as `FUN_0216C860`).
    fn fun_0216cb48(&mut self, gamesys: u32, k: u32, id: u32) -> u32;
    /// C: `FUN_0216CE6C(objsys, x, z, dir, id, code, a, zone)`: add an OBJ.
    fn fun_0216ce6c(
        &mut self,
        objsys: u32,
        x: u32,
        z: u32,
        dir: u32,
        id: u32,
        code: u32,
        a: u32,
        zone: u32,
    ) -> u32;
    /// C: `FUN_0216D5B4(obj, k)`.
    fn fun_0216d5b4(&mut self, obj: u32, k: u32);
    /// C: `FUN_0216D5E0(obj, x)`.
    fn fun_0216d5e0(&mut self, obj: u32, x: u32) -> u32;
    /// C: `FUN_0216D5F8(obj)`: the direction.
    fn fun_0216d5f8(&mut self, obj: u32) -> u32;
    /// C: `FUN_0216D620(obj, v, k)`: set parameter `k`.
    fn fun_0216d620(&mut self, obj: u32, v: u32, k: u32);
    /// C: `FUN_0216D63C(obj, k)`: parameter `k`.
    fn fun_0216d63c(&mut self, obj: u32, k: u32) -> u32;
    /// C: `FUN_0216D88C(obj, &pos)`: the position.
    fn fun_0216d88c(&mut self, obj: u32, pos: &mut [u32; 3]);
    /// C: `FUN_0216DAC8(obj, x)`.
    fn fun_0216dac8(&mut self, obj: u32, x: u32) -> u32;
    /// C: `FUN_0216DAF8(obj)`.
    fn fun_0216daf8(&mut self, obj: u32) -> u32;
    /// C: `FUN_0216DB4C(obj, x)`.
    fn fun_0216db4c(&mut self, obj: u32, x: u32) -> u32;
    /// C: `FUN_0216DE94(objsys, id)`: the OBJ of an id or null.
    fn fun_0216de94(&mut self, objsys: u32, id: u32) -> u32;
    /// C: `FUN_0216E014(obj, &pos, dir)`: set the position.
    fn fun_0216e014(&mut self, obj: u32, pos: &[u32; 3], dir: u32);
    // ---- overlay 21 and the overlays loaded around the calls
    /// C: `FUN_02188C54(fm)`.
    fn fun_02188c54(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C7C(fm)`: the field OBJ system.
    fn fun_02188c7c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C9C(fm)`: the zone.
    fn fun_02188c9c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CB4(fm)`: the player.
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_021909BC(x)`.
    fn fun_021909bc(&mut self, x: u32);
    /// C: `FUN_02190BE4(x, v)`.
    fn fun_02190be4(&mut self, x: u32, v: u32) -> u32;
    /// C: `FUN_02190C78(x)`: 0 / 1 = done, 2 = running.
    fn fun_02190c78(&mut self, x: u32) -> u32;
    /// C: `FUN_021A3038(player, &pos)`: the player position.
    fn fun_021a3038(&mut self, player: u32, pos: &mut [u32; 3]);
    /// C: `FUN_021A3050(player)`: the direction.
    fn fun_021a3050(&mut self, player: u32) -> u32;
    /// C: `FUN_021A30B0(player)`.
    fn fun_021a30b0(&mut self, player: u32) -> u32;
    /// C: `FUN_021A3190(x, k)`.
    fn fun_021a3190(&mut self, x: u32, k: u32) -> u32;
    /// C: `FUN_021A31A4(x, k)` (the spec's stub returns nothing: `r0` = x).
    fn fun_021a31a4(&mut self, x: u32, k: u32) -> u32;
    /// C: `FUN_021C9924(gamesys)` (command 343; overlay 128, X1).
    fn fun_021c9924(&mut self, gamesys: u32);
    /// C: `FUN_021CD48C(status, fm, a, b, c, d, e, f)` (comm_entry_menu.c, X1).
    fn fun_021cd48c(
        &mut self,
        status: u32,
        fm: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
        f: u32,
    ) -> u32;
    /// C: `FUN_021CD5A4(menu)` (comm_entry_menu.c, X1).
    fn fun_021cd5a4(&mut self, menu: u32);
    /// C: `FUN_021CD688(menu)` (comm_entry_menu.c, X1): 0 = running, else the result.
    fn fun_021cd688(&mut self, menu: u32) -> u32;
    /// C: `FUN_021D024C(x, a)`.
    fn fun_021d024c(&mut self, x: u32, a: u32) -> u32;
    /// C: `FUN_021D028C(x, a)`.
    fn fun_021d028c(&mut self, x: u32, a: u32) -> u32;
    /// C: `FUN_021EEE78(x)`.
    fn fun_021eee78(&mut self, x: u32) -> u32;
    /// C: `FUN_021F37C0(fm, a, rec)`.
    fn fun_021f37c0(&mut self, fm: u32, a: u32, rec: u32) -> u32;
    /// C: `FUN_021F380C(fm)`.
    fn fun_021f380c(&mut self, fm: u32) -> u32;
    /// C: `FUN_021F388C(fm)`.
    fn fun_021f388c(&mut self, fm: u32);
}

// ------------------------------------------------------------------ the work W

/// C: `FUN_overlay_d_21__021c9934`: create W (heap 4, 0x7EC bytes) for a new run of `mode` (`resume` = 0: slots / trainer ids
/// cleared, the best streak as the streak) or resumed from the play record P (mode, party slots, trainer ids, the multi
/// partner of modes 2 / 7, the streak from the rounds); W is set as the GAMEDATA's work.
pub fn work_create<H: BsubwayHooks>(h: &mut H, gamesys: u32, resume: u32, mode: u32) -> u32 {
    let gd = h.fun_020120f4(gamesys);
    let st = h.fun_02012944(gd);
    let save = h.fun_02012ebc(gd);
    let w = h.fun_02030734(W_HEAP, W_SIZE, 1, FILE_BSUBWAY, LINE_W);
    h.w32(w + 4, W_HEAP);
    h.w32(w, W_MAGIC);
    h.w32(w + W_GD, gd);
    let v = h.fun_02008550(st);
    h.w8(w + 0xA, v as u8);
    let p = h.fun_020071cc(save, SAVE_PLAY);
    h.w32(w + W_PLAY, p);
    let s = h.fun_020071cc(save, SAVE_SCORE);
    h.w32(w + W_SCORE, s);
    let x = h.fun_020071cc(save, SAVE_EXTRA);
    h.w32(w + W_EXTRA, x);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d868(p, 0);
    h.fun_02012ee4(gd, w);
    if resume == 0 {
        h.w8(w + W_MODE, mode as u8);
        let m = h.r8(w + W_MODE);
        h.w8(w + W_COUNT, party_size(u32::from(m)));
        for i in 0..4 {
            h.w8(w + W_SLOTS + i, 0xFF);
        }
        for i in 0..14 {
            h.w16(w + W_TRAINERS + 2 * i, 0xFFFF);
        }
        let p = h.r32(w + W_PLAY);
        h.fun_0200d85c(p);
        let p = h.r32(w + W_PLAY);
        h.fun_0200da14(p);
        let (s, m) = (h.r32(w + W_SCORE), u32::from(h.r8(w + W_MODE)));
        if h.fun_0200db44(s, m) == 1 {
            let (s, m) = (h.r32(w + W_SCORE), u32::from(h.r8(w + W_MODE)));
            let v = h.fun_0200dac4(s, m);
            set_streak(h, w, v as u16);
        }
        let m = h.r8(w + W_MODE);
        let p = h.r32(w + W_PLAY);
        h.fun_0200d914(p, 0, &[m]);
        return w;
    }
    let p = h.r32(w + W_PLAY);
    let m = h.fun_0200d884(p, 0, 0);
    h.w8(w + W_MODE, m as u8);
    let m = h.r8(w + W_MODE);
    h.w8(w + W_COUNT, party_size(u32::from(m)));
    let p = h.r32(w + W_PLAY);
    if h.fun_0200d884(p, 10, 0) != 0 {
        alloc_party(h, w);
    }
    let p = h.r32(w + W_PLAY);
    h.fun_0200d884(p, 5, w + W_SLOTS);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d884(p, 8, w + W_TRAINERS);
    let m = h.r8(w + W_MODE);
    if m == 2 || m == 7 {
        let p = h.r32(w + W_PLAY);
        let v = h.fun_0200d884(p, 9, 0) & 0xFF;
        let f = h.r16(w + W_FLAGS);
        h.w16(w + W_FLAGS, (f & !0xE0) | (((v << 29) >> 24) as u16));
        let k = partner(h, w);
        let p = h.r32(w + W_PLAY);
        h.fun_0200d884(p, 6, w + 0x628 + 0x14 * k);
        let mut base = 303u32;
        let a = h.fun_020127a4(gd);
        if h.fun_02008550(a + 0x20) != 0 {
            base -= 3;
        }
        let k = partner(h, w);
        let p = h.r32(w + W_PLAY);
        let x = h.fun_0200d884(p, 7, 0);
        let heap = h.r32(w + 4) & 0xFFFF;
        make_partner(
            h,
            w,
            w + 0x2C8 + 0x120 * k,
            (base + k) & 0xFFFF,
            x,
            w + 0x628 + 0x14 * k,
            heap,
        );
    }
    let m = u32::from(h.r8(w + W_MODE));
    let s = h.r32(w + W_SCORE);
    if h.fun_0200db44(s, m) == 1 {
        let p = h.r32(w + W_PLAY);
        let n = h.fun_0200da54(p);
        let s = h.r32(w + W_SCORE);
        let r = h.fun_0200db80(s, m);
        streak_from(h, w, r, n);
    }
    w
}

/// The multi partner index (bits 5..7 of the W flags).
fn partner<H: MemEnv>(h: &mut H, w: u32) -> u32 {
    u32::from(h.r16(w + W_FLAGS) & 0xFF) >> 5
}

/// C: `FUN_overlay_d_21__021c9b24`: free W (the party block +0x7C8, the battle setup +0x7CC, the work cleared first) and
/// clear the GAMEDATA's work.
pub fn work_free<H: BsubwayHooks>(h: &mut H, gamesys: u32, w: u32) {
    if w != 0 {
        let p = h.r32(w + W_PARTY_7C8);
        if p != 0 {
            h.fun_020307b0(p);
            h.w32(w + W_PARTY_7C8, 0);
        }
        let b = h.r32(w + W_BATTLE);
        if b != 0 {
            h.fun_020130b4(b);
            h.w32(w + W_BATTLE, 0);
        }
        for o in (0..W_SIZE).step_by(4) {
            h.w32(w + o, 0);
        }
        h.fun_020307b0(w);
    }
    let gd = h.fun_020120f4(gamesys);
    h.fun_02012ee4(gd, 0);
}

/// C: `FUN_overlay_d_21__021c9b78`: switch a multi run to its wireless mode (2 -> 3, 7 -> 8; another mode becomes the
/// original's unset stack byte, 0 here: UNVERIFIED), store it, the best streak as the streak (else 0).
pub fn multi_mode<H: BsubwayHooks>(h: &mut H, w: u32) {
    let m = match h.r8(w + W_MODE) {
        2 => 3,
        7 => 8,
        _ => 0,
    };
    h.w8(w + W_MODE, m);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d914(p, 0, &[m]);
    let (s, m) = (h.r32(w + W_SCORE), u32::from(h.r8(w + W_MODE)));
    if h.fun_0200db44(s, m) == 1 {
        let (s, m) = (h.r32(w + W_SCORE), u32::from(h.r8(w + W_MODE)));
        let v = h.fun_0200dac4(s, m);
        set_streak(h, w, v as u16);
    } else {
        zero_streak(h, w);
    }
}

/// Bytes of W at `a` (a field the play record stores).
fn bytes<H: MemEnv>(h: &mut H, a: u32, n: u32) -> Vec<u8> {
    (0..n).map(|i| h.r8(a + i)).collect()
}

/// Store the multi partner fields (9 / 6 / 7) of modes 2 / 7 into the play record.
fn store_partner<H: BsubwayHooks>(h: &mut H, w: u32) {
    let m = h.r8(w + W_MODE);
    if m != 2 && m != 7 {
        return;
    }
    let k = partner(h, w);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d914(p, 9, &[k as u8]);
    let k = partner(h, w);
    let rec = bytes(h, w + 0x628 + 0x14 * k, 0x14);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d914(p, 6, &rec);
    let k = partner(h, w);
    let rec = bytes(h, w + 0x664 + k, 1);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d914(p, 7, &rec);
}

/// C: `FUN_overlay_d_21__021c9bcc`: store the run into the play record (mode, party slots, trainer ids, the flag 1, the multi
/// partner).
pub fn play_save<H: BsubwayHooks>(h: &mut H, w: u32) {
    let m = h.r8(w + W_MODE);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d914(p, 0, &[m]);
    let slots = bytes(h, w + W_SLOTS, 4);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d914(p, 5, &slots);
    let ids = bytes(h, w + W_TRAINERS, 0x1C);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d914(p, 8, &ids);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d868(p, 1);
    store_partner(h, w);
}

/// C: `FUN_overlay_d_21__021c9c54`: the party count of the mode, the slots from the play record and the species / forms of
/// the party mons in them.
pub fn load_party<H: BsubwayHooks>(h: &mut H, w: u32) {
    let m = h.r8(w + W_MODE);
    h.w8(w + W_COUNT, party_size(u32::from(m)));
    let p = h.r32(w + W_PLAY);
    h.fun_0200d884(p, 5, w + W_SLOTS);
    let party = party(h, w);
    let mut i = 0;
    while i < u32::from(h.r8(w + W_COUNT)) {
        let slot = u32::from(h.r8(w + W_SLOTS + i));
        let mon = h.fun_0201ab28(party, slot);
        let v = h.fun_02017e1c(mon, 5, 0);
        h.w16(w + 0x22 + 2 * i, v as u16);
        let v = h.fun_02017e1c(mon, 6, 0);
        h.w16(w + 0x2A + 2 * i, v as u16);
        i += 1;
    }
}

/// C: `FUN_overlay_d_21__021c9cac`: as [`play_save`] without the trainer ids, with `FUN_0200DA14(P)` after the mode.
pub fn play_save_rest<H: BsubwayHooks>(h: &mut H, w: u32) {
    let m = h.r8(w + W_MODE);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d914(p, 0, &[m]);
    let p = h.r32(w + W_PLAY);
    h.fun_0200da14(p);
    let slots = bytes(h, w + W_SLOTS, 4);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d914(p, 5, &slots);
    let p = h.r32(w + W_PLAY);
    h.fun_0200d868(p, 1);
    store_partner(h, w);
}

/// C: `FUN_overlay_d_21__021c9d30`: the mode saved in the play record (u8), clearing its run (`FUN_0200DA14`,
/// `FUN_0200DB1C(S, mode)`).
pub fn saved_mode<H: BsubwayHooks>(h: &mut H, gamesys: u32) -> u8 {
    let gd = h.fun_020120f4(gamesys);
    let save = h.fun_02012ebc(gd);
    let p = h.fun_020071cc(save, SAVE_PLAY);
    let s = h.fun_020071cc(save, SAVE_SCORE);
    let m = h.fun_0200d884(p, 0, 0) & 0xFF;
    h.fun_0200da14(p);
    h.fun_0200db1c(s, m);
    m as u8
}

/// C: `FUN_overlay_d_21__021c9d70`: a won run: the streak into the score record, the party records ([`save_records`]).
pub fn record_win<H: BsubwayHooks>(h: &mut H, _gamesys: u32, w: u32) {
    let n = u32::from(h.r16(w + W_STREAK));
    let m = u32::from(h.r8(w + W_MODE));
    let s = h.r32(w + W_SCORE);
    h.fun_0200db08(s, m, n);
    h.fun_0200daec(s, m, n);
    let gd = h.r32(w + W_GD);
    let save = h.fun_02012ebc(gd);
    save_records(h, w, save);
    let s = h.r32(w + W_SCORE);
    h.fun_0200db1c(s, m);
    let p = h.r32(w + W_PLAY);
    h.fun_0200da14(p);
}

/// C: `FUN_overlay_d_21__021c9db0`: the BP of the finished run (mode 4: by level when the play record's field 0xB is 1, else
/// 5; others: the BP row of the mode by rounds, 10 / 30 with flag bits 1..2 set), at least 1, added to the score record.
pub fn bp_reward<H: BsubwayHooks>(h: &mut H, w: u32) -> u32 {
    let m = h.r8(w + W_MODE);
    let mut bp;
    if m == 4 {
        let p = h.r32(w + W_PLAY);
        if h.fun_0200d884(p, 0xB, 0) & 0xFFFF == 1 {
            let s = h.r32(w + W_SCORE);
            let lv = h.fun_0200dc08(s) as u8 as i8 as i32;
            let i = if lv < 0 { 0 } else { lv.min(9) };
            bp = u32::from(rom8(T_BP_LEVEL + i as u32));
        } else {
            bp = 5;
        }
    } else {
        let s = h.r32(w + W_SCORE);
        let rounds = h.fun_0200db80(s, u32::from(m));
        let (row, multi) = match m {
            0 => (0, false),
            1 => (2, false),
            2 | 3 => (4, false),
            5 => (1, true),
            6 => (3, true),
            7 | 8 => (5, true),
            _ => (0, false),
        };
        let i = rounds.wrapping_sub(1) & 0xFFFF;
        let i = if (i as u16 as i16) < 0 { 0 } else { i.min(9) };
        if (h.r16(w + W_FLAGS) >> 1) & 3 != 0 {
            bp = if multi { 30 } else { 10 };
        } else {
            bp = u32::from(rom8(T_BP_ROWS + 10 * row + i));
        }
    }
    if bp == 0 {
        bp = 1;
    }
    let s = h.r32(w + W_SCORE);
    h.fun_0200da80(s, bp);
    bp
}

/// C: `FUN_overlay_d_21__021c9e8c`: pick the trainer ids of the next rounds (7, or 14 for the multi modes 2 / 3 / 7 / 8 with
/// level at least u16 W+0x18), each new to the list ([`trainer_id`], retried while [`list_has`]).
pub fn pick_trainers<H: BsubwayHooks>(h: &mut H, w: u32) {
    let mode = u32::from(h.r8(w + W_MODE));
    let s = streak(h, w);
    let mut lv = u32::from(streak_rounds(u32::from(s)));
    if mode == 2 || mode == 3 || (mode + 0xF9) & 0xFF <= 1 {
        let lim = u32::from(h.r16(w + 0x18));
        if lv < lim {
            lv = lim;
        }
        for i in 0..14u32 {
            let id = loop {
                let id = trainer_id(h, w, lv, (i / 2) & 0xFF, mode, i & 1);
                if !list_has(h, w + W_TRAINERS, u32::from(id), i & 0xFFFF) {
                    break id;
                }
            };
            h.w16(w + W_TRAINERS + 2 * i, id);
        }
    } else {
        for i in 0..7u32 {
            let id = loop {
                let m = u32::from(h.r8(w + W_MODE));
                let id = trainer_id(h, w, lv, i & 0xFF, m, 0);
                if !list_has(h, w + W_TRAINERS, u32::from(id), i & 0xFFFF) {
                    break id;
                }
            };
            h.w16(w + W_TRAINERS + 2 * i, id);
        }
    }
}

/// C: `FUN_overlay_d_21__021c9f44`: take the party chosen in the selection (W+0x7C.., 1-based; values above 6 become 1) unless
/// W+0x84 is set or W+0x82 is 7 / 8: slots and species / forms; false when not taken.
pub fn party_from_select<H: BsubwayHooks>(h: &mut H, w: u32) -> bool {
    if h.r16(w + 0x84) != 0 || h.r16(w + 0x82).wrapping_sub(7) <= 1 {
        return false;
    }
    let party = party(h, w);
    let mut i = 0u32;
    while i < u32::from(h.r8(w + W_COUNT)) {
        let b = h.r8(w + 0x7C + i);
        if b > 6 {
            h.w8(w + 0x7C + i, 1);
        }
        let b = h.r8(w + 0x7C + i).wrapping_sub(1);
        h.w8(w + W_SLOTS + i, b);
        let mon = h.fun_0201ab28(party, u32::from(b));
        let v = h.fun_02017e1c(mon, 5, 0);
        h.w16(w + 0x22 + 2 * i, v as u16);
        let v = h.fun_02017e1c(mon, 6, 0);
        h.w16(w + 0x2A + 2 * i, v as u16);
        i = (i + 1) & 0xFFFF;
    }
    true
}

/// C: `FUN_overlay_d_21__021c9fd0`: flag bit 0 of W, set once the round number reaches 7.
pub fn round_7_reached<H: BsubwayHooks>(h: &mut H, w: u32) -> bool {
    if h.r16(w + W_FLAGS) & 1 != 0 {
        return true;
    }
    let p = h.r32(w + W_PLAY);
    if h.fun_0200da54(p) < 7 {
        return false;
    }
    let f = h.r16(w + W_FLAGS);
    h.w16(w + W_FLAGS, (f & !1) | 1);
    true
}

/// C: `FUN_overlay_d_21__021c9ffc`: a lost run: the streak and the best streak into the score record, the party records.
pub fn record_loss<H: BsubwayHooks>(h: &mut H, w: u32) {
    let m = u32::from(h.r8(w + W_MODE));
    let n = u32::from(streak(h, w));
    let s = h.r32(w + W_SCORE);
    h.fun_0200db08(s, m, n);
    let s = h.r32(w + W_SCORE);
    let best = h.fun_0200dac4(s, m);
    let s = h.r32(w + W_SCORE);
    h.fun_0200daec(s, m, best);
    let s = h.r32(w + W_SCORE);
    h.fun_0200db60(s, m);
    let gd = h.r32(w + W_GD);
    let save = h.fun_02012ebc(gd);
    save_records(h, w, save);
    let p = h.r32(w + W_PLAY);
    h.fun_0200da14(p);
}

/// C: `FUN_overlay_d_21__021ca048`: clear flag bits 0..2 of W.
pub fn clear_flags<H: MemEnv>(h: &mut H, w: u32) {
    let f = h.r16(w + W_FLAGS) & !1;
    h.w16(w + W_FLAGS, f);
    let f = h.r16(w + W_FLAGS) & !6;
    h.w16(w + W_FLAGS, f);
}

/// C: `FUN_overlay_d_21__021ca05c`: build the opponents of the current round (mode 4: `FUN_0200DEA8`; multi modes 2 / 3 /
/// 7 / 8: the two trainers 2n / 2n + 1, the second with the first's species / items; others: trainer n).
pub fn build_trainers<H: BsubwayHooks>(h: &mut H, w: u32) {
    let p = h.r32(w + W_PLAY);
    let n = h.fun_0200da54(p);
    let heap = h.r32(w + 4) & 0xFFFF;
    match h.r8(w + W_MODE) {
        4 => {
            let x = h.r32(w + W_EXTRA);
            h.fun_0200dea8(x, w + 0x88, n & 0xFF, heap);
        }
        2 | 3 | 7 | 8 => {
            let k = n << 1;
            let id = u32::from(h.r16(w + W_TRAINERS + 2 * k));
            let c = u32::from(h.r8(w + W_COUNT));
            make_trainer(h, w, w + 0x88, id, c, Buf::Addr(0), Buf::Addr(0), 0, heap);
            // The original keeps two u16[2] stack arrays (sp+0x14 ids, sp+0x18 species) next to each other and the saved r4;
            // a count above 2 runs over (`frame` keeps that layout: [0..2] items, [2..4] species, [4..6] the saved r4).
            let mut frame = [0u16; 6];
            let c = u32::from(h.r8(w + W_COUNT));
            for i in 0..c.min(4) {
                let s = h.r16(w + 0xB8 + 0x3C * i) & 0x7FF;
                frame[2 + i as usize] = s;
                let it = h.r16(w + 0xBA + 0x3C * i);
                frame[i as usize] = it;
            }
            let id = u32::from(h.r16(w + W_TRAINERS + 2 * (k + 1)));
            let c = u32::from(h.r8(w + W_COUNT));
            make_trainer(
                h,
                w,
                w + 0x1A8,
                id,
                c,
                Buf::Local(&frame[2..]),
                Buf::Local(&frame),
                0,
                heap,
            );
        }
        _ => {
            let id = u32::from(h.r16(w + W_TRAINERS + 2 * n));
            let c = u32::from(h.r8(w + W_COUNT));
            make_trainer(h, w, w + 0x88, id, c, Buf::Addr(0), Buf::Addr(0), 0, heap);
        }
    }
}

/// C: `FUN_021ca150`: `FUN_02168AEC(u16 W[0x8C + 0x120 i])`.
pub fn trainer_x<H: BsubwayHooks>(h: &mut H, w: u32, i: u32) -> u32 {
    let v = u32::from(h.r16(w.wrapping_add(i.wrapping_mul(0x120)).wrapping_add(0x8C)));
    h.fun_02168aec(v)
}

/// C: `FUN_overlay_d_21__021ca164`: the level of the score record `s` (u8) for k = 0; k = 3: step down (`FUN_0200DBA0(s, 0,
/// 2)`, `FUN_0200DBF0` unless at level 10) -> moved; k = 4: step up when `FUN_0200DC0C(s, 3)` reaches the level's step (0x021DACD0)
/// -> moved; else 0.
pub fn level_step<H: BsubwayHooks>(h: &mut H, s: u32, _gamesys: u32, k: u32) -> u32 {
    let lv = h.fun_0200dc08(s) & 0xFF;
    match k {
        0 => lv,
        3 => {
            h.fun_0200dba0(s, 0, 2);
            if lv == 10 {
                0
            } else {
                h.fun_0200dbf0(s);
                1
            }
        }
        4 => {
            let v = h.fun_0200dc0c(s, 3);
            if lv == 1 || v < u32::from(rom8(T_LEVEL_STEPS.wrapping_add(lv).wrapping_sub(1))) {
                return 0;
            }
            h.fun_0200dbfc(s);
            h.fun_0200dc0c(s, 2);
            h.fun_0200dba0(s, 0, 2);
            1
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_21__021ca1e4`: replace the party block W+0x7C8 (`FUN_0200BAF4(FUN_0200BAC8(save), 4)`; the old one is
/// freed, not cleared).
pub fn alloc_party<H: BsubwayHooks>(h: &mut H, w: u32) {
    let p = h.r32(w + W_PARTY_7C8);
    if p != 0 {
        h.fun_020307b0(p);
    }
    let gd = h.r32(w + W_GD);
    let save = h.fun_02012ebc(gd);
    let x = h.fun_0200bac8(save);
    let b = h.fun_0200baf4(x, 4);
    h.w32(w + W_PARTY_7C8, b);
}

/// C: `FUN_overlay_d_21__021ca210`: the party of the run: the player's, or the block W+0x7C8 when the play record's field 10 is
/// set.
pub fn party<H: BsubwayHooks>(h: &mut H, w: u32) -> u32 {
    let p = h.r32(w + W_PLAY);
    if h.fun_0200d884(p, 0xA, 0) == 0 {
        let gd = h.r32(w + W_GD);
        h.fun_02012934(gd)
    } else {
        h.r32(w + W_PARTY_7C8)
    }
}

/// C: `FUN_overlay_d_21__021ca234`: the win streak (u16 W+0xE).
pub fn streak<H: MemEnv>(h: &mut H, w: u32) -> u16 {
    h.r16(w + W_STREAK)
}

/// C: `FUN_overlay_d_21__021ca238`: set the win streak.
pub fn set_streak<H: MemEnv>(h: &mut H, w: u32, n: u16) {
    h.w16(w + W_STREAK, n);
}

/// C: `FUN_overlay_d_21__021ca23c`: add 1 to the win streak unless it is 0xFFFF.
pub fn inc_streak<H: MemEnv>(h: &mut H, w: u32) {
    let n = h.r16(w + W_STREAK);
    if n < 0xFFFF {
        h.w16(w + W_STREAK, n + 1);
    }
}

/// C: `FUN_overlay_d_21__021ca250`: clear the win streak.
pub fn zero_streak<H: MemEnv>(h: &mut H, w: u32) {
    h.w16(w + W_STREAK, 0);
}

/// C: `FUN_overlay_d_21__021ca258`: streak = min(7 * rounds + n, 0xFFFF) (32-bit wrapping arithmetic, unsigned compare).
pub fn streak_from<H: MemEnv>(h: &mut H, w: u32, rounds: u32, n: u32) {
    let v = rounds.wrapping_mul(7).wrapping_add(n);
    h.w16(w + W_STREAK, v.min(0xFFFF) as u16);
}

/// C: `FUN_overlay_d_21__021ca270`: party size of a subway mode (jump table: modes 0 / 4 / 5 -> 3, 1 / 6 -> 4, 2 / 3 / 7 / 8 -> 2;
/// other values 0).
pub fn party_size(mode: u32) -> u8 {
    match mode {
        0 | 4 | 5 => 3,
        1 | 6 => 4,
        2 | 3 | 7 | 8 => 2,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_21__021ca2a4`: `id` (all 32 bits) is one of the first `n` u16 of `list`.
pub fn list_has<H: MemEnv>(h: &mut H, list: u32, id: u32, n: u32) -> bool {
    let mut i = 0u32;
    while i < n {
        if id == u32::from(h.r16(list + 2 * i)) {
            return true;
        }
        i = (i + 1) & 0xFFFF;
    }
    false
}

/// C: `FUN_overlay_d_21__021ca2cc`: `x / 7` (signed division) as u16: the level of a streak.
pub fn streak_rounds(x: u32) -> u16 {
    ((x as i32) / 7) as u16
}

/// C: `FUN_overlay_d_21__021ca2dc`: store the party records of modes 0 (`k` 0) and 4 (`k` 1, then the play record's mode and
/// round + 1 and `FUN_0200DC94(S, P)`); other modes store nothing.
pub fn save_records<H: BsubwayHooks>(h: &mut H, w: u32, x: u32) {
    match h.r8(w + W_MODE) {
        0 => save_party_records(h, w, x, 0),
        4 => {
            save_party_records(h, w, x, 1);
            let m = h.r8(w + W_MODE);
            let p = h.r32(w + W_PLAY);
            h.fun_0200d914(p, 0, &[m]);
            let p = h.r32(w + W_PLAY);
            let n = h.fun_0200da54(p).wrapping_add(1) as u8;
            let p = h.r32(w + W_PLAY);
            h.fun_0200d914(p, 1, &[n]);
            let (s, p) = (h.r32(w + W_SCORE), h.r32(w + W_PLAY));
            h.fun_0200dc94(s, p);
        }
        _ => {}
    }
}

/// C: `FUN_overlay_d_21__021ca338`: the three frontier trainers (ids 300.. or 303.. with `flag`): their result bytes into
/// W+0x664...
pub fn frontier_trainers<H: BsubwayHooks>(h: &mut H, w: u32, flag: u32) {
    let base = if flag != 0 { 303 } else { 300 };
    for i in 0..3u32 {
        let c = u32::from(h.r8(w + W_COUNT));
        let heap = h.r32(w + 4) & 0xFFFF;
        let r = make_trainer(
            h,
            w,
            w + 0x2C8 + 0x120 * i,
            (base + i) & 0xFFFF,
            c,
            Buf::Addr(w + 0x22),
            Buf::Addr(w + 0x2A),
            w + 0x628 + 0x14 * i,
            heap,
        );
        h.w8(w + 0x664 + i, r as u8);
    }
}

/// C: `FUN_overlay_d_21__021ca3b4`: the 0x3C-byte record of `mon`: u16 +0 species (bits 0..10) / form (11..15), +2 item,
/// +4 four moves, +0x1E their PP-up bits, +0x1F, +0xC, +0x10, +0x14, +0x18 six bytes, +0x20, +0x21, the name at +0x22.
pub fn mon_record<H: BsubwayHooks>(h: &mut H, rec: u32, mon: u32) {
    let s = h.fun_02017e1c(mon, 5, 0) & 0xFFFF;
    let x = h.r16(rec);
    h.w16(rec, (x & 0xF800) | (s & 0x7FF) as u16);
    let f = h.fun_02017e1c(mon, 0x6F, 0) & 0xFFFF;
    let x = h.r16(rec);
    h.w16(rec, (x & 0x07FF) | (((f << 27) >> 16) as u16));
    let v = h.fun_02017e1c(mon, 6, 0);
    h.w16(rec + 2, v as u16);
    for i in 0..4u32 {
        let v = h.fun_02017e1c(mon, 0x36 + i, 0);
        h.w16(rec + 4 + 2 * i, v as u16);
        let v = h.fun_02017e1c(mon, 0x3E + i, 0);
        let b = h.r8(rec + 0x1E) | ((v << (2 * i)) & 0xFF) as u8;
        h.w8(rec + 0x1E, b);
    }
    let v = h.fun_02017e1c(mon, 0xC, 0);
    h.w8(rec + 0x1F, v as u8);
    let v = h.fun_02017e1c(mon, 7, 0);
    h.w32(rec + 0xC, v);
    let v = h.fun_02017e1c(mon, 0, 0);
    h.w32(rec + 0x10, v);
    let v = h.fun_02017e1c(mon, 0xAC, 0);
    h.w32(rec + 0x14, v);
    for j in 0..6u32 {
        let v = h.fun_02017e1c(mon, 0xD + j, 0);
        h.w8(rec + 0x18 + j, v as u8);
    }
    let v = h.fun_02017e1c(mon, 0xA, 0);
    h.w8(rec + 0x20, v as u8);
    let v = h.fun_02017e1c(mon, 9, 0);
    h.w8(rec + 0x21, v as u8);
    h.fun_02017e1c(mon, 0x74, rec + 0x22);
}

/// C: `FUN_overlay_d_21__021ca4c0`: the records of the three party mons in the slots ([`mon_record`], a cleared 0xB4-byte
/// block of heap `W[4] | 0x8000`) into the score record (`FUN_0200DC50(S, k, block)`); the block is cleared and freed.
pub fn save_party_records<H: BsubwayHooks>(h: &mut H, w: u32, _x: u32, k: u32) {
    let heap = ((h.r32(w + 4) & 0xFFFF) & 0x7FFF) | 0x8000;
    let blk = h.fun_02030734(heap, 0xB4, 0, FILE_BSUBWAY, LINE_RECORDS);
    for o in 0..0xB4 {
        h.w8(blk + o, 0);
    }
    let party = party(h, w);
    for i in 0..3u32 {
        let slot = u32::from(h.r8(w + W_SLOTS + i));
        let mon = h.fun_0201ab28(party, slot);
        mon_record(h, blk + 0x3C * i, mon);
    }
    let s = h.r32(w + W_SCORE);
    h.fun_0200dc50(s, k, blk);
    for o in 0..0xB4 {
        h.w8(blk + o, 0);
    }
    h.fun_020307b0(blk);
}

// ------------------------------------------------------------------ commands 343..345

/// C: `sub_021ca544` (command 344): create W ([`work_create`]: resume flag = u16 script word, mode = operand).
pub fn cmd_344<H: BsubwayHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158f58(sw);
    let gs = h.fun_02158f54(sw);
    let resume = h.fun_02011330(vm);
    let mode = h.fun_02159b10(vm, ctx);
    work_create(h, gs, resume, mode);
    0
}

/// C: `sub_021ca57c` (command 343): `FUN_021C9924(gs)` (overlay 128, X1).
pub fn cmd_343<H: BsubwayHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158f58(sw);
    let gs = h.fun_02158f54(sw);
    h.fun_021c9924(gs);
    0
}

/// C: `sub_021ca598` (command 345): free W ([`work_free`]).
pub fn cmd_345<H: BsubwayHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158f58(sw);
    let gs = h.fun_02158f54(sw);
    let gd = h.fun_020120f4(gs);
    let w = h.fun_02012ed8(gd);
    work_free(h, gs, w);
    0
}

// ------------------------------------------------------------------ command 346

/// How a sub-command of command 346 ends.
enum End {
    /// Return the result word (0, or 1 for 0x68 / 0x194).
    Ret,
    /// Store the low half of the value into the var, then return the result word.
    Store(u32),
    /// `FUN_0215897C(sw, event)` and return 1.
    Event(u32),
    /// Return 1 (a wait callback was installed).
    Wait,
}

/// C: `FUN_overlay_d_21__021ca5c4` (command 346): the Battle Subway sub-commands (u16 script word `cmd`, operands `a` / `b`,
/// a var). Every sub-command first reads the ctx, W (`FUN_02012ED8`), the play / score / extra records and the operands;
/// without W a sub-command from 0x12C on returns 0 at once. The cases are listed by sub-command below (UNVERIFIED roles).
pub fn cmd_346<H: BsubwayHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158f58(sw);
    let gs = h.fun_02158f54(sw);
    let gd = h.fun_020120f4(gs);
    let save = h.fun_02012ebc(gd);
    let x44 = h.fun_0201296c(gd);
    let w = h.fun_02012ed8(gd);
    let p = h.fun_020071cc(save, SAVE_PLAY);
    let pmode = h.fun_0200d884(p, 0, 0);
    let s = h.fun_020071cc(save, SAVE_SCORE);
    let x = h.fun_020071cc(save, SAVE_EXTRA);
    let fm = h.fun_0201210c(gs);
    let cmd = h.fun_02011330(vm);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let varid = h.fun_02011330(vm);
    let var = h.fun_02158fe0(sw, gd, varid);
    if w == 0 && cmd >= 0x12C {
        return 0;
    }
    let end = match cmd {
        0x00 => End::Store(zone_kind(h, fm)),
        0x01 => {
            h.fun_02087bcc(0);
            End::Ret
        }
        0x02 => {
            h.fun_0200d85c(p);
            End::Ret
        }
        0x03 => End::Store(h.fun_0200d87c(p)),
        0x04 => {
            let pl = h.fun_02188cb4(fm);
            let d = h.fun_021a3050(pl);
            let mut pos = [0u32; 3];
            h.fun_021a3038(pl, &mut pos);
            let k = if d <= 3 { d + 1 } else { 0 };
            let zone = h.fun_02188c9c(fm);
            warp_record(h, gd, x44, zone, k, pos[0], pos[1], pos[2]);
            End::Ret
        }
        0x05 => {
            h.fun_0201433c(x44, FLAG_966);
            End::Ret
        }
        0x06 => {
            let v = h.fun_0200dac4(s, a);
            h.w16(var, v as u16);
            if u32::from(h.r16(var)) <= CAP_9999 {
                End::Ret
            } else {
                End::Store(CAP_9999)
            }
        }
        0x07 => End::Store(u32::from(saved_mode(h, gs))),
        0x08 => End::Store(h.fun_0200da54(p).wrapping_add(1)),
        0x0B => {
            let pw = h.r32(w + W_PLAY);
            let v = h.fun_0200da54(pw);
            h.w16(var, v as u16);
            End::Store(u32::from(h.r16(var)) + 1)
        }
        0x0C => {
            let pl = h.fun_02188cb4(fm);
            let y = h.fun_021a30b0(pl);
            End::Store(h.fun_021a3190(y, 0))
        }
        0x0D => {
            let o = obj(h, fm, a);
            if o != 0 {
                h.fun_0216dac8(o, b);
            }
            End::Ret
        }
        0x0E => {
            let t = u32::from(rom16(T_BY_VALUE.wrapping_add(a << 1)));
            if t == 0xB {
                End::Store(1)
            } else {
                End::Store(h.fun_0200dba0(s, t, 0))
            }
        }
        0x0F => {
            let t = u32::from(rom16(T_BY_VALUE.wrapping_add(a << 1)));
            if t != 0xB {
                h.fun_0200dba0(s, t, 1);
            }
            End::Ret
        }
        0x10 => End::Store(h.fun_0200dba0(s, 0xA, 0)),
        0x11 => {
            h.fun_0200dba0(s, 0xA, 1);
            End::Ret
        }
        0x12 => {
            let v = rom32(T_U32_88.wrapping_add(a << 2));
            h.fun_021f37c0(fm, v, T_REC_70.wrapping_add(b.wrapping_mul(0xC)));
            End::Ret
        }
        0x13 => {
            let y = h.fun_021f380c(fm);
            h.fun_021d024c(y, a);
            End::Ret
        }
        0x14 => {
            let y = h.fun_021f380c(fm);
            h.fun_021d028c(y, a);
            End::Ret
        }
        0x15 => End::Store(h.fun_0200d884(p, 0, 0)),
        0x16 => {
            h.w16(var, 0);
            if matches!(a, 2 | 3 | 7 | 8) {
                End::Store(1)
            } else {
                End::Ret
            }
        }
        0x17 => {
            let y = b << 16;
            let o = obj(h, fm, a);
            if o != 0 {
                let mut pos = [0u32; 3];
                h.fun_0216d88c(o, &mut pos);
                pos[1] = y;
                let d = h.fun_0216d5f8(o);
                h.fun_0216e014(o, &pos, d);
                h.fun_0216db4c(o, 1);
            }
            End::Ret
        }
        0x18 => {
            let o = obj(h, fm, a);
            if o != 0 {
                h.fun_0216db4c(o, 0);
            }
            End::Ret
        }
        0x19 => {
            let o = obj(h, fm, a);
            h.w16(var, 0);
            if o != 0 {
                End::Store(1)
            } else {
                End::Ret
            }
        }
        0x1A => {
            let o = obj(h, fm, a);
            h.w16(var, 0);
            if o != 0 {
                End::Store(obj_npc_id(h, o, 0))
            } else {
                End::Ret
            }
        }
        0x1B => {
            let v = level_step(h, s, gs, 0);
            h.w16(var, v as u16);
            if h.r16(var) as i16 > 0 {
                End::Ret
            } else {
                End::Store(1)
            }
        }
        0x1C => End::Store(level_step(h, s, gs, 3)),
        0x1D => End::Store(level_step(h, s, gs, 4)),
        0x1E => End::Store(h.fun_0200db80(s, a)),
        0x1F => {
            let y = h.fun_020127a4(gd);
            let f = u32::from(h.fun_02008550(y + 0x20) == 0);
            End::Store(h.fun_021a31a4(f, 0))
        }
        0x20 => {
            let y = h.fun_0200bac8(save);
            h.w16(var, 0);
            if h.fun_0200bba8(y) == 1 {
                End::Store(1)
            } else {
                End::Ret
            }
        }
        0x22 => {
            h.fun_021f37c0(fm, 7, T_REC_70);
            End::Ret
        }
        0x23 => {
            let o = obj(h, fm, a);
            if o != 0 {
                h.fun_0216d5e0(o, b);
            }
            End::Ret
        }
        0x24 => {
            let o = obj(h, fm, a);
            if o != 0 {
                h.fun_0216daf8(o);
            }
            End::Ret
        }
        0x25 => {
            let o = obj(h, fm, a);
            let k = h.fun_0216d63c(o, 0);
            End::Event(h.fun_0216cb48(gs, k, a))
        }
        0x26 => End::Store(h.fun_0200db44(s, a)),
        0x27 => {
            let o = obj(h, fm, a);
            let k = h.fun_0216d63c(o, 0);
            h.w16(var, 0);
            if npc_has_talk(h, w, k) {
                End::Store(1)
            } else {
                End::Ret
            }
        }
        0x28..=0x2A => {
            let o = obj(h, fm, a);
            let k = h.fun_0216d63c(o, 0);
            let (t0, t1, t2) = npc_talk(h, w, k);
            End::Store(u32::from(match cmd {
                0x28 => t0,
                0x29 => t1,
                _ => t2,
            }))
        }
        0x2B => {
            let l = h.fun_0200df58(x, 4);
            let n = h.fun_0200df94(l);
            h.w16(var, n as u16);
            h.fun_020307b0(l);
            End::Ret
        }
        0x2C => {
            let f = u32::from(a == 1);
            End::Event(event_934_create(h, gs, ctx, var, f))
        }
        0x2D => {
            let z = zone_kind(h, fm) & 0xFFFF;
            warp_record_of(h, gd, x44, z);
            End::Ret
        }
        0x64 => {
            if a == 0 {
                h.fun_0200dba0(s, 1, 2);
            } else {
                h.fun_0200dba0(s, 1, 1);
            }
            End::Ret
        }
        0x65 => End::Store(h.fun_0200dba0(s, 1, 0)),
        0x67..=0x69 => {
            let data = [cmd - 0x67, var, 0, 0];
            End::Event(h.fun_020124ac(gs, 7, EVENT_67, &data))
        }
        0x6A => End::Store(h.fun_0200de94(x)),
        0x6B => End::Store(h.fun_0200de5c(x)),
        0x6C => {
            h.fun_0200d914(p, 0xB, &[a as u8]);
            End::Ret
        }
        0x6D => End::Store(h.fun_0200d884(p, 0xB, 0)),
        0x6E => End::Store(h.fun_0200de64(x)),
        0x6F => End::Event(h.fun_0216c860(gs)),
        0x70 => End::Store(h.fun_0200df40(x)),
        0x71 => End::Store(h.fun_0200df4c(x)),
        0x21 | 0xC8 | 0xCB | 0x12E => End::Store(0),
        0x12C => {
            let k = h.fun_0200d884(p, 0xA, 0) & 0xFF;
            End::Event(h.fun_0216c538(w, gs, k))
        }
        0x12D => End::Store(u32::from(party_from_select(h, w))),
        0x12F => End::Store(u32::from(round_7_reached(h, w))),
        0x130 => {
            record_win(h, gs, w);
            End::Ret
        }
        0x131 => {
            record_loss(h, w);
            End::Ret
        }
        0x132 => {
            play_save(h, w);
            End::Ret
        }
        0x133 => {
            build_trainers(h, w);
            End::Ret
        }
        0x134 => End::Store(trainer_x(h, w, a)),
        0x135 => End::Event(h.fun_0216c640(w, gs, fm)),
        0x136 => End::Store(u32::from(h.r8(w + W_MODE))),
        0x137 => {
            let f = h.r16(w + W_FLAGS);
            h.w16(w + W_FLAGS, (f & !6) | (((a << 30) >> 29) as u16));
            End::Ret
        }
        0x138 => End::Store(u32::from((h.r16(w + W_FLAGS) >> 1) & 3)),
        0x139 => End::Store(bp_reward(h, w)),
        0x13A => {
            let f = h.r16(w + W_FLAGS);
            h.w16(w + W_FLAGS, (f & !0xE0) | (((a << 29) >> 24) as u16));
            End::Ret
        }
        0x13B => End::Store(partner(h, w)),
        0x13C => {
            pick_trainers(h, w);
            End::Ret
        }
        0x13D => End::Store(u32::from(h.r8(w.wrapping_add(a).wrapping_add(W_SLOTS)))),
        0x13F => {
            h.w8(w + 0x724, 0);
            for o in 0..0x46 {
                h.w8(w + 0x6AE + o, 0);
            }
            End::Ret
        }
        0x140 => End::Event(h.fun_0216c6e8(w, gs, a, b, 5)),
        0x160 => End::Event(h.fun_0216c6e8(w, gs, a, b, 6)),
        0x141 => {
            play_save_rest(h, w);
            End::Ret
        }
        0x142 => {
            if pmode < 9 {
                warp_record_of(h, gd, x44, pmode);
            }
            End::Ret
        }
        0x143 => {
            load_party(h, w);
            End::Ret
        }
        0x144 => {
            h.fun_0200d914(p, 0xA, &[a as u8]);
            End::Ret
        }
        0x145 => {
            multi_mode(h, w);
            End::Ret
        }
        0x146 => End::Store(u32::from(pmode.wrapping_sub(5) <= 3)),
        0x149 => {
            let objsys = h.fun_02188c7c(fm);
            place_npcs(h, w, gd, objsys, fm);
            End::Ret
        }
        0x14A => {
            let f = h.r16(w + W_FLAGS);
            h.w16(w + W_FLAGS, (f & 0xFEFF) | (((a << 31) >> 23) as u16));
            End::Ret
        }
        0x14B => End::Store(u32::from((h.r16(w + W_FLAGS) >> 8) & 1)),
        0x14C => {
            let y = h.r32(w + W_BATTLE);
            End::Store(u32::from(y != 0 && h.r32(y + 0x8C) == 1))
        }
        0x14D => {
            clear_flags(h, w);
            End::Ret
        }
        0x14E => {
            let n = h.fun_0200da54(p);
            if n != 0 {
                let y = h.fun_02012134(gs);
                let e = h.fun_0202b578(y);
                let m = n.wrapping_add(1) & 0xFFFF;
                for i in 0..m {
                    if i < 9 && i != 0 {
                        h.fun_0202c0d0(e, i);
                    }
                }
            }
            End::Ret
        }
        0x14F => {
            let y = h.fun_020127a4(gd);
            let f = u32::from(h.fun_02008550(y + 0x20) == 0);
            frontier_trainers(h, w, f);
            End::Ret
        }
        0x150 => End::Store(u32::from(h.r16(w.wrapping_add(a << 1).wrapping_add(0x22)))),
        0x151 => {
            let f = h.r16(w + W_FLAGS);
            h.w16(w + W_FLAGS, (f & 0xFDFF) | (((a << 31) >> 22) as u16));
            End::Ret
        }
        0x152 => End::Store(u32::from((h.r16(w + W_FLAGS) >> 9) & 1)),
        0x153 => {
            h.w16(var, 0);
            let f = h.fun_0200d884(p, 0xA, 0);
            let k = mode_kind(pmode);
            let pt = h.fun_0201a354(k, 4);
            let (blk, party) = if f == 1 {
                let y = h.fun_0200bac8(save);
                let blk = h.fun_0200baf4(y, 4);
                (blk, blk)
            } else {
                (0, h.fun_02012934(gd))
            };
            let r = h.fun_02019ed4(pt, party);
            h.fun_020307b0(pt);
            if blk != 0 {
                h.fun_020307b0(blk);
            }
            if r <= 1 {
                End::Store(1)
            } else {
                End::Ret
            }
        }
        0x154 => {
            alloc_party(h, w);
            End::Ret
        }
        0x155 => End::Store(mode_kind(pmode)),
        0x156 => End::Store(u32::from(party_size(pmode & 0xFFFF))),
        0x157 => {
            let f = h.r16(w + W_FLAGS);
            if (f >> 10) & 3 == 0 {
                h.w16(w + W_FLAGS, (f & 0xF3FF) | 0x400);
                let mut out = 0u32;
                if h.fun_0200b5cc(save, 4, &mut out, 0) == 1 {
                    let f = h.r16(w + W_FLAGS);
                    h.w16(w + W_FLAGS, (f & 0xF3FF) | 0x800);
                }
            }
            End::Ret
        }
        0x158 => End::Store(u32::from((h.r16(w + W_FLAGS) >> 10) & 3 != 1)),
        0x159 => {
            h.fun_0200ba58();
            let y = h.r32(w + W_BATTLE);
            let r = h.fun_021eee78(y);
            h.fun_0200ba68(r);
            End::Ret
        }
        0x15A => {
            h.w16(w + 0x7E8, 0);
            h.w16(w + 0x7EA, 0);
            h.w16(w + 0x10, a as u16);
            h.fun_020113d0(vm, WAIT_C14);
            let f = h.r16(w + W_FLAGS);
            h.w16(w + W_FLAGS, (f & 0xF3FF) | 0x800);
            End::Wait
        }
        0x15B => {
            if h.r32(w + W_BATTLE) != 0 {
                h.fun_0200b424();
                let y = h.r32(w + W_BATTLE);
                h.fun_020130b4(y);
                h.w32(w + W_BATTLE, 0);
            }
            End::Ret
        }
        0x15C => {
            for o in (0..0x18).step_by(4) {
                h.w32(w + 0x7D0 + o, 0);
            }
            h.w32(w + 0x7D0, gd);
            h.w32(w + 0x7D4, 1);
            h.w32(w + 0x7D8, 0);
            h.w32(w + 0x7DC, 1);
            let v = u32::from(rom16(T_BY_MODE.wrapping_add(pmode << 1)));
            h.w32(w + 0x7E0, v);
            let n = u32::from(streak(h, w));
            h.w32(w + 0x7E4, n);
            if h.r32(w + 0x7E4) as i32 > CAP_9999 as i32 {
                h.w32(w + 0x7E4, CAP_9999);
            }
            End::Event(h.fun_02014808(gs, fm, PROC_15C.0, PROC_15C.1, w + 0x7D0, 0, 0))
        }
        0x15D => {
            h.fun_0200da14(p);
            h.fun_0200db1c(s, pmode);
            End::Ret
        }
        0x15E => {
            h.fun_021f388c(fm);
            End::Ret
        }
        0x15F => {
            let n = h.fun_0200da54(p);
            let i = match pmode {
                2 | 3 | 7 | 8 => a.wrapping_add(n << 1),
                _ => n,
            };
            End::Store(u32::from(
                h.r16(w.wrapping_add(i << 1).wrapping_add(W_TRAINERS)),
            ))
        }
        0x161 => {
            let y = h.r32(w + W_BATTLE);
            if y != 0 {
                let c = u32::from(h.r8(y + 0xB6));
                let d = u32::from(h.r8(y + 0xBF));
                let party = h.r32(y + 0x24);
                let n = h.fun_0201aa18(party) as i32;
                let (mut s6, mut s5) = (0u32, 0u32);
                for i in 0..n.max(0) as u32 {
                    let mon = h.fun_0201ab28(party, i);
                    s6 = s6.wrapping_add(h.fun_02017e1c(mon, 0xA0, 0));
                    s5 = s5.wrapping_add(h.fun_02017e1c(mon, 0xA1, 0));
                }
                h.fun_0200d9e8(p, d, c, s5.wrapping_sub(s6) & 0xFFFF);
            }
            End::Ret
        }
        0x162 => {
            let f = h.r16(w + W_FLAGS);
            h.w16(w + W_FLAGS, (f & 0xEFFF) | (((a << 31) >> 19) as u16));
            End::Ret
        }
        0x163 => End::Store(u32::from((h.r16(w + W_FLAGS) >> 12) & 1)),
        0x164 => End::Store(h.fun_0200d884(p, 0xA, 0)),
        0x165 => {
            h.fun_0200da28(p);
            inc_streak(h, w);
            let n = streak(h, w);
            h.w16(var, n);
            if u32::from(h.r16(var)) <= CAP_9999 {
                End::Ret
            } else {
                End::Store(CAP_9999)
            }
        }
        0x166 => {
            let n = streak(h, w);
            h.w16(var, n);
            if u32::from(h.r16(var)) <= CAP_9999 {
                End::Ret
            } else {
                End::Store(CAP_9999)
            }
        }
        0x167 => {
            if h.fun_0200db44(s, pmode) == 0 {
                h.fun_0200db30(s, pmode);
                h.fun_0200da14(p);
                zero_streak(h, w);
            }
            End::Ret
        }
        0x190 => {
            if (h.r16(w + W_FLAGS) >> 9) & 1 == 1 {
                h.fun_02167b00(w);
            } else {
                h.fun_02167ab0(w);
            }
            End::Ret
        }
        0x191 => End::Event(disconnect_create(h, gs, w)),
        0x192 => {
            h.w8(w + 0x727, a as u8);
            h.w16(w + 0x72A, varid as u16);
            let y = u32::from(h.r8(w + 0x727));
            h.fun_02167b24(y);
            h.fun_020113d0(vm, WAIT_700);
            End::Wait
        }
        0x193 | 0x194 => {
            h.w32(w + 0x720, var);
            let st = h.fun_02012944(gd);
            let k = u32::from(cmd == 0x194);
            let m = h.fun_021cd48c(st, fm, 2, 2, 4, k, 2, 0);
            h.w32(w + 0x71C, m);
            h.fun_020113d0(vm, if cmd == 0x193 { WAIT_754 } else { WAIT_7E8 });
            End::Wait
        }
        0x195 => {
            h.fun_02167bfc(w, a, b);
            h.fun_020113d0(vm, WAIT_6DC);
            End::Wait
        }
        0x196 => {
            h.fun_02167cf4(w, a & 0xFF);
            h.w16(w + 0x72A, varid as u16);
            h.fun_020113d0(vm, WAIT_86C);
            End::Wait
        }
        0x197 => End::Store(h.fun_02039338()),
        0x198 => {
            for o in (0..0x24).step_by(4) {
                h.w32(w + 0x7A4 + o, 0);
            }
            h.w32(w + 0x7A4, gd);
            h.w32(w + 0x7C0, 0xB);
            End::Event(h.fun_02014808(gs, fm, PROC_198.0, PROC_198.1, w + 0x7A4, 0, 0))
        }
        0x199 => End::Store(match h.r32(w + 0x7C0) {
            0xB => 0,
            0xD => 2,
            _ => 1,
        }),
        0x19A => {
            let y = h.fun_0200ed48(save);
            let v = h.r32(w + 0x73C);
            h.fun_0200ed54(y, v);
            End::Ret
        }
        0x19B | 0x19C => {
            let local = (h.fun_02039338() == 0) == (cmd == 0x19B);
            let y = if local {
                let pl = h.fun_02188cb4(fm);
                h.fun_021a30b0(pl)
            } else {
                u32::from(h.r8(w + 0x749))
            };
            End::Store(h.fun_021a3190(y, 0))
        }
        0x19D => {
            // r0 still holds the jump-table offset of this case (0xB4C) when the original calls FUN_0200FB38.
            h.fun_0200fb38(0xB4C);
            End::Ret
        }
        0x19E => {
            h.fun_02012124(gs, 1);
            End::Ret
        }
        0x19F => {
            h.w16(var, 0);
            if h.fun_0200fb24(0) != 0 {
                End::Store(1)
            } else {
                End::Ret
            }
        }
        0x1A0 => {
            h.w16(var, 0);
            let y = h.r32(w + W_BATTLE);
            if y != 0 {
                let v = u32::from(h.r8(y + 0x91));
                if h.fun_0200ba38(v) == 1 {
                    End::Store(1)
                } else {
                    End::Ret
                }
            } else {
                End::Ret
            }
        }
        _ => End::Ret,
    };
    let ret = u32::from(cmd == 0x68 || cmd == 0x194);
    match end {
        End::Ret => ret,
        End::Store(v) => {
            h.w16(var, v as u16);
            ret
        }
        End::Event(ev) => {
            h.fun_0215897c(sw, ev);
            1
        }
        End::Wait => 1,
    }
}

/// The OBJ of an id (`FUN_0216DE94(FUN_02188C7C(fm), id)`) or null.
fn obj<H: BsubwayHooks>(h: &mut H, fm: u32, id: u32) -> u32 {
    let objsys = h.fun_02188c7c(fm);
    h.fun_0216de94(objsys, id)
}

/// The warp record of command 346's sub-commands 4 / 0x2D / 0x142: `FUN_02014184(&rec, a, b, c, d, e)`, stored with
/// `FUN_0201289C(gd, &rec)`, flag 0x966 set.
#[allow(clippy::too_many_arguments)]
fn warp_record<H: BsubwayHooks>(
    h: &mut H,
    gd: u32,
    x44: u32,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
) {
    let mut rec = [0u8; 0x1C];
    h.fun_02014184(&mut rec, a, b, c, d, e);
    h.fun_0201289c(gd, &rec);
    h.fun_02014314(x44, FLAG_966);
}

/// [`warp_record`] of ROM row `i` (u32 at 0x021DADAC + 4 i, 4, the 12-byte record 0x021DAE50 + 0xC i).
fn warp_record_of<H: BsubwayHooks>(h: &mut H, gd: u32, x44: u32, i: u32) {
    let e = T_REC_50.wrapping_add(i.wrapping_mul(0xC));
    let (c, d, f) = (rom32(e), rom32(e + 4), rom32(e + 8));
    let a = rom32(T_U32_AC.wrapping_add(i << 2));
    warp_record(h, gd, x44, a, 4, c, d, f);
}

// ------------------------------------------------------------------ wait callbacks and events of command 346

/// The SW, GAMEDATA and W of a wait callback.
fn wait_ctx<H: BsubwayHooks>(h: &mut H, ctx: u32) -> (u32, u32, u32) {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_02158f54(sw);
    let gd = h.fun_020120f4(gs);
    let w = h.fun_02012ed8(gd);
    (sw, gd, w)
}

/// C: `sub_021cb6dc` (wait of sub-command 0x195): `FUN_02167CB4(W) == 1` (X1).
pub fn wait_6dc<H: BsubwayHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let (_, _, w) = wait_ctx(h, ctx);
    u32::from(h.fun_02167cb4(w) == 1)
}

/// C: `sub_021cb700` (wait of sub-command 0x192): the var (id u16 W+0x72A) = 0 once `FUN_0200FB24()` says so, or 1 once
/// `FUN_02167B34(W[0x727]) == 1` (X1).
pub fn wait_700<H: BsubwayHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let (sw, gd, w) = wait_ctx(h, ctx);
    let id = u32::from(h.r16(w + 0x72A));
    let var = h.fun_02158fe0(sw, gd, id);
    if h.fun_0200fb24(var) != 0 {
        h.w16(var, 0);
        return 1;
    }
    let x = u32::from(h.r8(w + 0x727));
    if h.fun_02167b34(x) == 1 {
        h.w16(var, 1);
        1
    } else {
        0
    }
}

/// The common part of the comm-entry-menu waits: the ctx reads, the records, then the menu result (0 = running; else the
/// menu is freed).
fn menu_result<H: BsubwayHooks>(h: &mut H, ctx: u32) -> (u32, u32) {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158f58(sw);
    let gs = h.fun_02158f54(sw);
    let gd = h.fun_020120f4(gs);
    let save = h.fun_02012ebc(gd);
    h.fun_0201296c(gd);
    let w = h.fun_02012ed8(gd);
    let p = h.fun_020071cc(save, SAVE_PLAY);
    h.fun_0200d884(p, 0, 0);
    h.fun_020071cc(save, SAVE_SCORE);
    let m = h.r32(w + 0x71C);
    let r = h.fun_021cd688(m);
    if r != 0 {
        let m = h.r32(w + 0x71C);
        h.fun_021cd5a4(m);
        h.w32(w + 0x71C, 0);
    }
    (w, r)
}

/// C: `sub_021cb754` (wait of sub-command 0x193): the comm entry menu's result into the var W+0x720 (1 -> 0, 2 -> 1, else 2).
pub fn wait_754<H: BsubwayHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let (w, r) = menu_result(h, ctx);
    if r == 0 {
        return 0;
    }
    let var = h.r32(w + 0x720);
    h.w16(
        var,
        match r {
            1 => 0,
            2 => 1,
            _ => 2,
        },
    );
    1
}

/// C: `sub_021cb7e8` (wait of sub-command 0x194): as [`wait_754`] with 1 -> 0, else 1.
pub fn wait_7e8<H: BsubwayHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let (w, r) = menu_result(h, ctx);
    if r == 0 {
        return 0;
    }
    let var = h.r32(w + 0x720);
    h.w16(var, u16::from(r != 1));
    1
}

/// C: `FUN_overlay_d_21__021cb86c` (wait of sub-command 0x196): `FUN_02167D00(W, var) == 1` (var id u16 W+0x72A; X1).
pub fn wait_86c<H: BsubwayHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let (sw, gd, w) = wait_ctx(h, ctx);
    let id = u32::from(h.r16(w + 0x72A));
    let var = h.fun_02158fe0(sw, gd, id);
    u32::from(h.fun_02167d00(w, var) == 1)
}

/// C: `sub_021cb8ac` (GMEVENT of [`disconnect_create`]): end the wireless session (X1 net calls) and end.
pub fn event_disconnect<H: BsubwayHooks>(h: &mut H, _event: u32, seq: u32, e: u32) -> u32 {
    match h.r32(seq) {
        0 => {
            if h.fun_0203d378() == 1 {
                h.fun_0203d7bc(0);
            }
            let w = h.r32(e);
            let bit = (u32::from(h.r16(w + W_FLAGS)) >> 9) & 1;
            if bit == 1 {
                h.fun_02167b18(w, bit);
            }
        }
        1 => {
            if h.fun_0203d378() == 1 && h.fun_0203d1b4() as i32 > 1 {
                return 0;
            }
            h.fun_0203ceac(0);
        }
        2 => return u32::from(h.fun_0203cdbc() == 1),
        _ => return 0,
    }
    let s = h.r32(seq);
    h.w32(seq, s.wrapping_add(1));
    0
}

/// C: `FUN_021CB914`: the event [`event_disconnect`] (work 4: W).
pub fn disconnect_create<H: BsubwayHooks>(h: &mut H, gamesys: u32, w: u32) -> u32 {
    let ev = h.fun_020122c0(gamesys, 0, EVENT_DISCONNECT, 4);
    let e = h.fun_020124e8(ev);
    h.w32(e, w);
    ev
}

/// C: `sub_021cb934` (GMEVENT of [`event_934_create`]): seq 0 `FUN_0215A684(ctx, 8)`, then as seq 1; seq 1 waits for `FUN_0215A690(ctx)` and
/// starts `FUN_02190BE4(.., E[0xC])` (E+8); seq 2 waits for it (`FUN_02190C78`: 0 -> var 0, 1 -> var 1); seq 3 frees it and
/// ends.
pub fn event_934<H: BsubwayHooks>(h: &mut H, _event: u32, seq: u32, e: u32) -> u32 {
    let ctx = h.r32(e);
    h.fun_0215a5f4(ctx);
    let s = h.r32(seq);
    match s {
        0 | 1 => {
            if s == 0 {
                // seq 0 falls through into seq 1 in the original
                let ctx = h.r32(e);
                h.fun_0215a684(ctx, 8);
                let s = h.r32(seq);
                h.w32(seq, s.wrapping_add(1));
            }
            let ctx = h.r32(e);
            if h.fun_0215a690(ctx) == 1 {
                let ctx = h.r32(e);
                let gs = h.fun_0215a5dc(ctx);
                let fm = h.fun_0201210c(gs);
                let y = h.fun_02188c54(fm);
                let v = h.r32(e + 0xC);
                let r = h.fun_02190be4(y, v);
                h.w32(e + 8, r);
                let s = h.r32(seq);
                h.w32(seq, s.wrapping_add(1));
            }
        }
        2 => {
            let x = h.r32(e + 8);
            let r = h.fun_02190c78(x);
            if r != 2 {
                let var = h.r32(e + 4);
                h.w16(var, u16::from(r != 0));
                let s = h.r32(seq);
                h.w32(seq, s.wrapping_add(1));
            }
        }
        3 => {
            let x = h.r32(e + 8);
            h.fun_021909bc(x);
            return 1;
        }
        _ => {}
    }
    0
}

/// C: `FUN_overlay_d_21__021cb9bc`: the event [`event_934`] (work 0x10: ctx, var, -, `f`).
pub fn event_934_create<H: BsubwayHooks>(
    h: &mut H,
    gamesys: u32,
    ctx: u32,
    var: u32,
    f: u32,
) -> u32 {
    let ev = h.fun_020122c0(gamesys, 0, EVENT_934, 0x10);
    let e = h.fun_020124e8(ev);
    h.w32(e, ctx);
    h.w32(e + 4, var);
    h.w32(e + 0xC, f);
    ev
}

/// C: `FUN_overlay_d_21__021cb9e4`: add the NPC OBJs of the platform: mode 4: up to 5 from the extra record's list (newest
/// first, places 0x021DADD0, codes 0x021DAD48 / 0x021DAD5C by the record's bit); others: the NPC records 0x021DAEBC whose
/// level (0x021DADF8) is the rounds (+3 for modes 5..8). OBJ ids from 0x80.
pub fn place_npcs<H: BsubwayHooks>(h: &mut H, w: u32, gd: u32, objsys: u32, fm: u32) {
    let zone = h.fun_02188c9c(fm);
    let mut id = 0x80u32;
    let mode = h.r8(w + W_MODE);
    if mode == 4 {
        let save = h.fun_02012ebc(gd);
        let xr = h.fun_020071cc(save, SAVE_EXTRA);
        let list = h.fun_0200df58(xr, 4);
        let mut n = h.fun_0200df94(list);
        if n != 0 {
            let mut rec = list.wrapping_add(0x22 * n.wrapping_sub(1));
            let mut place = T_PLACES;
            let mut i = 0u32;
            while n != 0 && i < 5 {
                let q = h.fun_0200dfb4(rec) % 10;
                let tbl = if (h.r8(rec + 0x20) >> 1) & 1 == 0 {
                    T_OBJ_A
                } else {
                    T_OBJ_B
                };
                let code = u32::from(rom16(tbl + 2 * q));
                let x = rom16(place) as i16 as i32 as u32;
                let z = rom16(place + 4) as i16 as i32 as u32;
                let o = h.fun_0216ce6c(objsys, x, z, i & 3, id, code, 2, zone);
                h.fun_0216d620(o, n.wrapping_sub(1) & 0xFFFF, 0);
                h.fun_0216d5b4(o, 5);
                rec = rec.wrapping_sub(0x22);
                i += 1;
                id = (id + 1) & 0xFFFF;
                n = n.wrapping_sub(1);
                place += 8;
            }
        }
        h.fun_020307b0(list);
        return;
    }
    let s = h.r32(w + W_SCORE);
    let mut rounds = h.fun_0200db80(s, u32::from(mode));
    if (5..=8).contains(&mode) {
        rounds = rounds.wrapping_add(3) & 0xFFFF;
    }
    let mut rec = T_NPCS;
    let mut i = 0u32;
    while rom16(rec) != 0xFFFF {
        let lv = rom32(T_LEVELS + 4 * u32::from(rom16(rec + 2)) - 4);
        if rounds == lv {
            let x = rom16(rec + 4) as i16 as i32 as u32;
            let z = rom16(rec + 8) as i16 as i32 as u32;
            let o = h.fun_0216ce6c(objsys, x, z, i & 3, id, u32::from(rom16(rec)), 2, zone);
            h.fun_0216d620(o, i & 0xFFFF, 0);
            h.fun_0216d620(o, u32::from(rom16(rec + 0xA)), 1);
            h.fun_0216d5b4(o, 5);
            if id < 0xFF {
                id += 1;
            }
        }
        rec += 0x10;
        i += 1;
    }
}

/// C: `FUN_021CBBA0`: `FUN_0216D63C(obj, 1)` (the NPC record index of an OBJ).
pub fn obj_npc_id<H: BsubwayHooks>(h: &mut H, obj: u32, _x: u32) -> u32 {
    h.fun_0216d63c(obj, 1)
}

/// C: `FUN_021CBBAC`: NPC record `k` (0x021DAEBC + 0x10 k) has a talk (u16 +0xE nonzero; never in mode 4).
pub fn npc_has_talk<H: MemEnv>(h: &mut H, w: u32, k: u32) -> bool {
    let rec = T_NPCS.wrapping_add(k << 4);
    h.r8(w + W_MODE) != 4 && rom16(rec + 0xE) != 0
}

/// C: `FUN_021CBBD0`: the talk of NPC record `k`: (u16 +0xA, u16 +0xC, u16 +0xE), or (0, 0, 4) without one (or in mode 4).
pub fn npc_talk<H: MemEnv>(h: &mut H, w: u32, k: u32) -> (u16, u16, u16) {
    let rec = T_NPCS.wrapping_add(k << 4);
    if h.r8(w + W_MODE) == 4 {
        return (0, 0, 4);
    }
    let t = rom16(rec + 0xE);
    if t == 0 {
        return (0, 0, 4);
    }
    (rom16(rec + 0xA), rom16(rec + 0xC), t)
}

/// C: `sub_021cbc14` (wait of sub-command 0x15A): `FUN_0200B6E0(gd, 4, kind, min(u16 W+0x10, 9999), 0, W+0x7E8, W+0x7EA)`
/// is 2 or 3.
pub fn wait_c14<H: BsubwayHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let (_, gd, w) = wait_ctx(h, ctx);
    let kind = mode_kind(u32::from(h.r8(w + W_MODE)));
    let v = u32::from(h.r16(w + 0x10)).min(CAP_9999);
    let r = h.fun_0200b6e0(gd, 4, kind, v, 0, w + 0x7E8, w + 0x7EA);
    u32::from(r.wrapping_sub(2) <= 1)
}

/// C: `FUN_overlay_d_21__021cbc94`: the record kind of a mode: 1 / 6 -> 0x15, 2 / 3 / 7 / 8 -> 0x16, else 0x14.
pub fn mode_kind(mode: u32) -> u32 {
    match mode {
        1 | 6 => 0x15,
        2 | 3 | 7 | 8 => 0x16,
        _ => 0x14,
    }
}

/// C: `FUN_overlay_d_21__021cbcc4`: the platform kind of the zone (0x44 -> 5, 0x45 -> 1, 0x46 -> 6, 0x47 -> 2, 0x48 -> 7,
/// 0x49 -> 4, else 0).
pub fn zone_kind<H: BsubwayHooks>(h: &mut H, fm: u32) -> u32 {
    match h.fun_02188c9c(fm).wrapping_sub(0x43) {
        1 => 5,
        2 => 1,
        3 => 6,
        4 => 2,
        5 => 7,
        6 => 4,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_21__021cbd08`: the battle setup (`FUN_021681C8(x, party, mode, W+0x88, partner trainer, count)`) with a
/// party of the mons in the slots (heap 0x8004, freed after).
pub fn battle_setup<H: BsubwayHooks>(h: &mut H, w: u32, x: u32) -> u32 {
    let pt = h.fun_0201a920(0x8004);
    let party = party(h, w);
    let n = u32::from(h.r8(w + W_COUNT));
    h.fun_0201a954(pt, n);
    let mut i = 0;
    while i < u32::from(h.r8(w + W_COUNT)) {
        let slot = u32::from(h.r8(w + W_SLOTS + i));
        let mon = h.fun_0201ab28(party, slot);
        h.fun_0201a98c(pt, mon);
        i += 1;
    }
    let k = partner(h, w);
    let n = u32::from(h.r8(w + W_COUNT));
    let m = u32::from(h.r8(w + W_MODE));
    let r = h.fun_021681c8(x, pt, m, w + 0x88, w + 0x2C8 + 0x120 * k, n);
    h.fun_020307b0(pt);
    r
}

/// C: `FUN_021cbd84`: W + 0x74C.
pub fn w_74c(w: u32) -> u32 {
    w.wrapping_add(0x74C)
}

/// C: `FUN_overlay_d_21__021cbd90`: the trainer id of a round: the boss of round 6 (single modes level 2, multi modes level 6:
/// ids 306..313 by mode and `second`), else random in the id range of the level / round (multi level 4 and up: 200..299);
/// 0 when there is none (a single-mode level above 2).
pub fn trainer_id<H: BsubwayHooks>(
    h: &mut H,
    w: u32,
    level: u32,
    round: u32,
    mode: u32,
    second: u32,
) -> u16 {
    let multi = (5..=8).contains(&mode);
    let mut boss = -1i32;
    if round == 6 {
        if !multi {
            if level == 2 {
                boss = 0;
            }
        } else if level == 6 {
            boss = 1;
        }
    }
    let second = second & 0xFF != 0;
    if boss >= 0 {
        let b = boss == 1;
        return match mode {
            0 | 5 => {
                if b {
                    307
                } else {
                    306
                }
            }
            1 | 6 => {
                if b {
                    309
                } else {
                    308
                }
            }
            2 | 3 | 7 | 8 => match (b, second) {
                (false, false) => 311,
                (false, true) => 310,
                (true, false) => 313,
                (true, true) => 312,
            },
            _ => 0,
        };
    }
    let tbl = if !multi {
        if level >= 3 {
            return 0;
        }
        if round < 6 {
            T_IDS_SINGLE
        } else {
            if level >= 2 {
                return 0;
            }
            T_IDS_SINGLE_BOSS
        }
    } else {
        if level >= 4 {
            let r = rand_u16(h, w) as i32;
            return ((r % 100) + 200) as u16;
        }
        if round < 6 {
            T_IDS_MULTI
        } else {
            T_IDS_MULTI_BOSS
        }
    };
    let lo = rom16(tbl + 4 * level);
    let hi = rom16(tbl + 4 * level + 2);
    let span = (hi.wrapping_sub(lo).wrapping_add(1)) as i32;
    let r = rand_u16(h, w) as i32;
    (i32::from(lo) + r % span) as u16
}

/// C: `FUN_overlay_d_21__021cbed8`: `FUN_02168878(rec, id, n, a, b, c, heap & 0xFFFF)` (W unused).
#[allow(clippy::too_many_arguments)]
pub fn make_trainer<H: BsubwayHooks>(
    h: &mut H,
    _w: u32,
    rec: u32,
    id: u32,
    n: u32,
    a: Buf,
    b: Buf,
    c: u32,
    heap: u32,
) -> u32 {
    h.fun_02168878(rec, id, n, a, b, c, heap & 0xFFFF)
}

/// C: `FUN_overlay_d_21__021cbefc`: the opponents' IV value for a trainer id (3, 6, .. 21 in steps of 20 ids from 100, else 31).
pub fn trainer_iv(id: u32) -> u32 {
    match id {
        0..=99 => 3,
        100..=119 => 6,
        120..=139 => 9,
        140..=159 => 12,
        160..=179 => 15,
        180..=199 => 18,
        200..=219 => 21,
        _ => 31,
    }
}

/// C: `FUN_overlay_d_21__021cbf38`: the multi partner trainer `id` into `rec` (`FUN_021688BC`), its two mons from the 0x14-byte
/// selection `sel` (`FUN_021684E8`, IVs by id), the block freed.
pub fn make_partner<H: BsubwayHooks>(
    h: &mut H,
    _w: u32,
    rec: u32,
    id: u32,
    x: u32,
    sel: u32,
    heap: u32,
) {
    let blk = h.fun_021688bc(rec, id, heap);
    let iv = trainer_iv(id);
    for i in 0..2u32 {
        let species = u32::from(h.r16(sel + 4 + 2 * i));
        let a = h.r32(sel);
        let b = h.r32(sel + 8 + 4 * i);
        h.fun_021684e8(rec + 0x30 + 0x3C * i, species, a, b, iv, i & 0xFF, x, heap);
    }
    h.fun_020307b0(blk);
}

/// C: `FUN_overlay_d_21__021cbf98`: a random u16 (`FUN_020056EC(0xFFFFFFFF) / 0xFFFF`).
pub fn rand_u16<H: BsubwayHooks>(h: &mut H, _w: u32) -> u16 {
    (h.fun_020056ec(0xFFFF_FFFF) / 0xFFFF) as u16
}

/// Run the command handler at `handler` (Thumb bit allowed) when it is one of this area's; `None` otherwise.
pub fn dispatch<H: BsubwayHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x021C_A57C => cmd_343(h, vm, ctx),
        0x021C_A544 => cmd_344(h, vm, ctx),
        0x021C_A598 => cmd_345(h, vm, ctx),
        0x021C_A5C4 => cmd_346(h, vm, ctx),
        _ => return None,
    })
}

/// Run the wait callback `f` (VM +0x10, Thumb bit set) when command 346 installed it; `None` otherwise.
pub fn call_wait<H: BsubwayHooks>(h: &mut H, f: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match f {
        WAIT_6DC => wait_6dc(h, vm, ctx),
        WAIT_700 => wait_700(h, vm, ctx),
        WAIT_754 => wait_754(h, vm, ctx),
        WAIT_7E8 => wait_7e8(h, vm, ctx),
        WAIT_86C => wait_86c(h, vm, ctx),
        WAIT_C14 => wait_c14(h, vm, ctx),
        _ => return None,
    })
}

/// Run the GMEVENT function `f(event, seq, work)` (Thumb bit set) when command 346 installed it; `None` otherwise.
pub fn call_event<H: BsubwayHooks>(
    h: &mut H,
    f: u32,
    event: u32,
    seq: u32,
    work: u32,
) -> Option<u32> {
    Some(match f {
        EVENT_DISCONNECT => event_disconnect(h, event, seq, work),
        EVENT_934 => event_934(h, event, seq, work),
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
