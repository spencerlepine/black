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
//! | [`crate::scrcmd_fld`] | overlay 21 0x021AE948-0x021B0083 (row 21.42, first part) | 80 ids (76..92, 94..99, 127..134, 136, 138, 139, 142, 146..148, 152, 155, 158..172, 181..187, 189..196, 538, 539, 584, 587, 589, 595, 596, 601, 602, 607) | 97/97 fns ported and verified (F8c) |
//! | [`crate::scrcmd_talk`] | overlay 21 0x021B0084-0x021B1667 (rows 21.42 tail, 21.43, 21.44 head) | 33 ids (51..75, 135, 173..178, 579) | 70/70 fns ported and verified (F8b) |
//! | [`crate::scrcmd_obj`] | overlay 21 0x021B1668-0x021B282F (row 21.44) | 55 ids (100..126, 384, 385, 387..389, 391..398, 400..405, 407..410, 577, 583, 588, 603, 608) | 63/63 fns ported and verified (F8c) |
//! | [`crate::scrcmd_screen`] | overlay 21 0x021B2830-0x021B2FD7 (rows 21.44 tail, 21.45, cmd 594) | 29 ids (179, 180, 292..306, 429..434, 436..440, 594) | 36/36 fns ported and verified (F8b) |
//! | [`crate::scrcmd_proc`] | `scrcmd_proc.c` overlay 10 0x0215C63C-0x0215CE97 (row 10.15 minus its tail) | 17 ids (332, 334, 336, 337, 339..341, 355..357, 370, 383, 386, 390, 399, 406, 489) | 22/22 fns ported and verified (F8d) |
//! | [`crate::scrcmd_sodateya`] | `scrcmd_sodateya.c` overlay 10 0x0215CE98-0x0215D34B (row 10.16 + the tail of 10.15) | 15 ids (235..248, 593) | 18/18 fns ported and verified (F8d) |
//! | [`crate::scrcmd_misc10`] | overlay 10 0x0216A420-0x0216AC3F (row 10.33), 0x0216B504-0x0216B86B (row 10.35), cmd 359 at 0x0215D34C (`scrcmd_musical.c`, X2) | 30 ids (359, 506..520, 528..533, 535, 540..543, 549, 550, 559) | 32/32 in-scope fns ported and verified (F8d; cmd 359 musical, X2, specced only) |
//! | [`crate::scrcmd_shop`] | `scrcmd_shop.c` overlay 21 0x021B2FD8-0x021B4BEB (row 21.46 minus its tail) | 1 id (329) | 13 logic fns ported and verified (F8e; 36 UI / render fns behind `ShopHooks`) |
//! | [`crate::scrcmd_proc_fld`] | `scrcmd_proc_fld.c` overlay 21 0x021B4BEC-0x021B518F (row 21.47 + the tail of 21.46) | 20 ids (188, 314, 333, 335, 338, 342, 351..354, 358, 376, 377, 380, 381, 411, 444, 536, 537, 580) | 21/21 fns ported and verified (F8e) |
//! | [`crate::scrcmd_encount`] | `scrcmd_encount.c` overlay 21 0x021B5190-0x021B56FF (row 21.48 + its helper) | 14 ids (319..328, 418..420, 561) | 19/19 fns ported and verified (F8e) |
//! | [`crate::scrcmd_fld_battle`] | `scrcmd_fld_battle.c` overlay 21 0x021B5700-0x021B5A17 (row 21.49) | 2 ids (441, 442) | 6/6 fns ported and verified (F8e) |
//! | [`crate::scrcmd_c03center`] | `scrcmd_c03center.c` overlay 21 0x021B5A18-0x021B66B7 (row 21.50; no X1 networking) | 22 ids (256, 261..263, 265, 285..288, 307, 446, 447, 462, 463, 484..487, 566..568, 578) | 28/28 fns ported and verified (F8e) |
//! | [`crate::scrcmd_pokemon_fld`] | `scrcmd_pokemon_fld.c` overlay 21 0x021B66B8-0x021B73FF (row 21.51; cmd 494 = Dream World, X2) | 14 ids (308, 459..461, 470, 471, 494, 526, 527, 564, 565, 581, 585, 586; 494 not dispatched) | 47/48 fns ported and verified (F8e; the cmd 494 handler, X2, specced only) |
//! | [`crate::scrcmd_misc21`] | overlay 21 0x021CF7DC-0x021D013F (row 21.91), 0x021D1670-0x021D30B7 (row 21.97), no `__FILE__` | 36 ids (233, 309, 347..350, 412..414, 472..476, 479, 481..483, 521..525, 551, 553, 569..571, 573..575, 582, 597..600) | 81/81 logic fns ported and verified (F8e; cmd 414 and 10 render / UI fns behind `Misc21Hooks`) |
//!
//! Command ids outside these areas (94, F8-spec4 count): 93, 310, 313, 315..318, 360..366, 425, 427, 445, 458, 467..469, 477,
//! 478, 480, 493, 495, 499..505, 534, 544..548, 552, 554..558, 560, 572, 590..592, 604..606 (other overlay-10 files, incl.
//! `scrcmd_musical.c` 360..366 and `scrcmd_network.c`), 367..369, 371..375 (overlay 18, intrude / comm, X1), 450..457 (overlay
//! 20), 382 (0x021F4650, outside the field set), 197..200, 343..346, 415..417, 421..424, 426, 428, 443, 448, 449, 496..498,
//! 576 (overlay-21 `sodateya.c`, `bsubway_scr.c`, 0x021C8xxx, `fld_trade.c` / `scrcmd_palpark.c`): F10 / X jobs or later F8
//! rows.
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

use crate::scrcmd_c03center::C03Hooks;
use crate::scrcmd_encount::EncountHooks;
use crate::scrcmd_fld::FldHooks;
use crate::scrcmd_fld_battle::FldBattleHooks;
use crate::scrcmd_misc10::Misc10Hooks;
use crate::scrcmd_misc21::Misc21Hooks;
use crate::scrcmd_obj::ObjHooks;
use crate::scrcmd_pokemon_fld::PokemonFldHooks;
use crate::scrcmd_proc::ProcHooks;
use crate::scrcmd_proc_fld::ProcFldHooks;
use crate::scrcmd_screen::ScreenHooks;
use crate::scrcmd_shop::ShopHooks;
use crate::scrcmd_sodateya::SodateyaHooks;
use crate::scrcmd_talk::TalkHooks;
use crate::scrcmd_work::WorkHooks;

/// The command tables of the ported areas (the F8a..F8e areas; `script_sys` apart).
fn area_tables() -> [&'static [(u16, u32)]; 16] {
    [
        crate::scrcmd_core::COMMANDS,
        crate::scrcmd_work::COMMANDS,
        crate::scrcmd_talk::COMMANDS,
        crate::scrcmd_screen::COMMANDS,
        crate::scrcmd_fld::COMMANDS,
        crate::scrcmd_obj::COMMANDS,
        crate::scrcmd_proc::COMMANDS,
        crate::scrcmd_sodateya::COMMANDS,
        crate::scrcmd_misc10::COMMANDS,
        crate::scrcmd_shop::COMMANDS,
        crate::scrcmd_proc_fld::COMMANDS,
        crate::scrcmd_encount::COMMANDS,
        crate::scrcmd_fld_battle::COMMANDS,
        crate::scrcmd_c03center::COMMANDS,
        crate::scrcmd_pokemon_fld::COMMANDS,
        crate::scrcmd_misc21::COMMANDS,
    ]
}

/// Ported command id -> handler address (bit 0 clear), in id order: the `COMMANDS` of every area in [`area_tables`] and the
/// three `script_sys` commands.
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

/// Run the command handler at `handler` (the command table word, Thumb bit allowed) when it is ported in one of the F8 areas;
/// `None` otherwise (`script_sys::call_known` has commands 49 / 50 / 311; commands 359 and 494 are X2, not ported).
pub fn dispatch<
    H: WorkHooks
        + TalkHooks
        + ScreenHooks
        + FldHooks
        + ObjHooks
        + ProcHooks
        + SodateyaHooks
        + Misc10Hooks
        + ShopHooks
        + ProcFldHooks
        + EncountHooks
        + FldBattleHooks
        + C03Hooks
        + PokemonFldHooks
        + Misc21Hooks,
>(
    h: &mut H,
    handler: u32,
    vm: u32,
    ctx: u32,
) -> Option<u32> {
    crate::scrcmd_core::dispatch(h, handler, vm, ctx)
        .or_else(|| crate::scrcmd_work::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_talk::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_screen::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_fld::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_obj::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_proc::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_sodateya::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_misc10::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_shop::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_proc_fld::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_encount::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_fld_battle::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_c03center::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_pokemon_fld::dispatch(h, handler, vm, ctx))
        .or_else(|| crate::scrcmd_misc21::dispatch(h, handler, vm, ctx))
}

/// Run the wait callback `f` (VM +0x10, Thumb bit set) when it belongs to a ported scrcmd command (3, 28, the overlay-21
/// message windows, 75, 180, 298, 433, scrcmd_fld 160 / 168 / 170 / 172, scrcmd_obj 101 / 117 / the talk lock, the shop 329);
/// `None` otherwise.
pub fn call_wait<H: TalkHooks + ScreenHooks + FldHooks + ObjHooks + ProcHooks + ShopHooks>(
    h: &mut H,
    f: u32,
    vm: u32,
    ctx: u32,
) -> Option<u32> {
    crate::scrcmd_core::call_wait(h, f, vm, ctx)
        .or_else(|| crate::scrcmd_talk::call_wait(h, f, vm, ctx))
        .or_else(|| crate::scrcmd_screen::call_wait(h, f, vm, ctx))
        .or_else(|| crate::scrcmd_fld::call_wait(h, f, vm, ctx))
        .or_else(|| crate::scrcmd_obj::call_wait(h, f, vm, ctx))
        .or_else(|| crate::scrcmd_proc::call_wait(h, f, vm, ctx))
        .or_else(|| crate::scrcmd_shop::call_wait(h, f, vm, ctx))
}

/// Run the GMEVENT function `f(event, seq, work)` when a ported command installed it (the yes / no event of command 71, the
/// menu event of commands 176 / 177, the party-select event of command 247, the object moves of commands 549 / 550, the F8e
/// events of scrcmd_fld_battle / scrcmd_encount / scrcmd_misc21); `None` otherwise.
pub fn call_event<
    H: TalkHooks + SodateyaHooks + Misc10Hooks + FldBattleHooks + EncountHooks + Misc21Hooks,
>(
    h: &mut H,
    f: u32,
    event: u32,
    seq: u32,
    work: u32,
) -> Option<u32> {
    crate::scrcmd_talk::call_event(h, f, event, seq, work)
        .or_else(|| crate::scrcmd_sodateya::call_event(h, f, event, seq, work))
        .or_else(|| crate::scrcmd_misc10::call_event(h, f, event, seq, work))
        .or_else(|| crate::scrcmd_fld_battle::call_event(h, f, event, seq, work))
        .or_else(|| crate::scrcmd_encount::call_event(h, f, event, seq, work))
        .or_else(|| crate::scrcmd_misc21::call_event(h, f, event, seq, work))
}

/// Run the app-proc result callback `cb(w)` of scrcmd_proc (commands 334 / 336 / 489, see
/// [`ProcHooks::call_proc_callback`] and the app events `FUN_02014808` / `FUN_02014840`) or scrcmd_proc_fld (command 335);
/// false otherwise.
pub fn call_callback<H: ProcHooks + ProcFldHooks>(h: &mut H, cb: u32, w: u32) -> bool {
    crate::scrcmd_proc::call_callback(h, cb, w) || crate::scrcmd_proc_fld::call_callback(h, cb, w)
}

/// Run the init proc `f(task, task + 0xC)` of `script_sys::INIT_PROCS` when it is ported (slots 1..6 and 9 in scrcmd_talk,
/// mask bits 10 / 12 / 13 in scrcmd_fld, slot 0 in scrcmd_encount, slot 8 in scrcmd_misc21); `None` otherwise.
pub fn call_init_proc<H: TalkHooks + FldHooks + EncountHooks + Misc21Hooks>(
    h: &mut H,
    f: u32,
    task: u32,
    x: u32,
) -> Option<u32> {
    crate::scrcmd_talk::call_init_proc(h, f, task, x)
        .or_else(|| crate::scrcmd_fld::init_proc(h, f, task, x))
        .or_else(|| crate::scrcmd_encount::call_init_proc(h, f, task, x))
        .or_else(|| crate::scrcmd_misc21::call_init_proc(h, f, task, x))
}

/// Run the task function `f(task, work)` when a ported command added it (the field fade of commands 429..440, the turn tween
/// of command 307); false otherwise.
pub fn call_task<H: ScreenHooks + C03Hooks>(h: &mut H, f: u32, task: u32, work: u32) -> bool {
    crate::scrcmd_screen::call_task(h, f, task, work)
        || crate::scrcmd_c03center::call_task(h, f, task, work)
}

/// Run the field-proc callback `f(a, b, work)` of a proc table a ported command loads (the OBJ animation of commands 347..350,
/// proc table 0x021DD5F0 via `FUN_021BB990`); false otherwise.
pub fn call_field_proc<H: Misc21Hooks>(h: &mut H, f: u32, a: u32, b: u32, work: u32) -> bool {
    crate::scrcmd_misc21::call_field_proc(h, f, a, b, work)
}

#[cfg(test)]
mod tests;
