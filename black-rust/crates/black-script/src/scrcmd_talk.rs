//! F8 (second batch): overlay-21 message / yes-no / menu script commands, Thumb code 0x021B0084-0x021B1668 (the tail of F1-SPLIT
//! row 21.42, all of row 21.43 and the head of row 21.44): the yes / no event, the choice-menu commands, the talk window, balloon
//! windows over field objects, system / info windows and the wait-for-button command (33 command ids, 70 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_talk.spec` (header: command-id table,
//! roles, the talk-window / balloon layouts). The functions listed in [`PORTED`] are ported here and checked with
//! `tools/check-fn.sh black-script scrcmd_talk`; the rest are the F8 port job's. The window callees (overlay-21 field_msgbg.c,
//! 0x021901EC..) are [`TalkHooks`] methods named by address; how they map onto [`crate::host::ScriptHost`] requests is in
//! `host.rs`.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_0084,
    0x021B_03E4,
    0x021B_0480,
    0x021B_09EC,
    0x021B_1550,
    0x021B_162C,
];

/// Init-proc mask bit of the talk window (set while it is open; `FUN_02159144` / `FUN_02159168` / `FUN_02159190`).
pub const BIT_TALK: u32 = 1;
/// Init-proc mask bit of the balloon window.
pub const BIT_BALLOON: u32 = 2;
/// Init-proc mask bit of the system window (commands 56 / 74).
pub const BIT_SYSTEM: u32 = 3;
/// Talk-window style of kind 1 (`FUN_02190FA0` argument).
pub const STYLE_1: u32 = 1;
/// Talk-window style of the other kinds.
pub const STYLE_13: u32 = 0x13;
/// Sound effect of the button press that ends command 75's wait.
pub const SE_BUTTON: u32 = 0x547;
/// Frames the yes / no event waits before it opens the menu.
pub const YES_NO_DELAY: u32 = 8;

/// Callees of the scrcmd_talk functions that this area does not port, named by address (meaning per the spec).
pub trait TalkHooks: MemEnv {
    /// C: `FUN_0215A5DC` (script_sys `ctx_gamesys`).
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A5F4` (script_sys `ctx_sw`).
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A60C` (scrcmd_work): the script's message data (ctx +0x14).
    fn fun_0215a60c(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A614` (scrcmd_work): SCRCMD WORK +0xFC = `w` (the open window).
    fn fun_0215a614(&mut self, ctx: u32, w: u32);
    /// C: `FUN_0215A61C` (scrcmd_work): SCRCMD WORK +0xFC.
    fn fun_0215a61c(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A624` (scrcmd_work): SCRCMD WORK +0x100 = `v` (the window icon object).
    fn fun_0215a624(&mut self, ctx: u32, v: u32);
    /// C: `FUN_0215A630` (scrcmd_work): SCRCMD WORK +0x100.
    fn fun_0215a630(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A654` (scrcmd_work): u8 SCRCMD WORK +0x104 (the talk-window kind; 7 initially).
    fn fun_0215a654(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A660` (scrcmd_work): u8 SCRCMD WORK +0x104 = `v`.
    fn fun_0215a660(&mut self, ctx: u32, v: u32);
    /// C: `FUN_0215A684` (scrcmd_work `set_countdown`).
    fn fun_0215a684(&mut self, ctx: u32, n: u32);
    /// C: `FUN_0215A690` (scrcmd_work `countdown`): 1 once the countdown is done.
    fn fun_0215a690(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02158F58` (script_sys): refresh the SW field block and return its address (word 0 = the message-window
    /// system `FUN_02188C54(field map)`).
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F74` (script_sys): the WORDSET (SW +0x2C).
    fn fun_02158f74(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F78` (script_sys): STRBUF 1 (SW +0x30).
    fn fun_02158f78(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F7C` (script_sys): STRBUF 2 (SW +0x34).
    fn fun_02158f7c(&mut self, sw: u32) -> u32;
    /// C: `FUN_02159144` (script_sys `proc_mask_set`).
    fn fun_02159144(&mut self, bit: u32);
    /// C: `FUN_02159168` (script_sys `proc_mask_clear`).
    fn fun_02159168(&mut self, bit: u32);
    /// C: `FUN_02159190` (script_sys `proc_mask_test`).
    fn fun_02159190(&mut self, bit: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02188C54(field map)` (overlay 21): the message-window system (msgbg).
    fn fun_02188c54(&mut self, fm: u32) -> u32;
    /// C: `FUN_02190BE4(msgbg, 0)`: open the yes / no menu (ScriptHost::yes_no_start).
    fn fun_02190be4(&mut self, msgbg: u32, arg: u32) -> u32;
    /// C: `FUN_02190C78(menu)`: poll the yes / no menu: 0 yes, 2 still open, other no (ScriptHost::yes_no_result).
    fn fun_02190c78(&mut self, menu: u32) -> u32;
    /// C: `FUN_021909BC(menu)`: close a list / yes-no menu.
    fn fun_021909bc(&mut self, menu: u32);
    /// C: `FUN_02190FA0(msgbg, msgdata, style)`: create the talk window (ScriptHost::message_open).
    fn fun_02190fa0(&mut self, msgbg: u32, msgdata: u32, style: u32) -> u32;
    /// C: `FUN_02191008(window)`: delete the talk window (ScriptHost::message_close).
    fn fun_02191008(&mut self, window: u32);
    /// C: `FUN_02191164(window)`: clear the talk window for the next message.
    fn fun_02191164(&mut self, window: u32);
    /// C: `FUN_02191134(window)`: talk window, waiting for a key (the "next" cursor; UNVERIFIED).
    fn fun_02191134(&mut self, window: u32);
    /// C: `FUN_02191794(window)`: the same for the system window (UNVERIFIED).
    fn fun_02191794(&mut self, window: u32);
    /// C: `FUN_021914CC(window)`: the same for the balloon window (UNVERIFIED).
    fn fun_021914cc(&mut self, window: u32);
    /// C: `FUN_0202F038(obj)` (main): delete the window icon object.
    fn fun_0202f038(&mut self, obj: u32);
    /// C: `FUN_020362DC()` (main): pad bits (ScriptHost::pad_held).
    fn fun_020362dc(&mut self) -> u32;
    /// C: `FUN_020061E4(se)` (main): play a sound effect.
    fn fun_020061e4(&mut self, se: u32);
    /// C: `FUN_02045CAC(msgdata, msg, strbuf)` (main, `MsgData` get-string, black-text).
    fn fun_02045cac(&mut self, msgdata: u32, msg: u32, strbuf: u32);
    /// C: `FUN_0201F250(wordset, dst, src)` (main, black-text wordset expand).
    fn fun_0201f250(&mut self, wordset: u32, dst: u32, src: u32);
}

/// C: unlabeled routine at `0x021B0084`: the GMEVENT function of command 71 (YES_NO; `event`, `seq` = the event's sequence
/// word, `work` = `{ctx, result var address, menu}`). Sequence 0: an 8-frame countdown is started (and sequence 1 runs in the
/// same call); 1: once the countdown is done the yes / no menu opens on the field's message-window system; 2: the menu's
/// answer goes into the var (0 yes, 1 no; 2 = still open); 3: the menu is closed and the event ends (1). Other sequences: 0.
pub fn yes_no_event<H: TalkHooks>(h: &mut H, _event: u32, seq: u32, work: u32) -> u32 {
    let ctx = h.r32(work);
    h.fun_0215a5f4(ctx);
    let s = h.r32(seq);
    if s > 3 {
        return 0;
    }
    if s == 3 {
        let menu = h.r32(work + 8);
        h.fun_021909bc(menu);
        return 1;
    }
    if s == 2 {
        let menu = h.r32(work + 8);
        let r = h.fun_02190c78(menu);
        if r == 2 {
            return 0;
        }
        let var = h.r32(work + 4);
        h.w16(var, (r != 0) as u16);
        h.w32(seq, s + 1);
        return 0;
    }
    if s == 0 {
        h.fun_0215a684(ctx, YES_NO_DELAY);
        let n = h.r32(seq).wrapping_add(1);
        h.w32(seq, n);
    }
    if h.fun_0215a690(ctx) == 1 {
        let gs = h.fun_0215a5dc(ctx);
        let fm = h.fun_0201210c(gs);
        let msgbg = h.fun_02188c54(fm);
        let menu = h.fun_02190be4(msgbg, 0);
        h.w32(work + 8, menu);
        let n = h.r32(seq).wrapping_add(1);
        h.w32(seq, n);
    }
    0
}

/// C: `FUN_overlay_d_21__021b03e4`: open the talk window (`kind` 0 = by the SCRCMD WORK kind byte, 1, 2, other). Already open
/// (init-proc bit 1) -> only cleared. Otherwise the style is 1 for kind 1 (and for kind 0 when the work byte is 1, 3 or 5),
/// 0x13 else (the kind then becomes 2); the window is created on the field's message-window system with the script's message
/// data, and the kind, the window and bit 1 are recorded.
pub fn talk_open<H: TalkHooks>(h: &mut H, ctx: u32, kind: u32) {
    if h.fun_02159190(BIT_TALK) != 0 {
        let w = h.fun_0215a61c(ctx);
        h.fun_02191164(w);
        return;
    }
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let msgdata = h.fun_0215a60c(ctx);
    let (style, kind) = match kind {
        1 => (STYLE_1, 1),
        2 => (STYLE_13, 2),
        0 => match h.fun_0215a654(ctx) {
            1 | 3 | 5 => (STYLE_1, 1),
            _ => (STYLE_13, 2),
        },
        _ => (STYLE_13, 2),
    };
    let msgbg = h.r32(fb);
    let w = h.fun_02190fa0(msgbg, msgdata, style);
    h.fun_0215a660(ctx, kind);
    h.fun_0215a614(ctx, w);
    h.fun_02159144(BIT_TALK);
}

/// C: `FUN_overlay_d_21__021b0480`: close the talk window when it is open (init-proc bit 1): delete it, delete the window icon
/// (SCRCMD WORK +0x100) if any, clear the window field and bit 1.
pub fn talk_close<H: TalkHooks>(h: &mut H, ctx: u32) {
    if h.fun_02159190(BIT_TALK) == 0 {
        return;
    }
    let w = h.fun_0215a61c(ctx);
    h.fun_02191008(w);
    let icon = h.fun_0215a630(ctx);
    if icon != 0 {
        h.fun_0202f038(icon);
        h.fun_0215a624(ctx, 0);
    }
    h.fun_0215a614(ctx, 0);
    h.fun_02159168(BIT_TALK);
}

/// C: `FUN_overlay_d_21__021b09ec`: the balloon tail of a balloon kind: `*a`, `*b` (u32) = 0, then kind 0: (1, 2), 2: a = 1,
/// 3: b = 7, 4: (1, 5), 5: b = 8, 6: (1, 6); kind 1 and kinds past 6 keep (0, 0).
pub fn balloon_tail<H: MemEnv>(h: &mut H, kind: u32, a: u32, b: u32) {
    h.w32(a, 0);
    h.w32(b, 0);
    let (va, vb) = match kind {
        0 => (Some(1), Some(2)),
        2 => (Some(1), None),
        3 => (None, Some(7)),
        4 => (Some(1), Some(5)),
        5 => (None, Some(8)),
        6 => (Some(1), Some(6)),
        _ => (None, None),
    };
    if let Some(v) = va {
        h.w32(a, v);
    }
    if let Some(v) = vb {
        h.w32(b, v);
    }
}

/// C: `sub_021b1550`: the wait callback of command 75 (wait for A / B): A or B (pad bits 0..1) -> the button sound, done (1).
/// Otherwise the open window shows its "next" cursor (talk window, else system window, else balloon) and the wait goes on (0);
/// with no window open it ends (1).
pub fn wait_button<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let pad = h.fun_020362dc();
    let w = h.fun_0215a61c(ctx);
    if pad & 3 != 0 {
        h.fun_020061e4(SE_BUTTON);
        return 1;
    }
    if h.fun_02159190(BIT_TALK) != 0 {
        h.fun_02191134(w);
    } else if h.fun_02159190(BIT_SYSTEM) != 0 {
        h.fun_02191794(w);
    } else if h.fun_02159190(BIT_BALLOON) != 0 {
        h.fun_021914cc(w);
    } else {
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_21__021b162c`: message `msg` of the script's message data, expanded with the script's WORDSET: the raw
/// text goes into STRBUF 2, the expansion into STRBUF 1, which is returned.
pub fn message_text<H: TalkHooks>(h: &mut H, ctx: u32, sw: u32, msg: u32) -> u32 {
    let msgdata = h.fun_0215a60c(ctx);
    let s1 = h.fun_02158f78(sw);
    let ws = h.fun_02158f74(sw);
    let s2 = h.fun_02158f7c(sw);
    h.fun_02045cac(msgdata, msg, s2);
    h.fun_0201f250(ws, s1, s2);
    s1
}

#[cfg(test)]
mod tests;
