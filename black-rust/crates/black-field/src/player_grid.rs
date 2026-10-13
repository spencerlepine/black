//! `field_player_grid.c` (overlay 21, `0x021A4140..0x021A5880`): PLAYER GRID, the 0x20-byte grid half of the player:
//! each frame the pad (keys, direction) becomes a move value by form (walk 021A44C8 / cycle 021A48C8 / surf
//! 021A4B00, with the ice / slide variant 021A46F0), the move value becomes an MMDL acmd (021A4CA8: walk, run, turn,
//! bump, jump, slide, fall) plus the footstep SE (021A428C, tables 0x021D8A00.. / 0x021D89F4..), and the forced-move
//! tiles take over (021A55A8, table 0x021D89EC).
//!
//! F3a-spec specced all 97 functions (`tools/rom-diff/specs/black-field/player_grid.spec`, header: layout, move
//! values, acmd codes); F3a ported them. GRID: `+0` move state (0 stop, 1 walk, 2 turn, 3 bump, 4 .., 5 slide, 6 ..,
//! 7 fall), `+4` move value, `+8` flags (1 forced move, 2 .., 4 .., 8 ice spin, 0x10 sliding, 0x20 running), `+0xC`
//! -> CORE, `+0x10` FIELDMAP, `+0x14 u16` / `+0x16 u8` ice-spin counter, `+0x18 u16` footstep SE (0x546 = none),
//! `+0x1A u16` SE frame counter, `+0x1C` a one-shot flag.
//!
//! INPUT (0x14, on the stack of [`frame`]): `+0` pad trigger, `+4` pad held (bit 1 = B), `+8 u16` dir (9 none),
//! `+0xC` flags (bit 0 running allowed), `+0x10` slow mode (1 = no run / no turn). Every callee outside this area
//! (the player core, fldmmdl, mmdl_move, eventdata, the attribute predicates of effect_encount, fldmmdl_acmd, SE) is
//! reached through [`FieldEnv::call`] by address; the INPUT record, the positions and attribute words the original
//! keeps on its stack are [`Mem::push_local`] windows or local byte arrays.

use crate::hooks::{CArg, FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_4140,
    0x021A_41B0,
    0x021A_4224,
    0x021A_4254,
    0x021A_428C,
    0x021A_43D8,
    0x021A_4444,
    0x021A_448C,
    0x021A_44BC,
    0x021A_44C8,
    0x021A_4544,
    0x021A_4598,
    0x021A_4664,
    0x021A_4698,
    0x021A_46F0,
    0x021A_4768,
    0x021A_47B4,
    0x021A_482C,
    0x021A_4890,
    0x021A_48A4,
    0x021A_48C8,
    0x021A_491C,
    0x021A_4960,
    0x021A_4994,
    0x021A_4A74,
    0x021A_4AA8,
    0x021A_4B00,
    0x021A_4B54,
    0x021A_4B98,
    0x021A_4C1C,
    0x021A_4C50,
    0x021A_4CA8,
    0x021A_4CF4,
    0x021A_4D8C,
    0x021A_4D90,
    0x021A_4DC0,
    0x021A_4E78,
    0x021A_4E9C,
    0x021A_4ED8,
    0x021A_4F04,
    0x021A_4F0C,
    0x021A_4F50,
    0x021A_4F58,
    0x021A_4FA8,
    0x021A_5010,
    0x021A_5014,
    0x021A_5044,
    0x021A_50BC,
    0x021A_50E0,
    0x021A_511C,
    0x021A_5148,
    0x021A_51A4,
    0x021A_51A8,
    0x021A_51D8,
    0x021A_5204,
    0x021A_5228,
    0x021A_5264,
    0x021A_5304,
    0x021A_530C,
    0x021A_5318,
    0x021A_5328,
    0x021A_5334,
    0x021A_5340,
    0x021A_5354,
    0x021A_5360,
    0x021A_536C,
    0x021A_5378,
    0x021A_5390,
    0x021A_53A4,
    0x021A_53BC,
    0x021A_53D0,
    0x021A_5408,
    0x021A_5458,
    0x021A_548C,
    0x021A_54A0,
    0x021A_54B4,
    0x021A_550C,
    0x021A_552C,
    0x021A_5538,
    0x021A_55A8,
    0x021A_55DC,
    0x021A_55FC,
    0x021A_5610,
    0x021A_5630,
    0x021A_569C,
    0x021A_56C0,
    0x021A_56F4,
    0x021A_5708,
    0x021A_5728,
    0x021A_572C,
    0x021A_5738,
    0x021A_5764,
    0x021A_5768,
    0x021A_5778,
    0x021A_5788,
    0x021A_57C0,
    0x021A_5810,
];

/// The "no footstep SE" value of `+0x18`.
pub const SE_NONE: u32 = 0x546;
/// No direction.
pub const DIR_NONE: u32 = 9;

// ARM9 / other units, named by address (meanings from the spec)
const HEAP_ALLOC: u32 = 0x0203_0734; // Heap_AllocDebug(heap, size, flag, file, line)
const FILE_PLAYER_GRID: u32 = 0x021D_DECC; // "field_player_grid.c"
const SE_PLAY: u32 = 0x0200_61E4; // sound: play SE (id)
const GAMEDATA_OF: u32 = 0x0201_20F4; // GAMESYS -> GAMEDATA
const EVENTDATA_OF: u32 = 0x0201_2844; // GAMEDATA -> EVENTDATA_SYSTEM
const SEASON: u32 = 0x0202_E83C; // GAMESYS getter (season / weather)
const WARP_AT: u32 = 0x0216_2564; // eventdata: warp index at a position (sys, &pos)
const WARP_REC: u32 = 0x0216_25CC; // eventdata: warp record (sys, idx)
const WARP_EXITS: u32 = 0x0216_26B8; // eventdata: warp exits in dir (warp, dir)
const HIT_CHECK: u32 = 0x0216_3994; // mmdl_move: step hit check (m, x, y, z, dir)
const ATTR_AT: u32 = 0x0216_3E34; // mmdl_move: attribute at a position (m, &pos, &attr)
const HEIGHT_AT: u32 = 0x0216_3E58; // mmdl_move: height at a position (m, &pos, &y)
const POS_REFRESH: u32 = 0x0216_3F10; // mmdl_move: grid position refresh (m)
const HEIGHT_REFRESH: u32 = 0x0216_4010; // mmdl_move: height refresh (m)
const ATTR_REFRESH: u32 = 0x0216_40B0; // mmdl_move: attribute refresh (m)
const DX: u32 = 0x0216_41E8; // mmdl_move: dx of dir
const DZ: u32 = 0x0216_41F8; // mmdl_move: dz of dir
const STEP: u32 = 0x0216_4208; // mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
const ACMD_SETTABLE: u32 = 0x0216_D3CC; // fldmmdl: MMDL acmd settable (m)
const SET_ACMD: u32 = 0x0216_D410; // fldmmdl: MMDL set acmd (m, code)
const ACMD_CLEAR: u32 = 0x0216_D4B4; // fldmmdl: MMDL acmd clear (m)
const STATUS_SET: u32 = 0x0216_D548; // fldmmdl: MMDL +0 |= v
const STATUS_CLEAR: u32 = 0x0216_D550; // fldmmdl: MMDL +0 &= ~v
const BITS_SET: u32 = 0x0216_D568; // fldmmdl: MMDL +4 |= v
const BITS_CLEAR: u32 = 0x0216_D570; // fldmmdl: MMDL +4 &= ~v
const SET_DISP_DIR: u32 = 0x0216_D5E0; // fldmmdl: MMDL set display dir (m, dir)
const DISP_DIR: u32 = 0x0216_D5F8; // fldmmdl: MMDL display dir
const MOVE_DIR: u32 = 0x0216_D608; // fldmmdl: MMDL move dir
const SET_2A: u32 = 0x0216_D674; // fldmmdl: MMDL +0x2A = v
const SYS_OF: u32 = 0x0216_D67C; // fldmmdl: MMDL -> MMDLSYS
const ATTR: u32 = 0x0216_D7DC; // fldmmdl: MMDL +0x74 (attribute)
const GX: u32 = 0x0216_D83C; // fldmmdl: MMDL gx
const GY: u32 = 0x0216_D854; // fldmmdl: MMDL gy
const SET_GY: u32 = 0x0216_D85C; // fldmmdl: MMDL gy = v
const GZ: u32 = 0x0216_D86C; // fldmmdl: MMDL gz
const GET_POS: u32 = 0x0216_D88C; // fldmmdl: MMDL position VecFx32 -> out
const SET_POS: u32 = 0x0216_D89C; // fldmmdl: MMDL position VecFx32 = *v
const DIR_LOCK: u32 = 0x0216_DB4C; // fldmmdl: MMDL dir lock (m, on)
const MOVE_BIT: u32 = 0x0216_DCE8; // fldmmdl: MMDL move bit (m, on)
const MMDL_AT: u32 = 0x0216_E5B0; // fldmmdl: MMDL at a position (sys, &pos)
const FM_GAMESYS: u32 = 0x0218_8C80; // fieldmap +4 (GAMESYS)
const FM_FX_CTRL: u32 = 0x0218_8CD0; // fieldmap +0x94 (field effect control)
const ACMD_CODE: u32 = 0x0219_7E7C; // fldmmdl_acmd: acmd code of (dir, base)
const RAIL_PRED: u32 = 0x0219_F7CC; // rail predicate (m, counter)
const DUST: u32 = 0x021A_C528; // fldeff_kemuri: dust (m, ctrl)
                               // player_core (another F3a area: force stubs)
const CORE_POS: u32 = 0x021A_3898; // core: MMDL position -> out (core, &pos)
const CORE_DISP_DIR: u32 = 0x021A_3920; // core: MMDL display dir (core)
const CORE_FIELDMAP: u32 = 0x021A_395C; // core: fieldmap (core)
const CORE_MMDL: u32 = 0x021A_3960; // core: the player MMDL (core)
const CORE_SET_STATE: u32 = 0x021A_3964; // core: set the move state (core, v)
const CORE_FORM: u32 = 0x021A_3974; // core: move form (core)
const CORE_3998: u32 = 0x021A_3998; // core: (core)
const CORE_SET_ENTRY: u32 = 0x021A_39AC; // core: form entry (core, i)
const CORE_HAS_FORM: u32 = 0x021A_3A20; // core: the OBJ code has a form (core)
const CORE_ACMD_OK: u32 = 0x021A_3AA0; // core: acmd settable (core)
const CORE_KEY_DIR: u32 = 0x021A_3AB8; // core: dir of keys (core, keys)
const CORE_KEYS: u32 = 0x021A_3AC4; // core: remember the keys (core, keys)
const CORE_SET_REQ: u32 = 0x021A_3CB0; // core: set requests (core, bits)
const CORE_REQ_DONE: u32 = 0x021A_3CC0; // core: requests done? (core)
const CORE_RUN_REQ: u32 = 0x021A_3CE8; // core: run requests (core)
                                       // map attribute predicates (effect_encount, overlay 21), named by address
const ATTR_VALUE: u32 = 0x021A_B0F0; // attribute value of an attribute
const ATTR_FLAGS: u32 = 0x021A_B0F8; // attribute flags of an attribute
const P_11C: u32 = 0x021A_B11C;
const P_140: u32 = 0x021A_B140;
const P_158: u32 = 0x021A_B158;
const P_164: u32 = 0x021A_B164;
const P_170: u32 = 0x021A_B170;
const P_17C: u32 = 0x021A_B17C;
const P_2DC: u32 = 0x021A_B2DC;
const P_328: u32 = 0x021A_B328;
const P_348: u32 = 0x021A_B348;
const P_404: u32 = 0x021A_B404;
const P_410: u32 = 0x021A_B410;
const P_41C: u32 = 0x021A_B41C;
const P_44C: u32 = 0x021A_B44C;

/// ROM `0x021D8A00` (`mode 0`) / `0x021D8A0C` (`mode 1`): footstep SE by attribute flags, records of 0xC
/// `{u32 flag mask (0 ends), u32 SE, s16 period (walking), s16 period (running / riding)}`.
const SE_FLAGS: [&[(u32, u32, i16, i16)]; 2] = [&[], &[(0x20, 0x67A, 2, 4)]];

/// ROM `0x021D89F4` (`mode 0`) / `0x021D8A2C` (`mode 1`): footstep SE by attribute predicate, records of 0xC
/// `{predicate (Thumb address, 0 ends), u16 SE, s8 period (walking), s8 period (running / riding), u8 season
/// (4 = any)}`.
/// One record of the predicate tables: (predicate, SE, walk period, run period, season).
type SePred = (u32, u16, i8, i8, u8);

const SE_PREDS: [&[SePred]; 2] = [
    &[],
    &[
        (0x021A_B141, 0x679, 2, 4, 4),
        (0x021A_B31D, 0x67C, 2, 4, 4),
        (0x021A_B329, 0x67C, 2, 4, 4),
        (0x021A_B305, 0x67E, 2, 4, 4),
        (0x021A_B2F9, 0x67E, 2, 4, 4),
        (0x021A_B365, 0x67E, 2, 4, 4),
        (0x021A_B41D, 0x6EC, 2, 4, 4),
        (0x021A_B391, 0x67F, 2, 4, 4),
        (0x021A_B3D5, 0x67C, 2, 4, 3),
        (0x021A_B3E1, 0x67C, 2, 4, 3),
    ],
];

/// ROM `0x021D8A24`: the next direction to try when turning on ice (u16 per direction: 3, 2, 0, 1).
const TURN_ORDER: [u16; 4] = [3, 2, 0, 1];

fn c1<E: FieldEnv>(env: &mut E, addr: u32, a: u32) -> u32 {
    env.callv(addr, &[a])
}

fn c2<E: FieldEnv>(env: &mut E, addr: u32, a: u32, b: u32) -> u32 {
    env.callv(addr, &[a, b])
}

fn core_of<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    env.r32(grid.wrapping_add(0xC))
}

/// The player MMDL (`FUN_021A3960(grid +0xC)`).
fn mmdl_of<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    let core = core_of(env, grid);
    c1(env, CORE_MMDL, core)
}

/// `FUN_0216D88C(m, &pos)`: the MMDL position into a local VecFx32.
fn mmdl_pos<E: FieldEnv>(env: &mut E, m: u32) -> [u8; 12] {
    let mut p = [0u8; 12];
    env.call(GET_POS, &mut [CArg::V(m), CArg::OutBuf(&mut p)]);
    p
}

/// `FUN_02164208(dir, &pos, 0x10000)`: one step in `dir` (the callee writes `pos`).
fn step<E: FieldEnv>(env: &mut E, dir: u32, p: &mut [u8; 12]) {
    env.call(STEP, &mut [CArg::V(dir), CArg::InBuf(p), CArg::V(0x1_0000)]);
}

fn word(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

fn input_dir<E: FieldEnv>(env: &mut E, inp: u32) -> u32 {
    env.r16(inp.wrapping_add(8)) as u32
}

/// C: `FUN_021A4140` (unlabeled routine at `0x021A4140`): create the PLAYER GRID: `Heap_AllocDebug(heap, 0x20, 1,
/// "field_player_grid.c", 0x17D)`, `+0xC` = core, `+0x10` = the core's fieldmap (`FUN_021A395C`), `+0x18` = 0x546;
/// [`set_flag_2`]; move form 2 (surf): request 4 (`FUN_021A3CB0(core, 4)`) and run it (`FUN_021A3CE8`); form 3:
/// request 0x100 and run it. Returns the block.
pub fn create<E: FieldEnv>(env: &mut E, core: u32, heap: u32) -> u32 {
    let g = env.callv(HEAP_ALLOC, &[heap, 0x20, 1, FILE_PLAYER_GRID, 0x17D]);
    env.w32(g.wrapping_add(0xC), core);
    let fm = c1(env, CORE_FIELDMAP, core);
    env.w32(g.wrapping_add(0x10), fm);
    env.w16(g.wrapping_add(0x18), SE_NONE as u16);
    set_flag_2(env, g);
    let core = core_of(env, g);
    let req = match c1(env, CORE_FORM, core) {
        2 => Some(4),
        3 => Some(0x100),
        _ => None,
    };
    if let Some(bits) = req {
        let core = core_of(env, g);
        c2(env, CORE_SET_REQ, core, bits);
        let core = core_of(env, g);
        c1(env, CORE_RUN_REQ, core);
    }
    g
}

/// C: `FUN_021A41B0` (unlabeled routine at `0x021A41B0`): the grid frame: dir = `FUN_021A3AB8(core, held)`; when
/// [`can_act`] and then `FUN_021A3CC0(core)` (the requests) and [`forced_move`]`(grid, dir, 0) == 0`: build the
/// INPUT record `{trigger, held, u16 dir, flags, 0}` on the stack, v = [`move_value`], [`apply_value`],
/// [`act`], [`step_se`].
pub fn frame<E: FieldEnv>(env: &mut E, grid: u32, trigger: u32, held: u32, flags: u32) {
    let core = core_of(env, grid);
    let dir = c2(env, CORE_KEY_DIR, core, held);
    if can_act(env, grid, dir) != 1 {
        return;
    }
    let core = core_of(env, grid);
    c1(env, CORE_REQ_DONE, core);
    if forced_move(env, grid, dir, 0) != 0 {
        return;
    }
    let inp = env.mem().push_local(0x14);
    env.w32(inp, trigger);
    env.w32(inp.wrapping_add(4), held);
    env.w16(inp.wrapping_add(8), dir as u16);
    env.w32(inp.wrapping_add(0xC), flags);
    env.w32(inp.wrapping_add(0x10), 0);
    let v = move_value(env, grid, inp);
    apply_value(env, grid, v, inp);
    act(env, grid, v, inp);
    step_se(env, grid, v, dir);
    env.mem().pop_local(inp);
}

/// C: `FUN_021A4224` (unlabeled routine at `0x021A4224`): `+4` = v; `FUN_021A3AC4(core, in +4)` (remember the keys);
/// [`clear_flag_4`]; flag 0x20 off; v == 2: [`clear_flag_2`]; `+0x1C` = 0.
pub fn apply_value<E: FieldEnv>(env: &mut E, grid: u32, v: u32, inp: u32) {
    let core = core_of(env, grid);
    let held = env.r32(inp.wrapping_add(4));
    env.w32(grid.wrapping_add(4), v);
    c2(env, CORE_KEYS, core, held);
    clear_flag_4(env, grid);
    clear_flags(env.mem(), grid, 0x20);
    if v == 2 {
        clear_flag_2(env, grid);
    }
    env.w32(grid.wrapping_add(0x1C), 0);
}

/// C: `FUN_021A4254` (unlabeled routine at `0x021A4254`): 1 when the MMDL acmd is settable (`FUN_0216D3CC`), or when
/// `dir` is not 9, the move state is 3 (bump) and `dir` is not the display dir; else 0.
pub fn can_act<E: FieldEnv>(env: &mut E, grid: u32, dir: u32) -> u32 {
    let m = mmdl_of(env, grid);
    if c1(env, ACMD_SETTABLE, m) == 1 {
        return 1;
    }
    if dir != DIR_NONE && env.r32(grid) == 3 && dir != c1(env, DISP_DIR, m) {
        return 1;
    }
    0
}

/// C: `FUN_021A428C` (unlabeled routine at `0x021A428C`): the footstep SE at `pos` (`r1` unused, `mode` 0 / 1 picks
/// the table pair, `season` is the stack argument): kind = 2 on a bicycle (form 1), 1 when running (flag 0x20), else
/// 0 (and `+0x18` = 0x546, `+0x1A` = 0). With the attribute at `pos` (`FUN_02163E34`): the first flag record whose
/// mask meets its flags (`FUN_021AB0F8`), else the first predicate record whose predicate holds for the u16
/// attribute and whose season byte is 4 or `season`, gives the SE and (by kind) its period. No SE: mode 1 clears
/// `+0x1A`. With an SE and kind != 0: `+0x1A` restarts when the SE changed, the SE is held back while `0 < +0x1A <
/// period`, `+0x1A` wraps at the period, `+0x18` = the SE, `+0x1A` += 1. Plays the SE and answers 1 when there is
/// one, else 0.
pub fn footstep_se<E: FieldEnv>(
    env: &mut E,
    grid: u32,
    _v: u32,
    pos: u32,
    mode: u32,
    season: u32,
) -> u32 {
    let mut se = SE_NONE;
    let mut period: i32 = 0;
    let core = core_of(env, grid);
    let kind = if c1(env, CORE_FORM, core) == 1 {
        2
    } else if has_flags(env.mem(), grid, 0x20) {
        1
    } else {
        0
    };
    if kind == 0 {
        env.w16(grid.wrapping_add(0x18), SE_NONE as u16);
        env.w16(grid.wrapping_add(0x1A), 0);
    }
    let m = mmdl_of(env, grid);
    let mut a = [0u8; 4];
    if env.call(
        ATTR_AT,
        &mut [CArg::V(m), CArg::In(pos, 12), CArg::OutBuf(&mut a)],
    ) != 1
    {
        return 0;
    }
    let attr = word(&a, 0);
    let fl = c1(env, ATTR_FLAGS, attr);
    let t = (mode != 0) as usize;
    for &(mask, s, walk, run) in SE_FLAGS[t] {
        if mask & fl != 0 {
            se = s;
            if kind == 1 {
                period = walk as i32;
            } else if kind == 2 {
                period = run as i32;
            }
            break;
        }
    }
    if se == SE_NONE {
        for &(f, s, walk, run, sea) in SE_PREDS[t] {
            if c1(env, f & !1, attr & 0xFFFF) == 1 && (sea == 4 || sea as u32 == season) {
                se = s as u32;
                if kind == 1 {
                    period = walk as i32;
                } else if kind == 2 {
                    period = run as i32;
                }
                break;
            }
        }
    }
    if se == SE_NONE {
        if mode == 1 {
            env.w16(grid.wrapping_add(0x1A), 0);
        }
    } else if kind != 0 {
        if env.r16(grid.wrapping_add(0x18)) as u32 != se {
            env.w16(grid.wrapping_add(0x1A), 0);
        }
        let c = env.r16(grid.wrapping_add(0x1A)) as i32;
        if c != 0 && c < period {
            se = SE_NONE;
        }
        if c >= period {
            env.w16(grid.wrapping_add(0x1A), 0);
        }
        env.w16(grid.wrapping_add(0x18), se as u16);
        let n = env.r16(grid.wrapping_add(0x1A)).wrapping_add(1);
        env.w16(grid.wrapping_add(0x1A), n);
    }
    if se != SE_NONE {
        c1(env, SE_PLAY, se);
        return 1;
    }
    0
}

/// C: `FUN_021A43D8` (unlabeled routine at `0x021A43D8`): v != 2: `+0x18` = 0x546, `+0x1A` = 0; else, with the MMDL
/// position and the season (`FUN_0202E83C` of the GAMESYS), [`footstep_se`] mode 0 there, and when that plays
/// nothing, one step ahead in `dir` with mode 1.
pub fn step_se<E: FieldEnv>(env: &mut E, grid: u32, v: u32, dir: u32) {
    if v != 2 {
        env.w16(grid.wrapping_add(0x18), SE_NONE as u16);
        env.w16(grid.wrapping_add(0x1A), 0);
        return;
    }
    let m = mmdl_of(env, grid);
    let fm = env.r32(grid.wrapping_add(0x10));
    let gs = c1(env, FM_GAMESYS, fm);
    let season = c1(env, SEASON, gs);
    let p = env.mem().push_local(12);
    let b = mmdl_pos(env, m);
    env.mem().write(p, &b);
    if footstep_se(env, grid, v, p, 0, season) == 0 {
        let mut b = [0u8; 12];
        for (i, x) in b.iter_mut().enumerate() {
            *x = env.r8(p + i as u32);
        }
        step(env, dir, &mut b);
        env.mem().write(p, &b);
        footstep_se(env, grid, v, p, 1, season);
    }
    env.mem().pop_local(p);
}

/// C: `FUN_021A4444` (unlabeled routine at `0x021A4444`): the move value by move form: 0 [`walk_value`], 1
/// [`cycle_value`], 2 / 3 [`surf_value`]; other forms 0.
pub fn move_value<E: FieldEnv>(env: &mut E, grid: u32, inp: u32) -> u32 {
    let core = core_of(env, grid);
    match c1(env, CORE_FORM, core) {
        0 => walk_value(env, grid, inp),
        1 => cycle_value(env, grid, inp),
        2 | 3 => surf_value(env, grid, inp),
        _ => 0,
    }
}

/// C: `FUN_021A448C` (unlabeled routine at `0x021A448C`): dir 9: 0; dir is the display dir or the move state is 1 /
/// 3: 1; else 2.
pub fn dir_kind<E: FieldEnv>(env: &mut E, grid: u32, dir: u32) -> u32 {
    let m = mmdl_of(env, grid);
    let d = c1(env, DISP_DIR, m);
    if dir == DIR_NONE {
        return 0;
    }
    if dir == d {
        return 1;
    }
    let s = env.r32(grid);
    if s == 1 || s == 3 {
        1
    } else {
        2
    }
}

/// C: `FUN_021A44BC` (unlabeled routine at `0x021A44BC`): `FUN_021A3AA0(core)` (acmd settable).
pub fn acmd_ok<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    let core = core_of(env, grid);
    c1(env, CORE_ACMD_OK, core)
}

/// C: `FUN_021A44C8` (unlabeled routine at `0x021A44C8`): the walk move value: ice under the player ([`on_ice`]) or
/// sliding ([`sliding`]): [`ice_value`]; else by move state: 0 [`walk_stop`], 1 [`walk_step`], 2 [`walk_turning`],
/// 3 [`walk_bumping`]; else 0.
pub fn walk_value<E: FieldEnv>(env: &mut E, grid: u32, inp: u32) -> u32 {
    let m = mmdl_of(env, grid);
    if on_ice(env, grid) == 1 || sliding(env, grid) == 1 {
        return ice_value(env, grid, inp);
    }
    match env.r32(grid) {
        0 => walk_stop(env, grid, m, inp),
        1 => walk_step(env, grid, m, inp),
        2 => walk_turning(env, grid, m, inp),
        3 => walk_bumping(env, grid, m, inp),
        _ => 0,
    }
}

/// C: `FUN_021A4544` (unlabeled routine at `0x021A4544`): acmd settable: dir 9: 1 (stop); dir is not the display dir,
/// slow mode off and (running not allowed or B not held): 3 (turn); else [`walk_step`]. Not settable: 0.
pub fn walk_stop<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) != 1 {
        return 0;
    }
    if input_dir(env, inp) == DIR_NONE {
        return 1;
    }
    let d = c1(env, DISP_DIR, m);
    if input_dir(env, inp) != d && env.r32(inp.wrapping_add(0x10)) == 0 {
        if env.r32(inp.wrapping_add(0xC)) & 1 == 0 {
            return 3;
        }
        if env.r32(inp.wrapping_add(4)) & 2 == 0 {
            return 3;
        }
    }
    walk_step(env, grid, m, inp)
}

/// The ledge direction of attribute value `v`: the first of `FUN_021AB158` / `164` / `170` / `17C` that holds gives
/// 0..3; none 9.
fn ledge_dir<E: FieldEnv>(env: &mut E, v: u32) -> u32 {
    for (d, p) in [P_158, P_164, P_170, P_17C].into_iter().enumerate() {
        if c1(env, p, v) != 0 {
            return d as u32;
        }
    }
    DIR_NONE
}

/// The head shared by [`walk_step`] / [`cycle_step`] / [`surf_step`]: [`hit_check`] into a local attribute word, with
/// slow mode a hit without bit 0x10 counts as none. Returns `(hit, attr)`.
fn step_hit<E: FieldEnv>(env: &mut E, grid: u32, m: u32, dir: u32, inp: u32) -> (u32, u32) {
    let a = env.mem().push_local(4);
    let mut hit = hit_check(env, grid, m, dir, a);
    let attr = env.r32(a);
    env.mem().pop_local(a);
    if env.r32(inp.wrapping_add(0x10)) == 1 && hit != 0 && hit & 0x10 == 0 {
        hit = 0;
    }
    (hit, attr)
}

/// C: `FUN_021A4598` (unlabeled routine at `0x021A4598`): the walk step: acmd not settable: 0; dir 9: [`walk_stop`];
/// hit = [`hit_check`] (slow mode: hits without bit 0x10 dropped); no attribute ahead (-1): 4; hit bit 2 and the
/// ledge direction of the attribute value is dir: 5 (jump); hit: 4; attribute flag bit 1 clear or slow mode: 2
/// (walk); else 4.
pub fn walk_step<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) == 0 {
        return 0;
    }
    let dir = input_dir(env, inp);
    if dir == DIR_NONE {
        return walk_stop(env, grid, m, inp);
    }
    let (hit, attr) = step_hit(env, grid, m, dir, inp);
    if attr == u32::MAX {
        return 4;
    }
    let v = c1(env, ATTR_VALUE, attr);
    let f = c1(env, ATTR_FLAGS, attr);
    if hit & 2 != 0 {
        let l = ledge_dir(env, v);
        if l != DIR_NONE && l == input_dir(env, inp) {
            return 5;
        }
    }
    if hit != 0 {
        return 4;
    }
    if f & 2 == 0 || env.r32(inp.wrapping_add(0x10)) == 1 {
        2
    } else {
        4
    }
}

/// C: `FUN_021A4664` (unlabeled routine at `0x021A4664`): acmd not settable: 0; dir 9: [`walk_stop`]; else
/// [`walk_step`].
pub fn walk_turning<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) == 0 {
        return 0;
    }
    if input_dir(env, inp) == DIR_NONE {
        walk_stop(env, grid, m, inp)
    } else {
        walk_step(env, grid, m, inp)
    }
}

/// C: `FUN_021A4698` (unlabeled routine at `0x021A4698`): a new dir (not 9, not the display dir): cancel the acmd
/// (`FUN_0216D4B4`), [`walk_step`]; else acmd not settable: 0; dir 9: [`walk_stop`]; else [`walk_step`].
pub fn walk_bumping<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    let d = c1(env, DISP_DIR, m);
    let dir = input_dir(env, inp);
    if dir != DIR_NONE && dir != d {
        c1(env, ACMD_CLEAR, m);
        return walk_step(env, grid, m, inp);
    }
    if c1(env, ACMD_SETTABLE, m) == 0 {
        return 0;
    }
    if input_dir(env, inp) == DIR_NONE {
        walk_stop(env, grid, m, inp)
    } else {
        walk_step(env, grid, m, inp)
    }
}

/// C: `FUN_021A46F0` (unlabeled routine at `0x021A46F0`): the ice / slide move value by move state 0..7:
/// [`ice_stop`], [`ice_step`], [`walk_turning`], [`walk_bumping`], [`ice_spin`], [`ice_wait`], [`ice_wait`],
/// [`ice_fall`]; else 0.
pub fn ice_value<E: FieldEnv>(env: &mut E, grid: u32, inp: u32) -> u32 {
    let m = mmdl_of(env, grid);
    match env.r32(grid) {
        0 => ice_stop(env, grid, m, inp),
        1 => ice_step(env, grid, m, inp),
        2 => walk_turning(env, grid, m, inp),
        3 => walk_bumping(env, grid, m, inp),
        4 => ice_spin(env, grid, m, inp),
        5 | 6 => ice_wait(env, grid, m, inp),
        7 => ice_fall(env, grid, m, inp),
        _ => 0,
    }
}

/// C: `FUN_021A4768` (unlabeled routine at `0x021A4768`): acmd settable: dir 9: [`ice_spin`]; dir not the display
/// dir and slow mode off: 3; else [`ice_step`]. Not settable: 0.
pub fn ice_stop<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) != 1 {
        return 0;
    }
    if input_dir(env, inp) == DIR_NONE {
        return ice_spin(env, grid, m, inp);
    }
    let d = c1(env, DISP_DIR, m);
    if input_dir(env, inp) != d && env.r32(inp.wrapping_add(0x10)) == 0 {
        return 3;
    }
    ice_step(env, grid, m, inp)
}

/// C: `FUN_021A47B4` (unlabeled routine at `0x021A47B4`): the slide step: acmd not settable: 0; dir 9: [`ice_stop`];
/// hit = [`hit_check_ice`]; an attribute ahead: hit bit 0x100 (ice ahead): no other bit 2, else 4; no ice:
/// `FUN_021AB11C(attr) == 0`: [`set_sliding`] on, 7; else 4. No attribute: 4.
pub fn ice_step<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) == 0 {
        return 0;
    }
    let dir = input_dir(env, inp);
    if dir == DIR_NONE {
        return ice_stop(env, grid, m, inp);
    }
    let a = env.mem().push_local(4);
    let hit = hit_check_ice(env, grid, m, dir, a);
    let attr = env.r32(a);
    env.mem().pop_local(a);
    if attr != u32::MAX {
        if hit & 0x100 != 0 {
            if hit & 0xFFFF_FEFF == 0 {
                return 2;
            }
        } else if c1(env, P_11C, attr) == 0 {
            set_sliding(env, grid, 1);
            return 7;
        }
    }
    4
}

/// C: `FUN_021A482C` (unlabeled routine at `0x021A482C`): the ice spin: dir 9: flag 8 clear: [`spin_start`];
/// `+0x16` == 1 and `FUN_0219F7CC(m, +0x14)`: `+0x16` = 0; `+0x14` += 1; at 60: [`spin_end`], [`set_sliding`] on, 7;
/// else 6. A dir: [`spin_end`], [`ice_step`].
pub fn ice_spin<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if input_dir(env, inp) != DIR_NONE {
        spin_end(env, grid);
        return ice_step(env, grid, m, inp);
    }
    if spinning(env, grid) == 0 {
        spin_start(env, grid);
    }
    if env.r8(grid.wrapping_add(0x16)) == 1 {
        let n = env.r16(grid.wrapping_add(0x14)) as u32;
        if c2(env, RAIL_PRED, m, n) != 0 {
            env.w8(grid.wrapping_add(0x16), 0);
        }
    }
    let n = env.r16(grid.wrapping_add(0x14)).wrapping_add(1);
    env.w16(grid.wrapping_add(0x14), n);
    if env.r16(grid.wrapping_add(0x14)) == 60 {
        spin_end(env, grid);
        set_sliding(env, grid, 1);
        return 7;
    }
    6
}

/// C: `FUN_021A4890` (unlabeled routine at `0x021A4890`): acmd settable: 9; else 8.
pub fn ice_wait<E: FieldEnv>(env: &mut E, _grid: u32, m: u32, _inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) == 1 {
        9
    } else {
        8
    }
}

/// C: `FUN_021A48A4` (unlabeled routine at `0x021A48A4`): on the ground ([`grounded`]): [`stop_sliding`],
/// [`set_sliding`] off, 1; else 9.
pub fn ice_fall<E: FieldEnv>(env: &mut E, grid: u32, _m: u32, _inp: u32) -> u32 {
    if grounded(env, grid) == 1 {
        stop_sliding(env, grid);
        set_sliding(env, grid, 0);
        1
    } else {
        9
    }
}

/// C: `FUN_021A48C8` (unlabeled routine at `0x021A48C8`): the cycle move value by move state: 0 [`cycle_stop`], 1
/// [`cycle_step`], 2 [`cycle_turning`], 3 [`cycle_bumping`]; else 0.
pub fn cycle_value<E: FieldEnv>(env: &mut E, grid: u32, inp: u32) -> u32 {
    let m = mmdl_of(env, grid);
    match env.r32(grid) {
        0 => cycle_stop(env, grid, m, inp),
        1 => cycle_step(env, grid, m, inp),
        2 => cycle_turning(env, grid, m, inp),
        3 => cycle_bumping(env, grid, m, inp),
        _ => 0,
    }
}

/// C: `FUN_021A491C` (unlabeled routine at `0x021A491C`): acmd settable: dir 9: 1; dir not the display dir and slow
/// mode off: 3; else [`cycle_step`]. Not settable: 0.
pub fn cycle_stop<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) != 1 {
        return 0;
    }
    if input_dir(env, inp) == DIR_NONE {
        return 1;
    }
    let d = c1(env, DISP_DIR, m);
    if input_dir(env, inp) != d && env.r32(inp.wrapping_add(0x10)) == 0 {
        return 3;
    }
    cycle_step(env, grid, m, inp)
}

/// C: `FUN_021A4960` (unlabeled routine at `0x021A4960`): any of `FUN_021AB140` / `021AB404` / `021AB410` /
/// `021AB328` holds for `v`.
pub fn cycle_ok_attr<E: FieldEnv>(env: &mut E, v: u32) -> u32 {
    for p in [P_140, P_404, P_410, P_328] {
        if c1(env, p, v) != 0 {
            return 1;
        }
    }
    0
}

/// C: `FUN_021A4994` (unlabeled routine at `0x021A4994`): the cycle step: as [`walk_step`] up to the jump (dir 9:
/// [`cycle_stop`]); hit: 4; flag bit 1 clear and [`cycle_ok_attr`] of the value 0: 2; else slow mode: 2; else 4.
pub fn cycle_step<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) == 0 {
        return 0;
    }
    let dir = input_dir(env, inp);
    if dir == DIR_NONE {
        return cycle_stop(env, grid, m, inp);
    }
    let (hit, attr) = step_hit(env, grid, m, dir, inp);
    if attr == u32::MAX {
        return 4;
    }
    let v = c1(env, ATTR_VALUE, attr);
    let f = c1(env, ATTR_FLAGS, attr);
    if hit & 2 != 0 {
        let l = ledge_dir(env, v);
        if l != DIR_NONE && l == input_dir(env, inp) {
            return 5;
        }
    }
    if hit != 0 {
        return 4;
    }
    if f & 2 == 0 && cycle_ok_attr(env, v) == 0 {
        return 2;
    }
    if env.r32(inp.wrapping_add(0x10)) == 1 {
        2
    } else {
        4
    }
}

/// C: `FUN_021A4A74` (unlabeled routine at `0x021A4A74`): acmd not settable: 0; dir 9: [`cycle_stop`]; else
/// [`cycle_step`].
pub fn cycle_turning<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) == 0 {
        return 0;
    }
    if input_dir(env, inp) == DIR_NONE {
        cycle_stop(env, grid, m, inp)
    } else {
        cycle_step(env, grid, m, inp)
    }
}

/// C: `FUN_021A4AA8` (unlabeled routine at `0x021A4AA8`): a new dir: cancel the acmd, [`cycle_step`]; else acmd not
/// settable: 0; dir 9: [`cycle_stop`]; else [`cycle_step`].
pub fn cycle_bumping<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    let d = c1(env, DISP_DIR, m);
    let dir = input_dir(env, inp);
    if dir != DIR_NONE && dir != d {
        c1(env, ACMD_CLEAR, m);
        return cycle_step(env, grid, m, inp);
    }
    if c1(env, ACMD_SETTABLE, m) == 0 {
        return 0;
    }
    if input_dir(env, inp) == DIR_NONE {
        cycle_stop(env, grid, m, inp)
    } else {
        cycle_step(env, grid, m, inp)
    }
}

/// C: `FUN_021A4B00` (unlabeled routine at `0x021A4B00`): the surf move value by move state: 0 [`surf_stop`], 1
/// [`surf_step`], 2 [`surf_turning`], 3 [`surf_bumping`]; else 0.
pub fn surf_value<E: FieldEnv>(env: &mut E, grid: u32, inp: u32) -> u32 {
    let m = mmdl_of(env, grid);
    match env.r32(grid) {
        0 => surf_stop(env, grid, m, inp),
        1 => surf_step(env, grid, m, inp),
        2 => surf_turning(env, grid, m, inp),
        3 => surf_bumping(env, grid, m, inp),
        _ => 0,
    }
}

/// C: `FUN_021A4B54` (unlabeled routine at `0x021A4B54`): acmd settable: dir 9: 1; dir not the display dir and slow
/// mode off: 3; else [`surf_step`]. Not settable: 0.
pub fn surf_stop<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) != 1 {
        return 0;
    }
    if input_dir(env, inp) == DIR_NONE {
        return 1;
    }
    let d = c1(env, DISP_DIR, m);
    if input_dir(env, inp) != d && env.r32(inp.wrapping_add(0x10)) == 0 {
        return 3;
    }
    surf_step(env, grid, m, inp)
}

/// C: `FUN_021A4B98` (unlabeled routine at `0x021A4B98`): the surf step: acmd not settable: 0; dir 9: [`surf_stop`];
/// hit as in [`walk_step`]; no attribute or a hit: 4 (the value `FUN_021AB0F0` is read and unused); flag bit 1
/// (water): 2; slow mode: 2; else 4.
pub fn surf_step<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) == 0 {
        return 0;
    }
    let dir = input_dir(env, inp);
    if dir == DIR_NONE {
        return surf_stop(env, grid, m, inp);
    }
    let (hit, attr) = step_hit(env, grid, m, dir, inp);
    if attr == u32::MAX {
        return 4;
    }
    c1(env, ATTR_VALUE, attr);
    let f = c1(env, ATTR_FLAGS, attr);
    if hit != 0 {
        return 4;
    }
    if f & 2 != 0 || env.r32(inp.wrapping_add(0x10)) == 1 {
        2
    } else {
        4
    }
}

/// C: `FUN_021A4C1C` (unlabeled routine at `0x021A4C1C`): acmd not settable: 0; dir 9: [`surf_stop`]; else
/// [`surf_step`].
pub fn surf_turning<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    if c1(env, ACMD_SETTABLE, m) == 0 {
        return 0;
    }
    if input_dir(env, inp) == DIR_NONE {
        surf_stop(env, grid, m, inp)
    } else {
        surf_step(env, grid, m, inp)
    }
}

/// C: `FUN_021A4C50` (unlabeled routine at `0x021A4C50`): a new dir: cancel the acmd, [`surf_step`]; else acmd not
/// settable: 0; dir 9: [`surf_stop`]; else [`surf_step`].
pub fn surf_bumping<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) -> u32 {
    let d = c1(env, DISP_DIR, m);
    let dir = input_dir(env, inp);
    if dir != DIR_NONE && dir != d {
        c1(env, ACMD_CLEAR, m);
        return surf_step(env, grid, m, inp);
    }
    if c1(env, ACMD_SETTABLE, m) == 0 {
        return 0;
    }
    if input_dir(env, inp) == DIR_NONE {
        surf_stop(env, grid, m, inp)
    } else {
        surf_step(env, grid, m, inp)
    }
}

/// C: `FUN_021A4CA8` (unlabeled routine at `0x021A4CA8`): act on the move value by move form: 0 [`walk_act`], 1
/// [`cycle_act`], 2 / 3 [`surf_act`].
pub fn act<E: FieldEnv>(env: &mut E, grid: u32, v: u32, inp: u32) {
    let core = core_of(env, grid);
    match c1(env, CORE_FORM, core) {
        0 => walk_act(env, grid, v, inp),
        1 => cycle_act(env, grid, v, inp),
        2 | 3 => surf_act(env, grid, v, inp),
        _ => {}
    }
}

/// C: `FUN_021A4CF4` (unlabeled routine at `0x021A4CF4`): the walk actions by move value 0..9: [`act_none`],
/// [`act_stand`], [`act_walk`], [`act_turn`], [`act_bump`], [`act_jump`], [`act_state_4`], [`act_slide`],
/// [`act_state_6`], [`act_fall`].
pub fn walk_act<E: FieldEnv>(env: &mut E, grid: u32, v: u32, inp: u32) {
    let m = mmdl_of(env, grid);
    match v {
        0 => act_none(env, grid, m, inp),
        1 => act_stand(env, grid, m, inp),
        2 => act_walk(env, grid, m, inp),
        3 => act_turn(env, grid, m, inp),
        4 => act_bump(env, grid, m, inp),
        5 => act_jump(env, grid, m, inp),
        6 => act_state_4(env, grid, m, inp),
        7 => act_slide(env, grid, m, inp),
        8 => act_state_6(env, grid, m, inp),
        9 => act_fall(env, grid, m, inp),
        _ => {}
    }
}

/// C: `FUN_021A4D8C` (unlabeled routine at `0x021A4D8C`): nothing.
pub fn act_none<E: FieldEnv>(_env: &mut E, _grid: u32, _m: u32, _inp: u32) {}

/// The stand action of [`act_stand`] / [`cycle_stand`] / [`surf_stand`]: dir = the input dir (9: the display dir);
/// acmd `FUN_02197E7C(dir, 0)`; `+0` = 0; `FUN_021A3964(core, 0)`.
fn stand<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    let mut d = input_dir(env, inp);
    if d == DIR_NONE {
        d = c1(env, DISP_DIR, m);
    }
    let code = c2(env, ACMD_CODE, d, 0);
    c2(env, SET_ACMD, m, code);
    let core = core_of(env, grid);
    env.w32(grid, 0);
    c2(env, CORE_SET_STATE, core, 0);
}

/// `FUN_0216D410(m, FUN_02197E7C(in dir, base))`; `+0` = state; `FUN_021A3964(core, core_state)`.
fn simple_act<E: FieldEnv>(
    env: &mut E,
    grid: u32,
    m: u32,
    inp: u32,
    base: u32,
    state: u32,
    core_state: u32,
) {
    let d = input_dir(env, inp);
    let code = c2(env, ACMD_CODE, d, base);
    c2(env, SET_ACMD, m, code);
    env.w32(grid, state);
    let core = core_of(env, grid);
    c2(env, CORE_SET_STATE, core, core_state);
}

/// The bump action: acmd base 0x1C, `+0` = 3, core state 0; no warp ahead ([`warp_ahead`] 0): SE 0x55A.
fn bump<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    simple_act(env, grid, m, inp, 0x1C, 3, 0);
    let d = input_dir(env, inp);
    if warp_ahead(env, grid, d) == 0 {
        c1(env, SE_PLAY, 0x55A);
    }
}

/// The jump action: acmd base 0x38, `+0` = 1, core state 1, SE 0x55E.
fn jump<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    simple_act(env, grid, m, inp, 0x38, 1, 1);
    c1(env, SE_PLAY, 0x55E);
}

/// C: `FUN_021A4D90` (unlabeled routine at `0x021A4D90`): stand (acmd base 0 in the input dir, or the display dir
/// when none); `+0` = 0; `FUN_021A3964(core, 0)`.
pub fn act_stand<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    stand(env, grid, m, inp);
}

/// C: `FUN_021A4DC0` (unlabeled routine at `0x021A4DC0`): walk: slow mode: acmd base 0x14, no slope check; running
/// allowed and B held: OBJ code without a form (`FUN_021A3A20` 0): 0x10; else 0x58 and flag 0x20 on, unless the
/// attribute under the player (`FUN_02163E34` at the core position) satisfies `FUN_021AB44C`: 0xC and flag 0x20 off;
/// otherwise 0xC. Slope down ahead ([`slope`] 1): 0xC -> 0xA7, 0x10 -> 0x4C, 0x58 -> 0xA3. Acmd; `+0` = 1;
/// `FUN_021A3964(core, 1)`.
pub fn act_walk<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    let mut slope_check = true;
    let mut code;
    if env.r32(inp.wrapping_add(0x10)) == 1 {
        code = 0x14;
        slope_check = false;
    } else if env.r32(inp.wrapping_add(0xC)) & 1 != 0 && env.r32(inp.wrapping_add(4)) & 2 != 0 {
        let core = core_of(env, grid);
        if c1(env, CORE_HAS_FORM, core) == 0 {
            code = 0x10;
        } else {
            code = 0x58;
            set_flags(env.mem(), grid, 0x20);
            let core = core_of(env, grid);
            let mut p = [0u8; 12];
            env.call(CORE_POS, &mut [CArg::V(core), CArg::OutBuf(&mut p)]);
            let mut a = [0u8; 4];
            if env.call(
                ATTR_AT,
                &mut [CArg::V(m), CArg::InBuf(&p), CArg::OutBuf(&mut a)],
            ) != 0
            {
                let v = c1(env, ATTR_VALUE, word(&a, 0));
                if c1(env, P_44C, v) != 0 {
                    code = 0xC;
                    slope_check = false;
                    clear_flags(env.mem(), grid, 0x20);
                }
            }
        }
    } else {
        code = 0xC;
    }
    if slope_check {
        let d = input_dir(env, inp);
        if slope(env, grid, d) == 1 {
            code = match code {
                0xC => 0xA7,
                0x10 => 0x4C,
                0x58 => 0xA3,
                c => c,
            };
        }
    }
    let d = input_dir(env, inp);
    let c = c2(env, ACMD_CODE, d, code);
    c2(env, SET_ACMD, m, c);
    env.w32(grid, 1);
    let core = core_of(env, grid);
    c2(env, CORE_SET_STATE, core, 1);
}

/// C: `FUN_021A4E78` (unlabeled routine at `0x021A4E78`): turn: acmd base 0x24; `+0` = 2; `FUN_021A3964(core, 2)`.
pub fn act_turn<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    simple_act(env, grid, m, inp, 0x24, 2, 2);
}

/// C: `FUN_021A4E9C` (unlabeled routine at `0x021A4E9C`): bump: acmd base 0x1C; `+0` = 3; `FUN_021A3964(core, 0)`;
/// no warp ahead: SE 0x55A.
pub fn act_bump<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    bump(env, grid, m, inp);
}

/// C: `FUN_021A4ED8` (unlabeled routine at `0x021A4ED8`): jump: acmd base 0x38; `+0` = 1; `FUN_021A3964(core, 1)`;
/// SE 0x55E.
pub fn act_jump<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    jump(env, grid, m, inp);
}

/// C: `FUN_021A4F04` (unlabeled routine at `0x021A4F04`): `+0` = 4.
pub fn act_state_4<E: FieldEnv>(env: &mut E, grid: u32, _m: u32, _inp: u32) {
    env.w32(grid, 4);
}

/// C: `FUN_021A4F0C` (unlabeled routine at `0x021A4F0C`): slide: dir = the input dir (9: [`free_dir`]); acmd
/// `FUN_02197E7C(dir, 0x34)`; dir lock on (`FUN_0216DB4C(m, 1)`), move bit on (`FUN_0216DCE8(m, 1)`), MMDL `+4 |=
/// 0x8000`; `+0` = 5.
pub fn act_slide<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    let mut d = input_dir(env, inp);
    if d == DIR_NONE {
        d = free_dir(env, grid);
    }
    let code = c2(env, ACMD_CODE, d, 0x34);
    c2(env, SET_ACMD, m, code);
    c2(env, DIR_LOCK, m, 1);
    c2(env, MOVE_BIT, m, 1);
    c2(env, BITS_SET, m, 0x8000);
    env.w32(grid, 5);
}

/// C: `FUN_021A4F50` (unlabeled routine at `0x021A4F50`): `+0` = 6.
pub fn act_state_6<E: FieldEnv>(env: &mut E, grid: u32, _m: u32, _inp: u32) {
    env.w32(grid, 6);
}

/// C: `FUN_021A4F58` (unlabeled routine at `0x021A4F58`): fall: pos = the MMDL position, h = the height there
/// (`FUN_02163E58`); pos.y > h (signed): y -= 0x8000, and when that is below h: y = h, MMDL `+4 &= ~0x8000`
/// (`FUN_0216D570`), `+0x1C` = 1; position = pos (`FUN_0216D89C`); `+0` = 7.
pub fn act_fall<E: FieldEnv>(env: &mut E, grid: u32, _m: u32, _inp: u32) {
    let m = mmdl_of(env, grid);
    let mut p = mmdl_pos(env, m);
    let mut h = [0u8; 4];
    env.call(
        HEIGHT_AT,
        &mut [CArg::V(m), CArg::InBuf(&p), CArg::OutBuf(&mut h)],
    );
    let h = word(&h, 0) as i32;
    let mut y = word(&p, 4) as i32;
    if y <= h {
        return;
    }
    y = y.wrapping_sub(0x8000);
    p[4..8].copy_from_slice(&(y as u32).to_le_bytes());
    if y < h {
        p[4..8].copy_from_slice(&(h as u32).to_le_bytes());
        c2(env, BITS_CLEAR, m, 0x8000);
        env.w32(grid.wrapping_add(0x1C), 1);
    }
    env.call(SET_POS, &mut [CArg::V(m), CArg::InBuf(&p)]);
    env.w32(grid, 7);
}

/// C: `FUN_021A4FA8` (unlabeled routine at `0x021A4FA8`): the cycle actions by move value 0..5: [`cycle_none`],
/// [`cycle_stand`], [`cycle_ride`], [`cycle_turn`], [`cycle_bump`], [`cycle_jump`].
pub fn cycle_act<E: FieldEnv>(env: &mut E, grid: u32, v: u32, inp: u32) {
    let m = mmdl_of(env, grid);
    match v {
        0 => cycle_none(env, grid, m, inp),
        1 => cycle_stand(env, grid, m, inp),
        2 => cycle_ride(env, grid, m, inp),
        3 => cycle_turn(env, grid, m, inp),
        4 => cycle_bump(env, grid, m, inp),
        5 => cycle_jump(env, grid, m, inp),
        _ => {}
    }
}

/// C: `FUN_021A5010` (unlabeled routine at `0x021A5010`): nothing.
pub fn cycle_none<E: FieldEnv>(_env: &mut E, _grid: u32, _m: u32, _inp: u32) {}

/// C: `FUN_021A5014` (unlabeled routine at `0x021A5014`): stand (as [`act_stand`]).
pub fn cycle_stand<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    stand(env, grid, m, inp);
}

/// C: `FUN_021A5044` (unlabeled routine at `0x021A5044`): ride: acmd base 0x14; not in slow mode: the attribute under
/// the player ([`attr_ahead`]`(grid, 9)`) satisfying `FUN_021AB41C`: 0x9B and the dust effect `FUN_021AC528(m,
/// FUN_02188CD0(fieldmap))`; then slope down ahead and still 0x14: 0x10. Acmd; `+0` = 1; `FUN_021A3964(core, 1)`.
pub fn cycle_ride<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    let mut code = 0x14;
    let slow = env.r32(inp.wrapping_add(0x10)) == 1;
    if !slow {
        let a = env.mem().push_local(4);
        let r = attr_ahead(env, grid, DIR_NONE, a);
        let attr = env.r32(a);
        env.mem().pop_local(a);
        if r == 1 {
            let v = c1(env, ATTR_VALUE, attr);
            if c1(env, P_41C, v) == 1 {
                code = 0x9B;
                let fm = env.r32(grid.wrapping_add(0x10));
                let ctrl = c1(env, FM_FX_CTRL, fm);
                c2(env, DUST, m, ctrl);
            }
        }
        let d = input_dir(env, inp);
        if slope(env, grid, d) == 1 && code == 0x14 {
            code = 0x10;
        }
    }
    let d = input_dir(env, inp);
    let c = c2(env, ACMD_CODE, d, code);
    c2(env, SET_ACMD, m, c);
    env.w32(grid, 1);
    let core = core_of(env, grid);
    c2(env, CORE_SET_STATE, core, 1);
}

/// C: `FUN_021A50BC` (unlabeled routine at `0x021A50BC`): turn (as [`act_turn`]).
pub fn cycle_turn<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    simple_act(env, grid, m, inp, 0x24, 2, 2);
}

/// C: `FUN_021A50E0` (unlabeled routine at `0x021A50E0`): bump (as [`act_bump`]).
pub fn cycle_bump<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    bump(env, grid, m, inp);
}

/// C: `FUN_021A511C` (unlabeled routine at `0x021A511C`): jump (as [`act_jump`]).
pub fn cycle_jump<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    jump(env, grid, m, inp);
}

/// C: `FUN_021A5148` (unlabeled routine at `0x021A5148`): the surf actions by move value 0..4: [`surf_none`],
/// [`surf_stand`], [`surf_swim`], [`surf_turn`], [`surf_bump`].
pub fn surf_act<E: FieldEnv>(env: &mut E, grid: u32, v: u32, inp: u32) {
    let m = mmdl_of(env, grid);
    match v {
        0 => surf_none(env, grid, m, inp),
        1 => surf_stand(env, grid, m, inp),
        2 => surf_swim(env, grid, m, inp),
        3 => surf_turn(env, grid, m, inp),
        4 => surf_bump(env, grid, m, inp),
        _ => {}
    }
}

/// C: `FUN_021A51A4` (unlabeled routine at `0x021A51A4`): nothing.
pub fn surf_none<E: FieldEnv>(_env: &mut E, _grid: u32, _m: u32, _inp: u32) {}

/// C: `FUN_021A51A8` (unlabeled routine at `0x021A51A8`): stand (as [`act_stand`]).
pub fn surf_stand<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    stand(env, grid, m, inp);
}

/// C: `FUN_021A51D8` (unlabeled routine at `0x021A51D8`): swim: acmd base 0x14 in slow mode, else 0x10; `+0` = 1;
/// `FUN_021A3964(core, 1)`.
pub fn surf_swim<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    let base = if env.r32(inp.wrapping_add(0x10)) == 1 {
        0x14
    } else {
        0x10
    };
    simple_act(env, grid, m, inp, base, 1, 1);
}

/// C: `FUN_021A5204` (unlabeled routine at `0x021A5204`): turn (as [`act_turn`]).
pub fn surf_turn<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    simple_act(env, grid, m, inp, 0x24, 2, 2);
}

/// C: `FUN_021A5228` (unlabeled routine at `0x021A5228`): bump (as [`act_bump`]).
pub fn surf_bump<E: FieldEnv>(env: &mut E, grid: u32, m: u32, inp: u32) {
    bump(env, grid, m, inp);
}

/// C: `FUN_021A5264` (unlabeled routine at `0x021A5264`): the hit bits of a step of `m` in `dir`: `FUN_02163994(m,
/// (s16)(gx + dx), gy, (s16)(gz + dz), dir)`; the attribute one step ahead of the MMDL position into `*a`
/// (`FUN_02163E34`; -1 when there is none); hit bit 2 with `FUN_021AB2DC` of the attribute value and an MMDL at that
/// position (`FUN_0216E5B0(MMDLSYS, &pos) == 1`): bit 2 cleared.
pub fn hit_check<E: FieldEnv>(env: &mut E, _grid: u32, m: u32, dir: u32, a: u32) -> u32 {
    let gx = c1(env, GX, m);
    let gy = c1(env, GY, m);
    let gz = c1(env, GZ, m);
    let dx = c1(env, DX, dir);
    let x = gx.wrapping_add(dx) as i16 as i32 as u32;
    let dz = c1(env, DZ, dir);
    let z = gz.wrapping_add(dz) as i16 as i32 as u32;
    let mut hit = env.callv(HIT_CHECK, &[m, x, gy, z, dir]);
    let mut p = mmdl_pos(env, m);
    step(env, dir, &mut p);
    if env.call(ATTR_AT, &mut [CArg::V(m), CArg::InBuf(&p), CArg::Out(a, 4)]) == 0 {
        env.w32(a, u32::MAX);
    }
    if hit & 2 != 0 {
        let at = env.r32(a);
        let v = c1(env, ATTR_VALUE, at);
        if c1(env, P_2DC, v) == 1 {
            let sys = c1(env, SYS_OF, m);
            if env.call(MMDL_AT, &mut [CArg::V(sys), CArg::InBuf(&p)]) == 1 {
                hit &= !2;
            }
        }
    }
    hit
}

/// C: `FUN_021A5304` (unlabeled routine at `0x021A5304`): `+8 |= bits`.
pub fn set_flags(mem: &mut Mem, grid: u32, bits: u32) {
    let a = grid.wrapping_add(8);
    mem.w32(a, mem.r32(a) | bits);
}

/// C: `FUN_021A530C` (unlabeled routine at `0x021A530C`): `+8 &= !bits`.
pub fn clear_flags(mem: &mut Mem, grid: u32, bits: u32) {
    let a = grid.wrapping_add(8);
    mem.w32(a, mem.r32(a) & !bits);
}

/// C: `FUN_021A5318` (unlabeled routine at `0x021A5318`): any of `bits` set in `+8`.
pub fn has_flags(mem: &Mem, grid: u32, bits: u32) -> bool {
    mem.r32(grid.wrapping_add(8)) & bits != 0
}

/// C: `FUN_021A5328` (unlabeled routine at `0x021A5328`): flag 1 on (forced move).
pub fn set_flag_1<E: FieldEnv>(env: &mut E, grid: u32) {
    set_flags(env.mem(), grid, 1);
}

/// C: `FUN_021A5334` (unlabeled routine at `0x021A5334`): flag 1 off.
pub fn clear_flag_1<E: FieldEnv>(env: &mut E, grid: u32) {
    clear_flags(env.mem(), grid, 1);
}

/// C: `FUN_021A5340` (unlabeled routine at `0x021A5340`): flag 1 set.
pub fn flag_1<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    has_flags(env.mem(), grid, 1) as u32
}

/// C: `FUN_021A5354` (unlabeled routine at `0x021A5354`): flag 2 on.
pub fn set_flag_2<E: FieldEnv>(env: &mut E, grid: u32) {
    set_flags(env.mem(), grid, 2);
}

/// C: `FUN_021A5360` (unlabeled routine at `0x021A5360`): flag 2 off.
pub fn clear_flag_2<E: FieldEnv>(env: &mut E, grid: u32) {
    clear_flags(env.mem(), grid, 2);
}

/// C: `FUN_021A536C` (unlabeled routine at `0x021A536C`): flag 4 off.
pub fn clear_flag_4<E: FieldEnv>(env: &mut E, grid: u32) {
    clear_flags(env.mem(), grid, 4);
}

/// C: `FUN_021A5378` (unlabeled routine at `0x021A5378`): flag 8 (ice spin) = `on == 1`.
pub fn set_spinning<E: FieldEnv>(env: &mut E, grid: u32, on: u32) {
    if on == 1 {
        set_flags(env.mem(), grid, 8);
    } else {
        clear_flags(env.mem(), grid, 8);
    }
}

/// C: `FUN_021A5390` (unlabeled routine at `0x021A5390`): flag 8 set.
pub fn spinning<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    has_flags(env.mem(), grid, 8) as u32
}

/// C: `FUN_021A53A4` (unlabeled routine at `0x021A53A4`): flag 0x10 (sliding) = `on == 1`.
pub fn set_sliding<E: FieldEnv>(env: &mut E, grid: u32, on: u32) {
    if on == 1 {
        set_flags(env.mem(), grid, 0x10);
    } else {
        clear_flags(env.mem(), grid, 0x10);
    }
}

/// C: `FUN_021A53BC` (unlabeled routine at `0x021A53BC`): flag 0x10 set.
pub fn sliding<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    has_flags(env.mem(), grid, 0x10) as u32
}

/// Cancel the MMDL acmd keeping its display dir: `FUN_0216D4B4(m)`, `FUN_0216D5E0(m, d)`, `FUN_0216D674(m, 0)`,
/// `FUN_021A3964(core, 0)`.
fn cancel<E: FieldEnv>(env: &mut E, grid: u32, m: u32) {
    let d = c1(env, DISP_DIR, m);
    c1(env, ACMD_CLEAR, m);
    c2(env, SET_DISP_DIR, m, d);
    c2(env, SET_2A, m, 0);
    let core = core_of(env, grid);
    c2(env, CORE_SET_STATE, core, 0);
}

/// C: `FUN_021A53D0` (unlabeled routine at `0x021A53D0`): end a turn: with move state 3, cancel the acmd keeping the
/// display dir.
pub fn end_turn<E: FieldEnv>(env: &mut E, grid: u32) {
    if env.r32(grid) == 3 {
        let m = mmdl_of(env, grid);
        cancel(env, grid, m);
    }
}

/// C: `FUN_021A5408` (unlabeled routine at `0x021A5408`): stop by move state (jump table): 1: 0; 2 / 3: cancel the
/// acmd keeping the display dir, 1; 0 and > 3: 1.
pub fn stop<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    match env.r32(grid) {
        1 => 0,
        2 | 3 => {
            let m = mmdl(env, grid);
            cancel(env, grid, m);
            1
        }
        _ => 1,
    }
}

/// C: `FUN_021A5458` (unlabeled routine at `0x021A5458`): flag 2 clear and the attribute under the player (MMDL
/// `+0x74`) satisfies `FUN_021AB348` (a forced-move tile).
pub fn on_forced_tile<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    let m = mmdl_of(env, grid);
    let a = c1(env, ATTR, m);
    if has_flags(env.mem(), grid, 2) {
        return 0;
    }
    let v = c1(env, ATTR_VALUE, a);
    (c1(env, P_348, v) == 1) as u32
}

/// C: `FUN_021A548C` (unlabeled routine at `0x021A548C`): [`on_forced_tile`] != 0 (the original passes 9 in r1,
/// which the callee does not read).
pub fn forced_tile<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    (on_forced_tile(env, grid) != 0) as u32
}

/// C: `FUN_021A54A0` (unlabeled routine at `0x021A54A0`): [`sliding`] == 1.
pub fn is_sliding<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    (sliding(env, grid) == 1) as u32
}

/// C: `FUN_021A54B4` (unlabeled routine at `0x021A54B4`): stop sliding: dir lock off, move bit off, MMDL `+4 |=
/// 0x140`; height refresh (`FUN_02164010`); gy = the tile of the position y (`(y >> 4) / 4096`, toward zero, s16);
/// attribute and grid position refresh (`FUN_021640B0`, `FUN_02163F10`).
pub fn stop_sliding<E: FieldEnv>(env: &mut E, grid: u32) {
    let m = mmdl_of(env, grid);
    c2(env, DIR_LOCK, m, 0);
    c2(env, MOVE_BIT, m, 0);
    c2(env, BITS_SET, m, 0x140);
    c1(env, HEIGHT_REFRESH, m);
    let p = mmdl_pos(env, m);
    let t = (word(&p, 4) as i32) >> 4;
    let q = (t.wrapping_add(((t >> 11) as u32 >> 20) as i32) << 4) >> 16;
    c2(env, SET_GY, m, q as u32);
    c1(env, ATTR_REFRESH, m);
    c1(env, POS_REFRESH, m);
}

/// C: `FUN_021A550C` (unlabeled routine at `0x021A550C`): end a forced move: flag 1 set: MMDL `+0 &= ~0x18`
/// (`FUN_0216D550`), flag 1 off.
pub fn end_forced<E: FieldEnv>(env: &mut E, grid: u32) {
    if flag_1(env, grid) == 1 {
        let m = mmdl_of(env, grid);
        c2(env, STATUS_CLEAR, m, 0x18);
        clear_flag_1(env, grid);
    }
}

/// C: `FUN_021A552C` (unlabeled routine at `0x021A552C`): forced-move table entry 0: [`end_forced`]; 0.
pub fn forced_none<E: FieldEnv>(env: &mut E, grid: u32, _dir: u32) -> u32 {
    end_forced(env, grid);
    0
}

/// C: `FUN_021A5538` (unlabeled routine at `0x021A5538`): forced-move table entry 1: d = the MMDL move dir;
/// [`hit_check`] in d hits: [`end_forced`], [`set_flag_2`], `FUN_021A3964(core, 0)`, 0; else flag 1 on, acmd
/// `FUN_02197E7C(d, 0x10)`, MMDL `+0 |= 0x18`, `+0` = 1, `FUN_021A3964(core, 1)`, 1.
pub fn forced_step<E: FieldEnv>(env: &mut E, grid: u32, _dir: u32) -> u32 {
    let m = mmdl_of(env, grid);
    let d = c1(env, MOVE_DIR, m);
    let a = env.mem().push_local(4);
    let hit = hit_check(env, grid, m, d, a);
    env.mem().pop_local(a);
    if hit != 0 {
        end_forced(env, grid);
        set_flag_2(env, grid);
        let core = core_of(env, grid);
        c2(env, CORE_SET_STATE, core, 0);
        return 0;
    }
    set_flag_1(env, grid);
    let code = c2(env, ACMD_CODE, d, 0x10);
    c2(env, SET_ACMD, m, code);
    c2(env, STATUS_SET, m, 0x18);
    env.w32(grid, 1);
    let core = core_of(env, grid);
    c2(env, CORE_SET_STATE, core, 1);
    1
}

/// C: `FUN_021A55A8` (unlabeled routine at `0x021A55A8`): `skip == 0` and the forced-move handler of ROM
/// `0x021D89EC[`[`on_forced_tile`]`]` ([`forced_none`], [`forced_step`]) answers 1: 1; else [`clear_flag_1`], 0.
pub fn forced_move<E: FieldEnv>(env: &mut E, grid: u32, dir: u32, skip: u32) -> u32 {
    if skip == 0 {
        let r = match on_forced_tile(env, grid) {
            0 => forced_none(env, grid, dir),
            1 => forced_step(env, grid, dir),
            i => unreachable!("forced-move table index {i} (the original reads past 0x021D89EC)"),
        };
        if r == 1 {
            return 1;
        }
    }
    clear_flag_1(env, grid);
    0
}

/// C: `FUN_021A55DC` (unlabeled routine at `0x021A55DC`): `FUN_021AB404(v) == 1` or `FUN_021AB410(v) == 1` (ice).
pub fn is_ice<E: FieldEnv>(env: &mut E, v: u32) -> u32 {
    (c1(env, P_404, v) == 1 || c1(env, P_410, v) == 1) as u32
}

/// C: `FUN_021A55FC` (unlabeled routine at `0x021A55FC`): `FUN_021AB404(v) == 1`.
pub fn is_ice_404<E: FieldEnv>(env: &mut E, v: u32) -> u32 {
    (c1(env, P_404, v) == 1) as u32
}

/// C: `FUN_021A5610` (unlabeled routine at `0x021A5610`): [`is_ice_404`] of the value of the attribute under the
/// player ([`attr_ahead`]`(grid, 9, &a)`; the answer of that call is not checked).
pub fn on_ice<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    let a = env.mem().push_local(4);
    attr_ahead(env, grid, DIR_NONE, a);
    let attr = env.r32(a);
    env.mem().pop_local(a);
    let v = c1(env, ATTR_VALUE, attr);
    (is_ice_404(env, v) == 1) as u32
}

/// C: `FUN_021A5630` (unlabeled routine at `0x021A5630`): a free direction to slide in: starting at the display dir
/// (`FUN_021A3920`), up to 4 tries: the attribute one step ahead ([`attr_ahead`] 1) that is not ice ([`is_ice`]) and
/// fails `FUN_021AB11C`: that dir; next try the dir of ROM `0x021D8A24` (3, 2, 0, 1). None: 9.
pub fn free_dir<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    let core = core_of(env, grid);
    let mut d = c1(env, CORE_DISP_DIR, core);
    let a = env.mem().push_local(4);
    let mut found = None;
    for _ in 0..4 {
        if attr_ahead(env, grid, d, a) == 1 {
            let attr = env.r32(a);
            let v = c1(env, ATTR_VALUE, attr);
            if is_ice(env, v) == 0 {
                let attr = env.r32(a);
                if c1(env, P_11C, attr) == 0 {
                    found = Some(d);
                    break;
                }
            }
        }
        // UNVERIFIED: d >= 4 reads past the 4-entry stack copy of the table (the original reads its stack)
        d = TURN_ORDER[d as usize] as u32;
    }
    env.mem().pop_local(a);
    found.unwrap_or(DIR_NONE)
}

/// C: `FUN_021A569C` (unlabeled routine at `0x021A569C`): [`hit_check`] | 0x100 when the attribute `*a` is ice
/// ([`is_ice`] of its value).
pub fn hit_check_ice<E: FieldEnv>(env: &mut E, grid: u32, m: u32, dir: u32, a: u32) -> u32 {
    let mut hit = hit_check(env, grid, m, dir, a);
    let attr = env.r32(a);
    let v = c1(env, ATTR_VALUE, attr);
    if is_ice(env, v) == 1 {
        hit |= 0x100;
    }
    hit
}

/// C: `FUN_021A56C0` (unlabeled routine at `0x021A56C0`): on the ground: the MMDL position y <= the height there
/// (`FUN_02163E58`, signed).
pub fn grounded<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    let m = mmdl_of(env, grid);
    let p = mmdl_pos(env, m);
    let mut h = [0u8; 4];
    env.call(
        HEIGHT_AT,
        &mut [CArg::V(m), CArg::InBuf(&p), CArg::OutBuf(&mut h)],
    );
    ((word(&p, 4) as i32) <= (word(&h, 0) as i32)) as u32
}

/// C: `FUN_021A56F4` (unlabeled routine at `0x021A56F4`): start the ice spin: [`set_spinning`] on;
/// `FUN_021A39AC(core, 7)` (form entry 7).
pub fn spin_start<E: FieldEnv>(env: &mut E, grid: u32) {
    set_spinning(env, grid, 1);
    let core = core_of(env, grid);
    c2(env, CORE_SET_ENTRY, core, 7);
}

/// C: `FUN_021A5708` (unlabeled routine at `0x021A5708`): end the ice spin: `+0x14` = 0, `+0x16` = 0;
/// [`set_sliding`] off; [`set_spinning`] off; `FUN_021A3998(core)`.
pub fn spin_end<E: FieldEnv>(env: &mut E, grid: u32) {
    env.w16(grid.wrapping_add(0x14), 0);
    env.w8(grid.wrapping_add(0x16), 0);
    set_sliding(env, grid, 0);
    set_spinning(env, grid, 0);
    let core = core_of(env, grid);
    c1(env, CORE_3998, core);
}

/// C: `FUN_021A5728` (unlabeled routine at `0x021A5728`): `+0xC` (the core).
pub fn core<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    core_of(env, grid)
}

/// C: `FUN_021A572C` (unlabeled routine at `0x021A572C`): `FUN_021A3960(+0xC)` (the player MMDL).
pub fn mmdl<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    mmdl_of(env, grid)
}

/// C: `FUN_021A5738` (unlabeled routine at `0x021A5738`): with the ice spin on: [`spin_end`], and with `keep == 1`
/// the counter `+0x14` is kept and `+0x16` = 1.
pub fn spin_reset<E: FieldEnv>(env: &mut E, grid: u32, keep: u32) {
    if spinning(env, grid) == 1 {
        let n = env.r16(grid.wrapping_add(0x14));
        spin_end(env, grid);
        if keep == 1 {
            env.w16(grid.wrapping_add(0x14), n);
            env.w8(grid.wrapping_add(0x16), 1);
        }
    }
}

/// C: `FUN_021A5764` (unlabeled routine at `0x021A5764`): `+0x1C`.
pub fn get_1c<E: FieldEnv>(env: &mut E, grid: u32) -> u32 {
    env.r32(grid.wrapping_add(0x1C))
}

/// C: `FUN_021A5768` (unlabeled routine at `0x021A5768`): `+0x1C` = 1 when `on == 1`, else 0.
pub fn set_1c(mem: &mut Mem, grid: u32, on: u32) {
    mem.w32(grid.wrapping_add(0x1C), (on == 1) as u32);
}

/// C: `FUN_021A5778` (unlabeled routine at `0x021A5778`): the move state `+0` is 3.
pub fn state_is_3(mem: &Mem, grid: u32) -> bool {
    mem.r32(grid) == 3
}

/// C: `FUN_021A5788` (unlabeled routine at `0x021A5788`): the attribute at the core position (`FUN_021A3898`), one
/// step ahead in `dir` unless 9, into `*a` (`FUN_02163E34`; its answer).
pub fn attr_ahead<E: FieldEnv>(env: &mut E, grid: u32, dir: u32, a: u32) -> u32 {
    let core = core_of(env, grid);
    let mut p = [0u8; 12];
    env.call(CORE_POS, &mut [CArg::V(core), CArg::OutBuf(&mut p)]);
    if dir != DIR_NONE {
        step(env, dir, &mut p);
    }
    let m = mmdl_of(env, grid);
    env.call(ATTR_AT, &mut [CArg::V(m), CArg::InBuf(&p), CArg::Out(a, 4)])
}

/// C: `FUN_021A57C0` (unlabeled routine at `0x021A57C0`): the slope one step ahead in `dir`: the height there
/// (`FUN_02163E58` answering 1) above the current y: 1, below: 2; else 0 (signed compares).
pub fn slope<E: FieldEnv>(env: &mut E, grid: u32, dir: u32) -> u32 {
    let m = mmdl_of(env, grid);
    let mut p = mmdl_pos(env, m);
    let y = word(&p, 4) as i32;
    step(env, dir, &mut p);
    let mut h = [0u8; 4];
    if env.call(
        HEIGHT_AT,
        &mut [CArg::V(m), CArg::InBuf(&p), CArg::OutBuf(&mut h)],
    ) == 1
    {
        let h = word(&h, 0) as i32;
        if y < h {
            return 1;
        }
        if y > h {
            return 2;
        }
    }
    0
}

/// C: `FUN_021A5810` (unlabeled routine at `0x021A5810`): a warp one step ahead in `dir` that exits in `dir`: the
/// EVENTDATA_SYSTEM of the GAMEDATA of the fieldmap's GAMESYS; warp index at the stepped position
/// (`FUN_02162564`); 0xFFFF: 0; else `FUN_021626B8(FUN_021625CC(sys, (u16) i), dir) != 0`.
pub fn warp_ahead<E: FieldEnv>(env: &mut E, grid: u32, dir: u32) -> u32 {
    let m = mmdl_of(env, grid);
    let fm = env.r32(grid.wrapping_add(0x10));
    let gs = c1(env, FM_GAMESYS, fm);
    let gd = c1(env, GAMEDATA_OF, gs);
    let ev = c1(env, EVENTDATA_OF, gd);
    let mut p = mmdl_pos(env, m);
    step(env, dir, &mut p);
    let i = env.call(WARP_AT, &mut [CArg::V(ev), CArg::InBuf(&p)]);
    if i == 0xFFFF {
        return 0;
    }
    let w = c2(env, WARP_REC, ev, i & 0xFFFF);
    (c2(env, WARP_EXITS, w, dir) != 0) as u32
}

#[cfg(test)]
mod tests;
