//! MMDL draw-side glue (overlay 21, `0x02197C4C..0x02197D3C`; F1 counted it in row 21.08 `ircbattlemenu.c`, whose code ends at
//! `0x02197C4A`): delete the draw systems of an MMDLSYS (02197C4C), the per-frame draw of one MMDL (02197C78), the "animation
//! held" predicate (02197C9C) and the draw position (02197CCC). The file is unknown (no allocation, so no `__FILE__`): the head of
//! `fldmmdl_acmd.c` or a small file between `ircbattlemenu.c` and it.
//!
//! F4b-spec1 specced all four (`tools/rom-diff/specs/black-field/mmdl_draw.spec`); F4b-port1 ported them. The
//! fldmmdl.c accessors and the blact / g3dobj draw side (D3) are reached through [`FieldEnv::call`] by address.

use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0219_7C4C, 0x0219_7C78, 0x0219_7C9C, 0x0219_7CCC];

/// `FUN_0216D520`: the MMDLSYS draw (blact) system (`+0x24`) or null.
const SYS_BLACT: u32 = 0x0216_D520;
/// `FUN_0219C5D0` (fldmmdl_blact): delete the actor system.
const BLACT_DELETE: u32 = 0x0219_C5D0;
/// `FUN_0216D528`: the MMDLSYS 3D-object system (`+0x28`) or null.
const SYS_G3DOBJ: u32 = 0x0216_D528;
/// `FUN_0219D488` (fldmmdl_g3dobj): delete the 3D-object system.
const G3DOBJ_DELETE: u32 = 0x0219_D488;
/// `FUN_0216D95C`: draw side on / off (sys, flag).
const SET_DRAW_ON: u32 = 0x0216_D95C;
/// `FUN_0216D67C`: MMDL -> MMDLSYS.
const SYS_OF: u32 = 0x0216_D67C;
/// `FUN_0216D4D4`: MMDLSYS flags & mask.
const SYS_FLAGS_TEST: u32 = 0x0216_D4D4;
/// `FUN_0216DAA0`: 1 when the MMDL's draw side is ready (move bit 2).
const DRAW_READY: u32 = 0x0216_DAA0;
/// `FUN_0216D758`: run the MMDL's draw proc (draw table `+4`).
const DRAW_PROC: u32 = 0x0216_D758;
/// `FUN_0216DB10`: 1 when the MMDL is paused (move bit 8).
const PAUSED: u32 = 0x0216_DB10;
/// `FUN_0216DC4C`: 1 when an acmd is set (move bit 0x200).
const ACMD_SET: u32 = 0x0216_DC4C;
/// `FUN_0216D55C`: status & mask.
const STATUS_TEST: u32 = 0x0216_D55C;
/// The five VecFx32 getters whose sum is the draw position: the position (`+0x44`), the VecFx32s at `+0x50` / `+0x5C` /
/// `+0x68`, and the s8 offsets `+0x7E..+0x80` << 12 (`FUN_0216D88C`, `FUN_0216D8B0`, `FUN_0216D8D0`, `FUN_0216D8F0`,
/// `FUN_0216D92C`), in call order.
const POS_PARTS: [u32; 5] = [
    0x0216_D88C,
    0x0216_D8B0,
    0x0216_D8D0,
    0x0216_D8F0,
    0x0216_D92C,
];

/// C: `FUN_02197C4C` (unlabeled routine at `0x02197C4C`): delete the draw systems of an MMDLSYS: the blact actor system
/// (`0219C5D0`) if `0216D520` is set, the 3D-object system (`0219D488`) if `0216D528` is set; then draw side off (`0216D95C(sys,
/// 0)`). The systems themselves are the draw side (D3), reached by address.
pub fn delete_draw<E: FieldEnv>(env: &mut E, sys: u32) {
    if env.callv(SYS_BLACT, &[sys]) != 0 {
        env.callv(BLACT_DELETE, &[sys]);
    }
    if env.callv(SYS_G3DOBJ, &[sys]) != 0 {
        env.callv(G3DOBJ_DELETE, &[sys]);
    }
    env.callv(SET_DRAW_ON, &[sys, 0]);
}

/// C: `FUN_02197C78` (unlabeled routine at `0x02197C78`): the per-frame draw of one MMDL: unless MMDLSYS flag 4 is set, and only
/// once the MMDL's draw side is ready (move bit 2), run its draw proc (`0216D758`, D3).
pub fn draw<E: FieldEnv>(env: &mut E, m: u32) {
    let sys = env.callv(SYS_OF, &[m]);
    if env.callv(SYS_FLAGS_TEST, &[sys, 4]) != 0 {
        return;
    }
    if env.callv(DRAW_READY, &[m]) != 0 {
        env.callv(DRAW_PROC, &[m]);
    }
}

/// C: `FUN_02197C9C` (unlabeled routine at `0x02197C9C`): 1 when the MMDL's animation is held: paused without an acmd set,
/// or status bit 0x10; else 0.
pub fn anim_held<E: FieldEnv>(env: &mut E, m: u32) -> u32 {
    if env.callv(PAUSED, &[m]) != 0 && env.callv(ACMD_SET, &[m]) == 0 {
        return 1;
    }
    u32::from(env.callv(STATUS_TEST, &[m, 0x10]) != 0)
}

/// C: `FUN_02197CCC` (unlabeled routine at `0x02197CCC`): `out` (VecFx32) = the sum of the five position parts, per
/// component (wrapping).
pub fn draw_pos<E: FieldEnv>(env: &mut E, m: u32, out: u32) {
    let mut parts = [[0u8; 12]; 5];
    for (addr, p) in POS_PARTS.iter().zip(parts.iter_mut()) {
        env.call(*addr, &mut [CArg::V(m), CArg::OutBuf(p)]);
    }
    for k in 0..3 {
        let s = parts.iter().fold(0u32, |acc, p| {
            let w = u32::from_le_bytes([p[4 * k], p[4 * k + 1], p[4 * k + 2], p[4 * k + 3]]);
            acc.wrapping_add(w)
        });
        env.w32(out.wrapping_add(4 * k as u32), s);
    }
}

#[cfg(test)]
mod tests;
