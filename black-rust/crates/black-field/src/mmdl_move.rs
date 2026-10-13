//! MMDL movement helpers in F1 row 10.23 (overlay 10, `0x02163010..0x021643AC`): the grid move procs, height and
//! attribute tracking, the attribute-driven field effects (ATTR INFO, 0x1C bytes), the hit checks of a step
//! (move limit, blocking attributes, height, other MMDLs, map bounds), direction tables and the rail-header glue.
//!
//! F6a-spec specced all 70 functions (`tools/rom-diff/specs/black-field/mmdl_move.spec`, whose header documents
//! ATTR INFO and the direction tables); F6a ported them. MMDL fields are read only through the fldmmdl getters
//! (`crate::fldmmdl`, F4a), which the spec keeps as `force` stubs: the ports reach them (and the map queries, the
//! attribute predicates of overlay 21, the field effects, `fldmmdl_acmd` and the rail system) through
//! [`FieldEnv::call`] by address. Callees inside the area are Rust calls; the ATTR INFO, positions and rectangles
//! the original keeps on its stack are [`Mem::push_local`] windows. Directions: 0 up, 1 down, 2 left, 3 right.
//!
//! ATTR INFO (0x1C): `+0 u32` previous attribute (-1 = none), `+4 u32` attribute, `+8 / +0xA u16` their values
//! (`021AB0F0`), `+0xC / +0xE u16` their flags (`021AB0F8`: 2, 8, 0x20 effect, 0x40, 0x80), `+0x10` -> OBJ DATA,
//! `+0x14` field-effect control, `+0x18 u8` season. Grid <-> fx32: tile = `(fx >> 4) / 4096` (toward zero),
//! centre = `tile << 16 + 0x8000`.

use crate::hooks::{CArg, FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_3010,
    0x0216_3028,
    0x0216_3090,
    0x0216_30F4,
    0x0216_310C,
    0x0216_3130,
    0x0216_3154,
    0x0216_3188,
    0x0216_31C0,
    0x0216_3230,
    0x0216_3288,
    0x0216_32F0,
    0x0216_3340,
    0x0216_33A8,
    0x0216_3418,
    0x0216_3464,
    0x0216_34D8,
    0x0216_34FC,
    0x0216_352C,
    0x0216_355C,
    0x0216_3580,
    0x0216_3618,
    0x0216_3650,
    0x0216_369C,
    0x0216_36A8,
    0x0216_36C8,
    0x0216_36F0,
    0x0216_372C,
    0x0216_375C,
    0x0216_377C,
    0x0216_37B0,
    0x0216_37E4,
    0x0216_3868,
    0x0216_38AC,
    0x0216_38C8,
    0x0216_38CC,
    0x0216_3994,
    0x0216_39C0,
    0x0216_3A08,
    0x0216_3A40,
    0x0216_3AF0,
    0x0216_3BAC,
    0x0216_3C04,
    0x0216_3C9C,
    0x0216_3D04,
    0x0216_3E1C,
    0x0216_3E34,
    0x0216_3E58,
    0x0216_3E7C,
    0x0216_3F10,
    0x0216_3F40,
    0x0216_3F8C,
    0x0216_3FC0,
    0x0216_4010,
    0x0216_40B0,
    0x0216_4168,
    0x0216_41C8,
    0x0216_41E8,
    0x0216_41F8,
    0x0216_4208,
    0x0216_4240,
    0x0216_4280,
    0x0216_4294,
    0x0216_42A4,
    0x0216_42B4,
    0x0216_42D4,
    0x0216_42F8,
    0x0216_4304,
    0x0216_435C,
    0x0216_4380,
];

/// Size of the ATTR INFO record (`FUN_021631C0`).
pub const ATTR_INFO_SIZE: u32 = 0x1C;
/// The marker word of a rail location stored in the MMDL move work (`FUN_0216435C`).
pub const RAIL_MARKER: u32 = 0xEFEF_EFEF;

/// Address of [`DIR_ROM`] in the ROM (overlay 10 data).
pub const DIR_ROM_BASE: u32 = 0x0217_1458;
/// ROM `0x02171458..0x021714F0`: the direction tables, read by address as the original does (indices past a table
/// read the next one): `+0` u16 angle offsets per direction (0x02171458), `+8` u8 sector -> direction
/// (0x02171460, 16), `+0x18` u32 dz (0x02171470: -1, 1, 0, 0), `+0x28` u32 dx (0x02171480: 0, 0, -1, 1), `+0x38` /
/// `+0x48` the attribute-predicate tables of [`attr_blocked`] (0x02171490 / 0x021714A0), `+0x58` u32 opposite
/// direction (0x021714B0: 1, 0, 3, 2), `+0x68` u8 rotated direction (0x021714C0, 4 per direction), `+0x78` the
/// draw offset of [`set_draw_offset`] (0x021714D0: 0, -0x7000, 0), then the first words of the field-menu table.
pub const DIR_ROM: [u8; 0x98] = [
    0x00, 0x80, 0x00, 0x00, 0x00, 0x40, 0x00, 0xC0, 0x00, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x02,
    0x02, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x00, 0x00, 0x00, 0x75, 0xB4, 0x1A, 0x02, 0x59, 0xB4, 0x1A, 0x02,
    0xA5, 0xB4, 0x1A, 0x02, 0x91, 0xB4, 0x1A, 0x02, 0x59, 0xB4, 0x1A, 0x02, 0x75, 0xB4, 0x1A, 0x02,
    0x91, 0xB4, 0x1A, 0x02, 0xA5, 0xB4, 0x1A, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x03, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x03, 0x02, 0x00, 0x01,
    0x01, 0x00, 0x03, 0x02, 0x02, 0x03, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x90, 0xFF, 0xFF,
    0x00, 0x00, 0x00, 0x00, 0x09, 0x45, 0x16, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x65, 0x45, 0x16, 0x02, 0x8D, 0x45, 0x16, 0x02,
];

/// A byte of [`DIR_ROM`] at its ROM address. UNVERIFIED: an address outside the block (an index far past a table)
/// panics; the original reads whatever ROM follows.
fn rom8(addr: u32) -> u32 {
    DIR_ROM[addr.wrapping_sub(DIR_ROM_BASE) as usize] as u32
}

fn rom16(addr: u32) -> u32 {
    rom8(addr) | rom8(addr.wrapping_add(1)) << 8
}

fn rom32(addr: u32) -> u32 {
    rom16(addr) | rom16(addr.wrapping_add(2)) << 16
}

// fldmmdl (F4a; force stubs in this area's spec)
const SYS_FLAG: u32 = 0x0216_D4D4; // MMDLSYS +0 & mask
const SYS_MAP: u32 = 0x0216_D52C; // MMDLSYS +0x34 (field map)
const SYS_3C: u32 = 0x0216_D534; // MMDLSYS +0x3C
const STATUS_SET: u32 = 0x0216_D548; // MMDL +0 |= v
const STATUS_TEST: u32 = 0x0216_D55C; // MMDL +0 & v
const MOVE_BITS: u32 = 0x0216_D564; // MMDL +4
const BITS_SET: u32 = 0x0216_D568; // MMDL +4 |= v
const BITS_CLEAR: u32 = 0x0216_D570; // MMDL +4 &= ~v
const BITS_TEST: u32 = 0x0216_D57C; // MMDL +4 & v
const MOVE_CODE: u32 = 0x0216_D5A0; // MMDL move code
const MOVE_DIR: u32 = 0x0216_D608; // MMDL move dir
const LIMIT_X: u32 = 0x0216_D660; // MMDL move limit x
const LIMIT_Z: u32 = 0x0216_D66C; // MMDL move limit z
const SYS_OF: u32 = 0x0216_D67C; // MMDL -> MMDLSYS
const MOVE_WORK_CLEAR: u32 = 0x0216_D684; // MMDL move work clear (m, n)
const MOVE_WORK: u32 = 0x0216_D69C; // MMDL move work (m + 0x94)
const MOVE_PROC_INIT: u32 = 0x0216_D6F4; // MMDL move proc init
const MOVE_PROC: u32 = 0x0216_D708; // MMDL move proc
const SET_ATTR: u32 = 0x0216_D7D8; // MMDL +0x74 = v
const ATTR: u32 = 0x0216_D7DC; // MMDL +0x74 (attribute)
const SET_PREV_ATTR: u32 = 0x0216_D7E0; // MMDL +0x78 = v
const PREV_ATTR: u32 = 0x0216_D7E4; // MMDL +0x78 (previous attribute)
const INIT_GX: u32 = 0x0216_D7E8; // MMDL initial gx
const INIT_GZ: u32 = 0x0216_D7F8; // MMDL initial gz
const OLD_GX: u32 = 0x0216_D804;
const SET_OLD_GX: u32 = 0x0216_D80C;
const OLD_GY: u32 = 0x0216_D810;
const SET_OLD_GY: u32 = 0x0216_D818;
const OLD_GZ: u32 = 0x0216_D81C;
const SET_OLD_GZ: u32 = 0x0216_D824;
const GX: u32 = 0x0216_D83C;
const ADD_GX: u32 = 0x0216_D848;
const GY: u32 = 0x0216_D854;
const SET_GY: u32 = 0x0216_D85C;
const GZ: u32 = 0x0216_D86C;
const ADD_GZ: u32 = 0x0216_D87C;
const GET_POS: u32 = 0x0216_D88C; // MMDL position VecFx32 -> out
const SET_POS: u32 = 0x0216_D89C; // MMDL position VecFx32 = *v
const POS_Y: u32 = 0x0216_D8AC; // MMDL position y (fx32)
const SET_DRAW_OFS: u32 = 0x0216_D900; // MMDL +0x68 VecFx32 = *v
const SIZE_X: u32 = 0x0216_D91C; // MMDL size x
const SIZE_Z: u32 = 0x0216_D924; // MMDL size z
const SYS_DRAW: u32 = 0x0216_D98C; // MMDLSYS draw flag
const SYS_PAUSED: u32 = 0x0216_DA20; // MMDLSYS flag of an MMDL
const BIT_10: u32 = 0x0216_DA5C; // MMDL +4 & 0x10
const SET_BIT_20: u32 = 0x0216_DA70; // MMDL +4 |= 0x20
const PAUSED: u32 = 0x0216_DB10; // MMDL +4 & 8
const DRAW_READY: u32 = 0x0216_DB24; // MMDL draw ready
const STATUS_200: u32 = 0x0216_DB68; // MMDL +0 & 0x200
const SET_BIT_40000: u32 = 0x0216_DBCC; // MMDL +4 bit 0x40000 = on
const BIT_40000: u32 = 0x0216_DBE8; // MMDL +4 & 0x40000
const STATUS_1000: u32 = 0x0216_DC00; // MMDL +0 & 0x1000
const SET_BIT_20000: u32 = 0x0216_DC18; // MMDL +4 bit 0x20000 = on
const BIT_20000: u32 = 0x0216_DC34; // MMDL +4 & 0x20000
const STATUS_400: u32 = 0x0216_DCB4; // MMDL +0 & 0x400
const BIT_200000: u32 = 0x0216_DD04; // MMDL +4 & 0x200000
const NEXT_IN_USE: u32 = 0x0216_DD1C; // MMDL iterator (sys, &m, &i)
const PLAYER: u32 = 0x0216_DEE0; // MMDLSYS player MMDL
const OBJ_DATA: u32 = 0x0216_E3C4; // MMDL OBJ data (m + 0xE4)
                                   // other units
const RAIL_KEY: u32 = 0x0201_441C; // dir -> rail key
const FX_CTRL: u32 = 0x0218_8CD0; // field effect control of a field system
const MAP_QUERY: u32 = 0x0218_DAEC; // map height / attribute query (fsys, &pos, &out)
const MAP_HIT: u32 = 0x0218_DC04; // map bound / hit test (fsys, &pos)
const ACMD_PROC: u32 = 0x0219_7EB4; // fldmmdl_acmd: acmd proc
const ACMD_INIT: u32 = 0x0219_B414; // fldmmdl_acmd: move init
const ACMD_BUSY: u32 = 0x0219_B41C; // fldmmdl_acmd: move busy
const RAIL_SET: u32 = 0x0219_E2C0; // rail: set MMDL rail location (m, &loc)
const ATTR_VALUE: u32 = 0x021A_B0F0; // attribute value of an attribute
const ATTR_FLAGS: u32 = 0x021A_B0F8; // attribute flags of an attribute
const SEASON: u32 = 0x021A_BE28; // season / weather byte
const FX_AC2C0: u32 = 0x021A_C2C0; // field effect (m, ctrl)
const FX_AC528: u32 = 0x021A_C528; // field effect (m, ctrl)
const FX_AC760: u32 = 0x021A_C760; // field effect (ctrl, m, flag, kind)
const FX_BBDD4: u32 = 0x021B_BDD4; // field effect (m, ctrl, kind)
const FX_BBFB8: u32 = 0x021B_BFB8; // field effect (sys, m, ctrl, kind)
const FX_C484C: u32 = 0x021C_484C; // field effect (ctrl, x, z, y)
const FX_C4A10: u32 = 0x021C_4A10; // field effect (ctrl, m)
const FX_C4BE0: u32 = 0x021C_4BE0; // field effect (ctrl, m)
const FX_C8F7C: u32 = 0x021C_8F7C; // field effect (m, ctrl)
                                   // map attribute predicates (overlay 21), named by address
const P_11C: u32 = 0x021A_B11C;
const P_250: u32 = 0x021A_B250;
const P_264: u32 = 0x021A_B264;
const P_270: u32 = 0x021A_B270;
const P_2F8: u32 = 0x021A_B2F8;
const P_304: u32 = 0x021A_B304;
const P_328: u32 = 0x021A_B328;
const P_334: u32 = 0x021A_B334;
const P_358: u32 = 0x021A_B358;
const P_364: u32 = 0x021A_B364;
const P_3D4: u32 = 0x021A_B3D4;
const P_3E0: u32 = 0x021A_B3E0;
const P_3EC: u32 = 0x021A_B3EC;
const P_41C: u32 = 0x021A_B41C;
const P_428: u32 = 0x021A_B428;
const P_434: u32 = 0x021A_B434;
const P_440: u32 = 0x021A_B440;
const P_4EC: u32 = 0x021A_B4EC;
const P_4F8: u32 = 0x021A_B4F8;

fn c1<E: FieldEnv>(env: &mut E, addr: u32, a: u32) -> u32 {
    env.callv(addr, &[a])
}

fn c2<E: FieldEnv>(env: &mut E, addr: u32, a: u32, b: u32) -> u32 {
    env.callv(addr, &[a, b])
}

/// `(s16)((h >> 4) / 4096)` (toward zero): the grid coordinate of an fx32 value, sign-extended as the register
/// holds it.
fn grid_of(h: u32) -> u32 {
    let t = (h as i32) >> 4;
    let q = (t + if t < 0 { 0xFFF } else { 0 }) >> 12;
    q as i16 as i32 as u32
}

/// `|(s16) v|` as the original computes it (`lsls / asrs` to 16 bits, `rsbs` and again 16 bits).
fn abs16(v: u32) -> i32 {
    let s = v as i16;
    if s < 0 {
        s.wrapping_neg() as i32
    } else {
        s as i32
    }
}

/// C: `FUN_02163010` (unlabeled routine at `0x02163010`): the grid move init proc: `FUN_0216D6F4(m)` (move proc
/// init), `FUN_0219B414(m)` (fldmmdl_acmd), status |= 2.
pub fn move_init<E: FieldEnv>(env: &mut E, m: u32) {
    c1(env, MOVE_PROC_INIT, m);
    c1(env, ACMD_INIT, m);
    c2(env, STATUS_SET, m, 2);
}

/// C: `FUN_02163028` (unlabeled routine at `0x02163028`): the grid move proc: nothing while the MMDLSYS flag 2 is
/// set for `m` (`FUN_0216DA20`); else [`height_update`], [`attr_update`], [`step_start_effects`]; with move bit
/// 0x200 (acmd set) `FUN_02197EB4(m)`, else, not paused, [`can_move`] and `FUN_0219B41C(m) == 0`: the move proc
/// `FUN_0216D708(m)`; then [`step_end_effects`] and [`step_end_effects_2`].
pub fn move_proc<E: FieldEnv>(env: &mut E, m: u32) {
    if c2(env, SYS_PAUSED, m, 2) != 0 {
        return;
    }
    height_update(env, m);
    attr_update(env, m);
    step_start_effects(env, m);
    if c2(env, BITS_TEST, m, 0x200) != 0 {
        c1(env, ACMD_PROC, m);
    } else if c1(env, PAUSED, m) == 0 && can_move(env, m) == 1 && c1(env, ACMD_BUSY, m) == 0 {
        c1(env, MOVE_PROC, m);
    }
    step_end_effects(env, m);
    step_end_effects_2(env, m);
}

/// C: `FUN_02163090` (unlabeled routine at `0x02163090`): may the move proc run? 1 when move bit 0x10 is set, no
/// bit of 0x1800 is set or the move code is 0x32; else 0 when bit 0x1000 is set and status 0x200 clear, or bit
/// 0x800 is set and status 0x400 clear; else 1.
pub fn can_move<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    if c1(env, BIT_10, m) != 0 {
        return 1;
    }
    if c2(env, BITS_TEST, m, 0x1800) == 0 {
        return 1;
    }
    if c1(env, MOVE_CODE, m) == 0x32 {
        return 1;
    }
    let bits = c1(env, MOVE_BITS, m);
    if bits & 0x1000 != 0 && c1(env, STATUS_200, m) == 0 {
        return 0;
    }
    if bits & 0x800 != 0 && c1(env, STATUS_400, m) == 0 {
        return 0;
    }
    1
}

/// C: `FUN_021630F4` (unlabeled routine at `0x021630F4`): with move bit 0x1000, [`height_from_map`].
pub fn height_update<E: FieldEnv>(env: &mut E, m: u32) {
    if c2(env, BITS_TEST, m, 0x1000) != 0 {
        height_from_map(env, m);
    }
}

/// C: `FUN_0216310C` (unlabeled routine at `0x0216310C`): with move bit 0x800 and [`attr_from_map`] == 1, move bit
/// 0x20 (`FUN_0216DA70`).
pub fn attr_update<E: FieldEnv>(env: &mut E, m: u32) {
    if c2(env, BITS_TEST, m, 0x800) != 0 && attr_from_map(env, m) == 1 {
        c1(env, SET_BIT_20, m);
    }
}

/// C: `FUN_02163130` (unlabeled routine at `0x02163130`): with move bit 0x20: [`step_start_move`] and
/// [`effects_step_start`]; then clear move bits 0xA0.
pub fn step_start_effects<E: FieldEnv>(env: &mut E, m: u32) {
    if c2(env, BITS_TEST, m, 0x20) != 0 {
        step_start_move(env, m);
        effects_step_start(env, m);
    }
    c2(env, BITS_CLEAR, m, 0xA0);
}

/// C: `FUN_02163154` (unlabeled routine at `0x02163154`): move bit 0x80: [`effects_jump_end`]; else bit 0x20:
/// [`effects_step_end`]; then clear move bits 0xA0.
pub fn step_end_effects<E: FieldEnv>(env: &mut E, m: u32) {
    if c2(env, BITS_TEST, m, 0x80) != 0 {
        effects_jump_end(env, m);
    } else if c2(env, BITS_TEST, m, 0x20) != 0 {
        effects_step_end(env, m);
    }
    c2(env, BITS_CLEAR, m, 0xA0);
}

/// C: `FUN_02163188` (unlabeled routine at `0x02163188`): move bit 0x100: [`effects_step_end_2`]; else bit 0x40:
/// [`effects_step_start_2`]; then clear move bits 0x140.
pub fn step_end_effects_2<E: FieldEnv>(env: &mut E, m: u32) {
    if c2(env, BITS_TEST, m, 0x100) != 0 {
        effects_step_end_2(env, m);
    } else if c2(env, BITS_TEST, m, 0x40) != 0 {
        effects_step_start_2(env, m);
    }
    c2(env, BITS_CLEAR, m, 0x140);
}

/// C: `FUN_021631C0` (unlabeled routine at `0x021631C0`): fill the ATTR INFO at `info` (cleared first): previous
/// attribute and attribute (`FUN_0216D7E4` / `FUN_0216D7DC`), for each that is not -1 its value (`FUN_021AB0F0`) and
/// flags (`FUN_021AB0F8`), the OBJ DATA (`FUN_0216E3C4`), the field-effect control
/// (`FUN_02188CD0(FUN_0216D534(MMDLSYS))`) and the season byte (`FUN_021ABE28(control)`).
pub fn attr_info<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    env.mem().fill(info, ATTR_INFO_SIZE, 0);
    let prev = c1(env, PREV_ATTR, m);
    env.w32(info, prev);
    let a = c1(env, ATTR, m);
    env.w32(info.wrapping_add(4), a);
    if a != u32::MAX {
        let v = c1(env, ATTR_VALUE, a);
        env.w16(info.wrapping_add(0xA), v as u16);
        let a = env.r32(info.wrapping_add(4));
        let f = c1(env, ATTR_FLAGS, a);
        env.w16(info.wrapping_add(0xE), f as u16);
    }
    let p = env.r32(info);
    if p != u32::MAX {
        let v = c1(env, ATTR_VALUE, p);
        env.w16(info.wrapping_add(8), v as u16);
        let p = env.r32(info);
        let f = c1(env, ATTR_FLAGS, p);
        env.w16(info.wrapping_add(0xC), f as u16);
    }
    let od = c1(env, OBJ_DATA, m);
    env.w32(info.wrapping_add(0x10), od);
    let sys = c1(env, SYS_OF, m);
    let x = c1(env, SYS_3C, sys);
    let ctrl = c1(env, FX_CTRL, x);
    env.w32(info.wrapping_add(0x14), ctrl);
    let s = c1(env, SEASON, ctrl);
    env.w8(info.wrapping_add(0x18), s as u8);
}

/// The common head of the five effect passes: [`attr_from_map`], then, with move bit 0x800 clear and the MMDL
/// draw-ready (`FUN_0216DB24 == 1`), the ATTR INFO on a local and `passes` run on it in order.
fn effect_pass<E: FieldEnv>(env: &mut E, m: u32, passes: &[fn(&mut E, u32, u32)]) {
    attr_from_map(env, m);
    if c2(env, BITS_TEST, m, 0x800) != 0 || c1(env, DRAW_READY, m) != 1 {
        return;
    }
    let info = env.mem().push_local(ATTR_INFO_SIZE);
    attr_info(env, m, info);
    for p in passes {
        p(env, m, info);
    }
    env.mem().pop_local(info);
}

/// C: `FUN_02163230` (unlabeled routine at `0x02163230`): the attribute effects of a step start:
/// [`fx_effect_start`], [`fx_c4be0`], [`fx_shadow`], [`set_draw_offset`], [`fx_bbfb8_on`].
pub fn effects_step_start<E: FieldEnv>(env: &mut E, m: u32) {
    effect_pass(
        env,
        m,
        &[
            fx_effect_start,
            fx_c4be0,
            fx_shadow,
            set_draw_offset,
            fx_bbfb8_on,
        ],
    );
}

/// C: `FUN_02163288` (unlabeled routine at `0x02163288`): the attribute effects of a step end:
/// [`fx_effect_moving`], [`fx_bbdd4`], [`fx_c4be0_or_c4a10`], [`fx_shadow`], [`fx_c484c_old`], [`fx_bbfb8_on`],
/// [`nothing`].
pub fn effects_step_end<E: FieldEnv>(env: &mut E, m: u32) {
    effect_pass(
        env,
        m,
        &[
            fx_effect_moving,
            fx_bbdd4,
            fx_c4be0_or_c4a10,
            fx_shadow,
            fx_c484c_old,
            fx_bbfb8_on,
            nothing,
        ],
    );
}

/// C: `FUN_021632F0` (unlabeled routine at `0x021632F0`): the attribute effects of a jump end: [`fx_shadow`],
/// [`fx_bbfb8_on`], [`clear_bit_40000`], [`nothing`].
pub fn effects_jump_end<E: FieldEnv>(env: &mut E, m: u32) {
    effect_pass(env, m, &[fx_shadow, fx_bbfb8_on, clear_bit_40000, nothing]);
}

/// C: `FUN_02163340` (unlabeled routine at `0x02163340`): the attribute effects of a step start (second set):
/// [`set_draw_offset`], [`fx_effect_still`], [`fx_c484c`], [`fx_c4be0_or_c4a10`], [`fx_bbfb8_off`],
/// [`shadow_bit`], [`fx_c8f7c`].
pub fn effects_step_start_2<E: FieldEnv>(env: &mut E, m: u32) {
    effect_pass(
        env,
        m,
        &[
            set_draw_offset,
            fx_effect_still,
            fx_c484c,
            fx_c4be0_or_c4a10,
            fx_bbfb8_off,
            shadow_bit,
            fx_c8f7c,
        ],
    );
}

/// C: `FUN_021633A8` (unlabeled routine at `0x021633A8`): the attribute effects of a step end (second set):
/// [`set_draw_offset`], [`fx_c484c`], [`fx_c4be0_or_c4a10`], [`fx_bbfb8_off`], [`shadow_bit`], [`fx_effect_end`],
/// [`fx_ac528`], [`fx_c8f7c`].
pub fn effects_step_end_2<E: FieldEnv>(env: &mut E, m: u32) {
    effect_pass(
        env,
        m,
        &[
            set_draw_offset,
            fx_c484c,
            fx_c4be0_or_c4a10,
            fx_bbfb8_off,
            shadow_bit,
            fx_effect_end,
            fx_ac528,
            fx_c8f7c,
        ],
    );
}

/// C: `FUN_02163418` (unlabeled routine at `0x02163418`): the draw offset (`FUN_0216D900`): (0, -0x7000, 0) (ROM
/// `0x021714D0`) when status 0x1000 is clear and `FUN_021AB41C(value) != 0`, else (0, 0, 0).
pub fn set_draw_offset<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    let mut v = [0u8; 12];
    if c1(env, STATUS_1000, m) == 0 {
        let a = env.r16(info.wrapping_add(0xA)) as u32;
        if c1(env, P_41C, a) != 0 {
            for (i, b) in v.iter_mut().enumerate() {
                *b = rom8(0x0217_14D0 + i as u32) as u8;
            }
        }
    }
    env.call(SET_DRAW_OFS, &mut [CArg::V(m), CArg::InBuf(&v)]);
}

/// C: `FUN_02163464` (unlabeled routine at `0x02163464`): the effect kind of an attribute value (predicates in this
/// order): `021AB4EC` 6, `021AB4F8` 7, `021AB434` 5 (season 3) or 1, `021AB428` 4 (season 3) or 0, `021AB250` 1,
/// `021AB264` 2, `021AB270` 3, else 0.
pub fn effect_kind<E: FieldEnv>(env: &mut E, attr: u32, season: u32) -> u32 {
    if c1(env, P_4EC, attr) != 0 {
        return 6;
    }
    if c1(env, P_4F8, attr) != 0 {
        return 7;
    }
    if c1(env, P_434, attr) != 0 {
        return if season == 3 { 5 } else { 1 };
    }
    if c1(env, P_428, attr) != 0 {
        return if season == 3 { 4 } else { 0 };
    }
    if c1(env, P_250, attr) != 0 {
        return 1;
    }
    if c1(env, P_264, attr) != 0 {
        return 2;
    }
    if c1(env, P_270, attr) != 0 {
        return 3;
    }
    0
}

fn info_kind<E: FieldEnv>(env: &mut E, info: u32) -> u32 {
    let a = env.r16(info.wrapping_add(0xA)) as u32;
    let s = env.r8(info.wrapping_add(0x18)) as u32;
    effect_kind(env, a, s)
}

/// C: `FUN_021634D8` (unlabeled routine at `0x021634D8`): with flag 0x20 of the attribute, `FUN_021AC760(control, m,
/// 0, kind)` ([`effect_kind`]).
pub fn fx_effect_start<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    if env.r16(info.wrapping_add(0xE)) & 0x20 == 0 {
        return;
    }
    let k = info_kind(env, info);
    let ctrl = env.r32(info.wrapping_add(0x14));
    env.callv(FX_AC760, &[ctrl, m, 0, k]);
}

/// C: `FUN_021634FC` (unlabeled routine at `0x021634FC`): with flag 0x20, `FUN_021AC760(control, m, 1, kind)` when
/// the move direction (`FUN_0216D608`, read first) is nonzero.
pub fn fx_effect_moving<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    if env.r16(info.wrapping_add(0xE)) & 0x20 == 0 {
        return;
    }
    let d = c1(env, MOVE_DIR, m);
    let k = info_kind(env, info);
    if d != 0 {
        let ctrl = env.r32(info.wrapping_add(0x14));
        env.callv(FX_AC760, &[ctrl, m, 1, k]);
    }
}

/// C: `FUN_0216352C` (unlabeled routine at `0x0216352C`): as [`fx_effect_moving`] when the move direction is 0.
pub fn fx_effect_still<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    if env.r16(info.wrapping_add(0xE)) & 0x20 == 0 {
        return;
    }
    let d = c1(env, MOVE_DIR, m);
    let k = info_kind(env, info);
    if d == 0 {
        let ctrl = env.r32(info.wrapping_add(0x14));
        env.callv(FX_AC760, &[ctrl, m, 1, k]);
    }
}

/// C: `FUN_0216355C` (unlabeled routine at `0x0216355C`): with flag 0x20, `FUN_021AC760(control, m, 1, kind)`.
pub fn fx_effect_end<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    if env.r16(info.wrapping_add(0xE)) & 0x20 == 0 {
        return;
    }
    let k = info_kind(env, info);
    let ctrl = env.r32(info.wrapping_add(0x14));
    env.callv(FX_AC760, &[ctrl, m, 1, k]);
}

/// C: `FUN_02163580` (unlabeled routine at `0x02163580`): with flag 8 of the previous attribute, the kind `k` from
/// its value: `021AB3D4`: 2 / 3 (OBJ DATA `+5 == 2`) in season 3, else none; `021AB3E0`: 2 / 3 in season 0 or 3,
/// else none; `021AB334` or `021AB328` (== 1): 2 / 3; `021AB41C` (== 1): 4; else 1 (`+5 == 2`) or 0; then
/// `FUN_021BBDD4(m, control, k)`.
pub fn fx_bbdd4<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    if env.r16(info.wrapping_add(0xC)) & 8 == 0 {
        return;
    }
    let obj5 = |env: &mut E| {
        let od = env.r32(info.wrapping_add(0x10));
        env.r8(od.wrapping_add(5))
    };
    let v = env.r16(info.wrapping_add(8)) as u32;
    let season = |env: &mut E| env.r8(info.wrapping_add(0x18));
    let k = if c1(env, P_3D4, v) != 0 {
        if season(env) != 3 {
            5
        } else if obj5(env) == 2 {
            3
        } else {
            2
        }
    } else if c1(env, P_3E0, v) != 0 {
        let s = season(env);
        if s != 0 && s != 3 {
            5
        } else if obj5(env) == 2 {
            3
        } else {
            2
        }
    } else if c1(env, P_334, v) == 1 || {
        let v = env.r16(info.wrapping_add(8)) as u32;
        c1(env, P_328, v) == 1
    } {
        if obj5(env) == 2 {
            3
        } else {
            2
        }
    } else {
        let v = env.r16(info.wrapping_add(8)) as u32;
        if c1(env, P_41C, v) == 1 {
            4
        } else if obj5(env) == 2 {
            1
        } else {
            0
        }
    };
    if k != 5 {
        let ctrl = env.r32(info.wrapping_add(0x14));
        env.callv(FX_BBDD4, &[m, ctrl, k]);
    }
}

/// C: `FUN_02163618` (unlabeled routine at `0x02163618`): `FUN_021AB304(value) == 1`: when move bit 0x40000 is
/// clear, `FUN_021C4BE0(control, m)` and set it; otherwise clear move bit 0x40000.
pub fn fx_c4be0<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    let a = env.r16(info.wrapping_add(0xA)) as u32;
    if c1(env, P_304, a) == 1 {
        if c1(env, BIT_40000, m) == 0 {
            let ctrl = env.r32(info.wrapping_add(0x14));
            c2(env, FX_C4BE0, ctrl, m);
            c2(env, SET_BIT_40000, m, 1);
        }
        return;
    }
    c2(env, SET_BIT_40000, m, 0);
}

/// C: `FUN_02163650` (unlabeled routine at `0x02163650`): as [`fx_c4be0`], but when `FUN_021AB304` does not match
/// and `FUN_021AB364(value) == 1`, `FUN_021C4A10(control, m)` instead of clearing the bit.
pub fn fx_c4be0_or_c4a10<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    let a = env.r16(info.wrapping_add(0xA)) as u32;
    if c1(env, P_304, a) == 1 {
        if c1(env, BIT_40000, m) == 0 {
            let ctrl = env.r32(info.wrapping_add(0x14));
            c2(env, FX_C4BE0, ctrl, m);
            c2(env, SET_BIT_40000, m, 1);
        }
        return;
    }
    let a = env.r16(info.wrapping_add(0xA)) as u32;
    if c1(env, P_364, a) == 1 {
        let ctrl = env.r32(info.wrapping_add(0x14));
        c2(env, FX_C4A10, ctrl, m);
        return;
    }
    c2(env, SET_BIT_40000, m, 0);
}

/// C: `FUN_0216369C` (unlabeled routine at `0x0216369C`): clear move bit 0x40000 (`FUN_0216DBCC(m, 0)`).
pub fn clear_bit_40000<E: FieldEnv>(env: &mut E, m: u32, _info: u32) {
    c2(env, SET_BIT_40000, m, 0);
}

/// C: `FUN_021636A8` (unlabeled routine at `0x021636A8`): does the MMDL draw a shadow: the MMDLSYS draw flag
/// (`FUN_0216D98C`) and OBJ DATA `+4` nonzero?
pub fn has_shadow<E: FieldEnv>(env: &mut E, m: u32, info: u32) -> u32 {
    let sys = c1(env, SYS_OF, m);
    if c1(env, SYS_DRAW, sys) == 0 {
        return 0;
    }
    let od = env.r32(info.wrapping_add(0x10));
    (env.r8(od.wrapping_add(4)) != 0) as u32
}

/// C: `FUN_021636C8` (unlabeled routine at `0x021636C8`): may the attribute show a shadow: flag 0x80 set, and not
/// (season 3 and `FUN_021AB3D4(value) != 0`)?
pub fn attr_shadow<E: FieldEnv>(env: &mut E, info: u32) -> u32 {
    if env.r16(info.wrapping_add(0xE)) & 0x80 == 0 {
        return 0;
    }
    if env.r8(info.wrapping_add(0x18)) == 3 {
        let a = env.r16(info.wrapping_add(0xA)) as u32;
        if c1(env, P_3D4, a) != 0 {
            return 0;
        }
    }
    1
}

/// C: `FUN_021636F0` (unlabeled routine at `0x021636F0`): with [`has_shadow`] and [`attr_shadow`] and move bit
/// 0x4000 clear: `FUN_021AC2C0(m, control)` and set bit 0x4000.
pub fn fx_shadow<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    if has_shadow(env, m, info) != 1 || attr_shadow(env, info) != 1 {
        return;
    }
    if c2(env, BITS_TEST, m, 0x4000) != 0 {
        return;
    }
    let ctrl = env.r32(info.wrapping_add(0x14));
    c2(env, FX_AC2C0, m, ctrl);
    c2(env, BITS_SET, m, 0x4000);
}

/// C: `FUN_0216372C` (unlabeled routine at `0x0216372C`): with [`has_shadow`]: move bit 0x8000 set when
/// [`attr_shadow`] is 0, else cleared.
pub fn shadow_bit<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    if has_shadow(env, m, info) != 1 {
        return;
    }
    if attr_shadow(env, info) == 0 {
        c2(env, BITS_SET, m, 0x8000);
    } else {
        c2(env, BITS_CLEAR, m, 0x8000);
    }
}

/// C: `FUN_0216375C` (unlabeled routine at `0x0216375C`): with flag 2 of the attribute clear and move bit 0x200000
/// clear, `FUN_021AC528(m, control)`.
pub fn fx_ac528<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    if env.r16(info.wrapping_add(0xE)) & 2 != 0 {
        return;
    }
    if c1(env, BIT_200000, m) != 0 {
        return;
    }
    let ctrl = env.r32(info.wrapping_add(0x14));
    c2(env, FX_AC528, m, ctrl);
}

/// C: `FUN_0216377C` (unlabeled routine at `0x0216377C`): `FUN_021AB2F8(previous value) == 1`:
/// `FUN_021C484C(control, old gx, old gz, position y)`.
pub fn fx_c484c_old<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    let a = env.r16(info.wrapping_add(8)) as u32;
    if c1(env, P_2F8, a) != 1 {
        return;
    }
    let x = c1(env, OLD_GX, m);
    let z = c1(env, OLD_GZ, m);
    let y = c1(env, POS_Y, m);
    let ctrl = env.r32(info.wrapping_add(0x14));
    env.callv(FX_C484C, &[ctrl, x, z, y]);
}

/// C: `FUN_021637B0` (unlabeled routine at `0x021637B0`): `FUN_021AB2F8(value) == 1`: `FUN_021C484C(control, gx,
/// gz, position y)`.
pub fn fx_c484c<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    let a = env.r16(info.wrapping_add(0xA)) as u32;
    if c1(env, P_2F8, a) != 1 {
        return;
    }
    let x = c1(env, GX, m);
    let z = c1(env, GZ, m);
    let y = c1(env, POS_Y, m);
    let ctrl = env.r32(info.wrapping_add(0x14));
    env.callv(FX_C484C, &[ctrl, x, z, y]);
}

/// C: `FUN_021637E4` (unlabeled routine at `0x021637E4`): with OBJ DATA `+6` nonzero and move bit 0x20000 clear: the
/// attribute (flag 0x40) or else the one in front ([`attr_ahead`]`(m, 1)` when its flags have 0x40); when there is
/// one, `FUN_021BBFB8(MMDLSYS, m, control, k)` with `k` 2 when `FUN_021AB440` or `FUN_021AB358` of its value is 1,
/// else 1, and move bit 0x20000 set.
pub fn fx_bbfb8_on<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    let od = env.r32(info.wrapping_add(0x10));
    if env.r8(od.wrapping_add(6)) == 0 || c1(env, BIT_20000, m) != 0 {
        return;
    }
    let a = if env.r16(info.wrapping_add(0xE)) & 0x40 != 0 {
        env.r32(info.wrapping_add(4))
    } else {
        let b = attr_ahead(env, m, 1);
        if c1(env, ATTR_FLAGS, b) & 0x40 != 0 {
            b
        } else {
            u32::MAX
        }
    };
    if a == u32::MAX {
        return;
    }
    let v = c1(env, ATTR_VALUE, a);
    let sys = c1(env, SYS_OF, m);
    let k = if c1(env, P_440, v) == 1 || c1(env, P_358, v) == 1 {
        2
    } else {
        1
    };
    let ctrl = env.r32(info.wrapping_add(0x14));
    env.callv(FX_BBFB8, &[sys, m, ctrl, k]);
    c2(env, SET_BIT_20000, m, 1);
}

/// C: `FUN_02163868` (unlabeled routine at `0x02163868`): with OBJ DATA `+6` nonzero and move bit 0x20000 set: clear
/// it when the attribute in front ([`attr_ahead`]`(m, 1)`) is -1 or lacks flag 0x40.
pub fn fx_bbfb8_off<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    let od = env.r32(info.wrapping_add(0x10));
    if env.r8(od.wrapping_add(6)) == 0 || c1(env, BIT_20000, m) == 0 {
        return;
    }
    let b = attr_ahead(env, m, 1);
    if b == u32::MAX || c1(env, ATTR_FLAGS, b) & 0x40 == 0 {
        c2(env, SET_BIT_20000, m, 0);
    }
}

/// C: `FUN_021638AC` (unlabeled routine at `0x021638AC`): `FUN_021AB3EC(value) == 1`: `FUN_021C8F7C(m, control)`.
pub fn fx_c8f7c<E: FieldEnv>(env: &mut E, m: u32, info: u32) {
    let a = env.r16(info.wrapping_add(0xA)) as u32;
    if c1(env, P_3EC, a) == 1 {
        let ctrl = env.r32(info.wrapping_add(0x14));
        c2(env, FX_C8F7C, m, ctrl);
    }
}

/// C: `FUN_021638C8` (unlabeled routine at `0x021638C8`): does nothing.
pub fn nothing<E: FieldEnv>(_env: &mut E, _m: u32, _info: u32) {}

/// C: `FUN_021638CC` (unlabeled routine at `0x021638CC`): the hit bits of a step of `m` to the tile `(x, z)` (`z`
/// as s16) with `pos` its position: 1 outside the move limit ([`outside_limit`]), 2 blocked by attributes
/// ([`attr_blocked`] with the tile centre, y = `pos.y`), 8 no map height at the centre or `|pos.y - h| >= 0x14000`
/// (else y becomes the grid of `h`), 4 another MMDL there ([`mmdl_at`]), 0x10 the map hit test
/// (`FUN_0218DC04(field map, &centre)`).
#[allow(clippy::too_many_arguments)]
pub fn step_hits<E: FieldEnv>(
    env: &mut E,
    m: u32,
    pos: u32,
    x: u32,
    y: u32,
    z: u32,
    dir: u16,
) -> u32 {
    let mut hits = 0;
    let z16 = z as i16 as i32 as u32;
    let v = env.mem().push_local(12);
    env.w32(v, (x << 16).wrapping_add(0x8000));
    let py = env.r32(pos.wrapping_add(4));
    env.w32(v + 4, py);
    env.w32(v + 8, (z16 << 16).wrapping_add(0x8000));
    if outside_limit(env, m, x, z16) == 1 {
        hits |= 1;
    }
    let (vx, vy, vz) = (env.r32(v), env.r32(v + 4), env.r32(v + 8));
    if attr_blocked(env, m, dir as u32, vx, vy, vz) == 1 {
        hits |= 2;
    }
    let h = env.mem().push_local(4);
    let mut y = y;
    if height_at(env, m, v, h) == 1 {
        let hv = env.r32(h);
        let d = env.r32(pos.wrapping_add(4)).wrapping_sub(hv) as i32;
        if d.wrapping_abs() >= 0x14000 {
            hits |= 8;
        }
        y = grid_of(hv);
    } else {
        hits |= 8;
    }
    env.mem().pop_local(h);
    if mmdl_at(env, m, x, y, z16) == 1 {
        hits |= 4;
    }
    let sys = c1(env, SYS_OF, m);
    let fs = c1(env, SYS_MAP, sys);
    if env.call(MAP_HIT, &mut [CArg::V(fs), CArg::In(v, 12)]) == 1 {
        hits |= 0x10;
    }
    env.mem().pop_local(v);
    hits
}

/// C: `FUN_02163994` (unlabeled routine at `0x02163994`): [`step_hits`] of `m` at its position
/// (`FUN_0216D88C`).
pub fn step_hits_here<E: FieldEnv>(env: &mut E, m: u32, x: u32, y: u32, z: u32, dir: u16) -> u32 {
    let p = env.mem().push_local(12);
    env.call(GET_POS, &mut [CArg::V(m), CArg::Out(p, 12)]);
    let r = step_hits(env, m, p, x, y, z, dir);
    env.mem().pop_local(p);
    r
}

/// C: `FUN_021639C0` (unlabeled routine at `0x021639C0`): [`step_hits_here`] for the tile next to `m` in direction
/// `dir` (`gx + dx`, `gy`, `gz + dz`, s16).
pub fn step_hits_dir<E: FieldEnv>(env: &mut E, m: u32, dir: u32) -> u32 {
    let gx = c1(env, GX, m);
    let x = gx.wrapping_add(dir_dx(dir)) as i16 as i32 as u32;
    let gy = c1(env, GY, m);
    let gz = c1(env, GZ, m);
    let z = gz.wrapping_add(dir_dz(dir)) as i16 as i32 as u32;
    step_hits_here(env, m, x, gy, z, dir as u16)
}

/// C: `FUN_02163A08` (unlabeled routine at `0x02163A08`): the footprint rectangle of `m` at `(x, z)` (s16 at `out`):
/// `(x, z - t, x + s, z)` with `s = max(size x - 1, 0)` and `t = max(size z - 1, 0)` (s16).
pub fn footprint<E: FieldEnv>(env: &mut E, m: u32, x: u32, z: u32, out: u32) {
    let mut s = c1(env, SIZE_X, m).wrapping_sub(1) as i16 as i32;
    if s < 0 {
        s = 0;
    }
    env.w16(out.wrapping_add(4), x.wrapping_add(s as u32) as u16);
    env.w16(out, x as u16);
    let mut t = c1(env, SIZE_Z, m).wrapping_sub(1) as i16 as i32;
    if t < 0 {
        t = 0;
    }
    env.w16(out.wrapping_add(2), z.wrapping_sub(t as u32) as u16);
    env.w16(out.wrapping_add(6), z as u16);
}

/// One axis of [`rects_overlap`]: an end of the narrower span lies inside the wider one.
fn spans_overlap(a0: i32, a1: i32, b0: i32, b1: i32) -> bool {
    let wa = abs16(a1.wrapping_sub(a0) as u32);
    let wb = abs16(b1.wrapping_sub(b0) as u32);
    let (w0, w1, n0, n1) = if wa > wb {
        (a0, a1, b0, b1)
    } else {
        (b0, b1, a0, a1)
    };
    (w0 <= n0 && w1 >= n0) || (w0 <= n1 && w1 >= n1)
}

/// C: `FUN_02163A40` (unlabeled routine at `0x02163A40`): do the s16 rectangles `(x0, z0, x1, z1)` at `r` and `s`
/// overlap? On each axis one end of the narrower span (by `|end - start|`, 16-bit) must lie inside the wider one.
pub fn rects_overlap(mem: &Mem, r: u32, s: u32) -> bool {
    let g = |a: u32, o: u32| mem.r16(a.wrapping_add(o)) as i16 as i32;
    spans_overlap(g(r, 0), g(r, 4), g(s, 0), g(s, 4))
        && spans_overlap(g(r, 2), g(r, 6), g(s, 2), g(s, 6))
}

/// C: `FUN_02163AF0` (unlabeled routine at `0x02163AF0`): does the MMDL `o` block `m` at `(x, y, z)` (`z` as s16)?
/// Not when `o`'s status has 0x80; else when the footprint of `m` there overlaps `o`'s at its grid and `|y - o gy|
/// < 1` (16-bit), or (`all == 1`) at its old grid with its old gy.
#[allow(clippy::too_many_arguments)]
pub fn mmdl_blocks<E: FieldEnv>(
    env: &mut E,
    m: u32,
    o: u32,
    x: u32,
    y: u32,
    z: u32,
    all: u32,
) -> u32 {
    let r = env.mem().push_local(8);
    let s = env.mem().push_local(8);
    let res = 'f: {
        footprint(env, m, x, z as i16 as i32 as u32, r);
        if c2(env, STATUS_TEST, o, 0x80) != 0 {
            break 'f 0;
        }
        let gx = c1(env, GX, o);
        let gz = c1(env, GZ, o);
        footprint(env, o, gx, gz, s);
        if rects_overlap(env.mem(), r, s) {
            let gy = c1(env, GY, o);
            if abs16(y.wrapping_sub(gy)) < 1 {
                break 'f 1;
            }
        }
        if all != 1 {
            break 'f 0;
        }
        let ox = c1(env, OLD_GX, o);
        let oz = c1(env, OLD_GZ, o);
        footprint(env, o, ox, oz, s);
        if rects_overlap(env.mem(), r, s) {
            let oy = c1(env, OLD_GY, o);
            if abs16(y.wrapping_sub(oy)) < 1 {
                break 'f 1;
            }
        }
        0
    };
    env.mem().pop_local(s);
    env.mem().pop_local(r);
    res
}

/// C: `FUN_02163BAC` (unlabeled routine at `0x02163BAC`): is there another MMDL in use (the iterator
/// `FUN_0216DD1C(MMDLSYS, &o, &i)`) that blocks `m` at `(x, y, z)` ([`mmdl_blocks`] with `all = 1`)?
pub fn mmdl_at<E: FieldEnv>(env: &mut E, m: u32, x: u32, y: u32, z: u32) -> u32 {
    let sys = c1(env, SYS_OF, m);
    loop {
        let mut o = [0u8; 4];
        let more = env.call(
            NEXT_IN_USE,
            &mut [CArg::V(sys), CArg::OutBuf(&mut o), CArg::P(0)],
        );
        if more != 1 {
            return 0;
        }
        let o = u32::from_le_bytes(o);
        if m != o && mmdl_blocks(env, m, o, x, y, z, 1) == 1 {
            return 1;
        }
    }
}

/// C: `FUN_02163C04` (unlabeled routine at `0x02163C04`): is `(x, z)` inside the footprint of `m` at its grid, or
/// (`old == 1`) at its old grid? (signed compares with the whole registers)
pub fn in_footprint<E: FieldEnv>(env: &mut E, m: u32, x: u32, z: u32, old: u32) -> u32 {
    let r = env.mem().push_local(8);
    let inside = |env: &mut E| {
        let g = |env: &mut E, o: u32| env.r16(r + o) as i16 as i32;
        let (x, z) = (x as i32, z as i32);
        g(env, 0) <= x && g(env, 4) >= x && g(env, 2) <= z && g(env, 6) >= z
    };
    let gx = c1(env, GX, m);
    let gz = c1(env, GZ, m);
    footprint(env, m, gx, gz, r);
    let mut res = inside(env) as u32;
    if res == 0 && old == 1 {
        let ox = c1(env, OLD_GX, m);
        let oz = c1(env, OLD_GZ, m);
        footprint(env, m, ox, oz, r);
        res = inside(env) as u32;
    }
    env.mem().pop_local(r);
    res
}

/// C: `FUN_02163C9C` (unlabeled routine at `0x02163C9C`): is `(x, z)` outside the move limit of `m`: a limit that
/// is not -1 with `x` outside `initial gx -+ limit` (s16), or likewise for z?
pub fn outside_limit<E: FieldEnv>(env: &mut E, m: u32, x: u32, z: u32) -> u32 {
    for (init, lim, p) in [(INIT_GX, LIMIT_X, x), (INIT_GZ, LIMIT_Z, z)] {
        let i = c1(env, init, m);
        let l = c1(env, lim, m);
        if l != u32::MAX {
            let lo = i.wrapping_sub(l) as i16 as i32;
            let hi = i.wrapping_add(l) as i16 as i32;
            if lo > p as i32 || hi < p as i32 {
                return 1;
            }
        }
    }
    0
}

/// C: `FUN_02163D04` (unlabeled routine at `0x02163D04`): is the step of `m` in direction `dir` to the position `v`
/// (passed by value) blocked by attributes? 1 with status 0x400; else for each footprint cell (size x by size z;
/// x += 1 tile, z -= 1 tile) 1 when there is no attribute ([`attr_at`]) or `FUN_021AB11C(a) == 1`; then the
/// direction predicates of the current attribute's value (ROM `0x021714A0[dir]`) and of the first cell's (ROM
/// `0x02171490[dir]`). UNVERIFIED: with an empty footprint the first cell's attribute is an uninitialised stack word
/// in the original (0 in the differential tests, Z19); the port uses 0.
pub fn attr_blocked<E: FieldEnv>(env: &mut E, m: u32, dir: u32, vx: u32, vy: u32, vz: u32) -> u32 {
    if c1(env, STATUS_400, m) != 0 {
        return 1;
    }
    let nx = c1(env, SIZE_X, m);
    let nz = c1(env, SIZE_Z, m);
    let mut first = 0u32;
    let mut have = false;
    let cur = env.mem().push_local(12);
    let a = env.mem().push_local(4);
    let mut row_z = vz;
    let mut i: u32 = 0;
    let mut res = None;
    'rows: while i < nz {
        env.w32(cur, vx);
        env.w32(cur + 4, vy);
        env.w32(cur + 8, row_z);
        let mut j: u32 = 0;
        while j < nx {
            if attr_at(env, m, cur, a) == 0 {
                res = Some(1);
                break 'rows;
            }
            let av = env.r32(a);
            if c1(env, P_11C, av) == 1 {
                res = Some(1);
                break 'rows;
            }
            if !have {
                first = env.r32(a);
                have = true;
            }
            j = (j + 1) & 0xFF;
            let cx = env.r32(cur).wrapping_add(0x1_0000);
            env.w32(cur, cx);
        }
        i = (i + 1) & 0xFF;
        row_z = row_z.wrapping_sub(0x1_0000);
    }
    env.mem().pop_local(a);
    env.mem().pop_local(cur);
    if let Some(r) = res {
        return r;
    }
    let at = c1(env, ATTR, m);
    let v = c1(env, ATTR_VALUE, at);
    let f = rom32(0x0217_14A0 + 4 * dir);
    if c1(env, f & !1, v) == 1 {
        return 1;
    }
    let v = c1(env, ATTR_VALUE, first);
    let f = rom32(0x0217_1490 + 4 * dir);
    if c1(env, f & !1, v) == 1 {
        return 1;
    }
    0
}

/// C: `FUN_02163E1C` (unlabeled routine at `0x02163E1C`): the map query at `pos` (`FUN_0218DAEC(field map, pos,
/// out)`, 16 bytes: `+8` attribute, `+0xC` height); returns its answer.
pub fn map_query<E: FieldEnv>(env: &mut E, m: u32, pos: u32, out: u32) -> u32 {
    let sys = c1(env, SYS_OF, m);
    let fs = c1(env, SYS_MAP, sys);
    env.call(
        MAP_QUERY,
        &mut [CArg::V(fs), CArg::In(pos, 12), CArg::Out(out, 16)],
    )
}

/// The common part of [`attr_at`] / [`height_at`]: `*out = 0`; on a map answer of 1, `*out` = the word at `field`
/// of the query result, 1; else 0.
fn query_field<E: FieldEnv>(env: &mut E, m: u32, pos: u32, out: u32, field: u32) -> u32 {
    env.w32(out, 0);
    let b = env.mem().push_local(16);
    let r = if map_query(env, m, pos, b) == 1 {
        let v = env.r32(b + field);
        env.w32(out, v);
        1
    } else {
        0
    };
    env.mem().pop_local(b);
    r
}

/// C: `FUN_02163E34` (unlabeled routine at `0x02163E34`): the attribute at `pos` (`*out`, 0 first); 1 when found.
pub fn attr_at<E: FieldEnv>(env: &mut E, m: u32, pos: u32, out: u32) -> u32 {
    query_field(env, m, pos, out, 8)
}

/// C: `FUN_02163E58` (unlabeled routine at `0x02163E58`): the map height at `pos` (`*out`, 0 first); 1 when found.
pub fn height_at<E: FieldEnv>(env: &mut E, m: u32, pos: u32, out: u32) -> u32 {
    query_field(env, m, pos, out, 0xC)
}

/// C: `FUN_02163E7C` (unlabeled routine at `0x02163E7C`): step `m` one tile in direction `dir` on the grid: old
/// grid = grid, `gx += dx`, `gz += dz`, then gy = the grid of the map height at the new tile centre with y =
/// position y (0 when there is none).
pub fn grid_step<E: FieldEnv>(env: &mut E, m: u32, dir: u32) {
    save_old_grid(env, m);
    c2(env, ADD_GX, m, dir_dx(dir));
    c2(env, ADD_GZ, m, dir_dz(dir));
    let h = env.mem().push_local(4);
    let v = env.mem().push_local(12);
    let gx = c1(env, GX, m);
    let gz = c1(env, GZ, m);
    tile_centre(env.mem(), gx, gz, v);
    let y = c1(env, POS_Y, m);
    env.w32(v + 4, y);
    height_at(env, m, v, h);
    let hv = env.r32(h);
    env.mem().pop_local(v);
    env.mem().pop_local(h);
    c2(env, SET_GY, m, grid_of(hv));
}

/// C: `FUN_02163F10` (unlabeled routine at `0x02163F10`): old grid (`+0x36..`) = grid (`+0x3C..`).
pub fn save_old_grid<E: FieldEnv>(env: &mut E, m: u32) {
    let x = c1(env, GX, m);
    c2(env, SET_OLD_GX, m, x);
    let y = c1(env, GY, m);
    c2(env, SET_OLD_GY, m, y);
    let z = c1(env, GZ, m);
    c2(env, SET_OLD_GZ, m, z);
}

/// C: `FUN_02163F40` (unlabeled routine at `0x02163F40`): the attribute of the cell one tile ahead in direction
/// `dir` (grid centre with y = position y, moved by [`vec_step`]); 0 when the map has none ([`attr_at`] stores 0
/// over the initial -1).
pub fn attr_ahead<E: FieldEnv>(env: &mut E, m: u32, dir: u32) -> u32 {
    let a = env.mem().push_local(4);
    env.w32(a, u32::MAX);
    let v = env.mem().push_local(12);
    let gx = c1(env, GX, m);
    let gz = c1(env, GZ, m);
    tile_centre(env.mem(), gx, gz, v);
    let y = c1(env, POS_Y, m);
    env.w32(v + 4, y);
    vec_step(env.mem(), dir, v, 0x1_0000);
    attr_at(env, m, v, a);
    let r = env.r32(a);
    env.mem().pop_local(v);
    env.mem().pop_local(a);
    r
}

/// C: `FUN_02163F8C` (unlabeled routine at `0x02163F8C`): position of `m` += the `VecFx32` at `d`.
pub fn pos_add<E: FieldEnv>(env: &mut E, m: u32, d: u32) {
    let p = env.mem().push_local(12);
    env.call(GET_POS, &mut [CArg::V(m), CArg::Out(p, 12)]);
    for o in [0, 4, 8] {
        let v = env.r32(p + o).wrapping_add(env.r32(d.wrapping_add(o)));
        env.w32(p + o, v);
    }
    env.call(SET_POS, &mut [CArg::V(m), CArg::In(p, 12)]);
    env.mem().pop_local(p);
}

/// C: `FUN_02163FC0` (unlabeled routine at `0x02163FC0`): move the position of `m` by `n` in direction `dir` (0: z
/// -= n, 1: z += n, 2: x -= n, 3: x += n; other directions leave it).
pub fn pos_step<E: FieldEnv>(env: &mut E, m: u32, dir: u32, n: u32) {
    let p = env.mem().push_local(12);
    env.call(GET_POS, &mut [CArg::V(m), CArg::Out(p, 12)]);
    vec_step(env.mem(), dir, p, n);
    env.call(SET_POS, &mut [CArg::V(m), CArg::In(p, 12)]);
    env.mem().pop_local(p);
}

/// C: `FUN_02164010` (unlabeled routine at `0x02164010`): put `m` on the map height: 0 on a rail (status 0x2000);
/// with status 0x200 clear move bit 0x1000 and 0; else when the map has a height at its position: position y = h,
/// old gy = gy, gy = the grid of h, clear move bit 0x1000; else set it. Returns whether the height was found.
pub fn height_from_map<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let p = env.mem().push_local(12);
    env.call(GET_POS, &mut [CArg::V(m), CArg::Out(p, 12)]);
    let v = env.mem().push_local(12);
    let h = env.mem().push_local(4);
    env.mem().copy(v, p, 12);
    let r = 'f: {
        if c2(env, STATUS_TEST, m, 0x2000) != 0 {
            break 'f 0;
        }
        if c1(env, STATUS_200, m) == 1 {
            c2(env, BITS_CLEAR, m, 0x1000);
            break 'f 0;
        }
        let found = height_at(env, m, v, h);
        if found == 1 {
            let hv = env.r32(h);
            env.w32(p + 4, hv);
            env.call(SET_POS, &mut [CArg::V(m), CArg::In(p, 12)]);
            let gy = c1(env, GY, m);
            c2(env, SET_OLD_GY, m, gy);
            c2(env, SET_GY, m, grid_of(hv));
            c2(env, BITS_CLEAR, m, 0x1000);
        } else {
            c2(env, BITS_SET, m, 0x1000);
        }
        found
    };
    env.mem().pop_local(h);
    env.mem().pop_local(v);
    env.mem().pop_local(p);
    r
}

/// C: `FUN_021640B0` (unlabeled routine at `0x021640B0`): update the attributes of `m` from the map (not with status
/// 0x400): the attribute at the old grid (`<< 16`, no centre offset) becomes the previous attribute, the one at the
/// grid centre (y = position y) the attribute; when the latter is found clear move bit 0x800 and return 1, else set
/// it and return 0.
pub fn attr_from_map<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let b = env.mem().push_local(4);
    let a = env.mem().push_local(4);
    env.w32(a, u32::MAX);
    env.w32(b, u32::MAX);
    let (mut fa, mut fb) = (0, 0);
    if c1(env, STATUS_400, m) == 0 {
        let v = env.mem().push_local(12);
        let ox = c1(env, OLD_GX, m);
        let oy = c1(env, OLD_GY, m);
        let oz = c1(env, OLD_GZ, m);
        env.w32(v, ox << 16);
        env.w32(v + 8, oz << 16);
        env.w32(v + 4, oy << 16);
        fa = attr_at(env, m, v, a);
        let gx = c1(env, GX, m) as i16 as i32 as u32;
        let gz = c1(env, GZ, m) as i16 as i32 as u32;
        tile_centre(env.mem(), gx, gz, v);
        let y = c1(env, POS_Y, m);
        env.w32(v + 4, y);
        fb = attr_at(env, m, v, b);
        env.mem().pop_local(v);
    }
    let (av, bv) = (env.r32(a), env.r32(b));
    env.mem().pop_local(a);
    env.mem().pop_local(b);
    if fa == 1 {
        c2(env, SET_PREV_ATTR, m, av);
    }
    if fb == 1 {
        c2(env, SET_ATTR, m, bv);
        c2(env, BITS_CLEAR, m, 0x800);
        return 1;
    }
    c2(env, BITS_SET, m, 0x800);
    0
}

/// C: `FUN_02164168` (unlabeled routine at `0x02164168`): is the player MMDL (`FUN_0216DEE0`) within `-0x12..+0x11`
/// tiles of `m` on both axes (s16 bounds)?
pub fn near_player<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let sys = c1(env, SYS_OF, m);
    let p = c1(env, PLAYER, sys);
    if p == 0 {
        return 0;
    }
    let px = c1(env, GX, p) as i32;
    let pz = c1(env, GZ, p) as i32;
    let mx = c1(env, GX, m);
    let mz = c1(env, GZ, m);
    let lo = |v: u32| v.wrapping_sub(0x12) as i16 as i32;
    let hi = |v: u32| v.wrapping_add(0x11) as i16 as i32;
    (lo(mz) <= pz && hi(mz) >= pz && lo(mx) <= px && hi(mx) >= px) as u32
}

/// C: `FUN_021641C8` (unlabeled routine at `0x021641C8`): for move codes 0x33..0x36, run the move proc
/// (`FUN_0216D708`).
pub fn step_start_move<E: FieldEnv>(env: &mut E, m: u32) {
    let c = c1(env, MOVE_CODE, m);
    if (c.wrapping_add(0xFFCD) & 0xFFFF) <= 3 {
        c1(env, MOVE_PROC, m);
    }
}

/// C: `FUN_021641E8` (unlabeled routine at `0x021641E8`): dx of a direction (ROM `0x02171480[dir]`, s16: 0, 0, -1, 1;
/// larger `dir` reads the following ROM words).
pub fn dir_dx(dir: u32) -> u32 {
    rom32(0x0217_1480u32.wrapping_add(dir << 2)) as i16 as i32 as u32
}

/// C: `FUN_021641F8` (unlabeled routine at `0x021641F8`): dz of a direction (ROM `0x02171470[dir]`, s16: -1, 1, 0, 0).
pub fn dir_dz(dir: u32) -> u32 {
    rom32(0x0217_1470u32.wrapping_add(dir << 2)) as i16 as i32 as u32
}

/// C: `FUN_02164208` (unlabeled routine at `0x02164208`): move the `VecFx32` at `v` by `n` in direction `dir` (0: z
/// -= n, 1: z += n, 2: x -= n, 3: x += n).
pub fn vec_step(mem: &mut Mem, dir: u32, v: u32, n: u32) {
    let (o, add) = match dir {
        0 => (8, false),
        1 => (8, true),
        2 => (0, false),
        3 => (0, true),
        _ => return,
    };
    let a = v.wrapping_add(o);
    let x = mem.r32(a);
    mem.w32(
        a,
        if add {
            x.wrapping_add(n)
        } else {
            x.wrapping_sub(n)
        },
    );
}

/// C: `FUN_02164240` (unlabeled routine at `0x02164240`): move the s16 pair `*x` / `*z` by `n` in direction `dir`.
pub fn grid_step_xz(mem: &mut Mem, dir: u32, x: u32, z: u32, n: u32) {
    let (a, add) = match dir {
        0 => (z, false),
        1 => (z, true),
        2 => (x, false),
        3 => (x, true),
        _ => return,
    };
    let v = mem.r16(a) as i16 as i32 as u32;
    mem.w16(
        a,
        if add {
            v.wrapping_add(n)
        } else {
            v.wrapping_sub(n)
        } as u16,
    );
}

/// C: `FUN_02164280` (unlabeled routine at `0x02164280`): the tile centre: `v.x = gx << 16 + 0x8000`, `v.z = gz <<
/// 16 + 0x8000` (`v.y` untouched).
pub fn tile_centre(mem: &mut Mem, gx: u32, gz: u32, v: u32) {
    mem.w32(v, (gx << 16).wrapping_add(0x8000));
    mem.w32(v.wrapping_add(8), (gz << 16).wrapping_add(0x8000));
}

/// C: `FUN_02164294` (unlabeled routine at `0x02164294`): `v = (x, y, z) << 16`.
pub fn grid_to_fx(mem: &mut Mem, x: u32, y: u32, z: u32, v: u32) {
    mem.w32(v, x << 16);
    mem.w32(v.wrapping_add(4), y << 16);
    mem.w32(v.wrapping_add(8), z << 16);
}

/// C: `FUN_021642A4` (unlabeled routine at `0x021642A4`): the opposite direction (ROM `0x021714B0[dir]`, u16: 1, 0,
/// 3, 2).
pub fn dir_opposite(dir: u32) -> u32 {
    rom32(0x0217_14B0u32.wrapping_add(dir << 2)) & 0xFFFF
}

/// C: `FUN_021642B4` (unlabeled routine at `0x021642B4`): the direction from `(x0, z0)` to `(x1, z1)`: x first
/// (`x0 > x1`: 2 left, `x0 < x1`: 3 right), then z (`z0 > z1`: 0 up, else 1 down). Signed compares of the
/// whole registers.
pub fn dir_between(x0: i32, z0: i32, x1: i32, z1: i32) -> u16 {
    if x0 > x1 {
        2
    } else if x0 < x1 {
        3
    } else if z0 > z1 {
        0
    } else {
        1
    }
}

/// C: `FUN_021642D4` (unlabeled routine at `0x021642D4`): the direction of `angle` turned by the facing `dir`: sector
/// `((angle + ROM 0x02171458[dir]) & 0xFFFF) >> 12` -> `ROM 0x02171460[sector]` -> `ROM 0x021714C1[4 x]` (u8).
pub fn dir_rotated(dir: u32, angle: u32) -> u32 {
    let a = angle.wrapping_add(rom16(0x0217_1458u32.wrapping_add(dir << 1)));
    let sector = (a & 0xFFFF) >> 12;
    let x = rom8(0x0217_1460 + sector);
    rom8(0x0217_14C1 + (x << 2))
}

/// C: `FUN_021642F8` (unlabeled routine at `0x021642F8`): status |= 0x2000 (rail move; tail call of
/// `FUN_0216D548`).
pub fn set_rail<E: FieldEnv>(env: &mut E, m: u32) {
    c2(env, STATUS_SET, m, 0x2000);
}

/// C: `FUN_02164304` (unlabeled routine at `0x02164304`): the rail location of a rail HEADER (`hdr +0x1C` line, type
/// 1, key `FUN_0201441C(s16 dir +0xC)`, side `+0x20`, front `+0x1E`): handed to the rail system (`FUN_0219E2C0(m,
/// &loc)`) when the MMDLSYS flag 0x10 is set, else kept in the move work (`FUN_0216D684(m, 0xC)`, then
/// [`rail_loc_store`]).
pub fn rail_from_header<E: FieldEnv>(env: &mut E, m: u32, hdr: u32) {
    let r = env.mem().push_local(8);
    let line = env.r16(hdr.wrapping_add(0x1C));
    env.w16(r, line);
    env.w8(r + 2, 1);
    let d = env.r16(hdr.wrapping_add(0xC)) as i16 as i32 as u32;
    let k = c1(env, RAIL_KEY, d);
    env.w8(r + 3, k as u8);
    let front = env.r16(hdr.wrapping_add(0x1E));
    env.w16(r + 6, front);
    let side = env.r16(hdr.wrapping_add(0x20));
    env.w16(r + 4, side);
    let sys = c1(env, SYS_OF, m);
    if c2(env, SYS_FLAG, sys, 0x10) != 0 {
        env.call(RAIL_SET, &mut [CArg::V(m), CArg::In(r, 8)]);
    } else {
        c2(env, MOVE_WORK_CLEAR, m, 0xC);
        rail_loc_store(env, m, r);
    }
    env.mem().pop_local(r);
}

/// C: `FUN_0216435C` (unlabeled routine at `0x0216435C`): store the rail location `r` (4 halfwords) at `+4` of the
/// move work (`FUN_0216D69C`), marker [`RAIL_MARKER`] at `+0`.
pub fn rail_loc_store<E: FieldEnv>(env: &mut E, m: u32, r: u32) {
    let w = c1(env, MOVE_WORK, m);
    for o in [0, 2, 4, 6] {
        let v = env.r16(r.wrapping_add(o));
        env.w16(w.wrapping_add(4 + o), v);
    }
    env.w32(w, RAIL_MARKER);
}

/// C: `FUN_02164380` (unlabeled routine at `0x02164380`): when the move work holds a rail location (marker), copy it
/// to `out` and return 1; else 0.
pub fn rail_loc_load<E: FieldEnv>(env: &mut E, m: u32, out: u32) -> u32 {
    let w = c1(env, MOVE_WORK, m);
    if env.r32(w) != RAIL_MARKER {
        return 0;
    }
    for o in [0, 2, 4, 6] {
        let v = env.r16(w.wrapping_add(4 + o));
        env.w16(out.wrapping_add(o), v);
    }
    1
}

#[cfg(test)]
mod tests;
