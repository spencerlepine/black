//! `field_player_core.c` (overlay 21, `0x021A36E4..0x021A4140`): PLAYER CORE, the 0x34-byte state of the player
//! MMDL shared by the grid and no-grid players: the move form (walk / cycle / surf, saved in the PLAYER record of the
//! GAMEDATA), the OBJ code changes that go with it, the pad -> direction mapping and the request state machine
//! (`+0x24` request bits, procs in ROM `0x021D89C4`).
//!
//! F3a-spec specced all 64 functions (`tools/rom-diff/specs/black-field/player_core.spec`, header: the CORE layout,
//! requests and forms); F3a ported them. CORE: `+4` GAMESYS, `+8` FIELDMAP, `+0xC` PLAYER record, `+0x10` move
//! state, `+0x14` move value, `+0x18` sex (form-table row), `+0x1C` the player MMDL, `+0x20` / `+0x22` last pressed
//! x / z direction, `+0x24` request bits (bit i = request i, 0..9), `+0x28` surf effect task, `+0x2C` / `+0x2E`
//! counters, `+0x30 s16` request step, `+0x32` request done.
//!
//! Every callee outside the area (the fldmmdl getters / setters (F4a), the PLAYER record accessors of `game_data`,
//! the form table of `crate::player` (`0x021A3190` / `0x021A31A4` / `0x021A31D0`), the surf Pokemon of
//! `crate::namipoke`, the draw side) is reached through [`FieldEnv::call`] by address; callees inside the area are
//! Rust calls, including the request procs of the ROM table [`REQUEST_PROCS`].

use crate::hooks::{CArg, FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_36E4,
    0x021A_3818,
    0x021A_3898,
    0x021A_38A4,
    0x021A_3920,
    0x021A_392C,
    0x021A_3938,
    0x021A_394C,
    0x021A_395C,
    0x021A_3960,
    0x021A_3964,
    0x021A_3968,
    0x021A_396C,
    0x021A_3970,
    0x021A_3974,
    0x021A_3980,
    0x021A_398C,
    0x021A_3990,
    0x021A_3994,
    0x021A_3998,
    0x021A_39AC,
    0x021A_39E4,
    0x021A_3A10,
    0x021A_3A20,
    0x021A_3A3C,
    0x021A_3AA0,
    0x021A_3AB8,
    0x021A_3AC4,
    0x021A_3AD0,
    0x021A_3AD4,
    0x021A_3AD8,
    0x021A_3B3C,
    0x021A_3B78,
    0x021A_3B94,
    0x021A_3BB4,
    0x021A_3C24,
    0x021A_3C3C,
    0x021A_3C54,
    0x021A_3C6C,
    0x021A_3C94,
    0x021A_3CB0,
    0x021A_3CC0,
    0x021A_3CE8,
    0x021A_3D08,
    0x021A_3D50,
    0x021A_3D6C,
    0x021A_3DA0,
    0x021A_3E14,
    0x021A_3E48,
    0x021A_3E54,
    0x021A_3EE0,
    0x021A_3F10,
    0x021A_3F48,
    0x021A_3F64,
    0x021A_3F70,
    0x021A_3F8C,
    0x021A_3F98,
    0x021A_3FB4,
    0x021A_3FC0,
    0x021A_3FDC,
    0x021A_3FE8,
    0x021A_408C,
    0x021A_40A8,
    0x021A_4114,
];

/// No direction (the pad helpers' answer when no key of the axis is held).
pub const DIR_NONE: u32 = 9;
/// No saved OBJ code (PLAYER record `+0x1C`).
pub const NO_CODE: u32 = 0xFFFF;

/// Address of the string `"field_player_core.c"` (the file argument of the allocation in [`create`]).
pub const FILE_PLAYER_CORE: u32 = 0x021D_DEB8;

/// ROM `0x021D89A0`: the HEADER template of the player MMDL (OBJ id 0xFF, OBJ code 1, event id 0xFFFF).
pub const HEADER_TEMPLATE: [u8; 0x24] = [
    0xFF, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

/// ROM `0x021D89C4`: per request (0..9) the address of its proc list (u32 Thumb pointers, indexed by the step
/// `+0x30`). The lists overlap: they are windows into [`REQUEST_PROCS`].
pub const REQUEST_LISTS: [u32; 10] = [
    0x021D_8960,
    0x021D_8990,
    0x021D_8964,
    0x021D_8980,
    0x021D_8978,
    0x021D_8968,
    0x021D_8970,
    0x021D_8988,
    0x021D_895C,
    0x021D_8998,
];

/// Address of [`REQUEST_PROCS`] in the ROM.
pub const REQUEST_PROCS_BASE: u32 = 0x021D_895C;
/// ROM `0x021D895C..0x021D89A0`: the request proc words (Thumb pointers) the lists of [`REQUEST_LISTS`] point into.
pub const REQUEST_PROCS: [u32; 17] = [
    0x021A_3FE9,
    0x021A_3DA1,
    0x021A_3E55,
    0x021A_3F71,
    0x021A_3F8D,
    0x021A_3F99,
    0x021A_3FB5,
    0x021A_3F49,
    0x021A_3F65,
    0x021A_3EE1,
    0x021A_3F11,
    0x021A_3FC1,
    0x021A_3FDD,
    0x021A_3E15,
    0x021A_3E49,
    0x021A_408D,
    0x021A_40A9,
];

// game_data: PLAYER record / GAMEDATA accessors
const GAMEDATA: u32 = 0x0201_20F4; // GAMESYS -> GAMEDATA
const GD_A30: u32 = 0x0201_2A30; // GAMEDATA getter
const REC_FORM: u32 = 0x0201_2B28; // PLAYER record +0x40 (move form)
const REC_SET_FORM: u32 = 0x0201_2B2C; // PLAYER record +0x40 = form
const REC_SET_CODE: u32 = 0x0201_2B30; // PLAYER record +0x1C = OBJ code
const REC_CODE: u32 = 0x0201_2B34; // PLAYER record +0x1C (saved OBJ code)
const ZONE_29D8C: u32 = 0x0202_9D8C; // (gamedata getter, gamedata, zone id)
const FIND_508EC: u32 = 0x0205_08EC; // (obj, id, &u16 out): found?
                                     // mmdl_move
const MOVE_PAIR: u32 = 0x0216_4240; // move an s16 grid pair in dir (dir, &x, &z, n)
                                    // fldmmdl
const SYS_UPDATE: u32 = 0x0216_CD88; // MMDLSYS (sys)
const CREATE: u32 = 0x0216_CE60; // MMDL create (sys, &header, zone)
const ACMD_SETTABLE: u32 = 0x0216_D3CC;
const ACMD_DONE: u32 = 0x0216_D454;
const SYS_DRAW: u32 = 0x0216_D520; // MMDLSYS +0x24
const OBJ_CODE: u32 = 0x0216_D598;
const SET_DISP_DIR: u32 = 0x0216_D5E0;
const DISP_DIR: u32 = 0x0216_D5F8;
const MOVE_DIR: u32 = 0x0216_D608;
const SYS_OF: u32 = 0x0216_D67C;
const PARAM: u32 = 0x0216_D7A8; // MMDL param (m, 0)
const SET_OLD_GX: u32 = 0x0216_D80C;
const SET_OLD_GY: u32 = 0x0216_D818;
const SET_OLD_GZ: u32 = 0x0216_D824;
const GX: u32 = 0x0216_D83C;
const SET_GX: u32 = 0x0216_D844;
const GY: u32 = 0x0216_D854;
const SET_GY: u32 = 0x0216_D85C;
const GZ: u32 = 0x0216_D86C;
const SET_GZ: u32 = 0x0216_D874;
const GET_POS: u32 = 0x0216_D88C;
const SET_POS: u32 = 0x0216_D89C;
const SET_DRAW_OFS: u32 = 0x0216_D8E0;
const SET_MOVE_BIT: u32 = 0x0216_DB80; // MMDL move bit (m, on)
const AT_GRID: u32 = 0x0216_DDC0; // MMDL at grid (sys, x, z, flag)
const BY_ID: u32 = 0x0216_DE94; // MMDL by OBJ id (sys, id)
const CHANGE_CODE: u32 = 0x0216_E204; // MMDL change OBJ code (m, code)
                                      // fieldmap
const FM_MMDLSYS: u32 = 0x0218_8C7C;
const FM_GAMESYS: u32 = 0x0218_8C80;
const FM_ZONE: u32 = 0x0218_8C9C;
const FM_FX_CTRL: u32 = 0x0218_8CD0;
// fldmmdl_acmd / blact / g3dobj, rail
const ACMD_PRED: u32 = 0x0219_7C9C;
const BLACT_PRED: u32 = 0x0219_C648;
const BLACT_DRAW: u32 = 0x0219_C9DC;
const CHANGE_MODEL: u32 = 0x0219_D208; // fldmmdl_g3dobj: change model (m, code)
const RAIL_F5C8: u32 = 0x0219_F5C8; // rail (m, flag)
                                    // player (form table)
const ENTRY_CODE: u32 = 0x021A_3190; // (sex, i)
const FORM_CODE: u32 = 0x021A_31A4; // (sex, form)
const CODE_ENTRY: u32 = 0x021A_31D0; // (code)
                                     // field effects / namipoke
const FX_DELETE: u32 = 0x021A_C154; // field effect: delete task
const SURF_ADD: u32 = 0x021A_CB38; // namipoke: (ctrl, dir, &pos, m, 1)
const SURF_MODE: u32 = 0x021A_CBC8; // namipoke: (task, v)
const FX_D05D8: u32 = 0x021D_05D8; // (MMDL, field effect control)
/// `Heap_AllocDebug`.
const HEAP_ALLOC: u32 = crate::hooks::HEAP_ALLOC;

fn c1<E: FieldEnv>(env: &mut E, addr: u32, a: u32) -> u32 {
    env.callv(addr, &[a])
}

fn c2<E: FieldEnv>(env: &mut E, addr: u32, a: u32, b: u32) -> u32 {
    env.callv(addr, &[a, b])
}

/// The tile of an fx32 value before the final shift: `(v >> 4) + ((v >> 4) < 0 ? 0xFFF : 0)` (the `asrs 4; asrs 0xB;
/// lsrs 0x14; adds` sequence).
fn tile_pre(v: u32) -> i32 {
    let t = (v as i32) >> 4;
    t.wrapping_add(((t >> 11) as u32 >> 20) as i32)
}

/// `tile_pre(v) >> 12` stored as a halfword (`asrs 0xC; strh`).
fn tile16(v: u32) -> u16 {
    (tile_pre(v) >> 12) as u16
}

/// `(s16)(tile_pre(v) << 4 >> 16)` as a register (`lsls 4; asrs 0x10`).
fn tile_s16(v: u32) -> u32 {
    ((tile_pre(v) << 4) >> 16) as u32
}

fn rd<E: FieldEnv>(env: &mut E, core: u32, o: u32) -> u32 {
    env.r32(core.wrapping_add(o))
}

/// The step `+0x30` (s16) plus one, stored back as a halfword.
fn step_up<E: FieldEnv>(env: &mut E, core: u32) {
    let s = env.r16(core.wrapping_add(0x30)) as i16 as i32;
    env.w16(core.wrapping_add(0x30), s.wrapping_add(1) as u16);
}

/// C: `FUN_021A36E4` (unlabeled routine at `0x021A36E4`): create the PLAYER CORE (`rec`, `fieldmap`, `&pos`, `sex`,
/// stack `heap`): `Heap_AllocDebug(heap, 0x34, 1, "field_player_core.c", 0x97)`; `+0xC` rec, `+8` fieldmap, `+4`
/// GAMESYS (`FUN_02188C80`), `+0x20` / `+0x22` = 9; `+0x1C` = the MMDL of OBJ id 0xFF (`FUN_0216DE94`), `+0x18` sex;
/// code = the saved OBJ code (`FUN_02012B34`). No MMDL: the [`HEADER_TEMPLATE`] copy gets the tile x / z of `pos`
/// (u16), `pos.y` and the OBJ code (`code`, or for 0xFFFF the code of the saved form `FUN_021A31A4(sex,
/// form)`), `+0x1C` = `FUN_0216CE60(sys, &header, 0)`. Else the MMDL gets grid x / y / z and the position, and for
/// code 0xFFFF [`set_form_code`]. Then `FUN_0216DB80(+0x1C, 1)`; returns the core.
pub fn create<E: FieldEnv>(env: &mut E, rec: u32, fm: u32, pos: u32, sex: u32, heap: u32) -> u32 {
    let core = env.callv(
        HEAP_ALLOC,
        &[heap & 0xFFFF, 0x34, 1, FILE_PLAYER_CORE, 0x97],
    );
    env.w32(core.wrapping_add(0xC), rec);
    env.w32(core.wrapping_add(8), fm);
    let gs = c1(env, FM_GAMESYS, fm);
    env.w32(core.wrapping_add(4), gs);
    env.w16(core.wrapping_add(0x20), 9);
    env.w16(core.wrapping_add(0x22), 9);
    let sys = c1(env, FM_MMDLSYS, fm);
    let m = c2(env, BY_ID, sys, 0xFF);
    env.w32(core.wrapping_add(0x1C), m);
    env.w32(core.wrapping_add(0x18), sex);
    let code = c1(env, REC_CODE, rec);
    let m = rd(env, core, 0x1C);
    if m == 0 {
        let mut h = HEADER_TEMPLATE;
        let px = env.r32(pos);
        h[0x1C..0x1E].copy_from_slice(&tile16(px).to_le_bytes());
        let pz = env.r32(pos.wrapping_add(8));
        h[0x1E..0x20].copy_from_slice(&tile16(pz).to_le_bytes());
        let py = env.r32(pos.wrapping_add(4));
        h[0x20..0x24].copy_from_slice(&py.to_le_bytes());
        let c = if code == NO_CODE {
            let f = form(env, core);
            c2(env, FORM_CODE, sex, f)
        } else {
            code
        };
        h[2..4].copy_from_slice(&(c as u16).to_le_bytes());
        let nm = env.call(CREATE, &mut [CArg::V(sys), CArg::InBuf(&h), CArg::V(0)]);
        env.w32(core.wrapping_add(0x1C), nm);
    } else {
        let y = tile_s16(env.r32(pos.wrapping_add(4)));
        let z = tile_s16(env.r32(pos.wrapping_add(8)));
        let x = tile_s16(env.r32(pos));
        c2(env, SET_GX, m, x);
        c2(env, SET_GY, m, y);
        c2(env, SET_GZ, m, z);
        env.call(SET_POS, &mut [CArg::V(m), CArg::In(pos, 12)]);
        if code == NO_CODE {
            set_form_code(env, core);
        }
    }
    let m = rd(env, core, 0x1C);
    c2(env, SET_MOVE_BIT, m, 1);
    core
}

/// C: `FUN_021A3818` (unlabeled routine at `0x021A3818`): update the move state `+0x10` (cleared first) from the
/// previous state `s` and the move value `v` (`+0x14`): acmd not settable (`FUN_0216D3CC == 0`): v 1: s 0 / 3 -> 1,
/// else 2; v 2: 2. Else, the acmd done (`FUN_0216D454 == 1`) and v 1 / 2: s 0 -> 0, s 3 -> 0, else 3.
pub fn update_state<E: FieldEnv>(env: &mut E, core: u32) {
    let m = rd(env, core, 0x1C);
    let s = rd(env, core, 0x10);
    env.w32(core.wrapping_add(0x10), 0);
    let v = rd(env, core, 0x14);
    let st = core.wrapping_add(0x10);
    if c1(env, ACMD_SETTABLE, m) == 0 {
        match v {
            1 => env.w32(st, if s == 0 || s == 3 { 1 } else { 2 }),
            2 => env.w32(st, 2),
            _ => {}
        }
    } else if c1(env, ACMD_DONE, m) == 1 && (v == 1 || v == 2) && s != 0 {
        env.w32(st, if s == 3 { 0 } else { 3 });
    }
}

/// C: `FUN_021A3898` (unlabeled routine at `0x021A3898`): the MMDL position into `out` (`FUN_0216D88C`).
pub fn get_pos<E: FieldEnv>(env: &mut E, core: u32, out: u32) {
    let m = rd(env, core, 0x1C);
    env.call(GET_POS, &mut [CArg::V(m), CArg::Out(out, 12)]);
}

/// C: `FUN_021A38A4` (unlabeled routine at `0x021A38A4`): warp the MMDL to `pos`: old grid = grid, grid x / y / z =
/// the tiles of `pos` (s16), position = `pos`.
pub fn warp<E: FieldEnv>(env: &mut E, core: u32, pos: u32) {
    let x = tile_s16(env.r32(pos));
    let m = rd(env, core, 0x1C);
    let y = tile_s16(env.r32(pos.wrapping_add(4)));
    let z = tile_s16(env.r32(pos.wrapping_add(8)));
    let g = c1(env, GX, m);
    c2(env, SET_OLD_GX, m, g);
    let g = c1(env, GY, m);
    c2(env, SET_OLD_GY, m, g);
    let g = c1(env, GZ, m);
    c2(env, SET_OLD_GZ, m, g);
    c2(env, SET_GX, m, x);
    c2(env, SET_GY, m, y);
    c2(env, SET_GZ, m, z);
    env.call(SET_POS, &mut [CArg::V(m), CArg::In(pos, 12)]);
}

/// C: `FUN_021A3920` (unlabeled routine at `0x021A3920`): the MMDL display dir (`FUN_0216D5F8`).
pub fn disp_dir<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let m = rd(env, core, 0x1C);
    c1(env, DISP_DIR, m)
}

/// C: `FUN_021A392C` (unlabeled routine at `0x021A392C`): set the MMDL display dir (`FUN_0216D5E0`).
pub fn set_disp_dir<E: FieldEnv>(env: &mut E, core: u32, dir: u32) {
    let m = rd(env, core, 0x1C);
    c2(env, SET_DISP_DIR, m, dir);
}

/// C: `FUN_021A3938` (unlabeled routine at `0x021A3938`): [`change_code`], then save the code in the PLAYER record
/// (`FUN_02012B30`).
pub fn change_and_save_code<E: FieldEnv>(env: &mut E, core: u32, code: u32) {
    change_code(env, core, code);
    let rec = rd(env, core, 0xC);
    c2(env, REC_SET_CODE, rec, code);
}

/// C: `FUN_021A394C` (unlabeled routine at `0x021A394C`): clear the saved OBJ code (`FUN_02012B30(rec, 0xFFFF)`).
pub fn clear_saved_code<E: FieldEnv>(env: &mut E, core: u32) {
    let rec = rd(env, core, 0xC);
    c2(env, REC_SET_CODE, rec, NO_CODE);
}

/// C: `FUN_021A395C` (unlabeled routine at `0x021A395C`): `+8`, the FIELDMAP.
pub fn fieldmap(mem: &mut Mem, core: u32) -> u32 {
    mem.r32(core.wrapping_add(8))
}

/// C: `FUN_021A3960` (unlabeled routine at `0x021A3960`): `+0x1C`, the player MMDL.
pub fn mmdl(mem: &mut Mem, core: u32) -> u32 {
    mem.r32(core.wrapping_add(0x1C))
}

/// C: `FUN_021A3964` (unlabeled routine at `0x021A3964`): `+0x14` = the move value.
pub fn set_move_value(mem: &mut Mem, core: u32, v: u32) {
    mem.w32(core.wrapping_add(0x14), v);
}

/// C: `FUN_021A3968` (unlabeled routine at `0x021A3968`): `+0x14`, the move value.
pub fn move_value(mem: &mut Mem, core: u32) -> u32 {
    mem.r32(core.wrapping_add(0x14))
}

/// C: `FUN_021A396C` (unlabeled routine at `0x021A396C`): `+0x10` = the move state.
pub fn set_move_state(mem: &mut Mem, core: u32, v: u32) {
    mem.w32(core.wrapping_add(0x10), v);
}

/// C: `FUN_021A3970` (unlabeled routine at `0x021A3970`): `+0x10`, the move state.
pub fn move_state(mem: &mut Mem, core: u32) -> u32 {
    mem.r32(core.wrapping_add(0x10))
}

/// C: `FUN_021A3974` (unlabeled routine at `0x021A3974`): the move form saved in the PLAYER record
/// (`FUN_02012B28`: 0 walk, 1 cycle, 2 surf, 3 surf variant).
pub fn form<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let rec = rd(env, core, 0xC);
    c1(env, REC_FORM, rec)
}

/// C: `FUN_021A3980` (unlabeled routine at `0x021A3980`): save the move form (`FUN_02012B2C`).
pub fn set_form<E: FieldEnv>(env: &mut E, core: u32, f: u32) {
    let rec = rd(env, core, 0xC);
    c2(env, REC_SET_FORM, rec, f);
}

/// C: `FUN_021A398C` (unlabeled routine at `0x021A398C`): `+0x18`, the sex (form-table row).
pub fn sex(mem: &mut Mem, core: u32) -> u32 {
    mem.r32(core.wrapping_add(0x18))
}

/// C: `FUN_021A3990` (unlabeled routine at `0x021A3990`): `+4`, the GAMESYS.
pub fn gamesys(mem: &mut Mem, core: u32) -> u32 {
    mem.r32(core.wrapping_add(4))
}

/// C: `FUN_021A3994` (unlabeled routine at `0x021A3994`): `+0xC`, the PLAYER record.
pub fn record(mem: &mut Mem, core: u32) -> u32 {
    mem.r32(core.wrapping_add(0xC))
}

/// C: `FUN_021A3998` (unlabeled routine at `0x021A3998`): [`set_form_and_code`] with the saved form.
pub fn set_form_code<E: FieldEnv>(env: &mut E, core: u32) {
    let f = form(env, core);
    set_form_and_code(env, core, f);
}

/// C: `FUN_021A39AC` (unlabeled routine at `0x021A39AC`): the OBJ code of form-table entry `i`
/// (`FUN_021A3190(sex, i)`); if the MMDL has another code, [`change_code`]; `+0x2C` = 0.
pub fn set_entry_code<E: FieldEnv>(env: &mut E, core: u32, i: u32) {
    let s = sex(env.mem(), core);
    let m = mmdl(env.mem(), core);
    let code = c2(env, ENTRY_CODE, s, i);
    if code != c1(env, OBJ_CODE, m) {
        change_code(env, core, code);
    }
    env.w16(core.wrapping_add(0x2C), 0);
}

/// C: `FUN_021A39E4` (unlabeled routine at `0x021A39E4`): MMDL param 0 (`FUN_0216D7A8(m, 0)`) not 0xFFFF: 1 when
/// the counter `+0x2C` is 1, else the counter + 1 and 0; param 0xFFFF: 0.
pub fn param_wait<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let m = mmdl(env.mem(), core);
    if c2(env, PARAM, m, 0) != NO_CODE {
        let n = env.r16(core.wrapping_add(0x2C));
        if n == 1 {
            return 1;
        }
        env.w16(core.wrapping_add(0x2C), n.wrapping_add(1));
    }
    0
}

/// C: `FUN_021A3A10` (unlabeled routine at `0x021A3A10`): the form-table entry of the MMDL OBJ code
/// (`FUN_021A31D0(FUN_0216D598(m))`, 10 = none).
pub fn code_entry<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let m = mmdl(env.mem(), core);
    let c = c1(env, OBJ_CODE, m);
    c1(env, CODE_ENTRY, c)
}

/// C: `FUN_021A3A20` (unlabeled routine at `0x021A3A20`): 1 when the MMDL OBJ code is a form-table code.
pub fn has_form_code<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let m = mmdl(env.mem(), core);
    let c = c1(env, OBJ_CODE, m);
    (c1(env, CODE_ENTRY, c) != 10) as u32
}

/// C: `FUN_021A3A3C` (unlabeled routine at `0x021A3A3C`): is the MMDL busy? d = `FUN_0219C9DC(FUN_0216D520(sys))`;
/// 1 when `FUN_02197C9C(m) == 1` and `FUN_0219C648(m) == 1`; else with param 0 p != 0xFFFF and
/// `FUN_020508EC(d, (u16)p, &out)` nonzero, 1 when the u16 `out` is 0x3FFF; else 0.
pub fn busy<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let m = rd(env, core, 0x1C);
    let sys = c1(env, SYS_OF, m);
    let x = c1(env, SYS_DRAW, sys);
    let d = c1(env, BLACT_DRAW, x);
    let m = rd(env, core, 0x1C);
    if c1(env, ACMD_PRED, m) == 1 {
        let m = rd(env, core, 0x1C);
        if c1(env, BLACT_PRED, m) == 1 {
            return 1;
        }
    }
    let m = rd(env, core, 0x1C);
    let p = c2(env, PARAM, m, 0);
    if p != NO_CODE {
        let mut out = [0u8; 2];
        let r = env.call(
            FIND_508EC,
            &mut [CArg::V(d), CArg::V(p & 0xFFFF), CArg::OutBuf(&mut out)],
        );
        if r != 0 && u16::from_le_bytes(out) as u32 == NO_CODE >> 2 {
            return 1;
        }
    }
    0
}

/// C: `FUN_021A3AA0` (unlabeled routine at `0x021A3AA0`): 1 when the MMDL acmd is settable (`FUN_0216D3CC == 1`).
pub fn acmd_settable<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let m = mmdl(env.mem(), core);
    (c1(env, ACMD_SETTABLE, m) == 1) as u32
}

/// C: `FUN_021A3AB8` (unlabeled routine at `0x021A3AB8`): [`dir_of_keys`] of the low halfword of `keys`.
pub fn dir_of_keys16<E: FieldEnv>(env: &mut E, core: u32, keys: u32) -> u32 {
    dir_of_keys(env, core, keys & 0xFFFF)
}

/// C: `FUN_021A3AC4` (unlabeled routine at `0x021A3AC4`): [`remember_keys`] of the low halfword of `keys`.
pub fn remember_keys16<E: FieldEnv>(env: &mut E, core: u32, keys: u32) {
    remember_keys(env.mem(), core, keys & 0xFFFF);
}

/// C: `FUN_021A3AD0` (unlabeled routine at `0x021A3AD0`): `+0x28` = the surf effect task.
pub fn set_surf_task(mem: &mut Mem, core: u32, v: u32) {
    mem.w32(core.wrapping_add(0x28), v);
}

/// C: `FUN_021A3AD4` (unlabeled routine at `0x021A3AD4`): `+0x28`, the surf effect task.
pub fn surf_task(mem: &mut Mem, core: u32) -> u32 {
    mem.r32(core.wrapping_add(0x28))
}

/// C: `FUN_021A3AD8` (unlabeled routine at `0x021A3AD8`): change the OBJ code of the player MMDL to `code`: when
/// `code` or the current code (`FUN_0216D598`) is a walk code (1 / 4): `FUN_0219D208(m, code)` (model),
/// `FUN_0216CD88(sys)`; else first the walk code `FUN_0216E204(m, FUN_021A31A4(sex, 0))`, `FUN_0216CD88(sys)`, then
/// `FUN_0219D208(m, code)`, `FUN_0216CD88(sys)`.
pub fn change_code<E: FieldEnv>(env: &mut E, core: u32, code: u32) {
    let m = mmdl(env.mem(), core);
    let sys = c1(env, SYS_OF, m);
    let cur = c1(env, OBJ_CODE, m);
    if code == 1 || code == 4 || cur == 1 || cur == 4 {
        c2(env, CHANGE_MODEL, m, code);
        c1(env, SYS_UPDATE, sys);
        return;
    }
    let s = sex(env.mem(), core);
    let walk = c2(env, FORM_CODE, s, 0);
    c2(env, CHANGE_CODE, m, walk);
    c1(env, SYS_UPDATE, sys);
    c2(env, CHANGE_MODEL, m, code);
    c1(env, SYS_UPDATE, sys);
}

/// C: `FUN_021A3B3C` (unlabeled routine at `0x021A3B3C`): code = `FUN_021A31A4(sex, form)`; if the MMDL has another
/// code, [`change_code`]; then [`set_form`].
pub fn set_form_and_code<E: FieldEnv>(env: &mut E, core: u32, f: u32) {
    let s = sex(env.mem(), core);
    let m = mmdl(env.mem(), core);
    let code = c2(env, FORM_CODE, s, f);
    if code != c1(env, OBJ_CODE, m) {
        change_code(env, core, code);
    }
    set_form(env, core, f);
}

/// C: `FUN_021A3B78` (unlabeled routine at `0x021A3B78`): 1 when the MMDL OBJ code is a walk code (1 / 4).
pub fn is_walk_code<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let m = mmdl(env.mem(), core);
    let c = c1(env, OBJ_CODE, m);
    (c == 1 || c == 4) as u32
}

/// C: `FUN_021A3B94` (unlabeled routine at `0x021A3B94`): 1 when the low halfword of MMDL param 0
/// (`FUN_0216D7A8(m, 0)`) is not 0xFFFF.
pub fn has_param<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let m = rd(env, core, 0x1C);
    (c2(env, PARAM, m, 0) & 0xFFFF != NO_CODE) as u32
}

/// C: `FUN_021A3BB4` (unlabeled routine at `0x021A3BB4`): the direction of the pad bits: x = [`key_dir_x`], z =
/// [`key_dir_z`]; one of them none: the other; both: the MMDL move dir d (`FUN_0216D608`) none: z; x and z equal to
/// the remembered `+0x20` / `+0x22`: z if d == z, x if d == x; then z if z is not the remembered one, else x (the
/// newly pressed axis wins). Returned as a u16.
pub fn dir_of_keys<E: FieldEnv>(env: &mut E, core: u32, keys: u32) -> u32 {
    let x = key_dir_x(keys);
    let z = key_dir_z(keys);
    if x == DIR_NONE {
        return z & 0xFFFF;
    }
    if z == DIR_NONE {
        return x & 0xFFFF;
    }
    let m = mmdl(env.mem(), core);
    let d = c1(env, MOVE_DIR, m);
    let rx = env.r16(core.wrapping_add(0x20)) as u32;
    let rz = env.r16(core.wrapping_add(0x22)) as u32;
    if d == DIR_NONE {
        return z & 0xFFFF;
    }
    if x == rx && z == rz {
        if z == d {
            return z & 0xFFFF;
        }
        if x == d {
            return x & 0xFFFF;
        }
    }
    if z != rz {
        z & 0xFFFF
    } else {
        x & 0xFFFF
    }
}

/// C: `FUN_021A3C24` (unlabeled routine at `0x021A3C24`): remember the pressed directions: `+0x20` = the x direction,
/// `+0x22` = the z direction of `keys` (u16 stores).
pub fn remember_keys(mem: &mut Mem, core: u32, keys: u32) {
    mem.w16(core.wrapping_add(0x20), key_dir_x(keys) as u16);
    mem.w16(core.wrapping_add(0x22), key_dir_z(keys) as u16);
}

/// C: `FUN_021A3C3C` (unlabeled routine at `0x021A3C3C`): the x direction of the pad bits: 0x20 (left) -> 2,
/// 0x10 (right) -> 3, else [`DIR_NONE`]; left wins.
pub fn key_dir_x(keys: u32) -> u32 {
    if keys & 0x20 != 0 {
        2
    } else if keys & 0x10 != 0 {
        3
    } else {
        DIR_NONE
    }
}

/// C: `FUN_021A3C54` (unlabeled routine at `0x021A3C54`): the z direction of the pad bits: 0x40 (up) -> 0,
/// 0x80 (down) -> 1, else [`DIR_NONE`]; up wins.
pub fn key_dir_z(keys: u32) -> u32 {
    if keys & 0x40 != 0 {
        0
    } else if keys & 0x80 != 0 {
        1
    } else {
        DIR_NONE
    }
}

/// Runs the request proc at ROM address `p` (Thumb bit set), one of the in-area procs of [`REQUEST_PROCS`].
fn run_proc<E: FieldEnv>(env: &mut E, p: u32, core: u32) -> u32 {
    match p & !1 {
        0x021A_3DA0 => req_walk(env, core),
        0x021A_3E14 => req_cycle(env, core),
        0x021A_3E48 => req_cycle_wait(env, core),
        0x021A_3E54 => req_surf(env, core),
        0x021A_3EE0 => req_saved(env, core),
        0x021A_3F10 => req_saved_wait(env, core),
        0x021A_3F48 => req_entry4(env, core),
        0x021A_3F64 => req_entry4_wait(env, core),
        0x021A_3F70 => req_entry5(env, core),
        0x021A_3F8C => req_entry5_wait(env, core),
        0x021A_3F98 => req_entry6(env, core),
        0x021A_3FB4 => req_entry6_wait(env, core),
        0x021A_3FC0 => req_entry9(env, core),
        0x021A_3FDC => req_entry9_wait(env, core),
        0x021A_3FE8 => req_surf_variant(env, core),
        0x021A_408C => req_entry8(env, core),
        0x021A_40A8 => req_entry8_wait(env, core),
        // UNVERIFIED: a step past the lists reads other ROM words as code pointers in the original
        _ => panic!("request proc {p:#010x} is not a request proc"),
    }
}

/// C: `FUN_021A3C6C` (unlabeled routine at `0x021A3C6C`): run request `req`: while `+0x32` (done) is 0, run the
/// proc `ROM 0x021D89C4[req][+0x30]` (s16 step) and again while it answers 1; returns `+0x32`. UNVERIFIED: a
/// request or step outside the ROM lists panics (the original reads the following ROM).
pub fn run_request<E: FieldEnv>(env: &mut E, core: u32, req: u32) -> u32 {
    if env.r16(core.wrapping_add(0x32)) == 0 {
        let list = REQUEST_LISTS[req as usize];
        loop {
            let step = env.r16(core.wrapping_add(0x30)) as i16 as i32;
            let a = list.wrapping_add((step as u32).wrapping_mul(4));
            let i = (a.wrapping_sub(REQUEST_PROCS_BASE) / 4) as usize;
            assert!(
                a >= REQUEST_PROCS_BASE && a & 3 == 0,
                "request list read outside the ROM block"
            );
            let p = REQUEST_PROCS[i];
            if run_proc(env, p, core) != 1 {
                break;
            }
        }
    }
    env.r16(core.wrapping_add(0x32)) as u32
}

/// C: `FUN_021A3C94` (unlabeled routine at `0x021A3C94`): the index of the lowest set bit among bits 0..9 of the
/// request bits, 10 when none is set.
pub fn first_request(bits: u32) -> u32 {
    (0..10).find(|&i| bits >> i & 1 != 0).unwrap_or(10)
}

/// C: `FUN_021A3CB0` (unlabeled routine at `0x021A3CB0`): start a request set: `+0x24` = bits; the step `+0x30`,
/// done `+0x32` and counters `+0x2C` / `+0x2E` = 0.
pub fn set_requests(mem: &mut Mem, core: u32, bits: u32) {
    mem.w32(core.wrapping_add(0x24), bits);
    for o in [0x30, 0x32, 0x2C, 0x2E] {
        mem.w16(core.wrapping_add(o), 0);
    }
}

/// C: `FUN_021A3CC0` (unlabeled routine at `0x021A3CC0`): are the requests done? i = [`first_request`]; none: 1;
/// else 1 when [`run_request`] answers 1, else 0.
pub fn requests_done<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let i = first_request(rd(env, core, 0x24));
    if i >= 10 {
        return 1;
    }
    (run_request(env, core, i) == 1) as u32
}

/// C: `FUN_021A3CE8` (unlabeled routine at `0x021A3CE8`): run the first request until [`run_request`] answers
/// nonzero (nothing when no request is set).
pub fn run_requests<E: FieldEnv>(env: &mut E, core: u32) {
    let i = first_request(rd(env, core, 0x24));
    if i < 10 {
        while run_request(env, core, i) == 0 {}
    }
}

/// C: `FUN_021A3D08` (unlabeled routine at `0x021A3D08`): switch to OBJ code `code`: already the MMDL code: `+0x32`
/// = 1, 1; the MMDL in a walk code ([`is_walk_code`]): [`change_code`], `+0x32` = 1, 1; else [`change_code`] to
/// the walk code `FUN_021A3190(sex, 0)` first and 0.
pub fn switch_code<E: FieldEnv>(env: &mut E, core: u32, code: u32) -> u32 {
    let m = mmdl(env.mem(), core);
    if code == c1(env, OBJ_CODE, m) {
        env.w16(core.wrapping_add(0x32), 1);
        return 1;
    }
    if is_walk_code(env, core) == 1 {
        change_code(env, core, code);
        env.w16(core.wrapping_add(0x32), 1);
        return 1;
    }
    let s = sex(env.mem(), core);
    let walk = c2(env, ENTRY_CODE, s, 0);
    change_code(env, core, walk);
    0
}

/// C: `FUN_021A3D50` (unlabeled routine at `0x021A3D50`): [`switch_code`] to form-table entry `i`
/// (`FUN_021A3190(sex, i)`).
pub fn switch_entry<E: FieldEnv>(env: &mut E, core: u32, i: u32) -> u32 {
    let s = sex(env.mem(), core);
    let code = c2(env, ENTRY_CODE, s, i);
    switch_code(env, core, code)
}

/// C: `FUN_021A3D6C` (unlabeled routine at `0x021A3D6C`): with MMDL param 0 set ([`has_param`] == 1): `+0x2E` += 1,
/// [`change_code`] to entry `i` (`FUN_021A3190(sex, i)`), `+0x32` = 1, 1; else 0.
pub fn param_entry<E: FieldEnv>(env: &mut E, core: u32, i: u32) -> u32 {
    if has_param(env, core) != 1 {
        return 0;
    }
    let n = env.r16(core.wrapping_add(0x2E));
    env.w16(core.wrapping_add(0x2E), n.wrapping_add(1));
    let s = sex(env.mem(), core);
    let code = c2(env, ENTRY_CODE, s, i);
    change_code(env, core, code);
    env.w16(core.wrapping_add(0x32), 1);
    1
}

/// C: `FUN_021A3DA0` (unlabeled routine at `0x021A3DA0`): request 0 (walk): the code of entry 0 (changed if
/// different), form 0, [`zone_update`]; a surf effect task: delete it (`FUN_021AC154`) and clear `+0x28`; MMDL draw
/// offset = (0, 0, 0) (`FUN_0216D8E0`); `+0x32` = 1; 0.
pub fn req_walk<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let zero = [0u8; 12];
    let s = sex(env.mem(), core);
    let m = mmdl(env.mem(), core);
    let code = c2(env, ENTRY_CODE, s, 0);
    if code != c1(env, OBJ_CODE, m) {
        change_code(env, core, code);
    }
    set_form(env, core, 0);
    zone_update(env, core);
    let t = surf_task(env.mem(), core);
    if t != 0 {
        c1(env, FX_DELETE, t);
        set_surf_task(env.mem(), core, 0);
    }
    env.call(SET_DRAW_OFS, &mut [CArg::V(m), CArg::InBuf(&zero)]);
    env.w16(core.wrapping_add(0x32), 1);
    0
}

/// C: `FUN_021A3E14` (unlabeled routine at `0x021A3E14`): request 1 step 0 (cycle): form 1, [`zone_update`];
/// [`switch_entry`] 1 not done: step + 1; 0. (It also reads the sex and the MMDL and drops them.)
pub fn req_cycle<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    set_form(env, core, 1);
    zone_update(env, core);
    if switch_entry(env, core, 1) != 1 {
        step_up(env, core);
    }
    0
}

/// C: `FUN_021A3E48` (unlabeled routine at `0x021A3E48`): request 1 step 1: [`param_entry`] 1; 0.
pub fn req_cycle_wait<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    param_entry(env, core, 1);
    0
}

/// The surf Pokemon of requests 2 and 8: `+0x28` = `FUN_021ACB38(FUN_02188CD0(+8), display dir, &position, m, 1)`.
fn add_surf<E: FieldEnv>(env: &mut E, core: u32, m: u32) {
    let fm = rd(env, core, 8);
    let ctrl = c1(env, FM_FX_CTRL, fm);
    let dir = c1(env, DISP_DIR, m);
    let mut pos = [0u8; 12];
    env.call(GET_POS, &mut [CArg::V(m), CArg::OutBuf(&mut pos)]);
    let t = env.call(
        SURF_ADD,
        &mut [
            CArg::V(ctrl),
            CArg::V(dir),
            CArg::InBuf(&pos),
            CArg::V(m),
            CArg::V(1),
        ],
    );
    set_surf_task(env.mem(), core, t);
}

/// C: `FUN_021A3E54` (unlabeled routine at `0x021A3E54`): request 2 (surf): the code of entry 2 (changed if
/// different); form not 2: form 2; [`zone_update`]; no surf effect task: add the surf Pokemon
/// (`FUN_021ACB38(ctrl, dir, &pos, m, 1)`); `+0x32` = 1; 0.
pub fn req_surf<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let s = sex(env.mem(), core);
    let m = mmdl(env.mem(), core);
    let code = c2(env, ENTRY_CODE, s, 2);
    if code != c1(env, OBJ_CODE, m) {
        change_code(env, core, code);
    }
    if form(env, core) != 2 {
        set_form(env, core, 2);
    }
    zone_update(env, core);
    if surf_task(env.mem(), core) == 0 {
        add_surf(env, core, m);
    }
    env.w16(core.wrapping_add(0x32), 1);
    0
}

/// C: `FUN_021A3EE0` (unlabeled routine at `0x021A3EE0`): request 3 step 0: [`switch_code`] to the code of the saved
/// form (`FUN_021A31A4(sex, form)`) not done: step + 1; 0.
pub fn req_saved<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let f = form(env, core);
    let s = sex(env.mem(), core);
    let code = c2(env, FORM_CODE, s, f);
    if switch_code(env, core, code) != 1 {
        step_up(env, core);
    }
    0
}

/// C: `FUN_021A3F10` (unlabeled routine at `0x021A3F10`): request 3 step 1: with MMDL param 0 set: `+0x2E` += 1,
/// [`change_code`] to the code of the saved form, `+0x32` = 1; 0.
pub fn req_saved_wait<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    if has_param(env, core) == 1 {
        let n = env.r16(core.wrapping_add(0x2E));
        env.w16(core.wrapping_add(0x2E), n.wrapping_add(1));
        let f = form(env, core);
        let s = sex(env.mem(), core);
        let code = c2(env, FORM_CODE, s, f);
        change_code(env, core, code);
        env.w16(core.wrapping_add(0x32), 1);
    }
    0
}

/// The step-0 procs of the entry requests: [`switch_entry`] `i` not done: step + 1; 0.
fn req_entry<E: FieldEnv>(env: &mut E, core: u32, i: u32) -> u32 {
    if switch_entry(env, core, i) != 1 {
        step_up(env, core);
    }
    0
}

/// C: `FUN_021A3F48` (unlabeled routine at `0x021A3F48`): request 4 step 0: entry 4.
pub fn req_entry4<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    req_entry(env, core, 4)
}

/// C: `FUN_021A3F64` (unlabeled routine at `0x021A3F64`): request 4 step 1: [`param_entry`] 4; 0.
pub fn req_entry4_wait<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    param_entry(env, core, 4);
    0
}

/// C: `FUN_021A3F70` (unlabeled routine at `0x021A3F70`): request 5 step 0: entry 5.
pub fn req_entry5<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    req_entry(env, core, 5)
}

/// C: `FUN_021A3F8C` (unlabeled routine at `0x021A3F8C`): request 5 step 1: [`param_entry`] 5; 0.
pub fn req_entry5_wait<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    param_entry(env, core, 5);
    0
}

/// C: `FUN_021A3F98` (unlabeled routine at `0x021A3F98`): request 6 step 0: entry 6.
pub fn req_entry6<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    req_entry(env, core, 6)
}

/// C: `FUN_021A3FB4` (unlabeled routine at `0x021A3FB4`): request 6 step 1: [`param_entry`] 6; 0.
pub fn req_entry6_wait<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    param_entry(env, core, 6);
    0
}

/// C: `FUN_021A3FC0` (unlabeled routine at `0x021A3FC0`): request 7 step 0: entry 9.
pub fn req_entry9<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    req_entry(env, core, 9)
}

/// C: `FUN_021A3FDC` (unlabeled routine at `0x021A3FDC`): request 7 step 1: [`param_entry`] 9; 0.
pub fn req_entry9_wait<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    param_entry(env, core, 9);
    0
}

/// C: `FUN_021A3FE8` (unlabeled routine at `0x021A3FE8`): request 8 (surf variant): ctrl = `FUN_02188CD0(+8)`; the
/// code of entry 3 (changed if different); form not 3: form 3; no surf effect task: add the surf Pokemon; then
/// `FUN_021D05D8(m, ctrl)`, a surf task: `FUN_021ACBC8(task, 0)`; `+0x32` = 1; 0.
pub fn req_surf_variant<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    let fm = rd(env, core, 8);
    let ctrl = c1(env, FM_FX_CTRL, fm);
    let s = sex(env.mem(), core);
    let m = mmdl(env.mem(), core);
    let code = c2(env, ENTRY_CODE, s, 3);
    if code != c1(env, OBJ_CODE, m) {
        change_code(env, core, code);
    }
    if form(env, core) != 3 {
        set_form(env, core, 3);
    }
    if surf_task(env.mem(), core) == 0 {
        add_surf(env, core, m);
    }
    let pm = rd(env, core, 0x1C);
    c2(env, FX_D05D8, pm, ctrl);
    let t = rd(env, core, 0x28);
    if t != 0 {
        c2(env, SURF_MODE, t, 0);
    }
    env.w16(core.wrapping_add(0x32), 1);
    0
}

/// C: `FUN_021A408C` (unlabeled routine at `0x021A408C`): request 9 step 0: entry 8.
pub fn req_entry8<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    req_entry(env, core, 8)
}

/// C: `FUN_021A40A8` (unlabeled routine at `0x021A40A8`): request 9 step 1: [`param_entry`] 8; then, done and form
/// 2: the MMDL grid x / z (s16 locals), `FUN_02164240(1, &x, &z, 1)` (one step down), k = `FUN_0216DDC0(sys, x, z, 1)
/// != 0`, `FUN_0219F5C8(m, k)`; 0.
pub fn req_entry8_wait<E: FieldEnv>(env: &mut E, core: u32) -> u32 {
    param_entry(env, core, 8);
    if env.r16(core.wrapping_add(0x32)) == 1 && form(env, core) == 2 {
        let m = mmdl(env.mem(), core);
        let sys = c1(env, SYS_OF, m);
        let x = c1(env, GX, m) as u16;
        let z = c1(env, GZ, m) as u16;
        let xb = x.to_le_bytes();
        let zb = z.to_le_bytes();
        env.call(
            MOVE_PAIR,
            &mut [CArg::V(1), CArg::InBuf(&xb), CArg::InBuf(&zb), CArg::V(1)],
        );
        let k = env.callv(
            AT_GRID,
            &[sys, x as i16 as i32 as u32, z as i16 as i32 as u32, 1],
        );
        c2(env, RAIL_F5C8, m, (k != 0) as u32);
    }
    0
}

/// C: `FUN_021A4114` (unlabeled routine at `0x021A4114`): gd = `FUN_020120F4(FUN_02188C80(+8))`;
/// `FUN_02029D8C(FUN_02012A30(gd), gd, (u16)FUN_02188C9C(+8))` (zone id).
pub fn zone_update<E: FieldEnv>(env: &mut E, core: u32) {
    let fm = rd(env, core, 8);
    let gs = c1(env, FM_GAMESYS, fm);
    let gd = c1(env, GAMEDATA, gs);
    let a = c1(env, GD_A30, gd);
    let fm = rd(env, core, 8);
    let z = c1(env, FM_ZONE, fm);
    env.callv(ZONE_29D8C, &[a, gd, z & 0xFFFF]);
}

#[cfg(test)]
mod tests;
