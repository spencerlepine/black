//! The GMEVENT at the end of F1's `sodateya.c` row (overlay 21, `0x021C46F8-0x021C47AE`; F13). Not day care
//! code: two constructors and the proc of an event that loads overlay 0x3F (63) around a task that overlay runs on
//! a field OBJ (UNVERIFIED what the task is). Callers: `fieldmap_event::push_down_event` (`FUN_0218B24C`) uses
//! [`player_event_new`], script command 122 uses [`obj_event_new`].
//!
//! Every callee is reached through [`FieldEnv::call`] by address; the event work (one word, the overlay-63 task)
//! is written through the [`Mem`](crate::hooks::Mem) window. Verified by
//! `tools/check-fn.sh black-field ov63_event --per-function` (spec `tools/rom-diff/specs/black-field/ov63_event.spec`).

use crate::hooks::FieldEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x021C_46F8, 0x021C_474C, 0x021C_4788];

/// The event proc [`proc_`] (`FUN_021C4788`, Thumb address).
pub const PROC: u32 = 0x021C_4789;
/// The overlay the event keeps loaded.
pub const OVERLAY: u32 = 0x3F;

/// C: `FUN_020122C0(gsys, parent, proc, work size)`: create a GMEVENT.
const EVENT_NEW: u32 = 0x0201_22C0;
/// C: `FUN_020124E8(event)`: the work of a GMEVENT.
const EVENT_WORK: u32 = 0x0201_24E8;
/// C: `FUN_02034AC4(id)` / `FUN_02034A5C(id)`: load / unload an overlay.
const OVERLAY_LOAD: u32 = 0x0203_4AC4;
const OVERLAY_UNLOAD: u32 = 0x0203_4A5C;
/// C: `FUN_02188CB4(fm)`: the player.
const PLAYER: u32 = 0x0218_8CB4;
/// C: `FUN_021A3074(player)`: the player's OBJ.
const PLAYER_OBJ: u32 = 0x021A_3074;
/// C: `FUN_021A305C(player, x)` (UNVERIFIED; result unused).
const PLAYER_305C: u32 = 0x021A_305C;
/// C: overlay 63 `FUN_021F36C0(gs, fm, obj, flag)`: start the task; `FUN_021F376C(task)`: done;
/// `FUN_021F3758(task)`: free it.
const TASK_START: u32 = 0x021F_36C0;
const TASK_DONE: u32 = 0x021F_376C;
const TASK_FREE: u32 = 0x021F_3758;

/// C: `FUN_021C46F8(gs, fm, x)`: the event for the player's OBJ (after `FUN_021A305C(player, x)`), flag 1.
pub fn player_event_new<E: FieldEnv>(env: &mut E, gs: u32, fm: u32, x: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gs, 0, PROC, 4]);
    let work = env.callv(EVENT_WORK, &[ev]);
    env.callv(OVERLAY_LOAD, &[OVERLAY]);
    let player = env.callv(PLAYER, &[fm]);
    let obj = env.callv(PLAYER_OBJ, &[player]);
    env.callv(PLAYER_305C, &[player, x]);
    let task = env.callv(TASK_START, &[gs, fm, obj, 1]);
    env.w32(work, task);
    ev
}

/// C: `FUN_021C474C(gs, fm, obj)`: the event for `obj`, flag 0.
pub fn obj_event_new<E: FieldEnv>(env: &mut E, gs: u32, fm: u32, obj: u32) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gs, 0, PROC, 4]);
    let work = env.callv(EVENT_WORK, &[ev]);
    env.callv(OVERLAY_LOAD, &[OVERLAY]);
    let task = env.callv(TASK_START, &[gs, fm, obj, 0]);
    env.w32(work, task);
    ev
}

/// C: `FUN_021C4788(event, seq, work)`: the proc: once the task is done free it, unload the overlay and end (1).
pub fn proc_<E: FieldEnv>(env: &mut E, ev: u32) -> u32 {
    let work = env.callv(EVENT_WORK, &[ev]);
    let task = env.r32(work);
    if env.callv(TASK_DONE, &[task]) == 0 {
        return 0;
    }
    let task = env.r32(work);
    env.callv(TASK_FREE, &[task]);
    env.callv(OVERLAY_UNLOAD, &[OVERLAY]);
    1
}

#[cfg(test)]
mod tests;
