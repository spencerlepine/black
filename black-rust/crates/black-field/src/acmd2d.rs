//! `fldmmdl_acmd.c`, part 2d (overlay 21, `0x02198E44..0x02199888`; F4b job 2): draw-animation starts and waits (codes 0x64,
//! 0xB5..0xBB), code 0x65, the timed animations 0x66 / 0x68 / 0xA2, the bounce 0x9A and the vector moves (0219912C, update
//! 021991F0).
//!
//! F4b-spec2 specced all 50 functions (`tools/rom-diff/specs/black-field/acmd2d.spec`); F4b-port1 ported them.

use crate::acmd2a::{
    ACMD_WORK, ACMD_WORK_INIT, INC_SEQ, PLAY_SE, SAVE_OLD_GRID, SET_ANIM, SET_DIR_DISP,
    SET_DRAW_OFFSET, SET_MOVE_BITS, WALK_DONE,
};
use crate::acmd2b::{vec_bytes, JUMP_HEIGHTS};
use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_8E44,
    0x0219_8E60,
    0x0219_8E7C,
    0x0219_8E98,
    0x0219_8EB0,
    0x0219_8EC8,
    0x0219_8F20,
    0x0219_8F68,
    0x0219_8FB0,
    0x0219_9034,
    0x0219_9050,
    0x0219_9070,
    0x0219_9088,
    0x0219_90A0,
    0x0219_90B8,
    0x0219_90D4,
    0x0219_912C,
    0x0219_91F0,
    0x0219_9234,
    0x0219_9268,
    0x0219_9298,
    0x0219_92CC,
    0x0219_92FC,
    0x0219_932C,
    0x0219_9360,
    0x0219_9394,
    0x0219_93C8,
    0x0219_93FC,
    0x0219_9430,
    0x0219_9460,
    0x0219_9490,
    0x0219_94C0,
    0x0219_94F0,
    0x0219_9520,
    0x0219_9550,
    0x0219_9584,
    0x0219_95B8,
    0x0219_95E8,
    0x0219_9618,
    0x0219_964C,
    0x0219_9680,
    0x0219_96B4,
    0x0219_96E8,
    0x0219_971C,
    0x0219_9750,
    0x0219_9784,
    0x0219_97B8,
    0x0219_97EC,
    0x0219_9820,
    0x0219_9854,
];

/// `FUN_0216D7A8(m, 0)`: the draw get proc: the MMDL's blact actor id (0xFFFF = none).
const DRAW_ACTOR: u32 = 0x0216_D7A8;
/// `FUN_0216D67C(m)`: the MMDLSYS of an MMDL.
const SYS_OF: u32 = 0x0216_D67C;
/// `FUN_0216D520(sys)`: the MMDLSYS draw system.
const DRAW_SYS: u32 = 0x0216_D520;
/// `FUN_0219C9DC(draw)`: the blact system of a draw system (fldmmdl_blact).
const BLACT_SYS: u32 = 0x0219_C9DC;
/// `FUN_020508EC(sys, actor, &out)`: animation state of a blact actor (1 / 0; the out word is not read).
const ACTOR_ANIM: u32 = 0x0205_08EC;
/// `FUN_0216D600(m, dir)`: move dir = dir.
const SET_DIR_MOVE: u32 = 0x0216_D600;
/// `FUN_0216DA70(m)`: move bit 0x20 on.
const SET_BIT20: u32 = 0x0216_DA70;
/// Grid getters x / y / z (`FUN_0216D83C` / `FUN_0216D854` / `FUN_0216D86C`).
const GRID: [u32; 3] = [0x0216_D83C, 0x0216_D854, 0x0216_D86C];
/// Old-grid setters x / y / z (`FUN_0216D80C` / `FUN_0216D818` / `FUN_0216D824`).
const SET_OLD_GRID: [u32; 3] = [0x0216_D80C, 0x0216_D818, 0x0216_D824];
/// Grid adders x / y / z (`FUN_0216D848` / `FUN_0216D860` / `FUN_0216D87C`).
const ADD_GRID: [u32; 3] = [0x0216_D848, 0x0216_D860, 0x0216_D87C];
/// Draw-actor id "none".
const NO_ACTOR: u32 = 0xFFFF;

/// ROM `0x021D4C1C`: the bounce heights of acmd code 0x9A (draw offset y per frame, the last one repeats).
pub const BOUNCE: [u32; 10] = [0, 0, 0, 0, 0, 0x4000, 0x4000, 0x3000, 0x1000, 0];

/// C: `FUN_02198E44` (unlabeled routine at `0x02198E44`): start draw animation 10 (acmd work cleared, 4 bytes); acmd seq + 1;
/// returns 0.
pub fn anim10_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(ACMD_WORK_INIT, &[m, 4]);
    env.callv(SET_ANIM, &[m, 10]);
    env.callv(INC_SEQ, &[m]);
    0
}

/// C: `FUN_02198E98` (unlabeled routine at `0x02198E98`): acmd code 0xB8: SE 0x7B7, then [`anim10_start`].
pub fn anim10_start_se<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(PLAY_SE, &[0x7B7]);
    anim10_start(env, m)
}

/// C: `FUN_02198EC8` (unlabeled routine at `0x02198EC8`): wait for the draw animation: with a blact actor whose animation state
/// is 1, draw animation code 0, acmd seq + 1, returns 1; else 0.
pub fn anim_wait<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(ACMD_WORK, &[m]);
    let actor = env.callv(DRAW_ACTOR, &[m, 0]);
    if actor == NO_ACTOR {
        return 0;
    }
    let sys = env.callv(SYS_OF, &[m]);
    let draw = env.callv(DRAW_SYS, &[sys]);
    let blact = env.callv(BLACT_SYS, &[draw]);
    if env.call(
        ACTOR_ANIM,
        &mut [CArg::V(blact), CArg::V(actor & 0xFFFF), CArg::P(0)],
    ) != 1
    {
        return 0;
    }
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_021990D4` (unlabeled routine at `0x021990D4`): bounce update: draw offset = (0, [`BOUNCE`]`[min(frame, 9)]`, 0);
/// frame + 1; from 0x14 on, acmd seq + 1. Returns 0. UNVERIFIED: a negative frame panics (the original reads its stack below
/// the copied table).
pub fn bounce_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let f = (env.r32(w) as i32).min(9);
    let v = vec_bytes(0, BOUNCE[f as usize], 0);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    let n = env.r32(w).wrapping_add(1);
    env.w32(w, n);
    if n as i32 >= 0x14 {
        env.callv(INC_SEQ, &[m]);
    }
    0
}

/// C: `FUN_0219912C` (unlabeled routine at `0x0219912C`): start a vector move: acmd work (0x10) = `u32 frames`, the VecFx32 at
/// `vec` (per-frame step); display dir = `disp`, move dir = `mv` (u16), draw animation code = `anim` (u8), move bit 0x20, old grid =
/// grid; the grid steps by the sign of each component of `*vec` (-1 / +1, nothing for 0); acmd seq + 1.
pub fn vec_move_start<E: FieldEnv>(
    env: &mut E,
    m: u32,
    vec: u32,
    disp: u32,
    mv: u32,
    frames: u32,
    anim: u32,
) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 0x10]);
    env.w32(w, frames);
    for k in 0..3 {
        let c = env.r32(vec.wrapping_add(4 * k));
        env.w32(w.wrapping_add(4 + 4 * k), c);
    }
    env.callv(SET_DIR_DISP, &[m, disp & 0xFFFF]);
    env.callv(SET_DIR_MOVE, &[m, mv & 0xFFFF]);
    env.callv(SET_ANIM, &[m, anim & 0xFF]);
    env.callv(SET_BIT20, &[m]);
    for k in 0..3 {
        let g = env.callv(GRID[k], &[m]);
        env.callv(SET_OLD_GRID[k], &[m, g]);
    }
    for (k, add) in ADD_GRID.iter().enumerate() {
        let c = env.r32(vec.wrapping_add(4 * k as u32)) as i32;
        if c != 0 {
            env.callv(*add, &[m, if c < 0 { 0xFFFF_FFFF } else { 1 }]);
        }
    }
    env.callv(INC_SEQ, &[m]);
}

/// `FUN_0219BF48(m)` (F4b job 3, move-proc work of a movement type): the effect handle at work `+4` (or 0).
const MOVE_FX: u32 = 0x0219_BF48;
/// `FUN_0219BF3C(m, v)` (F4b job 3): move-proc work `+4` = v.
const SET_MOVE_FX: u32 = 0x0219_BF3C;
/// `FUN_0219BF54(m)` (F4b job 3): move-proc work byte `+2` = 1.
const MOVE_FX_DONE: u32 = 0x0219_BF54;
/// `FUN_021D04A4(h)`: start the effect `h` (D3).
const FX_START: u32 = 0x021D_04A4;
/// `FUN_021D04C4(h)`: 1 when the effect `h` has finished.
const FX_FINISHED: u32 = 0x021D_04C4;
/// `FUN_021AC154(h)`: delete a field effect.
const FX_DELETE: u32 = 0x021A_C154;
/// `FUN_0216D570(m, v)`: move bits &= !v.
const CLEAR_MOVE_BITS: u32 = 0x0216_D570;
/// `FUN_02163F8C(m, &v)`: position += `*v` (VecFx32).
const POS_ADD: u32 = 0x0216_3F8C;
/// SE of the draw animations of codes 0xB8 / 0xB9.
const SE_ANIM: u32 = 0x7B7;

/// C: `FUN_02198E60` (unlabeled routine at `0x02198E60`): acmd codes 0xB6 / 0xB7 init: start draw animation 11 (acmd work cleared,
/// 4 bytes; nothing written); acmd seq + 1; returns 0.
pub fn anim11_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(ACMD_WORK_INIT, &[m, 4]);
    env.callv(SET_ANIM, &[m, 11]);
    env.callv(INC_SEQ, &[m]);
    0
}

/// C: `FUN_02198E7C` (unlabeled routine at `0x02198E7C`): acmd codes 0xBA / 0xBB init: start draw animation 12 (acmd work
/// cleared, 4 bytes); acmd seq + 1; returns 0.
pub fn anim12_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(ACMD_WORK_INIT, &[m, 4]);
    env.callv(SET_ANIM, &[m, 12]);
    env.callv(INC_SEQ, &[m]);
    0
}

/// C: `FUN_02198EB0` (unlabeled routine at `0x02198EB0`): acmd code 0xB9: SE 0x7B7, then [`anim11_start`].
pub fn anim11_start_se<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(PLAY_SE, &[SE_ANIM]);
    anim11_start(env, m)
}

/// C: `FUN_02198F20` (unlabeled routine at `0x02198F20`): wait for the draw animation, keeping the animation code (codes 0xB5,
/// 0xB7, 0xBB): with a blact actor whose animation state is 1, acmd seq + 1, returns 1; else 0.
pub fn anim_wait_keep<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(ACMD_WORK, &[m]);
    let actor = env.callv(DRAW_ACTOR, &[m, 0]);
    if actor == NO_ACTOR {
        return 0;
    }
    let sys = env.callv(SYS_OF, &[m]);
    let draw = env.callv(DRAW_SYS, &[sys]);
    let blact = env.callv(BLACT_SYS, &[draw]);
    if env.call(
        ACTOR_ANIM,
        &mut [CArg::V(blact), CArg::V(actor & 0xFFFF), CArg::P(0)],
    ) != 1
    {
        return 0;
    }
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02198F68` (unlabeled routine at `0x02198F68`): acmd code 0x65 init (hop with the movement type's effect): acmd work
/// cleared (4 bytes: index 0); the effect of the move-proc work (`0219BF48`), if any, is started (`021D04A4`); draw offset 0;
/// move bits |= 0xA0, &= !0x8000; acmd seq + 1; returns 0.
pub fn hop_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(ACMD_WORK_INIT, &[m, 4]);
    let h = env.callv(MOVE_FX, &[m]);
    if h != 0 {
        env.callv(FX_START, &[h]);
    }
    let v = [0u8; 12];
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    env.callv(SET_MOVE_BITS, &[m, 0xA0]);
    env.callv(CLEAR_MOVE_BITS, &[m, 0x8000]);
    env.callv(INC_SEQ, &[m]);
    0
}

/// C: `FUN_02198FB0` (unlabeled routine at `0x02198FB0`): acmd code 0x65 update: draw offset = (0, [`JUMP_HEIGHTS`]`[0][index]`,
/// 0) (the high table, index = u32 `+0`); a finished effect of the move-proc work is deleted and forgotten (`0219BF3C(m, 0)`);
/// index += 2; below 0x10 (unsigned) -> 0; else draw offset 0, move bits |= 0x540, `0219BF54(m)`, acmd seq + 1, returns 1.
/// UNVERIFIED: an index above 15 panics (the original reads past the 16-word table).
pub fn hop_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let y = JUMP_HEIGHTS[0][env.r32(w) as usize];
    let v = vec_bytes(0, y, 0);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    let h = env.callv(MOVE_FX, &[m]);
    if h != 0 && env.callv(FX_FINISHED, &[h]) == 1 {
        env.callv(FX_DELETE, &[h]);
        env.callv(SET_MOVE_FX, &[m, 0]);
    }
    let i = env.r32(w).wrapping_add(2);
    env.w32(w, i);
    if i < 0x10 {
        return 0;
    }
    let v = [0u8; 12];
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    env.callv(SET_MOVE_BITS, &[m, 0x540]);
    env.callv(MOVE_FX_DONE, &[m]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02199034` (unlabeled routine at `0x02199034`): start a timed draw animation: acmd work (4) `u16 +2` = frames (the
/// counter `+0` is left to the clear); draw animation code = `anim`. The callers step the seq.
pub fn timed_anim_start<E: FieldEnv>(env: &mut E, m: u32, anim: u32, frames: u32) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 4]);
    env.w16(w.wrapping_add(2), frames as u16);
    env.callv(SET_ANIM, &[m, anim]);
}

/// C: `FUN_02199050` (unlabeled routine at `0x02199050`): timed animation update (codes 0x66 / 0x68 / 0xA2): counter (u16) + 1;
/// once it reaches the frames (u16 `+2`), acmd seq + 1. Returns 0.
pub fn timed_anim_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let c = env.r16(w).wrapping_add(1);
    env.w16(w, c);
    if env.r16(w) >= env.r16(w.wrapping_add(2)) {
        env.callv(INC_SEQ, &[m]);
    }
    0
}

/// C: `FUN_02199070` (unlabeled routine at `0x02199070`): acmd code 0x66: [`timed_anim_start`]`(m, 1, 40)`, acmd seq + 1; returns
/// 0.
pub fn timed_anim_66<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    timed_anim_start(env, m, 1, 0x28);
    env.callv(INC_SEQ, &[m]);
    0
}

/// C: `FUN_02199088` (unlabeled routine at `0x02199088`): acmd code 0x68: [`timed_anim_start`]`(m, 2, 40)`, acmd seq + 1; returns
/// 0.
pub fn timed_anim_68<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    timed_anim_start(env, m, 2, 0x28);
    env.callv(INC_SEQ, &[m]);
    0
}

/// C: `FUN_021990A0` (unlabeled routine at `0x021990A0`): acmd code 0xA2 (grid and rail): [`timed_anim_start`]`(m, 0, 12)`, acmd
/// seq + 1; returns 0.
pub fn timed_anim_a2<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    timed_anim_start(env, m, 0, 0xC);
    env.callv(INC_SEQ, &[m]);
    0
}

/// C: `FUN_021990B8` (unlabeled routine at `0x021990B8`): acmd code 0x9A (grid and rail) init: acmd work cleared (4 bytes: frame
/// 0); draw animation code 1; acmd seq + 1; returns 1.
pub fn bounce_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(ACMD_WORK_INIT, &[m, 4]);
    env.callv(SET_ANIM, &[m, 1]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// The vector-move inits: [`vec_move_start`] with a stack copy of the ROM VecFx32 at `vec` ([`VEC_MOVES`]). Not a ROM function
/// (the original inlines the copy in each init). UNVERIFIED: an address not in the table panics.
fn vec_move_rom<E: FieldEnv>(
    env: &mut E,
    m: u32,
    vec: u32,
    disp: u32,
    mv: u32,
    frames: u32,
    anim: u32,
) {
    let v = VEC_MOVES
        .iter()
        .find(|t| t.0 == vec)
        .map(|t| t.1)
        .unwrap_or_else(|| panic!("no ROM VecFx32 at {vec:#010x}"));
    let p = env.mem().push_local(12);
    env.mem().write(p, &vec_bytes(v[0], v[1], v[2]));
    vec_move_start(env, m, p, disp, mv, frames, anim);
    env.mem().pop_local(p);
}

/// C: `FUN_021991F0` (unlabeled routine at `0x021991F0`): vector move update: position += the VecFx32 at acmd work `+4`
/// (`02163F8C`); frames - 1; still above 0 -> 0; else move bits 0x440, old grid = grid, draw animation code 0, acmd seq + 1,
/// returns 1.
pub fn vec_move_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    env.call(POS_ADD, &mut [CArg::V(m), CArg::In(w.wrapping_add(4), 12)]);
    let n = env.r32(w).wrapping_sub(1);
    env.w32(w, n);
    if n as i32 > 0 {
        return 0;
    }
    env.callv(SET_MOVE_BITS, &[m, WALK_DONE]);
    env.callv(SAVE_OLD_GRID, &[m]);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// The ROM VecFx32 constants the vector-move inits copy to their stack (per-frame steps), by address (sorted).
pub const VEC_MOVES: [(u32, [u32; 3]); 32] = [
    (0x021D_41E0, [0, 0, 0xffffe000]),
    (0x021D_4204, [0, 0xffffe000, 0]),
    (0x021D_4210, [0, 0x2000, 0]),
    (0x021D_45E8, [0, 0, 0x4000]),
    (0x021D_4648, [0, 0, 0xffffe000]),
    (0x021D_4AD8, [0x4000, 0, 0]),
    (0x021D_4AE4, [0xffffc000, 0, 0]),
    (0x021D_4AF0, [0, 0, 0xffffc000]),
    (0x021D_4AFC, [0, 0, 0x4000]),
    (0x021D_4B08, [0, 0, 0x4000]),
    (0x021D_4B14, [0, 0, 0xffffc000]),
    (0x021D_4B20, [0, 0xffffc000, 0]),
    (0x021D_4B2C, [0, 0x4000, 0]),
    (0x021D_4B38, [0, 0, 0xffffc000]),
    (0x021D_4B44, [0, 0x2000, 0]),
    (0x021D_4B50, [0, 0xffffc000, 0]),
    (0x021D_4B5C, [0, 0x4000, 0]),
    (0x021D_4B68, [0x8000, 0, 0]),
    (0x021D_4B74, [0xffff8000, 0, 0]),
    (0x021D_4B80, [0, 0, 0xffff8000]),
    (0x021D_4B8C, [0, 0, 0x8000]),
    (0x021D_4B98, [0x4000, 0, 0]),
    (0x021D_4BA4, [0xffffc000, 0, 0]),
    (0x021D_4BB0, [0, 0, 0xffffc000]),
    (0x021D_4BBC, [0, 0, 0x4000]),
    (0x021D_4BC8, [0x2000, 0, 0]),
    (0x021D_4BD4, [0xffffe000, 0, 0]),
    (0x021D_4BE0, [0, 0, 0xffffe000]),
    (0x021D_4BEC, [0, 0, 0x2000]),
    (0x021D_4BF8, [0, 0, 0x2000]),
    (0x021D_4C04, [0, 0, 0x2000]),
    (0x021D_4C10, [0, 0xffffe000, 0]),
];

/// C: `FUN_02199234` (unlabeled routine at `0x02199234`): acmd code 0x69 (move by VecFx32 (0, 0x2000, 0) per frame for 8 frames,
/// display dir left, move dir up, anim 3): [`vec_move_rom`]`(m, 0x021D_4B44, 2, 0, 8, 3)`; returns 1.
pub fn vec_move_69<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B44, 2, 0, 8, 3);
    1
}

/// C: `FUN_02199268` (unlabeled routine at `0x02199268`): acmd code 0x6A (move by VecFx32 (0, -0x2000, 0) per frame for 8 frames,
/// display dir right, move dir down, anim 3): [`vec_move_rom`]`(m, 0x021D_4204, 3, 1, 8, 3)`; returns 1.
pub fn vec_move_6a<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4204, 3, 1, 8, 3);
    1
}

/// C: `FUN_02199298` (unlabeled routine at `0x02199298`): acmd code 0x6B (move by VecFx32 (0, 0, 0x2000) per frame for 8 frames,
/// display dir down, move dir left, anim 3): [`vec_move_rom`]`(m, 0x021D_4C04, 1, 2, 8, 3)`; returns 1.
pub fn vec_move_6b<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4C04, 1, 2, 8, 3);
    1
}

/// C: `FUN_021992CC` (unlabeled routine at `0x021992CC`): acmd code 0x6C (move by VecFx32 (0, 0, -0x2000) per frame for 8 frames,
/// display dir up, move dir right, anim 3): [`vec_move_rom`]`(m, 0x021D_41E0, 0, 3, 8, 3)`; returns 1.
pub fn vec_move_6c<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_41E0, 0, 3, 8, 3);
    1
}

/// C: `FUN_021992FC` (unlabeled routine at `0x021992FC`): acmd code 0x6D (move by VecFx32 (0, 0x2000, 0) per frame for 8 frames,
/// display dir right, move dir up, anim 3): [`vec_move_rom`]`(m, 0x021D_4210, 3, 0, 8, 3)`; returns 1.
pub fn vec_move_6d<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4210, 3, 0, 8, 3);
    1
}

/// C: `FUN_0219932C` (unlabeled routine at `0x0219932C`): acmd code 0x6E (move by VecFx32 (0, -0x2000, 0) per frame for 8 frames,
/// display dir left, move dir down, anim 3): [`vec_move_rom`]`(m, 0x021D_4C10, 2, 1, 8, 3)`; returns 1.
pub fn vec_move_6e<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4C10, 2, 1, 8, 3);
    1
}

/// C: `FUN_02199360` (unlabeled routine at `0x02199360`): acmd code 0x6F (move by VecFx32 (0, 0, -0x2000) per frame for 8 frames,
/// display dir up, move dir left, anim 3): [`vec_move_rom`]`(m, 0x021D_4648, 0, 2, 8, 3)`; returns 1.
pub fn vec_move_6f<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4648, 0, 2, 8, 3);
    1
}

/// C: `FUN_02199394` (unlabeled routine at `0x02199394`): acmd code 0x70 (move by VecFx32 (0, 0, 0x2000) per frame for 8 frames,
/// display dir down, move dir left, anim 3): [`vec_move_rom`]`(m, 0x021D_4BF8, 1, 2, 8, 3)`; returns 1.
pub fn vec_move_70<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4BF8, 1, 2, 8, 3);
    1
}

/// C: `FUN_021993C8` (unlabeled routine at `0x021993C8`): acmd code 0x71 (move by VecFx32 (0, 0, 0x2000) per frame for 8 frames,
/// display dir down, move dir up, anim 3): [`vec_move_rom`]`(m, 0x021D_4BEC, 1, 0, 8, 3)`; returns 1.
pub fn vec_move_71<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4BEC, 1, 0, 8, 3);
    1
}

/// C: `FUN_021993FC` (unlabeled routine at `0x021993FC`): acmd code 0x72 (move by VecFx32 (0, 0, -0x2000) per frame for 8 frames,
/// display dir up, move dir down, anim 3): [`vec_move_rom`]`(m, 0x021D_4BE0, 0, 1, 8, 3)`; returns 1.
pub fn vec_move_72<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4BE0, 0, 1, 8, 3);
    1
}

/// C: `FUN_02199430` (unlabeled routine at `0x02199430`): acmd code 0x73 (move by VecFx32 (-0x2000, 0, 0) per frame for 8 frames,
/// display dir right, move dir left, anim 3): [`vec_move_rom`]`(m, 0x021D_4BD4, 3, 2, 8, 3)`; returns 1.
pub fn vec_move_73<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4BD4, 3, 2, 8, 3);
    1
}

/// C: `FUN_02199460` (unlabeled routine at `0x02199460`): acmd code 0x74 (move by VecFx32 (0x2000, 0, 0) per frame for 8 frames,
/// display dir left, move dir right, anim 3): [`vec_move_rom`]`(m, 0x021D_4BC8, 2, 3, 8, 3)`; returns 1.
pub fn vec_move_74<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4BC8, 2, 3, 8, 3);
    1
}

/// C: `FUN_02199490` (unlabeled routine at `0x02199490`): acmd code 0x91 (move by VecFx32 (0, 0, 0x4000) per frame for 4 frames,
/// display dir down, move dir up, anim 4): [`vec_move_rom`]`(m, 0x021D_4BBC, 1, 0, 4, 4)`; returns 1.
pub fn vec_move_91<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4BBC, 1, 0, 4, 4);
    1
}

/// C: `FUN_021994C0` (unlabeled routine at `0x021994C0`): acmd code 0x92 (move by VecFx32 (0, 0, -0x4000) per frame for 4 frames,
/// display dir up, move dir down, anim 4): [`vec_move_rom`]`(m, 0x021D_4BB0, 0, 1, 4, 4)`; returns 1.
pub fn vec_move_92<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4BB0, 0, 1, 4, 4);
    1
}

/// C: `FUN_021994F0` (unlabeled routine at `0x021994F0`): acmd code 0x93 (move by VecFx32 (-0x4000, 0, 0) per frame for 4 frames,
/// display dir right, move dir left, anim 4): [`vec_move_rom`]`(m, 0x021D_4BA4, 3, 2, 4, 4)`; returns 1.
pub fn vec_move_93<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4BA4, 3, 2, 4, 4);
    1
}

/// C: `FUN_02199520` (unlabeled routine at `0x02199520`): acmd code 0x94 (move by VecFx32 (0x4000, 0, 0) per frame for 4 frames,
/// display dir left, move dir right, anim 4): [`vec_move_rom`]`(m, 0x021D_4B98, 2, 3, 4, 4)`; returns 1.
pub fn vec_move_94<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B98, 2, 3, 4, 4);
    1
}

/// C: `FUN_02199550` (unlabeled routine at `0x02199550`): acmd code 0x95 (move by VecFx32 (0, 0, 0x8000) per frame for 2 frames,
/// display dir down, move dir up, anim 5): [`vec_move_rom`]`(m, 0x021D_4B8C, 1, 0, 2, 5)`; returns 1.
pub fn vec_move_95<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B8C, 1, 0, 2, 5);
    1
}

/// C: `FUN_02199584` (unlabeled routine at `0x02199584`): acmd code 0x96 (move by VecFx32 (0, 0, -0x8000) per frame for 2 frames,
/// display dir up, move dir down, anim 5): [`vec_move_rom`]`(m, 0x021D_4B80, 0, 1, 2, 5)`; returns 1.
pub fn vec_move_96<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B80, 0, 1, 2, 5);
    1
}

/// C: `FUN_021995B8` (unlabeled routine at `0x021995B8`): acmd code 0x97 (move by VecFx32 (-0x8000, 0, 0) per frame for 2 frames,
/// display dir right, move dir left, anim 5): [`vec_move_rom`]`(m, 0x021D_4B74, 3, 2, 2, 5)`; returns 1.
pub fn vec_move_97<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B74, 3, 2, 2, 5);
    1
}

/// C: `FUN_021995E8` (unlabeled routine at `0x021995E8`): acmd code 0x98 (move by VecFx32 (0x8000, 0, 0) per frame for 2 frames,
/// display dir left, move dir right, anim 5): [`vec_move_rom`]`(m, 0x021D_4B68, 2, 3, 2, 5)`; returns 1.
pub fn vec_move_98<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B68, 2, 3, 2, 5);
    1
}

/// C: `FUN_02199618` (unlabeled routine at `0x02199618`): acmd code 0x79 (move by VecFx32 (0, 0x4000, 0) per frame for 4 frames,
/// display dir left, move dir up, anim 10): [`vec_move_rom`]`(m, 0x021D_4B5C, 2, 0, 4, 0xa)`; returns 1.
pub fn vec_move_79<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B5C, 2, 0, 4, 0xa);
    1
}

/// C: `FUN_0219964C` (unlabeled routine at `0x0219964C`): acmd code 0x7A (move by VecFx32 (0, -0x4000, 0) per frame for 4 frames,
/// display dir right, move dir down, anim 10): [`vec_move_rom`]`(m, 0x021D_4B50, 3, 1, 4, 0xa)`; returns 1.
pub fn vec_move_7a<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B50, 3, 1, 4, 0xa);
    1
}

/// C: `FUN_02199680` (unlabeled routine at `0x02199680`): acmd code 0x7B (move by VecFx32 (0, 0, 0x4000) per frame for 4 frames,
/// display dir down, move dir left, anim 10): [`vec_move_rom`]`(m, 0x021D_45E8, 1, 2, 4, 0xa)`; returns 1.
pub fn vec_move_7b<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_45E8, 1, 2, 4, 0xa);
    1
}

/// C: `FUN_021996B4` (unlabeled routine at `0x021996B4`): acmd code 0x7C (move by VecFx32 (0, 0, -0x4000) per frame for 4 frames,
/// display dir up, move dir right, anim 10): [`vec_move_rom`]`(m, 0x021D_4B38, 0, 3, 4, 0xa)`; returns 1.
pub fn vec_move_7c<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B38, 0, 3, 4, 0xa);
    1
}

/// C: `FUN_021996E8` (unlabeled routine at `0x021996E8`): acmd code 0x7D (move by VecFx32 (0, 0x4000, 0) per frame for 4 frames,
/// display dir right, move dir up, anim 10): [`vec_move_rom`]`(m, 0x021D_4B2C, 3, 0, 4, 0xa)`; returns 1.
pub fn vec_move_7d<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B2C, 3, 0, 4, 0xa);
    1
}

/// C: `FUN_0219971C` (unlabeled routine at `0x0219971C`): acmd code 0x7E (move by VecFx32 (0, -0x4000, 0) per frame for 4 frames,
/// display dir left, move dir down, anim 10): [`vec_move_rom`]`(m, 0x021D_4B20, 2, 1, 4, 0xa)`; returns 1.
pub fn vec_move_7e<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B20, 2, 1, 4, 0xa);
    1
}

/// C: `FUN_02199750` (unlabeled routine at `0x02199750`): acmd code 0x7F (move by VecFx32 (0, 0, -0x4000) per frame for 4 frames,
/// display dir up, move dir left, anim 10): [`vec_move_rom`]`(m, 0x021D_4B14, 0, 2, 4, 0xa)`; returns 1.
pub fn vec_move_7f<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B14, 0, 2, 4, 0xa);
    1
}

/// C: `FUN_02199784` (unlabeled routine at `0x02199784`): acmd code 0x80 (move by VecFx32 (0, 0, 0x4000) per frame for 4 frames,
/// display dir down, move dir left, anim 10): [`vec_move_rom`]`(m, 0x021D_4B08, 1, 2, 4, 0xa)`; returns 1.
pub fn vec_move_80<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4B08, 1, 2, 4, 0xa);
    1
}

/// C: `FUN_021997B8` (unlabeled routine at `0x021997B8`): acmd code 0x81 (move by VecFx32 (0, 0, 0x4000) per frame for 4 frames,
/// display dir down, move dir up, anim 10): [`vec_move_rom`]`(m, 0x021D_4AFC, 1, 0, 4, 0xa)`; returns 1.
pub fn vec_move_81<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4AFC, 1, 0, 4, 0xa);
    1
}

/// C: `FUN_021997EC` (unlabeled routine at `0x021997EC`): acmd code 0x82 (move by VecFx32 (0, 0, -0x4000) per frame for 4 frames,
/// display dir up, move dir down, anim 10): [`vec_move_rom`]`(m, 0x021D_4AF0, 0, 1, 4, 0xa)`; returns 1.
pub fn vec_move_82<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4AF0, 0, 1, 4, 0xa);
    1
}

/// C: `FUN_02199820` (unlabeled routine at `0x02199820`): acmd code 0x83 (move by VecFx32 (-0x4000, 0, 0) per frame for 4 frames,
/// display dir right, move dir left, anim 10): [`vec_move_rom`]`(m, 0x021D_4AE4, 3, 2, 4, 0xa)`; returns 1.
pub fn vec_move_83<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4AE4, 3, 2, 4, 0xa);
    1
}

/// C: `FUN_02199854` (unlabeled routine at `0x02199854`): acmd code 0x84 (move by VecFx32 (0x4000, 0, 0) per frame for 4 frames,
/// display dir left, move dir right, anim 10): [`vec_move_rom`]`(m, 0x021D_4AD8, 2, 3, 4, 0xa)`; returns 1.
pub fn vec_move_84<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    vec_move_rom(env, m, 0x021D_4AD8, 2, 3, 4, 0xa);
    1
}

#[cfg(test)]
mod tests;
