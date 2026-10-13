//! `ScriptHost`: what a running script asks of the game shell (`black-game`, D6: no Bevy here).
//!
//! The original has no such interface: command handlers call the window / menu / fade / input code of overlays 10 and
//! 21 directly. The rewrite keeps that code on the Bevy side (D3: UI is rewritten, not ported), so the ported command
//! handlers (F8) call this trait instead. It follows the VM's own concurrency model:
//!
//! 1. a command handler (`fn(vm, ctx) -> 0 next / 1 yield`) issues a request (`*_start` / `*_open`), installs a wait
//!    callback with `vm::wait` (`FUN_020113D0`) and yields 1 (or calls a child GMEVENT that does the waiting);
//! 2. every frame `vm::step` (`FUN_02011298`) calls that callback, which polls the host (`*_done` / `*_result`) and
//!    returns 1 once the request finished (the VM then runs the next command on the following frame);
//! 3. results go back into script vars through `script_sys::cmd_var_addr` (`FUN_02159AE8`), as the original's handlers
//!    do.
//!
//! So no method blocks, and the host never calls back into the VM. Mapping to the original (F8-spec1: scrcmd core and
//! `scrcmd_work.c`; F8-spec2: the overlay-21 areas `scrcmd_fld` / `scrcmd_talk` / `scrcmd_obj` / `scrcmd_screen`, specs in
//! `tools/rom-diff/specs/black-script/`). "Verified" = the call sites and arguments are read from the code and the smoke port
//! of the caller matches the ROM; the meaning of the overlay-21 window routines themselves (field_msgbg.c, 0x021901EC..,
//! render side, G5 / U2) is inferred:
//!
//! | method | original | callers |
//! |---|---|---|
//! | `pad_held` | `FUN_020362DC()` (pad bits, NitroSDK `PAD_BUTTON_*` layout: A 1, B 2, .., X 0x400) | commands 49 / 50 (`script_sys::wait_pad_a` / `wait_pad_b`), the menu wait `0x0215A990` (X), command 75 (`scrcmd_talk::wait_button`, A or B) |
//! | `message_bank_open` | `FUN_0215A760(ctx, type, msg)`: ctx +0x14 = `MsgData_Create` (`0x02045B38(0, type, msg, heap)`), +0x18 = msg | `script_sys::vm_new` for ctx mode 0 |
//! | `message_start` | talk window: `0x021B03E4` (open or reuse: `FUN_02190FA0(msgbg, msgdata, style)`) + `FUN_02191084(window, 0, 0, text)` (streamed, command 52) or `FUN_02191164` + `FUN_02191184` (put at once, command 53); system window: `FUN_02191558` + `FUN_021916A8` (commands 56 / 74); balloon: `0x021B0BAC` (`FUN_0219129C(msgbg, tail, &position, text, ..)`, or `FUN_02191348` for the next page; commands 60 / 61 / 72 / 73 / 135) | `scrcmd_talk::cmd_52` / `cmd_53` (through `talk_open` + `message_text`), `system_message` (56 / 74), `balloon_print` (60 / 61 / 72 / 73), `cmd_135` (its text is `FUN_0202A310(a, b, STRBUF1, heap)`, not a message id: a [`MessageRequest`] cannot express it yet; UNVERIFIED: a trainer text) |
//! | `message_done` | `FUN_021910C0(window) == 1` (talk), `FUN_021916E0` (system), `FUN_02191388` (balloon, which also follows its object every frame: `0x021B0B08`) | the wait callbacks `0x021B04C0` / `0x021B0FF0` / `0x021B0CEC` |
//! | `message_close` | talk `0x021B0480` (`FUN_02191008`), system `0x021B10A4` (`FUN_021915B4`), balloon `0x021B0A88` (`FUN_021912D8`, then polls `FUN_021912F8` until 1) / `0x021B0AB0` (`FUN_0219131C`, at once); command 63 closes every open one | commands 54 / 57 / 62 / 63, the init procs of mask bits 1..3 |
//! | `message_cursor` | `FUN_02191134` (talk) / `FUN_02191794` (system) / `FUN_021914CC` (balloon), called every frame while command 75 waits | `scrcmd_talk::wait_button` |
//! | `play_se` | `FUN_020061E4(se)` (main, 100 callers; UNVERIFIED: sound effect) | command 75 (0x547 on A / B), 126 (0x676), 166, 297 |
//! | `yes_no_start` / `yes_no_result` | `FUN_02190BE4(msgbg, 0)` / `FUN_02190C78(menu)`: 0 yes, 2 open, other no; closed by `FUN_021909BC` | command 71: GMEVENT `0x021B0084` (`scrcmd_talk::yes_no_event`): 8 frames of delay (`0x0215A684` / `690`), then the menu; var = 0 yes / 1 no |
//! | `menu_start` | `0x0215A77C` (begin: x, y, cursor, 2 flag bytes, result var, WORDSET, message data) + `0x0215A7F0` per item (text msg, help msg or 0xFFFF, param) + `0x0215A864` (open the overlay-21 list menu `0x02190988`) | commands 173 / 174 / 178 (begin; 173 without, 174 / 178 with the script's message data, 178 right-aligned), 175 (item), 176 / 177 (open + wait: GMEVENT `0x021B015C`, 8 frames of delay first) |
//! | `menu_result` | `0x0215A954` (poll `0x021909E4`; result = the chosen item's param (UNVERIFIED: what the overlay-21 list returns), 0xFFFE cancel) / `0x0215A990` (same, X -> 0xFFFD) | the GMEVENT `0x021B015C` (176 and 177 both pass "no X") |
//! | `fade_start` | `FUN_0204E720(mode, start, end, wait)` (fade.c) | command 179 (four u16 operands; `scrcmd_screen::cmd_fade`), the field fade task `0x021B2F18` (commands 429..440: `scrcmd_screen::field_fade_task`) |
//! | `fade_done` | `FUN_0204E804() != 1` (fade.c "running" is 1) | command 180 (`scrcmd_screen::wait_fade`); the field fade task (`scrcmd_screen::field_fade_task`) tests `FUN_0204E804() == 0` instead (the same for a 0 / 1 answer), then clears its field-map flag, which command 433 waits for (`FUN_02189A24 != 1`) |
//!
//! Not host requests: command 3 (wait N frames) counts down in SCRCMD WORK +0x106 inside the VM (`0x0215A684` /
//! `0x0215A690`); the movement wait `0x0215A6FC` polls overlay-21 movement objects (field side, F4b). The further overlay-21
//! window kinds of `scrcmd_talk` (F8b: the info board of commands 64..66, the sized info window of 58 / 59, the windows of
//! commands 67 / 579 and 69) are [`MessageWindow`] variants with the parameters their call sites pass; the window icon of
//! command 55 (`FUN_0202EE30` on the talk window, deleted by `FUN_0202F038` and by every talk-window close) is a decoration of
//! [`MessageWindow::Talk`], not a window. Command 51 toggles a msgbg flag (`FUN_021904DC(msgbg, 0 / 1)`, init-proc bit 9;
//! UNVERIFIED meaning).
//!
//! Proposed, not a method yet (F8d): the app-proc requests of `scrcmd_proc` (`FUN_020120B4(GAMESYS, ovl, proc table, param)`
//! starts an application, `FUN_020120F0(GAMESYS)` is nonzero while it runs; the app events `FUN_02014808` / `FUN_02014840`
//! wrap the same in a GMEVENT) would become `app_start(ovl, proc table, param)` / `app_running()`; which application each
//! overlay id is is not identified yet, so they stay `scrcmd_proc::ProcHooks` methods (mapping in `scrcmd_proc`'s module docs).
//!
//! Text is passed as message ids (bank, index) plus the script's WORDSET (`SW + 0x2C`, `black_text::wordset`), not as
//! rendered strings: the original expands the message into STRBUF 1 (`0x021B162C`: `FUN_02045CAC` + `FUN_0201F250`) right
//! before printing, and that expansion belongs to `black-text`; layout and printing belong to the UI.

/// Which message window a request is for (`scrcmd_talk`; the window object of every kind is SCRCMD WORK +0xFC).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageWindow {
    /// The talk window (`0x021B03E4`, init-proc mask bit 1). `style` is `FUN_02190FA0`'s third argument: 1 for kind 1 (and
    /// kind 0 when the SCRCMD WORK kind byte is 1 / 3 / 5), 0x13 otherwise (UNVERIFIED meaning: the frame type).
    Talk { style: u8 },
    /// A balloon over a field object (`0x021B0BAC`, bit 2): the object's OBJ id (0xFF = the player when the target is
    /// missing) and the tail kind (`0x021B09EC`; placement from the ROM table 0x021D939C or projected, `0x021B0960`).
    Balloon { object: u16, tail: u16 },
    /// The system window (`0x021B1008`, bit 3): `kind` 0 (command 56) / 1 (command 74); style 1 or 0x13 by a script byte.
    System { kind: u8, style: u8 },
    /// The info board (commands 64 / 66 print message 1 of message file 0x9E (type 3) with WORDSET slot 2 = a 7-digit number;
    /// 65 deletes it): `FUN_02190708(msgbg, msgdata, x, y, 9, 2)` with `x` = the first operand - 9 (u16) and `y` = the
    /// second (UNVERIFIED: a tile position, the board 9 x 2 tiles). Kept by the field map (`FUN_02188D48` / `FUN_02188D4C`),
    /// so it outlives the script (no init-proc bit).
    InfoBoard { x: u16, y: u16 },
    /// The sized info window (command 58, init-proc bit 5; 59 updates / closes it): `FUN_021918DC(msgbg, text, a, x, y, w, h)`
    /// after `FUN_02191830` measured the text (`w`, `h`), x pulled left to `x + w <= 0x1F`, `y + h > 0x18` -> y = h = 4
    /// (UNVERIFIED: tiles; `a` meaning unknown).
    Sized { a: u16, x: u8, y: u8, w: u8, h: u8 },
    /// The window of commands 67 (`FUN_02191A04(msgbg, a)`) / 579 (`FUN_02191A10(msgbg, a, obj)` with an extra object
    /// `FUN_0201D7EC(0x17, 4, 0, 0, heap)` kept in SW +0x38), init-proc bit 4; printed by its wait callback
    /// (`FUN_02191BE8(window, text) == 1` = done). `extra` = command 579 (UNVERIFIED meaning of `a` and of the object).
    Window67 { a: u16, extra: bool },
    /// The window of command 69 (`FUN_02192570(msgbg, a, x, y, w, h)`, init-proc bit 6): `x`, `y` script bytes, `w` / `h` the
    /// text's size (`FUN_0219270C` / `FUN_0219272C(msgbg, text, 4)`), printed at (4, 4) (`FUN_0219269C`), done =
    /// `FUN_021926D8 == 1` (UNVERIFIED meaning of `a`).
    Window69 { a: u16, x: u8, y: u8 },
}

/// Where the message text comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageBank {
    /// The script's own message data (`message_bank_open`; ctx +0x14). Balloons ask for it with bank 0x400.
    Script,
    /// Message file `n` of type 3 (`FUN_02045B38(0, 3, n, heap)`, loaded for the one message; balloon commands).
    File(u16),
}

/// How the text is printed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrintMode {
    /// Character by character; the command waits until [`ScriptHost::message_done`] (command 52, balloons, system window).
    Streamed,
    /// All at once, no wait (command 53: `FUN_02191184`).
    Instant,
}

/// A message request (verified against the call sites in `scrcmd_talk`; see the module docs). Waiting for a key is a separate
/// command (75), not part of the request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MessageRequest {
    pub window: MessageWindow,
    pub bank: MessageBank,
    /// Message index in the bank.
    pub index: u16,
    /// The script's WORDSET address (`SW + 0x2C`) for `{F000 ..}` tags.
    pub wordset: u32,
    pub print: PrintMode,
}

/// A screen fade (`FUN_0204E720(mode, start, end, wait)`, fade.c; the four values of command 179 are u16 script operands).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FadeRequest {
    /// fade.c mode bits; the field fade uses 3 (to / from black, both screens) and 0xC (white, both screens), so bit 0 / 1 =
    /// main / sub screen to black and bit 2 / 3 = to white (UNVERIFIED beyond those two values).
    pub mode: u16,
    /// Brightness at the start and the end (the field fade uses 0 and 16: its kinds 0 / 2, which also re-enable the field
    /// display, go 16 -> 0 (in), kinds 1 / 3 go 0 -> 16 (out); UNVERIFIED for other values).
    pub start: i16,
    pub end: i16,
    /// Speed (read from `FUN_0204E720`, UNVERIFIED in detail): < 0 = |wait| + 1 brightness steps per frame, >= 0 = one step
    /// every `wait` frames (the field fade uses -1 and 4).
    pub wait: i16,
}

/// One item of a choice menu (`0x0215A7F0(ctx, msg, help, param, ..)`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MenuItem {
    /// Item text: message id in the menu's message data (the script's bank, or message file 0x106 of type 3 when the menu
    /// was begun without message data), expanded with the menu's WORDSET.
    pub msg: u16,
    /// Help text shown while the cursor is on the item (`0xFFFF` in the original = none), same bank.
    pub help: Option<u16>,
    /// Value the menu returns when this item is chosen (UNVERIFIED: the list menu returns the item's param).
    pub param: u16,
}

/// A choice menu (`0x0215A77C` begin + one [`MenuItem`] per `0x0215A7F0` + `0x0215A864` open).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MenuRequest {
    /// Position (MENU +0 / +2; `right_align`: x is the right edge, the original subtracts the width).
    pub x: u16,
    pub y: u16,
    /// Initial cursor (MENU +4).
    pub cursor: u16,
    /// The u8 at MENU +6 (passed to the list menu as its last argument; UNVERIFIED meaning).
    pub flag6: u8,
    /// MENU +7 == 1 (command 178).
    pub right_align: bool,
    pub items: Vec<MenuItem>,
}

/// [`ScriptHost::menu_result`] when the menu was cancelled (B; the list returned -2).
pub const MENU_CANCEL: u16 = 0xFFFE;
/// [`ScriptHost::menu_result`] when X closed the menu (the `0x0215A990` wait only).
pub const MENU_X: u16 = 0xFFFD;

/// Requests a running script makes to the game shell. Implemented by `black-game`; a headless test host can complete
/// every request at once.
pub trait ScriptHost {
    /// C: `FUN_020362DC()`: the pad bits of this frame in NitroSDK `PAD_BUTTON_*` order (A 1, B 2, SELECT 4, START 8,
    /// RIGHT 0x10, LEFT 0x20, UP 0x40, DOWN 0x80, R 0x100, L 0x200, X 0x400, Y 0x800): commands 49 / 50 test `& 3` /
    /// `& 0xF3`, the menu wait `0x0215A990` tests 0x400, command 75 tests `& 3`. UNVERIFIED: held vs. newly pressed, and the
    /// touch bits.
    fn pad_held(&mut self) -> u32;

    /// C: `FUN_0215A760(ctx, type, msg)`: the script's message bank (`type`, `msg` from `script_sys::script_lookup`)
    /// becomes the current one for its message commands (`MsgData_Create(0, type, msg, heap)` stored in the ctx).
    fn message_bank_open(&mut self, ctx: u32, ty: u32, msg: u32);

    /// Open (or, when that window is already open, clear and reuse) a message window and print a message into it.
    fn message_start(&mut self, req: &MessageRequest);
    /// The message started last in `window` finished printing (`FUN_021910C0` / `FUN_021916E0` / `FUN_02191388` == 1).
    fn message_done(&mut self, window: MessageWindow) -> bool;
    /// Close a message window (the balloon closes animated: the script polls [`ScriptHost::message_closed`]).
    fn message_close(&mut self, window: MessageWindow);
    /// The balloon close animation finished (`FUN_021912F8(window) == 1`); other windows close at once (true).
    fn message_closed(&mut self, window: MessageWindow) -> bool;
    /// One frame of "waiting for a key" in the open window (the "next" cursor; command 75).
    fn message_cursor(&mut self, window: MessageWindow);

    /// C: `FUN_020061E4(se)`: play sound effect `se` (UNVERIFIED: the sound side of main; 0x547 = the message button).
    fn play_se(&mut self, se: u16);

    /// Show the yes / no menu (`FUN_02190BE4(msgbg, 0)`; the script delays it by 8 frames itself).
    fn yes_no_start(&mut self);
    /// `Some(true)` = yes (`FUN_02190C78` 0), `Some(false)` = no (any value but 0 and 2), `None` = still open (2). The menu is
    /// closed (`FUN_021909BC`) once a result was returned.
    fn yes_no_result(&mut self) -> Option<bool>;

    /// Open a choice menu (`0x0215A77C` .. `0x0215A864`).
    fn menu_start(&mut self, req: &MenuRequest);
    /// `None` while open; then the chosen item's `param`, [`MENU_CANCEL`] or [`MENU_X`] (`0x0215A954` / `0x0215A990`
    /// store it into the result var).
    fn menu_result(&mut self) -> Option<u16>;

    /// C: `FUN_0204E720`: start a screen fade.
    fn fade_start(&mut self, req: FadeRequest);
    /// C: `FUN_0204E804() != 1`: no fade is running.
    fn fade_done(&mut self) -> bool;
}
