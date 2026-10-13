//! F8: script command handlers, split into jobs by area (index of the areas; the code lives in the area modules), and the
//! command-id -> Rust handler table of the ported commands (the F7 [`CommandDispatch`](crate::hooks::CommandDispatch)
//! seam).
//!
//! The command table is at 0x0217062C (overlay 10 data, 609 entries = the count word at 0x021705D8, index = command id; F7-spec).
//! `tools/re/F1-SPLIT.md` counted a 624-slot run from 0x021705F0, whose first 15 words are the separate init-proc table of
//! `0x021591B4`: its "slot N" is command N - 15. Handlers are `fn(vm, ctx) -> 0 (next) / 1 (yield)`; see the `script_sys` and
//! `vm` spec headers. The VM reaches the handlers through `hooks::CommandDispatch` (handler address -> port); UI requests go
//! through `host::ScriptHost`.
//!
//! | area | original | commands | status |
//! |---|---|---|---|
//! | [`crate::script_sys`] | script_work.c tail 0x02159960.. | 49, 50, 311 | ported (F7) |
//! | [`crate::scrcmd_core`] | overlay 10 0x02159204-0x0215995E (F1 row 10.09) | 0..48 | 53/53 fns ported and verified (F8a) |
//! | [`crate::scrcmd_work`] | `scrcmd_work.c` 0x0215A514-0x0215C63A (F1 row 10.14) | 86 ids (140, 141, 149..151, 201..232, 234, 249..255, 257..260, 264, 266..284, 289..291, 312, 330, 331, 378, 379, 464..466, 488, 490..492, 562, 563) | 130/130 fns ported and verified (F8a) |
//! | [`crate::scrcmd_fld`] | overlay 21 0x021AE948-0x021B0083 (row 21.42, first part) | 80 ids (76..92, 94..99, 127..134, 136, 138, 139, 142, 146..148, 152, 155, 158..172, 181..187, 189..196, 538, 539, 584, 587, 589, 595, 596, 601, 602, 607) | 97 fns specced (F8-spec2), smoke ports |
//! | [`crate::scrcmd_talk`] | overlay 21 0x021B0084-0x021B1667 (rows 21.42 tail, 21.43, 21.44 head) | 33 ids (51..75, 135, 173..178, 579) | 70/70 fns ported and verified (F8b) |
//! | [`crate::scrcmd_obj`] | overlay 21 0x021B1668-0x021B282F (row 21.44) | 55 ids (100..126, 384, 385, 387..389, 391..398, 400..405, 407..410, 577, 583, 588, 603, 608) | 63 fns specced (F8-spec2), smoke ports |
//! | [`crate::scrcmd_screen`] | overlay 21 0x021B2830-0x021B2FD7 (rows 21.44 tail, 21.45, cmd 594) | 29 ids (179, 180, 292..306, 429..434, 436..440, 594) | 36/36 fns ported and verified (F8b) |
//!
//! Remaining F8 jobs (F1-SPLIT list): `scrcmd_proc.c` 0x0215C63C.., `scrcmd_sodateya.c`, the overlay-10 clusters at 0x0216A420 /
//! 0x0216B504, and the overlay-21 files / clusters 0x021B2FD8-0x021B73F6 (shop, proc_fld, encount, fld_battle, c03center,
//! pokemon_fld), 0x021CF7DC, 0x021D1670.
//!
//! # Dispatch
//!
//! [`commands`] lists every ported command id with its handler address (the ROM table word without the Thumb bit; checked
//! against the ROM by the tests). An implementation of `CommandDispatch` for the game routes the handler word to
//! [`dispatch`] (scrcmd_core, scrcmd_work, scrcmd_talk, scrcmd_screen), then to `script_sys::call_known` (commands 49 / 50 /
//! 311); `None` from both means the command belongs to an unported F8 job. Wait callbacks installed by these commands (VM
//! +0x10) go through [`call_wait`] (commands 3 / 28 and the overlay-21 message / fade waits) and `script_sys::call_known` (49 /
//! 50 / 311). The overlay-21 commands also hand code addresses to other systems: GMEVENT functions ([`call_event`]: the yes /
//! no and menu events), init procs ([`call_init_proc`]: `script_sys::INIT_PROCS` slots 1..6 and 9, run through
//! `ScriptSysHooks::call_init_proc`) and a field task ([`call_task`]: the field fade).

use crate::scrcmd_screen::ScreenHooks;
use crate::scrcmd_talk::TalkHooks;
use crate::scrcmd_work::WorkHooks;

/// The command tables of the ported areas (scrcmd_core, scrcmd_work, scrcmd_talk, scrcmd_screen; `script_sys` apart).
fn area_tables() -> [&'static [(u16, u32)]; 4] {
    [
        crate::scrcmd_core::COMMANDS,
        crate::scrcmd_work::COMMANDS,
        crate::scrcmd_talk::COMMANDS,
        crate::scrcmd_screen::COMMANDS,
    ]
}

/// Ported command id -> handler address (bit 0 clear), in id order: [`scrcmd_core::COMMANDS`](crate::scrcmd_core::COMMANDS),
/// [`scrcmd_work::COMMANDS`](crate::scrcmd_work::COMMANDS), [`scrcmd_talk::COMMANDS`](crate::scrcmd_talk::COMMANDS),
/// [`scrcmd_screen::COMMANDS`](crate::scrcmd_screen::COMMANDS) and the three `script_sys` commands.
pub fn commands() -> Vec<(u16, u32)> {
    let mut v: Vec<(u16, u32)> = area_tables()
        .iter()
        .flat_map(|t| t.iter())
        .copied()
        .collect();
    v.extend(SCRIPT_SYS_COMMANDS.iter().copied());
    v.sort_unstable();
    v
}

/// The commands ported in `script_sys` (F7): 49 / 50 wait for a key, 311.
pub const SCRIPT_SYS_COMMANDS: &[(u16, u32)] = &[
    (49, crate::script_sys::CMD_WAIT_PAD_A & !1),
    (50, crate::script_sys::CMD_WAIT_PAD_B & !1),
    (311, crate::script_sys::CMD_311 & !1),
];

/// Handler address (bit 0 clear) of ported command `id`, if it is ported.
pub fn handler(id: u16) -> Option<u32> {
    area_tables()
        .iter()
        .flat_map(|t| t.iter())
        .chain(SCRIPT_SYS_COMMANDS.iter())
        .find(|c| c.0 == id)
        .map(|c| c.1)
}

/// Run the command handler at `handler` (the command table word, Thumb bit allowed) when it is ported in scrcmd_core,
/// scrcmd_work, scrcmd_talk or scrcmd_screen; `None` otherwise (`script_sys::call_known` has commands 49 / 50 / 311).
pub fn dispatch<H: WorkHooks + TalkHooks + ScreenHooks>(
    h: &mut H,
    handler: u32,
    vm: u32,
    ctx: u32,
) -> Option<u32> {
    crate::scrcmd_core::dispatch(h, handler, vm, ctx)
        .or_else(|| crate::scrcmd_work::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_talk::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_screen::dispatch(h, handler, vm, ctx))
}

/// Run the wait callback `f` (VM +0x10, Thumb bit set) when it belongs to a ported scrcmd command (3, 28, the overlay-21
/// message windows, 75, 180, 298, 433); `None` otherwise.
pub fn call_wait<H: TalkHooks + ScreenHooks>(h: &mut H, f: u32, vm: u32, ctx: u32) -> Option<u32> {
    crate::scrcmd_core::call_wait(h, f, vm, ctx)
        .or_else(|| crate::scrcmd_talk::call_wait(h, f, vm, ctx))
        .or_else(|| crate::scrcmd_screen::call_wait(h, f, vm, ctx))
}

/// Run the GMEVENT function `f(event, seq, work)` when a ported command installed it (the yes / no event of command 71, the
/// menu event of commands 176 / 177); `None` otherwise.
pub fn call_event<H: TalkHooks>(h: &mut H, f: u32, event: u32, seq: u32, work: u32) -> Option<u32> {
    crate::scrcmd_talk::call_event(h, f, event, seq, work)
}

/// Run the init proc `f(task, task + 0xC)` of `script_sys::INIT_PROCS` when it is ported (slots 1..6 and 9, scrcmd_talk);
/// `None` otherwise.
pub fn call_init_proc<H: TalkHooks>(h: &mut H, f: u32, task: u32, x: u32) -> Option<u32> {
    crate::scrcmd_talk::call_init_proc(h, f, task, x)
}

/// Run the task function `f(task, work)` when a ported command added it (the field fade of commands 429..440); false
/// otherwise.
pub fn call_task<H: ScreenHooks>(h: &mut H, f: u32, task: u32, work: u32) -> bool {
    crate::scrcmd_screen::call_task(h, f, task, work)
}

#[cfg(test)]
mod tests;
