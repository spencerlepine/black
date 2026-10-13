//! F8 (second batch): overlay-21 screen-effect script commands, Thumb code 0x021B2830-0x021B2FD8 (the tail of F1-SPLIT row 21.44
//! and row 21.45 `scrcmd_screeneffects.c` with command 594 after it): the screen fade of fade.c (commands 179 / 180), the field
//! fades (429..440 and their wait 433), field 3D objects (292..302) and a few child events (29 command ids, 36 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_screen.spec` (header: command-id table,
//! roles, the fade.c state). The functions listed in [`PORTED`] are ported here and checked with `tools/check-fn.sh black-script
//! scrcmd_screen`; the rest are the F8 port job's. The fade callees (`FUN_0204E720` / `FUN_0204E804`) are what
//! [`crate::host::ScriptHost::fade_start`] / [`crate::host::ScriptHost::fade_done`] model.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_2830,
    0x021B_2860,
    0x021B_2910,
    0x021B_2E60,
    0x021B_2F18,
];

/// u16 overlay-21 BSS word: the 3D-object id command 298 waits for.
pub const WAIT_OBJ_ID: u32 = 0x021E_E720;
/// Wait callback of command 298 (`0x021B2E80`, Thumb).
pub const WAIT_298: u32 = 0x021B_2E81;
/// fade.c mode of the field fade to / from black (both screens).
pub const FADE_MODE_BLACK: u32 = 3;
/// fade.c mode of the field fade to / from white (both screens).
pub const FADE_MODE_WHITE: u32 = 0xC;
/// Brightness magnitude of a full fade (fade.c works in -16..16).
pub const FADE_FULL: u32 = 0x10;

/// FIELD FADE work (0x18 B, `0x021B2EBC`): +0 GAMESYS, +4 field map, +8 color (0 black), +0xC no_wait, +0x10 kind (0 / 2 in,
/// 1 / 3 out; 0 / 1 fast), +0x14 seq.
pub const FF_FIELD_MAP: u32 = 0x04;
pub const FF_COLOR: u32 = 0x08;
pub const FF_NO_WAIT: u32 = 0x0C;
pub const FF_KIND: u32 = 0x10;
pub const FF_SEQ: u32 = 0x14;

/// Callees of the scrcmd_screen functions that this area does not port, named by address (meaning per the spec).
pub trait ScreenHooks: MemEnv {
    /// C: `FUN_02011330` (vm.c `read_u16`): the next u16 operand.
    fn fun_02011330(&mut self, vm: u32) -> u32;
    /// C: `FUN_020113D0` (vm.c `wait`): install a wait callback.
    fn fun_020113d0(&mut self, vm: u32, cb: u32);
    /// C: `FUN_02159B10` (script_sys `cmd_var_value`).
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_0215A5DC` (script_sys `ctx_gamesys`).
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A5F4` (script_sys `ctx_sw`).
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_0204E720(mode, start, end, wait)` (fade.c): start a master-brightness fade (ScriptHost::fade_start).
    fn fun_0204e720(&mut self, mode: u32, start: u32, end: u32, wait: u32);
    /// C: `FUN_0204E804()` (fade.c): 1 while a fade runs (ScriptHost::fade_done is its negation).
    fn fun_0204e804(&mut self) -> u32;
    /// C: `FUN_021B2EBC(gamesys, field map, kind, color, no_wait)` (this area, unported): start the field fade task.
    fn fun_021b2ebc(&mut self, gamesys: u32, fm: u32, kind: u32, color: u32, no_wait: u32);
    /// C: `FUN_02043658(x)` (main; UNVERIFIED: a display / BG getter).
    fn fun_02043658(&mut self, x: u32) -> u32;
    /// C: `FUN_02043534(x)` (main; UNVERIFIED).
    fn fun_02043534(&mut self, x: u32);
    /// C: `FUN_02188DA0(x)` (overlay 21; UNVERIFIED).
    fn fun_02188da0(&mut self, x: u32);
    /// C: `FUN_02188DB0(field map)` (overlay 21; UNVERIFIED: re-enable the field display).
    fn fun_02188db0(&mut self, fm: u32);
    /// C: `FUN_02189A1C(field map, on)` (overlay 21): the "field fade running" flag (read back by `FUN_02189A24`).
    fn fun_02189a1c(&mut self, fm: u32, on: u32);
    /// C: `FUN_02030EAC(task)` (main): remove a task.
    fn fun_02030eac(&mut self, task: u32);
    /// C: `FUN_020307B0(block)` (`Heap_Free`).
    fn fun_020307b0(&mut self, block: u32);
}

/// C: `sub_021b2830`: command 179: `FUN_0204E720(mode, start, end, wait)` with four u16 operands (a screen fade; the script
/// waits for it with command 180).
pub fn cmd_fade<H: ScreenHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let mode = h.fun_02011330(vm);
    let start = h.fun_02011330(vm);
    let end = h.fun_02011330(vm);
    let wait = h.fun_02011330(vm);
    h.fun_0204e720(mode, start, end, wait);
    0
}

/// C: `sub_021b2860`: the wait callback of command 180: done (1) once `FUN_0204E804()` is not 1 (no fade running).
pub fn wait_fade<H: ScreenHooks>(h: &mut H, _vm: u32, _ctx: u32) -> u32 {
    (h.fun_0204e804() != 1) as u32
}

/// C: `FUN_overlay_d_21__021b2910`: command 429: the field fade in from black (`FUN_021B2EBC(GAMESYS, field map, 0, 0, 0)`).
pub fn cmd_field_fade_429<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    h.fun_021b2ebc(gs, fm, 0, 0, 0);
    0
}

/// C: `sub_021b2e60`: command 298: the u16 at [`WAIT_OBJ_ID`] = value; wait on [`WAIT_298`]; yield.
pub fn cmd_298<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    h.w16(WAIT_OBJ_ID, v as u16);
    h.fun_020113d0(vm, WAIT_298);
    1
}

/// C: `sub_021b2f18`: the task function of the field fade (`task`, `w` = the FIELD FADE work). Sequence 0: the brightness fade
/// starts (in: from full to 0 after the display calls; out: from 0 to full; kinds 0 / 1 step every frame (wait -1), 2 / 3 every
/// 4 frames; black or white by the color) and the sequence advances. Sequence 1: once the fade is over (or at once with
/// `no_wait == 1`) the field-map flag is cleared and the task and its work are freed. Other sequences: nothing.
pub fn field_fade_task<H: ScreenHooks>(h: &mut H, task: u32, w: u32) {
    match h.r32(w + FF_SEQ) {
        0 => {
            let kind = h.r32(w + FF_KIND);
            let wait = if kind <= 1 { u32::MAX } else { 4 };
            let (start, end) = if kind == 0 || kind == 2 {
                let x = h.fun_02043658(FADE_FULL);
                h.fun_02188da0(x);
                h.fun_02043534(x);
                let fm = h.r32(w + FF_FIELD_MAP);
                h.fun_02188db0(fm);
                (FADE_FULL, 0)
            } else {
                (0, FADE_FULL)
            };
            let mode = if h.r32(w + FF_COLOR) == 0 {
                FADE_MODE_BLACK
            } else {
                FADE_MODE_WHITE
            };
            h.fun_0204e720(mode, start, end, wait);
            let s = h.r32(w + FF_SEQ).wrapping_add(1);
            h.w32(w + FF_SEQ, s);
        }
        1 => {
            let no_wait = h.r32(w + FF_NO_WAIT) == 1;
            let fading = h.fun_0204e804() != 0;
            if no_wait || !fading {
                let fm = h.r32(w + FF_FIELD_MAP);
                h.fun_02189a1c(fm, 0);
                h.fun_02030eac(task);
                h.fun_020307b0(w);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests;
