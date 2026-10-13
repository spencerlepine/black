//! `fldmmdl.c` (overlay 10, `0x0216CC58..0x0216E62C`): the field map-model objects (MMDL: NPCs and the player
//! object) and their manager (MMDLSYS): create / delete from 0x24-byte headers, position, direction, move code,
//! status flags, the hooks into the movement and draw procs, and the save-position work.
//!
//! F4a-spec specced all 253 functions of the area (`tools/rom-diff/specs/black-field/fldmmdl.spec`, whose header
//! documents the MMDLSYS (0x48 bytes), MMDL (0x100 bytes), HEADER (0x24 bytes) and save-position layouts; summary in
//! the crate README); F4a ported them. Records are read through the [`Mem`] window of the environment at their
//! original addresses. The draw side (blact / 3D objects), the movement procs, the task system, the archives and
//! the heap are reached only through [`FieldEnv::call`] by address: the proc tables (`+0x8C` / `+0x90`) hold
//! callbacks that are called by address, and the overlay-21 callees (`fldmmdl_blact` `0219C5FC` / `0219C624` /
//! `0219C630`, `fldmmdl_g3dobj` `0219D6B8` / `0219D6CC`, `fldmmdl_acmd` `02197C4C` / `02197C78`, the rail
//! functions `021642F8` ..) are other F items (D3: the draw side is rewritten, G8/U2; movement F4b). Callees inside
//! the area are called as Rust functions, except the three that the spec forces behind the seam (`0216E400` /
//! `0216E40C` ROM proc-table lookups and `0216D8AC` from `0216DDDC`), which go through [`FieldEnv::call`] at their
//! own address (a real environment dispatches them to [`move_proc_table`] / [`draw_proc_table`] / [`pos_y`]).
//! Planned next to this module: `fldmmdl_acmd.c` 0x02197D3C-0x0219C552 (overlay 21, 452 fns; F4b) and
//! `event_trainer_eye.c` 0x021AE454 (F4c).
//!
//! MMDLSYS: `+0x00` flags, `+0x04 u16` MMDL count, `+0x06 s16` objects in use, `+0x18` -> `MMDL[count]` (0x100 each),
//! `+0x20` task system, `+0x2C` -> save positions, `+0x44` -> OBJ code table. MMDL: `+0x00 u32` status (bit 0 = in
//! use), `+0x04 u32` move bits, `+0x08 u16` OBJ id, `+0x0A u16` zone id, `+0x0C` OBJ code, `+0x0E` move code,
//! `+0x18 u16` display dir, `+0x3C..+0x40` grid, `+0x44` position, `+0x88` -> MMDLSYS, `+0x8C` / `+0x90` -> proc
//! tables, `+0xE4` OBJ DATA (0x1C bytes).

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_CC58,
    0x0216_CCB0,
    0x0216_CCCC,
    0x0216_CCDC,
    0x0216_CD30,
    0x0216_CD50,
    0x0216_CD64,
    0x0216_CD78,
    0x0216_CD88,
    0x0216_CDB8,
    0x0216_CDC4,
    0x0216_CE60,
    0x0216_CE6C,
    0x0216_CEAC,
    0x0216_CEE4,
    0x0216_CF2C,
    0x0216_CF6C,
    0x0216_CFC4,
    0x0216_D02C,
    0x0216_D040,
    0x0216_D0E8,
    0x0216_D0F8,
    0x0216_D1C8,
    0x0216_D208,
    0x0216_D230,
    0x0216_D244,
    0x0216_D268,
    0x0216_D28C,
    0x0216_D2AC,
    0x0216_D2F8,
    0x0216_D38C,
    0x0216_D3A8,
    0x0216_D3B4,
    0x0216_D3CC,
    0x0216_D410,
    0x0216_D438,
    0x0216_D454,
    0x0216_D480,
    0x0216_D4B4,
    0x0216_D4D4,
    0x0216_D4DC,
    0x0216_D4E4,
    0x0216_D4F0,
    0x0216_D4F4,
    0x0216_D4F8,
    0x0216_D4FC,
    0x0216_D508,
    0x0216_D514,
    0x0216_D518,
    0x0216_D51C,
    0x0216_D520,
    0x0216_D524,
    0x0216_D528,
    0x0216_D52C,
    0x0216_D530,
    0x0216_D534,
    0x0216_D538,
    0x0216_D548,
    0x0216_D550,
    0x0216_D55C,
    0x0216_D564,
    0x0216_D568,
    0x0216_D570,
    0x0216_D57C,
    0x0216_D584,
    0x0216_D588,
    0x0216_D58C,
    0x0216_D590,
    0x0216_D594,
    0x0216_D598,
    0x0216_D59C,
    0x0216_D5A0,
    0x0216_D5A4,
    0x0216_D5A8,
    0x0216_D5AC,
    0x0216_D5B0,
    0x0216_D5B4,
    0x0216_D5B8,
    0x0216_D5BC,
    0x0216_D5D4,
    0x0216_D5D8,
    0x0216_D5E0,
    0x0216_D5F8,
    0x0216_D5FC,
    0x0216_D600,
    0x0216_D608,
    0x0216_D60C,
    0x0216_D620,
    0x0216_D63C,
    0x0216_D65C,
    0x0216_D660,
    0x0216_D668,
    0x0216_D66C,
    0x0216_D674,
    0x0216_D678,
    0x0216_D67C,
    0x0216_D684,
    0x0216_D69C,
    0x0216_D6A0,
    0x0216_D6B8,
    0x0216_D6BC,
    0x0216_D6D4,
    0x0216_D6D8,
    0x0216_D6F0,
    0x0216_D6F4,
    0x0216_D708,
    0x0216_D71C,
    0x0216_D730,
    0x0216_D744,
    0x0216_D758,
    0x0216_D76C,
    0x0216_D780,
    0x0216_D794,
    0x0216_D7A8,
    0x0216_D7C0,
    0x0216_D7C4,
    0x0216_D7C8,
    0x0216_D7CC,
    0x0216_D7D4,
    0x0216_D7D8,
    0x0216_D7DC,
    0x0216_D7E0,
    0x0216_D7E4,
    0x0216_D7E8,
    0x0216_D7F0,
    0x0216_D7F4,
    0x0216_D7F8,
    0x0216_D800,
    0x0216_D804,
    0x0216_D80C,
    0x0216_D810,
    0x0216_D818,
    0x0216_D81C,
    0x0216_D824,
    0x0216_D828,
    0x0216_D83C,
    0x0216_D844,
    0x0216_D848,
    0x0216_D854,
    0x0216_D85C,
    0x0216_D860,
    0x0216_D86C,
    0x0216_D874,
    0x0216_D87C,
    0x0216_D888,
    0x0216_D88C,
    0x0216_D89C,
    0x0216_D8AC,
    0x0216_D8B0,
    0x0216_D8C0,
    0x0216_D8D0,
    0x0216_D8E0,
    0x0216_D8F0,
    0x0216_D900,
    0x0216_D910,
    0x0216_D91C,
    0x0216_D924,
    0x0216_D92C,
    0x0216_D948,
    0x0216_D95C,
    0x0216_D974,
    0x0216_D98C,
    0x0216_D9A0,
    0x0216_D9E4,
    0x0216_DA20,
    0x0216_DA30,
    0x0216_DA44,
    0x0216_DA50,
    0x0216_DA5C,
    0x0216_DA70,
    0x0216_DA7C,
    0x0216_DA88,
    0x0216_DA94,
    0x0216_DAA0,
    0x0216_DAB4,
    0x0216_DAC8,
    0x0216_DAE0,
    0x0216_DAF8,
    0x0216_DB04,
    0x0216_DB10,
    0x0216_DB24,
    0x0216_DB4C,
    0x0216_DB68,
    0x0216_DB80,
    0x0216_DB98,
    0x0216_DBB4,
    0x0216_DBCC,
    0x0216_DBE8,
    0x0216_DC00,
    0x0216_DC18,
    0x0216_DC34,
    0x0216_DC4C,
    0x0216_DC64,
    0x0216_DC80,
    0x0216_DC98,
    0x0216_DCB4,
    0x0216_DCCC,
    0x0216_DCE8,
    0x0216_DD04,
    0x0216_DD1C,
    0x0216_DD60,
    0x0216_DDC0,
    0x0216_DDDC,
    0x0216_DE54,
    0x0216_DE94,
    0x0216_DEE0,
    0x0216_DEEC,
    0x0216_DF1C,
    0x0216_DF64,
    0x0216_DFAC,
    0x0216_DFC4,
    0x0216_E014,
    0x0216_E084,
    0x0216_E0AC,
    0x0216_E0E8,
    0x0216_E114,
    0x0216_E160,
    0x0216_E174,
    0x0216_E180,
    0x0216_E190,
    0x0216_E1A4,
    0x0216_E1C4,
    0x0216_E204,
    0x0216_E280,
    0x0216_E290,
    0x0216_E29C,
    0x0216_E2AC,
    0x0216_E2E0,
    0x0216_E2E8,
    0x0216_E39C,
    0x0216_E3C4,
    0x0216_E3C8,
    0x0216_E3CC,
    0x0216_E3D8,
    0x0216_E400,
    0x0216_E40C,
    0x0216_E418,
    0x0216_E434,
    0x0216_E464,
    0x0216_E468,
    0x0216_E46C,
    0x0216_E470,
    0x0216_E474,
    0x0216_E47C,
    0x0216_E490,
    0x0216_E4B0,
    0x0216_E4EC,
    0x0216_E51C,
    0x0216_E544,
    0x0216_E57C,
    0x0216_E5B0,
    0x0216_E5FC,
    0x0216_E618,
];

/// Size of an MMDL record.
pub const MMDL_SIZE: u32 = 0x100;
/// Size of the MMDLSYS record.
pub const SYS_SIZE: u32 = 0x48;
/// Size of a MMDL HEADER (the 36-byte NPC record of ARC 125).
pub const HEADER_SIZE: u32 = 0x24;
/// Size of the OBJ code table (`MMDLSYS +0x44`).
pub const OBJ_TABLE_SIZE: u32 = 0x310;
/// Size of an OBJ DATA record (`MMDL +0xE4`, 14 halfwords).
pub const OBJ_DATA_SIZE: u32 = 0x1C;
/// Size of the save-position work (30 VecFx32).
pub const SAVE_WORK_SIZE: u32 = 0x168;
/// MMDL status bit: the object is in use.
pub const ST_IN_USE: u32 = 1;
/// "No save slot" answer of [`save_slot`] (and the slot count of the save-position work).
pub const NO_SLOT: u16 = 0x1E;
/// OBJ id of the player object.
pub const PLAYER_ID: u32 = 0xFF;

/// Address of the string `"fldmmdl.c"` (the file argument of the allocations).
pub const FILE_FLDMMDL: u32 = 0x0217_2B48;
/// The MMDL task callback (`FUN_0216D3A8`, Thumb address) registered by [`task_add`].
pub const TASK_CALLBACK: u32 = 0x0216_D3A9;

/// C: `FUN_02030CB0(n)`: size of a task system for `n` tasks.
const TASK_SYS_SIZE: u32 = 0x0203_0CB0;
/// C: `FUN_02030CBC(n, block)`: initialise a task system in `block`, returns it.
const TASK_SYS_INIT: u32 = 0x0203_0CBC;
/// C: `FUN_02030D28(sys)`: run the tasks of a task system.
const TASK_SYS_RUN: u32 = 0x0203_0D28;
/// C: `FUN_02030DA4(sys)`: shut a task system down.
const TASK_SYS_EXIT: u32 = 0x0203_0DA4;
/// C: `FUN_02030DA8(sys, callback, arg, priority)`: add a task, returns its handle.
const TASK_ADD: u32 = 0x0203_0DA8;
/// C: `FUN_02030EAC(task)`: delete a task.
const TASK_DEL: u32 = 0x0203_0EAC;
/// C: `FUN_020490F4(arc, heap)`: open an archive, returns its handle.
const ARC_OPEN: u32 = 0x0204_90F4;
/// C: `FUN_02049238(handle)`: close an archive.
const ARC_CLOSE: u32 = 0x0204_9238;
/// C: `FUN_0204944C(handle, 0, offset, size, dst)`: read `size` bytes of the archive at `offset` into `dst`.
const ARC_READ: u32 = 0x0204_944C;
/// C: `FUN_02048F4C(dst, arc, member)`: load an archive member into `dst`.
const ARC_LOAD: u32 = 0x0204_8F4C;
/// C: `FUN_0200F128(code)`: the record index of an OBJ code in ARC 48.
const OBJ_CODE_INDEX: u32 = 0x0200_F128;
/// C: `FUN_020143CC(ew, var)`: address of an event var (`event_work::var_addr`, another area).
const VAR_ADDR: u32 = 0x0201_43CC;
/// C: `FUN_020142E8(ew, flag)` (reached through the veneer `FUN_0216E42C`): event flag test (`event_work::check_flag`).
const CHECK_FLAG: u32 = 0x0201_42E8;
/// C: `FUN_02087A3C(1, 1)`: wait for the next frame (the draw-side sync loop of [`sync_draw`]).
const WAIT_FRAME: u32 = 0x0208_7A3C;
/// C: `FUN_02163010(m)` / `FUN_02163028(m)`: the grid movement setup / update of an MMDL (other area).
const GRID_MOVE_SETUP: u32 = 0x0216_3010;
const GRID_MOVE_UPDATE: u32 = 0x0216_3028;
/// C: `FUN_02163C04(m, x, z, flag)`: is the MMDL at grid `(x, z)` (other area).
const AT_GRID: u32 = 0x0216_3C04;
/// C: `FUN_02163F10(m)`: after a position change (other area).
const POS_CHANGED: u32 = 0x0216_3F10;
/// C: `FUN_02164010(m)`: the rail-position refresh (other area).
const RAIL_REFRESH: u32 = 0x0216_4010;
/// C: `FUN_021642F8(m, hdr, kind)` / `FUN_02164304(m, hdr, kind)`: rail header setup (other area).
const RAIL_HEADER: u32 = 0x0216_42F8;
const RAIL_HEADER_POST: u32 = 0x0216_4304;
/// C: `FUN_02197C4C(sys)` / `FUN_02197C78(m)` (`fldmmdl_acmd`, overlay 21): draw-side shutdown / per-object update.
const ACMD_EXIT: u32 = 0x0219_7C4C;
const ACMD_UPDATE: u32 = 0x0219_7C78;
/// C: `FUN_0219C5FC(sys)` / `FUN_0219C624(sys, blact)` / `FUN_0219C630(sys)` (`fldmmdl_blact`, overlay 21).
const BLACT_UPDATE: u32 = 0x0219_C5FC;
const BLACT_DRAW: u32 = 0x0219_C624;
const BLACT_BUSY: u32 = 0x0219_C630;
/// C: `FUN_0219D6B8(m)` / `FUN_0219D6CC(m)` (`fldmmdl_g3dobj`, overlay 21): the 3D-object movement setup / update.
const G3D_MOVE_SETUP: u32 = 0x0219_D6B8;
const G3D_MOVE_UPDATE: u32 = 0x0219_D6CC;

/// ROM table `0x021D5E04`: the move-proc table of each move code (47 entries, [`move_proc_table`]).
pub const MOVE_PROC_TABLES: [u32; 47] = [
    0x021D_5A24,
    0x021D_58A8,
    0x021D_5D44,
    0x021D_5D58,
    0x021D_5880,
    0x021D_56F0,
    0x021D_5D08,
    0x021D_5CF4,
    0x021D_5858,
    0x021D_5CCC,
    0x021D_5CB8,
    0x021D_572C,
    0x021D_5740,
    0x021D_5C7C,
    0x021D_5808,
    0x021D_5C2C,
    0x021D_5C18,
    0x021D_57E0,
    0x021D_577C,
    0x021D_5BDC,
    0x021D_5BB4,
    0x021D_57B8,
    0x021D_5B8C,
    0x021D_5830,
    0x021D_58E4,
    0x021D_58F8,
    0x021D_5B3C,
    0x021D_5B28,
    0x021D_5B14,
    0x021D_5934,
    0x021D_5AEC,
    0x021D_5AD8,
    0x021D_5970,
    0x021D_5984,
    0x021D_5A9C,
    0x021D_5A88,
    0x021D_5A74,
    0x021D_59C0,
    0x021D_5A4C,
    0x021D_59E8,
    0x021D_59FC,
    0x021D_5A10,
    0x021D_5A38,
    0x021D_5A60,
    0x021D_5AB0,
    0x021D_5C68,
    0x021D_5C54,
];

/// ROM table `0x021D770C`: the draw-proc table of each OBJ DATA draw kind (21 entries, [`draw_proc_table`]).
pub const DRAW_PROC_TABLES: [u32; 21] = [
    0x021D_86C0,
    0x021D_85B8,
    0x021D_8660,
    0x021D_85D0,
    0x021D_8678,
    0x021D_8648,
    0x021D_8600,
    0x021D_85E8,
    0x021D_8588,
    0x021D_8630,
    0x021D_8570,
    0x021D_8570,
    0x021D_86F0,
    0x021D_8690,
    0x021D_85A0,
    0x021D_8648,
    0x021D_86A8,
    0x021D_8558,
    0x021D_86D8,
    0x021D_8618,
    0x021D_8648,
];

fn alloc<E: FieldEnv>(env: &mut E, heap: u32, size: u32, flag: u32, line: u32) -> u32 {
    env.callv(HEAP_ALLOC, &[heap, size, flag, FILE_FLDMMDL, line])
}

/// fx32 -> grid (`(fx >> 4) / 4096`, signed, rounding toward zero: 16 units per tile).
fn grid_of(fx: u32) -> i32 {
    ((fx as i32) >> 4) / 4096
}

/// fx32 -> grid as the `s16` that the original passes on (`lsls 16; asrs 16` of the grid).
fn grid16(fx: u32) -> u32 {
    grid_of(fx) as i16 as i32 as u32
}

/// Copy a VecFx32 (three words).
fn copy_vec(mem: &mut Mem, dst: u32, src: u32) {
    for k in 0..3 {
        let w = mem.r32(src.wrapping_add(4 * k));
        mem.w32(dst.wrapping_add(4 * k), w);
    }
}

/// Copy `n` halfwords from `src` to `dst` (the original's `ldrh` / `strh` loops).
fn copy_halfwords(mem: &mut Mem, dst: u32, src: u32, n: u32) {
    for k in 0..n {
        let h = mem.r16(src.wrapping_add(2 * k));
        mem.w16(dst.wrapping_add(2 * k), h);
    }
}

/// The "every MMDL in use" loop step of the callers of [`next_in_use`] (`while (FUN_0216DD1C(sys, &m, &i))`, with
/// `m` and `i` stack locals): the same steps on Rust locals.
fn next_local(mem: &Mem, sys: u32, i: &mut u32) -> Option<u32> {
    let n = count(mem, sys) as u32;
    if *i >= n {
        return None;
    }
    let mut p = mem
        .r32(sys.wrapping_add(0x18))
        .wrapping_add(i.wrapping_shl(8));
    loop {
        *i = i.wrapping_add(1);
        if status(mem, p, ST_IN_USE) != 0 {
            return Some(p);
        }
        p = p.wrapping_add(MMDL_SIZE);
        if *i >= n {
            return None;
        }
    }
}

// ---- MMDLSYS create / delete / tasks -------------------------------------------------------------------------

/// C: `FUN_0216CC58` (unlabeled routine at `0x0216CC58`): create the MMDLSYS for `max` objects: the record
/// (`Heap_AllocDebug(heap, 0x48, 1, "fldmmdl.c", 0xC3)`), the MMDL array (`max << 8` bytes, line 0xC4), count,
/// heap, the save-position work `save` (`+0x2C`), task priority 0x80; opens ARC 48 ([`open_obj_arc`]) and allocates
/// the OBJ code table (0x310 bytes, line 0xCF). Returns the record. Nothing is cleared.
pub fn sys_create<E: FieldEnv>(env: &mut E, heap: u32, max: u32, save: u32) -> u32 {
    let sys = alloc(env, heap, SYS_SIZE, 1, 0xC3);
    let arr = alloc(env, heap, max.wrapping_shl(8), 1, 0xC4);
    env.w32(sys + 0x18, arr);
    env.w16(sys + 4, max as u16);
    env.w16(sys + 8, heap as u16);
    env.w32(sys + 0x2C, save);
    env.w16(sys + 0xC, 0x80);
    open_obj_arc(env, sys, heap);
    let t = alloc(env, heap, OBJ_TABLE_SIZE, 1, 0xCF);
    env.w32(sys + 0x44, t);
    sys
}

/// C: `FUN_0216CCB0` (unlabeled routine at `0x0216CCB0`): close ARC 48 ([`close_obj_arc`]), free the OBJ code
/// table, the MMDL array and the record.
pub fn sys_delete<E: FieldEnv>(env: &mut E, sys: u32) {
    close_obj_arc(env, sys);
    for o in [0x44, 0x18] {
        let b = env.r32(sys + o);
        env.callv(HEAP_FREE, &[b]);
    }
    env.callv(HEAP_FREE, &[sys]);
}

/// C: `FUN_0216CCCC` (unlabeled routine at `0x0216CCCC`): clear the OBJ code table (`MI_CpuFill8(+0x44, 0, 0x310)`).
pub fn clear_obj_table(mem: &mut Mem, sys: u32) {
    let t = mem.r32(sys + 0x44);
    mem.fill(t, OBJ_TABLE_SIZE, 0);
}

/// C: `FUN_0216CCDC` (unlabeled routine at `0x0216CCDC`): start the MMDL tasks: `+0x34 = c`, `+0xA = heap` (u16),
/// `+0x30 = a`, `+0x3C = b`; the task system block (`FUN_02030CB0(count)` bytes, `Heap_AllocDebug(heap, n, 0, ..,
/// 0x109)`, `+0x1C`) and system (`+0x20`), ARC 49 (`+0x10`), `+0x38 = d`; sets flag 0x10.
#[allow(clippy::too_many_arguments)]
pub fn tasks_start<E: FieldEnv>(env: &mut E, sys: u32, heap: u32, a: u32, b: u32, c: u32, d: u32) {
    env.w32(sys + 0x34, c);
    env.w16(sys + 0xA, heap as u16);
    env.w32(sys + 0x30, a);
    env.w32(sys + 0x3C, b);
    let n = env.r16(sys + 4) as u32;
    let size = env.callv(TASK_SYS_SIZE, &[n]);
    let blk = alloc(env, heap, size, 0, 0x109);
    let n = env.r16(sys + 4) as u32;
    env.w32(sys + 0x1C, blk);
    let ts = env.callv(TASK_SYS_INIT, &[n, blk]);
    env.w32(sys + 0x20, ts);
    let h = env.callv(ARC_OPEN, &[0x31, heap]);
    env.w32(sys + 0x10, h);
    env.w32(sys + 0x38, d);
    sys_set_flags(env.mem(), sys, 0x10);
}

/// C: `FUN_0216CD30` (unlabeled routine at `0x0216CD30`): stop the MMDL tasks: shut the task system down, free its
/// block, close ARC 49, clear flag 0x10.
pub fn tasks_stop<E: FieldEnv>(env: &mut E, sys: u32) {
    let ts = env.r32(sys + 0x20);
    env.callv(TASK_SYS_EXIT, &[ts]);
    let b = env.r32(sys + 0x1C);
    env.callv(HEAP_FREE, &[b]);
    let h = env.r32(sys + 0x10);
    env.callv(ARC_CLOSE, &[h]);
    sys_clear_flags(env.mem(), sys, 0x10);
}

/// C: `FUN_0216CD50` (unlabeled routine at `0x0216CD50`): `FUN_02197C4C(sys)` (acmd side), `+0x40 = 0`,
/// [`tasks_stop`].
pub fn sys_stop<E: FieldEnv>(env: &mut E, sys: u32) {
    env.callv(ACMD_EXIT, &[sys]);
    env.w32(sys + 0x40, 0);
    tasks_stop(env, sys);
}

/// C: `FUN_0216CD64` (unlabeled routine at `0x0216CD64`): run the MMDL tasks, then the blact update
/// `FUN_0219C5FC(sys)`.
pub fn sys_update<E: FieldEnv>(env: &mut E, sys: u32) {
    let ts = env.r32(sys + 0x20);
    env.callv(TASK_SYS_RUN, &[ts]);
    env.callv(BLACT_UPDATE, &[sys]);
}

/// C: `FUN_0216CD78` (unlabeled routine at `0x0216CD78`): when the draw system (`+0x24`) exists,
/// `FUN_0219C624(sys, +0x24)`.
pub fn sys_draw<E: FieldEnv>(env: &mut E, sys: u32) {
    let d = env.r32(sys + 0x24);
    if d != 0 {
        env.callv(BLACT_DRAW, &[sys, d]);
    }
}

/// C: `FUN_0216CD88` (unlabeled routine at `0x0216CD88`): while `FUN_0219C630(sys) == 1` (draw side busy):
/// [`sys_update`], wait a frame (`FUN_02087A3C(1, 1)`), [`sys_draw`].
pub fn sync_draw<E: FieldEnv>(env: &mut E, sys: u32) {
    while env.callv(BLACT_BUSY, &[sys]) == 1 {
        sys_update(env, sys);
        env.callv(WAIT_FRAME, &[1, 1]);
        sys_draw(env, sys);
    }
}

/// C: `FUN_0216CDB8` (unlabeled routine at `0x0216CDB8`): `+0x40 = p`, then [`set_draw_on`]`(sys, 1)`.
pub fn sys_set_40(mem: &mut Mem, sys: u32, p: u32) {
    mem.w32(sys + 0x40, p);
    set_draw_on(mem, sys, 1);
}

// ---- create / delete MMDLs -----------------------------------------------------------------------------------

/// C: `FUN_0216CDC4` (unlabeled routine at `0x0216CDC4`): create an MMDL from the HEADER `hdr` (copied to a
/// local first) in the first free slot ([`first_free`]): [`init_base`], [`init_from_header`], [`init_dirs`]; a
/// stored position ([`has_saved_pos`]) sets status 0x280; with the task system on (flag 0x10) [`move_setup`] and
/// [`move_init_kind`], with the draw side on (flag 1) [`draw_setup`]; objects in use + 1; [`header_post`]. Returns
/// the MMDL. UNVERIFIED: with no free slot the original goes on with a null MMDL (the port panics at the window).
pub fn create<E: FieldEnv>(env: &mut E, sys: u32, hdr: u32, zone: u32, ew: u32) -> u32 {
    let lh = env.mem().push_local(HEADER_SIZE);
    env.mem().copy(lh, hdr, HEADER_SIZE);
    let m = first_free(env.mem(), sys);
    init_base(env.mem(), m, sys, zone);
    init_from_header(env, m, lh, ew, sys);
    init_dirs(env.mem(), m);
    if has_saved_pos(env.mem(), m) == 1 {
        set_status(env.mem(), m, 0x280);
    }
    if sys_flags(env.mem(), sys, 0x10) != 0 {
        move_setup(env, sys, m);
        move_init_kind(env, sys, m);
    }
    if sys_flags(env.mem(), sys, 1) != 0 {
        draw_setup(env, m);
    }
    let s = sys_of(env.mem(), m);
    inc_in_use(env.mem(), s);
    header_post(env, m, lh, 0);
    env.mem().pop_local(lh);
    m
}

/// C: `FUN_0216CE60` (unlabeled routine at `0x0216CE60`): [`create`] with no event work.
pub fn create_simple<E: FieldEnv>(env: &mut E, sys: u32, hdr: u32, zone: u32) -> u32 {
    create(env, sys, hdr, zone, 0)
}

/// C: `FUN_0216CE6C` (unlabeled routine at `0x0216CE6C`): build a grid HEADER on the stack (zeroed; OBJ id `id`,
/// OBJ code `obj`, move code `mv`, dir, kind 0, grid `gx` / `gz`; each truncated to u16) and [`create_simple`] it.
#[allow(clippy::too_many_arguments)]
pub fn create_at<E: FieldEnv>(
    env: &mut E,
    sys: u32,
    gx: u32,
    gz: u32,
    dir: u32,
    id: u32,
    obj: u32,
    mv: u32,
    zone: u32,
) -> u32 {
    let h = env.mem().push_local(HEADER_SIZE);
    env.w16(h, id as u16);
    env.w16(h + 2, obj as u16);
    env.w16(h + 4, mv as u16);
    env.w16(h + 0xC, dir as u16);
    env.w32(h + 0x18, 0);
    env.w16(h + 0x1C, gx as u16);
    env.w16(h + 0x1E, gz as u16);
    let m = create_simple(env, sys, h, zone);
    env.mem().pop_local(h);
    m
}

/// Event flag test through the veneer `FUN_0216E42C` (-> `FUN_020142E8(ew, flag)`).
fn check_flag<E: FieldEnv>(env: &mut E, ew: u32, flag: u32) -> u32 {
    env.callv(CHECK_FLAG, &[ew, flag])
}

/// C: `FUN_0216CEAC` (unlabeled routine at `0x0216CEAC`): [`create`] each of the `n` HEADERS at `hdrs` that is an
/// alias ([`header_is_alias`]) or whose event flag (`+8`) is clear. A do-while: `n = 0` runs 2^32 times.
pub fn create_all<E: FieldEnv>(env: &mut E, sys: u32, hdrs: u32, zone: u32, n: u32, ew: u32) {
    let (mut h, mut n) = (hdrs, n);
    loop {
        let make = header_is_alias(env.mem(), h) == 1 || {
            let f = env.r16(h + 8) as u32;
            check_flag(env, ew, f) == 0
        };
        if make {
            create(env, sys, h, zone, ew);
        }
        h = h.wrapping_add(HEADER_SIZE);
        n = n.wrapping_sub(1);
        if n == 0 {
            break;
        }
    }
}

/// C: `FUN_0216CEE4` (unlabeled routine at `0x0216CEE4`): [`create`] the first of the `n` HEADERS that is no alias,
/// has OBJ id `id` (`u16 +0` against the whole register) and a clear event flag; returns the MMDL. UNVERIFIED: when
/// none matches, the original returns an uninitialised stack word (0 in the differential tests); the port returns 0.
pub fn create_by_id<E: FieldEnv>(
    env: &mut E,
    sys: u32,
    hdrs: u32,
    zone: u32,
    n: u32,
    ew: u32,
    id: u32,
) -> u32 {
    let (mut h, mut n) = (hdrs, n);
    loop {
        if header_is_alias(env.mem(), h) == 0 && env.r16(h) as u32 == id {
            let f = env.r16(h + 8) as u32;
            if check_flag(env, ew, f) == 0 {
                return create(env, sys, h, zone, ew);
            }
        }
        h = h.wrapping_add(HEADER_SIZE);
        n = n.wrapping_sub(1);
        if n == 0 {
            return 0;
        }
    }
}

/// C: `FUN_0216CF2C` (unlabeled routine at `0x0216CF2C`): delete an MMDL: draw delete (draw side on), move delete
/// and task delete (status 2), objects in use - 1, clear the record.
pub fn delete<E: FieldEnv>(env: &mut E, m: u32) {
    let s = sys_of(env.mem(), m);
    if draw_on(env.mem(), s) == 1 {
        draw_delete(env, m);
    }
    if status(env.mem(), m, 2) != 0 {
        move_delete(env, m);
        task_del(env, m);
    }
    let s = env.r32(m + 0x88);
    dec_in_use(env.mem(), s);
    clear(env.mem(), m);
}

/// C: `FUN_0216CF6C` (unlabeled routine at `0x0216CF6C`): delete every MMDL in use without the move / task side:
/// draw delete (draw side on), objects in use - 1, clear the record.
pub fn delete_all<E: FieldEnv>(env: &mut E, sys: u32) {
    let mut i = 0;
    while let Some(m) = next_local(env.mem(), sys, &mut i) {
        if draw_on(env.mem(), sys) == 1 {
            draw_delete(env, m);
        }
        status(env.mem(), m, 2);
        let s = env.r32(m + 0x88);
        dec_in_use(env.mem(), s);
        clear(env.mem(), m);
    }
}

/// C: `FUN_0216CFC4` (unlabeled routine at `0x0216CFC4`): copy the HEADER fields into the MMDL: OBJ id, move code,
/// event type / flag / id, header dir (`+0x16`, also move dir `+0x1A`), params 0..2, move limits (s16).
pub fn init_header_fields(mem: &mut Mem, m: u32, hdr: u32) {
    set_obj_id(mem, m, mem.r16(hdr) as u32);
    set_move_code(mem, m, mem.r16(hdr + 4) as u32);
    set_event_type(mem, m, mem.r16(hdr + 6) as u32);
    set_event_flag(mem, m, mem.r16(hdr + 8) as u32);
    set_event_id(mem, m, mem.r16(hdr + 0xA) as u32);
    let d = mem.r16(hdr + 0xC);
    mem.w16(m + 0x16, d);
    mem.w16(m + 0x1A, d);
    for k in 0..3 {
        let p = mem.r16(hdr + 0xE + 2 * k) as u32;
        set_param(mem, m, p, k);
    }
    set_limit_x(mem, m, mem.r16(hdr + 0x14) as i16 as i32 as u32);
    set_limit_z(mem, m, mem.r16(hdr + 0x16) as i16 as i32 as u32);
}

/// C: `FUN_0216D02C` (unlabeled routine at `0x0216D02C`): position from the header: kind (`+0x18`) 0 = grid
/// ([`init_grid_pos`]), else the rail setup `FUN_021642F8(m, hdr, kind)`.
pub fn init_pos<E: FieldEnv>(env: &mut E, m: u32, hdr: u32) {
    let kind = env.r32(hdr + 0x18);
    if kind == 0 {
        init_grid_pos(env, m, hdr);
    } else {
        env.call(
            RAIL_HEADER,
            &mut [CArg::V(m), CArg::In(hdr, HEADER_SIZE), CArg::V(kind)],
        );
    }
}

/// Copy the OBJ DATA sizes and offsets into the MMDL (`+0x7C` / `+0x7D` = bytes `+0xB` / `+0xC`, `+0x7E..+0x80` =
/// the signed bytes `+0xD..+0xF`).
fn obj_data_sizes(mem: &mut Mem, m: u32, with_size: bool) {
    let e = m + 0xE4;
    if with_size {
        let (a, b) = (mem.r8(e + 0xB), mem.r8(e + 0xC));
        mem.w8(m + 0x7C, a);
        mem.w8(m + 0x7D, b);
    }
    for k in 0..3 {
        let o = mem.r8(e + 0xD + k);
        mem.w8(m + 0x7E + k, o);
    }
}

/// C: `FUN_0216D040` (unlabeled routine at `0x0216D040`): [`init_header_fields`]; OBJ code = [`resolve_obj_code`]
/// of the header's (`+2`); [`load_obj_data`]; OBJ code 0xB9 takes its size from header params 0 / 1 (0 -> 1), the
/// others from the OBJ DATA; the offsets from the OBJ DATA; [`init_pos`].
pub fn init_from_header<E: FieldEnv>(env: &mut E, m: u32, hdr: u32, ew: u32, sys: u32) {
    init_header_fields(env.mem(), m, hdr);
    let c = env.r16(hdr + 2) as u32;
    let code = resolve_obj_code(env, ew, c);
    set_obj_code(env.mem(), m, code);
    load_obj_data(env, sys, code, m);
    if code == 0xB9 {
        let (a, b) = (env.r16(hdr + 0xE) as u8, env.r16(hdr + 0x10) as u8);
        env.w8(m + 0x7C, a);
        env.w8(m + 0x7D, b);
        if env.r8(m + 0x7C) == 0 {
            let v = env.r8(m + 0x7C).wrapping_add(1);
            env.w8(m + 0x7C, v);
        }
        if env.r8(m + 0x7D) == 0 {
            let v = env.r8(m + 0x7D).wrapping_add(1);
            env.w8(m + 0x7D, v);
        }
        obj_data_sizes(env.mem(), m, false);
    } else {
        obj_data_sizes(env.mem(), m, true);
    }
    init_pos(env, m, hdr);
}

/// C: `FUN_0216D0E8` (unlabeled routine at `0x0216D0E8`): rail headers (kind 1): `FUN_02164304(m, hdr, 1)`. The
/// third argument `x` is not read (the original overwrites r2 with the kind).
pub fn header_post<E: FieldEnv>(env: &mut E, m: u32, hdr: u32, _x: u32) {
    let kind = env.r32(hdr + 0x18);
    if kind == 1 {
        env.call(
            RAIL_HEADER_POST,
            &mut [CArg::V(m), CArg::In(hdr, HEADER_SIZE), CArg::V(kind)],
        );
    }
}

/// C: `FUN_0216D0F8` (unlabeled routine at `0x0216D0F8`): grid position from the header (`+0x1C` gx, `+0x1E` gz,
/// `+0x20` fx32 y), or from the stored position ([`saved_pos`]) when there is one: initial grid from the header,
/// old and current grid from the chosen position, position = `{gx << 16 + 0x8000, y, gz << 16 + 0x8000}`.
pub fn init_grid_pos<E: FieldEnv>(env: &mut E, m: u32, hdr: u32) {
    let p = hdr + 0x1C;
    let (gx, gz, y);
    if has_saved_pos(env.mem(), m) == 0 {
        gx = env.r16(p) as u32;
        gz = env.r16(p + 2) as u32;
        y = env.r32(p + 4);
    } else {
        let v = env.mem().push_local(12);
        saved_pos(env.mem(), m, v);
        gx = grid_of(env.r32(v)) as u32;
        gz = grid_of(env.r32(v + 8)) as u32;
        y = env.r32(v + 4);
        env.mem().pop_local(v);
    }
    let mem = env.mem();
    set_init_gx(mem, m, mem.r16(p) as i16 as i32 as u32);
    set_old_gx(mem, m, gx as i16 as i32 as u32);
    set_gx(mem, m, gx as i16 as i32 as u32);
    set_init_gy(mem, m, grid16(mem.r32(p + 4)));
    let gy = grid_of(y) as u32;
    set_old_gy(mem, m, gy as i16 as i32 as u32);
    set_gy(mem, m, gy as i16 as i32 as u32);
    set_init_gz(mem, m, mem.r16(p + 2) as i16 as i32 as u32);
    set_old_gz(mem, m, gz as i16 as i32 as u32);
    set_gz(mem, m, gz as i16 as i32 as u32);
    let v = mem.push_local(12);
    mem.w32(v, (gx << 16).wrapping_add(0x8000));
    mem.w32(v + 4, y);
    mem.w32(v + 8, (gz << 16).wrapping_add(0x8000));
    set_pos(mem, m, v);
    mem.pop_local(v);
}

/// C: `FUN_0216D1C8` (unlabeled routine at `0x0216D1C8`): `+0x88 = sys`, zone id, status 1 (in use), move bits
/// 0x1800; an alias header id ([`is_alias_id`]) sets status 0x800; [`acmd_reset`].
pub fn init_base(mem: &mut Mem, m: u32, sys: u32, zone: u32) {
    mem.w32(m + 0x88, sys);
    set_zone(mem, m, zone & 0xFFFF);
    set_status(mem, m, 1);
    set_bits(mem, m, 0x1800);
    if is_alias_id(mem, m) == 1 {
        set_alias(mem, m, 1);
    }
    acmd_reset(mem, m);
}

/// C: `FUN_0216D208` (unlabeled routine at `0x0216D208`): display and move dir = the header dir.
pub fn init_dirs(mem: &mut Mem, m: u32) {
    let d = header_dir(mem, m) as u32;
    set_dir_disp_raw(mem, m, d);
    let d = header_dir(mem, m) as u32;
    set_dir_move(mem, m, d);
}

/// C: `FUN_0216D230` (unlabeled routine at `0x0216D230`): `+0x8C` = the move-proc table of the move code
/// (`FUN_0216E400`, reached through the seam).
pub fn set_move_procs<E: FieldEnv>(env: &mut E, m: u32) {
    let c = move_code(env.mem(), m) as u32;
    let t = env.callv(0x0216_E400, &[c]);
    env.w32(m + 0x8C, t);
}

/// C: `FUN_0216D244` (unlabeled routine at `0x0216D244`): [`set_move_procs`], [`task_add`], move bits 0x1820.
pub fn move_setup<E: FieldEnv>(env: &mut E, sys: u32, m: u32) {
    set_move_procs(env, m);
    task_add(env, m, sys);
    set_bits(env.mem(), m, 0x1820);
}

/// C: `FUN_0216D268` (unlabeled routine at `0x0216D268`): movement setup by kind: status 0x2000 (3D object)
/// `FUN_0219D6B8(m)`, else the grid one `FUN_02163010(m)`.
pub fn move_init_kind<E: FieldEnv>(env: &mut E, _sys: u32, m: u32) {
    if status(env.mem(), m, 0x2000) == 0 {
        env.callv(GRID_MOVE_SETUP, &[m]);
    } else {
        env.callv(G3D_MOVE_SETUP, &[m]);
    }
}

/// C: `FUN_0216D28C` (unlabeled routine at `0x0216D28C`): movement update by kind (status 0x2000: `FUN_0219D6CC`,
/// else `FUN_02163028`).
pub fn move_update_kind<E: FieldEnv>(env: &mut E, m: u32) {
    if status(env.mem(), m, 0x2000) == 0 {
        env.callv(GRID_MOVE_UPDATE, &[m]);
    } else {
        env.callv(G3D_MOVE_UPDATE, &[m]);
    }
}

/// C: `FUN_0216D2AC` (unlabeled routine at `0x0216D2AC`): with the draw side on, every MMDL in use: [`task_del`],
/// draw push, move bits 5.
pub fn suspend_all<E: FieldEnv>(env: &mut E, sys: u32) {
    if draw_on(env.mem(), sys) == 0 {
        return;
    }
    let mut i = 0;
    while let Some(m) = next_local(env.mem(), sys, &mut i) {
        task_del(env, m);
        draw_push(env, m);
        set_bits(env.mem(), m, 5);
    }
}

/// C: `FUN_0216D2F8` (unlabeled routine at `0x0216D2F8`): every MMDL in use: [`move_setup`]; without status 2
/// [`move_init_kind`]; move bit 1: move return, clear it; no draw (bit 2): [`draw_setup`]; bit 4: draw pop,
/// clear it; [`clear_bits_6c000`].
pub fn resume_all<E: FieldEnv>(env: &mut E, sys: u32) {
    let mut i = 0;
    while let Some(m) = next_local(env.mem(), sys, &mut i) {
        move_setup(env, sys, m);
        if status(env.mem(), m, 2) == 0 {
            move_init_kind(env, sys, m);
        }
        if bits(env.mem(), m, 1) != 0 {
            move_return(env, m);
            clear_bits(env.mem(), m, 1);
        }
        if draw_ready(env.mem(), m) == 0 {
            draw_setup(env, m);
        }
        if bits(env.mem(), m, 4) != 0 {
            draw_pop(env, m);
            clear_bits(env.mem(), m, 4);
        }
        clear_bits_6c000(env.mem(), m);
    }
}

/// C: `FUN_0216D38C` (unlabeled routine at `0x0216D38C`): the MMDL task body: [`move_update_kind`], then (still in
/// use) [`draw_update`].
pub fn task_body<E: FieldEnv>(env: &mut E, m: u32) {
    move_update_kind(env, m);
    if in_use(env.mem(), m) == 1 {
        draw_update(env, m);
    }
}

/// C: `FUN_0216D3A8` (unlabeled routine at `0x0216D3A8`, the task callback [`TASK_CALLBACK`]): [`task_body`] of
/// the task's MMDL.
pub fn task_callback<E: FieldEnv>(env: &mut E, _task: u32, m: u32) {
    task_body(env, m);
}

/// C: `FUN_0216D3B4` (unlabeled routine at `0x0216D3B4`): with the draw side on, the acmd draw update
/// `FUN_02197C78(m)`.
pub fn draw_update<E: FieldEnv>(env: &mut E, m: u32) {
    let s = sys_of(env.mem(), m);
    if draw_on(env.mem(), s) == 1 {
        env.callv(ACMD_UPDATE, &[m]);
    }
}

// ---- acmd state ----------------------------------------------------------------------------------------------

/// C: `FUN_0216D3CC` (unlabeled routine at `0x0216D3CC`): 1 when the MMDL can take an acmd: in use, not move bit
/// 0x10, and no acmd running (bit 0x200 without 0x400); else 0.
pub fn acmd_can_set(mem: &Mem, m: u32) -> u32 {
    if status(mem, m, 1) == 0 || bits(mem, m, 0x10) != 0 {
        return 0;
    }
    if bits(mem, m, 0x200) != 0 && bits(mem, m, 0x400) == 0 {
        return 0;
    }
    1
}

/// C: `FUN_0216D410` (unlabeled routine at `0x0216D410`): start the acmd `code`: `+0x26 = code`, `+0x28 = 0`,
/// bit 0x200 on, 0x400 off.
pub fn acmd_set(mem: &mut Mem, m: u32, code: u32) {
    set_acmd_code(mem, m, code);
    set_acmd_seq(mem, m, 0);
    set_bits(mem, m, 0x200);
    clear_bits(mem, m, 0x400);
}

/// C: `FUN_0216D438` (unlabeled routine at `0x0216D438`): change the running acmd: `+0x26 = code`, `+0x28 = 0`,
/// bit 0x400 off.
pub fn acmd_change(mem: &mut Mem, m: u32, code: u32) {
    set_acmd_code(mem, m, code);
    set_acmd_seq(mem, m, 0);
    clear_bits(mem, m, 0x400);
}

/// C: `FUN_0216D454` (unlabeled routine at `0x0216D454`): 1 when no acmd is set (bit 0x200) or it is done (0x400).
pub fn acmd_done(mem: &Mem, m: u32) -> u32 {
    if bits(mem, m, 0x200) == 0 || bits(mem, m, 0x400) != 0 {
        1
    } else {
        0
    }
}

/// C: `FUN_0216D480` (unlabeled routine at `0x0216D480`): like [`acmd_done`], and a done acmd is ended (bits 0x600
/// off).
pub fn acmd_end(mem: &mut Mem, m: u32) -> u32 {
    if bits(mem, m, 0x200) == 0 {
        return 1;
    }
    if bits(mem, m, 0x400) == 0 {
        return 0;
    }
    clear_bits(mem, m, 0x600);
    1
}

/// C: `FUN_0216D4B4` (unlabeled routine at `0x0216D4B4`): no acmd: bits 0x600 off, code 0xFF, seq 0.
pub fn acmd_reset(mem: &mut Mem, m: u32) {
    clear_bits(mem, m, 0x600);
    set_acmd_code(mem, m, 0xFF);
    set_acmd_seq(mem, m, 0);
}

// ---- MMDLSYS accessors ---------------------------------------------------------------------------------------

/// C: `FUN_0216D4D4` (unlabeled routine at `0x0216D4D4`): `flags (+0) & mask` of the MMDLSYS.
pub fn sys_flags(mem: &Mem, sys: u32, mask: u32) -> u32 {
    mem.r32(sys) & mask
}

/// C: `FUN_0216D4DC` (unlabeled routine at `0x0216D4DC`): `flags |= mask`.
pub fn sys_set_flags(mem: &mut Mem, sys: u32, mask: u32) {
    let v = mem.r32(sys) | mask;
    mem.w32(sys, v);
}

/// C: `FUN_0216D4E4` (unlabeled routine at `0x0216D4E4`): `flags &= !mask`.
pub fn sys_clear_flags(mem: &mut Mem, sys: u32, mask: u32) {
    let v = mem.r32(sys) & !mask;
    mem.w32(sys, v);
}

/// C: `FUN_0216D4F0` (unlabeled routine at `0x0216D4F0`): the MMDL count of a MMDLSYS (`u16 +4`).
pub fn count(mem: &Mem, sys: u32) -> u16 {
    mem.r16(sys.wrapping_add(4))
}

/// C: `FUN_0216D4F4` (unlabeled routine at `0x0216D4F4`): the task priority (`u16 +0xC`).
pub fn task_priority(mem: &Mem, sys: u32) -> u16 {
    mem.r16(sys + 0xC)
}

/// C: `FUN_0216D4F8` (unlabeled routine at `0x0216D4F8`): the heap id (`u16 +0xA`).
pub fn sys_heap(mem: &Mem, sys: u32) -> u16 {
    mem.r16(sys + 0xA)
}

/// C: `FUN_0216D4FC` (unlabeled routine at `0x0216D4FC`): objects in use (`s16 +6`) + 1.
pub fn inc_in_use(mem: &mut Mem, sys: u32) {
    let v = mem.r16(sys + 6).wrapping_add(1);
    mem.w16(sys + 6, v);
}

/// C: `FUN_0216D508` (unlabeled routine at `0x0216D508`): objects in use (`s16 +6`) - 1.
pub fn dec_in_use(mem: &mut Mem, sys: u32) {
    let v = mem.r16(sys + 6).wrapping_sub(1);
    mem.w16(sys + 6, v);
}

/// C: `FUN_0216D514` (unlabeled routine at `0x0216D514`): the task system (`+0x20`).
pub fn task_sys(mem: &Mem, sys: u32) -> u32 {
    mem.r32(sys + 0x20)
}

/// C: `FUN_0216D518` (unlabeled routine at `0x0216D518`): the ARC 49 handle (`+0x10`).
pub fn arc49(mem: &Mem, sys: u32) -> u32 {
    mem.r32(sys + 0x10)
}

/// C: `FUN_0216D51C` (unlabeled routine at `0x0216D51C`): `+0x24 = v` (draw system).
pub fn set_draw_sys(mem: &mut Mem, sys: u32, v: u32) {
    mem.w32(sys + 0x24, v);
}

/// C: `FUN_0216D520` (unlabeled routine at `0x0216D520`): the draw system (`+0x24`).
pub fn draw_sys(mem: &Mem, sys: u32) -> u32 {
    mem.r32(sys + 0x24)
}

/// C: `FUN_0216D524` (unlabeled routine at `0x0216D524`): `+0x28 = v`.
pub fn set_sys_28(mem: &mut Mem, sys: u32, v: u32) {
    mem.w32(sys + 0x28, v);
}

/// C: `FUN_0216D528` (unlabeled routine at `0x0216D528`): `+0x28`.
pub fn sys_28(mem: &Mem, sys: u32) -> u32 {
    mem.r32(sys + 0x28)
}

/// C: `FUN_0216D52C` (unlabeled routine at `0x0216D52C`): `+0x34`.
pub fn sys_34(mem: &Mem, sys: u32) -> u32 {
    mem.r32(sys + 0x34)
}

/// C: `FUN_0216D530` (unlabeled routine at `0x0216D530`): `+0x38`.
pub fn sys_38(mem: &Mem, sys: u32) -> u32 {
    mem.r32(sys + 0x38)
}

/// C: `FUN_0216D534` (unlabeled routine at `0x0216D534`): `+0x3C`.
pub fn sys_3c(mem: &Mem, sys: u32) -> u32 {
    mem.r32(sys + 0x3C)
}

/// C: `FUN_0216D538` (unlabeled routine at `0x0216D538`): the u16 that `+0x40` points at, 0 when it is null.
pub fn sys_40_value(mem: &Mem, sys: u32) -> u32 {
    let p = mem.r32(sys + 0x40);
    if p == 0 {
        0
    } else {
        mem.r16(p) as u32
    }
}

// ---- MMDL status / move bits / fields ------------------------------------------------------------------------

/// C: `FUN_0216D548` (unlabeled routine at `0x0216D548`): `status (+0) |= v`.
pub fn set_status(mem: &mut Mem, m: u32, v: u32) {
    let s = mem.r32(m) | v;
    mem.w32(m, s);
}

/// C: `FUN_0216D550` (unlabeled routine at `0x0216D550`): `status &= !v`.
pub fn clear_status(mem: &mut Mem, m: u32, v: u32) {
    let s = mem.r32(m) & !v;
    mem.w32(m, s);
}

/// C: `FUN_0216D55C` (unlabeled routine at `0x0216D55C`): `status (+0) & mask` of an MMDL.
pub fn status(mem: &Mem, m: u32, mask: u32) -> u32 {
    mem.r32(m) & mask
}

/// C: `FUN_0216D564` (unlabeled routine at `0x0216D564`): the move bits (`+4`).
pub fn move_bits(mem: &Mem, m: u32) -> u32 {
    mem.r32(m + 4)
}

/// C: `FUN_0216D568` (unlabeled routine at `0x0216D568`): `move bits (+4) |= v`.
pub fn set_bits(mem: &mut Mem, m: u32, v: u32) {
    let s = mem.r32(m + 4) | v;
    mem.w32(m + 4, s);
}

/// C: `FUN_0216D570` (unlabeled routine at `0x0216D570`): `move bits &= !v`.
pub fn clear_bits(mem: &mut Mem, m: u32, v: u32) {
    let s = mem.r32(m + 4) & !v;
    mem.w32(m + 4, s);
}

/// C: `FUN_0216D57C` (unlabeled routine at `0x0216D57C`): `move bits & mask`.
pub fn bits(mem: &Mem, m: u32, mask: u32) -> u32 {
    mem.r32(m + 4) & mask
}

/// C: `FUN_0216D584` (unlabeled routine at `0x0216D584`): OBJ id (`u16 +8`) = v.
pub fn set_obj_id(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 8, v as u16);
}

/// C: `FUN_0216D588` (unlabeled routine at `0x0216D588`): the OBJ id (`u16 +8`).
pub fn obj_id(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 8)
}

/// C: `FUN_0216D58C` (unlabeled routine at `0x0216D58C`): zone id (`u16 +0xA`) = v.
pub fn set_zone(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0xA, v as u16);
}

/// C: `FUN_0216D590` (unlabeled routine at `0x0216D590`): the zone id (`u16 +0xA`).
pub fn zone(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0xA)
}

/// C: `FUN_0216D594` (unlabeled routine at `0x0216D594`): OBJ code (`u16 +0xC`) = v.
pub fn set_obj_code(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0xC, v as u16);
}

/// C: `FUN_0216D598` (unlabeled routine at `0x0216D598`): the OBJ code (`u16 +0xC`).
pub fn obj_code(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0xC)
}

/// C: `FUN_0216D59C` (unlabeled routine at `0x0216D59C`): move code (`u16 +0xE`) = v.
pub fn set_move_code(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0xE, v as u16);
}

/// C: `FUN_0216D5A0` (unlabeled routine at `0x0216D5A0`): the move code (`u16 +0xE`).
pub fn move_code(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0xE)
}

/// C: `FUN_0216D5A4` (unlabeled routine at `0x0216D5A4`): event type (`u16 +0x10`) = v.
pub fn set_event_type(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x10, v as u16);
}

/// C: `FUN_0216D5A8` (unlabeled routine at `0x0216D5A8`): the event type (`u16 +0x10`).
pub fn event_type(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0x10)
}

/// C: `FUN_0216D5AC` (unlabeled routine at `0x0216D5AC`): event flag (`u16 +0x12`) = v.
pub fn set_event_flag(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x12, v as u16);
}

/// C: `FUN_0216D5B0` (unlabeled routine at `0x0216D5B0`): the event flag (`u16 +0x12`).
pub fn event_flag(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0x12)
}

/// C: `FUN_0216D5B4` (unlabeled routine at `0x0216D5B4`): event id (`u16 +0x14`) = v.
pub fn set_event_id(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x14, v as u16);
}

/// C: `FUN_0216D5B8` (unlabeled routine at `0x0216D5B8`): the event id (`u16 +0x14`).
pub fn event_id(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0x14)
}

/// C: `FUN_0216D5BC` (unlabeled routine at `0x0216D5BC`): 1 when the event id is 0xFFFF (an alias).
pub fn is_alias_id(mem: &Mem, m: u32) -> u32 {
    (event_id(mem, m) == 0xFFFF) as u32
}

/// C: `FUN_0216D5D4` (unlabeled routine at `0x0216D5D4`): the header dir (`u16 +0x16`).
pub fn header_dir(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0x16)
}

/// C: `FUN_0216D5D8` (unlabeled routine at `0x0216D5D8`): display dir = `d` (the old one goes to `+0x1C`).
pub fn set_dir_disp_raw(mem: &mut Mem, m: u32, d: u32) {
    let o = mem.r16(m + 0x18);
    mem.w16(m + 0x1C, o);
    mem.w16(m + 0x18, d as u16);
}

/// C: `FUN_0216D5E0` (unlabeled routine at `0x0216D5E0`): [`set_dir_disp_raw`] unless the dir is locked (status 8).
pub fn set_dir_disp(mem: &mut Mem, m: u32, d: u32) {
    if status(mem, m, 8) == 0 {
        set_dir_disp_raw(mem, m, d);
    }
}

/// C: `FUN_0216D5F8` (unlabeled routine at `0x0216D5F8`): the display direction of an MMDL (`u16 +0x18`).
pub fn dir_disp(mem: &Mem, m: u32) -> u16 {
    mem.r16(m.wrapping_add(0x18))
}

/// C: `FUN_0216D5FC` (unlabeled routine at `0x0216D5FC`): the previous display dir (`u16 +0x1C`).
pub fn dir_disp_old(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0x1C)
}

/// C: `FUN_0216D600` (unlabeled routine at `0x0216D600`): move dir = `d` (the old one goes to `+0x1E`).
pub fn set_dir_move(mem: &mut Mem, m: u32, d: u32) {
    let o = mem.r16(m + 0x1A);
    mem.w16(m + 0x1E, o);
    mem.w16(m + 0x1A, d as u16);
}

/// C: `FUN_0216D608` (unlabeled routine at `0x0216D608`): the move dir (`u16 +0x1A`).
pub fn dir_move(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0x1A)
}

/// C: `FUN_0216D60C` (unlabeled routine at `0x0216D60C`): [`set_dir_disp`] and [`set_dir_move`].
pub fn set_dir(mem: &mut Mem, m: u32, d: u32) {
    set_dir_disp(mem, m, d);
    set_dir_move(mem, m, d);
}

/// C: `FUN_0216D620` (unlabeled routine at `0x0216D620`): header param `i` (0..2: `u16 +0x20 + 2i`) = v; other `i`:
/// nothing.
pub fn set_param(mem: &mut Mem, m: u32, v: u32, i: u32) {
    if i <= 2 {
        mem.w16(m + 0x20 + 2 * i, v as u16);
    }
}

/// C: `FUN_0216D63C` (unlabeled routine at `0x0216D63C`): header param `i` (0..2), else 0.
pub fn param(mem: &Mem, m: u32, i: u32) -> u16 {
    if i <= 2 {
        mem.r16(m + 0x20 + 2 * i)
    } else {
        0
    }
}

/// C: `FUN_0216D65C` (unlabeled routine at `0x0216D65C`): move limit x (`+0x2C`) = v.
pub fn set_limit_x(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x2C, v as u16);
}

/// C: `FUN_0216D660` (unlabeled routine at `0x0216D660`): move limit x (`s16 +0x2C`).
pub fn limit_x(mem: &Mem, m: u32) -> i32 {
    mem.r16(m + 0x2C) as i16 as i32
}

/// C: `FUN_0216D668` (unlabeled routine at `0x0216D668`): move limit z (`+0x2E`) = v.
pub fn set_limit_z(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x2E, v as u16);
}

/// C: `FUN_0216D66C` (unlabeled routine at `0x0216D66C`): move limit z (`s16 +0x2E`).
pub fn limit_z(mem: &Mem, m: u32) -> i32 {
    mem.r16(m + 0x2E) as i16 as i32
}

/// C: `FUN_0216D674` (unlabeled routine at `0x0216D674`): `u16 +0x2A = v`.
pub fn set_field_2a(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x2A, v as u16);
}

/// C: `FUN_0216D678` (unlabeled routine at `0x0216D678`): `u16 +0x2A`.
pub fn field_2a(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0x2A)
}

/// C: `FUN_0216D67C` (unlabeled routine at `0x0216D67C`): the MMDLSYS an MMDL belongs to (`+0x88`).
pub fn sys_of(mem: &Mem, m: u32) -> u32 {
    mem.r32(m.wrapping_add(0x88))
}

/// Clear `n` bytes of the work block at `m + ofs` and return its address.
fn work_init(mem: &mut Mem, m: u32, ofs: u32, n: u32) -> u32 {
    let w = m + ofs;
    mem.fill(w, n, 0);
    w
}

/// C: `FUN_0216D684` (unlabeled routine at `0x0216D684`): clear `n` bytes of the move work (`m + 0x94`), return it.
pub fn move_work_init(mem: &mut Mem, m: u32, n: u32) -> u32 {
    work_init(mem, m, 0x94, n)
}

/// C: `FUN_0216D69C` (unlabeled routine at `0x0216D69C`): the move work (`m + 0x94`, 0x10 bytes).
pub fn move_work(m: u32) -> u32 {
    m.wrapping_add(0x94)
}

/// C: `FUN_0216D6A0` (unlabeled routine at `0x0216D6A0`): clear `n` bytes of the sub work (`m + 0xA4`), return it.
pub fn sub_work_init(mem: &mut Mem, m: u32, n: u32) -> u32 {
    work_init(mem, m, 0xA4, n)
}

/// C: `FUN_0216D6B8` (unlabeled routine at `0x0216D6B8`): the sub work (`m + 0xA4`, 0x10 bytes).
pub fn sub_work(m: u32) -> u32 {
    m.wrapping_add(0xA4)
}

/// C: `FUN_0216D6BC` (unlabeled routine at `0x0216D6BC`): clear `n` bytes of the acmd work (`m + 0xB4`), return it.
pub fn acmd_work_init(mem: &mut Mem, m: u32, n: u32) -> u32 {
    work_init(mem, m, 0xB4, n)
}

/// C: `FUN_0216D6D4` (unlabeled routine at `0x0216D6D4`): the acmd work (`m + 0xB4`, 0x10 bytes).
pub fn acmd_work(m: u32) -> u32 {
    m.wrapping_add(0xB4)
}

/// C: `FUN_0216D6D8` (unlabeled routine at `0x0216D6D8`): clear `n` bytes of the draw work (`m + 0xC4`), return it.
pub fn draw_work_init(mem: &mut Mem, m: u32, n: u32) -> u32 {
    work_init(mem, m, 0xC4, n)
}

/// C: `FUN_0216D6F0` (unlabeled routine at `0x0216D6F0`): the draw work (`m + 0xC4`, 0x20 bytes).
pub fn draw_work(m: u32) -> u32 {
    m.wrapping_add(0xC4)
}

/// Call proc `slot` of the table at `m + tbl` with `m`, if it is set.
fn call_proc<E: FieldEnv>(env: &mut E, m: u32, tbl: u32, slot: u32) {
    let t = env.r32(m + tbl);
    let f = env.r32(t.wrapping_add(slot));
    if f != 0 {
        env.callv(f & !1, &[m]);
    }
}

/// C: `FUN_0216D6F4` (unlabeled routine at `0x0216D6F4`): the move init proc (move table `+4`), if set.
pub fn move_init<E: FieldEnv>(env: &mut E, m: u32) {
    call_proc(env, m, 0x8C, 4);
}

/// C: `FUN_0216D708` (unlabeled routine at `0x0216D708`): the move proc (move table `+8`), if set.
pub fn move_run<E: FieldEnv>(env: &mut E, m: u32) {
    call_proc(env, m, 0x8C, 8);
}

/// C: `FUN_0216D71C` (unlabeled routine at `0x0216D71C`): the move delete proc (move table `+0xC`), if set.
pub fn move_delete<E: FieldEnv>(env: &mut E, m: u32) {
    call_proc(env, m, 0x8C, 0xC);
}

/// C: `FUN_0216D730` (unlabeled routine at `0x0216D730`): the move return proc (move table `+0x10`), if set.
pub fn move_return<E: FieldEnv>(env: &mut E, m: u32) {
    call_proc(env, m, 0x8C, 0x10);
}

/// C: `FUN_0216D744` (unlabeled routine at `0x0216D744`): the draw init proc (draw table `+0`), if set.
pub fn draw_init<E: FieldEnv>(env: &mut E, m: u32) {
    call_proc(env, m, 0x90, 0);
}

/// C: `FUN_0216D758` (unlabeled routine at `0x0216D758`): the draw proc (draw table `+4`), if set.
pub fn draw<E: FieldEnv>(env: &mut E, m: u32) {
    call_proc(env, m, 0x90, 4);
}

/// C: `FUN_0216D76C` (unlabeled routine at `0x0216D76C`): the draw delete proc (draw table `+8`), if set.
pub fn draw_delete<E: FieldEnv>(env: &mut E, m: u32) {
    call_proc(env, m, 0x90, 8);
}

/// C: `FUN_0216D780` (unlabeled routine at `0x0216D780`): the draw push proc (draw table `+0xC`), if set.
pub fn draw_push<E: FieldEnv>(env: &mut E, m: u32) {
    call_proc(env, m, 0x90, 0xC);
}

/// C: `FUN_0216D794` (unlabeled routine at `0x0216D794`): the draw pop proc (draw table `+0x10`), if set.
pub fn draw_pop<E: FieldEnv>(env: &mut E, m: u32) {
    call_proc(env, m, 0x90, 0x10);
}

/// C: `FUN_0216D7A8` (unlabeled routine at `0x0216D7A8`): the draw get proc (draw table `+0x14`) `f(m, k)`, or 0
/// when it is not set.
pub fn draw_get<E: FieldEnv>(env: &mut E, m: u32, k: u32) -> u32 {
    let t = env.r32(m + 0x90);
    let f = env.r32(t.wrapping_add(0x14));
    if f != 0 {
        env.callv(f & !1, &[m, k])
    } else {
        0
    }
}

/// C: `FUN_0216D7C0` (unlabeled routine at `0x0216D7C0`): acmd code (`u16 +0x26`) = v.
pub fn set_acmd_code(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x26, v as u16);
}

/// C: `FUN_0216D7C4` (unlabeled routine at `0x0216D7C4`): the acmd code (`u16 +0x26`).
pub fn acmd_code(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0x26)
}

/// C: `FUN_0216D7C8` (unlabeled routine at `0x0216D7C8`): acmd seq (`u16 +0x28`) = v.
pub fn set_acmd_seq(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x28, v as u16);
}

/// C: `FUN_0216D7CC` (unlabeled routine at `0x0216D7CC`): acmd seq + 1.
pub fn inc_acmd_seq(mem: &mut Mem, m: u32) {
    let v = mem.r16(m + 0x28).wrapping_add(1);
    mem.w16(m + 0x28, v);
}

/// C: `FUN_0216D7D4` (unlabeled routine at `0x0216D7D4`): the acmd seq (`u16 +0x28`).
pub fn acmd_seq(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0x28)
}

/// C: `FUN_0216D7D8` (unlabeled routine at `0x0216D7D8`): `+0x74 = v`.
pub fn set_field_74(mem: &mut Mem, m: u32, v: u32) {
    mem.w32(m + 0x74, v);
}

/// C: `FUN_0216D7DC` (unlabeled routine at `0x0216D7DC`): `+0x74`.
pub fn field_74(mem: &Mem, m: u32) -> u32 {
    mem.r32(m + 0x74)
}

/// C: `FUN_0216D7E0` (unlabeled routine at `0x0216D7E0`): `+0x78 = v`.
pub fn set_field_78(mem: &mut Mem, m: u32, v: u32) {
    mem.w32(m + 0x78, v);
}

/// C: `FUN_0216D7E4` (unlabeled routine at `0x0216D7E4`): `+0x78`.
pub fn field_78(mem: &Mem, m: u32) -> u32 {
    mem.r32(m + 0x78)
}

/// A signed halfword field of the MMDL.
fn s16_at(mem: &Mem, m: u32, ofs: u32) -> i32 {
    mem.r16(m + ofs) as i16 as i32
}

/// C: `FUN_0216D7E8` (unlabeled routine at `0x0216D7E8`): initial grid x (`s16 +0x30`).
pub fn init_gx(mem: &Mem, m: u32) -> i32 {
    s16_at(mem, m, 0x30)
}

/// C: `FUN_0216D7F0` (unlabeled routine at `0x0216D7F0`): initial grid x = v.
pub fn set_init_gx(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x30, v as u16);
}

/// C: `FUN_0216D7F4` (unlabeled routine at `0x0216D7F4`): initial grid y (`+0x32`) = v.
pub fn set_init_gy(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x32, v as u16);
}

/// C: `FUN_0216D7F8` (unlabeled routine at `0x0216D7F8`): initial grid z (`s16 +0x34`).
pub fn init_gz(mem: &Mem, m: u32) -> i32 {
    s16_at(mem, m, 0x34)
}

/// C: `FUN_0216D800` (unlabeled routine at `0x0216D800`): initial grid z = v.
pub fn set_init_gz(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x34, v as u16);
}

/// C: `FUN_0216D804` (unlabeled routine at `0x0216D804`): old grid x (`s16 +0x36`).
pub fn old_gx(mem: &Mem, m: u32) -> i32 {
    s16_at(mem, m, 0x36)
}

/// C: `FUN_0216D80C` (unlabeled routine at `0x0216D80C`): old grid x = v.
pub fn set_old_gx(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x36, v as u16);
}

/// C: `FUN_0216D810` (unlabeled routine at `0x0216D810`): old grid y (`s16 +0x38`).
pub fn old_gy(mem: &Mem, m: u32) -> i32 {
    s16_at(mem, m, 0x38)
}

/// C: `FUN_0216D818` (unlabeled routine at `0x0216D818`): old grid y = v.
pub fn set_old_gy(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x38, v as u16);
}

/// C: `FUN_0216D81C` (unlabeled routine at `0x0216D81C`): old grid z (`s16 +0x3A`).
pub fn old_gz(mem: &Mem, m: u32) -> i32 {
    s16_at(mem, m, 0x3A)
}

/// C: `FUN_0216D824` (unlabeled routine at `0x0216D824`): old grid z = v.
pub fn set_old_gz(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x3A, v as u16);
}

/// C: `FUN_0216D828` (unlabeled routine at `0x0216D828`): `out` (s16[3]) = the current grid x / y / z.
pub fn grid(mem: &mut Mem, m: u32, out: u32) {
    for k in 0..3 {
        let v = mem.r16(m + 0x3C + 2 * k);
        mem.w16(out + 2 * k, v);
    }
}

/// C: `FUN_0216D83C` (unlabeled routine at `0x0216D83C`): grid x (`s16 +0x3C`).
pub fn gx(mem: &Mem, m: u32) -> i32 {
    s16_at(mem, m, 0x3C)
}

/// C: `FUN_0216D844` (unlabeled routine at `0x0216D844`): grid x = v.
pub fn set_gx(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x3C, v as u16);
}

/// C: `FUN_0216D848` (unlabeled routine at `0x0216D848`): grid x += d.
pub fn add_gx(mem: &mut Mem, m: u32, d: u32) {
    let v = (mem.r16(m + 0x3C) as u32).wrapping_add(d);
    mem.w16(m + 0x3C, v as u16);
}

/// C: `FUN_0216D854` (unlabeled routine at `0x0216D854`): grid y (`s16 +0x3E`).
pub fn gy(mem: &Mem, m: u32) -> i32 {
    s16_at(mem, m, 0x3E)
}

/// C: `FUN_0216D85C` (unlabeled routine at `0x0216D85C`): grid y = v.
pub fn set_gy(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x3E, v as u16);
}

/// C: `FUN_0216D860` (unlabeled routine at `0x0216D860`): grid y += d.
pub fn add_gy(mem: &mut Mem, m: u32, d: u32) {
    let v = (mem.r16(m + 0x3E) as u32).wrapping_add(d);
    mem.w16(m + 0x3E, v as u16);
}

/// C: `FUN_0216D86C` (unlabeled routine at `0x0216D86C`): grid z (`s16 +0x40`).
pub fn gz(mem: &Mem, m: u32) -> i32 {
    s16_at(mem, m, 0x40)
}

/// C: `FUN_0216D874` (unlabeled routine at `0x0216D874`): grid z = v.
pub fn set_gz(mem: &mut Mem, m: u32, v: u32) {
    mem.w16(m + 0x40, v as u16);
}

/// C: `FUN_0216D87C` (unlabeled routine at `0x0216D87C`): grid z += d.
pub fn add_gz(mem: &mut Mem, m: u32, d: u32) {
    let v = (mem.r16(m + 0x40) as u32).wrapping_add(d);
    mem.w16(m + 0x40, v as u16);
}

/// C: `FUN_0216D888` (unlabeled routine at `0x0216D888`): the position vector (`m + 0x44`).
pub fn pos_ptr(m: u32) -> u32 {
    m.wrapping_add(0x44)
}

/// C: `FUN_0216D88C` (unlabeled routine at `0x0216D88C`): `out` = the position (VecFx32 `+0x44`).
pub fn get_pos(mem: &mut Mem, m: u32, out: u32) {
    copy_vec(mem, out, m + 0x44);
}

/// C: `FUN_0216D89C` (unlabeled routine at `0x0216D89C`): the position = `*v`.
pub fn set_pos(mem: &mut Mem, m: u32, v: u32) {
    copy_vec(mem, m + 0x44, v);
}

/// C: `FUN_0216D8AC` (unlabeled routine at `0x0216D8AC`): the position y (fx32 `+0x48`).
pub fn pos_y(mem: &Mem, m: u32) -> u32 {
    mem.r32(m + 0x48)
}

/// C: `FUN_0216D8B0` (unlabeled routine at `0x0216D8B0`): `out` = the VecFx32 at `+0x50`.
pub fn get_vec50(mem: &mut Mem, m: u32, out: u32) {
    copy_vec(mem, out, m + 0x50);
}

/// C: `FUN_0216D8C0` (unlabeled routine at `0x0216D8C0`): the VecFx32 at `+0x50` = `*v`.
pub fn set_vec50(mem: &mut Mem, m: u32, v: u32) {
    copy_vec(mem, m + 0x50, v);
}

/// C: `FUN_0216D8D0` (unlabeled routine at `0x0216D8D0`): `out` = the VecFx32 at `+0x5C`.
pub fn get_vec5c(mem: &mut Mem, m: u32, out: u32) {
    copy_vec(mem, out, m + 0x5C);
}

/// C: `FUN_0216D8E0` (unlabeled routine at `0x0216D8E0`): the VecFx32 at `+0x5C` = `*v`.
pub fn set_vec5c(mem: &mut Mem, m: u32, v: u32) {
    copy_vec(mem, m + 0x5C, v);
}

/// C: `FUN_0216D8F0` (unlabeled routine at `0x0216D8F0`): `out` = the VecFx32 at `+0x68`.
pub fn get_vec68(mem: &mut Mem, m: u32, out: u32) {
    copy_vec(mem, out, m + 0x68);
}

/// C: `FUN_0216D900` (unlabeled routine at `0x0216D900`): the VecFx32 at `+0x68` = `*v`.
pub fn set_vec68(mem: &mut Mem, m: u32, v: u32) {
    copy_vec(mem, m + 0x68, v);
}

/// C: `FUN_0216D910` (unlabeled routine at `0x0216D910`): the draw system of the MMDL's MMDLSYS.
pub fn draw_sys_of(mem: &Mem, m: u32) -> u32 {
    draw_sys(mem, sys_of(mem, m))
}

/// C: `FUN_0216D91C` (unlabeled routine at `0x0216D91C`): size byte `+0x7C`.
pub fn size_x(mem: &Mem, m: u32) -> u8 {
    mem.r8(m + 0x7C)
}

/// C: `FUN_0216D924` (unlabeled routine at `0x0216D924`): size byte `+0x7D`.
pub fn size_z(mem: &Mem, m: u32) -> u8 {
    mem.r8(m + 0x7D)
}

/// C: `FUN_0216D92C` (unlabeled routine at `0x0216D92C`): `out` (s32[3]) = the s8 offsets `+0x7E..+0x80`, `<< 12`.
pub fn offsets(mem: &mut Mem, m: u32, out: u32) {
    for k in 0..3 {
        let o = (mem.r8(m + 0x7E + k) as i8 as i32) << 12;
        mem.w32(out + 4 * k, o as u32);
    }
}

/// C: `FUN_0216D948` (unlabeled routine at `0x0216D948`): 1 when the draw side is on (flag 1).
pub fn draw_on(mem: &Mem, sys: u32) -> u32 {
    (sys_flags(mem, sys, 1) != 0) as u32
}

/// C: `FUN_0216D95C` (unlabeled routine at `0x0216D95C`): draw side on (`on == 1`) or off.
pub fn set_draw_on(mem: &mut Mem, sys: u32, on: u32) {
    if on == 1 {
        sys_set_flags(mem, sys, 1);
    } else {
        sys_clear_flags(mem, sys, 1);
    }
}

/// C: `FUN_0216D974` (unlabeled routine at `0x0216D974`): flag 8 set when `x == 0`, else cleared.
pub fn set_flag8(mem: &mut Mem, sys: u32, x: u32) {
    if x == 0 {
        sys_set_flags(mem, sys, 8);
    } else {
        sys_clear_flags(mem, sys, 8);
    }
}

/// C: `FUN_0216D98C` (unlabeled routine at `0x0216D98C`): 1 when flag 8 is clear.
pub fn flag8_clear(mem: &Mem, sys: u32) -> u32 {
    (sys_flags(mem, sys, 8) == 0) as u32
}

/// C: `FUN_0216D9A0` (unlabeled routine at `0x0216D9A0`): pause every MMDL in use (move bit 8) that does not have
/// move bit 0x80000; sets flag 0x20.
pub fn pause_all(mem: &mut Mem, sys: u32) {
    let mut i = 0;
    while let Some(m) = next_local(mem, sys, &mut i) {
        if bit_80000(mem, m) == 0 {
            pause(mem, m);
        }
    }
    sys_set_flags(mem, sys, 0x20);
}

/// C: `FUN_0216D9E4` (unlabeled routine at `0x0216D9E4`): unpause every MMDL in use; clears flag 0x20.
pub fn unpause_all(mem: &mut Mem, sys: u32) {
    let mut i = 0;
    while let Some(m) = next_local(mem, sys, &mut i) {
        unpause(mem, m);
    }
    sys_clear_flags(mem, sys, 0x20);
}

/// C: `FUN_0216DA20` (unlabeled routine at `0x0216DA20`): the MMDLSYS flags of the MMDL's system `& v`.
pub fn sys_flags_of(mem: &Mem, m: u32, v: u32) -> u32 {
    sys_flags(mem, sys_of(mem, m), v)
}

/// C: `FUN_0216DA30` (unlabeled routine at `0x0216DA30`): 1 when the MMDL is in use (status 1).
pub fn in_use(mem: &Mem, m: u32) -> u32 {
    (status(mem, m, 1) != 0) as u32
}

/// C: `FUN_0216DA44` (unlabeled routine at `0x0216DA44`): move bit 0x10 on.
pub fn set_bit10(mem: &mut Mem, m: u32) {
    set_bits(mem, m, 0x10);
}

/// C: `FUN_0216DA50` (unlabeled routine at `0x0216DA50`): move bit 0x10 off.
pub fn clear_bit10(mem: &mut Mem, m: u32) {
    clear_bits(mem, m, 0x10);
}

/// C: `FUN_0216DA5C` (unlabeled routine at `0x0216DA5C`): 1 when move bit 0x10 is set.
pub fn bit10(mem: &Mem, m: u32) -> u32 {
    (bits(mem, m, 0x10) != 0) as u32
}

/// C: `FUN_0216DA70` (unlabeled routine at `0x0216DA70`): move bit 0x20 on.
pub fn set_bit20(mem: &mut Mem, m: u32) {
    set_bits(mem, m, 0x20);
}

/// C: `FUN_0216DA7C` (unlabeled routine at `0x0216DA7C`): move bit 0x40 off.
pub fn clear_bit40(mem: &mut Mem, m: u32) {
    clear_bits(mem, m, 0x40);
}

/// C: `FUN_0216DA88` (unlabeled routine at `0x0216DA88`): move bit 2 (draw ready) on.
pub fn set_draw_ready(mem: &mut Mem, m: u32) {
    set_bits(mem, m, 2);
}

/// C: `FUN_0216DA94` (unlabeled routine at `0x0216DA94`): move bit 2 (draw ready) off.
pub fn clear_draw_ready(mem: &mut Mem, m: u32) {
    clear_bits(mem, m, 2);
}

/// C: `FUN_0216DAA0` (unlabeled routine at `0x0216DAA0`): 1 when move bit 2 (draw ready) is set.
pub fn draw_ready(mem: &Mem, m: u32) -> u32 {
    (bits(mem, m, 2) != 0) as u32
}

/// C: `FUN_0216DAB4` (unlabeled routine at `0x0216DAB4`): 1 when status 4 is set.
pub fn status4(mem: &Mem, m: u32) -> u32 {
    (status(mem, m, 4) != 0) as u32
}

/// Status bit `b` on when `on == 1`, else off.
fn status_onoff(mem: &mut Mem, m: u32, on: u32, b: u32) {
    if on == 1 {
        set_status(mem, m, b);
    } else {
        clear_status(mem, m, b);
    }
}

/// Move bit `b` on when `on == 1`, else off.
fn bits_onoff(mem: &mut Mem, m: u32, on: u32, b: u32) {
    if on == 1 {
        set_bits(mem, m, b);
    } else {
        clear_bits(mem, m, b);
    }
}

/// C: `FUN_0216DAC8` (unlabeled routine at `0x0216DAC8`): status 4 on (`on == 1`) or off.
pub fn set_status4(mem: &mut Mem, m: u32, on: u32) {
    status_onoff(mem, m, on, 4);
}

/// C: `FUN_0216DAE0` (unlabeled routine at `0x0216DAE0`): status 0x80 off when `on == 1`, else on.
pub fn set_status80_inv(mem: &mut Mem, m: u32, on: u32) {
    if on == 1 {
        clear_status(mem, m, 0x80);
    } else {
        set_status(mem, m, 0x80);
    }
}

/// C: `FUN_0216DAF8` (unlabeled routine at `0x0216DAF8`): pause (move bit 8 on).
pub fn pause(mem: &mut Mem, m: u32) {
    set_bits(mem, m, 8);
}

/// C: `FUN_0216DB04` (unlabeled routine at `0x0216DB04`): unpause (move bit 8 off).
pub fn unpause(mem: &mut Mem, m: u32) {
    clear_bits(mem, m, 8);
}

/// C: `FUN_0216DB10` (unlabeled routine at `0x0216DB10`): 1 when paused (move bit 8).
pub fn paused(mem: &Mem, m: u32) -> u32 {
    (bits(mem, m, 8) != 0) as u32
}

/// C: `FUN_0216DB24` (unlabeled routine at `0x0216DB24`): 1 when the draw side is on and the MMDL is draw ready.
pub fn drawable(mem: &Mem, m: u32) -> u32 {
    if draw_on(mem, sys_of(mem, m)) == 0 {
        return 0;
    }
    (draw_ready(mem, m) != 0) as u32
}

/// C: `FUN_0216DB4C` (unlabeled routine at `0x0216DB4C`): status 0x200 on (`on == 1`) or off.
pub fn set_status200(mem: &mut Mem, m: u32, on: u32) {
    status_onoff(mem, m, on, 0x200);
}

/// C: `FUN_0216DB68` (unlabeled routine at `0x0216DB68`): 1 when status 0x200 is set.
pub fn status200(mem: &Mem, m: u32) -> u32 {
    (status(mem, m, 0x200) != 0) as u32
}

/// C: `FUN_0216DB80` (unlabeled routine at `0x0216DB80`): status 0x20 on (`on == 1`) or off.
pub fn set_status20(mem: &mut Mem, m: u32, on: u32) {
    status_onoff(mem, m, on, 0x20);
}

/// C: `FUN_0216DB98` (unlabeled routine at `0x0216DB98`): status 0x800 (alias) on (`on == 1`) or off.
pub fn set_alias(mem: &mut Mem, m: u32, on: u32) {
    status_onoff(mem, m, on, 0x800);
}

/// C: `FUN_0216DBB4` (unlabeled routine at `0x0216DBB4`): 1 when the MMDL is an alias (status 0x800).
pub fn is_alias(mem: &Mem, m: u32) -> u32 {
    (status(mem, m, 0x800) != 0) as u32
}

/// C: `FUN_0216DBCC` (unlabeled routine at `0x0216DBCC`): move bit 0x40000 on (`on == 1`) or off.
pub fn set_bit40000(mem: &mut Mem, m: u32, on: u32) {
    bits_onoff(mem, m, on, 0x40000);
}

/// C: `FUN_0216DBE8` (unlabeled routine at `0x0216DBE8`): 1 when move bit 0x40000 is set.
pub fn bit_40000(mem: &Mem, m: u32) -> u32 {
    (bits(mem, m, 0x40000) != 0) as u32
}

/// C: `FUN_0216DC00` (unlabeled routine at `0x0216DC00`): 1 when status 0x1000 is set.
pub fn status1000(mem: &Mem, m: u32) -> u32 {
    (status(mem, m, 0x1000) != 0) as u32
}

/// C: `FUN_0216DC18` (unlabeled routine at `0x0216DC18`): move bit 0x20000 on (`on == 1`) or off.
pub fn set_bit20000(mem: &mut Mem, m: u32, on: u32) {
    bits_onoff(mem, m, on, 0x20000);
}

/// C: `FUN_0216DC34` (unlabeled routine at `0x0216DC34`): 1 when move bit 0x20000 is set.
pub fn bit_20000(mem: &Mem, m: u32) -> u32 {
    (bits(mem, m, 0x20000) != 0) as u32
}

/// C: `FUN_0216DC4C` (unlabeled routine at `0x0216DC4C`): 1 when move bit 0x200 (acmd set) is set.
pub fn acmd_active(mem: &Mem, m: u32) -> u32 {
    (bits(mem, m, 0x200) != 0) as u32
}

/// C: `FUN_0216DC64` (unlabeled routine at `0x0216DC64`): move bit 0x80000 on (`on == 1`) or off.
pub fn set_bit80000(mem: &mut Mem, m: u32, on: u32) {
    bits_onoff(mem, m, on, 0x80000);
}

/// C: `FUN_0216DC80` (unlabeled routine at `0x0216DC80`): 1 when move bit 0x80000 is set.
pub fn bit_80000(mem: &Mem, m: u32) -> u32 {
    (bits(mem, m, 0x80000) != 0) as u32
}

/// C: `FUN_0216DC98` (unlabeled routine at `0x0216DC98`): status 0x400 on (`on == 1`) or off.
pub fn set_status400(mem: &mut Mem, m: u32, on: u32) {
    status_onoff(mem, m, on, 0x400);
}

/// C: `FUN_0216DCB4` (unlabeled routine at `0x0216DCB4`): 1 when status 0x400 is set.
pub fn status400(mem: &Mem, m: u32) -> u32 {
    (status(mem, m, 0x400) != 0) as u32
}

/// C: `FUN_0216DCCC` (unlabeled routine at `0x0216DCCC`): move bit 0x100000 on (`on == 1`) or off.
pub fn set_bit100000(mem: &mut Mem, m: u32, on: u32) {
    bits_onoff(mem, m, on, 0x10_0000);
}

/// C: `FUN_0216DCE8` (unlabeled routine at `0x0216DCE8`): move bit 0x200000 on (`on == 1`) or off.
pub fn set_bit200000(mem: &mut Mem, m: u32, on: u32) {
    bits_onoff(mem, m, on, 0x20_0000);
}

/// C: `FUN_0216DD04` (unlabeled routine at `0x0216DD04`): 1 when move bit 0x200000 is set.
pub fn bit_200000(mem: &Mem, m: u32) -> u32 {
    (bits(mem, m, 0x20_0000) != 0) as u32
}

// ---- search --------------------------------------------------------------------------------------------------

/// C: `FUN_0216DD1C` (unlabeled routine at `0x0216DD1C`): the "next MMDL in use" iterator. From index `*idx`
/// (`u32` at `idx`) while it is below the count: the index moves on, and the first MMDL in use (status bit 0) is
/// stored to `*out`, returning `true`; none left: `false`. Callers loop `while next_in_use(sys, &m, &i)` with
/// `i = 0`. The index compare is unsigned (an index at or past the count returns `false` at once).
pub fn next_in_use(mem: &mut Mem, sys: u32, out: u32, idx: u32) -> bool {
    let n = count(mem, sys) as u32;
    if mem.r32(idx) >= n {
        return false;
    }
    let mut p = mem
        .r32(sys.wrapping_add(0x18))
        .wrapping_add(mem.r32(idx).wrapping_shl(8));
    loop {
        let i = mem.r32(idx).wrapping_add(1);
        mem.w32(idx, i);
        if status(mem, p, ST_IN_USE) != 0 {
            mem.w32(out, p);
            return true;
        }
        p = p.wrapping_add(MMDL_SIZE);
        if mem.r32(idx) >= n {
            return false;
        }
    }
}

/// C: `FUN_0216DD60` (unlabeled routine at `0x0216DD60`): collect the MMDLs in use at grid `(x, z)`
/// (`FUN_02163C04(m, x, z, flag) == 1`) into the LIST at `list` (0x44 bytes, cleared first: `u32` count, then up to
/// 16 pointers); returns the count.
pub fn list_at<E: FieldEnv>(env: &mut E, sys: u32, x: u32, z: u32, flag: u32, list: u32) -> u32 {
    env.mem().fill(list, 0x44, 0);
    let mut i = 0;
    while let Some(m) = next_local(env.mem(), sys, &mut i) {
        if env.callv(AT_GRID, &[m, x, z, flag]) == 1 {
            let n = env.r32(list);
            env.w32(list, n.wrapping_add(1));
            env.w32(list.wrapping_add(n.wrapping_shl(2)).wrapping_add(4), m);
            if env.r32(list) as i32 >= 16 {
                break;
            }
        }
    }
    env.r32(list)
}

/// C: `FUN_0216DDC0` (unlabeled routine at `0x0216DDC0`): the first MMDL of [`list_at`] (a local list), or 0.
pub fn find_at<E: FieldEnv>(env: &mut E, sys: u32, x: u32, z: u32, flag: u32) -> u32 {
    let l = env.mem().push_local(0x44);
    let n = list_at(env, sys, x, z, flag, l);
    let r = if n != 0 { env.r32(l + 4) } else { 0 };
    env.mem().pop_local(l);
    r
}

/// C: `FUN_0216DDDC` (unlabeled routine at `0x0216DDDC`): the first MMDL of [`list_at`] whose height
/// (`FUN_0216D8AC`, reached through the seam) is within `dy` of `y`: `|h - y| < dy` (signed), where a negative
/// difference `d` is negated as `FX_Mul(d, -1.0)` with rounding (`(d * -4096 + 0x800) >> 12`, 64-bit); else 0.
pub fn find_at_height<E: FieldEnv>(
    env: &mut E,
    sys: u32,
    x: u32,
    z: u32,
    y: u32,
    dy: u32,
    flag: u32,
) -> u32 {
    let l = env.mem().push_local(0x44);
    let n = list_at(env, sys, x, z, flag, l);
    let mut r = 0;
    let mut i = 0u32;
    while i < n {
        let m = env.r32(l + 4 + 4 * i);
        let h = env.callv(0x0216_D8AC, &[m]);
        let mut d = h.wrapping_sub(y);
        if (d as i32) < 0 {
            d = ((d as i32 as i64 * -4096 + 0x800) >> 12) as u32;
        }
        if (d as i32) < dy as i32 {
            r = env.r32(l + 4 + 4 * i);
            break;
        }
        i += 1;
    }
    env.mem().pop_local(l);
    r
}

/// C: `FUN_0216DE54` (unlabeled routine at `0x0216DE54`): the first MMDL in use with move code `code`, or 0.
pub fn find_by_move_code(mem: &Mem, sys: u32, code: u32) -> u32 {
    let mut i = 0;
    while let Some(m) = next_local(mem, sys, &mut i) {
        if code == move_code(mem, m) as u32 {
            return m;
        }
    }
    0
}

/// C: `FUN_0216DE94` (unlabeled routine at `0x0216DE94`): the first MMDL in use that is no alias and has OBJ id
/// `id`, or 0.
pub fn find_by_id(mem: &Mem, sys: u32, id: u32) -> u32 {
    let mut i = 0;
    while let Some(m) = next_local(mem, sys, &mut i) {
        if is_alias(mem, m) == 0 && id == obj_id(mem, m) as u32 {
            return m;
        }
    }
    0
}

/// C: `FUN_0216DEE0` (unlabeled routine at `0x0216DEE0`): the player object ([`find_by_id`] of 0xFF).
pub fn find_player(mem: &Mem, sys: u32) -> u32 {
    find_by_id(mem, sys, PLAYER_ID)
}

/// C: `FUN_0216DEEC` (unlabeled routine at `0x0216DEEC`): the first MMDL not in use, or 0 (a do-while over the
/// count, so MMDL 0 is always looked at).
pub fn first_free(mem: &Mem, sys: u32) -> u32 {
    let n = count(mem, sys) as i32;
    let mut m = mem.r32(sys + 0x18);
    let mut i = 0i32;
    loop {
        if status(mem, m, 1) == 0 {
            return m;
        }
        i += 1;
        m = m.wrapping_add(MMDL_SIZE);
        if i >= n {
            return 0;
        }
    }
}

/// C: `FUN_0216DF1C` (unlabeled routine at `0x0216DF1C`): [`delete`] every MMDL in use without status 0x20.
pub fn delete_unkept<E: FieldEnv>(env: &mut E, sys: u32) {
    let mut i = 0;
    while let Some(m) = next_local(env.mem(), sys, &mut i) {
        if status(env.mem(), m, 0x20) == 0 {
            obj_id(env.mem(), m);
            delete(env, m);
        }
    }
}

/// C: `FUN_0216DF64` (unlabeled routine at `0x0216DF64`): register the MMDL task (`+0x84`) with [`TASK_CALLBACK`]
/// at the system's priority + 1 for move codes 0x30 / 0x32, - 1 for the player, else + 0.
pub fn task_add<E: FieldEnv>(env: &mut E, m: u32, sys: u32) {
    let prio = task_priority(env.mem(), sys) as u32;
    let c = move_code(env.mem(), m);
    let k: u32 = if c == 0x30 || c == 0x32 {
        1
    } else if obj_id(env.mem(), m) as u32 == PLAYER_ID {
        u32::MAX
    } else {
        0
    };
    let ts = task_sys(env.mem(), sys);
    let t = env.callv(TASK_ADD, &[ts, TASK_CALLBACK, m, prio.wrapping_add(k)]);
    env.w32(m + 0x84, t);
}

/// C: `FUN_0216DFAC` (unlabeled routine at `0x0216DFAC`): delete the MMDL task (`+0x84`) if there is one.
pub fn task_del<E: FieldEnv>(env: &mut E, m: u32) {
    let t = env.r32(m + 0x84);
    if t != 0 {
        env.callv(TASK_DEL, &[t]);
        env.w32(m + 0x84, 0);
    }
}

/// C: `FUN_0216DFC4` (unlabeled routine at `0x0216DFC4`): 1 when another MMDL in use of the same system has OBJ code
/// `code` (and not 0xFFFF), else 0.
pub fn obj_code_in_use(mem: &Mem, m: u32, code: u32) -> u32 {
    let sys = sys_of(mem, m);
    let mut i = 0;
    while let Some(o) = next_local(mem, sys, &mut i) {
        if o != m {
            let c = obj_code(mem, o);
            if c != 0xFFFF && c as u32 == code {
                return 1;
            }
        }
    }
    0
}

// ---- position / header changes -------------------------------------------------------------------------------

/// C: `FUN_0216E014` (unlabeled routine at `0x0216E014`): move the MMDL to the position `*v`: grid = grid of each
/// coordinate, position = `*v`, `FUN_02163F10(m)`, display dir `dir`, [`acmd_reset`], move bit 0x20 on, 0x50 off.
pub fn set_position<E: FieldEnv>(env: &mut E, m: u32, v: u32, dir: u32) {
    let mem = env.mem();
    set_gx(mem, m, grid16(mem.r32(v)));
    set_gy(mem, m, grid16(mem.r32(v + 4)));
    set_gz(mem, m, grid16(mem.r32(v + 8)));
    set_pos(mem, m, v);
    env.callv(POS_CHANGED, &[m]);
    let mem = env.mem();
    set_dir_disp_raw(mem, m, dir);
    acmd_reset(mem, m);
    set_bits(mem, m, 0x20);
    clear_bits(mem, m, 0x50);
}

/// C: `FUN_0216E084` (unlabeled routine at `0x0216E084`): [`set_position`] to the grid `(gx, gy, gz)` (x / z at the
/// tile centre) with dir `(u16) dir`.
pub fn set_grid_position<E: FieldEnv>(env: &mut E, m: u32, gx: u32, gy: u32, gz: u32, dir: u32) {
    let v = env.mem().push_local(12);
    env.w32(v, (gx << 16).wrapping_add(0x8000));
    env.w32(v + 4, gy << 16);
    env.w32(v + 8, (gz << 16).wrapping_add(0x8000));
    set_position(env, m, v, dir & 0xFFFF);
    env.mem().pop_local(v);
}

/// C: `FUN_0216E0AC` (unlabeled routine at `0x0216E0AC`): re-read an MMDL from the HEADER `hdr`: move delete,
/// [`init_header_fields`], [`init_pos`], [`set_move_procs`], [`move_init_kind`], [`header_post`].
pub fn reset_from_header<E: FieldEnv>(env: &mut E, m: u32, hdr: u32) {
    let sys = sys_of(env.mem(), m);
    move_delete(env, m);
    init_header_fields(env.mem(), m, hdr);
    init_pos(env, m, hdr);
    set_move_procs(env, m);
    move_init_kind(env, sys, m);
    header_post(env, m, hdr, 0);
}

/// C: `FUN_0216E0E8` (unlabeled routine at `0x0216E0E8`): change the move code: move delete, `+0xE = code`,
/// [`set_move_procs`], [`move_init_kind`].
pub fn change_move_code<E: FieldEnv>(env: &mut E, m: u32, code: u32) {
    let sys = sys_of(env.mem(), m);
    move_delete(env, m);
    set_move_code(env.mem(), m, code);
    set_move_procs(env, m);
    move_init_kind(env, sys, m);
}

/// C: `FUN_0216E114` (unlabeled routine at `0x0216E114`): with the draw side on: the rail refresh
/// `FUN_02164010(m)` (move bit 0x1000), `+0x2A = 0`, and when not draw ready: [`set_draw_procs`], draw init, draw
/// ready.
pub fn draw_setup<E: FieldEnv>(env: &mut E, m: u32) {
    let s = sys_of(env.mem(), m);
    if draw_on(env.mem(), s) == 0 {
        return;
    }
    if bits(env.mem(), m, 0x1000) != 0 {
        env.callv(RAIL_REFRESH, &[m]);
    }
    set_field_2a(env.mem(), m, 0);
    if draw_ready(env.mem(), m) == 0 {
        set_draw_procs(env, m);
        draw_init(env, m);
        set_draw_ready(env.mem(), m);
    }
}

/// C: `FUN_0216E160` (unlabeled routine at `0x0216E160`): `+0x90` = the draw-proc table of the OBJ DATA draw kind
/// (byte 3; `FUN_0216E40C`, reached through the seam).
pub fn set_draw_procs<E: FieldEnv>(env: &mut E, m: u32) {
    let k = env.r8(obj_data(m) + 3) as u32;
    let t = env.callv(0x0216_E40C, &[k]);
    env.w32(m + 0x90, t);
}

/// C: `FUN_0216E174` (unlabeled routine at `0x0216E174`): move bits 0x6C000 off.
pub fn clear_bits_6c000(mem: &mut Mem, m: u32) {
    clear_bits(mem, m, 0x6C000);
}

/// C: `FUN_0216E180` (unlabeled routine at `0x0216E180`): clear the MMDL (0x100 bytes).
pub fn clear(mem: &mut Mem, m: u32) {
    mem.fill(m, MMDL_SIZE, 0);
}

/// C: `FUN_0216E190` (unlabeled routine at `0x0216E190`): move bit 0x20 on, [`clear_bits_6c000`].
pub fn mark_moved(mem: &mut Mem, m: u32) {
    set_bits(mem, m, 0x20);
    clear_bits_6c000(mem, m);
}

/// C: `FUN_0216E1A4` (unlabeled routine at `0x0216E1A4`): `out` (u16[3]) = OBJ id, OBJ code, zone id.
pub fn key(mem: &mut Mem, m: u32, out: u32) {
    let v = obj_id(mem, m);
    mem.w16(out, v);
    let v = obj_code(mem, m);
    mem.w16(out + 2, v);
    let v = zone(mem, m);
    mem.w16(out + 4, v);
}

/// C: `FUN_0216E1C4` (unlabeled routine at `0x0216E1C4`): 1 when the MMDL is in use and matches the key `k`
/// (u16[3]: OBJ id, OBJ code, zone id).
pub fn matches_key(mem: &Mem, m: u32, k: u32) -> u32 {
    if status(mem, m, 1) == 0 {
        return 0;
    }
    if mem.r16(k) != obj_id(mem, m)
        || mem.r16(k + 2) != obj_code(mem, m)
        || mem.r16(k + 4) != zone(mem, m)
    {
        return 0;
    }
    1
}

/// C: `FUN_0216E204` (unlabeled routine at `0x0216E204`): change the OBJ code: draw delete (draw side on and draw
/// ready), `+0xC = code`, [`load_obj_data`], sizes and offsets from the OBJ DATA, not draw ready, [`mark_moved`],
/// [`draw_setup`].
pub fn change_obj_code<E: FieldEnv>(env: &mut E, m: u32, code: u32) {
    let sys = sys_of(env.mem(), m);
    if draw_on(env.mem(), sys) == 1 && draw_ready(env.mem(), m) == 1 {
        draw_delete(env, m);
    }
    set_obj_code(env.mem(), m, code);
    load_obj_data(env, sys, code, m);
    obj_data_sizes(env.mem(), m, true);
    clear_draw_ready(env.mem(), m);
    mark_moved(env.mem(), m);
    draw_setup(env, m);
}

// ---- OBJ code data -------------------------------------------------------------------------------------------

/// C: `FUN_0216E280` (unlabeled routine at `0x0216E280`): open ARC 48 (OBJ data) into `+0x14`.
pub fn open_obj_arc<E: FieldEnv>(env: &mut E, sys: u32, heap: u32) {
    let h = env.callv(ARC_OPEN, &[0x30, heap]);
    env.w32(sys + 0x14, h);
}

/// C: `FUN_0216E290` (unlabeled routine at `0x0216E290`): close ARC 48 (`+0x14`).
pub fn close_obj_arc<E: FieldEnv>(env: &mut E, sys: u32) {
    let h = env.r32(sys + 0x14);
    env.callv(ARC_CLOSE, &[h]);
}

/// C: `FUN_0216E29C` (unlabeled routine at `0x0216E29C`): load member `i` of ARC 58 into the OBJ code table
/// (`FUN_02048F4C(+0x44, 0x3A, i)`).
pub fn load_obj_table<E: FieldEnv>(env: &mut E, sys: u32, i: u32) {
    let t = env.r32(sys + 0x44);
    env.callv(ARC_LOAD, &[t, 0x3A, i]);
}

/// Address of the OBJ DATA records of the OBJ code table `t` (after the u16 count and the codes, padded to a
/// word): `t + (((count + 1) / 2 * 2) & 0xFFFF + 2) * 2`.
fn table_records(t: u32, count: u32) -> u32 {
    let h = ((((count + 1) as i32 / 2) as u32) << 1) & 0xFFFF;
    t.wrapping_add((h + 2) * 2)
}

/// C: `FUN_0216E2AC` (unlabeled routine at `0x0216E2AC`): copy OBJ DATA record `i` of the OBJ code table to `out`
/// when `i` is below the count.
pub fn table_record(mem: &mut Mem, sys: u32, i: u32, out: u32) {
    let t = mem.r32(sys + 0x44);
    let n = mem.r16(t) as u32;
    if i < n {
        let r = table_records(t, n).wrapping_add(i.wrapping_mul(OBJ_DATA_SIZE));
        copy_halfwords(mem, out, r, 14);
    }
}

/// C: `FUN_0216E2E0` (unlabeled routine at `0x0216E2E0`): the count of the OBJ code table.
pub fn table_count(mem: &Mem, sys: u32) -> u16 {
    let t = mem.r32(sys + 0x44);
    mem.r16(t)
}

/// C: `FUN_0216E2E8` (unlabeled routine at `0x0216E2E8`): the OBJ DATA of `code` into `out`: from the OBJ code table
/// (count below 0x1A and the code listed), else from an MMDL in use with that OBJ code (its `+0xE4`), else read
/// from ARC 48 (`FUN_0204944C(+0x14, 0, (u16) (FUN_0200F128(code) * 0x1C + 4), 0x1C, out)`).
pub fn get_obj_data<E: FieldEnv>(env: &mut E, sys: u32, code: u32, out: u32) {
    let t = env.r32(sys + 0x44);
    let n = env.r16(t) as u32;
    if n < 0x1A {
        let recs = table_records(t, n);
        for i in 0..n {
            if code == env.r16(t + 2 + 2 * i) as u32 {
                copy_halfwords(env.mem(), out, recs + OBJ_DATA_SIZE * i, 14);
                return;
            }
        }
    }
    let mut i = 0;
    while let Some(m) = next_local(env.mem(), sys, &mut i) {
        if code == obj_code(env.mem(), m) as u32 {
            copy_halfwords(env.mem(), out, m + 0xE4, 14);
            return;
        }
    }
    let k = env.callv(OBJ_CODE_INDEX, &[code]);
    let ofs = (k.wrapping_mul(OBJ_DATA_SIZE).wrapping_add(4)) & 0xFFFF;
    let h = env.r32(sys + 0x14);
    env.call(
        ARC_READ,
        &mut [
            CArg::V(h),
            CArg::V(0),
            CArg::V(ofs),
            CArg::V(OBJ_DATA_SIZE),
            CArg::Out(out, OBJ_DATA_SIZE),
        ],
    );
}

/// C: `FUN_0216E39C` (unlabeled routine at `0x0216E39C`): load the OBJ DATA of `code` into the MMDL (`+0xE4`,
/// [`get_obj_data`]) with the MMDL out of use meanwhile (status 1 off, then on).
pub fn load_obj_data<E: FieldEnv>(env: &mut E, sys: u32, code: u32, m: u32) {
    clear_status(env.mem(), m, 1);
    get_obj_data(env, sys, code, m + 0xE4);
    set_status(env.mem(), m, 1);
}

/// C: `FUN_0216E3C4` (unlabeled routine at `0x0216E3C4`): the OBJ DATA of the MMDL (`m + 0xE4`, 0x1C bytes).
pub fn obj_data(m: u32) -> u32 {
    m.wrapping_add(0xE4)
}

/// C: `FUN_0216E3C8` (unlabeled routine at `0x0216E3C8`): `x + 0x10`.
pub fn plus_10a(x: u32) -> u32 {
    x.wrapping_add(0x10)
}

/// C: `FUN_0216E3CC` (unlabeled routine at `0x0216E3CC`): `x + 0x10`.
pub fn plus_10b(x: u32) -> u32 {
    x.wrapping_add(0x10)
}

/// C: `FUN_0216E3D8` (unlabeled routine at `0x0216E3D8`, also reached through the veneer `0216E3D0`): the OBJ code
/// of a header code: 0xA2..=0xB1 are taken from the event vars 0x4020.. (`FUN_020143CC(ew, code + 0x3F7E)`; 0xA with
/// no event work), the others are returned as they are (the whole register).
pub fn resolve_obj_code<E: FieldEnv>(env: &mut E, ew: u32, code: u32) -> u32 {
    if !(0xA2..=0xB1).contains(&code) {
        return code;
    }
    if ew == 0 {
        return 0xA;
    }
    let p = env.callv(VAR_ADDR, &[ew, (code + 0x3F7E) & 0xFFFF]);
    env.r16(p) as u32
}

/// C: `FUN_0216E400` (unlabeled routine at `0x0216E400`): the move-proc table of move code `code` (ROM
/// `0x021D5E04[code]`, [`MOVE_PROC_TABLES`]). Panics past the 47 entries (the original reads what follows).
pub fn move_proc_table(code: u32) -> u32 {
    MOVE_PROC_TABLES[code as usize]
}

/// C: `FUN_0216E40C` (unlabeled routine at `0x0216E40C`): the draw-proc table of draw kind `k` (ROM
/// `0x021D770C[k]`, [`DRAW_PROC_TABLES`]). Panics past the 21 entries (the original reads what follows).
pub fn draw_proc_table(k: u32) -> u32 {
    DRAW_PROC_TABLES[k as usize]
}

/// C: `FUN_0216E418` (unlabeled routine at `0x0216E418`): 1 when the HEADER is an alias (event id `+0xA` = 0xFFFF).
pub fn header_is_alias(mem: &Mem, hdr: u32) -> u32 {
    (mem.r16(hdr + 0xA) == 0xFFFF) as u32
}

/// C: `FUN_0216E434` (unlabeled routine at `0x0216E434`): 1 for `x` in 4..=8 and 11..=13; every other `x` is
/// returned as it is (the jump table's default path leaves r0 alone).
pub fn kind_434(x: u32) -> u32 {
    match x {
        4..=8 | 11..=13 => 1,
        _ => x,
    }
}

/// C: `FUN_0216E464` (unlabeled routine at `0x0216E464`): an empty callback (`bx lr`; move-proc table entries).
pub fn nop_464(_m: u32) {}

/// C: `FUN_0216E468` (unlabeled routine at `0x0216E468`): an empty callback.
pub fn nop_468(_m: u32) {}

/// C: `FUN_0216E46C` (unlabeled routine at `0x0216E46C`): an empty callback.
pub fn nop_46c(_m: u32) {}

/// C: `FUN_0216E470` (unlabeled routine at `0x0216E470`): an empty callback.
pub fn nop_470(_m: u32) {}

// ---- save positions ------------------------------------------------------------------------------------------

/// C: `FUN_0216E474` (unlabeled routine at `0x0216E474`): the size of the save-position work (0x168).
pub fn save_work_size() -> u32 {
    SAVE_WORK_SIZE
}

/// C: `FUN_0216E47C` (unlabeled routine at `0x0216E47C`): init the save-position work (`MIi_CpuFill32(-1, p,
/// 0x168)`).
pub fn save_work_init(mem: &mut Mem, p: u32) {
    for k in 0..SAVE_WORK_SIZE / 4 {
        mem.w32(p + 4 * k, u32::MAX);
    }
}

/// C: `FUN_0216E490` (unlabeled routine at `0x0216E490`): 1 when the VecFx32 at `v` is a stored position (not all
/// three -1).
pub fn is_stored(mem: &Mem, v: u32) -> u32 {
    let all = mem.r32(v) == u32::MAX && mem.r32(v + 4) == u32::MAX && mem.r32(v + 8) == u32::MAX;
    (!all) as u32
}

/// ROM table `0x02172A70`: `(zone, OBJ id, save slot)` of the objects whose position is kept in the
/// save-position work (OBJ codes 0x6D and 0x2003..0x2005), then the terminator `(0xFFFF, 0xFFFF, 0x1E)`.
pub const SAVE_SLOTS: [(u16, u16, u16); 27] = [
    (0x13F, 2, 0),
    (0x14D, 0, 1),
    (0x14D, 1, 2),
    (0x14D, 2, 3),
    (0x14E, 0, 4),
    (0x14E, 1, 5),
    (0x14E, 2, 6),
    (0x14F, 0, 7),
    (0x14F, 1, 8),
    (0xE2, 1, 9),
    (0xE0, 1, 10),
    (0xD8, 1, 11),
    (0x172, 0, 12),
    (0x172, 1, 13),
    (0x17A, 0, 14),
    (0x17A, 1, 15),
    (0x183, 0, 16),
    (0x183, 1, 17),
    (0x98, 0xD, 18),
    (0xD0, 0, 19),
    (0xE7, 0, 20),
    (0xE7, 1, 21),
    (0xE7, 3, 22),
    (0xE7, 2, 23),
    (0x17F, 0xA, 24),
    (0x15A, 5, 25),
    (0xFFFF, 0xFFFF, 0x1E),
];

/// C: `FUN_0216E4B0` (unlabeled routine at `0x0216E4B0`): the save slot of the object `id` of
/// `zone` ([`SAVE_SLOTS`]), or [`NO_SLOT`]. Each entry is matched before the end test, so `(0xFFFF, 0xFFFF)`
/// matches the terminator (whose slot is 0x1E anyway). The arguments are compared as whole registers.
pub fn save_slot(zone: u32, id: u32) -> u16 {
    for &(z, i, slot) in SAVE_SLOTS.iter() {
        if z as u32 == zone && i as u32 == id {
            return slot;
        }
        if z == 0xFFFF || i == 0xFFFF {
            break;
        }
    }
    NO_SLOT
}

/// C: `FUN_0216E4EC` (unlabeled routine at `0x0216E4EC`): the save slot of an MMDL whose OBJ code keeps its position
/// ([`keeps_position`]): [`save_slot`] of its zone and OBJ id; else [`NO_SLOT`].
pub fn save_slot_of(mem: &Mem, m: u32) -> u16 {
    if keeps_position(obj_code(mem, m) as u32) == 1 {
        save_slot(zone(mem, m) as u32, obj_id(mem, m) as u32)
    } else {
        NO_SLOT
    }
}

/// The save-position entry of slot `k` of the MMDL's system.
fn save_entry(mem: &Mem, m: u32, k: u16) -> u32 {
    let w = mem.r32(sys_of(mem, m) + 0x2C);
    w.wrapping_add(12 * k as u32)
}

/// C: `FUN_0216E51C` (unlabeled routine at `0x0216E51C`): 1 when the MMDL has a save slot holding a stored
/// position.
pub fn has_saved_pos(mem: &Mem, m: u32) -> u32 {
    let k = save_slot_of(mem, m);
    if k == NO_SLOT {
        return 0;
    }
    is_stored(mem, save_entry(mem, m, k))
}

/// C: `FUN_0216E544` (unlabeled routine at `0x0216E544`): copy the stored position of the MMDL to `out`, returning
/// 1; 0 (nothing written) when there is none.
pub fn saved_pos(mem: &mut Mem, m: u32, out: u32) -> u32 {
    let k = save_slot_of(mem, m);
    if k == NO_SLOT {
        return 0;
    }
    let e = save_entry(mem, m, k);
    if is_stored(mem, e) == 0 {
        return 0;
    }
    copy_vec(mem, out, e);
    1
}

/// C: `FUN_0216E57C` (unlabeled routine at `0x0216E57C`): store the position of an MMDL with a save slot.
pub fn store_pos(mem: &mut Mem, m: u32) {
    let k = save_slot_of(mem, m);
    if k == NO_SLOT {
        return;
    }
    let e = save_entry(mem, m, k);
    copy_vec(mem, e, m + 0x44);
}

/// C: `FUN_0216E5B0` (unlabeled routine at `0x0216E5B0`): 1 when an MMDL stands at the position `*v`
/// ([`find_at_height`] at the grid of `v.x` / `v.z` (s16), height `v.y` within 2 tiles, flag 0) and has a stored
/// position; else 0.
pub fn stored_object_at<E: FieldEnv>(env: &mut E, sys: u32, v: u32) -> u32 {
    let x = grid16(env.r32(v));
    let z = grid16(env.r32(v + 8));
    let y = env.r32(v + 4);
    let m = find_at_height(env, sys, x, z, y, 0x20000, 0);
    if m != 0 && has_saved_pos(env.mem(), m) == 1 {
        1
    } else {
        0
    }
}

/// C: `FUN_0216E5FC` (unlabeled routine at `0x0216E5FC`): 1 for the OBJ codes whose position is saved: 0x6D and
/// 0x2003..=0x2005 (`(u16) (code - 0x2003) <= 2`).
pub fn keeps_position(code: u32) -> u32 {
    (code == 0x6D || (code.wrapping_add(0xDFFD) & 0xFFFF) <= 2) as u32
}

/// C: `FUN_0216E618` (unlabeled routine at `0x0216E618`): make `hdr` a grid header at `(gx, gz)` (u16 each) with
/// height `y`.
pub fn header_set_grid(mem: &mut Mem, hdr: u32, gx: u32, gz: u32, y: u32) {
    mem.w32(hdr + 0x18, 0);
    mem.w16(hdr + 0x1C, gx as u16);
    mem.w16(hdr + 0x1E, gz as u16);
    mem.w32(hdr + 0x20, y);
}

#[cfg(test)]
mod tests;
