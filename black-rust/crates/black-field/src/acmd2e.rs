//! `fldmmdl_acmd.c`, part 2e (overlay 21, `0x02199888..0x0219A098`; F4b job 2): the arc moves (02199888 and its 12 init steps,
//! update 02199970) and the special codes 0xAB..0xB4 / 0xBC (counters, a slow step left, waits on the 3D model animation).
//!
//! F4b-spec2 specced all 32 functions (`tools/rom-diff/specs/black-field/acmd2e.spec`); F4b-port1 ported them.

use crate::acmd2a::{
    ACMD_WORK, ACMD_WORK_INIT, GRID_STEP, INC_SEQ, PLAY_SE, POS_HEIGHT, POS_MOVE, SAVE_OLD_GRID,
    SET_ANIM, SET_DIR_DISP, SET_DRAW_OFFSET, SET_MOVE_BITS,
};
use crate::acmd2b::{vec_bytes, JUMP_HEIGHTS};
use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_9888,
    0x0219_9970,
    0x0219_9AEC,
    0x0219_9B10,
    0x0219_9B38,
    0x0219_9B60,
    0x0219_9B88,
    0x0219_9BAC,
    0x0219_9BD8,
    0x0219_9C00,
    0x0219_9C28,
    0x0219_9C50,
    0x0219_9C7C,
    0x0219_9CA4,
    0x0219_9CC8,
    0x0219_9CE4,
    0x0219_9D00,
    0x0219_9D14,
    0x0219_9D4C,
    0x0219_9D8C,
    0x0219_9DB4,
    0x0219_9EAC,
    0x0219_9EC0,
    0x0219_9ED4,
    0x0219_9F30,
    0x0219_9F50,
    0x0219_9F94,
    0x0219_A014,
    0x0219_A028,
    0x0219_A03C,
    0x0219_A058,
    0x0219_A07C,
];

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
/// `FUN_0216D88C(m, &out)`: the position VecFx32.
const GET_POS: u32 = 0x0216_D88C;
/// `FUN_0216D89C(m, &v)`: position = `*v`.
const SET_POS: u32 = 0x0216_D89C;
/// `FUN_0216D598(m)`: the OBJ code.
const OBJ_CODE: u32 = 0x0216_D598;
/// `FUN_0219FCC0(m, &kind, &frame)` (fldmmdl_g3dobj): the 3D model animation frame; 1 when valid.
const MODEL_FRAME: u32 = 0x0219_FCC0;
/// Move bits a finished arc sets: 0x400 (done) | 0x100 | 0x40.
const LANDED: u32 = 0x540;

/// C: `_s32_div_f` (`FUN_0209C0A4`, ARM9 library): signed quotient truncating toward zero; a zero divisor returns the
/// dividend (the routine skips the loop and restores the sign; checked in the disassembly, F4b-port1 / V-0026).
fn s32_div(n: i32, d: i32) -> i32 {
    if d == 0 {
        n
    } else {
        n.wrapping_div(d)
    }
}

/// The grid step of an arc move on `axis` (0 x, 1 y, 2 z): one tile by the sign of `dist` (y moves by two units).
fn arc_grid_step<E: FieldEnv>(env: &mut E, m: u32, axis: u8, dist: i32) {
    let s: i32 = if dist < 0 { -1 } else { 1 };
    match axis {
        0 => env.callv(ADD_GRID[0], &[m, s as u32]),
        1 => env.callv(ADD_GRID[1], &[m, (2 * s) as u32]),
        2 => env.callv(ADD_GRID[2], &[m, s as u32]),
        _ => 0,
    };
}

/// Old grid x / y / z = grid x / y / z.
fn save_old_grid_xyz<E: FieldEnv>(env: &mut E, m: u32) {
    for k in 0..3 {
        let g = env.callv(GRID[k], &[m]);
        env.callv(SET_OLD_GRID[k], &[m, g]);
    }
}

/// C: `FUN_02199888` (unlabeled routine at `0x02199888`): start an arc move: acmd work (0x10) = `u8 frames`, `u8 arc_axis`,
/// `u8 axis`, `u8 neg`, `s32 dist` (fx32 per frame), `u16 +0xE` = 0x1000 / frames; display dir, move dir (u16), draw animation code
/// (u8), move bit 0x20, old grid = grid; `dist` != 0 steps the grid one tile on `axis` by its sign; acmd seq + 1. The stack
/// arguments are read as bytes. Frames 0 gives a step of 0x1000 ([`s32_div`] returns the dividend for a zero divisor;
/// V-0026).
#[allow(clippy::too_many_arguments)]
pub fn arc_start<E: FieldEnv>(
    env: &mut E,
    m: u32,
    dist: u32,
    disp: u32,
    mv: u32,
    frames: u32,
    anim: u32,
    axis: u32,
    arc_axis: u32,
    neg: u32,
) {
    let w = env.callv(ACMD_WORK_INIT, &[m, 0x10]);
    env.w8(w, frames as u8);
    env.w32(w.wrapping_add(4), dist);
    env.w8(w.wrapping_add(2), axis as u8);
    env.w8(w.wrapping_add(1), arc_axis as u8);
    env.w8(w.wrapping_add(3), neg as u8);
    let step = s32_div(0x1000, i32::from(env.r8(w)));
    env.w16(w.wrapping_add(0xE), step as u16);
    env.callv(SET_DIR_DISP, &[m, disp & 0xFFFF]);
    env.callv(SET_DIR_MOVE, &[m, mv & 0xFFFF]);
    env.callv(SET_ANIM, &[m, anim & 0xFF]);
    env.callv(SET_BIT20, &[m]);
    save_old_grid_xyz(env, m);
    if dist != 0 {
        arc_grid_step(env, m, axis as u8, dist as i32);
    }
    env.callv(INC_SEQ, &[m]);
}

/// C: `FUN_02199970` (unlabeled routine at `0x02199970`): arc move update: the position moves by dist on the axis; the distance
/// grows by |dist| and the phase by the step (capped at 0xF00); the draw offset on the arc axis is the low jump height
/// ([`JUMP_HEIGHTS`]`[1][phase >> 8]`, negated when `neg` is 1); frames - 1; every whole tile of distance while frames are left
/// moves the grid by the sign of dist (also for dist 0: +1); the last frame clears the draw offset, sets move bits 0x540, old grid
/// = grid, draw animation code 0, acmd seq + 1, and returns 1; else 0.
pub fn arc_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let mut pos = [0u8; 12];
    env.call(GET_POS, &mut [CArg::V(m), CArg::OutBuf(&mut pos)]);
    let axis = env.r8(w.wrapping_add(2));
    if axis < 3 {
        let o = 4 * usize::from(axis);
        let c = u32::from_le_bytes([pos[o], pos[o + 1], pos[o + 2], pos[o + 3]]);
        let c = c.wrapping_add(env.r32(w.wrapping_add(4)));
        pos[o..o + 4].copy_from_slice(&c.to_le_bytes());
    }
    env.call(SET_POS, &mut [CArg::V(m), CArg::InBuf(&pos)]);
    let ad = (env.r32(w.wrapping_add(4)) as i32).wrapping_abs() as u32;
    let d = env.r32(w.wrapping_add(8)).wrapping_add(ad);
    env.w32(w.wrapping_add(8), d);
    let mut phase = env
        .r16(w.wrapping_add(0xC))
        .wrapping_add(env.r16(w.wrapping_add(0xE)));
    if phase > 0xF00 {
        phase = 0xF00;
    }
    env.w16(w.wrapping_add(0xC), phase);
    let mut y = JUMP_HEIGHTS[1][usize::from(phase >> 8)];
    if env.r8(w.wrapping_add(3)) == 1 {
        y = y.wrapping_neg();
    }
    let mut off = [0u32; 3];
    let a = env.r8(w.wrapping_add(1));
    if a < 3 {
        off[usize::from(a)] = y;
    }
    let v = vec_bytes(off[0], off[1], off[2]);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    let n = env.r8(w).wrapping_sub(1);
    env.w8(w, n);
    let d = env.r32(w.wrapping_add(8));
    if d as i32 >= 0x1_0000 && n != 0 {
        env.w32(w.wrapping_add(8), d.wrapping_sub(0x1_0000));
        let dist = env.r32(w.wrapping_add(4)) as i32;
        save_old_grid_xyz(env, m);
        let axis = env.r8(w.wrapping_add(2));
        arc_grid_step(env, m, axis, dist);
    }
    if env.r8(w) != 0 {
        return 0;
    }
    let v = [0u8; 12];
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    env.callv(SET_MOVE_BITS, &[m, LANDED]);
    env.callv(SAVE_OLD_GRID, &[m]);
    env.callv(SET_ANIM, &[m, 0]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02199AEC` (unlabeled routine at `0x02199AEC`): acmd code 0x85: [`arc_start`]`(m, 0x2000, 2, 0, 8, 3, 1, 1, 0)`
/// (x += 0x2000 per frame for 8 frames, the arc on y); returns 1.
pub fn arc_85<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0x2000, 2, 0, 8, 3, 1, 1, 0);
    1
}

/// C: `FUN_02199CE4` (unlabeled routine at `0x02199CE4`): acmd code 0xAB update: frame + 1; above 0x15, acmd seq + 1. Returns 0.
pub fn count_22<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let n = env.r32(w).wrapping_add(1);
    env.w32(w, n);
    if n as i32 > 0x15 {
        env.callv(INC_SEQ, &[m]);
    }
    0
}

/// C: `FUN_02199D14` (unlabeled routine at `0x02199D14`): acmd code 0xAD: start a slow step left (acmd work = anim 0xC, frames 2,
/// dir 2, speed 0x8000; the grid steps one tile left; draw animation code = the stored anim); acmd seq + 1; returns 1.
pub fn slow_left_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK_INIT, &[m, 0xC]);
    env.w16(w, 0xC);
    env.w16(w.wrapping_add(2), 2);
    env.w32(w.wrapping_add(4), 2);
    env.w32(w.wrapping_add(8), 0x8000);
    env.callv(GRID_STEP, &[m, 2]);
    let anim = u32::from(env.r16(w));
    env.callv(SET_ANIM, &[m, anim]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02199F94` (unlabeled routine at `0x02199F94`): acmd code 0xBC update: while the 3D model animation frame is valid,
/// count frames (u16 +0); for OBJ code 0xDD with animation kind 0, a new frame (frame - 1 != u8 +2) is stored and frame 0 / 3
/// play SE 0x7E5 / 0x7E6; from 0x5F counted frames on, acmd seq + 1, returns 1; else 0.
pub fn model_wait<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let mut kind = [0u8; 2];
    let mut frame = [0u8; 2];
    let ok = env.call(
        MODEL_FRAME,
        &mut [
            CArg::V(m),
            CArg::OutBuf(&mut kind),
            CArg::OutBuf(&mut frame),
        ],
    );
    if ok != 1 {
        return 0;
    }
    let c = env.r16(w).wrapping_add(1);
    env.w16(w, c);
    let frame = u16::from_le_bytes(frame).wrapping_sub(1);
    let kind = u16::from_le_bytes(kind);
    if env.callv(OBJ_CODE, &[m]) == 0xDD
        && kind == 0
        && u16::from(env.r8(w.wrapping_add(2))) != frame
    {
        env.w8(w.wrapping_add(2), frame as u8);
        match frame {
            0 => {
                env.callv(PLAY_SE, &[0x7E5]);
            }
            3 => {
                env.callv(PLAY_SE, &[0x7E6]);
            }
            _ => {}
        }
    }
    if env.r16(w) < 0x5F {
        return 0;
    }
    env.callv(INC_SEQ, &[m]);
    1
}

/// `FUN_0219FC68(m, &kind, &frame)` (fldmmdl_g3dobj): the 3D model animation frame; 1 when valid.
const MODEL_FRAME_RISE: u32 = 0x0219_FC68;
/// `FUN_0219FE64(m)` (fldmmdl_g3dobj): start the 3D model animation (D3).
const MODEL_ANIM_START: u32 = 0x0219_FE64;
/// `FUN_0219FE20(m)` (fldmmdl_g3dobj): 1 when the 3D model animation is done.
const MODEL_ANIM_DONE: u32 = 0x0219_FE20;
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
/// ROM `0x021D4ACC`: the draw offset of acmd code 0xBC (VecFx32).
pub const MODEL_OFFSET: [u32; 3] = [0, 0x8000, 0];

/// Draw animation code `anim`, acmd seq + 1, returns `r`.
fn anim_step<E: FieldEnv>(env: &mut E, m: u32, anim: u32, r: u32) -> u32 {
    env.callv(SET_ANIM, &[m, anim]);
    env.callv(INC_SEQ, &[m]);
    r
}

/// C: `FUN_02199CC8` (unlabeled routine at `0x02199CC8`): acmd code 0xAB init: acmd work cleared (4 bytes: counter 0); draw
/// animation code 1; acmd seq + 1; returns 0.
pub fn count_22_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(ACMD_WORK_INIT, &[m, 4]);
    anim_step(env, m, 1, 0)
}

/// C: `FUN_02199D00` (unlabeled routine at `0x02199D00`): acmd code 0xAC: draw animation code 10; acmd seq + 1; returns 1.
pub fn anim10_ac<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    anim_step(env, m, 10, 1)
}

/// C: `FUN_02199D4C` (unlabeled routine at `0x02199D4C`): acmd code 0xAD update (as `acmd2a::walk_update` without the done
/// bits): position += speed (`+8`) in the u16 dir (`+4`), map height; frames (s16 `+2`) - 1; still above 0 -> 0; else old grid =
/// grid, acmd seq + 1, returns 1.
pub fn slow_left_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let dir = env.r32(w.wrapping_add(4)) & 0xFFFF;
    let speed = env.r32(w.wrapping_add(8));
    env.callv(POS_MOVE, &[m, dir, speed]);
    env.callv(POS_HEIGHT, &[m]);
    let n = (env.r16(w.wrapping_add(2)) as i16).wrapping_sub(1);
    env.w16(w.wrapping_add(2), n as u16);
    if n > 0 {
        return 0;
    }
    env.callv(SAVE_OLD_GRID, &[m]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02199D8C` (unlabeled routine at `0x02199D8C`): acmd code 0xAE init: acmd work (0x10) `+4` = 0x2000 (rise speed), `u16
/// +8` = 0 (counter), `u8 +0xA` = 0 (last frame; the height `+0` is left to the clear); draw animation code 0xB; acmd seq + 1;
/// returns 1.
pub fn model_rise_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK_INIT, &[m, 0x10]);
    env.w32(w.wrapping_add(4), 0x2000);
    env.w16(w.wrapping_add(8), 0);
    env.w8(w.wrapping_add(0xA), 0);
    anim_step(env, m, 0xB, 1)
}

/// C: `FUN_02199DB4` (unlabeled routine at `0x02199DB4`): acmd code 0xAE update: wait on the 3D model animation (`0219FC68`; not
/// valid -> 0), then rise: counter (u16 `+8`) + 1; frame - 1 (u16); for OBJ code 0xD3 with animation kind 2, a new frame
/// (frame != u8 `+0xA`) is stored and frames 7 / 12 / 17 play SE 0x7E5, 10 / 15 / 20 SE 0x7E6; for other OBJ codes the counter
/// reaching 0x5F plays SE 0x7E7. Below 0x5F counted frames -> 0; else height += speed, draw offset = (0, height, 0), below 0x10
/// half tiles (`height / 0x8000`, signed) -> 0, else draw animation code 0, acmd seq + 1, returns 1.
pub fn model_rise_update<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let w = env.callv(ACMD_WORK, &[m]);
    let mut kind = [0u8; 2];
    let mut frame = [0u8; 2];
    let ok = env.call(
        MODEL_FRAME_RISE,
        &mut [
            CArg::V(m),
            CArg::OutBuf(&mut kind),
            CArg::OutBuf(&mut frame),
        ],
    );
    if ok != 1 {
        return 0;
    }
    let c = env.r16(w.wrapping_add(8)).wrapping_add(1);
    env.w16(w.wrapping_add(8), c);
    let frame = u16::from_le_bytes(frame).wrapping_sub(1);
    let kind = u16::from_le_bytes(kind);
    if env.callv(OBJ_CODE, &[m]) == 0xD3 {
        if kind == 2 && u16::from(env.r8(w.wrapping_add(0xA))) != frame {
            env.w8(w.wrapping_add(0xA), frame as u8);
            match frame {
                7 | 12 | 17 => {
                    env.callv(PLAY_SE, &[0x7E5]);
                }
                10 | 15 | 20 => {
                    env.callv(PLAY_SE, &[0x7E6]);
                }
                _ => {}
            }
        }
    } else if env.r16(w.wrapping_add(8)) == 0x5F {
        env.callv(PLAY_SE, &[0x7E7]);
    }
    if env.r16(w.wrapping_add(8)) < 0x5F {
        return 0;
    }
    let h = env.r32(w).wrapping_add(env.r32(w.wrapping_add(4)));
    env.w32(w, h);
    let v = vec_bytes(0, env.r32(w), 0);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    if (env.r32(w) as i32) / 0x8000 < 0x10 {
        return 0;
    }
    anim_step(env, m, 0, 1)
}

/// C: `FUN_02199EAC` (unlabeled routine at `0x02199EAC`): acmd code 0xAF: draw animation code 10; acmd seq + 1; returns 1.
pub fn anim10_af<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    anim_step(env, m, 10, 1)
}

/// C: `FUN_02199EC0` (unlabeled routine at `0x02199EC0`): acmd code 0xB0 init: draw animation code 11; acmd seq + 1; returns 0.
pub fn anim11_b0<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    anim_step(env, m, 11, 0)
}

/// C: `FUN_02199ED4` (unlabeled routine at `0x02199ED4`): acmd code 0xB0 update: while the MMDL's blact actor (if it has one)
/// still animates (state 0) -> 0; else display dir down (1), draw animation code 0, acmd seq + 1, returns 1.
pub fn anim_face_down<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let sys = env.callv(SYS_OF, &[m]);
    let draw = env.callv(DRAW_SYS, &[sys]);
    let blact = env.callv(BLACT_SYS, &[draw]);
    let actor = env.callv(DRAW_ACTOR, &[m, 0]);
    if actor != 0xFFFF
        && env.call(
            ACTOR_ANIM,
            &mut [CArg::V(blact), CArg::V(actor & 0xFFFF), CArg::P(0)],
        ) == 0
    {
        return 0;
    }
    env.callv(SET_DIR_DISP, &[m, 1]);
    anim_step(env, m, 0, 1)
}

/// C: `FUN_02199F30` (unlabeled routine at `0x02199F30`): acmd code 0xB4: draw offset 0; acmd seq + 1; returns 1.
pub fn offset_reset<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let v = [0u8; 12];
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02199F50` (unlabeled routine at `0x02199F50`): acmd code 0xBC init: acmd work (6) = `u16` counter 0, `u8 +2` last frame
/// 0; draw offset = [`MODEL_OFFSET`] (ROM `0x021D4ACC`); draw animation code 10; acmd seq + 1; returns 1.
pub fn model_wait_start<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    let v = vec_bytes(MODEL_OFFSET[0], MODEL_OFFSET[1], MODEL_OFFSET[2]);
    let w = env.callv(ACMD_WORK_INIT, &[m, 6]);
    env.w16(w, 0);
    env.w8(w.wrapping_add(2), 0);
    env.call(SET_DRAW_OFFSET, &mut [CArg::V(m), CArg::InBuf(&v)]);
    anim_step(env, m, 10, 1)
}

/// C: `FUN_0219A014` (unlabeled routine at `0x0219A014`): acmd code 0xB1 init: draw animation code 10; acmd seq + 1; returns 0.
pub fn anim10_b1<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    anim_step(env, m, 10, 0)
}

/// C: `FUN_0219A028` (unlabeled routine at `0x0219A028`): acmd code 0xB2 init: draw animation code 11; acmd seq + 1; returns 0.
pub fn anim11_b2<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    anim_step(env, m, 11, 0)
}

/// C: `FUN_0219A03C` (unlabeled routine at `0x0219A03C`): acmd code 0xB3 init: draw animation code 12, start the 3D model
/// animation (`0219FE64`, D3); acmd seq + 1; returns 0.
pub fn anim12_b3<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    env.callv(SET_ANIM, &[m, 12]);
    env.callv(MODEL_ANIM_START, &[m]);
    env.callv(INC_SEQ, &[m]);
    0
}

/// C: `FUN_0219A058` (unlabeled routine at `0x0219A058`): acmd code 0xB1 update: once the 3D model animation is done
/// (`0219FE20 == 1`), draw animation code 0, acmd seq + 1, returns 1; else 0.
pub fn model_done_b1<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    if env.callv(MODEL_ANIM_DONE, &[m]) != 1 {
        return 0;
    }
    anim_step(env, m, 0, 1)
}

/// C: `FUN_0219A07C` (unlabeled routine at `0x0219A07C`): acmd codes 0xB2 / 0xB3 update: once the 3D model animation is done,
/// acmd seq + 1, returns 1; else 0.
pub fn model_done<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    if env.callv(MODEL_ANIM_DONE, &[m]) != 1 {
        return 0;
    }
    env.callv(INC_SEQ, &[m]);
    1
}

/// C: `FUN_02199B10` (unlabeled routine at `0x02199B10`): acmd code 0x86 (move -0x2000 per frame along y for 8 frames (display
/// dir right, move dir down, anim 3), arc offset on y): [`arc_start`]`(m, 0xffffe000, 3, 1, 8, 3, 1, 1, 0)`; returns 1.
pub fn arc_86<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0xffffe000, 3, 1, 8, 3, 1, 1, 0);
    1
}

/// C: `FUN_02199B38` (unlabeled routine at `0x02199B38`): acmd code 0x87 (move 0x2000 per frame along z for 8 frames (display dir
/// down, move dir left, anim 3), arc offset on y): [`arc_start`]`(m, 0x2000, 1, 2, 8, 3, 2, 1, 0)`; returns 1.
pub fn arc_87<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0x2000, 1, 2, 8, 3, 2, 1, 0);
    1
}

/// C: `FUN_02199B60` (unlabeled routine at `0x02199B60`): acmd code 0x88 (move -0x2000 per frame along z for 8 frames (display
/// dir up, move dir right, anim 3), arc offset on y): [`arc_start`]`(m, 0xffffe000, 0, 3, 8, 3, 2, 1, 0)`; returns 1.
pub fn arc_88<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0xffffe000, 0, 3, 8, 3, 2, 1, 0);
    1
}

/// C: `FUN_02199B88` (unlabeled routine at `0x02199B88`): acmd code 0x89 (move 0x2000 per frame along y for 8 frames (display dir
/// right, move dir up, anim 3), arc offset on y): [`arc_start`]`(m, 0x2000, 3, 0, 8, 3, 1, 1, 0)`; returns 1.
pub fn arc_89<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0x2000, 3, 0, 8, 3, 1, 1, 0);
    1
}

/// C: `FUN_02199BAC` (unlabeled routine at `0x02199BAC`): acmd code 0x8A (move -0x2000 per frame along y for 8 frames (display
/// dir left, move dir down, anim 3), arc offset on y): [`arc_start`]`(m, 0xffffe000, 2, 1, 8, 3, 1, 1, 0)`; returns 1.
pub fn arc_8a<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0xffffe000, 2, 1, 8, 3, 1, 1, 0);
    1
}

/// C: `FUN_02199BD8` (unlabeled routine at `0x02199BD8`): acmd code 0x8B (move -0x2000 per frame along z for 8 frames (display
/// dir up, move dir left, anim 3), arc offset on y): [`arc_start`]`(m, 0xffffe000, 0, 2, 8, 3, 2, 1, 0)`; returns 1.
pub fn arc_8b<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0xffffe000, 0, 2, 8, 3, 2, 1, 0);
    1
}

/// C: `FUN_02199C00` (unlabeled routine at `0x02199C00`): acmd code 0x8C (move 0x2000 per frame along z for 8 frames (display dir
/// down, move dir left, anim 3), arc offset on y): [`arc_start`]`(m, 0x2000, 1, 2, 8, 3, 2, 1, 0)`; returns 1.
pub fn arc_8c<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0x2000, 1, 2, 8, 3, 2, 1, 0);
    1
}

/// C: `FUN_02199C28` (unlabeled routine at `0x02199C28`): acmd code 0x8D (move 0x2000 per frame along z for 8 frames (display dir
/// down, move dir up, anim 3), arc offset on y): [`arc_start`]`(m, 0x2000, 1, 0, 8, 3, 2, 1, 0)`; returns 1.
pub fn arc_8d<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0x2000, 1, 0, 8, 3, 2, 1, 0);
    1
}

/// C: `FUN_02199C50` (unlabeled routine at `0x02199C50`): acmd code 0x8E (move -0x2000 per frame along z for 8 frames (display
/// dir up, move dir down, anim 3), arc offset on y): [`arc_start`]`(m, 0xffffe000, 0, 1, 8, 3, 2, 1, 0)`; returns 1.
pub fn arc_8e<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0xffffe000, 0, 1, 8, 3, 2, 1, 0);
    1
}

/// C: `FUN_02199C7C` (unlabeled routine at `0x02199C7C`): acmd code 0x8F (move -0x2000 per frame along x for 8 frames (display
/// dir right, move dir left, anim 3), arc offset on y): [`arc_start`]`(m, 0xffffe000, 3, 2, 8, 3, 0, 1, 0)`; returns 1.
pub fn arc_8f<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0xffffe000, 3, 2, 8, 3, 0, 1, 0);
    1
}

/// C: `FUN_02199CA4` (unlabeled routine at `0x02199CA4`): acmd code 0x90 (move 0x2000 per frame along x for 8 frames (display dir
/// left, move dir right, anim 3), arc offset on y): [`arc_start`]`(m, 0x2000, 2, 3, 8, 3, 0, 1, 0)`; returns 1.
pub fn arc_90<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    arc_start(env, m, 0x2000, 2, 3, 8, 3, 0, 1, 0);
    1
}

#[cfg(test)]
mod tests;
