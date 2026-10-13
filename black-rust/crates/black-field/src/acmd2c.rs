//! `fldmmdl_acmd.c`, part 2c (overlay 21, `0x02198960..0x02198E44`; F4b job 2): waits (02198960, update 02198978), rise / fall
//! (codes 0x43 / 0x44), the status toggles 0x45..0x4A, the balloon marks (02198B4C, update 02198B80) and the walks with a per-frame
//! speed table (02198C08, update 02198C4C).
//!
//! F4b-spec2 specced all 53 functions (`tools/rom-diff/specs/black-field/acmd2c.spec`); F4b-port1 ported them.

use crate::acmd2a::{
    ACMD_WORK, ACMD_WORK_INIT, GRID_STEP, INC_SEQ, MOVING, PLAY_SE, POS_HEIGHT, POS_MOVE,
    SAVE_OLD_GRID, SET_ANIM, SET_DIR, SET_DRAW_OFFSET, SET_MOVE_BITS, WALK_DONE,
};
use crate::acmd2b::vec_bytes;
use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_8960,
    0x0219_8978,
    0x0219_8998,
    0x0219_89A4,
    0x0219_89B0,
    0x0219_89BC,
    0x0219_89C8,
    0x0219_89D4,
    0x0219_89E0,
    0x0219_89EC,
    0x0219_8A10,
    0x0219_8A58,
    0x0219_8A8C,
    0x0219_8AD4,
    0x0219_8AE8,
    0x0219_8AFC,
    0x0219_8B10,
    0x0219_8B24,
    0x0219_8B38,
    0x0219_8B4C,
    0x0219_8B80,
    0x0219_8BA8,
    0x0219_8BB8,
    0x0219_8BC8,
    0x0219_8BD8,
    0x0219_8BE8,
    0x0219_8BF8,
    0x0219_8C08,
    0x0219_8C4C,
    0x0219_8CA4,
    0x0219_8CB4,
    0x0219_8CC4,
    0x0219_8CD4,
    0x0219_8CE4,
    0x0219_8CF4,
    0x0219_8D04,
    0x0219_8D14,
    0x0219_8D24,
    0x0219_8D3C,
    0x0219_8D4C,
    0x0219_8D5C,
    0x0219_8D6C,
    0x0219_8D7C,
    0x0219_8D94,
    0x0219_8DA4,
    0x0219_8DB4,
    0x0219_8DC4,
    0x0219_8DD4,
    0x0219_8DEC,
    0x0219_8DFC,
    0x0219_8E0C,
    0x0219_8E1C,
    0x0219_8E2C,
];

/// The ROM speed tables of the speed-table walks (fx32 per frame), by address: codes 0x4C..0x4F / 0xA3..0xA6 (6 frames),
/// 0x50..0x53 (3), 0x60..0x63 (7), 0xA7..0xAA (12).
pub const SPEED_TABLES: [(u32, &[u32]); 4] = [
    (
        0x021D_4A68,
        &[0x2000, 0x3000, 0x3000, 0x2000, 0x3000, 0x3000],
    ),
    (0x021D_4A44, &[0x5000, 0x6000, 0x5000]),
    (
        0x021D_4A80,
        &[0x2500, 0x2480, 0x2480, 0x2480, 0x2480, 0x2480, 0x2480],
    ),
    (
        0x021D_4A9C,
        &[
            0x1000, 0x1000, 0x2000, 0x1000, 0x1000, 0x2000, 0x1000, 0x1000, 0x2000, 0x1000, 0x1000,
            0x2000,
        ],
    ),
];

/// C: `FUN_02198960` (unlabeled routine at `0x02198960`): start a wait of `frames` frames (acmd work `u32 +0`); acmd seq + 1.
pub fn wait_start<E: FieldEnv>(env: &mut E, m: u32, frames: u32) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 4]);
    env.w32(w, frames);
    env.callv(INC_SEQ, &[m]);
}

/// C: `FUN_02198978` (unlabeled routine at `0x02198978`): wait update: frames left -> one less, returns 0; none -> acmd seq + 1,
/// returns 1.
pub fn wait_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let n = env.r32(w);
    if n != 0 {
        env.w32(w, n.wrapping_sub(1));
        return 0;
    }
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_021989BC` (unlabeled routine at `0x021989BC`): acmd code 0x3F (wait 8 frames): [`wait_start`]`(m, 8)`; returns 1.
pub fn wait_8<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    wait_start(env, m, 8);
    1
}

/// C: `FUN_02198A8C` (unlabeled routine at `0x02198A8C`): fall update: height += speed, clamped at 0 from below; draw offset =
/// (0, height, 0); still above 0 -> 0; else acmd seq + 1, returns 1.
pub fn fall_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let mut h = env.r32(w).wrapping_add(env.r32(w.wrapping_add(4)));
    if (h as i32) < 0 {
        h = 0;
    }
    env.w32(w, h);
    let v = vec_bytes(0, h, 0);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    if (h as i32) > 0 {
        return 0;
    }
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02198C4C` (unlabeled routine at `0x02198C4C`): speed-table walk update: position += `table[index]` in the u16 dir
/// (+0), map height; index (s16 +6) + 1; below the frame count (s16 +4) -> 0; else move bits 0x440, old grid = grid, draw
/// animation code 0, acmd seq + 1, returns 1. `table` is the ROM address of one of [`SPEED_TABLES`]. UNVERIFIED: another
/// address, or an index past the table, panics (the original reads whatever follows).
pub fn table_walk_update<E: FieldEnv>(env: &mut E, m: u32, table: u32) -> u32 {
    let t = SPEED_TABLES
        .iter()
        .find(|t| t.0 == table)
        .map(|t| t.1)
        .unwrap_or_else(|| panic!("02198C4C: unknown speed table {table:#010x}"));
    let w = env.callv(ACMD_WORK, &[m]);
    let i = env.r16(w.wrapping_add(6)) as i16;
    let dir = u32::from(env.r16(w));
    env.callv(POS_MOVE, &[m, dir, t[i as usize]]);
    env.callv(POS_HEIGHT, &[m]);
    let i = i.wrapping_add(1);
    env.w16(w.wrapping_add(6), i as u16);
    if i < env.r16(w.wrapping_add(4)) as i16 {
        return 0;
    }
    env.callv(SET_MOVE_BITS, &[m, WALK_DONE]);
    env.callv(SAVE_OLD_GRID, &[m]);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02198D7C` (unlabeled routine at `0x02198D7C`): update of acmd codes 0x50..0x53: [`table_walk_update`] with the 3-frame
/// table `0x021D4A44`; 1 when it returned 1, else 0.
pub fn table_walk_update_3<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    u32::from(table_walk_update(env, m, 0x021D_4A44) == 1)
}

/// `FUN_0216D548(m, v)`: status |= v.
const STATUS_SET: u32 = 0x0216_D548;
/// `FUN_0216D550(m, v)`: status &= !v.
const STATUS_CLEAR: u32 = 0x0216_D550;
/// `FUN_0216D67C(m)`: MMDL -> MMDLSYS.
const SYS_OF: u32 = 0x0216_D67C;
/// `FUN_0216D534(sys)`: the MMDLSYS fieldmap (`+0x3C`).
const SYS_FIELDMAP: u32 = 0x0216_D534;
/// `FUN_02188CD0(fm)`: the fieldmap's field-effect control.
const FM_FX_CTRL: u32 = 0x0218_8CD0;
/// `FUN_021BB528(ctrl, m, kind, 1)` (fldeff_gyoe): add a balloon mark over the MMDL; returns its handle (D3).
const BALLOON_NEW: u32 = 0x021B_B528;
/// `FUN_021BB5C8(handle)` (fldeff_gyoe): 1 when the balloon mark has finished.
const BALLOON_DONE: u32 = 0x021B_B5C8;
/// `FUN_021AC154(handle)`: delete a field effect.
const FX_DELETE: u32 = 0x021A_C154;
/// SE of the fall (code 0x44).
const SE_FALL: u32 = 0x676;

/// C: `FUN_021989EC` (unlabeled routine at `0x021989EC`): acmd code 0x43 (rise) init: acmd work (8) `+4` = 0x10000 (fx32 per frame;
/// the height `+0` is left to the clear); draw animation code 0; acmd seq + 1; returns 1.
pub fn rise_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK_INIT, &[m, 8]);
    env.w32(w.wrapping_add(4), 0x1_0000);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02198A10` (unlabeled routine at `0x02198A10`): rise update: height += speed; draw offset = (0, height, 0); below 0x28
/// half tiles (`height / 0x8000`, signed, rounded towards 0) -> 0; else acmd seq + 1, returns 1.
pub fn rise_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let h = env.r32(w).wrapping_add(env.r32(w.wrapping_add(4)));
    env.w32(w, h);
    let v = vec_bytes(0, env.r32(w), 0);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    if (env.r32(w) as i32) / 0x8000 < 0x28 {
        return 0;
    }
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02198A58` (unlabeled routine at `0x02198A58`): acmd code 0x44 (fall) init: acmd work (8) = height 0x140000, speed
/// 0xFFFF0000 (-1.0 per frame); SE 0x676; draw animation code 0; acmd seq + 1; returns 1.
pub fn fall_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK_INIT, &[m, 8]);
    env.w32(w, 0x14_0000);
    env.w32(w.wrapping_add(4), 0xFFFF_0000);
    env.callv(PLAY_SE, &[SE_FALL]);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// Status toggle steps: status |= / &= !`bit`, acmd seq + 1; returns 1.
fn status_step<E: FieldEnv>(env: &mut E, m: u32, set: bool, bit: u32) -> u32 {
    env.callv(if set { STATUS_SET } else { STATUS_CLEAR }, &[m, bit]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02198AD4` (unlabeled routine at `0x02198AD4`): acmd code 0x45 (grid and rail): status |= 4 (vanish? name guessed); acmd
/// seq + 1; returns 1.
pub fn status4_on<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    status_step(env, m, true, 4)
}

/// C: `FUN_02198AE8` (unlabeled routine at `0x02198AE8`): acmd code 0x46: status &= !4; acmd seq + 1; returns 1.
pub fn status4_off<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    status_step(env, m, false, 4)
}

/// C: `FUN_02198AFC` (unlabeled routine at `0x02198AFC`): acmd code 0x47: status |= 8 (dir locked); acmd seq + 1; returns 1.
pub fn dir_lock_on<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    status_step(env, m, true, 8)
}

/// C: `FUN_02198B10` (unlabeled routine at `0x02198B10`): acmd code 0x48: status &= !8 (dir lock off); acmd seq + 1; returns 1.
pub fn dir_lock_off<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    status_step(env, m, false, 8)
}

/// C: `FUN_02198B24` (unlabeled routine at `0x02198B24`): acmd code 0x49: status |= 0x10 (animation held, cf.
/// `mmdl_draw::anim_held`); acmd seq + 1; returns 1.
pub fn anim_hold_on<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    status_step(env, m, true, 0x10)
}

/// C: `FUN_02198B38` (unlabeled routine at `0x02198B38`): acmd code 0x4A: status &= !0x10; acmd seq + 1; returns 1.
pub fn anim_hold_off<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    status_step(env, m, false, 0x10)
}

/// C: `FUN_02198B4C` (unlabeled routine at `0x02198B4C`): start a balloon mark of `kind` over the MMDL: acmd work (8) `+0` = kind,
/// `+4` = `021BB528(ctrl, m, kind, 1)` (ctrl = the field-effect control of the MMDLSYS's fieldmap; the mark itself is D3); acmd
/// seq + 1.
pub fn balloon_start<E: FieldEnv>(env: &mut E, m: u32, kind: u32) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 8]);
    env.w32(w, kind);
    let sys = env.callv(SYS_OF, &[m]);
    let fm = env.callv(SYS_FIELDMAP, &[sys]);
    let ctrl = env.callv(FM_FX_CTRL, &[fm]);
    let h = env.callv(BALLOON_NEW, &[ctrl, m, kind, 1]);
    env.w32(w.wrapping_add(4), h);
    env.callv(INC_SEQ, &[m]);
}

/// C: `FUN_02198B80` (unlabeled routine at `0x02198B80`): balloon mark update: once the mark has finished (`021BB5C8 == 1`),
/// delete it, acmd seq + 1, returns 1; else 0.
pub fn balloon_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let h = env.r32(w.wrapping_add(4));
    if env.callv(BALLOON_DONE, &[h]) != 1 {
        return 0;
    }
    let h = env.r32(w.wrapping_add(4));
    env.callv(FX_DELETE, &[h]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02198C08` (unlabeled routine at `0x02198C08`): start a one-tile walk with a per-frame speed table: acmd work (8) = `u16
/// dir`, `u16 anim`, `u16 frames` (the index `+6` is left to the clear); step the grid one tile in `dir` (u16), display and move
/// dir, draw animation code = `anim` (the whole word), move bit 0x20, acmd seq + 1.
pub fn table_walk_start<E: FieldEnv>(env: &mut E, m: u32, dir: u32, frames: u32, anim: u32) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 8]);
    env.w16(w, dir as u16);
    env.w16(w.wrapping_add(2), anim as u16);
    env.w16(w.wrapping_add(4), frames as u16);
    env.callv(GRID_STEP, &[m, dir & 0xFFFF]);
    env.callv(SET_DIR, &[m, dir & 0xFFFF]);
    env.callv(SET_ANIM, &[m, anim]);
    env.callv(SET_MOVE_BITS, &[m, MOVING]);
    env.callv(INC_SEQ, &[m]);
}

/// C: `FUN_02198D24` (unlabeled routine at `0x02198D24`): update of acmd codes 0x4C..0x4F / 0xA3..0xA6: [`table_walk_update`] with
/// the 6-frame table `0x021D4A68`; 1 when it returned 1, else 0.
pub fn table_walk_update_6<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    u32::from(table_walk_update(env, m, 0x021D_4A68) == 1)
}

/// C: `FUN_02198DD4` (unlabeled routine at `0x02198DD4`): update of acmd codes 0x60..0x63: [`table_walk_update`] with the 7-frame
/// table `0x021D4A80`; 1 when it returned 1, else 0.
pub fn table_walk_update_7<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    u32::from(table_walk_update(env, m, 0x021D_4A80) == 1)
}

/// C: `FUN_02198E2C` (unlabeled routine at `0x02198E2C`): update of acmd codes 0xA7..0xAA: [`table_walk_update`] with the 12-frame
/// table `0x021D4A9C`; 1 when it returned 1, else 0.
pub fn table_walk_update_12<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    u32::from(table_walk_update(env, m, 0x021D_4A9C) == 1)
}

/// C: `FUN_02198998` (unlabeled routine at `0x02198998`): acmd code 0x3C (wait 1 frame(s)): [`wait_start`]`(m, 1)`; returns 1.
pub fn wait_3c<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    wait_start(env, m, 1);
    1
}

/// C: `FUN_021989A4` (unlabeled routine at `0x021989A4`): acmd code 0x3D (wait 2 frame(s)): [`wait_start`]`(m, 2)`; returns 1.
pub fn wait_3d<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    wait_start(env, m, 2);
    1
}

/// C: `FUN_021989B0` (unlabeled routine at `0x021989B0`): acmd code 0x3E (wait 4 frame(s)): [`wait_start`]`(m, 4)`; returns 1.
pub fn wait_3e<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    wait_start(env, m, 4);
    1
}

/// C: `FUN_021989C8` (unlabeled routine at `0x021989C8`): acmd code 0x40 (wait 15 frame(s)): [`wait_start`]`(m, 0xf)`; returns 1.
pub fn wait_40<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    wait_start(env, m, 0xf);
    1
}

/// C: `FUN_021989D4` (unlabeled routine at `0x021989D4`): acmd code 0x41 (wait 16 frame(s)): [`wait_start`]`(m, 0x10)`; returns
/// 1.
pub fn wait_41<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    wait_start(env, m, 0x10);
    1
}

/// C: `FUN_021989E0` (unlabeled routine at `0x021989E0`): acmd code 0x42 (wait 32 frame(s)): [`wait_start`]`(m, 0x20)`; returns
/// 1.
pub fn wait_42<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    wait_start(env, m, 0x20);
    1
}

/// C: `FUN_02198BA8` (unlabeled routine at `0x02198BA8`): acmd code 0x4B (balloon mark kind 0 over the MMDL):
/// [`balloon_start`]`(m, 0)`; returns 0.
pub fn balloon_4b<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    balloon_start(env, m, 0);
    0
}

/// C: `FUN_02198BB8` (unlabeled routine at `0x02198BB8`): acmd code 0x67 (balloon mark kind 0 over the MMDL):
/// [`balloon_start`]`(m, 0)`; returns 0.
pub fn balloon_67<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    balloon_start(env, m, 0);
    0
}

/// C: `FUN_02198BC8` (unlabeled routine at `0x02198BC8`): acmd code 0x99 (balloon mark kind 0 over the MMDL):
/// [`balloon_start`]`(m, 0)`; returns 0.
pub fn balloon_99<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    balloon_start(env, m, 0);
    0
}

/// C: `FUN_02198BD8` (unlabeled routine at `0x02198BD8`): acmd code 0x9F (balloon mark kind 1 over the MMDL):
/// [`balloon_start`]`(m, 1)`; returns 0.
pub fn balloon_9f<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    balloon_start(env, m, 1);
    0
}

/// C: `FUN_02198BE8` (unlabeled routine at `0x02198BE8`): acmd code 0xA0 (balloon mark kind 2 over the MMDL):
/// [`balloon_start`]`(m, 2)`; returns 0.
pub fn balloon_a0<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    balloon_start(env, m, 2);
    0
}

/// C: `FUN_02198BF8` (unlabeled routine at `0x02198BF8`): acmd code 0xA1 (balloon mark kind 3 over the MMDL):
/// [`balloon_start`]`(m, 3)`; returns 0.
pub fn balloon_a1<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    balloon_start(env, m, 3);
    0
}

/// C: `FUN_02198CA4` (unlabeled routine at `0x02198CA4`): acmd code 0x4C (walk one tile up with a per-frame speed table (6
/// frames)): [`table_walk_start`]`(m, 0, 6, 6)`; returns 1.
pub fn table_walk_4c<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 0, 6, 6);
    1
}

/// C: `FUN_02198CB4` (unlabeled routine at `0x02198CB4`): acmd code 0x4D (walk one tile down with a per-frame speed table (6
/// frames)): [`table_walk_start`]`(m, 1, 6, 6)`; returns 1.
pub fn table_walk_4d<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 1, 6, 6);
    1
}

/// C: `FUN_02198CC4` (unlabeled routine at `0x02198CC4`): acmd code 0x4E (walk one tile left with a per-frame speed table (6
/// frames)): [`table_walk_start`]`(m, 2, 6, 6)`; returns 1.
pub fn table_walk_4e<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 2, 6, 6);
    1
}

/// C: `FUN_02198CD4` (unlabeled routine at `0x02198CD4`): acmd code 0x4F (walk one tile right with a per-frame speed table (6
/// frames)): [`table_walk_start`]`(m, 3, 6, 6)`; returns 1.
pub fn table_walk_4f<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 3, 6, 6);
    1
}

/// C: `FUN_02198CE4` (unlabeled routine at `0x02198CE4`): acmd code 0xA3 (walk one tile up with a per-frame speed table (6
/// frames)): [`table_walk_start`]`(m, 0, 6, 0xb)`; returns 1.
pub fn table_walk_a3<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 0, 6, 0xb);
    1
}

/// C: `FUN_02198CF4` (unlabeled routine at `0x02198CF4`): acmd code 0xA4 (walk one tile down with a per-frame speed table (6
/// frames)): [`table_walk_start`]`(m, 1, 6, 0xb)`; returns 1.
pub fn table_walk_a4<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 1, 6, 0xb);
    1
}

/// C: `FUN_02198D04` (unlabeled routine at `0x02198D04`): acmd code 0xA5 (walk one tile left with a per-frame speed table (6
/// frames)): [`table_walk_start`]`(m, 2, 6, 0xb)`; returns 1.
pub fn table_walk_a5<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 2, 6, 0xb);
    1
}

/// C: `FUN_02198D14` (unlabeled routine at `0x02198D14`): acmd code 0xA6 (walk one tile right with a per-frame speed table (6
/// frames)): [`table_walk_start`]`(m, 3, 6, 0xb)`; returns 1.
pub fn table_walk_a6<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 3, 6, 0xb);
    1
}

/// C: `FUN_02198D3C` (unlabeled routine at `0x02198D3C`): acmd code 0x50 (walk one tile up with a per-frame speed table (3
/// frames)): [`table_walk_start`]`(m, 0, 3, 7)`; returns 1.
pub fn table_walk_50<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 0, 3, 7);
    1
}

/// C: `FUN_02198D4C` (unlabeled routine at `0x02198D4C`): acmd code 0x51 (walk one tile down with a per-frame speed table (3
/// frames)): [`table_walk_start`]`(m, 1, 3, 7)`; returns 1.
pub fn table_walk_51<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 1, 3, 7);
    1
}

/// C: `FUN_02198D5C` (unlabeled routine at `0x02198D5C`): acmd code 0x52 (walk one tile left with a per-frame speed table (3
/// frames)): [`table_walk_start`]`(m, 2, 3, 7)`; returns 1.
pub fn table_walk_52<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 2, 3, 7);
    1
}

/// C: `FUN_02198D6C` (unlabeled routine at `0x02198D6C`): acmd code 0x53 (walk one tile right with a per-frame speed table (3
/// frames)): [`table_walk_start`]`(m, 3, 3, 7)`; returns 1.
pub fn table_walk_53<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 3, 3, 7);
    1
}

/// C: `FUN_02198D94` (unlabeled routine at `0x02198D94`): acmd code 0x60 (walk one tile up with a per-frame speed table (7
/// frames)): [`table_walk_start`]`(m, 0, 7, 8)`; returns 1.
pub fn table_walk_60<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 0, 7, 8);
    1
}

/// C: `FUN_02198DA4` (unlabeled routine at `0x02198DA4`): acmd code 0x61 (walk one tile down with a per-frame speed table (7
/// frames)): [`table_walk_start`]`(m, 1, 7, 8)`; returns 1.
pub fn table_walk_61<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 1, 7, 8);
    1
}

/// C: `FUN_02198DB4` (unlabeled routine at `0x02198DB4`): acmd code 0x62 (walk one tile left with a per-frame speed table (7
/// frames)): [`table_walk_start`]`(m, 2, 7, 8)`; returns 1.
pub fn table_walk_62<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 2, 7, 8);
    1
}

/// C: `FUN_02198DC4` (unlabeled routine at `0x02198DC4`): acmd code 0x63 (walk one tile right with a per-frame speed table (7
/// frames)): [`table_walk_start`]`(m, 3, 7, 8)`; returns 1.
pub fn table_walk_63<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 3, 7, 8);
    1
}

/// C: `FUN_02198DEC` (unlabeled routine at `0x02198DEC`): acmd code 0xA7 (walk one tile up with a per-frame speed table (12
/// frames)): [`table_walk_start`]`(m, 0, 0xc, 9)`; returns 1.
pub fn table_walk_a7<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 0, 0xc, 9);
    1
}

/// C: `FUN_02198DFC` (unlabeled routine at `0x02198DFC`): acmd code 0xA8 (walk one tile down with a per-frame speed table (12
/// frames)): [`table_walk_start`]`(m, 1, 0xc, 9)`; returns 1.
pub fn table_walk_a8<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 1, 0xc, 9);
    1
}

/// C: `FUN_02198E0C` (unlabeled routine at `0x02198E0C`): acmd code 0xA9 (walk one tile left with a per-frame speed table (12
/// frames)): [`table_walk_start`]`(m, 2, 0xc, 9)`; returns 1.
pub fn table_walk_a9<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 2, 0xc, 9);
    1
}

/// C: `FUN_02198E1C` (unlabeled routine at `0x02198E1C`): acmd code 0xAA (walk one tile right with a per-frame speed table (12
/// frames)): [`table_walk_start`]`(m, 3, 0xc, 9)`; returns 1.
pub fn table_walk_aa<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    table_walk_start(env, m, 3, 0xc, 9);
    1
}

#[cfg(test)]
mod tests;
