//! `fldmmdl_acmd.c`, part 3d (overlay 21, `0x0219BDC4..0x0219C4AC`; F1 row 21.09, F4b job 3): the movement-type procs with a field
//! effect (move codes 0x33..0x36; the effect handle is shared with the acmd2d code-0x65 steps), mirroring the player's facing
//! (0x37..0x3E) and walking along a wall (0x3F..0x42).
//!
//! F4b-spec3 specced all 45 functions (`tools/rom-diff/specs/black-field/mvproc3d.spec`); F4b-port2 ported all of them. The ROM
//! tables (state tables, wall offsets, vectors) are read through [`crate::mvproc3b::rom`].

use crate::hooks::{CArg, FieldEnv};
use crate::mvproc3a::{
    start_acmd, ACMD_ACTION, BIT40_OFF, DIR_DISP, HIT, MOVE_WORK, MOVE_WORK_INIT, OPPOSITE,
    SET_ANIM, SET_DIR_DISP, WALK_OFF, WALK_ON,
};
use crate::mvproc3b::rom::rom32;
use crate::mvproc3b::{FIND_PLAYER, MOVE_DIR, SYS_OF};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_BDC4,
    0x0219_BE0C,
    0x0219_BE18,
    0x0219_BE24,
    0x0219_BE30,
    0x0219_BE3C,
    0x0219_BE60,
    0x0219_BE70,
    0x0219_BEB4,
    0x0219_BED0,
    0x0219_BF00,
    0x0219_BF3C,
    0x0219_BF48,
    0x0219_BF54,
    0x0219_BF60,
    0x0219_BF7C,
    0x0219_BF88,
    0x0219_BF94,
    0x0219_BFA0,
    0x0219_BFAC,
    0x0219_BFB8,
    0x0219_BFC4,
    0x0219_BFD0,
    0x0219_BFDC,
    0x0219_C000,
    0x0219_C034,
    0x0219_C04C,
    0x0219_C080,
    0x0219_C0AC,
    0x0219_C0D4,
    0x0219_C0E8,
    0x0219_C0F4,
    0x0219_C100,
    0x0219_C10C,
    0x0219_C118,
    0x0219_C13C,
    0x0219_C150,
    0x0219_C164,
    0x0219_C180,
    0x0219_C20C,
    0x0219_C298,
    0x0219_C2BC,
    0x0219_C2E0,
    0x0219_C314,
    0x0219_C338,
];

/// `FUN_021AC154(h)`: delete a field effect task.
const EFFECT_DELETE: u32 = 0x021A_C154;
/// `FUN_0216D568(m, v)`: move bits |= v.
const SET_MOVE_BITS: u32 = 0x0216_D568;
/// `FUN_0216D8C0(m, &v)`: draw offset (VecFx32 `+0x50`) = `*v`.
const SET_DRAW_OFFSET: u32 = 0x0216_D8C0;
/// `FUN_0216D88C(m, &out)`: the position VecFx32.
const POSITION: u32 = 0x0216_D88C;
/// `FUN_0216D534(sys)`: MMDLSYS `+0x3C` (the fieldmap).
const SYS_FIELDMAP: u32 = 0x0216_D534;
/// `FUN_02188CD0(fieldmap)`: the field effect control (fieldmap `+0x94`).
const EFFECT_CTRL: u32 = 0x0218_8CD0;
/// `FUN_021D046C(ctrl, m, kind)`: add the movement-type field effect; its handle.
const EFFECT_ADD: u32 = 0x021D_046C;
/// `FUN_0216DB24(m)`: the draw side is on and ready.
const DRAWABLE: u32 = 0x0216_DB24;
/// `FUN_02163E34(m, &pos, &attr)`: the attribute at a position (1 = found; mmdl_move).
const ATTR_AT: u32 = 0x0216_3E34;
/// `FUN_021AB11C(attr)`: the attribute's blocking bits (map_attr).
const ATTR_BLOCKS: u32 = 0x021A_B11C;
/// Move bit 0x8000 of the effect types.
const EFFECT_BIT: u32 = 0x8000;

/// ROM `0x021D5474` / `0x021D5480`: the draw offsets of the effect types (VecFx32 (0, -0x20000, 0)).
const OFFSET_INIT: u32 = 0x021D_5474;
const OFFSET_RETURN: u32 = 0x021D_5480;
/// ROM `0x021D537C`: the states of [`effect_move`].
pub const EFFECT_STATES: [u32; 2] = [0x0219_BED1, 0x0219_BF01];
/// ROM `0x021D53A0`: the states of [`mirror_move`].
pub const MIRROR_STATES: [u32; 5] = [
    0x0219_C001,
    0x0219_C035,
    0x0219_C04D,
    0x0219_C081,
    0x0219_C0AD,
];
/// ROM `0x021D5394`: the states of [`wall_move`].
pub const WALL_STATES: [u32; 3] = [0x0219_C13D, 0x0219_C151, 0x0219_C165];
/// ROM wall tables (u32 [dir][hand]): x / z offsets of the tile beside (`0x021D53B4` / `0x021D53D4`), ahead (`0x021D5414` /
/// `0x021D5434`), the turn toward the wall (`0x021D53F4`), the turn at an inner corner (`0x021D5454`); the hand switch
/// `0x021D5384` (u32 [hand]).
const BESIDE_X: u32 = 0x021D_53B4;
const BESIDE_Z: u32 = 0x021D_53D4;
const AHEAD_X: u32 = 0x021D_5414;
const AHEAD_Z: u32 = 0x021D_5434;
const TURN_TO_WALL: u32 = 0x021D_53F4;
const CORNER_TURN: u32 = 0x021D_5454;
const HAND_SWITCH: u32 = 0x021D_5384;

/// The 12 bytes of the ROM VecFx32 at `a` (the original copies it to its stack and passes the address).
fn rom_vec(a: u32) -> [u8; 12] {
    let mut b = [0u8; 12];
    for i in 0..3 {
        b[4 * i..4 * i + 4].copy_from_slice(&rom32(a + 4 * i as u32).to_le_bytes());
    }
    b
}

/// C: `FUN_0219BDC4` (unlabeled routine at `0x0219BDC4`): init of the effect codes 0x33..0x36: move work (8 bytes cleared) `u8 +1` =
/// `k` (the effect kind); draw animation code 0; move bit 0x10 off; move bits |= 0x8000; draw offset = ROM `0x021D5474`.
pub fn effect_init<E: FieldEnv>(env: &mut E, m: u32, k: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 8]);
    env.w8(w.wrapping_add(1), k as u8);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(WALK_OFF, &[m]);
    env.callv(SET_MOVE_BITS, &[m, EFFECT_BIT]);
    let v = rom_vec(OFFSET_INIT);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
}

/// C: `FUN_0219BE0C` (unlabeled routine at `0x0219BE0C`): move code 0x33 init (thunk): [`effect_init`] kind 0.
pub fn effect_init_33<E: FieldEnv>(env: &mut E, m: u32) {
    effect_init(env, m, 0);
}

/// C: `FUN_0219BE18` (unlabeled routine at `0x0219BE18`): move code 0x34 init (thunk): [`effect_init`] kind 1.
pub fn effect_init_34<E: FieldEnv>(env: &mut E, m: u32) {
    effect_init(env, m, 1);
}

/// C: `FUN_0219BE24` (unlabeled routine at `0x0219BE24`): move code 0x35 init (thunk): [`effect_init`] kind 1.
pub fn effect_init_35<E: FieldEnv>(env: &mut E, m: u32) {
    effect_init(env, m, 1);
}

/// C: `FUN_0219BE30` (unlabeled routine at `0x0219BE30`): move code 0x36 init (thunk): [`effect_init`] kind 0.
pub fn effect_init_36<E: FieldEnv>(env: &mut E, m: u32) {
    effect_init(env, m, 0);
}

/// C: `FUN_0219BE3C` (unlabeled routine at `0x0219BE3C`): move proc of the effect codes: [`EFFECT_STATES`]`[u8 +0]` while it answers
/// 1 (both states answer 0).
pub fn effect_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    loop {
        let r = match env.r8(w) {
            0 => effect_start(env, m, w),
            1 => effect_keep(env, m, w),
            s => panic!("UNVERIFIED: 0219BE3C state {s} past the ROM table 0x021D537C"),
        };
        if r != 1 {
            break;
        }
    }
}

/// C: `FUN_0219BE60` (unlabeled routine at `0x0219BE60`): delete proc of the effect codes: delete the effect task when there is a
/// handle.
pub fn effect_delete<E: FieldEnv>(env: &mut E, m: u32) {
    let h = effect_handle(env, m);
    if h != 0 {
        env.callv(EFFECT_DELETE, &[h]);
    }
}

/// C: `FUN_0219BE70` (unlabeled routine at `0x0219BE70`): return proc of the effect codes: state 0; forget the handle; unless the
/// effect is over (`u8 +2`), draw offset = ROM `0x021D5480` and move bits |= 0x8000.
pub fn effect_return<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    env.w8(w, 0);
    set_effect_handle(env, m, 0);
    if env.r8(w.wrapping_add(2)) == 0 {
        let v = rom_vec(OFFSET_RETURN);
        env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
        env.callv(SET_MOVE_BITS, &[m, EFFECT_BIT]);
    }
}

/// C: `FUN_0219BEB4` (unlabeled routine at `0x0219BEB4`): add the movement-type field effect of kind `k` for `m`
/// (`021D046C(02188CD0(0216D534(0216D67C(m))), m, k)`); its handle.
pub fn effect_add<E: FieldEnv>(env: &mut E, m: u32, k: u32) -> u32 {
    let sys = env.callv(SYS_OF, &[m]);
    let fm = env.callv(SYS_FIELDMAP, &[sys]);
    let ctrl = env.callv(EFFECT_CTRL, &[fm]);
    env.callv(EFFECT_ADD, &[ctrl, m, k])
}

/// C: `FUN_0219BED0` (unlabeled routine at `0x0219BED0`): state 0 of [`effect_move`]: unless the effect is over (`u8 +2`), add it
/// (kind `u8 +1`) and keep its handle; move bits 0x10 / 0x40 off; state + 1; returns 0.
pub fn effect_start<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.r8(w.wrapping_add(2)) == 0 {
        let k = u32::from(env.r8(w.wrapping_add(1)));
        let h = effect_add(env, m, k);
        set_effect_handle(env, m, h);
    }
    env.callv(WALK_OFF, &[m]);
    env.callv(BIT40_OFF, &[m]);
    let s = env.r8(w).wrapping_add(1);
    env.w8(w, s);
    0
}

/// C: `FUN_0219BF00` (unlabeled routine at `0x0219BF00`): state 1 of [`effect_move`]: unless the effect is over (`u8 +2`): re-add it
/// when there is no handle and the MMDL is drawable; move bits |= 0x8000. Returns 0.
pub fn effect_keep<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.r8(w.wrapping_add(2)) == 0 {
        if effect_handle(env, m) == 0 && env.callv(DRAWABLE, &[m]) == 1 {
            let k = u32::from(env.r8(w.wrapping_add(1)));
            let h = effect_add(env, m, k);
            set_effect_handle(env, m, h);
        }
        env.callv(SET_MOVE_BITS, &[m, EFFECT_BIT]);
    }
    0
}

/// C: `FUN_0219BF3C` (unlabeled routine at `0x0219BF3C`): `u32 (move work)+4` = `h` (the movement-type effect handle).
pub fn set_effect_handle<E: FieldEnv>(env: &mut E, m: u32, h: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    env.w32(w.wrapping_add(4), h);
}

/// C: `FUN_0219BF48` (unlabeled routine at `0x0219BF48`): the effect handle `u32 (move work)+4` (or 0).
pub fn effect_handle<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(MOVE_WORK, &[m]);
    env.r32(w.wrapping_add(4))
}

/// C: `FUN_0219BF54` (unlabeled routine at `0x0219BF54`): `u8 (move work)+2` = 1 (the effect is over; set by the acmd step
/// `02198FB0`).
pub fn effect_done<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    env.w8(w.wrapping_add(2), 1);
}

/// C: `FUN_0219BF60` (unlabeled routine at `0x0219BF60`): init of the mirror codes 0x37..0x3E: move work (4 bytes cleared) `u8 +1` =
/// 9 (no copied dir yet), `+2` = `k`; display dir 0 (the dir argument is ignored: r1 is overwritten before use).
pub fn mirror_init<E: FieldEnv>(env: &mut E, m: u32, _dir: u32, k: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 4]);
    env.w8(w.wrapping_add(1), 9);
    env.w8(w.wrapping_add(2), k as u8);
    env.callv(SET_DIR_DISP, &[m, 0]);
}

/// C: `FUN_0219BF7C` (unlabeled routine at `0x0219BF7C`): move code 0x37 init (thunk): [`mirror_init`]`(m, 0, 0)`.
pub fn mirror_init_37<E: FieldEnv>(env: &mut E, m: u32) {
    mirror_init(env, m, 0, 0);
}

/// C: `FUN_0219BF88` (unlabeled routine at `0x0219BF88`): move code 0x38 init (thunk): [`mirror_init`]`(m, 1, 0)`.
pub fn mirror_init_38<E: FieldEnv>(env: &mut E, m: u32) {
    mirror_init(env, m, 1, 0);
}

/// C: `FUN_0219BF94` (unlabeled routine at `0x0219BF94`): move code 0x39 init (thunk): [`mirror_init`]`(m, 2, 0)`.
pub fn mirror_init_39<E: FieldEnv>(env: &mut E, m: u32) {
    mirror_init(env, m, 2, 0);
}

/// C: `FUN_0219BFA0` (unlabeled routine at `0x0219BFA0`): move code 0x3A init (thunk): [`mirror_init`]`(m, 3, 0)`.
pub fn mirror_init_3a<E: FieldEnv>(env: &mut E, m: u32) {
    mirror_init(env, m, 3, 0);
}

/// C: `FUN_0219BFAC` (unlabeled routine at `0x0219BFAC`): move code 0x3B init (thunk): [`mirror_init`]`(m, 0, 1)`.
pub fn mirror_init_3b<E: FieldEnv>(env: &mut E, m: u32) {
    mirror_init(env, m, 0, 1);
}

/// C: `FUN_0219BFB8` (unlabeled routine at `0x0219BFB8`): move code 0x3C init (thunk): [`mirror_init`]`(m, 1, 1)`.
pub fn mirror_init_3c<E: FieldEnv>(env: &mut E, m: u32) {
    mirror_init(env, m, 1, 1);
}

/// C: `FUN_0219BFC4` (unlabeled routine at `0x0219BFC4`): move code 0x3D init (thunk): [`mirror_init`]`(m, 2, 1)`.
pub fn mirror_init_3d<E: FieldEnv>(env: &mut E, m: u32) {
    mirror_init(env, m, 2, 1);
}

/// C: `FUN_0219BFD0` (unlabeled routine at `0x0219BFD0`): move code 0x3E init (thunk): [`mirror_init`]`(m, 3, 1)`.
pub fn mirror_init_3e<E: FieldEnv>(env: &mut E, m: u32) {
    mirror_init(env, m, 3, 1);
}

/// C: `FUN_0219BFDC` (unlabeled routine at `0x0219BFDC`): move proc of the mirror codes: [`MIRROR_STATES`]`[u8 +0]` while it
/// answers 1.
pub fn mirror_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    loop {
        let r = match env.r8(w) {
            0 => mirror_face(env, m, w),
            1 => mirror_face_wait(env, m, w),
            2 => mirror_copy(env, m, w),
            3 => mirror_turn(env, m, w),
            4 => mirror_turn_wait(env, m, w),
            s => panic!("UNVERIFIED: 0219BFDC state {s} past the ROM table 0x021D53A0"),
        };
        if r != 1 {
            break;
        }
    }
}

/// C: `FUN_0219C000` (unlabeled routine at `0x0219C000`): state 0 of [`mirror_move`]: face the display dir; move bits 0x10 / 0x40
/// off; state 1; returns 1.
pub fn mirror_face<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    let d = env.callv(DIR_DISP, &[m]) & 0xFFFF;
    start_acmd(env, m, d, 0);
    env.callv(WALK_OFF, &[m]);
    env.callv(BIT40_OFF, &[m]);
    env.w8(w, 1);
    1
}

/// C: `FUN_0219C034` (unlabeled routine at `0x0219C034`): state 1 of [`mirror_move`]: when the acmd is done (1) state 2, returns 1;
/// else 0.
pub fn mirror_face_wait<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.callv(ACMD_ACTION, &[m]) == 1 {
        env.w8(w, 2);
        return 1;
    }
    0
}

/// C: `FUN_0219C04C` (unlabeled routine at `0x0219C04C`): state 2 of [`mirror_move`]: with no copied dir yet (`s8 +1` == 9) and a
/// player, `u8 +1` = his move dir; move bits 0x10 / 0x40 off; state 3; returns 1.
pub fn mirror_copy<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.r8(w.wrapping_add(1)) as i8 == 9 {
        let sys = env.callv(SYS_OF, &[m]);
        let p = env.callv(FIND_PLAYER, &[sys]);
        if p != 0 {
            let d = env.callv(MOVE_DIR, &[p]);
            env.w8(w.wrapping_add(1), d as u8);
        }
    }
    env.callv(WALK_OFF, &[m]);
    env.callv(BIT40_OFF, &[m]);
    env.w8(w, 3);
    1
}

/// C: `FUN_0219C080` (unlabeled routine at `0x0219C080`): state 3 of [`mirror_move`]: face the player's move dir (no null check, the
/// dir not masked); state 4; returns 1.
pub fn mirror_turn<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    let sys = env.callv(SYS_OF, &[m]);
    let p = env.callv(FIND_PLAYER, &[sys]);
    let d = env.callv(MOVE_DIR, &[p]);
    start_acmd(env, m, d, 0);
    env.w8(w, 4);
    1
}

/// C: `FUN_0219C0AC` (unlabeled routine at `0x0219C0AC`): state 4 of [`mirror_move`]: when the acmd is done (nonzero), move bits
/// 0x10 / 0x40 off and state 2; returns 0.
pub fn mirror_turn_wait<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.callv(ACMD_ACTION, &[m]) == 0 {
        return 0;
    }
    env.callv(WALK_OFF, &[m]);
    env.callv(BIT40_OFF, &[m]);
    env.w8(w, 2);
    0
}

/// C: `FUN_0219C0D4` (unlabeled routine at `0x0219C0D4`): init of the wall codes 0x3F..0x42: move work (0xC bytes cleared) `u32 +4`
/// = `mode` (2: may turn back at a dead end), `+8` = `hand` (0 left, 1 right).
pub fn wall_init<E: FieldEnv>(env: &mut E, m: u32, mode: u32, hand: u32) {
    let w = env.callv(MOVE_WORK_INIT, &[m, 0xC]);
    env.w32(w.wrapping_add(4), mode);
    env.w32(w.wrapping_add(8), hand);
}

/// C: `FUN_0219C0E8` (unlabeled routine at `0x0219C0E8`): move code 0x3F init (thunk): [`wall_init`]`(m, 0, 0)`.
pub fn wall_init_3f<E: FieldEnv>(env: &mut E, m: u32) {
    wall_init(env, m, 0, 0);
}

/// C: `FUN_0219C0F4` (unlabeled routine at `0x0219C0F4`): move code 0x40 init (thunk): [`wall_init`]`(m, 1, 1)`.
pub fn wall_init_40<E: FieldEnv>(env: &mut E, m: u32) {
    wall_init(env, m, 1, 1);
}

/// C: `FUN_0219C100` (unlabeled routine at `0x0219C100`): move code 0x41 init (thunk): [`wall_init`]`(m, 2, 0)`.
pub fn wall_init_41<E: FieldEnv>(env: &mut E, m: u32) {
    wall_init(env, m, 2, 0);
}

/// C: `FUN_0219C10C` (unlabeled routine at `0x0219C10C`): move code 0x42 init (thunk): [`wall_init`]`(m, 2, 1)`.
pub fn wall_init_42<E: FieldEnv>(env: &mut E, m: u32) {
    wall_init(env, m, 2, 1);
}

/// C: `FUN_0219C118` (unlabeled routine at `0x0219C118`): move proc of the wall codes: [`WALL_STATES`]`[u8 +0]` while it answers 1.
pub fn wall_move<E: FieldEnv>(env: &mut E, m: u32) {
    let w = env.callv(MOVE_WORK, &[m]);
    loop {
        let r = match env.r8(w) {
            0 => wall_ready(env, m, w),
            1 => wall_go(env, m, w),
            2 => wall_wait(env, m, w),
            s => panic!("UNVERIFIED: 0219C118 state {s} past the ROM table 0x021D5394"),
        };
        if r != 1 {
            break;
        }
    }
}

/// C: `FUN_0219C13C` (unlabeled routine at `0x0219C13C`): state 0 of [`wall_move`]: move bit 0x10 off; state + 1; returns 1.
pub fn wall_ready<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    env.callv(WALK_OFF, &[m]);
    let s = env.r8(w).wrapping_add(1);
    env.w8(w, s);
    1
}

/// C: `FUN_0219C150` (unlabeled routine at `0x0219C150`): state 1 of [`wall_move`]: [`wall_step`] with acmd base 0xC; state + 1;
/// returns 1.
pub fn wall_go<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    wall_step(env, m, w, 0xC);
    let s = env.r8(w).wrapping_add(1);
    env.w8(w, s);
    1
}

/// C: `FUN_0219C164` (unlabeled routine at `0x0219C164`): state 2 of [`wall_move`]: when the acmd is done (1), move bit 0x10 off and
/// state 0; returns 0.
pub fn wall_wait<E: FieldEnv>(env: &mut E, m: u32, w: u32) -> u32 {
    if env.callv(ACMD_ACTION, &[m]) == 1 {
        env.callv(WALK_OFF, &[m]);
        env.w8(w, 0);
    }
    0
}

/// The ROM word `table[dir][hand]` (u32 rows of 2; unchecked indexes, panics outside the embedded block).
fn wall_entry(table: u32, dir: u32, hand: u32) -> u32 {
    rom32(
        table
            .wrapping_add(dir.wrapping_mul(8))
            .wrapping_add(hand.wrapping_mul(4)),
    )
}

/// The tile of an fx32 coordinate as the original computes it (`(fx >> 4)` rounded toward zero to a multiple of 0x1000, then
/// `>> 12`, through the `<< 4` / `>> 16` of the code).
fn fx_tile(fx: u32) -> i32 {
    let a = (fx as i32) >> 4;
    let bias = (((a >> 11) as u32) >> 20) as i32;
    ((a.wrapping_add(bias) as u32) << 4) as i32 >> 16
}

/// The wall test of `0219C180` / `0219C20C`: the tile of `*pos` offset by `xt` / `zt`[dir][hand] holds a blocking attribute.
fn wall_test<E: FieldEnv>(
    env: &mut E,
    m: u32,
    pos: u32,
    dir: u32,
    hand: u32,
    xt: u32,
    zt: u32,
) -> u32 {
    let px = env.r32(pos);
    let py = env.r32(pos.wrapping_add(4));
    let pz = env.r32(pos.wrapping_add(8));
    let ox = wall_entry(xt, dir, hand) as u16 as i16 as i32;
    let x = fx_tile(px).wrapping_add(ox);
    let oz = wall_entry(zt, dir, hand) as u16 as i16 as i32;
    let z = fx_tile(pz).wrapping_add(oz);
    let mut p = [0u8; 12];
    p[0..4].copy_from_slice(&((x as u32) << 16).to_le_bytes());
    p[4..8].copy_from_slice(&py.to_le_bytes());
    p[8..12].copy_from_slice(&((z as u32) << 16).to_le_bytes());
    let mut attr = [0u8; 4];
    let found = env.call(
        ATTR_AT,
        &mut [CArg::V(m), CArg::InBuf(&p), CArg::OutBuf(&mut attr)],
    );
    if found == 1 && env.callv(ATTR_BLOCKS, &[u32::from_le_bytes(attr)]) != 0 {
        return 1;
    }
    0
}

/// C: `FUN_0219C180` (unlabeled routine at `0x0219C180`): 1 when the tile beside the next one (offsets ROM `0x021D53B4` /
/// `0x021D53D4`[dir][hand] from the tile of the VecFx32 at `pos`) holds a blocking attribute, else 0.
pub fn wall_beside<E: FieldEnv>(env: &mut E, m: u32, pos: u32, dir: u32, hand: u32) -> u32 {
    wall_test(env, m, pos, dir, hand, BESIDE_X, BESIDE_Z)
}

/// C: `FUN_0219C20C` (unlabeled routine at `0x0219C20C`): as [`wall_beside`] with the offsets ROM `0x021D5414` / `0x021D5434` (the
/// tile ahead).
pub fn wall_ahead<E: FieldEnv>(env: &mut E, m: u32, pos: u32, dir: u32, hand: u32) -> u32 {
    wall_test(env, m, pos, dir, hand, AHEAD_X, AHEAD_Z)
}

/// The position of `m` (`0216D88C`) in a local window, handed to `f` (the original's stack VecFx32).
fn at_position<E: FieldEnv>(env: &mut E, m: u32, f: impl FnOnce(&mut E, u32) -> u32) -> u32 {
    let pos = env.mem().push_local(12);
    env.call(POSITION, &mut [CArg::V(m), CArg::Out(pos, 12)]);
    let r = f(env, pos);
    env.mem().pop_local(pos);
    r
}

/// C: `FUN_0219C298` (unlabeled routine at `0x0219C298`): [`wall_beside`] at the position of `m`.
pub fn wall_beside_here<E: FieldEnv>(env: &mut E, m: u32, dir: u32, hand: u32) -> u32 {
    at_position(env, m, |env, pos| wall_beside(env, m, pos, dir, hand))
}

/// C: `FUN_0219C2BC` (unlabeled routine at `0x0219C2BC`): [`wall_ahead`] at the position of `m`.
pub fn wall_ahead_here<E: FieldEnv>(env: &mut E, m: u32, dir: u32, hand: u32) -> u32 {
    at_position(env, m, |env, pos| wall_ahead(env, m, pos, dir, hand))
}

/// C: `FUN_0219C2E0` (unlabeled routine at `0x0219C2E0`): `dir` while the wall is beside; else the turn toward the wall (ROM
/// `0x021D53F4`[dir][hand]) when it is ahead; else 9.
pub fn wall_dir<E: FieldEnv>(env: &mut E, m: u32, dir: u32, hand: u32) -> u32 {
    if wall_beside_here(env, m, dir, hand) != 0 {
        return dir;
    }
    if wall_ahead_here(env, m, dir, hand) == 0 {
        return 9;
    }
    wall_entry(TURN_TO_WALL, dir, hand)
}

/// `0219C314` on a value: the new dir ([`wall_dir`]) and, unless it is 9, the hit bits of the step in it (else 0).
fn wall_try<E: FieldEnv>(env: &mut E, m: u32, dir: u32, hand: u32) -> (u32, u32) {
    let d = wall_dir(env, m, dir, hand);
    if d == 9 {
        return (d, 0);
    }
    (d, env.callv(HIT, &[m, d & 0xFFFF]))
}

/// C: `FUN_0219C314` (unlabeled routine at `0x0219C314`): `*dir` = [`wall_dir`]`(m, *dir, hand)`; 9 -> 0, else the hit bits of the
/// step in it.
pub fn wall_dir_hit<E: FieldEnv>(env: &mut E, m: u32, dir: u32, hand: u32) -> u32 {
    let d0 = env.r32(dir);
    let (d, hit) = wall_try(env, m, d0, hand);
    env.w32(dir, d);
    hit
}

/// C: `FUN_0219C338` (unlabeled routine at `0x0219C338`): start a step along the wall (hand `u32 w+8`) from the move dir: no wall
/// -> walk in place (base 0x1C), 0; free -> step with acmd base `code`, move bit 0x10 on, 1. Blocked (hit bit 0) in mode 2: switch
/// hands (ROM `0x021D5384`, stored to `+8`) and retry from the opposite dir; then hit bit 1: turn at the inner corner (ROM
/// `0x021D5454`[dir][hand]) and retry; still blocked -> walk in place, 0.
pub fn wall_step<E: FieldEnv>(env: &mut E, m: u32, w: u32, code: u32) -> u32 {
    let mut hand = env.r32(w.wrapping_add(8));
    let d0 = env.callv(MOVE_DIR, &[m]);
    // the outcome of one try: in place (None) / walked (Some(1)) / blocked with these hit bits (Some(0) never)
    let finish = |env: &mut E, d: u32, hit: u32| -> Option<u32> {
        if d == 9 {
            let md = env.callv(MOVE_DIR, &[m]);
            start_acmd(env, m, md & 0xFFFF, 0x1C);
            return Some(0);
        }
        if hit == 0 {
            let c = env.callv(crate::mvproc3a::ACMD_CODE_FOR, &[d & 0xFFFF, code & 0xFFFF]);
            env.callv(WALK_ON, &[m]);
            env.callv(crate::mvproc3a::ACMD_CHANGE, &[m, c & 0xFFFF]);
            return Some(1);
        }
        None
    };
    let (mut d, mut hit) = wall_try(env, m, d0, hand);
    if let Some(r) = finish(env, d, hit) {
        return r;
    }
    if hit & 1 != 0 && env.r32(w.wrapping_add(4)) == 2 {
        let md = env.callv(MOVE_DIR, &[m]);
        let od = env.callv(OPPOSITE, &[md]);
        hand = rom32(HAND_SWITCH.wrapping_add(hand.wrapping_mul(4)));
        env.w32(w.wrapping_add(8), hand);
        (d, hit) = wall_try(env, m, od, hand);
        if let Some(r) = finish(env, d, hit) {
            return r;
        }
    }
    if hit & 2 != 0 {
        let nd = wall_entry(CORNER_TURN, d, hand);
        let (d2, hit2) = wall_try(env, m, nd, hand);
        if let Some(r) = finish(env, d2, hit2) {
            return r;
        }
    }
    let md = env.callv(MOVE_DIR, &[m]);
    start_acmd(env, m, md & 0xFFFF, 0x1C);
    0
}

#[cfg(test)]
mod tests;
