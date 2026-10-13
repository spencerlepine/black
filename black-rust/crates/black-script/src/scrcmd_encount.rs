//! F8 (fourth batch): overlay-21 `scrcmd_encount.c`, Thumb code 0x021B5190-0x021B5700 (F1-SPLIT row 21.48 plus its static
//! helper 0x021B5190): the encounter-table query 561 and the field effect object commands 319..328 / 418..420 (14 command ids,
//! 19 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_encount.spec`. The functions listed in
//! [`PORTED`] are ported here and checked with `tools/check-fn.sh black-script scrcmd_encount`; the rest are the F8 port job's.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x021B_5190, 0x021B_538C, 0x021B_55F0];

/// Callees of the scrcmd_encount functions that this area does not port, named by address (meaning per the spec).
pub trait EncountHooks: MemEnv {
    /// C: `FUN_0215A5DC` (script_sys `ctx_gamesys`).
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020124E4(event)` (main): the event's GAMESYS.
    fn fun_020124e4(&mut self, event: u32) -> u32;
    /// C: `FUN_02188C58(fm)` (overlay 21): the field effect / camera object E (UNVERIFIED role).
    fn fun_02188c58(&mut self, fm: u32) -> u32;
    /// C: `FUN_0218EC54(E)` (overlay 21; UNVERIFIED role).
    fn fun_0218ec54(&mut self, e: u32);
    /// C: `FUN_0218FE50(E)` (overlay 21): nonzero while E is still busy (UNVERIFIED).
    fn fun_0218fe50(&mut self, e: u32) -> u32;
}

/// C: `FUN_overlay_d_21__021b5190`: `list[0..n)` (u16) already holds `v` -> `n`; else append `v` -> `n + 1` (u16). The helper
/// of command 561 (unique ids).
pub fn add_unique<H: MemEnv>(h: &mut H, list: u32, n: u32, v: u32) -> u16 {
    let mut i: i32 = 0;
    while i < n as i32 {
        if v == u32::from(h.r16(list + 2 * i as u32)) {
            return n as u16;
        }
        i += 1;
    }
    h.w16(list.wrapping_add(n.wrapping_mul(2)), v as u16);
    n.wrapping_add(1) as u16
}

/// C: `FUN_overlay_d_21__021b538c`: command 321: `FUN_0218EC54(FUN_02188C58(field map))`; 0.
pub fn cmd_321<H: EncountHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gamesys = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gamesys);
    let e = h.fun_02188c58(fm);
    h.fun_0218ec54(e);
    0
}

/// C: `sub_021b55f0`: the GMEVENT function of command 325: done (1) once `FUN_0218FE50(E)` answers 0, else 0 (wait).
pub fn wait_325<H: EncountHooks>(h: &mut H, event: u32) -> u32 {
    let gamesys = h.fun_020124e4(event);
    let fm = h.fun_0201210c(gamesys);
    let e = h.fun_02188c58(fm);
    if h.fun_0218fe50(e) != 0 {
        0
    } else {
        1
    }
}

#[cfg(test)]
mod tests;
