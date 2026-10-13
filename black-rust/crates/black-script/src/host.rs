//! `ScriptHost`: what a running script asks of the game shell (`black-game`, D6: no Bevy here).
//!
//! The original has no such interface: command handlers call the window / menu / fade / input code of overlays 10 and
//! 21 directly. The rewrite keeps that code on the Bevy side (D3: UI is rewritten, not ported), so the ported command
//! handlers (F8) call this trait instead. It follows the VM's own concurrency model:
//!
//! 1. a command handler (`fn(vm, ctx) -> 0 next / 1 yield`) issues a request (`*_start`), installs a wait callback with
//!    `vm::wait` (`FUN_020113D0`) and yields 1;
//! 2. every frame `vm::step` (`FUN_02011298`) calls that callback, which polls the host (`*_done` / `*_result`) and
//!    returns 1 once the request finished (the VM then runs the next command on the following frame);
//! 3. results go back into script vars through `script_sys::cmd_var_addr` (`FUN_02159AE8`), as the original's handlers
//!    do.
//!
//! So no method blocks, and the host never calls back into the VM. Mapping to the original (F8-spec1 read the scrcmd core
//! 0x02159204.. and `scrcmd_work.c` 0x0215A514..; specs `scrcmd_core.spec` / `scrcmd_work.spec`):
//!
//! | method | original | caller |
//! |---|---|---|
//! | `pad_held` | `FUN_020362DC()` (pad bits, NitroSDK `PAD_BUTTON_*` layout: A 1, B 2, .., X 0x400) | commands 49 / 50 (`script_sys::wait_pad_a` / `wait_pad_b`, verified); the menu wait `0x0215A990` tests 0x400 (X) |
//! | `message_bank_open` | `FUN_0215A760(ctx, type, msg)`: ctx +0x14 = `MsgData_Create` (`0x02045B38(0, type, msg, heap)`), +0x18 = msg | `script_sys::vm_new` for ctx mode 0 (verified call site) |
//! | `menu_start` | `0x0215A77C` (begin: x, y, cursor, 2 flag bytes, result var, WORDSET, message data) + `0x0215A7F0` per item (text msg, help msg or 0xFFFF, param) + `0x0215A864` (open the overlay-21 list menu `0x02190988`) | overlay-21 menu commands (calls from 0x021B018C..0x021B0388, F8 job 21.42) |
//! | `menu_result` | `0x0215A954` (poll `0x021909E4`; result = the chosen item's param (UNVERIFIED: what the overlay-21 list returns), 0xFFFE cancel) / `0x0215A990` (same, X -> 0xFFFD) | the wait callbacks of those commands (written to the var at MENU +8) |
//! | `message_*` | not in these ranges: the talk-window object is SCRCMD WORK +0xFC (`0x0215A614` / `0x0215A61C`), set by overlay-21 commands (0x021B0472..0x021B14B4) | F8 jobs 21.42 / 21.44 |
//! | `yes_no_*` | not in these ranges (UNVERIFIED: overlay-21 scrcmd clusters) | F8 jobs 21.42 / 21.44 |
//! | `fade_*` | not in these ranges (probably `21.45 scrcmd_screeneffects.c`, by name only) | F8 job 21.45 |
//!
//! Not host requests: command 3 (wait N frames) counts down in SCRCMD WORK +0x106 inside the VM (`0x0215A684` /
//! `0x0215A690`); the movement wait `0x0215A6FC` polls overlay-21 movement objects (field side, F4b).
//!
//! The `message_*`, `yes_no_*` and `fade_*` methods and their request structs are still the planned interface, not
//! ported behaviour: the F8 jobs that own those handlers adjust their fields. [`MenuRequest`] follows the menu builder
//! above (UNVERIFIED meaning of the flag bytes).
//!
//! Text is passed as message ids (bank, index) plus the script's WORDSET (`SW + 0x2C`, `black_text::wordset`), not as
//! rendered strings: expansion and layout belong to `black-text` / the UI.

/// A message window request (UNVERIFIED field set; the F8 jobs owning the message commands confirm it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MessageRequest {
    /// Message bank (ARC 2/3 file, `SCRIPT_RANGES[..].msg` or the zone's bank).
    pub bank: u16,
    /// Message index in the bank.
    pub index: u16,
    /// The script's WORDSET address (`SW + 0x2C`) for `{F000 ..}` tags.
    pub wordset: u32,
    /// Wait for a key press at the end of the text (vs. leaving the window open for the next command).
    pub wait_key: bool,
}

/// A fade request (UNVERIFIED field set; the F8 job of `scrcmd_screeneffects.c` confirms it).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FadeRequest {
    /// Fade out (to `color`) or in (from it).
    pub out: bool,
    /// Which screens: bit 0 top, bit 1 bottom.
    pub screens: u8,
    /// 0 black, 1 white.
    pub color: u8,
    /// Duration in frames.
    pub frames: u16,
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
    /// MENU +7 == 1.
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
    /// `& 0xF3`, the menu wait `0x0215A990` tests 0x400. UNVERIFIED: held vs. newly pressed, and the touch bits.
    fn pad_held(&mut self) -> u32;

    /// C: `FUN_0215A760(ctx, type, msg)`: the script's message bank (`type`, `msg` from `script_sys::script_lookup`)
    /// becomes the current one for its message commands (`MsgData_Create(0, type, msg, heap)` stored in the ctx).
    fn message_bank_open(&mut self, ctx: u32, ty: u32, msg: u32);

    /// Open (or reuse) the talk window and start printing a message.
    fn message_start(&mut self, req: MessageRequest);
    /// The message started last finished printing (and, with `wait_key`, the key was pressed).
    fn message_done(&mut self) -> bool;
    /// Close the talk window.
    fn message_close(&mut self);

    /// Show the yes/no menu.
    fn yes_no_start(&mut self);
    /// `Some(true)` = yes, `Some(false)` = no / cancelled, `None` = still open.
    fn yes_no_result(&mut self) -> Option<bool>;

    /// Open a choice menu (`0x0215A77C` .. `0x0215A864`).
    fn menu_start(&mut self, req: &MenuRequest);
    /// `None` while open; then the chosen item's `param`, [`MENU_CANCEL`] or [`MENU_X`] (`0x0215A954` / `0x0215A990`
    /// store it into the result var).
    fn menu_result(&mut self) -> Option<u16>;

    /// Start a screen fade.
    fn fade_start(&mut self, req: FadeRequest);
    /// The last fade finished.
    fn fade_done(&mut self) -> bool;
}
