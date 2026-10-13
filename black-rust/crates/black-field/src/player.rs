//! The grid MAP CTRL callbacks (`fieldmap_ctrl_grid.c`, ROM table `0x021D88F8`) and `field_player.c` (overlay 21,
//! `0x021A2D88..0x021A36E4`): FIELD_PLAYER, the 0x10-byte front object of the player (`+0` FIELDMAP, `+4` -> PLAYER
//! CORE (`crate::player_core`), `+8` -> PLAYER GRID (`crate::player_grid`, grid maps), `+0xC` -> the no-grid player
//! (F3b, rail maps)); most of its functions pick the grid or the no-grid half by the map kind (`FUN_02188CFC`) or
//! tail-call the core.
//!
//! F3a-spec specced all 72 functions (`tools/rom-diff/specs/black-field/player.spec`, header: records, move forms);
//! F3a ported them. Every callee outside this area (the core, the grid player and the no-grid player of the other
//! F3a areas / F3b, fldmmdl, mmdl_move, fieldmap, the rail system, the PLAYER record of the GAMEDATA, the heap) is
//! reached through [`FieldEnv::call`] by address (the spec keeps them as `force` stubs); callees inside the area are
//! Rust calls. The stack locals the original hands to its callees (positions, rail locations, grid triples) are
//! local byte arrays or [`Mem::push_local`] windows. The move-form table (ROM `0x021D890C`, 2 rows (player sex) x 10
//! entries `{u16 OBJ code, u16 form}`) is embedded as [`FORM_TABLE`]: form 0 walk (codes 1 / 4), 1 cycle (2 / 5),
//! 2 and 3 surf (3 / 6), 4 = the special codes 0x88.. (entries 4..9). Directions: 0 up, 1 down, 2 left, 3 right.

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_2D88,
    0x021A_2DE4,
    0x021A_2DF0,
    0x021A_2E34,
    0x021A_2E44,
    0x021A_2E50,
    0x021A_2E90,
    0x021A_2EB8,
    0x021A_2F20,
    0x021A_2F48,
    0x021A_2F54,
    0x021A_2F60,
    0x021A_2F6C,
    0x021A_2F8C,
    0x021A_2F98,
    0x021A_2FA4,
    0x021A_2FB0,
    0x021A_2FD0,
    0x021A_2FE4,
    0x021A_3038,
    0x021A_3044,
    0x021A_3050,
    0x021A_305C,
    0x021A_3068,
    0x021A_3074,
    0x021A_3080,
    0x021A_308C,
    0x021A_3098,
    0x021A_30A4,
    0x021A_30B0,
    0x021A_30BC,
    0x021A_30C8,
    0x021A_3128,
    0x021A_3190,
    0x021A_31A4,
    0x021A_31D0,
    0x021A_3208,
    0x021A_3220,
    0x021A_322C,
    0x021A_3238,
    0x021A_3244,
    0x021A_3250,
    0x021A_32A8,
    0x021A_330C,
    0x021A_3364,
    0x021A_3460,
    0x021A_3484,
    0x021A_34EC,
    0x021A_34F8,
    0x021A_3504,
    0x021A_3524,
    0x021A_3550,
    0x021A_356C,
    0x021A_3584,
    0x021A_35A0,
    0x021A_35BC,
    0x021A_35E0,
    0x021A_35F0,
    0x021A_35FC,
    0x021A_3624,
    0x021A_3644,
    0x021A_3664,
    0x021A_3674,
    0x021A_3680,
    0x021A_368C,
    0x021A_3698,
    0x021A_36A4,
    0x021A_36B0,
    0x021A_36BC,
    0x021A_36C8,
    0x021A_36D4,
    0x021A_36E0,
];

/// ROM `0x021D890C`: per player sex (row), 10 entries `(OBJ code, form)`.
pub const FORM_TABLE: [[(u16, u16); 10]; 2] = [
    [
        (0x01, 0),
        (0x02, 1),
        (0x03, 2),
        (0x03, 3),
        (0x88, 4),
        (0x8A, 4),
        (0x8C, 4),
        (0x9E, 4),
        (0xA0, 4),
        (0x9C, 4),
    ],
    [
        (0x04, 0),
        (0x05, 1),
        (0x06, 2),
        (0x06, 3),
        (0x89, 4),
        (0x8B, 4),
        (0x8D, 4),
        (0x9F, 4),
        (0xA1, 4),
        (0x9D, 4),
    ],
];

/// Address of the string `"fieldmap_ctrl_grid.c"` (the file argument of the allocation in [`ctrl_create`]).
pub const FILE_CTRL_GRID: u32 = 0x021D_DE90;
/// Address of the string `"field_player.c"` (the file argument of the allocation in [`create`]).
pub const FILE_FIELD_PLAYER: u32 = 0x021D_DEA8;
/// The running shoes event flag ([`running_shoes`]).
pub const FLAG_RUNNING_SHOES: u32 = 0x963;

// ARM9 / other units (force stubs in this area's spec)
const GAMESYS_DATA: u32 = 0x0201_20F4; // GAMESYS -> GAMEDATA
const REC_SET_POS: u32 = 0x0201_2A90; // PLAYER record: position = *v
const REC_SET_RAIL: u32 = 0x0201_2AA4; // PLAYER record: rail location = *v
const REC_SET_KIND: u32 = 0x0201_2ABC; // PLAYER record: position kind (0 grid, 1 rail)
const REC_SET_DIR: u32 = 0x0201_2AD4; // PLAYER record: direction
const EVENT_WORK: u32 = 0x0201_296C; // GAMEDATA -> EVENT WORK
const CHECK_FLAG: u32 = 0x0201_42E8; // event_work check_flag (ew, flag)
const PAD_TRIGGER: u32 = 0x0203_62DC; // key input: pad trigger
const PAD_HELD: u32 = 0x0203_630C; // key input: pad held
const VEC_NORMALIZE: u32 = 0x0207_CCA4; // VEC_Normalize(&in, &out)
                                        // mmdl_move (F6a)
const DIR_DX: u32 = 0x0216_41E8;
const DIR_DZ: u32 = 0x0216_41F8;
const TILE_CENTRE: u32 = 0x0216_4280; // grid (x, z) -> VecFx32 tile centre (x, z, &pos)
                                      // fldmmdl (F4a)
const MMDL_SET_DIR: u32 = 0x0216_D5E0; // MMDL set display dir (m, dir)
const MMDL_DIR: u32 = 0x0216_D5F8; // MMDL display dir
const MMDL_GX: u32 = 0x0216_D83C;
const MMDL_GY: u32 = 0x0216_D854;
const MMDL_GZ: u32 = 0x0216_D86C;
const MMDL_POS_ADDR: u32 = 0x0216_D888; // address of the MMDL position VecFx32 (m + 0x44)
const MMDL_GET_POS: u32 = 0x0216_D88C; // MMDL position VecFx32 -> out
const MMDL_AT: u32 = 0x0216_DDC0; // MMDL at grid (sys, x, z, flag)
const MMDL_AT_HEIGHT: u32 = 0x0216_DDDC; // MMDL at grid + height (sys, x, z, y, height, flag)
                                         // fieldmap (F2a)
const FM_RAIL_MAP: u32 = 0x0218_8C5C; // fieldmap +0x3C (rail map)
const FM_MMDLSYS: u32 = 0x0218_8C7C; // fieldmap +0x38 (MMDLSYS)
const FM_HEAP: u32 = 0x0218_8C84; // fieldmap heap id
const FM_MAP: u32 = 0x0218_8C98; // fieldmap +0x48 (map draw object)
const FM_CTRL_WORK: u32 = 0x0218_8CA4; // fieldmap +0x10C (MAP CTRL work)
const FM_SET_CTRL_WORK: u32 = 0x0218_8CAC; // fieldmap +0x10C = v
const FM_PLAYER: u32 = 0x0218_8CB4; // fieldmap +0x8C (FIELD_PLAYER)
const FM_KIND: u32 = 0x0218_8CFC; // fieldmap map kind (0 grid, 1 rail)
const FM_SET_POS_SRC: u32 = 0x0218_8D40; // fieldmap +0xF0 = position source
                                         // rail system (F3b)
const RAIL_MMDL_AT: u32 = 0x0219_E570; // MMDL at a rail location (sys, &loc, 0)
const RAIL_FRONT_LOC: u32 = 0x0219_E634; // front location of an MMDL (m, &loc)
const RAIL_LOC: u32 = 0x0219_E2F4; // MMDL rail location -> out
const RAIL_LOC_IN_DIR: u32 = 0x0219_E664; // location in dir (m, dir, &loc)
const RAIL_DIR_VEC: u32 = 0x0219_E784; // direction vector of dir (m, dir, &VecFx16)
const RAIL_UNIT: u32 = 0x021B_7C38; // unit length (rail map)
const RAIL_LOC_POS: u32 = 0x021B_7D38; // location -> VecFx32 (rail map, &loc, &pos)
const RAIL_ATTR: u32 = 0x021B_B3C4; // attribute at a rail location (rail map, &loc)
const RAIL_MAP_OF: u32 = 0x021B_B480; // rail map of a map
                                      // map attributes (overlay 21)
const MAP_ATTR: u32 = 0x021A_B0D4; // map attribute at a position (map, &pos)
const ATTR_VALUE: u32 = 0x021A_B0F0; // attribute value of an attribute
const ATTR_P11C: u32 = 0x021A_B11C; // map attribute predicate (attribute)
const ATTR_P310: u32 = 0x021A_B310; // map attribute predicate (attribute value)
const ATTR_P5AC: u32 = 0x021A_B5AC; // attribute predicate (attr, value)
                                    // player_core (F3a)
const CORE_CREATE: u32 = 0x021A_36E4;
const CORE_DELETE_FREE: u32 = HEAP_FREE; // 021A3810 is a Heap_Free veneer
const CORE_UPDATE_STATE: u32 = 0x021A_3818;
const CORE_MMDL_POS: u32 = 0x021A_3898;
const CORE_WARP: u32 = 0x021A_38A4;
const CORE_DIR: u32 = 0x021A_3920;
const CORE_SET_DIR: u32 = 0x021A_392C;
const CORE_CHANGE_CODE: u32 = 0x021A_3938;
const CORE_CLEAR_CODE: u32 = 0x021A_394C;
const CORE_FIELDMAP: u32 = 0x021A_395C;
const CORE_MMDL: u32 = 0x021A_3960;
const CORE_MOVE_VALUE: u32 = 0x021A_3968;
const CORE_SET_STATE: u32 = 0x021A_396C;
const CORE_STATE: u32 = 0x021A_3970;
const CORE_FORM: u32 = 0x021A_3974;
const CORE_SET_FORM: u32 = 0x021A_3980;
const CORE_SEX: u32 = 0x021A_398C;
const CORE_GAMESYS: u32 = 0x021A_3990;
const CORE_RECORD: u32 = 0x021A_3994;
const CORE_PARAM_COUNTER: u32 = 0x021A_39E4;
const CORE_CODE_FORM: u32 = 0x021A_3A10;
const CORE_CODE_HAS_FORM: u32 = 0x021A_3A20;
const CORE_MMDL_BUSY: u32 = 0x021A_3A3C;
const CORE_KEY_DIR: u32 = 0x021A_3AB8;
const CORE_SET_SURF_TASK: u32 = 0x021A_3AD0;
const CORE_SURF_TASK: u32 = 0x021A_3AD4;
const CORE_SET_REQUESTS: u32 = 0x021A_3CB0;
const CORE_REQUESTS_DONE: u32 = 0x021A_3CC0;
const CORE_RUN_REQUESTS: u32 = 0x021A_3CE8;
// player_grid (F3a)
const GRID_CREATE: u32 = 0x021A_4140;
const GRID_FREE: u32 = HEAP_FREE; // 021A41A8 is a Heap_Free veneer
const GRID_FRAME: u32 = 0x021A_41B0;
const GRID_END_TURN: u32 = 0x021A_53D0;
const GRID_548C: u32 = 0x021A_548C;
const GRID_SLIDING: u32 = 0x021A_54A0;
const GRID_STOP_SLIDE: u32 = 0x021A_54B4;
const GRID_550C: u32 = 0x021A_550C; // reached through the veneer 021A575C
const GRID_5738: u32 = 0x021A_5738;
const GRID_ONE_SHOT: u32 = 0x021A_5764;
const GRID_SET_ONE_SHOT: u32 = 0x021A_5768;
const GRID_STATE_IS_3: u32 = 0x021A_5778;
// height_ex (F3a)
const STEP_EVENTS: u32 = 0x021C_0CA4;
// field_player_nogrid (F3b)
const NOGRID_CREATE: u32 = 0x021A_5880;
const NOGRID_DELETE: u32 = 0x021A_5920;
const NOGRID_5928: u32 = 0x021A_5928;
const NOGRID_5964: u32 = 0x021A_5964;
const NOGRID_596C: u32 = 0x021A_596C;
const NOGRID_59DC: u32 = 0x021A_59DC;
const NOGRID_59EC: u32 = 0x021A_59EC;
const NOGRID_5A00: u32 = 0x021A_5A00;
const NOGRID_5A0C: u32 = 0x021A_5A0C;
const NOGRID_5A18: u32 = 0x021A_5A18;
const NOGRID_5A24: u32 = 0x021A_5A24;
const NOGRID_5A30: u32 = 0x021A_5A30;
const NOGRID_5A34: u32 = 0x021A_5A34;

fn c0<E: FieldEnv>(env: &mut E, addr: u32) -> u32 {
    env.callv(addr, &[])
}

fn c1<E: FieldEnv>(env: &mut E, addr: u32, a: u32) -> u32 {
    env.callv(addr, &[a])
}

fn c2<E: FieldEnv>(env: &mut E, addr: u32, a: u32, b: u32) -> u32 {
    env.callv(addr, &[a, b])
}

fn core<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    env.r32(player.wrapping_add(4))
}

fn grid<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    env.r32(player.wrapping_add(8))
}

fn nogrid<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    env.r32(player.wrapping_add(0xC))
}

fn kind<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let fm = env.r32(player);
    c1(env, FM_KIND, fm)
}

/// `ldrsh` of a halfword of a local byte array.
fn sh(b: &[u8], o: usize) -> u32 {
    i16::from_le_bytes([b[o], b[o + 1]]) as i32 as u32
}

fn word(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

/// C: `FUN_021A2D88` (unlabeled routine at `0x021A2D88`): the grid MAP CTRL create callback (`fieldmap`, unused,
/// `dir`): allocates the 0x10-byte work on the fieldmap heap (`+0 u16` heap, `+4` fieldmap), sets the player MMDL's
/// display dir, creates the grid player ([`create_grid`]), makes the MMDL position the fieldmap's position source
/// (`FUN_02188D40`) and stores the work in the fieldmap (`FUN_02188CAC`).
pub fn ctrl_create<E: FieldEnv>(env: &mut E, fm: u32, _unused: u32, dir: u32) {
    let heap = c1(env, FM_HEAP, fm);
    let w = env.callv(HEAP_ALLOC, &[heap, 0x10, 1, FILE_CTRL_GRID, 0x68]);
    env.w16(w, heap as u16);
    env.w32(w.wrapping_add(4), fm);
    let p = c1(env, FM_PLAYER, fm);
    let m = mmdl(env, p);
    c2(env, MMDL_SET_DIR, m, dir);
    create_grid(env, p, heap);
    let pos = c1(env, MMDL_POS_ADDR, m);
    c2(env, FM_SET_POS_SRC, fm, pos);
    c2(env, FM_SET_CTRL_WORK, fm, w);
}

/// C: `FUN_021A2DE4` (unlabeled routine at `0x021A2DE4`): the grid MAP CTRL delete callback: frees the work.
pub fn ctrl_delete<E: FieldEnv>(env: &mut E, fm: u32) {
    let w = c1(env, FM_CTRL_WORK, fm);
    c1(env, HEAP_FREE, w);
}

/// C: `FUN_021A2DF0` (unlabeled routine at `0x021A2DF0`): the grid MAP CTRL frame callback: unless the work's hold
/// word `+0xC` is set, reads the pad (trigger `FUN_020362DC`, held `FUN_0203630C`) and runs the grid player frame
/// ([`grid_frame`]) with flags bit 0 = the running shoes ([`running_shoes`] == 1).
pub fn ctrl_frame<E: FieldEnv>(env: &mut E, fm: u32) {
    let w = c1(env, FM_CTRL_WORK, fm);
    let p = c1(env, FM_PLAYER, fm);
    if env.r32(w.wrapping_add(0xC)) != 0 {
        return;
    }
    let t = c0(env, PAD_TRIGGER);
    let h = c0(env, PAD_HELD);
    let mut flags = 0;
    if running_shoes(env, p) == 1 {
        flags |= 1;
    }
    grid_frame(env, p, t, h, flags);
}

/// C: `FUN_021A2E34` (unlabeled routine at `0x021A2E34`): the grid MAP CTRL position callback: the address of the
/// player MMDL's position (`FUN_0216D888`).
pub fn ctrl_pos<E: FieldEnv>(env: &mut E, fm: u32) -> u32 {
    let p = c1(env, FM_PLAYER, fm);
    let m = mmdl(env, p);
    c1(env, MMDL_POS_ADDR, m)
}

/// C: `FUN_021A2E44` (unlabeled routine at `0x021A2E44`): set the MAP CTRL work's hold word `+0xC` (nonzero: no pad
/// input reaches the player).
pub fn ctrl_hold<E: FieldEnv>(env: &mut E, fm: u32, v: u32) {
    let w = c1(env, FM_CTRL_WORK, fm);
    env.w32(w.wrapping_add(0xC), v);
}

/// C: `FUN_021A2E50` (unlabeled routine at `0x021A2E50`): create the FIELD_PLAYER (`x`, `fieldmap`, `&pos`, `sex`,
/// `heap` (a u16 stack argument)): allocates 0x10 bytes, `+0` = fieldmap, `+4` = the core (`FUN_021A36E4` with the
/// same arguments). The grid / no-grid halves are created later ([`create_grid`], [`create_nogrid`]).
pub fn create<E: FieldEnv>(env: &mut E, x: u32, fm: u32, pos: u32, sex: u32, heap: u32) -> u32 {
    let heap = heap & 0xFFFF;
    let p = env.callv(HEAP_ALLOC, &[heap, 0x10, 1, FILE_FIELD_PLAYER, 0x4C]);
    env.w32(p, fm);
    let c = env.call(
        CORE_CREATE,
        &mut [
            CArg::V(x),
            CArg::V(fm),
            CArg::In(pos, 12),
            CArg::V(sex),
            CArg::V(heap),
        ],
    );
    env.w32(p.wrapping_add(4), c);
    p
}

/// C: `FUN_021A2E90` (unlabeled routine at `0x021A2E90`): delete the FIELD_PLAYER: the grid player (`+8`, freed by
/// the Heap_Free veneer `FUN_021A41A8`) and the no-grid player (`+0xC`, `FUN_021A5920`) when present, the core
/// (Heap_Free veneer `FUN_021A3810`), then the block itself.
pub fn delete<E: FieldEnv>(env: &mut E, player: u32) {
    let g = grid(env, player);
    if g != 0 {
        c1(env, GRID_FREE, g);
    }
    let n = nogrid(env, player);
    if n != 0 {
        c1(env, NOGRID_DELETE, n);
    }
    let c = core(env, player);
    c1(env, CORE_DELETE_FREE, c);
    c1(env, HEAP_FREE, player);
}

/// C: `FUN_021A2EB8` (unlabeled routine at `0x021A2EB8`): save the player into its PLAYER record (`FUN_021A3994`):
/// the MMDL position and display dir; on a grid map (kind 0) position kind 0, else the MMDL's rail location and
/// position kind 1.
pub fn save<E: FieldEnv>(env: &mut E, player: u32) {
    let c = core(env, player);
    let rec = c1(env, CORE_RECORD, c);
    let c = core(env, player);
    let m = c1(env, CORE_MMDL, c);
    let mut pos = [0u8; 12];
    env.call(MMDL_GET_POS, &mut [CArg::V(m), CArg::OutBuf(&mut pos)]);
    env.call(REC_SET_POS, &mut [CArg::V(rec), CArg::InBuf(&pos)]);
    let d = c1(env, MMDL_DIR, m);
    c2(env, REC_SET_DIR, rec, d);
    if kind(env, player) == 0 {
        c2(env, REC_SET_KIND, rec, 0);
        return;
    }
    let mut loc = [0u8; 8];
    env.call(RAIL_LOC, &mut [CArg::V(m), CArg::OutBuf(&mut loc)]);
    env.call(REC_SET_RAIL, &mut [CArg::V(rec), CArg::InBuf(&loc)]);
    c2(env, REC_SET_KIND, rec, 1);
}

/// C: `FUN_021A2F20` (unlabeled routine at `0x021A2F20`): the step events of the grid player (`FUN_021C0CA4(+8, a,
/// keys, flags)`) on a grid map, else 0.
pub fn step_events<E: FieldEnv>(env: &mut E, player: u32, a: u32, keys: u32, flags: u32) -> u32 {
    if kind(env, player) != 0 {
        return 0;
    }
    let g = grid(env, player);
    env.callv(STEP_EVENTS, &[g, a, keys, flags])
}

/// C: `FUN_021A2F48` (unlabeled routine at `0x021A2F48`): tail call `FUN_021A3CB0(core, bits)` (start a request set).
pub fn set_requests<E: FieldEnv>(env: &mut E, player: u32, bits: u32) {
    let c = core(env, player);
    c2(env, CORE_SET_REQUESTS, c, bits);
}

/// C: `FUN_021A2F54` (unlabeled routine at `0x021A2F54`): tail call `FUN_021A3CC0(core)` (requests done?).
pub fn requests_done<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_REQUESTS_DONE, c)
}

/// C: `FUN_021A2F60` (unlabeled routine at `0x021A2F60`): tail call `FUN_021A3CE8(core)` (run the requests).
pub fn run_requests<E: FieldEnv>(env: &mut E, player: u32) {
    let c = core(env, player);
    c1(env, CORE_RUN_REQUESTS, c);
}

/// C: `FUN_021A2F6C` (unlabeled routine at `0x021A2F6C`): grid map: `FUN_021A53D0(grid)` (end a turn); else
/// `FUN_021A5A34(no-grid player)`.
pub fn end_turn<E: FieldEnv>(env: &mut E, player: u32) {
    if kind(env, player) == 0 {
        let g = grid(env, player);
        c1(env, GRID_END_TURN, g);
    } else {
        let n = nogrid(env, player);
        c1(env, NOGRID_5A34, n);
    }
}

/// C: `FUN_021A2F8C` (unlabeled routine at `0x021A2F8C`): tail call `FUN_021A3AD0(core, task)` (set the surf effect
/// task).
pub fn set_surf_task<E: FieldEnv>(env: &mut E, player: u32, task: u32) {
    let c = core(env, player);
    c2(env, CORE_SET_SURF_TASK, c, task);
}

/// C: `FUN_021A2F98` (unlabeled routine at `0x021A2F98`): tail call `FUN_021A3AD4(core)` (the surf effect task).
pub fn surf_task<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_SURF_TASK, c)
}

/// C: `FUN_021A2FA4` (unlabeled routine at `0x021A2FA4`): tail call `FUN_021A3AB8(core, keys)` (the direction of the
/// pad keys).
pub fn key_dir<E: FieldEnv>(env: &mut E, player: u32, keys: u32) -> u32 {
    let c = core(env, player);
    c2(env, CORE_KEY_DIR, c, keys)
}

/// C: `FUN_021A2FB0` (unlabeled routine at `0x021A2FB0`): grid map: `FUN_021A5778(grid)` (move state 3); else
/// `FUN_021A59DC(no-grid player)`; returns the callee's answer (the spec does not compare it).
pub fn state_check<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    if kind(env, player) == 0 {
        let g = grid(env, player);
        c1(env, GRID_STATE_IS_3, g)
    } else {
        let n = nogrid(env, player);
        c1(env, NOGRID_59DC, n)
    }
}

/// C: `FUN_021A2FD0` (unlabeled routine at `0x021A2FD0`): request the walk form (`FUN_021A3CB0(core, 1)`) and run the
/// request (`FUN_021A3CE8`).
pub fn request_walk<E: FieldEnv>(env: &mut E, player: u32) {
    let c = core(env, player);
    c2(env, CORE_SET_REQUESTS, c, 1);
    let c = core(env, player);
    c1(env, CORE_RUN_REQUESTS, c);
}

/// C: `FUN_021A2FE4` (unlabeled routine at `0x021A2FE4`): update the core's move state: grid map, `FUN_021A548C(grid)
/// == 1` or `FUN_021A54A0(grid) == 1` (sliding); no-grid map, `FUN_021A59EC(no-grid) == 1`: `FUN_021A396C(core, 2)`;
/// otherwise `FUN_021A3818(core)`.
pub fn update_move_state<E: FieldEnv>(env: &mut E, player: u32) {
    let two = if kind(env, player) == 0 {
        let g = grid(env, player);
        c1(env, GRID_548C, g) == 1 || {
            let g = grid(env, player);
            c1(env, GRID_SLIDING, g) == 1
        }
    } else {
        let n = nogrid(env, player);
        c1(env, NOGRID_59EC, n) == 1
    };
    let c = core(env, player);
    if two {
        c2(env, CORE_SET_STATE, c, 2);
    } else {
        c1(env, CORE_UPDATE_STATE, c);
    }
}

/// C: `FUN_021A3038` (unlabeled routine at `0x021A3038`): tail call `FUN_021A3898(core, out)` (the MMDL position ->
/// the 12 bytes at `out`).
pub fn mmdl_pos<E: FieldEnv>(env: &mut E, player: u32, out: u32) {
    let c = core(env, player);
    env.call(CORE_MMDL_POS, &mut [CArg::V(c), CArg::Out(out, 12)]);
}

/// [`mmdl_pos`] into a local of the caller.
fn mmdl_pos_local<E: FieldEnv>(env: &mut E, player: u32, out: &mut [u8; 12]) {
    let c = core(env, player);
    env.call(CORE_MMDL_POS, &mut [CArg::V(c), CArg::OutBuf(out)]);
}

/// C: `FUN_021A3044` (unlabeled routine at `0x021A3044`): tail call `FUN_021A38A4(core, pos)` (warp the player to the
/// position at `pos`).
pub fn warp_to<E: FieldEnv>(env: &mut E, player: u32, pos: u32) {
    let c = core(env, player);
    env.call(CORE_WARP, &mut [CArg::V(c), CArg::In(pos, 12)]);
}

/// C: `FUN_021A3050` (unlabeled routine at `0x021A3050`): tail call `FUN_021A3920(core)` (the MMDL display dir).
pub fn dir<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_DIR, c)
}

/// C: `FUN_021A305C` (unlabeled routine at `0x021A305C`): tail call `FUN_021A392C(core, dir)` (set the MMDL display
/// dir).
pub fn set_dir<E: FieldEnv>(env: &mut E, player: u32, d: u32) {
    let c = core(env, player);
    c2(env, CORE_SET_DIR, c, d);
}

/// C: `FUN_021A3068` (unlabeled routine at `0x021A3068`): tail call `FUN_021A395C(core)` (the fieldmap).
pub fn fieldmap<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_FIELDMAP, c)
}

/// C: `FUN_021A3074` (unlabeled routine at `0x021A3074`): tail call `FUN_021A3960(core)` (the player MMDL).
pub fn mmdl<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_MMDL, c)
}

/// C: `FUN_021A3080` (unlabeled routine at `0x021A3080`): tail call `FUN_021A3968(core)` (the move value).
pub fn move_value<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_MOVE_VALUE, c)
}

/// C: `FUN_021A308C` (unlabeled routine at `0x021A308C`): tail call `FUN_021A3970(core)` (the move state).
pub fn move_state<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_STATE, c)
}

/// C: `FUN_021A3098` (unlabeled routine at `0x021A3098`): tail call `FUN_021A3974(core)` (the move form).
pub fn form<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_FORM, c)
}

/// C: `FUN_021A30A4` (unlabeled routine at `0x021A30A4`): tail call `FUN_021A3980(core, form)` (set the move form).
pub fn set_form<E: FieldEnv>(env: &mut E, player: u32, f: u32) {
    let c = core(env, player);
    c2(env, CORE_SET_FORM, c, f);
}

/// C: `FUN_021A30B0` (unlabeled routine at `0x021A30B0`): tail call `FUN_021A398C(core)` (the sex).
pub fn sex<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_SEX, c)
}

/// C: `FUN_021A30BC` (unlabeled routine at `0x021A30BC`): tail call `FUN_021A3990(core)` (the GAMESYS).
pub fn gamesys<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_GAMESYS, c)
}

/// C: `FUN_021A30C8` (unlabeled routine at `0x021A30C8`): the map attribute under the player: grid map (kind 0):
/// `FUN_021AB0D4(map, &pos)` with the MMDL position ([`mmdl_pos`]); rail map (kind 1): `FUN_021BB3C4(rail map,
/// &loc)` with the MMDL's rail location. For any other kind the original returns its caller's r4 unchanged; the port
/// takes that register as `stale` (0 in every harness run: the map-kind stub answers 0 / 1).
pub fn attr_under<E: FieldEnv>(env: &mut E, player: u32, stale: u32) -> u32 {
    let fm = env.r32(player);
    if c1(env, FM_KIND, fm) == 0 {
        let map = c1(env, FM_MAP, fm);
        let mut pos = [0u8; 12];
        mmdl_pos_local(env, player, &mut pos);
        return env.call(MAP_ATTR, &mut [CArg::V(map), CArg::InBuf(&pos)]);
    }
    if c1(env, FM_KIND, fm) != 1 {
        return stale;
    }
    let m = mmdl(env, player);
    let fm = env.r32(player);
    let rail = c1(env, FM_RAIL_MAP, fm);
    let mut loc = [0u8; 8];
    env.call(RAIL_LOC, &mut [CArg::V(m), CArg::OutBuf(&mut loc)]);
    env.call(RAIL_ATTR, &mut [CArg::V(rail), CArg::InBuf(&loc)])
}

/// C: `FUN_021A3128` (unlabeled routine at `0x021A3128`): the map attribute in direction `dir` of the player: grid
/// map: `FUN_021AB0D4(map, &pos)` with the position in front ([`pos_in_dir`]); rail map: `FUN_021BB3C4(rail map,
/// &loc)` with the rail location in that direction (`FUN_0219E664`). For any other kind the original returns its
/// caller's r6 unchanged (`stale`, as in [`attr_under`]).
pub fn attr_front<E: FieldEnv>(env: &mut E, player: u32, d: u32, stale: u32) -> u32 {
    let fm = env.r32(player);
    if c1(env, FM_KIND, fm) == 0 {
        let map = c1(env, FM_MAP, fm);
        let pos = env.mem().push_local(12);
        pos_in_dir(env, player, d, pos);
        let a = env.call(MAP_ATTR, &mut [CArg::V(map), CArg::In(pos, 12)]);
        env.mem().pop_local(pos);
        return a;
    }
    if c1(env, FM_KIND, fm) != 1 {
        return stale;
    }
    let m = mmdl(env, player);
    let fm = env.r32(player);
    let rail = c1(env, FM_RAIL_MAP, fm);
    let mut loc = [0u8; 8];
    env.call(
        RAIL_LOC_IN_DIR,
        &mut [CArg::V(m), CArg::V(d), CArg::OutBuf(&mut loc)],
    );
    env.call(RAIL_ATTR, &mut [CArg::V(rail), CArg::InBuf(&loc)])
}

/// C: `FUN_021A3190` (unlabeled routine at `0x021A3190`): the OBJ code of entry `i` of row `sex` of the form table.
/// The original does not check the indices (it reads past the 0x50-byte table); the port panics outside
/// `sex < 2`, `i < 10` (UNVERIFIED).
pub fn entry_code(sex: u32, i: u32) -> u16 {
    FORM_TABLE[sex as usize][i as usize].0
}

/// C: `FUN_021A31A4` (unlabeled routine at `0x021A31A4`): the OBJ code of the first entry of row `sex` whose form is
/// `form` (the whole register is compared with the u16 form), 1 when there is none.
pub fn form_code(sex: u32, form: u32) -> u16 {
    FORM_TABLE[sex as usize]
        .iter()
        .find(|e| e.1 as u32 == form)
        .map_or(1, |e| e.0)
}

/// C: `FUN_021A31D0` (unlabeled routine at `0x021A31D0`): the entry index (0..9) of OBJ code `code`, searching row 0
/// then row 1 (the whole register is compared with the u16 code); 10 when the code is not a player code.
pub fn code_entry(code: u32) -> u32 {
    for row in &FORM_TABLE {
        if let Some(i) = row.iter().position(|e| e.0 as u32 == code) {
            return i as u32;
        }
    }
    10
}

/// C: `FUN_021A3208` (unlabeled routine at `0x021A3208`): 1 unless the move form ([`form`]) is 2 or 3 (surfing).
pub fn not_surfing<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let f = form(env, player);
    if f == 2 || f == 3 {
        0
    } else {
        1
    }
}

/// C: `FUN_021A3220` (unlabeled routine at `0x021A3220`): tail call `FUN_021A3938(core, code)` (change and save the
/// OBJ code).
pub fn change_code<E: FieldEnv>(env: &mut E, player: u32, code: u32) {
    let c = core(env, player);
    c2(env, CORE_CHANGE_CODE, c, code);
}

/// C: `FUN_021A322C` (unlabeled routine at `0x021A322C`): tail call `FUN_021A394C(core)` (clear the saved OBJ code).
pub fn clear_saved_code<E: FieldEnv>(env: &mut E, player: u32) {
    let c = core(env, player);
    c1(env, CORE_CLEAR_CODE, c);
}

/// C: `FUN_021A3238` (unlabeled routine at `0x021A3238`): tail call `FUN_021A3A20(core)` (the OBJ code has a form).
pub fn code_has_form<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_CODE_HAS_FORM, c)
}

/// C: `FUN_021A3244` (unlabeled routine at `0x021A3244`): tail call `FUN_021A3A3C(core)` (the MMDL is busy).
pub fn mmdl_busy<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_MMDL_BUSY, c)
}

/// C: `FUN_021A3250` (unlabeled routine at `0x021A3250`): the MMDL in front of the player: grid map: the front tile
/// ([`front_tile`]) -> `FUN_0216DDC0(MMDLSYS, x, z, 0)` (s16 coordinates); rail map: the front rail location
/// (`FUN_0219E634`) -> `FUN_0219E570(MMDLSYS, &loc, 0)`.
pub fn mmdl_in_front<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let fm = env.r32(player);
    let sys = c1(env, FM_MMDLSYS, fm);
    if kind(env, player) == 0 {
        let l = env.mem().push_local(8);
        front_tile(env, player, l.wrapping_add(4), l.wrapping_add(2), l);
        let x = env.r16(l.wrapping_add(4)) as i16 as i32 as u32;
        let z = env.r16(l) as i16 as i32 as u32;
        env.mem().pop_local(l);
        return env.callv(MMDL_AT, &[sys, x, z, 0]);
    }
    let c = core(env, player);
    let m = c1(env, CORE_MMDL, c);
    let mut loc = [0u8; 8];
    env.call(RAIL_FRONT_LOC, &mut [CArg::V(m), CArg::OutBuf(&mut loc)]);
    env.call(
        RAIL_MMDL_AT,
        &mut [CArg::V(sys), CArg::InBuf(&loc), CArg::V(0)],
    )
}

/// C: `FUN_021A32A8` (unlabeled routine at `0x021A32A8`): the MMDL in front of the player at height `h`: grid map:
/// the front tile -> `FUN_0216DDDC(MMDLSYS, x, z, y << 16, h, 0)`; rail map: as [`mmdl_in_front`].
pub fn mmdl_in_front_at<E: FieldEnv>(env: &mut E, player: u32, h: u32) -> u32 {
    let fm = env.r32(player);
    let sys = c1(env, FM_MMDLSYS, fm);
    if kind(env, player) == 0 {
        let l = env.mem().push_local(8);
        front_tile(env, player, l.wrapping_add(4), l.wrapping_add(2), l);
        let y = env.r16(l.wrapping_add(2)) as i16 as i32 as u32;
        let x = env.r16(l.wrapping_add(4)) as i16 as i32 as u32;
        let z = env.r16(l) as i16 as i32 as u32;
        env.mem().pop_local(l);
        return env.callv(MMDL_AT_HEIGHT, &[sys, x, z, y << 16, h, 0]);
    }
    let c = core(env, player);
    let m = c1(env, CORE_MMDL, c);
    let mut loc = [0u8; 8];
    env.call(RAIL_FRONT_LOC, &mut [CArg::V(m), CArg::OutBuf(&mut loc)]);
    env.call(
        RAIL_MMDL_AT,
        &mut [CArg::V(sys), CArg::InBuf(&loc), CArg::V(0)],
    )
}

/// C: `FUN_021A330C` (unlabeled routine at `0x021A330C`): the grid tile in direction `dir` of the player (`dir`,
/// `&x`, `&y`, `&z` (a stack argument)): on a grid map the u16 MMDL grid position, then `x += dx(dir)`, `z += dz(dir)`
/// (`FUN_021641E8` / `FUN_021641F8`, on the s16 values); on other maps nothing is written.
pub fn tile_in_dir<E: FieldEnv>(env: &mut E, player: u32, d: u32, px: u32, py: u32, pz: u32) {
    let m = mmdl(env, player);
    if kind(env, player) != 0 {
        return;
    }
    let gx = c1(env, MMDL_GX, m);
    env.w16(px, gx as u16);
    let gy = c1(env, MMDL_GY, m);
    env.w16(py, gy as u16);
    let gz = c1(env, MMDL_GZ, m);
    env.w16(pz, gz as u16);
    let dx = c1(env, DIR_DX, d);
    let x = env.r16(px) as i16 as i32 as u32;
    env.w16(px, x.wrapping_add(dx) as u16);
    let dz = c1(env, DIR_DZ, d);
    let z = env.r16(pz) as i16 as i32 as u32;
    env.w16(pz, z.wrapping_add(dz) as u16);
}

/// `(v * u + 0x800) >> 12` on 64-bit products, low word (the rail offset of [`pos_in_dir`]).
fn fx_mul_round(v: u32, u: u32) -> u32 {
    let p = (v as i32 as i64).wrapping_mul(u as i32 as i64) as u64;
    (p.wrapping_add(0x800) >> 12) as u32
}

/// C: `FUN_021A3364` (unlabeled routine at `0x021A3364`): the position (VecFx32 at `out`) in direction `dir` of the
/// player: grid map: the tile ([`tile_in_dir`]) -> its centre (`FUN_02164280(x, z, out)`), `out.y = y << 16`; rail
/// map: the MMDL's rail location -> position `p` (`FUN_021B7D38`), the direction vector `d` (VecFx16,
/// `FUN_0219E784`) and the unit length `u` (`FUN_021B7C38`): `out = p + (d * u + 0x800) >> 12` per component.
pub fn pos_in_dir<E: FieldEnv>(env: &mut E, player: u32, d: u32, out: u32) {
    let c = core(env, player);
    let m = c1(env, CORE_MMDL, c);
    if kind(env, player) == 0 {
        let l = env.mem().push_local(8);
        // locals: z at +0, y at +2, x at +4 (sp+4 / sp+6 / sp+8 of the original)
        tile_in_dir(env, player, d, l.wrapping_add(4), l.wrapping_add(2), l);
        let x = env.r16(l.wrapping_add(4)) as i16 as i32 as u32;
        let z = env.r16(l) as i16 as i32 as u32;
        env.call(
            TILE_CENTRE,
            &mut [CArg::V(x), CArg::V(z), CArg::Out(out, 12)],
        );
        let y = env.r16(l.wrapping_add(2)) as i16 as i32 as u32;
        env.mem().pop_local(l);
        env.w32(out.wrapping_add(4), y << 16);
        return;
    }
    let fm = env.r32(player);
    let r = c1(env, FM_RAIL_MAP, fm);
    let rail = c1(env, RAIL_MAP_OF, r);
    let mut loc = [0u8; 8];
    env.call(RAIL_LOC, &mut [CArg::V(m), CArg::OutBuf(&mut loc)]);
    let mut p = [0u8; 12];
    env.call(
        RAIL_LOC_POS,
        &mut [CArg::V(rail), CArg::InBuf(&loc), CArg::OutBuf(&mut p)],
    );
    let mut v = [0u8; 6];
    env.call(
        RAIL_DIR_VEC,
        &mut [CArg::V(m), CArg::V(d), CArg::OutBuf(&mut v)],
    );
    let u = c1(env, RAIL_UNIT, rail);
    for k in 0..3 {
        let o = fx_mul_round(sh(&v, 2 * k), u);
        env.w32(
            out.wrapping_add(4 * k as u32),
            word(&p, 4 * k).wrapping_add(o),
        );
    }
}

/// C: `FUN_021A3460` (unlabeled routine at `0x021A3460`): the tile in front of the player: [`tile_in_dir`] with the
/// MMDL display dir (`FUN_0216D5F8`).
pub fn front_tile<E: FieldEnv>(env: &mut E, player: u32, px: u32, py: u32, pz: u32) {
    let m = mmdl(env, player);
    let d = c1(env, MMDL_DIR, m);
    tile_in_dir(env, player, d, px, py, pz);
}

/// C: `FUN_021A3484` (unlabeled routine at `0x021A3484`): the unit direction vector (VecFx32 at `out`) of `dir`: grid
/// map: `(dx << 16, 0, dz << 16)` normalised in place (`FUN_0207CCA4`); rail map: the s16 vector of
/// `FUN_0219E784(MMDL, dir)`, sign-extended.
pub fn dir_vector<E: FieldEnv>(env: &mut E, player: u32, d: u32, out: u32) {
    let c = core(env, player);
    let m = c1(env, CORE_MMDL, c);
    if kind(env, player) == 0 {
        env.w32(out.wrapping_add(4), 0);
        let dx = c1(env, DIR_DX, d);
        env.w32(out, dx);
        let dz = c1(env, DIR_DZ, d);
        let x = env.r32(out);
        env.w32(out, x << 16);
        env.w32(out.wrapping_add(8), dz << 16);
        env.call(VEC_NORMALIZE, &mut [CArg::In(out, 12), CArg::Out(out, 12)]);
        return;
    }
    let mut v = [0u8; 6];
    env.call(
        RAIL_DIR_VEC,
        &mut [CArg::V(m), CArg::V(d), CArg::OutBuf(&mut v)],
    );
    env.w32(out, sh(&v, 0));
    env.w32(out.wrapping_add(4), sh(&v, 2));
    env.w32(out.wrapping_add(8), sh(&v, 4));
}

/// C: `FUN_021A34EC` (unlabeled routine at `0x021A34EC`): tail call `FUN_021A3A10(core)` (the form of the OBJ code).
pub fn code_form<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_CODE_FORM, c)
}

/// C: `FUN_021A34F8` (unlabeled routine at `0x021A34F8`): tail call `FUN_021A39E4(core)` (the param counter).
pub fn param_counter<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let c = core(env, player);
    c1(env, CORE_PARAM_COUNTER, c)
}

/// C: `FUN_021A3504` (unlabeled routine at `0x021A3504`): 0 while surfing (form 2 / 3), else the attribute predicate
/// `FUN_021AB5AC(attr, v)`.
pub fn attr_check<E: FieldEnv>(env: &mut E, player: u32, attr: u32, v: u32) -> u32 {
    if form(env, player).wrapping_sub(2) <= 1 {
        return 0;
    }
    c2(env, ATTR_P5AC, attr, v)
}

/// C: `FUN_021A3524` (unlabeled routine at `0x021A3524`): with `FUN_021AB11C(attr) == 0` and the surf form (2):
/// `FUN_021AB310(FUN_021AB0F0(value))`; else 0.
pub fn surf_attr_check<E: FieldEnv>(env: &mut E, player: u32, attr: u32, value: u32) -> u32 {
    let f = form(env, player);
    if c1(env, ATTR_P11C, attr) != 0 || f != 2 {
        return 0;
    }
    let a = c1(env, ATTR_VALUE, value);
    c1(env, ATTR_P310, a)
}

/// C: `FUN_021A3550` (unlabeled routine at `0x021A3550`): grid map: `FUN_021A5738(grid, keep)`.
pub fn grid_5738<E: FieldEnv>(env: &mut E, player: u32, keep: u32) {
    if kind(env, player) == 0 {
        let g = grid(env, player);
        c2(env, GRID_5738, g, keep);
    }
}

/// C: `FUN_021A356C` (unlabeled routine at `0x021A356C`): grid map: `FUN_021A550C(grid)` (through the veneer
/// `FUN_021A575C`).
pub fn grid_550c<E: FieldEnv>(env: &mut E, player: u32) {
    if kind(env, player) == 0 {
        let g = grid(env, player);
        c1(env, GRID_550C, g);
    }
}

/// C: `FUN_021A3584` (unlabeled routine at `0x021A3584`): grid map: the grid player's one-shot flag
/// (`FUN_021A5764`); else 0.
pub fn one_shot<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    if kind(env, player) != 0 {
        return 0;
    }
    let g = grid(env, player);
    c1(env, GRID_ONE_SHOT, g)
}

/// C: `FUN_021A35A0` (unlabeled routine at `0x021A35A0`): grid map: set the grid player's one-shot flag
/// (`FUN_021A5768(grid, on)`).
pub fn set_one_shot<E: FieldEnv>(env: &mut E, player: u32, on: u32) {
    if kind(env, player) == 0 {
        let g = grid(env, player);
        c2(env, GRID_SET_ONE_SHOT, g, on);
    }
}

/// C: `FUN_021A35BC` (unlabeled routine at `0x021A35BC`): 1 when the running shoes event flag 0x963 is set (in the
/// EVENT WORK of the GAMEDATA of the GAMESYS), else 0.
pub fn running_shoes<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let gs = gamesys(env, player);
    let gd = c1(env, GAMESYS_DATA, gs);
    let ew = c1(env, EVENT_WORK, gd);
    if c2(env, CHECK_FLAG, ew, FLAG_RUNNING_SHOES) != 0 {
        1
    } else {
        0
    }
}

/// C: `FUN_021A35E0` (unlabeled routine at `0x021A35E0`): create the grid player: `+8 = FUN_021A4140(core, heap)`.
pub fn create_grid<E: FieldEnv>(env: &mut E, player: u32, heap: u32) {
    let c = core(env, player);
    let g = c2(env, GRID_CREATE, c, heap);
    env.w32(player.wrapping_add(8), g);
}

/// C: `FUN_021A35F0` (unlabeled routine at `0x021A35F0`): the grid player frame `FUN_021A41B0(grid, trigger, held,
/// flags)`.
pub fn grid_frame<E: FieldEnv>(env: &mut E, player: u32, t: u32, h: u32, flags: u32) {
    let g = grid(env, player);
    env.callv(GRID_FRAME, &[g, t, h, flags]);
}

/// C: `FUN_021A35FC` (unlabeled routine at `0x021A35FC`): the MMDL grid position as u16 `x`, `y`, `z` at `px`,
/// `py`, `pz`.
pub fn grid_pos<E: FieldEnv>(env: &mut E, player: u32, px: u32, py: u32, pz: u32) {
    let c = core(env, player);
    let m = c1(env, CORE_MMDL, c);
    let x = c1(env, MMDL_GX, m);
    env.w16(px, x as u16);
    let y = c1(env, MMDL_GY, m);
    env.w16(py, y as u16);
    let z = c1(env, MMDL_GZ, m);
    env.w16(pz, z as u16);
}

/// C: `FUN_021A3624` (unlabeled routine at `0x021A3624`): 1 on a grid map whose grid player is sliding
/// (`FUN_021A54A0 == 1`), else 0.
pub fn sliding<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    if kind(env, player) == 0 {
        let g = grid(env, player);
        if c1(env, GRID_SLIDING, g) == 1 {
            return 1;
        }
    }
    0
}

/// C: `FUN_021A3644` (unlabeled routine at `0x021A3644`): on a grid map whose grid player is sliding, stop sliding
/// (`FUN_021A54B4(grid)`).
pub fn stop_sliding<E: FieldEnv>(env: &mut E, player: u32) {
    if kind(env, player) == 0 {
        let g = grid(env, player);
        if c1(env, GRID_SLIDING, g) == 1 {
            let g = grid(env, player);
            c1(env, GRID_STOP_SLIDE, g);
        }
    }
}

/// C: `FUN_021A3664` (unlabeled routine at `0x021A3664`): create the no-grid player: `+0xC = FUN_021A5880(core)`.
pub fn create_nogrid<E: FieldEnv>(env: &mut E, player: u32) {
    let c = core(env, player);
    let n = c1(env, NOGRID_CREATE, c);
    env.w32(player.wrapping_add(0xC), n);
}

/// C: `FUN_021A3674` (unlabeled routine at `0x021A3674`): tail call `FUN_021A596C(no-grid player, a, b)` (F3b).
pub fn nogrid_596c<E: FieldEnv>(env: &mut E, player: u32, a: u32, b: u32) {
    let n = nogrid(env, player);
    env.callv(NOGRID_596C, &[n, a, b]);
}

/// C: `FUN_021A3680` (unlabeled routine at `0x021A3680`): tail call `FUN_021A5928(no-grid player, a)` (F3b).
pub fn nogrid_5928<E: FieldEnv>(env: &mut E, player: u32, a: u32) -> u32 {
    let n = nogrid(env, player);
    c2(env, NOGRID_5928, n, a)
}

/// C: `FUN_021A368C` (unlabeled routine at `0x021A368C`): tail call `FUN_021A5964(no-grid player)` (F3b).
pub fn nogrid_5964<E: FieldEnv>(env: &mut E, player: u32) {
    let n = nogrid(env, player);
    c1(env, NOGRID_5964, n);
}

/// C: `FUN_021A3698` (unlabeled routine at `0x021A3698`): tail call `FUN_021A59EC(no-grid player)` (F3b).
pub fn nogrid_59ec<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let n = nogrid(env, player);
    c1(env, NOGRID_59EC, n)
}

/// C: `FUN_021A36A4` (unlabeled routine at `0x021A36A4`): tail call `FUN_021A5A30(no-grid player)` (F3b).
pub fn nogrid_5a30<E: FieldEnv>(env: &mut E, player: u32) -> u32 {
    let n = nogrid(env, player);
    c1(env, NOGRID_5A30, n)
}

/// C: `FUN_021A36B0` (unlabeled routine at `0x021A36B0`): tail call `FUN_021A5A00(no-grid player, a)` (F3b).
pub fn nogrid_5a00<E: FieldEnv>(env: &mut E, player: u32, a: u32) -> u32 {
    let n = nogrid(env, player);
    c2(env, NOGRID_5A00, n, a)
}

/// C: `FUN_021A36BC` (unlabeled routine at `0x021A36BC`): tail call `FUN_021A5A0C(no-grid player, a)` (F3b).
pub fn nogrid_5a0c<E: FieldEnv>(env: &mut E, player: u32, a: u32) -> u32 {
    let n = nogrid(env, player);
    c2(env, NOGRID_5A0C, n, a)
}

/// C: `FUN_021A36C8` (unlabeled routine at `0x021A36C8`): tail call `FUN_021A5A18(no-grid player, a, b)` (F3b).
pub fn nogrid_5a18<E: FieldEnv>(env: &mut E, player: u32, a: u32, b: u32) -> u32 {
    let n = nogrid(env, player);
    env.callv(NOGRID_5A18, &[n, a, b])
}

/// C: `FUN_021A36D4` (unlabeled routine at `0x021A36D4`): tail call `FUN_021A5A24(no-grid player, a)` (F3b).
pub fn nogrid_5a24<E: FieldEnv>(env: &mut E, player: u32, a: u32) -> u32 {
    let n = nogrid(env, player);
    c2(env, NOGRID_5A24, n, a)
}

/// C: `FUN_021A36E0` (unlabeled routine at `0x021A36E0`): FIELD_PLAYER `+8`, the grid player.
pub fn grid_of(mem: &Mem, player: u32) -> u32 {
    mem.r32(player.wrapping_add(8))
}

#[cfg(test)]
mod tests;
