//! `fldmmdl_acmd.c`, part 2d (overlay 21, `0x02198E44..0x02199888`; F4b job 2): draw-animation starts and waits (codes 0x64,
//! 0xB5..0xBB), code 0x65, the timed animations 0x66 / 0x68 / 0xA2, the bounce 0x9A and the vector moves (0219912C, update
//! 021991F0).
//!
//! F4b-spec2 specced all 50 functions (`tools/rom-diff/specs/black-field/acmd2d.spec`); this module holds the smoke ports.

use crate::acmd2a::{
    ACMD_WORK, ACMD_WORK_INIT, INC_SEQ, PLAY_SE, SET_ANIM, SET_DIR_DISP, SET_DRAW_OFFSET,
};
use crate::acmd2b::vec_bytes;
use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0219_8E44,
    0x0219_8E98,
    0x0219_8EC8,
    0x0219_90D4,
    0x0219_912C,
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

#[cfg(test)]
mod tests;
