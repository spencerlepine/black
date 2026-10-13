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
//! So no method blocks, and the host never calls back into the VM. Mapping to the original (UNVERIFIED where the handler
//! is not ported yet; F8 fills in the command ids and their exact arguments):
//!
//! | method | original | caller |
//! |---|---|---|
//! | `pad_held` | `FUN_020362DC()` (held pad / touch bits) | commands 49 / 50 (`script_sys::wait_pad_a` / `wait_pad_b`, verified) |
//! | `message_bank_open` | `FUN_0215A760(ctx, type, msg)` (scrcmd_work.c) | `script_sys::vm_new` for ctx mode 0 (verified call site) |
//! | `message_*` | talk window / message printing (handlers not located yet) | F8 |
//! | `yes_no_*` | yes/no menu (handlers not located yet) | F8 |
//! | `menu_*` | list / choice menus (handlers not located yet) | F8 |
//! | `fade_*` | screen fades / wipes (probably `21.45 scrcmd_screeneffects.c`, by name only) | F8 |
//!
//! The request structs and the `message_*` .. `fade_*` methods are the planned interface, not ported behaviour: F8
//! adjusts their fields to what the handlers really pass.
//!
//! Text is passed as message ids (bank, index) plus the script's WORDSET (`SW + 0x2C`, `black_text::wordset`), not as
//! rendered strings: expansion and layout belong to `black-text` / the UI.

/// A message window request (UNVERIFIED field set; F8 confirms it against the message commands).
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

/// A fade request (UNVERIFIED field set; F8 confirms it against `scrcmd_screeneffects.c`).
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

/// Requests a running script makes to the game shell. Implemented by `black-game`; a headless test host can complete
/// every request at once.
pub trait ScriptHost {
    /// C: `FUN_020362DC()`: the held pad / touch bits of this frame, in the original's layout (commands 49 / 50 test
    /// `& 3` and `& 0xF3`; UNVERIFIED: presumably the NDS KEYINPUT order, bit 0 = A, bit 1 = B).
    fn pad_held(&mut self) -> u32;

    /// C: `FUN_0215A760(ctx, type, msg)`: the script's message bank (`type`, `msg` from `script_sys::script_lookup`)
    /// becomes the current one for its message commands.
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

    /// Open a choice menu of `items` (message ids in the current bank), `cancel` = B allowed.
    fn menu_start(&mut self, items: &[u16], cancel: bool);
    /// `Some(index)` of the chosen item, `Some(u16::MAX)` on cancel, `None` = still open.
    fn menu_result(&mut self) -> Option<u16>;

    /// Start a screen fade.
    fn fade_start(&mut self, req: FadeRequest);
    /// The last fade finished.
    fn fade_done(&mut self) -> bool;
}
