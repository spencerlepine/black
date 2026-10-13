//! F8b: overlay-21 screen-effect script commands, Thumb code 0x021B2830-0x021B2FD8 (the tail of F1-SPLIT row 21.44 and row
//! 21.45 `scrcmd_screeneffects.c` with command 594 after it): the screen fade of fade.c (commands 179 / 180), the field fades
//! (429..440 and their wait 433), field 3D objects (292..302) and a few child events (29 command ids, 36 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_screen.spec` (header: command-id table,
//! roles, the fade.c state), ported here, listed in [`PORTED`] and checked with `tools/check-fn.sh black-script
//! scrcmd_screen`. [`COMMANDS`] maps the command ids to their handlers and [`dispatch`] / [`call_wait`] / [`call_task`] run a
//! handler / wait callback / task function by its address.
//!
//! Callees outside the area are [`ScreenHooks`] methods named by address (the vm.c / script_sys / scrcmd ctx ones come from
//! [`CoreHooks`]). The fade callees `FUN_0204E720` / `FUN_0204E804` are what [`crate::host::ScriptHost::fade_start`] /
//! [`crate::host::ScriptHost::fade_done`] model: a game-side implementation of [`ScreenHooks::fun_0204e720`] issues a
//! [`crate::host::FadeRequest`] `{mode, start, end, wait}` (the four values as passed, start / end / wait as signed 16-bit)
//! and [`ScreenHooks::fun_0204e804`] answers 1 while the host's fade is not done.

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_2830,
    0x021B_2860,
    0x021B_2874,
    0x021B_2890,
    0x021B_28E0,
    0x021B_2910,
    0x021B_2940,
    0x021B_2970,
    0x021B_29A0,
    0x021B_29D0,
    0x021B_2A00,
    0x021B_2A30,
    0x021B_2A60,
    0x021B_2A90,
    0x021B_2AA0,
    0x021B_2AB0,
    0x021B_2ADC,
    0x021B_2B24,
    0x021B_2B5C,
    0x021B_2B94,
    0x021B_2BD8,
    0x021B_2BEC,
    0x021B_2C08,
    0x021B_2C5C,
    0x021B_2CA0,
    0x021B_2CE8,
    0x021B_2D34,
    0x021B_2D98,
    0x021B_2DBC,
    0x021B_2E08,
    0x021B_2E3C,
    0x021B_2E60,
    0x021B_2E80,
    0x021B_2EBC,
    0x021B_2F18,
    0x021B_2FB0,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (179, 0x021B_2830),
    (180, 0x021B_2A90),
    (292, 0x021B_2E08),
    (293, 0x021B_2AB0),
    (294, 0x021B_2CE8),
    (295, 0x021B_2D34),
    (296, 0x021B_2D98),
    (297, 0x021B_2DBC),
    (298, 0x021B_2E60),
    (299, 0x021B_2C08),
    (300, 0x021B_2E3C),
    (301, 0x021B_2CA0),
    (302, 0x021B_2C5C),
    (303, 0x021B_2ADC),
    (304, 0x021B_2B24),
    (305, 0x021B_2B5C),
    (306, 0x021B_2B94),
    (429, 0x021B_2910),
    (430, 0x021B_2940),
    (431, 0x021B_2970),
    (432, 0x021B_29A0),
    (433, 0x021B_2AA0),
    (434, 0x021B_28E0),
    (436, 0x021B_2890),
    (437, 0x021B_29D0),
    (438, 0x021B_2A00),
    (439, 0x021B_2A30),
    (440, 0x021B_2A60),
    (594, 0x021B_2FB0),
];

/// u16 overlay-21 BSS word: the 3D-object id command 298 waits for.
pub const WAIT_OBJ_ID: u32 = 0x021E_E720;
/// Wait callback of command 180 ([`wait_fade`], Thumb).
pub const WAIT_FADE: u32 = 0x021B_2861;
/// Wait callback of command 433 ([`wait_field_fade`], Thumb).
pub const WAIT_FIELD_FADE: u32 = 0x021B_2875;
/// Wait callback of command 298 ([`wait_298`], Thumb).
pub const WAIT_298: u32 = 0x021B_2E81;
/// Task function of the field fade ([`field_fade_task`], Thumb).
pub const FIELD_FADE_TASK: u32 = 0x021B_2F19;
/// fade.c mode of the field fade to / from black (both screens).
pub const FADE_MODE_BLACK: u32 = 3;
/// fade.c mode of the field fade to / from white (both screens).
pub const FADE_MODE_WHITE: u32 = 0xC;
/// Brightness magnitude of a full fade (fade.c works in -16..16).
pub const FADE_FULL: u32 = 0x10;
/// Heap of the field-fade work.
pub const FIELD_FADE_HEAP: u32 = 4;
/// Size of the field-fade work.
pub const FIELD_FADE_SIZE: u32 = 0x18;
/// The source-file string the field-fade allocation passes to `Heap_AllocDebug` (overlay-21 rodata).
pub const FILE_SCREENEFFECTS: u32 = 0x021D_E14C;
/// The source line the field-fade allocation passes.
pub const FIELD_FADE_LINE: u32 = 0x369;

/// FIELD FADE work (0x18 B, [`field_fade_start`]): +0 GAMESYS, +4 field map, +8 color (0 black), +0xC no_wait, +0x10 kind
/// (0 / 2 in, 1 / 3 out; 0 / 1 fast), +0x14 seq.
pub const FF_GAMESYS: u32 = 0x00;
pub const FF_FIELD_MAP: u32 = 0x04;
pub const FF_COLOR: u32 = 0x08;
pub const FF_NO_WAIT: u32 = 0x0C;
pub const FF_KIND: u32 = 0x10;
pub const FF_SEQ: u32 = 0x14;

/// Callees of the scrcmd_screen functions outside the area, named by address (meaning per the spec; names of main /
/// overlay-21 routines are inferred, UNVERIFIED unless stated). vm.c, the script_sys operand readers, the scrcmd ctx getters
/// and the child-event call come from [`CoreHooks`].
#[allow(clippy::too_many_arguments)]
pub trait ScreenHooks: CoreHooks {
    /// C: `FUN_02158F50` (`script_sys::sw_event`): the script's GMEVENT.
    fn fun_02158f50(&mut self, sw: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::sw_event(self, sw)
    }
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012984(gamedata)` (main; command 436 uses its low byte; UNVERIFIED meaning).
    fn fun_02012984(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02030734(heap, size, flag, file, line)` (`Heap_AllocDebug`).
    fn fun_02030734(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32;
    /// C: `FUN_020307B0(block)` (`Heap_Free`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_02030DA8(taskman, f, work, prio)` (main): add a task running `f(task, work)` every frame.
    fn fun_02030da8(&mut self, taskman: u32, f: u32, work: u32, prio: u32);
    /// C: `FUN_02030EAC(task)` (main): remove a task.
    fn fun_02030eac(&mut self, task: u32);
    /// C: `FUN_020061E4(se)` (main): play a sound effect (`ScriptHost::play_se`).
    fn fun_020061e4(&mut self, se: u32);
    /// C: `FUN_0204E720(mode, start, end, wait)` (fade.c): start a master-brightness fade (`ScriptHost::fade_start`).
    fn fun_0204e720(&mut self, mode: u32, start: u32, end: u32, wait: u32);
    /// C: `FUN_0204E804()` (fade.c): 1 while a fade runs (`ScriptHost::fade_done` is "not 1").
    fn fun_0204e804(&mut self) -> u32;
    /// C: `FUN_02043658(x)` (main; UNVERIFIED: a display / BG getter).
    fn fun_02043658(&mut self, x: u32) -> u32;
    /// C: `FUN_02043534(x)` (main; UNVERIFIED).
    fn fun_02043534(&mut self, x: u32);
    /// C: `FUN_02188C98(field map)` (overlay 21): the field's 3D-object owner (UNVERIFIED).
    fn fun_02188c98(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C9C(field map)` (overlay 21; command 594).
    fn fun_02188c9c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CB4(field map)` (overlay 21: the player object; command 303 discards it).
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D20(field map)` (overlay 21; command 594).
    fn fun_02188d20(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D30(field map)` (overlay 21): the field's task manager.
    fn fun_02188d30(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188DA0(x)` (overlay 21; UNVERIFIED).
    fn fun_02188da0(&mut self, x: u32);
    /// C: `FUN_02188DB0(field map)` (overlay 21; UNVERIFIED: re-enable the field display).
    fn fun_02188db0(&mut self, fm: u32);
    /// C: `FUN_02189A1C(field map, on)` (overlay 21): the "field fade running" flag.
    fn fun_02189a1c(&mut self, fm: u32, on: u32);
    /// C: `FUN_02189A24(field map)` (overlay 21): that flag (1 = running).
    fn fun_02189a24(&mut self, fm: u32) -> u32;
    /// C: `FUN_0218DC6C(x)` (overlay 21): the 3D-object system of [`ScreenHooks::fun_02188c98`]'s owner (UNVERIFIED).
    fn fun_0218dc6c(&mut self, x: u32) -> u32;
    /// C: `FUN_0218BA98(sys, id)` (overlay 21): the 3D object with id `id`, 0 if none.
    fn fun_0218ba98(&mut self, sys: u32, id: u32) -> u32;
    /// C: `FUN_0218BACC(sys, o)` (overlay 21; command 295 stores its u16 result).
    fn fun_0218bacc(&mut self, sys: u32, o: u32) -> u32;
    /// C: `FUN_0218C54C(o, v)` (overlay 21; command 301).
    fn fun_0218c54c(&mut self, o: u32, v: u32);
    /// C: `FUN_0218C574(sys, o, v)` (overlay 21; command 294).
    fn fun_0218c574(&mut self, sys: u32, o: u32, v: u32);
    /// C: `FUN_0218C654(sys, id, &pos)` (overlay 21): the 3D object of `id` at the grid position `pos` (3 fx32 words), 0 if
    /// none.
    fn fun_0218c654(&mut self, sys: u32, id: u32, pos: &[u8; 12]) -> u32;
    /// C: `FUN_0218C69C(sys, o, a, b)` (overlay 21; commands 299 / 302).
    fn fun_0218c69c(&mut self, sys: u32, o: u32, a: u32, b: u32);
    /// C: `FUN_0218C7D8(sys, id, &pos)` (overlay 21; command 295).
    fn fun_0218c7d8(&mut self, sys: u32, id: u32, pos: &[u8; 12]) -> u32;
    /// C: `FUN_0218C860(o)` (overlay 21; command 296).
    fn fun_0218c860(&mut self, o: u32);
    /// C: `FUN_0218C88C(o, a, mode)` (overlay 21; commands 292 / 297).
    fn fun_0218c88c(&mut self, o: u32, a: u32, mode: u32);
    /// C: `FUN_0218C89C(o, v)` (overlay 21; command 300).
    fn fun_0218c89c(&mut self, o: u32, v: u32);
    /// C: `FUN_0218C8D8(o)` (overlay 21; the wait of command 298).
    fn fun_0218c8d8(&mut self, o: u32) -> u32;
    /// C: `FUN_0218C990(o, &se)` (overlay 21): nonzero = `se` (u16) holds a sound effect to play (command 297).
    fn fun_0218c990(&mut self, o: u32, se: &mut [u8; 4]) -> u32;
    /// C: `FUN_0218C9A4(o)` (overlay 21; the wait of command 298 ends when it is 0).
    fn fun_0218c9a4(&mut self, o: u32) -> u32;
    /// C: `FUN_021BC5A0(a, b)` (overlay 21; command 594).
    fn fun_021bc5a0(&mut self, a: u32, b: u32);
    /// C: `FUN_021BE92C(gamesys, field map, a, b, c, d, e)` (overlay 21): the child event of command 436.
    fn fun_021be92c(
        &mut self,
        gamesys: u32,
        fm: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
    ) -> u32;
    /// C: `FUN_021C0AA0(gamesys, event)` (overlay 21): the child event of command 293.
    fn fun_021c0aa0(&mut self, gamesys: u32, event: u32) -> u32;
    /// C: `FUN_021C0820(gamesys, event, v)` (overlay 21): the child event of command 303.
    fn fun_021c0820(&mut self, gamesys: u32, event: u32, v: u32) -> u32;
    /// C: `FUN_021C978C(event, gamesys, field map)` (overlay 21): the child event of command 304.
    fn fun_021c978c(&mut self, event: u32, gamesys: u32, fm: u32) -> u32;
    /// C: `FUN_021C9828(event, gamesys, field map)` (overlay 21): the child event of command 305.
    fn fun_021c9828(&mut self, event: u32, gamesys: u32, fm: u32) -> u32;
    /// C: `FUN_021C98F4(event, gamesys, field map, v)` (overlay 21): the child event of command 306.
    fn fun_021c98f4(&mut self, event: u32, gamesys: u32, fm: u32, v: u32) -> u32;
}

// ---- fades

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

/// C: `FUN_overlay_d_21__021b2874`: the wait callback of command 433: done (1) once the field map's fade flag (`FUN_02189A24`) is not 1.
pub fn wait_field_fade<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    (h.fun_02189a24(fm) != 1) as u32
}

/// C: `sub_021b2890`: command 436: the child event `FUN_021BE92C(GAMESYS, field map, 3, 0, 0, z, z)` with
/// `z = FUN_02012984(GAMEDATA) & 0xFF`; 0.
pub fn cmd_436<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let fm = h.fun_0201210c(gs);
    let z = h.fun_02012984(gd) & 0xFF;
    let ev = h.fun_021be92c(gs, fm, 3, 0, 0, z, z);
    h.fun_0215897c(sw, ev);
    0
}

/// Body shared by the field-fade commands 429..432 / 434 / 437..440 (not a function of the original: each command has its
/// own copy): the ctx's SCRIPT_WORK is fetched (unused), then [`field_fade_start`]`(GAMESYS, field map, kind, color, 0)`; 0.
fn field_fade_cmd<H: ScreenHooks>(h: &mut H, ctx: u32, kind: u32, color: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    field_fade_start(h, gs, fm, kind, color, 0);
    0
}

/// C: `sub_021b28e0`: command 434: the field fade in from black (kind 0, color 0; the same as command 429).
pub fn cmd_434<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    field_fade_cmd(h, ctx, 0, 0)
}

/// C: `FUN_overlay_d_21__021b2910`: command 429: the field fade in from black (kind 0, color 0).
pub fn cmd_field_fade_429<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    field_fade_cmd(h, ctx, 0, 0)
}

/// C: `FUN_overlay_d_21__021b2940`: command 430: the field fade out to black (kind 1, color 0).
pub fn cmd_430<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    field_fade_cmd(h, ctx, 1, 0)
}

/// C: `FUN_overlay_d_21__021b2970`: command 431: the field fade in from white (kind 0, color 1).
pub fn cmd_431<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    field_fade_cmd(h, ctx, 0, 1)
}

/// C: `FUN_overlay_d_21__021b29a0`: command 432: the field fade out to white (kind 1, color 1).
pub fn cmd_432<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    field_fade_cmd(h, ctx, 1, 1)
}

/// C: `FUN_overlay_d_21__021b29d0`: command 437: the slow field fade in from black (kind 2, color 0).
pub fn cmd_437<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    field_fade_cmd(h, ctx, 2, 0)
}

/// C: `FUN_overlay_d_21__021b2a00`: command 438: the slow field fade out to black (kind 3, color 0).
pub fn cmd_438<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    field_fade_cmd(h, ctx, 3, 0)
}

/// C: `FUN_overlay_d_21__021b2a30`: command 439: the slow field fade in from white (kind 2, color 1).
pub fn cmd_439<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    field_fade_cmd(h, ctx, 2, 1)
}

/// C: `FUN_overlay_d_21__021b2a60`: command 440: the slow field fade out to white (kind 3, color 1).
pub fn cmd_440<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    field_fade_cmd(h, ctx, 3, 1)
}

/// C: `sub_021b2a90`: command 180: wait on [`WAIT_FADE`]; yield.
pub fn cmd_wait_fade<H: ScreenHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113d0(vm, WAIT_FADE);
    1
}

/// C: `sub_021b2aa0`: command 433: wait on [`WAIT_FIELD_FADE`]; yield.
pub fn cmd_wait_field_fade<H: ScreenHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113d0(vm, WAIT_FIELD_FADE);
    1
}

// ---- child events

/// C: `sub_021b2ab0`: command 293: the child event `FUN_021C0AA0(GAMESYS, the script's GMEVENT)`; yield.
pub fn cmd_293<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let ev = h.fun_02158f50(sw);
    let child = h.fun_021c0aa0(gs, ev);
    h.fun_0215897c(sw, child);
    1
}

/// C: `sub_021b2adc`: command 303: `v` = value; `FUN_02188CB4(field map)` (result unused); the child event
/// `FUN_021C0820(GAMESYS, GMEVENT, v & 0xFF)`; yield.
pub fn cmd_303<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let fm = h.fun_0201210c(gs);
    h.fun_02188cb4(fm);
    let ev = h.fun_02158f50(sw);
    let child = h.fun_021c0820(gs, ev, v & 0xFF);
    h.fun_0215897c(sw, child);
    1
}

/// C: `sub_021b2b24`: command 304: the child event `FUN_021C978C(GMEVENT, GAMESYS, field map)`; yield.
pub fn cmd_304<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let fm = h.fun_0201210c(gs);
    let ev = h.fun_02158f50(sw);
    let child = h.fun_021c978c(ev, gs, fm);
    h.fun_0215897c(sw, child);
    1
}

/// C: `FUN_overlay_d_21__021b2b5c`: command 305: the child event `FUN_021C9828(GMEVENT, GAMESYS, field map)`; yield.
pub fn cmd_305<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let fm = h.fun_0201210c(gs);
    let ev = h.fun_02158f50(sw);
    let child = h.fun_021c9828(ev, gs, fm);
    h.fun_0215897c(sw, child);
    1
}

/// C: `FUN_overlay_d_21__021b2b94`: command 306: the child event `FUN_021C98F4(GMEVENT, GAMESYS, field map, value)`; yield.
pub fn cmd_306<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let fm = h.fun_0201210c(gs);
    let v = h.fun_02159b10(vm, ctx);
    let ev = h.fun_02158f50(sw);
    let child = h.fun_021c98f4(ev, gs, fm, v);
    h.fun_0215897c(sw, child);
    1
}

// ---- field 3D objects

/// C: `FUN_overlay_d_21__021b2bd8`: the field's 3D-object system: `FUN_0218DC6C(FUN_02188C98(field map))`.
pub fn obj_system<H: ScreenHooks>(h: &mut H, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let x = h.fun_02188c98(fm);
    h.fun_0218dc6c(x)
}

/// The fx32 position record `{x << 16, 0, z << 16}` (a 12-byte stack record in the original) of a grid cell.
fn grid_pos(x: u32, z: u32) -> [u8; 12] {
    let mut p = [0u8; 12];
    p[0..4].copy_from_slice(&(x << 16).to_le_bytes());
    p[8..12].copy_from_slice(&(z << 16).to_le_bytes());
    p
}

/// C: `FUN_overlay_d_21__021b2bec`: the 3D object `id` at grid cell (`x`, `z`): `FUN_0218C654(sys, id, &{x << 16, 0, z << 16})`.
pub fn obj_at<H: ScreenHooks>(h: &mut H, sys: u32, id: u32, x: u32, z: u32) -> u32 {
    h.fun_0218c654(sys, id, &grid_pos(x, z))
}

/// C: `FUN_overlay_d_21__021b2c08`: command 299: id, x, z = values; the object at the cell (if any) gets `FUN_0218C69C(sys, o, 0, 1)` and
/// `FUN_0218C69C(sys, o, 0, 3)`; 0.
pub fn cmd_299<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let x = h.fun_02159b10(vm, ctx);
    let z = h.fun_02159b10(vm, ctx);
    let sys = obj_system(h, ctx);
    let o = obj_at(h, sys, id, x, z);
    if o != 0 {
        h.fun_0218c69c(sys, o, 0, 1);
        h.fun_0218c69c(sys, o, 0, 3);
    }
    0
}

/// C: `FUN_overlay_d_21__021b2c5c`: command 302: id, x, z = values; the object at the cell (if any) gets `FUN_0218C69C(sys, o, 0, 2)`; 0.
pub fn cmd_302<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let x = h.fun_02159b10(vm, ctx);
    let z = h.fun_02159b10(vm, ctx);
    let sys = obj_system(h, ctx);
    let o = obj_at(h, sys, id, x, z);
    if o != 0 {
        h.fun_0218c69c(sys, o, 0, 2);
    }
    0
}

/// C: `sub_021b2ca0`: command 301: id, x, z, v = values; the object at the cell (if any) gets `FUN_0218C54C(o, v)`; 0.
pub fn cmd_301<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let x = h.fun_02159b10(vm, ctx);
    let z = h.fun_02159b10(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let sys = obj_system(h, ctx);
    let o = obj_at(h, sys, id, x, z);
    if o != 0 {
        h.fun_0218c54c(o, v);
    }
    0
}

/// C: `sub_021b2ce8`: command 294: id, x, z, v = values; the object at the cell (if any) gets `FUN_0218C574(sys, o, v)`; 0.
pub fn cmd_294<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let x = h.fun_02159b10(vm, ctx);
    let z = h.fun_02159b10(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let sys = obj_system(h, ctx);
    let o = obj_at(h, sys, id, x, z);
    if o != 0 {
        h.fun_0218c574(sys, o, v);
    }
    0
}

/// C: `sub_021b2d34`: command 295: the result var; id, x, z = values; var (u16) =
/// `FUN_0218BACC(sys, FUN_0218C7D8(sys, id, &{x << 16, 0, z << 16}))` (the system fetched inline, as [`obj_system`]); 0.
pub fn cmd_295<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let id = h.fun_02159b10(vm, ctx);
    let x = h.fun_02159b10(vm, ctx);
    let z = h.fun_02159b10(vm, ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let owner = h.fun_02188c98(fm);
    let sys = h.fun_0218dc6c(owner);
    let o = h.fun_0218c7d8(sys, id, &grid_pos(x, z));
    let r = h.fun_0218bacc(sys, o);
    h.w16(var, r as u16);
    0
}

/// C: `sub_021b2d98`: command 296: the object with id = value (if any) gets `FUN_0218C860(o)`; 0.
pub fn cmd_296<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let sys = obj_system(h, ctx);
    let o = h.fun_0218ba98(sys, id);
    if o != 0 {
        h.fun_0218c860(o);
    }
    0
}

/// C: `FUN_overlay_d_21__021b2dbc`: command 297: id, a = values; the object with that id (if any) gets `FUN_0218C88C(o, a, 0)`, then when
/// `FUN_0218C990(o, &se)` is nonzero the sound effect `se` (u16) plays; 0.
pub fn cmd_297<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159b10(vm, ctx);
    let sys = obj_system(h, ctx);
    let o = h.fun_0218ba98(sys, id);
    if o != 0 {
        h.fun_0218c88c(o, a, 0);
        let mut se = [0u8; 4];
        if h.fun_0218c990(o, &mut se) != 0 {
            h.fun_020061e4(u16::from_le_bytes([se[0], se[1]]) as u32);
        }
    }
    0
}

/// C: `FUN_overlay_d_21__021b2e08`: command 292: id, a = values; the object with that id (if any) gets `FUN_0218C88C(o, a, 2)`; 0.
pub fn cmd_292<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159b10(vm, ctx);
    let sys = obj_system(h, ctx);
    let o = h.fun_0218ba98(sys, id);
    if o != 0 {
        h.fun_0218c88c(o, a, 2);
    }
    0
}

/// C: `FUN_overlay_d_21__021b2e3c`: command 300: the object with id = value (if any) gets `FUN_0218C89C(o, 3)`; 0.
pub fn cmd_300<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let sys = obj_system(h, ctx);
    let o = h.fun_0218ba98(sys, id);
    if o != 0 {
        h.fun_0218c89c(o, 3);
    }
    0
}

/// C: `sub_021b2e60`: command 298: the u16 at [`WAIT_OBJ_ID`] = value; wait on [`WAIT_298`]; yield.
pub fn cmd_298<H: ScreenHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    h.w16(WAIT_OBJ_ID, v as u16);
    h.fun_020113d0(vm, WAIT_298);
    1
}

/// C: `sub_021b2e80`: the wait callback of command 298: the object with the id at [`WAIT_OBJ_ID`]; none -> done (1);
/// `FUN_0218C8D8(o) == 0` -> 0 (wait); else done once `FUN_0218C9A4(o) == 0`.
pub fn wait_298<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sys = obj_system(h, ctx);
    let id = h.r16(WAIT_OBJ_ID) as u32;
    let o = h.fun_0218ba98(sys, id);
    if o == 0 {
        return 1;
    }
    if h.fun_0218c8d8(o) == 0 {
        return 0;
    }
    (h.fun_0218c9a4(o) == 0) as u32
}

// ---- the field fade task

/// C: `FUN_overlay_d_21__021b2ebc`: start the field fade task (`gamesys`, `fm` = field map, `kind`, `color`, `no_wait`; the fifth argument is
/// a stack word): its work W = `Heap_AllocDebug(4, 0x18, 1, "..", 0x369)` becomes the task [`FIELD_FADE_TASK`] of the field's
/// task manager (`FUN_02030DA8(FUN_02188D30(fm), task, W, 0)`), the field map's fade flag is set (`FUN_02189A1C(fm, 1)`), and W =
/// `{gamesys, fm, color, no_wait, kind, seq 0}`.
pub fn field_fade_start<H: ScreenHooks>(
    h: &mut H,
    gamesys: u32,
    fm: u32,
    kind: u32,
    color: u32,
    no_wait: u32,
) {
    let taskman = h.fun_02188d30(fm);
    let w = h.fun_02030734(
        FIELD_FADE_HEAP,
        FIELD_FADE_SIZE,
        1,
        FILE_SCREENEFFECTS,
        FIELD_FADE_LINE,
    );
    h.fun_02030da8(taskman, FIELD_FADE_TASK, w, 0);
    h.fun_02189a1c(fm, 1);
    h.w32(w + FF_SEQ, 0);
    h.w32(w + FF_GAMESYS, gamesys);
    h.w32(w + FF_FIELD_MAP, fm);
    h.w32(w + FF_COLOR, color);
    h.w32(w + FF_NO_WAIT, no_wait);
    h.w32(w + FF_KIND, kind);
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

/// C: unlabeled routine at `0x021B2FB0`: command 594: `FUN_021BC5A0(FUN_02188D20(field map), FUN_02188C9C(field map))`; 0.
pub fn cmd_594<H: ScreenHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let a = h.fun_02188d20(fm);
    let b = h.fun_02188c9c(fm);
    h.fun_021bc5a0(a, b);
    0
}

// ---- dispatch

/// Run the command handler at `handler` (the command table word, Thumb bit allowed) when it belongs to this area; `None`
/// otherwise.
pub fn dispatch<H: ScreenHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    let f: fn(&mut H, u32, u32) -> u32 = match handler & !1 {
        0x021B_2830 => cmd_fade,
        0x021B_2A90 => cmd_wait_fade,
        0x021B_2E08 => cmd_292,
        0x021B_2AB0 => cmd_293,
        0x021B_2CE8 => cmd_294,
        0x021B_2D34 => cmd_295,
        0x021B_2D98 => cmd_296,
        0x021B_2DBC => cmd_297,
        0x021B_2E60 => cmd_298,
        0x021B_2C08 => cmd_299,
        0x021B_2E3C => cmd_300,
        0x021B_2CA0 => cmd_301,
        0x021B_2C5C => cmd_302,
        0x021B_2ADC => cmd_303,
        0x021B_2B24 => cmd_304,
        0x021B_2B5C => cmd_305,
        0x021B_2B94 => cmd_306,
        0x021B_2910 => cmd_field_fade_429,
        0x021B_2940 => cmd_430,
        0x021B_2970 => cmd_431,
        0x021B_29A0 => cmd_432,
        0x021B_2AA0 => cmd_wait_field_fade,
        0x021B_28E0 => cmd_434,
        0x021B_2890 => cmd_436,
        0x021B_29D0 => cmd_437,
        0x021B_2A00 => cmd_438,
        0x021B_2A30 => cmd_439,
        0x021B_2A60 => cmd_440,
        0x021B_2FB0 => cmd_594,
        _ => return None,
    };
    Some(f(h, vm, ctx))
}

/// Run the wait callback `f` (VM +0x10, Thumb bit set) when it is one of this area's ([`WAIT_FADE`], [`WAIT_FIELD_FADE`],
/// [`WAIT_298`]); `None` otherwise.
pub fn call_wait<H: ScreenHooks>(h: &mut H, f: u32, vm: u32, ctx: u32) -> Option<u32> {
    match f {
        WAIT_FADE => Some(wait_fade(h, vm, ctx)),
        WAIT_FIELD_FADE => Some(wait_field_fade(h, vm, ctx)),
        WAIT_298 => Some(wait_298(h, vm, ctx)),
        _ => None,
    }
}

/// Run the task function `f(task, work)` when it is this area's ([`FIELD_FADE_TASK`]); false otherwise.
pub fn call_task<H: ScreenHooks>(h: &mut H, f: u32, task: u32, work: u32) -> bool {
    if f == FIELD_FADE_TASK {
        field_fade_task(h, task, work);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests;
