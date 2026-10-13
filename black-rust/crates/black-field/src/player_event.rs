//! `field_player_grid_event.c` (overlay 21, `0x021C1024..0x021C1A08`): the surf start task (jump onto the water:
//! 021C1024 creates it, proc 021C10B4, work 0x18) and the surf end task (jump off onto land: 021C1228, proc 021C1988
//! over the step table ROM `0x021DA128`, or 021C16AC for a jump down; work 0x4C), plus the MMDL-in-front lookup they
//! share with the boulder push (021C19B4).
//!
//! F3a-spec specced all 17 functions (`tools/rom-diff/specs/black-field/player_event.spec`, header: task works); F3a ported
//! all of them.

use crate::hooks::{CArg, FieldEnv, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_1024,
    0x021C_1094,
    0x021C_10A0,
    0x021C_10B4,
    0x021C_1228,
    0x021C_12C4,
    0x021C_12D0,
    0x021C_12E4,
    0x021C_1318,
    0x021C_146C,
    0x021C_14F8,
    0x021C_157C,
    0x021C_161C,
    0x021C_1670,
    0x021C_16AC,
    0x021C_1988,
    0x021C_19B4,
];

/// `FUN_02030EE8`: the work of a task (task system, main).
const TASK_WORK: u32 = 0x0203_0EE8;
/// `FUN_02030EAC`: delete a task.
const TASK_DELETE: u32 = 0x0203_0EAC;
/// `FUN_0216D8E0`: set the draw offset VecFx32 of an MMDL (fldmmdl, F4a).
const MMDL_SET_DRAW_OFFSET: u32 = 0x0216_D8E0;

/// C: `FUN_021C1094` (unlabeled routine at `0x021C1094`): the surf start task is done (work `+4`, u16).
pub fn surf_start_done<E: FieldEnv>(env: &mut E, task: u32) -> u16 {
    let w = env.callv(TASK_WORK, &[task]);
    env.r16(w.wrapping_add(4))
}

/// C: `FUN_021C10A0` (unlabeled routine at `0x021C10A0`): free the surf start task's work and delete the task.
pub fn surf_start_delete<E: FieldEnv>(env: &mut E, task: u32) {
    let w = env.callv(TASK_WORK, &[task]);
    env.callv(HEAP_FREE, &[w]);
    env.callv(TASK_DELETE, &[task]);
}

/// C: `FUN_021C12C4` (unlabeled routine at `0x021C12C4`): the surf end task is done (work `+6`, u16).
pub fn surf_end_done<E: FieldEnv>(env: &mut E, task: u32) -> u16 {
    let w = env.callv(TASK_WORK, &[task]);
    env.r16(w.wrapping_add(6))
}

/// C: `FUN_021C12D0` (unlabeled routine at `0x021C12D0`): free the surf end task's work and delete the task.
pub fn surf_end_delete<E: FieldEnv>(env: &mut E, task: u32) {
    surf_start_delete(env, task);
}

/// C: `FUN_021C12E4` (unlabeled routine at `0x021C12E4`): the MMDL draw offset while riding the surf Pokemon: the
/// VecFx32 at `off` raised by (0, 0x1000, 0x5000) (wrapping), passed to `FUN_0216D8E0` as a local copy.
pub fn set_ride_offset<E: FieldEnv>(env: &mut E, m: u32, off: u32) {
    let mut v = [0u8; 12];
    for (i, add) in [0u32, 0x1000, 0x5000].into_iter().enumerate() {
        let w = env.r32(off.wrapping_add(4 * i as u32)).wrapping_add(add);
        v[4 * i..4 * i + 4].copy_from_slice(&w.to_le_bytes());
    }
    env.call(MMDL_SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
}

// other units (force stubs in this area's spec), named by address
const HEAP_ALLOC_DEBUG: u32 = 0x0203_0734; // Heap_AllocDebug(heap, size, flag, file, line)
const TASK_ADD: u32 = 0x0203_0DA8; // task add (sys, proc, work, priority)
const SE_PLAY: u32 = 0x0200_61E4; // sound: play SE (id)
const HEIGHT_AT: u32 = 0x0216_3E58; // mmdl_move: height at a position (m, &pos, &y)
const DX_OF: u32 = 0x0216_41E8; // mmdl_move: dx of dir
const DZ_OF: u32 = 0x0216_41F8; // mmdl_move: dz of dir
const POS_STEP: u32 = 0x0216_4208; // mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
const TILE_CENTRE: u32 = 0x0216_4280; // mmdl_move: grid (x, z) -> VecFx32 tile centre (x, z, &pos)
const SET_ACMD: u32 = 0x0216_D410; // fldmmdl: MMDL set acmd (m, code)
const ACMD_DONE: u32 = 0x0216_D454; // fldmmdl: MMDL acmd done (m)
const ACMD_END: u32 = 0x0216_D480; // fldmmdl: MMDL acmd end (m)
const DISPLAY_DIR: u32 = 0x0216_D5F8; // fldmmdl: MMDL display dir
const SYS_OF: u32 = 0x0216_D67C; // fldmmdl: MMDL -> MMDLSYS
const GX: u32 = 0x0216_D83C; // fldmmdl: MMDL gx
const GZ: u32 = 0x0216_D86C; // fldmmdl: MMDL gz
const GET_POS: u32 = 0x0216_D88C; // fldmmdl: MMDL position VecFx32 -> out
const SET_POS: u32 = 0x0216_D89C; // fldmmdl: MMDL position VecFx32 = *v
const POS_Y: u32 = 0x0216_D8AC; // fldmmdl: MMDL position y (fx32)
const DIR_LOCK: u32 = 0x0216_DB4C; // fldmmdl: MMDL dir lock (m, on)
const AT_GRID_HEIGHT: u32 = 0x0216_DDDC; // fldmmdl: MMDL at grid + height (sys, x, z, y, height, flag)
const SET_POS_DIR: u32 = 0x0216_E014; // fldmmdl: MMDL set position + dir (m, &pos, dir)
const FX_CTRL: u32 = 0x0218_8CD0; // fieldmap +0x94 (field effect control)
const ENCOUNT: u32 = 0x0218_8CE8; // fieldmap +0x90 (encounter object)
const TASK_SYS: u32 = 0x0218_8D30; // fieldmap +0x110 (task system)
const ACMD_CODE: u32 = 0x0219_7E7C; // fldmmdl_acmd: acmd code of (dir, base)
const P_SET_REQUESTS: u32 = 0x021A_2F48; // player: set requests (player, bits)
const P_REQUESTS_DONE: u32 = 0x021A_2F54; // player: requests done? (player)
const P_SET_SURF_TASK: u32 = 0x021A_2F8C; // player: set the surf effect task (player, v)
const P_SURF_TASK: u32 = 0x021A_2F98; // player: surf effect task (player)
const P_FIELDMAP: u32 = 0x021A_3068; // player: fieldmap
const P_MMDL: u32 = 0x021A_3074; // player: the player MMDL
const P_SET_35A0: u32 = 0x021A_35A0; // player: grid one-shot flag (player, on)
const P_GRID: u32 = 0x021A_36E0; // player: FIELD_PLAYER +8 (grid player)
const G_MMDL: u32 = 0x021A_572C; // player_grid: the player MMDL (grid)
const ENC_AT: u32 = 0x021A_A900; // field_encount: (encount, &s16 grid xyz) predicate
const ENC_SET: u32 = 0x021A_AB1C; // field_encount (encount, 1)
const ATTR_VALUE: u32 = 0x021A_B0F0; // attribute value of an attribute
const ATTR_2E8: u32 = 0x021A_B2E8; // map attribute predicate (attribute value)
const FX_REGISTER: u32 = 0x021A_BE30; // field effect: register (ctrl, &id, n)
const FX_UNREGISTER: u32 = 0x021A_BE64; // field effect: unregister (ctrl, id)
const FX_TASK_DELETE: u32 = 0x021A_C154; // field effect: delete task
const SURF_ADD: u32 = 0x021A_CB38; // namipoke: add the surf effect (ctrl, dir, &pos, m, 0)
const SURF_MODE: u32 = 0x021A_CBB8; // namipoke: set the surf effect mode (task, mode)
const SPLASH_ADD: u32 = 0x021A_D564; // namipoke: splash (ctrl, kind, surf task)
const SPLASH_OVER: u32 = 0x021A_D5CC; // namipoke: splash over (task)
const SHAKE: u32 = 0x021C_6B18; // fld3d / camera shake (fieldmap, 0x10, amount)

/// Address of the string `"field_player_grid_event.c"`.
const FILE_GRID_EVENT: u32 = 0x021D_E3AC;
/// The surf start task proc (`FUN_021C10B4`, Thumb).
const SURF_START_PROC: u32 = 0x021C_10B5;
/// The surf end task proc (`FUN_021C1988`, Thumb).
const SURF_END_PROC: u32 = 0x021C_1989;
/// ROM `0x021DA128`: the step table of the surf end task (Thumb addresses of `FUN_021C1318` .. `FUN_021C1670`).
pub const SURF_END_STEPS: [u32; 6] = [
    0x021C_1319,
    0x021C_146D,
    0x021C_14F9,
    0x021C_157D,
    0x021C_161D,
    0x021C_1671,
];
/// The field effect id registered for the splash (`FUN_021ABE30(ctrl, &9, 1)`).
const SPLASH_FX_ID: u32 = 9;

fn c1<E: FieldEnv>(env: &mut E, addr: u32, a: u32) -> u32 {
    env.callv(addr, &[a])
}

fn c2<E: FieldEnv>(env: &mut E, addr: u32, a: u32, b: u32) -> u32 {
    env.callv(addr, &[a, b])
}

/// `((v >> 4) + bias) >> 12` (toward zero, arithmetic): the grid coordinate of an fx32 value (the caller narrows it).
fn grid_of(v: u32) -> u32 {
    let t = (v as i32) >> 4;
    ((t + if t < 0 { 0xFFF } else { 0 }) >> 12) as u32
}

/// The grid coordinate as the s16 register value (`lsls #4; asrs #16`).
fn grid_s16(v: u32) -> u32 {
    grid_of(v) as i16 as i32 as u32
}

/// Signed `v / 16`, toward zero (`asrs #3; lsrs #28; adds; asrs #4`).
fn div16(v: u32) -> u32 {
    ((v as i32) / 16) as u32
}

/// Signed `v / 2`, toward zero (`lsrs #31; adds; asrs #1`).
fn div2(v: u32) -> u32 {
    ((v as i32) / 2) as u32
}

/// `ldrsh +2; adds 1; strh`: advance the step of a surf end work.
fn step_s16<E: FieldEnv>(env: &mut E, w: u32) {
    let s = env.r16(w.wrapping_add(2)) as i16 as i32;
    env.w16(w.wrapping_add(2), s.wrapping_add(1) as u16);
}

/// `ldrh +0; adds 1; strh`: advance the step of a surf start work.
fn step_u16<E: FieldEnv>(env: &mut E, w: u32) {
    let s = env.r16(w) as u32;
    env.w16(w, s.wrapping_add(1) as u16);
}

fn get_pos<E: FieldEnv>(env: &mut E, m: u32, out: u32) {
    env.call(GET_POS, &mut [CArg::V(m), CArg::Out(out, 12)]);
}

fn set_pos<E: FieldEnv>(env: &mut E, m: u32, p: u32) {
    env.call(SET_POS, &mut [CArg::V(m), CArg::In(p, 12)]);
}

fn pos_step<E: FieldEnv>(env: &mut E, dir: u32, p: u32, amount: u32) {
    env.call(
        POS_STEP,
        &mut [CArg::V(dir), CArg::In(p, 12), CArg::V(amount)],
    );
}

fn tile_centre<E: FieldEnv>(env: &mut E, x: u32, z: u32, out: u32) {
    env.call(
        TILE_CENTRE,
        &mut [CArg::V(x), CArg::V(z), CArg::Out(out, 12)],
    );
}

fn height_at<E: FieldEnv>(env: &mut E, m: u32, p: u32, out: u32) -> u32 {
    env.call(
        HEIGHT_AT,
        &mut [CArg::V(m), CArg::In(p, 12), CArg::Out(out, 4)],
    )
}

/// `FUN_021ABE30(ctrl, &9, 1)` with the id on a local.
fn fx_register_splash<E: FieldEnv>(env: &mut E, ctrl: u32) {
    let id = SPLASH_FX_ID.to_le_bytes();
    env.call(
        FX_REGISTER,
        &mut [CArg::V(ctrl), CArg::InBuf(&id), CArg::V(1)],
    );
}

/// The splash kind of a jump in direction `dir`: 3 for left / right (`(u16)(dir - 2) <= 1`), else 1.
fn splash_kind(dir: u32) -> u32 {
    if (dir.wrapping_add(0xFFFE) & 0xFFFF) <= 1 {
        3
    } else {
        1
    }
}

/// C: `FUN_021C1024` (unlabeled routine at `0x021C1024`): create the surf start task: work = `Heap_AllocDebug((heap &
/// 0x7FFF) | 0x8000, 0x18, 1, "field_player_grid_event.c", 0x1E6)`; `+0xC` = the fieldmap of `player`
/// (`FUN_021A3068`), `+6` = `dir` (u16), `+0x10` = `player`, `+0x14` = its grid player (`FUN_021A36E0`); `+8` = 1 when
/// `FUN_021AB2E8(FUN_021AB0F0(attr)) == 1` (two-tile jump); returns `FUN_02030DA8(FUN_02188D30(fieldmap),
/// FUN_021C10B4, work, 0)`.
pub fn surf_start_new<E: FieldEnv>(
    env: &mut E,
    player: u32,
    dir: u32,
    attr: u32,
    heap: u32,
) -> u32 {
    let h = ((heap & 0x7FFF) | 0x8000) & 0xFFFF;
    let w = env.callv(HEAP_ALLOC_DEBUG, &[h, 0x18, 1, FILE_GRID_EVENT, 0x1E6]);
    let fm = c1(env, P_FIELDMAP, player);
    env.w32(w.wrapping_add(0xC), fm);
    env.w16(w.wrapping_add(6), dir as u16);
    env.w32(w.wrapping_add(0x10), player);
    let g = c1(env, P_GRID, player);
    env.w32(w.wrapping_add(0x14), g);
    let v = c1(env, ATTR_VALUE, attr);
    if c1(env, ATTR_2E8, v) == 1 {
        env.w16(w.wrapping_add(8), 1);
    }
    let fm = env.r32(w.wrapping_add(0xC));
    let ts = c1(env, TASK_SYS, fm);
    env.callv(TASK_ADD, &[ts, SURF_START_PROC, w, 0])
}

/// C: `FUN_021C10B4` (unlabeled routine at `0x021C10B4`): the surf start task proc, by the step `+0` (u16): 0: the
/// position = the MMDL position with x / z replaced by the tile centre of its grid position (`FUN_02164280`), moved one
/// step (0x10000, 0x20000 with `+8`) in the display dir (`FUN_02164208`), y = the height there (`FUN_02163E58`, 0 by
/// default); the surf effect `FUN_021ACB38(control, dir, &pos, m, 0)` goes to the player (`FUN_021A2F8C`); when
/// `FUN_021AA900(encount, &s16 grid xyz)`, `FUN_021AAB1C(encount, 1)`; next step. 1: frame `+2` (s16) += 1, past 15:
/// next step. 2: acmd `FUN_02197E7C(dir, 0x34)` (`0x38` with `+8`) on the MMDL, next step. 3: once the acmd is done
/// (`FUN_0216D454`): `FUN_0216D480`, surf effect mode 1 (`FUN_021ACBB8`), request 4 (surf, `FUN_021A2F48`), next step,
/// then as 4. 4: once the requests are done (`FUN_021A2F54 == 1`): `FUN_021A35A0(player, 1)`, done `+4` = 1, next step.
/// 5 and above: nothing.
pub fn surf_start_proc<E: FieldEnv>(env: &mut E, _task: u32, w: u32) {
    let player = env.r32(w.wrapping_add(0x10));
    let m = c1(env, P_MMDL, player);
    let step = env.r16(w) as u32;
    match step {
        0 => {
            let dir = c1(env, DISPLAY_DIR, m);
            let gx = c1(env, GX, m);
            let gz = c1(env, GZ, m);
            let pos = env.mem().push_local(12);
            let y = env.mem().push_local(4);
            get_pos(env, m, pos);
            tile_centre(env, gx, gz, pos);
            let amount = if env.r16(w.wrapping_add(8)) == 1 {
                0x2_0000
            } else {
                0x1_0000
            };
            pos_step(env, dir, pos, amount);
            env.w32(y, 0);
            height_at(env, m, pos, y);
            let yv = env.r32(y);
            env.w32(pos.wrapping_add(4), yv);
            let fm = env.r32(w.wrapping_add(0xC));
            let ctrl = c1(env, FX_CTRL, fm);
            let t = env.call(
                SURF_ADD,
                &mut [
                    CArg::V(ctrl),
                    CArg::V(dir),
                    CArg::In(pos, 12),
                    CArg::V(m),
                    CArg::V(0),
                ],
            );
            let player = env.r32(w.wrapping_add(0x10));
            c2(env, P_SET_SURF_TASK, player, t);
            let fm = env.r32(w.wrapping_add(0xC));
            let enc = c1(env, ENCOUNT, fm);
            let mut g = [0u8; 6];
            for i in 0..3 {
                let v = grid_of(env.r32(pos.wrapping_add(4 * i))) as u16;
                g[2 * i as usize..2 * i as usize + 2].copy_from_slice(&v.to_le_bytes());
            }
            if env.call(ENC_AT, &mut [CArg::V(enc), CArg::InBuf(&g)]) != 0 {
                c2(env, ENC_SET, enc, 1);
            }
            env.mem().pop_local(y);
            env.mem().pop_local(pos);
            step_u16(env, w);
        }
        1 => {
            let f = (env.r16(w.wrapping_add(2)) as i16).wrapping_add(1);
            env.w16(w.wrapping_add(2), f as u16);
            if f > 15 {
                env.w16(w, step.wrapping_add(1) as u16);
            }
        }
        2 => {
            let dir = c1(env, DISPLAY_DIR, m);
            let mut code = c2(env, ACMD_CODE, dir, 0x34);
            if env.r16(w.wrapping_add(8)) == 1 {
                code = c2(env, ACMD_CODE, dir, 0x38);
            }
            c2(env, SET_ACMD, m, code);
            step_u16(env, w);
        }
        3 | 4 => {
            if step == 3 {
                if c1(env, ACMD_DONE, m) == 0 {
                    return;
                }
                c1(env, ACMD_END, m);
                let player = env.r32(w.wrapping_add(0x10));
                let t = c1(env, P_SURF_TASK, player);
                c2(env, SURF_MODE, t, 1);
                let player = env.r32(w.wrapping_add(0x10));
                c2(env, P_SET_REQUESTS, player, 4);
                step_u16(env, w);
            }
            let player = env.r32(w.wrapping_add(0x10));
            if c1(env, P_REQUESTS_DONE, player) == 1 {
                let player = env.r32(w.wrapping_add(0x10));
                c2(env, P_SET_35A0, player, 1);
                env.w16(w.wrapping_add(4), 1);
                step_u16(env, w);
            }
        }
        _ => {}
    }
}

/// C: `FUN_021C1228` (unlabeled routine at `0x021C1228`): create the surf end task: work = `Heap_AllocDebug((heap &
/// 0x7FFF) | 0x8000, 0x4C, 1, "field_player_grid_event.c", 0x2B3)`; `+4` = `dir` (u16), `+0x3C` = `player`, `+0x40` =
/// its fieldmap; the MMDL position moved 0x20000 in `dir` (`FUN_02164208` on a copy); `+0` = 1 when the height there
/// (`FUN_02163E58`) is above the MMDL's y (signed); `+0x44` = the grid player; returns `FUN_02030DA8(task system,
/// FUN_021C1988, work, 0)`.
pub fn surf_end_new<E: FieldEnv>(env: &mut E, player: u32, dir: u32, _a: u32, heap: u32) -> u32 {
    let h = ((heap & 0x7FFF) | 0x8000) & 0xFFFF;
    let w = env.callv(HEAP_ALLOC_DEBUG, &[h, 0x4C, 1, FILE_GRID_EVENT, 0x2B3]);
    env.w16(w.wrapping_add(4), dir as u16);
    env.w32(w.wrapping_add(0x3C), player);
    let fm = c1(env, P_FIELDMAP, player);
    env.w32(w.wrapping_add(0x40), fm);
    let m = c1(env, P_MMDL, player);
    let a = env.mem().push_local(12);
    let b = env.mem().push_local(12);
    let y = env.mem().push_local(4);
    get_pos(env, m, a);
    env.mem().copy(b, a, 12);
    let d = env.r16(w.wrapping_add(4)) as u32;
    pos_step(env, d, b, 0x2_0000);
    height_at(env, m, b, y);
    if (env.r32(y) as i32) > (env.r32(a.wrapping_add(4)) as i32) {
        env.w16(w, 1);
    }
    env.mem().pop_local(y);
    env.mem().pop_local(b);
    env.mem().pop_local(a);
    let player = env.r32(w.wrapping_add(0x3C));
    let g = c1(env, P_GRID, player);
    env.w32(w.wrapping_add(0x44), g);
    let fm = env.r32(w.wrapping_add(0x40));
    let ts = c1(env, TASK_SYS, fm);
    env.callv(TASK_ADD, &[ts, SURF_END_PROC, w, 0])
}

/// C: `FUN_021C1318` (unlabeled routine at `0x021C1318`): surf end step 0 (jump off): dir lock on (`FUN_0216DB4C(m,
/// 1)`); from the MMDL position A, copies C / D moved 0x10000 / 0x20000 in the dir; C = the tile centre of C's grid
/// position, the tile centre of D's goes to `+0x24` and is then overwritten by D itself; the height at D to `+0x28`;
/// `+0x30` / `+0x38` = (C - A) x / z halved then / 16 (toward zero); `+0x34` = 0x6000, `+0x1C` = 0x1000; surf effect mode
/// 2; field effect 9 registered; `+0x48` = the splash `FUN_021AD564(control, kind, surf task)` (kind 3 for left /
/// right, else 1); camera shake `FUN_021C6B18(fieldmap, 0x10, 0x20000)`; SE 0x6E0; next step; returns 1.
pub fn surf_end_jump<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let player = env.r32(w.wrapping_add(0x3C));
    let m = c1(env, P_MMDL, player);
    c2(env, DIR_LOCK, m, 1);
    let a = env.mem().push_local(12);
    let c = env.mem().push_local(12);
    let d = env.mem().push_local(12);
    get_pos(env, m, a);
    env.mem().copy(c, a, 12);
    env.mem().copy(d, a, 12);
    let dir = env.r16(w.wrapping_add(4)) as u32;
    pos_step(env, dir, c, 0x1_0000);
    let dir = env.r16(w.wrapping_add(4)) as u32;
    pos_step(env, dir, d, 0x2_0000);
    let (x, z) = (env.r32(c), env.r32(c.wrapping_add(8)));
    tile_centre(env, grid_s16(x), grid_s16(z), c);
    let (x, z) = (env.r32(d), env.r32(d.wrapping_add(8)));
    tile_centre(env, grid_s16(x), grid_s16(z), w.wrapping_add(0x24));
    env.mem().copy(w.wrapping_add(0x24), d, 12);
    height_at(env, m, d, w.wrapping_add(0x28));
    let dx = env.r32(c).wrapping_sub(env.r32(a));
    env.w32(w.wrapping_add(0x30), div16(div2(dx)));
    let dz = env
        .r32(c.wrapping_add(8))
        .wrapping_sub(env.r32(a.wrapping_add(8)));
    env.w32(w.wrapping_add(0x38), div16(div2(dz)));
    env.mem().pop_local(d);
    env.mem().pop_local(c);
    env.mem().pop_local(a);
    env.w32(w.wrapping_add(0x34), 0x6000);
    env.w32(w.wrapping_add(0x1C), 0x1000);
    let player = env.r32(w.wrapping_add(0x3C));
    let t = c1(env, P_SURF_TASK, player);
    c2(env, SURF_MODE, t, 2);
    let kind = splash_kind(env.r16(w.wrapping_add(4)) as u32);
    let fm = env.r32(w.wrapping_add(0x40));
    let ctrl = c1(env, FX_CTRL, fm);
    fx_register_splash(env, ctrl);
    let s = env.callv(SPLASH_ADD, &[ctrl, kind, t]);
    env.w32(w.wrapping_add(0x48), s);
    let fm = env.r32(w.wrapping_add(0x40));
    env.callv(SHAKE, &[fm, 0x10, 0x2_0000]);
    c1(env, SE_PLAY, 0x6E0);
    step_s16(env, w);
    1
}

/// The hop frame shared by [`surf_end_hop`] and step 1 of [`surf_end_down`]: the MMDL position += (`+0x30`, 0, `+0x38`);
/// the bob `+0x1C` negated; frame `+8` (u16) += 1; at 16: `+8` = 0, `+0x1C` = 0, next step, the splash task deleted
/// (`FUN_021AC154`) and replaced by `FUN_021AD564(control, 2, surf task)`; then the ride offset ([`set_ride_offset`]).
fn hop_frame<E: FieldEnv>(env: &mut E, w: u32, m: u32) {
    let p = env.mem().push_local(12);
    get_pos(env, m, p);
    let x = env.r32(p).wrapping_add(env.r32(w.wrapping_add(0x30)));
    env.w32(p, x);
    let z = env
        .r32(p.wrapping_add(8))
        .wrapping_add(env.r32(w.wrapping_add(0x38)));
    env.w32(p.wrapping_add(8), z);
    set_pos(env, m, p);
    env.mem().pop_local(p);
    let b = env.r32(w.wrapping_add(0x1C)).wrapping_neg();
    env.w32(w.wrapping_add(0x1C), b);
    let f = env.r16(w.wrapping_add(8)).wrapping_add(1);
    env.w16(w.wrapping_add(8), f);
    if env.r16(w.wrapping_add(8)) >= 0x10 {
        env.w16(w.wrapping_add(8), 0);
        env.w32(w.wrapping_add(0x1C), 0);
        step_s16(env, w);
        let s = env.r32(w.wrapping_add(0x48));
        c1(env, FX_TASK_DELETE, s);
        let player = env.r32(w.wrapping_add(0x3C));
        let t = c1(env, P_SURF_TASK, player);
        let fm = env.r32(w.wrapping_add(0x40));
        let ctrl = c1(env, FX_CTRL, fm);
        let s = env.callv(SPLASH_ADD, &[ctrl, 2, t]);
        env.w32(w.wrapping_add(0x48), s);
    }
    set_ride_offset(env, m, w.wrapping_add(0x18));
}

/// C: `FUN_021C146C` (unlabeled routine at `0x021C146C`): surf end step 1 (hop): one hop frame (the MMDL moved by
/// `+0x30` / `+0x38`, the bob negated, 16 frames then the next step and splash kind 2) and the ride offset; returns 0.
pub fn surf_end_hop<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let player = env.r32(w.wrapping_add(0x3C));
    let m = c1(env, P_MMDL, player);
    hop_frame(env, w, m);
    0
}

/// C: `FUN_021C14F8` (unlabeled routine at `0x021C14F8`): surf end step 2 (rise): MMDL y += `+0x34`; at or above the
/// target height `+0x28` (signed): y = target, next step, `+0x30` / `+0x38` = (target - pos) x / z / 16, `+0x34` =
/// 0x6000, `+0x10` = -0x8000 / 11 (`_s32_div_f`), camera shake `FUN_021C6B18(fieldmap, 0x10, 0xFFFE0000)`, the splash
/// task deleted; the position written back; returns 0.
pub fn surf_end_rise<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let player = env.r32(w.wrapping_add(0x3C));
    let m = c1(env, P_MMDL, player);
    let p = env.mem().push_local(12);
    get_pos(env, m, p);
    let y = env
        .r32(p.wrapping_add(4))
        .wrapping_add(env.r32(w.wrapping_add(0x34)));
    env.w32(p.wrapping_add(4), y);
    let target = env.r32(w.wrapping_add(0x28));
    if (y as i32) >= (target as i32) {
        env.w32(p.wrapping_add(4), target);
        step_s16(env, w);
        let dx = env.r32(w.wrapping_add(0x24)).wrapping_sub(env.r32(p));
        env.w32(w.wrapping_add(0x30), div16(dx));
        env.w32(w.wrapping_add(0x10), 0xFFFF_8000);
        let dz = env
            .r32(w.wrapping_add(0x2C))
            .wrapping_sub(env.r32(p.wrapping_add(8)));
        env.w32(w.wrapping_add(0x38), div16(dz));
        env.w32(w.wrapping_add(0x34), 0x6000);
        env.w32(w.wrapping_add(0x10), (-0x8000i32 / 11) as u32);
        let fm = env.r32(w.wrapping_add(0x40));
        env.callv(SHAKE, &[fm, 0x10, 0xFFFE_0000]);
        let s = env.r32(w.wrapping_add(0x48));
        c1(env, FX_TASK_DELETE, s);
    }
    set_pos(env, m, p);
    env.mem().pop_local(p);
    0
}

/// One axis of the arc of [`surf_end_arc`]: `v + d`, clamped at `t` on the side `v` approaches it from (signed).
fn approach(v: u32, d: u32, t: u32) -> u32 {
    let n = v.wrapping_add(d);
    if (v as i32) < (t as i32) {
        if (n as i32) <= (t as i32) {
            n
        } else {
            t
        }
    } else if (n as i32) >= (t as i32) {
        n
    } else {
        t
    }
}

/// C: `FUN_021C157C` (unlabeled routine at `0x021C157C`): surf end step 3 (arc): MMDL x / z += `+0x30` / `+0x38`,
/// clamped at the target `+0x24` / `+0x2C`; y += `+0x34`, `+0x34` += the gravity `+0x10`; frame `+8` += 1, at 16: the
/// position = the target, the ride offset `+0x18..` = 0, next step; the position written back, then the ride offset
/// ([`set_ride_offset`]); returns 0.
pub fn surf_end_arc<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let player = env.r32(w.wrapping_add(0x3C));
    let m = c1(env, P_MMDL, player);
    let p = env.mem().push_local(12);
    get_pos(env, m, p);
    let x = approach(
        env.r32(p),
        env.r32(w.wrapping_add(0x30)),
        env.r32(w.wrapping_add(0x24)),
    );
    env.w32(p, x);
    let z = approach(
        env.r32(p.wrapping_add(8)),
        env.r32(w.wrapping_add(0x38)),
        env.r32(w.wrapping_add(0x2C)),
    );
    env.w32(p.wrapping_add(8), z);
    let vy = env.r32(w.wrapping_add(0x34));
    let y = env.r32(p.wrapping_add(4)).wrapping_add(vy);
    env.w32(p.wrapping_add(4), y);
    let g = env.r32(w.wrapping_add(0x10));
    env.w32(w.wrapping_add(0x34), vy.wrapping_add(g));
    let f = env.r16(w.wrapping_add(8)).wrapping_add(1);
    env.w16(w.wrapping_add(8), f);
    if env.r16(w.wrapping_add(8)) >= 0x10 {
        env.mem().copy(p, w.wrapping_add(0x24), 12);
        for o in [0x18, 0x1C, 0x20] {
            env.w32(w.wrapping_add(o), 0);
        }
        step_s16(env, w);
    }
    set_pos(env, m, p);
    env.mem().pop_local(p);
    set_ride_offset(env, m, w.wrapping_add(0x18));
    0
}

/// C: `FUN_021C161C` (unlabeled routine at `0x021C161C`): surf end step 4 (land): dir lock off; the MMDL set to the
/// target `+0x24` facing `+4` (`FUN_0216E014`); surf effect mode 1; `+0x48` = `FUN_021AD564(control, 0, surf task)`;
/// SE 0x6E2; next step; returns 0.
pub fn surf_end_land<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let player = env.r32(w.wrapping_add(0x3C));
    let m = c1(env, P_MMDL, player);
    c2(env, DIR_LOCK, m, 0);
    let dir = env.r16(w.wrapping_add(4)) as u32;
    env.call(
        SET_POS_DIR,
        &mut [CArg::V(m), CArg::In(w.wrapping_add(0x24), 12), CArg::V(dir)],
    );
    let player = env.r32(w.wrapping_add(0x3C));
    let t = c1(env, P_SURF_TASK, player);
    c2(env, SURF_MODE, t, 1);
    let fm = env.r32(w.wrapping_add(0x40));
    let ctrl = c1(env, FX_CTRL, fm);
    let s = env.callv(SPLASH_ADD, &[ctrl, 0, t]);
    env.w32(w.wrapping_add(0x48), s);
    c1(env, SE_PLAY, 0x6E2);
    step_s16(env, w);
    0
}

/// C: `FUN_021C1670` (unlabeled routine at `0x021C1670`): surf end step 5 (wait for the splash): when `+0x48` is set
/// and `FUN_021AD5CC(+0x48) == 1`: the splash task deleted, field effect 9 unregistered, `FUN_021A35A0(player, 1)`,
/// `+0x48` = 0, done `+6` = 1; returns 0.
pub fn surf_end_wait<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let s = env.r32(w.wrapping_add(0x48));
    if s != 0 && c1(env, SPLASH_OVER, s) == 1 {
        let fm = env.r32(w.wrapping_add(0x40));
        let ctrl = c1(env, FX_CTRL, fm);
        let s = env.r32(w.wrapping_add(0x48));
        c1(env, FX_TASK_DELETE, s);
        c2(env, FX_UNREGISTER, ctrl, SPLASH_FX_ID);
        let player = env.r32(w.wrapping_add(0x3C));
        c2(env, P_SET_35A0, player, 1);
        env.w32(w.wrapping_add(0x48), 0);
        env.w16(w.wrapping_add(6), 1);
    }
    0
}

/// C: `FUN_021C16AC` (unlabeled routine at `0x021C16AC`): the surf end task when `+0` is 0, by the step `+2` (s16,
/// 0..3; others nothing): 0: as [`surf_end_jump`] but C = the tile centre of C's grid position then moved 0x4000 more,
/// D = its own tile centre, copied to `+0x24`, height at D to `+0x28`, `+0x30` / `+0x38` = (C - A) / 16, `+0x34` =
/// -0x6000, SE 0x6E1, next step, then the frame of step 1; 1: a hop frame (as [`surf_end_hop`], the ride offset
/// included); 2: fall: y += `+0x34`, at or below `+0x28`: y = `+0x28`, next step, `+0x30` / `+0x38` = (target - pos) /
/// 16, shake 0xFFFE0000, the splash replaced by `FUN_021AD564(control, 0, surf task)` after surf effect mode 1; the
/// position written back; 3: x / z += `+0x30` / `+0x38`, frame `+8` += 1, at 16: dir lock off, the MMDL set to the
/// target facing `+4`, the splash task deleted, field effect 9 unregistered, `FUN_021A35A0(player, 1)`, `+0x48` = 0,
/// done `+6` = 1, SE 0x6E2, next step.
pub fn surf_end_down<E: FieldEnv>(env: &mut E, w: u32) {
    let player = env.r32(w.wrapping_add(0x3C));
    let m = c1(env, P_MMDL, player);
    let step = env.r16(w.wrapping_add(2)) as i16 as i32 as u32;
    match step {
        0 | 1 => {
            if step == 0 {
                down_start(env, w, m);
            }
            hop_frame(env, w, m);
        }
        2 => {
            let p = env.mem().push_local(12);
            get_pos(env, m, p);
            let y = env
                .r32(p.wrapping_add(4))
                .wrapping_add(env.r32(w.wrapping_add(0x34)));
            env.w32(p.wrapping_add(4), y);
            let target = env.r32(w.wrapping_add(0x28));
            if (y as i32) <= (target as i32) {
                env.w32(p.wrapping_add(4), target);
                step_s16(env, w);
                let dx = env.r32(w.wrapping_add(0x24)).wrapping_sub(env.r32(p));
                env.w32(w.wrapping_add(0x30), div16(dx));
                let dz = env
                    .r32(w.wrapping_add(0x2C))
                    .wrapping_sub(env.r32(p.wrapping_add(8)));
                env.w32(w.wrapping_add(0x38), div16(dz));
                let fm = env.r32(w.wrapping_add(0x40));
                env.callv(SHAKE, &[fm, 0x10, 0xFFFE_0000]);
                let s = env.r32(w.wrapping_add(0x48));
                c1(env, FX_TASK_DELETE, s);
                let player = env.r32(w.wrapping_add(0x3C));
                let t = c1(env, P_SURF_TASK, player);
                c2(env, SURF_MODE, t, 1);
                let fm = env.r32(w.wrapping_add(0x40));
                let ctrl = c1(env, FX_CTRL, fm);
                let s = env.callv(SPLASH_ADD, &[ctrl, 0, t]);
                env.w32(w.wrapping_add(0x48), s);
            }
            set_pos(env, m, p);
            env.mem().pop_local(p);
        }
        3 => {
            let p = env.mem().push_local(12);
            get_pos(env, m, p);
            let x = env.r32(p).wrapping_add(env.r32(w.wrapping_add(0x30)));
            env.w32(p, x);
            let z = env
                .r32(p.wrapping_add(8))
                .wrapping_add(env.r32(w.wrapping_add(0x38)));
            env.w32(p.wrapping_add(8), z);
            set_pos(env, m, p);
            env.mem().pop_local(p);
            let f = env.r16(w.wrapping_add(8)).wrapping_add(1);
            env.w16(w.wrapping_add(8), f);
            if env.r16(w.wrapping_add(8)) >= 0x10 {
                let fm = env.r32(w.wrapping_add(0x40));
                let ctrl = c1(env, FX_CTRL, fm);
                c2(env, DIR_LOCK, m, 0);
                let dir = env.r16(w.wrapping_add(4)) as u32;
                env.call(
                    SET_POS_DIR,
                    &mut [CArg::V(m), CArg::In(w.wrapping_add(0x24), 12), CArg::V(dir)],
                );
                let s = env.r32(w.wrapping_add(0x48));
                c1(env, FX_TASK_DELETE, s);
                c2(env, FX_UNREGISTER, ctrl, SPLASH_FX_ID);
                let player = env.r32(w.wrapping_add(0x3C));
                c2(env, P_SET_35A0, player, 1);
                env.w32(w.wrapping_add(0x48), 0);
                env.w16(w.wrapping_add(6), 1);
                c1(env, SE_PLAY, 0x6E2);
                step_s16(env, w);
            }
        }
        _ => {}
    }
}

/// Step 0 of [`surf_end_down`] up to the next step (the hop frame follows in the caller).
fn down_start<E: FieldEnv>(env: &mut E, w: u32, m: u32) {
    c2(env, DIR_LOCK, m, 1);
    let a = env.mem().push_local(12);
    let c = env.mem().push_local(12);
    let d = env.mem().push_local(12);
    get_pos(env, m, a);
    env.mem().copy(c, a, 12);
    env.mem().copy(d, a, 12);
    let dir = env.r16(w.wrapping_add(4)) as u32;
    pos_step(env, dir, c, 0x1_0000);
    let dir = env.r16(w.wrapping_add(4)) as u32;
    pos_step(env, dir, d, 0x2_0000);
    let (x, z) = (env.r32(c), env.r32(c.wrapping_add(8)));
    tile_centre(env, grid_s16(x), grid_s16(z), c);
    let dir = env.r16(w.wrapping_add(4)) as u32;
    pos_step(env, dir, c, 0x4000);
    let (x, z) = (env.r32(d), env.r32(d.wrapping_add(8)));
    tile_centre(env, grid_s16(x), grid_s16(z), d);
    env.mem().copy(w.wrapping_add(0x24), d, 12);
    height_at(env, m, d, w.wrapping_add(0x28));
    let dx = env.r32(c).wrapping_sub(env.r32(a));
    env.w32(w.wrapping_add(0x30), div16(dx));
    let dz = env
        .r32(c.wrapping_add(8))
        .wrapping_sub(env.r32(a.wrapping_add(8)));
    env.w32(w.wrapping_add(0x38), div16(dz));
    env.mem().pop_local(d);
    env.mem().pop_local(c);
    env.mem().pop_local(a);
    env.w32(w.wrapping_add(0x34), 0xFFFF_A000);
    env.w32(w.wrapping_add(0x1C), 0x1000);
    let player = env.r32(w.wrapping_add(0x3C));
    let t = c1(env, P_SURF_TASK, player);
    c2(env, SURF_MODE, t, 2);
    let kind = splash_kind(env.r16(w.wrapping_add(4)) as u32);
    let fm = env.r32(w.wrapping_add(0x40));
    let ctrl = c1(env, FX_CTRL, fm);
    fx_register_splash(env, ctrl);
    let s = env.callv(SPLASH_ADD, &[ctrl, kind, t]);
    env.w32(w.wrapping_add(0x48), s);
    let fm = env.r32(w.wrapping_add(0x40));
    env.callv(SHAKE, &[fm, 0x10, 0x2_0000]);
    c1(env, SE_PLAY, 0x6E1);
    step_s16(env, w);
}

/// Runs the entry of [`SURF_END_STEPS`] at Thumb address `f` on `w` (the in-area step procs are Rust calls).
fn run_step<E: FieldEnv>(env: &mut E, f: u32, w: u32) -> u32 {
    match f {
        0x021C_1319 => surf_end_jump(env, w),
        0x021C_146D => surf_end_hop(env, w),
        0x021C_14F9 => surf_end_rise(env, w),
        0x021C_157D => surf_end_arc(env, w),
        0x021C_161D => surf_end_land(env, w),
        0x021C_1671 => surf_end_wait(env, w),
        _ => unreachable!(),
    }
}

/// C: `FUN_021C1988` (unlabeled routine at `0x021C1988`): the surf end task proc: `+0` == 0: [`surf_end_down`]; else
/// run the step proc ROM `0x021DA128[+2]` ([`SURF_END_STEPS`], `+2` read as s16 each time) while it answers 1.
/// UNVERIFIED: a step index outside 0..5 panics (the original calls whatever word follows the table).
pub fn surf_end_proc<E: FieldEnv>(env: &mut E, _task: u32, w: u32) {
    if env.r16(w) == 0 {
        surf_end_down(env, w);
        return;
    }
    loop {
        let i = env.r16(w.wrapping_add(2)) as i16 as isize;
        let f = SURF_END_STEPS[i as usize];
        if run_step(env, f, w) != 1 {
            break;
        }
    }
}

/// C: `FUN_021C19B4` (unlabeled routine at `0x021C19B4`): the MMDL in front of the grid player in `dir`:
/// `FUN_0216DDDC(MMDLSYS, (s16)(gx + dx(dir)), (s16)(gz + dz(dir)), y, 1, 0)` with the player MMDL's grid position
/// and fx32 y (`FUN_0216D8AC`).
pub fn mmdl_in_front<E: FieldEnv>(env: &mut E, grid: u32, dir: u32) -> u32 {
    let m = c1(env, G_MMDL, grid);
    let sys = c1(env, SYS_OF, m);
    let x = c1(env, GX, m).wrapping_add(c1(env, DX_OF, dir));
    let z = c1(env, GZ, m).wrapping_add(c1(env, DZ_OF, dir));
    let y = c1(env, POS_Y, m);
    let s16 = |v: u32| v as i16 as i32 as u32;
    env.callv(AT_GRID_HEIGHT, &[sys, s16(x), s16(z), y, 1, 0])
}

#[cfg(test)]
mod tests;
