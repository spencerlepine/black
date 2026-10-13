//! F8b: overlay-21 message / yes-no / menu script commands, Thumb code 0x021B0084-0x021B1668 (the tail of F1-SPLIT row 21.42,
//! all of row 21.43 and the head of row 21.44): the yes / no event, the choice-menu commands, the talk window, balloon windows
//! over field objects, system / info windows and the wait-for-button command (33 command ids, 70 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_talk.spec` (header: command-id table,
//! roles, the talk-window / balloon layouts), ported here, listed in [`PORTED`] and checked with `tools/check-fn.sh black-script
//! scrcmd_talk`. [`COMMANDS`] maps the command ids to their handlers; [`dispatch`], [`call_wait`], [`call_event`] and
//! [`call_init_proc`] run a handler / wait callback / GMEVENT function / init proc of this area by its address.
//!
//! Callees outside the area are [`TalkHooks`] methods named by address (vm.c, the script_sys operand readers, the scrcmd ctx
//! getters, the countdown and the child-event call come from [`CoreHooks`]). The window routines (overlay-21 field_msgbg.c,
//! 0x021901EC..) are the render side; how a game-side implementation of these hooks maps them onto
//! [`crate::host::ScriptHost`] requests is in the module docs of `host.rs` (e.g. command 52 = [`talk_open`] +
//! [`message_text`] + `FUN_02191084` = one `message_start` with [`crate::host::MessageWindow::Talk`] and
//! [`crate::host::PrintMode::Streamed`]; its wait callback [`wait_talk`] polls `message_done` through `FUN_021910C0`).
//!
//! Stack records the original passes to its callees (positions, camera vectors and matrices, size bytes) are byte arrays.

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_0084,
    0x021B_010C,
    0x021B_0130,
    0x021B_015C,
    0x021B_01B8,
    0x021B_01DC,
    0x021B_024C,
    0x021B_02C8,
    0x021B_0344,
    0x021B_0394,
    0x021B_03BC,
    0x021B_03E4,
    0x021B_0480,
    0x021B_04C0,
    0x021B_04D8,
    0x021B_0530,
    0x021B_05AC,
    0x021B_0624,
    0x021B_0630,
    0x021B_063C,
    0x021B_0730,
    0x021B_07E0,
    0x021B_0808,
    0x021B_090C,
    0x021B_0960,
    0x021B_09D8,
    0x021B_09EC,
    0x021B_0A40,
    0x021B_0A60,
    0x021B_0A88,
    0x021B_0AB0,
    0x021B_0ADC,
    0x021B_0B08,
    0x021B_0BAC,
    0x021B_0CEC,
    0x021B_0D14,
    0x021B_0D74,
    0x021B_0DF0,
    0x021B_0E74,
    0x021B_0EDC,
    0x021B_0EE8,
    0x021B_0EF4,
    0x021B_0FF0,
    0x021B_1008,
    0x021B_10A4,
    0x021B_10D0,
    0x021B_10DC,
    0x021B_10E8,
    0x021B_10F4,
    0x021B_1100,
    0x021B_11D8,
    0x021B_120C,
    0x021B_1234,
    0x021B_1240,
    0x021B_1268,
    0x021B_12B0,
    0x021B_1334,
    0x021B_13E0,
    0x021B_13EC,
    0x021B_13F8,
    0x021B_1424,
    0x021B_143C,
    0x021B_14DC,
    0x021B_14E8,
    0x021B_14F4,
    0x021B_1550,
    0x021B_15B8,
    0x021B_15C8,
    0x021B_1604,
    0x021B_162C,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (51, 0x021B_15C8),
    (52, 0x021B_04D8),
    (53, 0x021B_0530),
    (54, 0x021B_0624),
    (55, 0x021B_05AC),
    (56, 0x021B_10D0),
    (57, 0x021B_10E8),
    (58, 0x021B_1100),
    (59, 0x021B_11D8),
    (60, 0x021B_0D14),
    (61, 0x021B_0D74),
    (62, 0x021B_0EDC),
    (63, 0x021B_14F4),
    (64, 0x021B_063C),
    (65, 0x021B_07E0),
    (66, 0x021B_0730),
    (67, 0x021B_12B0),
    (68, 0x021B_13E0),
    (69, 0x021B_143C),
    (70, 0x021B_14DC),
    (71, 0x021B_0130),
    (72, 0x021B_0DF0),
    (73, 0x021B_0E74),
    (74, 0x021B_10DC),
    (75, 0x021B_15B8),
    (135, 0x021B_0EF4),
    (173, 0x021B_01DC),
    (174, 0x021B_024C),
    (175, 0x021B_0344),
    (176, 0x021B_0394),
    (177, 0x021B_03BC),
    (178, 0x021B_02C8),
    (579, 0x021B_1334),
];

/// Init-proc mask bit of the talk window (set while it is open; `FUN_02159144` / `FUN_02159168` / `FUN_02159190`).
pub const BIT_TALK: u32 = 1;
/// Init-proc mask bit of the balloon window.
pub const BIT_BALLOON: u32 = 2;
/// Init-proc mask bit of the system window (commands 56 / 74).
pub const BIT_SYSTEM: u32 = 3;
/// Init-proc mask bit of the window of commands 67 / 579.
pub const BIT_WINDOW_67: u32 = 4;
/// Init-proc mask bit of the sized info window (command 58).
pub const BIT_INFO: u32 = 5;
/// Init-proc mask bit of the window of command 69.
pub const BIT_WINDOW_69: u32 = 6;
/// Init-proc mask bit of the msgbg flag of command 51.
pub const BIT_MSGBG_FLAG: u32 = 9;
/// Talk-window style of kind 1 (`FUN_02190FA0` argument).
pub const STYLE_1: u32 = 1;
/// Talk-window style of the other kinds.
pub const STYLE_13: u32 = 0x13;
/// Sound effect of the button press that ends command 75's wait.
pub const SE_BUTTON: u32 = 0x547;
/// Frames the yes / no and menu events wait before they open the menu.
pub const YES_NO_DELAY: u32 = 8;
/// Bank value of the balloon commands meaning "the script's own message data".
pub const BANK_SCRIPT: u32 = 0x400;
/// Message file of the info board (commands 64 / 66; type 3).
pub const MSG_INFO_BOARD: u32 = 0x9E;
/// The OBJ id of the player.
pub const OBJ_PLAYER: u32 = 0xFF;

/// The GMEVENT function of command 71 ([`yes_no_event`], Thumb).
pub const EVENT_YES_NO: u32 = 0x021B_0085;
/// The GMEVENT function of commands 176 / 177 ([`menu_event`], Thumb).
pub const EVENT_MENU: u32 = 0x021B_015D;
/// Wait callback of command 52 ([`wait_talk`], Thumb).
pub const WAIT_TALK: u32 = 0x021B_04C1;
/// Wait callback of the balloon close ([`wait_balloon_close`], Thumb).
pub const WAIT_BALLOON_CLOSE: u32 = 0x021B_0A61;
/// Wait callback of the balloon messages ([`wait_balloon`], Thumb).
pub const WAIT_BALLOON: u32 = 0x021B_0CED;
/// Wait callback of the system window ([`wait_system`], Thumb).
pub const WAIT_SYSTEM: u32 = 0x021B_0FF1;
/// Wait callback of commands 67 / 579 ([`wait_window_67`], Thumb).
pub const WAIT_WINDOW_67: u32 = 0x021B_1241;
/// Wait callback of command 69 ([`wait_window_69`], Thumb).
pub const WAIT_WINDOW_69: u32 = 0x021B_1425;
/// Wait callback of command 75 ([`wait_button`], Thumb).
pub const WAIT_BUTTON: u32 = 0x021B_1551;

/// Balloon work B (SCRCMD WORK +0xD4, 0x28 bytes; `FUN_0215A63C`): u16 object id.
pub const B_OBJ: u32 = 0x00;
/// Balloon work: u16 tail.
pub const B_TAIL: u32 = 0x02;
/// Balloon work: the window position (3 fx32 words) = object position + offset.
pub const B_POS: u32 = 0x04;
/// Balloon work: the object position last seen (3 words).
pub const B_OBJ_POS: u32 = 0x10;
/// Balloon work: the offset from the object (3 words; [`balloon_place`]).
pub const B_OFS: u32 = 0x1C;
/// Size of the balloon work.
pub const B_SIZE: u32 = 0x28;

/// The fx32 up vector `{0, 0x1000, 0}` at overlay-21 0x021D9390 ([`balloon_project`]).
pub const UP: [u32; 3] = [0, 0x1000, 0];
/// ROM address of [`UP`].
pub const UP_ADDR: u32 = 0x021D_9390;
/// The balloon offsets at overlay-21 0x021D939C: `[col][row]` (2 x 4) fx32 vectors ([`balloon_project`] picks one by the
/// screen position, [`balloon_place`] by the kind).
pub const OFFSETS: [[[u32; 3]; 4]; 2] = [
    [
        [0, 0x10000, 0xFFFF_8000],
        [0xFFFF_8000, 0xC000, 0xFFFF_8000],
        [0x8000, 0xC000, 0xFFFF_8000],
        [0, 0x10000, 0xFFFF_8000],
    ],
    [
        [0, 0xC000, 0xFFFF_8000],
        [0xFFFF_8000, 0xC000, 0xFFFF_8000],
        [0x8000, 0xC000, 0xFFFF_8000],
        [0, 0xC000, 0x8000],
    ],
];
/// ROM address of [`OFFSETS`].
pub const OFFSETS_ADDR: u32 = 0x021D_939C;

/// Callees of the scrcmd_talk functions outside the area, named by address (meaning per the spec; names of main / overlay-21
/// routines are inferred, UNVERIFIED unless stated). Methods with a body default to their port in this crate (the area test
/// overrides them with the spec's stubs). Some overlay-10 / main callees are also [`crate::scrcmd_work::WorkHooks`] methods
/// of the same name; a type implementing both traits implements them twice.
#[allow(clippy::too_many_arguments)]
pub trait TalkHooks: CoreHooks {
    // ---- script_sys / scrcmd_work (overlay 10)
    /// C: `FUN_0215A5CC` (`script_sys::ctx_heap`): the ctx's heap id.
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::ctx_heap(self, ctx)
    }
    /// C: `FUN_0215A5EC` (`script_sys::ctx_work_3`): the field-object (MMDL) system.
    fn fun_0215a5ec(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::ctx_work_3(self, ctx)
    }
    /// C: `FUN_0215A60C` (`scrcmd_work::ctx_msgdata`): the script's message data.
    fn fun_0215a60c(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::scrcmd_work::ctx_msgdata(self, ctx)
    }
    /// C: `FUN_0215A614` (`scrcmd_work::set_window`): SCRCMD WORK +0xFC = `w` (the open window).
    fn fun_0215a614(&mut self, ctx: u32, w: u32)
    where
        Self: Sized,
    {
        crate::scrcmd_work::set_window(self, ctx, w)
    }
    /// C: `FUN_0215A61C` (`scrcmd_work::window`): SCRCMD WORK +0xFC.
    fn fun_0215a61c(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::scrcmd_work::window(self, ctx)
    }
    /// C: `FUN_0215A624` (`scrcmd_work::set_work_100`): SCRCMD WORK +0x100 = `v` (the window icon object).
    fn fun_0215a624(&mut self, ctx: u32, v: u32)
    where
        Self: Sized,
    {
        crate::scrcmd_work::set_work_100(self, ctx, v)
    }
    /// C: `FUN_0215A630` (`scrcmd_work::work_100`): SCRCMD WORK +0x100.
    fn fun_0215a630(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::scrcmd_work::work_100(self, ctx)
    }
    /// C: `FUN_0215A63C` (`scrcmd_work::work_d4_addr`): the balloon work B (SCRCMD WORK +0xD4).
    fn fun_0215a63c(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::scrcmd_work::work_d4_addr(self, ctx)
    }
    /// C: `FUN_0215A654` (`scrcmd_work::work_104`): u8 SCRCMD WORK +0x104 (the talk-window / balloon kind; 7 initially).
    fn fun_0215a654(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::scrcmd_work::work_104(self, ctx)
    }
    /// C: `FUN_0215A660` (`scrcmd_work::set_work_104`): u8 SCRCMD WORK +0x104 = `v`.
    fn fun_0215a660(&mut self, ctx: u32, v: u32)
    where
        Self: Sized,
    {
        crate::scrcmd_work::set_work_104(self, ctx, v)
    }
    /// C: `FUN_0215A66C` (`scrcmd_work::work_105`): u8 SCRCMD WORK +0x105 (the balloon placement kind).
    fn fun_0215a66c(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::scrcmd_work::work_105(self, ctx)
    }
    /// C: `FUN_0215A678` (`scrcmd_work::set_work_105`): u8 SCRCMD WORK +0x105 = `v`.
    fn fun_0215a678(&mut self, ctx: u32, v: u32)
    where
        Self: Sized,
    {
        crate::scrcmd_work::set_work_105(self, ctx, v)
    }
    /// C: `FUN_0215A77C(ctx, x, y, cursor, flag6, right_align, var, wordset, msgdata)` (`scrcmd_work::menu_begin`;
    /// `ScriptHost::menu_start` begins here).
    fn fun_0215a77c(
        &mut self,
        ctx: u32,
        x: u32,
        y: u32,
        cursor: u32,
        flag6: u32,
        right_align: u32,
        var: u32,
        wordset: u32,
        msgdata: u32,
    );
    /// C: `FUN_0215A7F0(ctx, msg, help, param, strbuf1, strbuf2)` (`scrcmd_work::menu_add`): one menu item.
    fn fun_0215a7f0(&mut self, ctx: u32, msg: u32, help: u32, param: u32, s1: u32, s2: u32);
    /// C: `FUN_0215A864(ctx)` (`scrcmd_work::menu_open`): open the menu.
    fn fun_0215a864(&mut self, ctx: u32);
    /// C: `FUN_0215A954(ctx)` (`scrcmd_work::menu_poll`): 1 once the menu closed (`ScriptHost::menu_result`).
    fn fun_0215a954(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A990(ctx)` (`scrcmd_work::menu_poll_x`): the same, X closes too.
    fn fun_0215a990(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02158F58` (`script_sys::sw_field_block`): refresh the SW field block and return its address (word 0 = the
    /// message-window system `FUN_02188C54(field map)`, word 1 = the field map).
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F74` (`script_sys::sw_wordset`): the WORDSET (SW +0x2C).
    fn fun_02158f74(&mut self, sw: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::sw_wordset(self, sw)
    }
    /// C: `FUN_02158F78` (`script_sys::sw_strbuf1`): STRBUF 1 (SW +0x30).
    fn fun_02158f78(&mut self, sw: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::sw_strbuf1(self, sw)
    }
    /// C: `FUN_02158F7C` (`script_sys::sw_strbuf2`): STRBUF 2 (SW +0x34).
    fn fun_02158f7c(&mut self, sw: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::sw_strbuf2(self, sw)
    }
    /// C: `FUN_02158F80` (`script_sys::sw_set_38`): SW +0x38 = `v` (the object of commands 579 / 68).
    fn fun_02158f80(&mut self, sw: u32, v: u32)
    where
        Self: Sized,
    {
        crate::script_sys::sw_set_38(self, sw, v)
    }
    /// C: `FUN_02158F84` (`script_sys::sw_get_38`): SW +0x38.
    fn fun_02158f84(&mut self, sw: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::sw_get_38(self, sw)
    }
    /// C: `FUN_02158F90` (`script_sys::sw_mmdl`): the field object the script talks to (0 = none).
    fn fun_02158f90(&mut self, sw: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::sw_mmdl(self, sw)
    }
    /// C: `FUN_02158FE0(sw, gamedata, var)` (`script_sys` var address): address of var `var`.
    fn fun_02158fe0(&mut self, sw: u32, gamedata: u32, var: u32) -> u32;
    /// C: `FUN_02159144` (`script_sys::proc_mask_set`).
    fn fun_02159144(&mut self, bit: u32)
    where
        Self: Sized,
    {
        crate::script_sys::proc_mask_set(self, bit)
    }
    /// C: `FUN_02159168` (`script_sys::proc_mask_clear`).
    fn fun_02159168(&mut self, bit: u32)
    where
        Self: Sized,
    {
        crate::script_sys::proc_mask_clear(self, bit)
    }
    /// C: `FUN_02159190` (thunk to `script_sys::proc_mask_test`, `0x02159198`).
    fn fun_02159190(&mut self, bit: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::proc_mask_test(self, bit)
    }

    // ---- main
    /// C: `FUN_020056BC(x)` (main; command 55: the icon resource of the window; UNVERIFIED).
    fn fun_020056bc(&mut self, x: u32) -> u32;
    /// C: `FUN_020061E4(se)` (main): play a sound effect (`ScriptHost::play_se`).
    fn fun_020061e4(&mut self, se: u32);
    /// C: `FUN_02008550(x)` (main; command 72 picks its second message when nonzero; UNVERIFIED: player gender).
    fn fun_02008550(&mut self, x: u32) -> u32;
    /// C: `FUN_0200C274(x)` (main; the info board's number; UNVERIFIED).
    fn fun_0200c274(&mut self, x: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020127A4(gamedata)` (main): the player record.
    fn fun_020127a4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F08(gamedata)` (main; info board).
    fn fun_02012f08(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201D7EC(a, b, c, d, heap)` (main; command 579's window object).
    fn fun_0201d7ec(&mut self, a: u32, b: u32, c: u32, d: u32, heap: u32) -> u32;
    /// C: `FUN_0201D83C(x)` (main): free that object.
    fn fun_0201d83c(&mut self, x: u32);
    /// C: `FUN_0201EFA4(wordset, slot, value, digits, a, b)` (main, black-text: put a number into a WORDSET slot).
    fn fun_0201efa4(&mut self, wordset: u32, slot: u32, value: u32, digits: u32, a: u32, b: u32);
    /// C: `FUN_0201F250(wordset, dst, src)` (main, black-text wordset expand).
    fn fun_0201f250(&mut self, wordset: u32, dst: u32, src: u32);
    /// C: `FUN_0202A310(a, b, strbuf, heap)` (main; command 135's text; UNVERIFIED: a trainer message).
    fn fun_0202a310(&mut self, a: u32, b: u32, strbuf: u32, heap: u32);
    /// C: `FUN_0202EE30(res, window, a, b, heap)` (main): create the window icon (command 55).
    fn fun_0202ee30(&mut self, res: u32, window: u32, a: u32, b: u32, heap: u32) -> u32;
    /// C: `FUN_0202F038(obj)` (main): delete the window icon object.
    fn fun_0202f038(&mut self, obj: u32);
    /// C: `FUN_020362DC()` (main): pad bits (`ScriptHost::pad_held`).
    fn fun_020362dc(&mut self) -> u32;
    /// C: `FUN_020457B0(len, heap)` (main, black-text `STRBUF_Create`).
    fn fun_020457b0(&mut self, len: u32, heap: u32) -> u32;
    /// C: `FUN_02045808(strbuf)` (main, black-text `STRBUF_Delete`).
    fn fun_02045808(&mut self, strbuf: u32);
    /// C: `FUN_02045B38(a, type, msg, heap)` (main, `MsgData_Create`).
    fn fun_02045b38(&mut self, a: u32, ty: u32, msg: u32, heap: u32) -> u32;
    /// C: `FUN_02045C04(msgdata)` (main, `MsgData_Delete`).
    fn fun_02045c04(&mut self, msgdata: u32);
    /// C: `FUN_02045CAC(msgdata, msg, strbuf)` (main, `MsgData` get-string, black-text).
    fn fun_02045cac(&mut self, msgdata: u32, msg: u32, strbuf: u32);
    /// C: `FUN_02048AD0(camera)` (main, camera; UNVERIFIED).
    fn fun_02048ad0(&mut self, camera: u32);
    /// C: `FUN_02048AE8(camera, &v)` (main): a camera vector (UNVERIFIED: the target).
    fn fun_02048ae8(&mut self, camera: u32, out: &mut [u8; 12]);
    /// C: `FUN_02048B0C(camera, &v)` (main): a camera vector (UNVERIFIED: the up vector).
    fn fun_02048b0c(&mut self, camera: u32, out: &mut [u8; 12]);
    /// C: `FUN_02048B30(camera, &v)` (main): a camera vector (UNVERIFIED: the position).
    fn fun_02048b30(&mut self, camera: u32, out: &mut [u8; 12]);
    /// C: `FUN_0206DF3C(&pos, &y, &x)` (main): project a world position to the screen (UNVERIFIED).
    fn fun_0206df3c(&mut self, pos: &[u8; 12], y: &mut [u8; 4], x: &mut [u8; 4]);
    /// C: `FUN_0207B0A4(&m, &out)` (main, MTX: invert a 4x3 matrix; UNVERIFIED).
    fn fun_0207b0a4(&mut self, m: &[u8; 0x30], out: &mut [u8; 0x30]);
    /// C: `FUN_0207B51C(&v, &m, &out)` (main, MTX: transform a vector; UNVERIFIED).
    fn fun_0207b51c(&mut self, v: &[u8; 12], m: &[u8; 0x30], out: &mut [u8; 12]);
    /// C: `FUN_0207BCA0(&a, &b, &c, &m)` (main, MTX look-at; UNVERIFIED).
    fn fun_0207bca0(&mut self, a: &[u8; 12], b: &[u8; 12], c: &[u8; 12], out: &mut [u8; 0x30]);
    /// C: `FUN_0207CA00(&a, &b, &out)` (main, VEC subtract; UNVERIFIED).
    fn fun_0207ca00(&mut self, a: &[u8; 12], b: &[u8; 12], out: &mut [u8; 12]);
    /// C: `FUN_0207CAE8(&a, &b, &out)` (main, VEC cross product; UNVERIFIED).
    fn fun_0207cae8(&mut self, a: &[u8; 12], b: &[u8; 12], out: &mut [u8; 12]);
    /// C: `FUN_0207CCA4(&a, &out)` (main, VEC normalize; UNVERIFIED).
    fn fun_0207cca4(&mut self, a: &[u8; 12], out: &mut [u8; 12]);
    /// C: `FUN_02082B60(&a, &out)` (main: copy a 4x3 matrix; UNVERIFIED).
    fn fun_02082b60(&mut self, a: &[u8; 0x30], out: &mut [u8; 0x30]);

    // ---- overlay 21 (field)
    /// C: `FUN_0216D588(mmdl)` (overlay 21): the object's OBJ id.
    fn fun_0216d588(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216D5F8(mmdl)` (overlay 21; called before the position is read; UNVERIFIED).
    fn fun_0216d5f8(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216D88C(mmdl, &pos)` (overlay 21): the object's position (3 fx32 words).
    fn fun_0216d88c(&mut self, mmdl: u32, out: &mut [u8; 12]);
    /// C: `FUN_0216DE94(mmdls, obj)` (overlay 21): the field object with OBJ id `obj`, 0 if none.
    fn fun_0216de94(&mut self, mmdls: u32, obj: u32) -> u32;
    /// C: `FUN_02188C54(field map)` (overlay 21): the message-window system (msgbg).
    fn fun_02188c54(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C58(field map)` (overlay 21): the field camera.
    fn fun_02188c58(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C84(field map)` (overlay 21): the field heap.
    fn fun_02188c84(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CB4(field map)` (overlay 21): the player object.
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D48(field map)` (overlay 21): the info board, 0 if none.
    fn fun_02188d48(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D4C(field map, board)` (overlay 21): set the info board.
    fn fun_02188d4c(&mut self, fm: u32, board: u32);
    /// C: `FUN_0218EC2C(camera)` (overlay 21): the camera's NNS camera (UNVERIFIED).
    fn fun_0218ec2c(&mut self, camera: u32) -> u32;
    /// C: `FUN_021904DC(msgbg, on)` (overlay 21, field_msgbg.c; command 51's flag).
    fn fun_021904dc(&mut self, msgbg: u32, on: u32);
    /// C: `FUN_02190708(msgbg, msgdata, x, y, w, h)` (field_msgbg.c): create the info board.
    fn fun_02190708(&mut self, msgbg: u32, msgdata: u32, x: u32, y: u32, w: u32, h: u32) -> u32;
    /// C: `FUN_02190728(board)` (field_msgbg.c): delete the info board.
    fn fun_02190728(&mut self, board: u32);
    /// C: `FUN_02190758(board, x, y, text)` (field_msgbg.c): print into the info board.
    fn fun_02190758(&mut self, board: u32, x: u32, y: u32, text: u32);
    /// C: `FUN_02190788(board)` (field_msgbg.c): clear the info board.
    fn fun_02190788(&mut self, board: u32);
    /// C: `FUN_021909BC(menu)` (field_msgbg.c): close a list / yes-no menu.
    fn fun_021909bc(&mut self, menu: u32);
    /// C: `FUN_02190BE4(msgbg, 0)` (field_msgbg.c): open the yes / no menu (`ScriptHost::yes_no_start`).
    fn fun_02190be4(&mut self, msgbg: u32, arg: u32) -> u32;
    /// C: `FUN_02190C78(menu)` (field_msgbg.c): poll the yes / no menu: 0 yes, 2 still open, other no
    /// (`ScriptHost::yes_no_result`).
    fn fun_02190c78(&mut self, menu: u32) -> u32;
    /// C: `FUN_02190FA0(msgbg, msgdata, style)` (field_msgbg.c): create the talk window (`ScriptHost::message_start`, talk).
    fn fun_02190fa0(&mut self, msgbg: u32, msgdata: u32, style: u32) -> u32;
    /// C: `FUN_02191008(window)`: delete the talk window (`ScriptHost::message_close`).
    fn fun_02191008(&mut self, window: u32);
    /// C: `FUN_02191084(window, x, y, text)`: print streamed into the talk window (command 52).
    fn fun_02191084(&mut self, window: u32, x: u32, y: u32, text: u32);
    /// C: `FUN_021910C0(window)`: 1 once the talk window finished printing (`ScriptHost::message_done`).
    fn fun_021910c0(&mut self, window: u32) -> u32;
    /// C: `FUN_02191134(window)`: talk window, waiting for a key (`ScriptHost::message_cursor`).
    fn fun_02191134(&mut self, window: u32);
    /// C: `FUN_02191164(window)`: clear the talk window for the next message.
    fn fun_02191164(&mut self, window: u32);
    /// C: `FUN_02191184(window, x, y, text)`: put the text at once (command 53, `PrintMode::Instant`).
    fn fun_02191184(&mut self, window: u32, x: u32, y: u32, text: u32);
    /// C: `FUN_021911B4(window)`: the talk window's BG window (command 55's icon anchor; UNVERIFIED).
    fn fun_021911b4(&mut self, window: u32) -> u32;
    /// C: `FUN_0219129C(msgbg, tail, &pos, text, a, b)`: create a balloon window at the window position `pos` (3 words;
    /// `ScriptHost::message_start`, balloon).
    fn fun_0219129c(&mut self, msgbg: u32, tail: u32, pos: u32, text: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_021912D8(window)`: start closing the balloon (animated).
    fn fun_021912d8(&mut self, window: u32);
    /// C: `FUN_021912F8(window)`: 1 once the balloon closed (`ScriptHost::message_closed`).
    fn fun_021912f8(&mut self, window: u32) -> u32;
    /// C: `FUN_0219131C(window)`: delete the balloon at once.
    fn fun_0219131c(&mut self, window: u32);
    /// C: `FUN_02191348(window, text)`: the next page of an open balloon.
    fn fun_02191348(&mut self, window: u32, text: u32);
    /// C: `FUN_02191388(window)`: 1 once the balloon finished printing (`ScriptHost::message_done`).
    fn fun_02191388(&mut self, window: u32) -> u32;
    /// C: `FUN_021914CC(window)`: the balloon, waiting for a key (`ScriptHost::message_cursor`).
    fn fun_021914cc(&mut self, window: u32);
    /// C: `FUN_02191558(msgbg, a, kind, b, style, c, d)`: create the system window (`ScriptHost::message_start`, system).
    fn fun_02191558(
        &mut self,
        msgbg: u32,
        a: u32,
        kind: u32,
        b: u32,
        style: u32,
        c: u32,
        d: u32,
    ) -> u32;
    /// C: `FUN_021915B4(window)`: delete the system window.
    fn fun_021915b4(&mut self, window: u32);
    /// C: `FUN_021915EC(window)`: clear the system window for the next message.
    fn fun_021915ec(&mut self, window: u32);
    /// C: `FUN_021916A8(window, x, y, text)`: print streamed into the system window.
    fn fun_021916a8(&mut self, window: u32, x: u32, y: u32, text: u32);
    /// C: `FUN_021916E0(window)`: 1 once the system window finished printing.
    fn fun_021916e0(&mut self, window: u32) -> u32;
    /// C: `FUN_02191794(window)`: the system window, waiting for a key.
    fn fun_02191794(&mut self, window: u32);
    /// C: `FUN_02191830(msgbg, text, &w, &h)`: the size of a text in the sized info window (command 58; the second size
    /// byte is preset to 2).
    fn fun_02191830(&mut self, msgbg: u32, text: u32, w: &mut u8, h: &mut u8);
    /// C: `FUN_021918DC(msgbg, text, a, x, y, w, h)`: open the sized info window (command 58).
    fn fun_021918dc(&mut self, msgbg: u32, text: u32, a: u32, x: u32, y: u32, w: u32, h: u32);
    /// C: `FUN_0219196C(msgbg, v)` (command 59).
    fn fun_0219196c(&mut self, msgbg: u32, v: u32);
    /// C: `FUN_02191978(msgbg)`: close the sized info window.
    fn fun_02191978(&mut self, msgbg: u32);
    /// C: `FUN_021919AC(msgbg)`: 0 once the sized info window is gone (command 59).
    fn fun_021919ac(&mut self, msgbg: u32) -> u32;
    /// C: `FUN_02191A04(msgbg, a)`: create the window of command 67.
    fn fun_02191a04(&mut self, msgbg: u32, a: u32) -> u32;
    /// C: `FUN_02191A10(msgbg, a, x)`: create the window of command 579.
    fn fun_02191a10(&mut self, msgbg: u32, a: u32, x: u32) -> u32;
    /// C: `FUN_02191B14(window)`: delete the window of commands 67 / 579.
    fn fun_02191b14(&mut self, window: u32);
    /// C: `FUN_02191BE8(window, text)`: print into it; 1 once done.
    fn fun_02191be8(&mut self, window: u32, text: u32) -> u32;
    /// C: `FUN_02192570(msgbg, a, x, y, w, h)`: create the window of command 69.
    fn fun_02192570(&mut self, msgbg: u32, a: u32, x: u32, y: u32, w: u32, h: u32) -> u32;
    /// C: `FUN_02192628(window)`: delete the window of command 69.
    fn fun_02192628(&mut self, window: u32);
    /// C: `FUN_0219269C(window, x, y, text)`: print into it.
    fn fun_0219269c(&mut self, window: u32, x: u32, y: u32, text: u32);
    /// C: `FUN_021926D8(window)`: 1 once it finished printing.
    fn fun_021926d8(&mut self, window: u32) -> u32;
    /// C: `FUN_0219270C(msgbg, text, a)`: the width of a text (u16).
    fn fun_0219270c(&mut self, msgbg: u32, text: u32, a: u32) -> u32;
    /// C: `FUN_0219272C(msgbg, text, a)`: the height of a text (u16).
    fn fun_0219272c(&mut self, msgbg: u32, text: u32, a: u32) -> u32;
    /// C: `FUN_021A3074(player)` (overlay 21): the player's field object.
    fn fun_021a3074(&mut self, player: u32) -> u32;
}

fn rd(b: &[u8; 12], i: usize) -> u32 {
    u32::from_le_bytes(b[4 * i..4 * i + 4].try_into().unwrap())
}

fn wr(b: &mut [u8; 12], i: usize, v: u32) {
    b[4 * i..4 * i + 4].copy_from_slice(&v.to_le_bytes());
}

fn vec_bytes(v: [u32; 3]) -> [u8; 12] {
    let mut b = [0u8; 12];
    for (i, w) in v.iter().enumerate() {
        wr(&mut b, i, *w);
    }
    b
}

fn read12<H: TalkHooks>(h: &mut H, a: u32) -> [u8; 12] {
    let mut b = [0u8; 12];
    for i in 0..3 {
        wr(&mut b, i, h.r32(a + 4 * i as u32));
    }
    b
}

fn write12<H: TalkHooks>(h: &mut H, a: u32, b: &[u8; 12]) {
    for i in 0..3 {
        h.w32(a + 4 * i as u32, rd(b, i));
    }
}

/// The next script byte (`*PC++` of the VM; the menu / window commands read some operands as bytes).
fn script_byte<H: TalkHooks>(h: &mut H, vm: u32) -> u32 {
    let pc = h.r32(vm + crate::vm::VM_PC);
    let b = h.r8(pc) as u32;
    h.w32(vm + crate::vm::VM_PC, pc.wrapping_add(1));
    b
}

// ---- yes / no and choice menus

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

/// C: `FUN_overlay_d_21__021b010c`: create the yes / no event: `FUN_020122C0(gamesys, 0, `[`EVENT_YES_NO`]`, 0xC)` with work
/// `{ctx, var}`; returns the event.
pub fn yes_no_event_new<H: TalkHooks>(h: &mut H, gamesys: u32, ctx: u32, var: u32) -> u32 {
    let ev = h.fun_020122c0(gamesys, 0, EVENT_YES_NO, 0xC);
    let w = h.fun_020124e8(ev);
    h.w32(w, ctx);
    h.w32(w + 4, var);
    ev
}

/// C: `FUN_overlay_d_21__021b0130`: command 71 (YES_NO): the result var = the next operand; the yes / no event runs as the
/// script's child event; yield.
pub fn cmd_71<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = yes_no_event_new(h, gs, ctx, var);
    h.fun_0215897c(sw, ev);
    1
}

/// C: unlabeled routine at `0x021B015C`: the GMEVENT function of commands 176 / 177 (`work` = `{ctx, x_allowed}`). Sequence
/// 0: an 8-frame countdown (sequence 1 runs in the same call); 1: once it is done the menu opens (`FUN_0215A864`); 2: done (1)
/// once the menu poll (`FUN_0215A990` with X allowed, else `FUN_0215A954`) says 1. Other sequences: 0.
pub fn menu_event<H: TalkHooks>(h: &mut H, _event: u32, seq: u32, work: u32) -> u32 {
    let s = h.r32(seq);
    if s > 2 {
        return 0;
    }
    if s == 2 {
        let x = h.r32(work + 4);
        let ctx = h.r32(work);
        let r = if x != 0 {
            h.fun_0215a990(ctx)
        } else {
            h.fun_0215a954(ctx)
        };
        return (r == 1) as u32;
    }
    if s == 0 {
        let ctx = h.r32(work);
        h.fun_0215a684(ctx, YES_NO_DELAY);
        let n = h.r32(seq).wrapping_add(1);
        h.w32(seq, n);
    }
    let ctx = h.r32(work);
    if h.fun_0215a690(ctx) == 1 {
        let ctx = h.r32(work);
        h.fun_0215a864(ctx);
        let n = h.r32(seq).wrapping_add(1);
        h.w32(seq, n);
    }
    0
}

/// C: `FUN_overlay_d_21__021b01b8`: create the menu event: `FUN_020122C0(gamesys, 0, `[`EVENT_MENU`]`, 8)` with work `{ctx,
/// x_allowed}`; returns the event.
pub fn menu_event_new<H: TalkHooks>(h: &mut H, gamesys: u32, ctx: u32, x_allowed: u32) -> u32 {
    let ev = h.fun_020122c0(gamesys, 0, EVENT_MENU, 8);
    let w = h.fun_020124e8(ev);
    h.w32(w, ctx);
    h.w32(w + 4, x_allowed);
    ev
}

/// Body shared by commands 173 / 174 / 178 (not a function of the original: each command has its own copy): x, y = script
/// bytes, cursor = value, flag6 = script byte, the result var's address from its u16 id (`FUN_02158FE0`); then the menu
/// begins (`FUN_0215A77C(ctx, x, y, cursor, flag6, right_align, var, WORDSET, msgdata)`); yield.
fn menu_begin_cmd<H: TalkHooks>(
    h: &mut H,
    vm: u32,
    ctx: u32,
    own_msgdata: bool,
    right_align: u32,
) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = script_byte(h, vm);
    let y = script_byte(h, vm);
    let cursor = h.fun_02159b10(vm, ctx);
    let flag6 = script_byte(h, vm);
    let id = h.fun_02011330(vm);
    let var = h.fun_02158fe0(sw, gd, id);
    let ws = h.fun_02158f74(sw);
    let md = if own_msgdata { h.fun_0215a60c(ctx) } else { 0 };
    h.fun_0215a77c(ctx, x, y, cursor, flag6, right_align, var, ws, md);
    1
}

/// C: `FUN_overlay_d_21__021b01dc`: command 173: begin a choice menu with the default message data (`msgdata` 0); yield.
pub fn cmd_173<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    menu_begin_cmd(h, vm, ctx, false, 0)
}

/// C: `sub_021b024c`: command 174: begin a choice menu with the script's message data; yield.
pub fn cmd_174<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    menu_begin_cmd(h, vm, ctx, true, 0)
}

/// C: `sub_021b02c8`: command 178: as command 174, right-aligned (MENU +7 = 1); yield.
pub fn cmd_178<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    menu_begin_cmd(h, vm, ctx, true, 1)
}

/// C: `sub_021b0344`: command 175 (menu item): msg, help = u16 operands, param = value;
/// `FUN_0215A7F0(ctx, msg, help, param, STRBUF1, STRBUF2)`; 0.
pub fn cmd_175<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let msg = h.fun_02011330(vm);
    let help = h.fun_02011330(vm);
    let param = h.fun_02159b10(vm, ctx);
    let s1 = h.fun_02158f78(sw);
    let s2 = h.fun_02158f7c(sw);
    h.fun_0215a7f0(ctx, msg, help, param, s1, s2);
    0
}

/// C: `FUN_overlay_d_21__021b0394`: command 176: open the menu and wait (the menu event without X as the child event); yield.
pub fn cmd_176<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = menu_event_new(h, gs, ctx, 0);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `FUN_overlay_d_21__021b03bc`: command 177: the same body as command 176 (X not allowed either); yield.
pub fn cmd_177<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = menu_event_new(h, gs, ctx, 0);
    h.fun_0215897c(sw, ev);
    1
}

// ---- the talk window

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

/// C: `sub_021b04c0`: the wait callback of command 52: done (1) once the talk window finished printing (`FUN_021910C0` == 1).
pub fn wait_talk<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let w = h.fun_0215a61c(ctx);
    (h.fun_021910c0(w) == 1) as u32
}

/// C: `sub_021b04d8`: command 52 (message, streamed): msg = value, kind = u16 operand & 0xFF; the talk window opens
/// ([`talk_open`]), the expanded text ([`message_text`]) starts printing (`FUN_02191084(window, 0, 0, text)`); wait on
/// [`WAIT_TALK`]; yield.
pub fn cmd_52<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let msg = h.fun_02159b10(vm, ctx);
    let kind = h.fun_02011330(vm) & 0xFF;
    talk_open(h, ctx, kind);
    let text = message_text(h, ctx, sw, msg);
    let w = h.fun_0215a61c(ctx);
    h.fun_02191084(w, 0, 0, text);
    h.fun_020113d0(vm, WAIT_TALK);
    1
}

/// C: `FUN_overlay_d_21__021b0530`: command 53 (message, all at once): msg = value, kind = u16 operand & 0xFF; the talk
/// window opens, the message is expanded into STRBUF 1 and put at once (`FUN_02191164` + `FUN_02191184(window, 0, 0,
/// STRBUF1)`); 0.
pub fn cmd_53<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let s1 = h.fun_02158f78(sw);
    let msg = h.fun_02159b10(vm, ctx);
    let kind = h.fun_02011330(vm) & 0xFF;
    talk_open(h, ctx, kind);
    let md = h.fun_0215a60c(ctx);
    let ws = h.fun_02158f74(sw);
    let s2 = h.fun_02158f7c(sw);
    h.fun_02045cac(md, msg, s2);
    h.fun_0201f250(ws, s1, s2);
    let w = h.fun_0215a61c(ctx);
    h.fun_02191164(w);
    h.fun_02191184(w, 0, 0, s1);
    0
}

/// C: `sub_021b05ac`: command 55 (window icon): v = value; with the talk window open the old icon (SCRCMD WORK +0x100) is
/// deleted and, for v == 0, a new one is created on the window (`FUN_0202EE30(FUN_020056BC(a), a = FUN_021911B4(window),
/// 0xF, 0x10, field heap)`); 0.
pub fn cmd_55<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let heap = h.fun_02188c84(fm);
    let v = h.fun_02159b10(vm, ctx);
    if h.fun_02159190(BIT_TALK) == 0 {
        return 0;
    }
    let icon = h.fun_0215a630(ctx);
    if icon != 0 {
        h.fun_0202f038(icon);
        h.fun_0215a624(ctx, 0);
    }
    if v == 0 {
        let w = h.fun_0215a61c(ctx);
        let a = h.fun_021911b4(w);
        let res = h.fun_020056bc(a);
        let obj = h.fun_0202ee30(res, a, 0xF, 0x10, heap);
        h.fun_0215a624(ctx, obj);
    }
    0
}

/// C: `FUN_overlay_d_21__021b0624`: command 54: close the talk window ([`talk_close`]); 0.
pub fn cmd_54<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    talk_close(h, ctx);
    0
}

/// C: `sub_021b0630`: the init proc of mask bit 1 (`task`, its ctx at +4): [`talk_close`]; 1.
pub fn init_talk<H: TalkHooks>(h: &mut H, task: u32, _x: u32) -> u32 {
    let ctx = h.r32(task + 4);
    talk_close(h, ctx);
    1
}

// ---- the info board (message file 0x9E)

/// C: `sub_021b063c`: command 64 (info board): a, b = values; a new board `FUN_02190708(msgbg, msgdata(0x9E), (a - 9) &
/// 0xFFFF, b, 9, 2)` replaces the old one (`FUN_02190728`); WORDSET slot 2 = `FUN_0200C274(FUN_02012F08(GAMEDATA))` (7
/// digits); message 1 of file 0x9E, expanded, is printed into it; 0.
pub fn cmd_64<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    h.fun_020127a4(gd);
    let fm = h.fun_0201210c(gs);
    let msgbg = h.fun_02188c54(fm);
    let old = h.fun_02188d48(fm);
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let s2 = h.fun_02158f7c(sw);
    let heap = h.fun_02188c84(fm);
    let md = h.fun_02045b38(0, 3, MSG_INFO_BOARD, heap);
    let s = h.fun_020457b0(0x80, heap);
    let x = h.fun_02012f08(gd);
    let n = h.fun_0200c274(x);
    if old != 0 {
        h.fun_02190728(old);
    }
    let board = h.fun_02190708(msgbg, md, a.wrapping_sub(9) & 0xFFFF, b, 9, 2);
    h.fun_02188d4c(fm, board);
    h.fun_0201efa4(ws, 2, n, 7, 1, 1);
    h.fun_02045cac(md, 1, s2);
    h.fun_0201f250(ws, s, s2);
    h.fun_02190758(board, 0, 0, s);
    h.fun_02045808(s);
    h.fun_02045c04(md);
    0
}

/// C: `FUN_overlay_d_21__021b0730`: command 66: refresh the info board's text as command 64 does (clear `FUN_02190788`,
/// print); 0.
pub fn cmd_66<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    h.fun_020127a4(gd);
    let fm = h.fun_0201210c(gs);
    let board = h.fun_02188d48(fm);
    let sw = h.fun_0215a5f4(ctx);
    let ws = h.fun_02158f74(sw);
    let s2 = h.fun_02158f7c(sw);
    let heap = h.fun_02188c84(fm);
    let md = h.fun_02045b38(0, 3, MSG_INFO_BOARD, heap);
    let s = h.fun_020457b0(0x80, heap);
    let x = h.fun_02012f08(gd);
    let n = h.fun_0200c274(x);
    h.fun_0201efa4(ws, 2, n, 7, 1, 1);
    h.fun_02045cac(md, 1, s2);
    h.fun_0201f250(ws, s, s2);
    h.fun_02190788(board);
    h.fun_02190758(board, 0, 0, s);
    h.fun_02045808(s);
    h.fun_02045c04(md);
    0
}

/// C: `sub_021b07e0`: command 65: delete the info board if there is one (`FUN_02190728`, `FUN_02188D4C(field map, 0)`); 0.
pub fn cmd_65<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let board = h.fun_02188d48(fm);
    if board != 0 {
        h.fun_02190728(board);
        h.fun_02188d4c(fm, 0);
    }
    0
}

// ---- balloon placement (F1 row 21.43)

/// C: `FUN_overlay_d_21__021b0808`: the projected balloon offset of the object at `pos` (3 words) seen by `camera`, into `out`
/// (3 words): the camera's view direction flattened to the ground (y = 0) and normalized, crossed with [`UP`] and again with
/// the view direction, normalized, is added to the position, which is projected to the screen (`FUN_0206DF3C`). Row by the
/// screen y >> 3 (< 10: 0, < 16: 1, < 22: 2, else 3), column = (x >> 3) / 12; column 0 / 1 -> the [`OFFSETS`] record, else
/// (0, 0, 0).
pub fn balloon_project<H: TalkHooks>(h: &mut H, pos: u32, out: u32, camera: u32) {
    let mut p = read12(h, pos);
    h.fun_02048ad0(camera);
    let mut a = [0u8; 12];
    h.fun_02048b30(camera, &mut a);
    let mut b = [0u8; 12];
    h.fun_02048ae8(camera, &mut b);
    let mut c = [0u8; 12];
    h.fun_0207ca00(&a, &b, &mut c);
    let mut d = c;
    wr(&mut d, 1, 0);
    let d_in = d;
    h.fun_0207cca4(&d_in, &mut d);
    let mut e = [0u8; 12];
    h.fun_0207cae8(&d, &vec_bytes(UP), &mut e);
    let mut f = [0u8; 12];
    h.fun_0207cae8(&c, &e, &mut f);
    let f_in = f;
    h.fun_0207cca4(&f_in, &mut f);
    for i in 0..3 {
        let v = rd(&p, i).wrapping_add(rd(&f, i));
        wr(&mut p, i, v);
    }
    let mut y = [0u8; 4];
    let mut x = [0u8; 4];
    h.fun_0206df3c(&p, &mut y, &mut x);
    let y = u32::from_le_bytes(y) >> 3;
    let row = if y < 10 {
        0
    } else if y <= 15 {
        1
    } else if y < 22 {
        2
    } else {
        3
    };
    let col = (u32::from_le_bytes(x) >> 3) / 12;
    let rec = if row < 4 && col < 2 {
        OFFSETS[col as usize][row]
    } else {
        [0; 3]
    };
    write12(h, out, &vec_bytes(rec));
}

/// C: `FUN_overlay_d_21__021b090c`: transform the vector at `v` (3 words, in place) by the inverse of the camera's look-at
/// matrix (`FUN_0207BCA0` of the camera vectors `FUN_02048AE8` / `FUN_02048B0C` / `FUN_02048B30`, copied with `FUN_02082B60`,
/// inverted with `FUN_0207B0A4`, applied with `FUN_0207B51C`).
pub fn balloon_to_view<H: TalkHooks>(h: &mut H, v: u32, camera: u32) {
    let mut a = [0u8; 12];
    h.fun_02048ae8(camera, &mut a);
    let mut b = [0u8; 12];
    h.fun_02048b0c(camera, &mut b);
    let mut c = [0u8; 12];
    h.fun_02048b30(camera, &mut c);
    let mut m = [0u8; 0x30];
    h.fun_0207bca0(&a, &b, &c, &mut m);
    let mut n = [0u8; 0x30];
    h.fun_02082b60(&m, &mut n);
    let n_in = n;
    h.fun_0207b0a4(&n_in, &mut n);
    let vin = read12(h, v);
    let mut vout = [0u8; 12];
    h.fun_0207b51c(&vin, &n, &mut vout);
    write12(h, v, &vout);
}

/// C: `FUN_overlay_d_21__021b0960`: the balloon offset of placement `kind` (the low byte of the stack word; `_a3` unused) into
/// `out`: kinds 0..2 -> projected ([`balloon_project`]`(pos, out, camera)`); 3 -> [`OFFSETS`] (col 0, row 1), 4 -> (1, 1), 5 ->
/// (0, 2), 6 -> (1, 2), others (0, 0); then [`balloon_to_view`]`(out, camera)`.
pub fn balloon_place<H: TalkHooks>(
    h: &mut H,
    pos: u32,
    out: u32,
    camera: u32,
    _a3: u32,
    kind: u32,
) {
    let (col, row) = match kind as u8 {
        0..=2 => {
            balloon_project(h, pos, out, camera);
            balloon_to_view(h, out, camera);
            return;
        }
        3 => (0, 1),
        4 => (1, 1),
        5 => (0, 2),
        6 => (1, 2),
        _ => (0, 0),
    };
    write12(h, out, &vec_bytes(OFFSETS[col][row]));
    balloon_to_view(h, out, camera);
}

/// C: `FUN_021B09D8`: [`balloon_place`] with the same arguments (a veneer; it passes the stack word's low byte on).
pub fn balloon_place_veneer<H: TalkHooks>(
    h: &mut H,
    pos: u32,
    out: u32,
    camera: u32,
    a3: u32,
    kind: u32,
) {
    balloon_place(h, pos, out, camera, a3, kind & 0xFF);
}

/// The balloon tail values of balloon kind `kind` (`(a, b)`, `None` = left as zeroed; not a function of the original):
/// kind 0: (1, 2), 2: (1, -), 3: (-, 7), 4: (1, 5), 5: (-, 8), 6: (1, 6); kind 1 and kinds past 6: nothing.
fn tail_writes(kind: u32) -> (Option<u32>, Option<u32>) {
    match kind {
        0 => (Some(1), Some(2)),
        2 => (Some(1), None),
        3 => (None, Some(7)),
        4 => (Some(1), Some(5)),
        5 => (None, Some(8)),
        6 => (Some(1), Some(6)),
        _ => (None, None),
    }
}

/// The balloon tail `(a, b)` of balloon kind `kind` as [`balloon_tail`] leaves it in two separate words (not a function of
/// the original).
pub fn balloon_tail_of(kind: u32) -> (u32, u32) {
    let (a, b) = tail_writes(kind);
    (a.unwrap_or(0), b.unwrap_or(0))
}

/// C: `FUN_overlay_d_21__021b09ec`: the balloon tail of a balloon kind: `*a`, `*b` (u32) = 0, then kind 0: (1, 2), 2: a = 1,
/// 3: b = 7, 4: (1, 5), 5: b = 8, 6: (1, 6); kind 1 and kinds past 6 keep (0, 0).
pub fn balloon_tail<H: TalkHooks>(h: &mut H, kind: u32, a: u32, b: u32) {
    h.w32(a, 0);
    h.w32(b, 0);
    let (va, vb) = tail_writes(kind);
    if let Some(v) = va {
        h.w32(a, v);
    }
    if let Some(v) = vb {
        h.w32(b, v);
    }
}

/// C: `FUN_021B0A40`: which side of `mmdl` (the player) a balloon goes for a target at `pos` (3 words): 1 when the target's z
/// (word 2, signed) is less than the object's, else 2.
pub fn balloon_side<H: TalkHooks>(h: &mut H, mmdl: u32, pos: &[u8; 12]) -> u32 {
    let mut p = [0u8; 12];
    h.fun_0216d88c(mmdl, &mut p);
    if (rd(pos, 2) as i32) < (rd(&p, 2) as i32) {
        1
    } else {
        2
    }
}

// ---- balloon windows

/// C: `sub_021b0a60`: the wait callback of the balloon close: once `FUN_021912F8(window) == 1` the window field is cleared,
/// init-proc bit 2 too, done (1); else 0.
pub fn wait_balloon_close<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let w = h.fun_0215a61c(ctx);
    if h.fun_021912f8(w) == 1 {
        h.fun_0215a614(ctx, 0);
        h.fun_02159168(BIT_BALLOON);
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_21__021b0a88`: with a balloon open (bit 2) it starts closing (`FUN_021912D8`) and the VM waits on
/// [`WAIT_BALLOON_CLOSE`].
pub fn balloon_close<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) {
    if h.fun_02159190(BIT_BALLOON) != 0 {
        let w = h.fun_0215a61c(ctx);
        h.fun_021912d8(w);
        h.fun_020113d0(vm, WAIT_BALLOON_CLOSE);
    }
}

/// C: `FUN_overlay_d_21__021b0ab0`: with a balloon open it is deleted at once (`FUN_0219131C`), the window field and bit 2
/// cleared.
pub fn balloon_delete<H: TalkHooks>(h: &mut H, ctx: u32) {
    if h.fun_02159190(BIT_BALLOON) != 0 {
        let w = h.fun_0215a61c(ctx);
        h.fun_0219131c(w);
        h.fun_0215a614(ctx, 0);
        h.fun_02159168(BIT_BALLOON);
    }
}

/// C: `FUN_overlay_d_21__021b0adc`: create a balloon window `FUN_0219129C(msgbg, tail, pos, text, a5, a6)` (`text`, `a5`, `a6`
/// are stack words), record it and set init-proc bit 2.
#[allow(clippy::too_many_arguments)]
pub fn balloon_create<H: TalkHooks>(
    h: &mut H,
    ctx: u32,
    msgbg: u32,
    tail: u32,
    pos: u32,
    text: u32,
    a5: u32,
    a6: u32,
) {
    let w = h.fun_0219129c(msgbg, tail, pos, text, a5, a6);
    h.fun_0215a614(ctx, w);
    h.fun_02159144(BIT_BALLOON);
}

/// C: `FUN_overlay_d_21__021b0b08`: keep the balloon on its object: the object (OBJ id = balloon work +0) or nothing; when
/// `force == 1` or its position changed, the position is stored (B +0x10) and the offset re-placed ([`balloon_place`] with the
/// field camera and the placement kind SCRCMD WORK +0x105); the window position (B +4) = position + offset.
pub fn balloon_follow<H: TalkHooks>(h: &mut H, ctx: u32, force: u32) {
    let mmdls = h.fun_0215a5ec(ctx);
    let b = h.fun_0215a63c(ctx);
    let obj = h.r16(b + B_OBJ) as u32;
    let m = h.fun_0216de94(mmdls, obj);
    if m == 0 {
        return;
    }
    h.fun_0216d5f8(m);
    let mut p = [0u8; 12];
    h.fun_0216d88c(m, &mut p);
    let same = (0..3).all(|i| rd(&p, i) == h.r32(b + B_OBJ_POS + 4 * i as u32));
    if force == 1 || !same {
        write12(h, b + B_OBJ_POS, &p);
        let gs = h.fun_0215a5dc(ctx);
        let fm = h.fun_0201210c(gs);
        let cam = h.fun_02188c58(fm);
        let c2 = h.fun_0218ec2c(cam);
        let kind = h.fun_0215a66c(ctx);
        balloon_place(h, b + B_OBJ_POS, b + B_OFS, c2, cam, kind);
    }
    for i in 0..3 {
        let v = h
            .r32(b + B_OBJ_POS + 4 * i)
            .wrapping_add(h.r32(b + B_OFS + 4 * i));
        h.w32(b + B_POS + 4 * i, v);
    }
}

/// C: `FUN_overlay_d_21__021b0bac`: print message `msg` of `bank` (0x400 = the script's message data, else message file `bank`
/// of type 3, loaded for the call) in a balloon over object `obj` (`kind`, `a5` are stack words; `kind`'s low halfword is the
/// balloon kind, 0 = by the target's side of the player). No such object -> 0. A balloon already open gets the next page
/// (`FUN_02191348`); else the balloon work is reset (object, tail ([`balloon_tail_of`]), kind bytes SCRCMD WORK +0x104 / +0x105),
/// placed ([`balloon_follow`]) and the window created ([`balloon_create`]). 1.
pub fn balloon_print<H: TalkHooks>(
    h: &mut H,
    ctx: u32,
    obj: u32,
    bank: u32,
    msg: u32,
    kind: u32,
    a5: u32,
) -> u32 {
    let skind = kind & 0xFFFF;
    let sw = h.fun_0215a5f4(ctx);
    let md = h.fun_0215a60c(ctx);
    let fb = h.fun_02158f58(sw);
    let mmdls = h.fun_0215a5ec(ctx);
    let player = h.fun_0216de94(mmdls, OBJ_PLAYER);
    let target = h.fun_0216de94(mmdls, obj);
    if target == 0 {
        return 0;
    }
    let s1 = h.fun_02158f78(sw);
    let ws = h.fun_02158f74(sw);
    let s2 = h.fun_02158f7c(sw);
    if bank == BANK_SCRIPT {
        h.fun_02045cac(md, msg, s2);
    } else {
        let heap = h.fun_0215a5cc(ctx);
        let tmp = h.fun_02045b38(0, 3, bank, heap);
        h.fun_02045cac(tmp, msg, s2);
        h.fun_02045c04(tmp);
    }
    h.fun_0201f250(ws, s1, s2);
    let b = h.fun_0215a63c(ctx);
    if h.fun_02159190(BIT_BALLOON) != 0 {
        let w = h.fun_0215a61c(ctx);
        h.fun_02191348(w, s1);
        return 1;
    }
    for i in 0..B_SIZE {
        h.w8(b + i, 0);
    }
    h.w16(b + B_OBJ, obj as u16);
    let mut k = skind;
    if skind == 0 {
        h.fun_0215a654(ctx);
        let mut p = [0u8; 12];
        h.fun_0216d88c(target, &mut p);
        k = balloon_side(h, player, &p);
    }
    let (tail, tb) = balloon_tail_of(k);
    h.w16(b + B_TAIL, tail as u16);
    h.fun_0215a660(ctx, k & 0xFF);
    h.fun_0215a678(ctx, skind & 0xFF);
    balloon_follow(h, ctx, 1);
    let msgbg = h.r32(fb);
    balloon_create(h, ctx, msgbg, tail, b + B_POS, s1, a5, tb);
    1
}

/// C: unlabeled routine at `0x021B0CEC`: the wait callback of the balloon messages: the balloon follows its object
/// ([`balloon_follow`]`(ctx, 0)`); done (1) once `FUN_02191388(window) == 1`.
pub fn wait_balloon<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let w = h.fun_0215a61c(ctx);
    balloon_follow(h, ctx, 0);
    (h.fun_02191388(w) == 1) as u32
}

/// Wait on [`WAIT_BALLOON`] and yield when the balloon was printed (1), else 0 (shared tail of commands 60 / 61 / 72 / 73).
fn balloon_wait_tail<H: TalkHooks>(h: &mut H, vm: u32, r: u32) -> u32 {
    if r == 1 {
        h.fun_020113d0(vm, WAIT_BALLOON);
        1
    } else {
        0
    }
}

/// C: `FUN_overlay_d_21__021b0d14`: command 60 (balloon message): bank, msg, obj, kind, a5 = values; [`balloon_print`]; printed
/// -> wait on [`WAIT_BALLOON`], yield; else 0.
pub fn cmd_60<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let bank = h.fun_02159b10(vm, ctx);
    let msg = h.fun_02159b10(vm, ctx);
    let obj = h.fun_02159b10(vm, ctx);
    let kind = h.fun_02159b10(vm, ctx);
    let a5 = h.fun_02159b10(vm, ctx);
    let r = balloon_print(h, ctx, obj, bank, msg, kind, a5);
    balloon_wait_tail(h, vm, r)
}

/// C: `FUN_overlay_d_21__021b0d74`: command 61 (balloon over the object the script talks to): bank, msg, kind, a5 = values;
/// no such object (`FUN_02158F90`) -> yield 1; else [`balloon_print`] over its OBJ id (`FUN_0216D588` & 0xFF), as command 60.
pub fn cmd_61<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let target = h.fun_02158f90(sw);
    let bank = h.fun_02159b10(vm, ctx);
    let msg = h.fun_02159b10(vm, ctx);
    let kind = h.fun_02159b10(vm, ctx);
    let a5 = h.fun_02159b10(vm, ctx);
    if target == 0 {
        return 1;
    }
    let obj = h.fun_0216d588(target) & 0xFF;
    let r = balloon_print(h, ctx, obj, bank, msg, kind, a5);
    balloon_wait_tail(h, vm, r)
}

/// C: `sub_021b0df0`: command 72 (balloon message by player): bank, msg1, msg2, obj, kind, a5 = values; msg = msg2 when
/// `FUN_02008550(player record + 0x20)` is nonzero (UNVERIFIED: the player's gender), else msg1; as command 60.
pub fn cmd_72<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let bank = h.fun_02159b10(vm, ctx);
    let msg1 = h.fun_02159b10(vm, ctx);
    let msg2 = h.fun_02159b10(vm, ctx);
    let obj = h.fun_02159b10(vm, ctx);
    let kind = h.fun_02159b10(vm, ctx);
    let a5 = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let player = h.fun_020127a4(gd);
    let msg = if h.fun_02008550(player.wrapping_add(0x20)) != 0 {
        msg2
    } else {
        msg1
    };
    let r = balloon_print(h, ctx, obj, bank, msg, kind, a5);
    balloon_wait_tail(h, vm, r)
}

/// C: `sub_021b0e74`: command 73: bank, (one value skipped), msg, obj, kind, a5 = values; as command 60.
pub fn cmd_73<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let bank = h.fun_02159b10(vm, ctx);
    h.fun_02159b10(vm, ctx);
    let msg = h.fun_02159b10(vm, ctx);
    let obj = h.fun_02159b10(vm, ctx);
    let kind = h.fun_02159b10(vm, ctx);
    let a5 = h.fun_02159b10(vm, ctx);
    let r = balloon_print(h, ctx, obj, bank, msg, kind, a5);
    balloon_wait_tail(h, vm, r)
}

/// C: `sub_021b0edc`: command 62: close the balloon ([`balloon_close`]); yield.
pub fn cmd_62<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    balloon_close(h, vm, ctx);
    1
}

/// C: `FUN_overlay_d_21__021b0ee8`: the init proc of mask bit 2 (`task`, its ctx at +4): [`balloon_delete`]; 1.
pub fn init_balloon<H: TalkHooks>(h: &mut H, task: u32, _x: u32) -> u32 {
    let ctx = h.r32(task + 4);
    balloon_delete(h, ctx);
    1
}

/// C: `sub_021b0ef4`: command 135 (balloon with a text of `FUN_0202A310(a, b, STRBUF1, heap)`): a, b, obj = values; with a
/// balloon open -> yield at once; else the balloon goes over `obj` (0xFF = the player when there is no such object) on the
/// side of the player (`FUN_021A3074(FUN_02188CB4(field map))`), tail = side != 1, kind byte = side; wait on
/// [`WAIT_BALLOON`]; yield.
pub fn cmd_135<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let a = h.fun_02159b10(vm, ctx);
    let b2 = h.fun_02159b10(vm, ctx);
    let mut obj = h.fun_02159b10(vm, ctx);
    let s1 = h.fun_02158f78(sw);
    if h.fun_02159190(BIT_BALLOON) != 0 {
        return 1;
    }
    let heap = h.fun_0215a5cc(ctx);
    h.fun_0202a310(a, b2, s1, heap);
    let mmdls = h.fun_0215a5ec(ctx);
    if h.fun_0216de94(mmdls, obj) == 0 {
        obj = OBJ_PLAYER;
    }
    let b = h.fun_0215a63c(ctx);
    for i in 0..B_SIZE {
        h.w8(b + i, 0);
    }
    h.w16(b + B_OBJ, obj as u16);
    let fm = h.r32(fb + 4);
    let pl = h.fun_02188cb4(fm);
    let mmdls2 = h.fun_0215a5ec(ctx);
    let player = h.fun_021a3074(pl);
    let o = h.r16(b + B_OBJ) as u32;
    let m = h.fun_0216de94(mmdls2, o);
    let mut p = [0u8; 12];
    h.fun_0216d88c(m, &mut p);
    let side = balloon_side(h, player, &p);
    let tail = (side != 1) as u32;
    h.w16(b + B_TAIL, tail as u16);
    balloon_follow(h, ctx, 1);
    let msgbg = h.r32(fb);
    balloon_create(h, ctx, msgbg, tail, b + B_POS, s1, 0, 0);
    h.fun_0215a660(ctx, side);
    h.fun_020113d0(vm, WAIT_BALLOON);
    1
}

// ---- the system window

/// C: `sub_021b0ff0`: the wait callback of commands 56 / 74: done (1) once `FUN_021916E0(window) == 1`.
pub fn wait_system<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let w = h.fun_0215a61c(ctx);
    (h.fun_021916e0(w) == 1) as u32
}

/// C: `sub_021b1008`: the body of commands 56 / 74 (`kind` 0 / 1): msg = value, style = 1 when the next script byte is 1,
/// else 0x13; the expanded text ([`message_text`]) goes into the system window (cleared when open (bit 3), else created
/// `FUN_02191558(msgbg, 0, kind, 1, style, 0x1E, 4)`), printed streamed (`FUN_021916A8(window, 8, 0, text)`); bit 3 on; wait
/// on [`WAIT_SYSTEM`]; yield.
pub fn system_message<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32, kind: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let msg = h.fun_02159b10(vm, ctx);
    let sbyte = script_byte(h, vm);
    let text = message_text(h, ctx, sw, msg);
    let style = if sbyte == 1 { STYLE_1 } else { STYLE_13 };
    let w = if h.fun_02159190(BIT_SYSTEM) != 0 {
        let w = h.fun_0215a61c(ctx);
        h.fun_021915ec(w);
        w
    } else {
        let msgbg = h.r32(fb);
        let w = h.fun_02191558(msgbg, 0, kind, 1, style, 0x1E, 4);
        h.fun_0215a614(ctx, w);
        w
    };
    h.fun_021916a8(w, 8, 0, text);
    h.fun_02159144(BIT_SYSTEM);
    h.fun_020113d0(vm, WAIT_SYSTEM);
    1
}

/// C: `FUN_021b10a4`: close the system window when it is open (bit 3): delete it (`FUN_021915B4`), clear the window field and
/// bit 3.
pub fn system_close<H: TalkHooks>(h: &mut H, ctx: u32) {
    if h.fun_02159190(BIT_SYSTEM) != 0 {
        let w = h.fun_0215a61c(ctx);
        h.fun_021915b4(w);
        h.fun_0215a614(ctx, 0);
        h.fun_02159168(BIT_SYSTEM);
    }
}

/// C: `sub_021b10d0`: command 56: [`system_message`] with kind 0.
pub fn cmd_56<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    system_message(h, vm, ctx, 0)
}

/// C: `sub_021b10dc`: command 74: [`system_message`] with kind 1.
pub fn cmd_74<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    system_message(h, vm, ctx, 1)
}

/// C: `sub_021b10e8`: command 57: [`system_close`]; 0.
pub fn cmd_57<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    system_close(h, ctx);
    0
}

/// C: `sub_021b10f4`: the init proc of mask bit 3 (`task`, its ctx at +4): [`system_close`]; 1.
pub fn init_system<H: TalkHooks>(h: &mut H, task: u32, _x: u32) -> u32 {
    let ctx = h.r32(task + 4);
    system_close(h, ctx);
    1
}

// ---- the sized info window and the windows of commands 67 / 579 / 69

/// C: `sub_021b1100`: command 58 (sized info window): msg = u16 operand, x, y = values, a = u16 operand; the expanded message
/// (into a new 0x40-char STRBUF) is measured (`FUN_02191830(msgbg, s, &w, &h)`, h preset 2); x is pulled left so that x + w
/// <= 0x1F; y + h > 0x18 -> y = h = 4; `FUN_021918DC(msgbg, s, a, x & 0xFF, y & 0xFF, w, h)`; the STRBUF is freed; bit 5 on; 0.
pub fn cmd_58<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let msg = h.fun_02011330(vm);
    let mut x = h.fun_02159b10(vm, ctx);
    let mut y = h.fun_02159b10(vm, ctx);
    let a = h.fun_02011330(vm);
    let mut sz_h: u8 = 2;
    let heap = h.fun_0215a5cc(ctx);
    let s = h.fun_020457b0(0x40, heap);
    let md = h.fun_0215a60c(ctx);
    let ws = h.fun_02158f74(sw);
    let s2 = h.fun_02158f7c(sw);
    h.fun_02045cac(md, msg, s2);
    h.fun_0201f250(ws, s, s2);
    let mut sz_w: u8 = 0;
    let msgbg = h.r32(fb);
    h.fun_02191830(msgbg, s, &mut sz_w, &mut sz_h);
    let w = sz_w as u32;
    if (x.wrapping_add(w) as i32) > 0x1F {
        x = 0x1Fu32.wrapping_sub(w) & 0xFFFF;
    }
    if (y.wrapping_add(sz_h as u32) as i32) > 0x18 {
        y = 4;
        sz_h = 4;
    }
    let msgbg = h.r32(fb);
    h.fun_021918dc(msgbg, s, a, x & 0xFF, y & 0xFF, w, sz_h as u32);
    h.fun_02045808(s);
    h.fun_02159144(BIT_INFO);
    0
}

/// C: `FUN_overlay_d_21__021b11d8`: command 59: `FUN_0219196C(msgbg, u16 operand)`; once `FUN_021919AC(msgbg)` is 0 bit 5 is
/// cleared; 0.
pub fn cmd_59<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let v = h.fun_02011330(vm);
    let msgbg = h.r32(fb);
    h.fun_0219196c(msgbg, v);
    let msgbg = h.r32(fb);
    if h.fun_021919ac(msgbg) == 0 {
        h.fun_02159168(BIT_INFO);
    }
    0
}

/// C: `FUN_overlay_d_21__021b120c`: with the sized info window open (bit 5) it is closed (`FUN_02191978(msgbg)`) and bit 5
/// cleared.
pub fn info_close<H: TalkHooks>(h: &mut H, ctx: u32) {
    if h.fun_02159190(BIT_INFO) != 0 {
        let sw = h.fun_0215a5f4(ctx);
        let fb = h.fun_02158f58(sw);
        let msgbg = h.r32(fb);
        h.fun_02191978(msgbg);
        h.fun_02159168(BIT_INFO);
    }
}

/// C: `sub_021b1234`: the init proc of mask bit 5 (`task`, its ctx at +4): [`info_close`]; 1.
pub fn init_info<H: TalkHooks>(h: &mut H, task: u32, _x: u32) -> u32 {
    let ctx = h.r32(task + 4);
    info_close(h, ctx);
    1
}

/// C: unlabeled routine at `0x021B1240`: the wait callback of commands 67 / 579: done (1) once `FUN_02191BE8(window, STRBUF1) == 1`.
pub fn wait_window_67<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let s1 = h.fun_02158f78(sw);
    let w = h.fun_0215a61c(ctx);
    (h.fun_02191be8(w, s1) == 1) as u32
}

/// C: `FUN_overlay_d_21__021b1268`: close the window of commands 67 / 579 when it is open (bit 4): delete it
/// (`FUN_02191B14`), free SW +0x38 (`FUN_0201D83C`, if set; the field is cleared), clear the window field and bit 4.
pub fn window_67_close<H: TalkHooks>(h: &mut H, ctx: u32) {
    let sw = h.fun_0215a5f4(ctx);
    if h.fun_02159190(BIT_WINDOW_67) == 0 {
        return;
    }
    let w = h.fun_0215a61c(ctx);
    h.fun_02191b14(w);
    let x = h.fun_02158f84(sw);
    h.fun_02158f80(sw, 0);
    if x != 0 {
        h.fun_0201d83c(x);
    }
    h.fun_0215a614(ctx, 0);
    h.fun_02159168(BIT_WINDOW_67);
}

/// C: `sub_021b12b0`: command 67: msg, a = u16 operands; the window `FUN_02191A04(msgbg, a)` is recorded, the message expanded
/// into STRBUF 1 (printed by the wait callback); wait on [`WAIT_WINDOW_67`]; bit 4 on; yield.
pub fn cmd_67<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let msg = h.fun_02011330(vm);
    let a = h.fun_02011330(vm);
    let fb = h.fun_02158f58(sw);
    let md = h.fun_0215a60c(ctx);
    let msgbg = h.r32(fb);
    let w = h.fun_02191a04(msgbg, a);
    h.fun_0215a614(ctx, w);
    let s1 = h.fun_02158f78(sw);
    let ws = h.fun_02158f74(sw);
    let s2 = h.fun_02158f7c(sw);
    h.fun_02045cac(md, msg, s2);
    h.fun_0201f250(ws, s1, s2);
    h.fun_020113d0(vm, WAIT_WINDOW_67);
    h.fun_02159144(BIT_WINDOW_67);
    1
}

/// C: `FUN_overlay_d_21__021b1334`: command 579: as command 67 with an extra object `x = FUN_0201D7EC(0x17, 4, 0, 0, field
/// heap)` stored in SW +0x38 and the window `FUN_02191A10(msgbg, a, x)`; yield.
pub fn cmd_579<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let msg = h.fun_02011330(vm);
    let a = h.fun_02011330(vm);
    let fm = h.r32(fb + 4);
    let heap = h.fun_02188c84(fm);
    let fb2 = h.fun_02158f58(sw);
    let md = h.fun_0215a60c(ctx);
    let x = h.fun_0201d7ec(0x17, 4, 0, 0, heap);
    h.fun_02158f80(sw, x);
    let msgbg = h.r32(fb2);
    let w = h.fun_02191a10(msgbg, a, x);
    h.fun_0215a614(ctx, w);
    let s1 = h.fun_02158f78(sw);
    let ws = h.fun_02158f74(sw);
    let s2 = h.fun_02158f7c(sw);
    h.fun_02045cac(md, msg, s2);
    h.fun_0201f250(ws, s1, s2);
    h.fun_020113d0(vm, WAIT_WINDOW_67);
    h.fun_02159144(BIT_WINDOW_67);
    1
}

/// C: `sub_021b13e0`: command 68: [`window_67_close`]; 0.
pub fn cmd_68<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    window_67_close(h, ctx);
    0
}

/// C: `sub_021b13ec`: the init proc of mask bit 4 (`task`, its ctx at +4): [`window_67_close`]; 1.
pub fn init_window_67<H: TalkHooks>(h: &mut H, task: u32, _x: u32) -> u32 {
    let ctx = h.r32(task + 4);
    window_67_close(h, ctx);
    1
}

/// C: `FUN_021b13f8`: close the window of command 69 when it is open (bit 6): delete it (`FUN_02192628`), clear the window
/// field and bit 6.
pub fn window_69_close<H: TalkHooks>(h: &mut H, ctx: u32) {
    if h.fun_02159190(BIT_WINDOW_69) != 0 {
        let w = h.fun_0215a61c(ctx);
        h.fun_02192628(w);
        h.fun_0215a614(ctx, 0);
        h.fun_02159168(BIT_WINDOW_69);
    }
}

/// C: `sub_021b1424`: the wait callback of command 69: done (1) once `FUN_021926D8(window) == 1`.
pub fn wait_window_69<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let w = h.fun_0215a61c(ctx);
    (h.fun_021926d8(w) == 1) as u32
}

/// C: `sub_021b143c`: command 69: msg = u16 operand, x, y = script bytes, a = u16 operand; the expanded text
/// ([`message_text`]) is measured (u16 `FUN_0219270C` / `FUN_0219272C(msgbg, text, 4)`), the window `FUN_02192570(msgbg, a, x,
/// y, w, h)` recorded and printed (`FUN_0219269C(window, 4, 4, text)`); wait on [`WAIT_WINDOW_69`]; bit 6 on; yield.
pub fn cmd_69<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let msg = h.fun_02011330(vm);
    let x = script_byte(h, vm);
    let y = script_byte(h, vm);
    let a = h.fun_02011330(vm);
    let text = message_text(h, ctx, sw, msg);
    let fb = h.fun_02158f58(sw);
    let msgbg = h.r32(fb);
    let w = h.fun_0219270c(msgbg, text, 4) & 0xFFFF;
    let msgbg = h.r32(fb);
    let ht = h.fun_0219272c(msgbg, text, 4) & 0xFFFF;
    let msgbg = h.r32(fb);
    let win = h.fun_02192570(msgbg, a, x, y, w, ht);
    h.fun_0215a614(ctx, win);
    h.fun_0219269c(win, 4, 4, text);
    h.fun_020113d0(vm, WAIT_WINDOW_69);
    h.fun_02159144(BIT_WINDOW_69);
    1
}

/// C: `sub_021b14dc`: command 70: [`window_69_close`]; 0.
pub fn cmd_70<H: TalkHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    window_69_close(h, ctx);
    0
}

/// C: `sub_021b14e8`: the init proc of mask bit 6 (`task`, its ctx at +4): [`window_69_close`]; 1.
pub fn init_window_69<H: TalkHooks>(h: &mut H, task: u32, _x: u32) -> u32 {
    let ctx = h.r32(task + 4);
    window_69_close(h, ctx);
    1
}

/// C: `sub_021b14f4`: command 63: close every open message window: talk (bit 1, [`talk_close`]), system (3), the window of
/// 67 / 579 (4), of 69 (6), then the balloon (2, [`balloon_close`], which waits); yield.
pub fn cmd_63<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    if h.fun_02159190(BIT_TALK) != 0 {
        talk_close(h, ctx);
    }
    if h.fun_02159190(BIT_SYSTEM) != 0 {
        system_close(h, ctx);
    }
    if h.fun_02159190(BIT_WINDOW_67) != 0 {
        window_67_close(h, ctx);
    }
    if h.fun_02159190(BIT_WINDOW_69) != 0 {
        window_69_close(h, ctx);
    }
    if h.fun_02159190(BIT_BALLOON) != 0 {
        balloon_close(h, vm, ctx);
    }
    1
}

// ---- wait for a button, the msgbg flag, the message text

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

/// C: `sub_021b15b8`: command 75: wait on [`WAIT_BUTTON`]; yield.
pub fn cmd_75<H: TalkHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113d0(vm, WAIT_BUTTON);
    1
}

/// C: `sub_021b15c8`: command 51: u16 operand != 0 -> `FUN_021904DC(msgbg, 1)` and init-proc bit 9 on; else
/// `FUN_021904DC(msgbg, 0)` and bit 9 off; 0.
pub fn cmd_51<H: TalkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let v = h.fun_02011330(vm);
    let msgbg = h.r32(fb);
    if v != 0 {
        h.fun_021904dc(msgbg, 1);
        h.fun_02159144(BIT_MSGBG_FLAG);
    } else {
        h.fun_021904dc(msgbg, 0);
        h.fun_02159168(BIT_MSGBG_FLAG);
    }
    0
}

/// C: `FUN_overlay_d_21__021b1604`: the init proc of mask bit 9 (`task`; the SW at +8): with bit 9 set, `FUN_021904DC(msgbg,
/// 0)` and bit 9 off; 1.
pub fn init_msgbg_flag<H: TalkHooks>(h: &mut H, task: u32, _x: u32) -> u32 {
    let sw = h.r32(task + 8);
    let fb = h.fun_02158f58(sw);
    if h.fun_02159190(BIT_MSGBG_FLAG) != 0 {
        let msgbg = h.r32(fb);
        h.fun_021904dc(msgbg, 0);
        h.fun_02159168(BIT_MSGBG_FLAG);
    }
    1
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

// ---- dispatch

/// Run the command handler at `handler` (the command table word, Thumb bit allowed) when it belongs to this area; `None`
/// otherwise.
pub fn dispatch<H: TalkHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    let f: fn(&mut H, u32, u32) -> u32 = match handler & !1 {
        0x021B_15C8 => cmd_51,
        0x021B_04D8 => cmd_52,
        0x021B_0530 => cmd_53,
        0x021B_0624 => cmd_54,
        0x021B_05AC => cmd_55,
        0x021B_10D0 => cmd_56,
        0x021B_10E8 => cmd_57,
        0x021B_1100 => cmd_58,
        0x021B_11D8 => cmd_59,
        0x021B_0D14 => cmd_60,
        0x021B_0D74 => cmd_61,
        0x021B_0EDC => cmd_62,
        0x021B_14F4 => cmd_63,
        0x021B_063C => cmd_64,
        0x021B_07E0 => cmd_65,
        0x021B_0730 => cmd_66,
        0x021B_12B0 => cmd_67,
        0x021B_13E0 => cmd_68,
        0x021B_143C => cmd_69,
        0x021B_14DC => cmd_70,
        0x021B_0130 => cmd_71,
        0x021B_0DF0 => cmd_72,
        0x021B_0E74 => cmd_73,
        0x021B_10DC => cmd_74,
        0x021B_15B8 => cmd_75,
        0x021B_0EF4 => cmd_135,
        0x021B_01DC => cmd_173,
        0x021B_024C => cmd_174,
        0x021B_0344 => cmd_175,
        0x021B_0394 => cmd_176,
        0x021B_03BC => cmd_177,
        0x021B_02C8 => cmd_178,
        0x021B_1334 => cmd_579,
        _ => return None,
    };
    Some(f(h, vm, ctx))
}

/// Run the wait callback `f` (VM +0x10, Thumb bit set) when it is one of this area's; `None` otherwise.
pub fn call_wait<H: TalkHooks>(h: &mut H, f: u32, vm: u32, ctx: u32) -> Option<u32> {
    let w: fn(&mut H, u32, u32) -> u32 = match f {
        WAIT_TALK => wait_talk,
        WAIT_BALLOON_CLOSE => wait_balloon_close,
        WAIT_BALLOON => wait_balloon,
        WAIT_SYSTEM => wait_system,
        WAIT_WINDOW_67 => wait_window_67,
        WAIT_WINDOW_69 => wait_window_69,
        WAIT_BUTTON => wait_button,
        _ => return None,
    };
    Some(w(h, vm, ctx))
}

/// Run the GMEVENT function `f(event, seq, work)` when it is one of this area's ([`EVENT_YES_NO`], [`EVENT_MENU`]); `None`
/// otherwise.
pub fn call_event<H: TalkHooks>(h: &mut H, f: u32, event: u32, seq: u32, work: u32) -> Option<u32> {
    match f {
        EVENT_YES_NO => Some(yes_no_event(h, event, seq, work)),
        EVENT_MENU => Some(menu_event(h, event, seq, work)),
        _ => None,
    }
}

/// Run the init proc `f(task, task + 0xC)` (`script_sys::INIT_PROCS` slots 1..6 and 9) when it is one of this area's; `None`
/// otherwise.
pub fn call_init_proc<H: TalkHooks>(h: &mut H, f: u32, task: u32, x: u32) -> Option<u32> {
    let p: fn(&mut H, u32, u32) -> u32 = match f {
        0x021B_0631 => init_talk,
        0x021B_0EE9 => init_balloon,
        0x021B_10F5 => init_system,
        0x021B_13ED => init_window_67,
        0x021B_1235 => init_info,
        0x021B_14E9 => init_window_69,
        0x021B_1605 => init_msgbg_flag,
        _ => return None,
    };
    Some(p(h, task, x))
}

#[cfg(test)]
mod tests;
