//! F8f (fifth batch): overlay-10 script commands of F1 rows 10.24 (`calender.c`: 458 / 467..469), 10.32 (cmd 505), 10.36
//! (0x0216B86C-0x0216BF9C, an unanchored scrcmd file: the challenge records of 0x021723BC, 534 / 544..548 / 552 / 445), 10.37
//! (`namein_setup.c`: 572) and 10.40 (`trcard_sys.c`: 590..592) (37 functions, 17 command ids).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_misc10c.spec`, ported here, listed in
//! [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_misc10c`. [`COMMANDS`] maps the command ids to their
//! handlers, [`dispatch`] runs one by address and [`call_event`] the GMEVENT of command 468.
//!
//! Callees outside the area are [`Misc10cHooks`] methods named by address (vm.c, the operand readers, the ctx getters and the
//! GMEVENT calls come from [`CoreHooks`]); the overlay calls of 458 / 467..469 / 545..548 / 591 / 592 (overlays loaded
//! around the call, outside the `field` set) stay hooks. Roles are UNVERIFIED (named by address).

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_4778,
    0x0216_4910,
    0x0216_4978,
    0x0216_4994,
    0x0216_49E8,
    0x0216_A3F8,
    0x0216_B86C,
    0x0216_B8B8,
    0x0216_B91C,
    0x0216_B970,
    0x0216_B994,
    0x0216_B9D8,
    0x0216_BA08,
    0x0216_BA40,
    0x0216_BA88,
    0x0216_BAE8,
    0x0216_BB18,
    0x0216_BB70,
    0x0216_BB80,
    0x0216_BB90,
    0x0216_BBB4,
    0x0216_BBC4,
    0x0216_BBD4,
    0x0216_BC88,
    0x0216_BCE4,
    0x0216_BD1C,
    0x0216_BD54,
    0x0216_BE20,
    0x0216_BEC0,
    0x0216_BF38,
    0x0216_BF74,
    0x0216_CC30,
    0x0216_CC40,
    0x0216_F8B8,
    0x0216_F8C8,
    0x0216_F8DC,
    0x0216_F900,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (445, 0x0216_BF38),
    (458, 0x0216_4778),
    (467, 0x0216_4910),
    (468, 0x0216_4994),
    (469, 0x0216_4978),
    (505, 0x0216_A3F8),
    (534, 0x0216_B86C),
    (544, 0x0216_BD54),
    (545, 0x0216_BBD4),
    (546, 0x0216_BC88),
    (547, 0x0216_BCE4),
    (548, 0x0216_BD1C),
    (552, 0x0216_BEC0),
    (572, 0x0216_CC30),
    (590, 0x0216_F8B8),
    (591, 0x0216_F8C8),
    (592, 0x0216_F8DC),
];

/// The GMEVENT of command 468 (Thumb).
pub const EVENT_468: u32 = 0x0216_49E9;
/// The calender overlays: 458 loads 0x2F around `FUN_021F3800`, 467..469 load / unload 0x52.
pub const OVERLAY_458: u32 = 0x2F;
pub const OVERLAY_467: u32 = 0x52;
/// The field-proc flag 467 sets and 468's event clears (`FUN_02159144` / `FUN_02159168`).
pub const FIELD_PROC_FLAG: u32 = 7;
/// The event-work flag words 572 / 590 set (`FUN_0200B3A4(ew, id)`).
pub const EW_FLAG_572: u32 = 0x34;
pub const EW_FLAG_590: u32 = 0x36;
/// The smallest-record start of `FUN_0216BA88` and the per-record threshold of `FUN_0216BA40`.
pub const RECORD_MIN_START: u32 = 0xF_694E;
pub const RECORD_FULL: u32 = 0xF_423F;
/// The day count `FUN_0216BB18` answers when the clock is off by more than a day (or when the elapsed time does not fit).
pub const DAYS_UNKNOWN: u32 = 0x18;
/// ROM address of [`CHALLENGES`].
pub const CHALLENGES_ADDR: u32 = 0x0217_23BC;
/// The 8 keys of `FUN_0216BF74` (0x02172508, overlay-10 ROM).
pub const KEYS_ADDR: u32 = 0x0217_2508;
pub const KEYS: [u32; 8] = [0x0132_B4E5, 0x0132_B536, 0, 0, 0, 0, 0, 0];
/// fx32 half-tile offset of the camera positions (x << 16 + 0x8000).
pub const HALF_TILE: u32 = 0x8000;

pub const CHALLENGES: [[u8; 7]; 47] = [
    [0x01, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00],
    [0x01, 0x01, 0x00, 0xFF, 0xFF, 0x01, 0x02],
    [0x01, 0x01, 0x08, 0xFF, 0xFF, 0x01, 0x02],
    [0x01, 0x01, 0x1D, 0xFF, 0xFF, 0x01, 0x04],
    [0x01, 0x01, 0x01, 0xFF, 0xFF, 0x01, 0x04],
    [0x01, 0x01, 0x1A, 0xFF, 0xFF, 0x01, 0x04],
    [0x01, 0x01, 0x19, 0xFF, 0xFF, 0x01, 0x04],
    [0x01, 0x01, 0x02, 0xFF, 0xFF, 0x01, 0x08],
    [0x01, 0x01, 0x07, 0xFF, 0xFF, 0x01, 0x08],
    [0x01, 0x01, 0x1C, 0xFF, 0xFF, 0x01, 0x08],
    [0x01, 0x01, 0x03, 0xFF, 0xFF, 0x01, 0x08],
    [0x01, 0x01, 0x05, 0x0B, 0xFF, 0x01, 0x0C],
    [0x01, 0x03, 0x04, 0x06, 0x0E, 0x01, 0x0C],
    [0x01, 0x02, 0x0C, 0x0D, 0xFF, 0x01, 0x0C],
    [0x01, 0x02, 0x18, 0x14, 0xFF, 0x01, 0x0C],
    [0x01, 0x01, 0x14, 0xFF, 0xFF, 0x01, 0x10],
    [0x01, 0x02, 0x15, 0x16, 0xFF, 0x01, 0x10],
    [0x01, 0x01, 0x0A, 0xFF, 0xFF, 0x01, 0x10],
    [0x01, 0x02, 0x10, 0x11, 0xFF, 0x01, 0x10],
    [0x01, 0x01, 0x17, 0xFF, 0xFF, 0x01, 0x14],
    [0x01, 0x02, 0x13, 0x09, 0xFF, 0x01, 0x14],
    [0x01, 0x01, 0x12, 0xFF, 0xFF, 0x01, 0x14],
    [0x01, 0x01, 0x1B, 0xFF, 0xFF, 0x01, 0x18],
    [0x01, 0x01, 0x0F, 0xFF, 0xFF, 0x01, 0x18],
    [0x00, 0x01, 0x00, 0xFF, 0xFF, 0x05, 0x00],
    [0x00, 0x01, 0x08, 0xFF, 0xFF, 0x05, 0x00],
    [0x00, 0x01, 0x1D, 0xFF, 0xFF, 0x0A, 0x00],
    [0x00, 0x01, 0x01, 0xFF, 0xFF, 0x0A, 0x00],
    [0x00, 0x01, 0x1A, 0xFF, 0xFF, 0x0A, 0x00],
    [0x00, 0x01, 0x19, 0xFF, 0xFF, 0x0A, 0x00],
    [0x00, 0x01, 0x02, 0xFF, 0xFF, 0x14, 0x00],
    [0x00, 0x01, 0x07, 0xFF, 0xFF, 0x14, 0x00],
    [0x00, 0x01, 0x1C, 0xFF, 0xFF, 0x14, 0x00],
    [0x00, 0x01, 0x03, 0xFF, 0xFF, 0x14, 0x00],
    [0x00, 0x02, 0x05, 0x0B, 0xFF, 0x1E, 0x00],
    [0x00, 0x03, 0x04, 0x06, 0x0E, 0x1E, 0x00],
    [0x00, 0x02, 0x0C, 0x0D, 0xFF, 0x1E, 0x00],
    [0x00, 0x02, 0x18, 0x14, 0xFF, 0x1E, 0x00],
    [0x00, 0x01, 0x14, 0xFF, 0xFF, 0x28, 0x00],
    [0x00, 0x02, 0x15, 0x16, 0xFF, 0x28, 0x00],
    [0x00, 0x01, 0x0A, 0xFF, 0xFF, 0x28, 0x00],
    [0x00, 0x02, 0x10, 0x11, 0xFF, 0x28, 0x00],
    [0x00, 0x01, 0x17, 0xFF, 0xFF, 0x32, 0x00],
    [0x00, 0x02, 0x13, 0x09, 0xFF, 0x32, 0x00],
    [0x00, 0x01, 0x12, 0xFF, 0xFF, 0x32, 0x00],
    [0x00, 0x01, 0x1B, 0xFF, 0xFF, 0x64, 0x00],
    [0x00, 0x01, 0x0F, 0xFF, 0xFF, 0x64, 0x00],
];
/// Record sums are capped at this value (`FUN_0216B970`).
pub const RECORD_MAX: u32 = 999_999;

/// Callees of the scrcmd_misc10c functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the ctx getters and the GMEVENT calls come from [`CoreHooks`].
#[allow(clippy::too_many_arguments)]
pub trait Misc10cHooks: CoreHooks {
    // ---- main
    /// C: `FUN_02008530(x)`: the player's id.
    fn fun_02008530(&mut self, x: u32) -> u32;
    /// C: `FUN_0200B3A4(ew, id)`: the event-work flag word `id`.
    fn fun_0200b3a4(&mut self, ew: u32, id: u32) -> u32;
    /// C: `FUN_0200C0E8(save)`: the challenge save block S.
    fn fun_0200c0e8(&mut self, save: u32) -> u32;
    /// C: `FUN_0200C0F0(save)`: the challenge save block S (the copy `FUN_0216BB18` reads).
    fn fun_0200c0f0(&mut self, save: u32) -> u32;
    /// C: `FUN_0200C32C(S)`: the current challenge.
    fn fun_0200c32c(&mut self, s: u32) -> u32;
    /// C: `FUN_0200C34C(S, i)`: flag `i` of the current challenge.
    fn fun_0200c34c(&mut self, s: u32, i: u32) -> u32;
    /// C: `FUN_0200C37C(S)`: the 64-bit start time of the challenge.
    fn fun_0200c37c(&mut self, s: u32) -> u64;
    /// C: `FUN_0200C38C(S, k, v)`.
    fn fun_0200c38c(&mut self, s: u32, k: u32, v: u32);
    /// C: `FUN_0200D418(save)`.
    fn fun_0200d418(&mut self, save: u32) -> u32;
    /// C: `FUN_0200D468(x, i)`.
    fn fun_0200d468(&mut self, x: u32, i: u32) -> u32;
    /// C: `FUN_0200E394(save)`: the record block T.
    fn fun_0200e394(&mut self, save: u32) -> u32;
    /// C: `FUN_0200E458(T, id)` (main): record value a of `id` (UNVERIFIED).
    fn fun_0200e458(&mut self, t: u32, id: u32) -> u32;
    /// C: `FUN_0200E47C(T, id)` (main): record value b of `id` (UNVERIFIED).
    fn fun_0200e47c(&mut self, t: u32, id: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)`: the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012374(event, child)`: call a child event.
    fn fun_02012374(&mut self, event: u32, child: u32);
    /// C: `FUN_020124E4(event)`: the event's GAMESYS.
    fn fun_020124e4(&mut self, event: u32) -> u32;
    /// C: `FUN_02012934(gamedata)`: the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)`: the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EC4(gamedata)`: the event work.
    fn fun_02012ec4(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F50(gamedata)`: the fld_btl_inst_tool work slot (a pointer to the work).
    fn fun_02012f50(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02017E1C(mon, id, x)`: a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_0201AA18(party)`: the party count.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)`: party mon `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_0202F488(mon, k)`: the mon has form / move kind `k`.
    fn fun_0202f488(&mut self, mon: u32, k: u32) -> u32;
    /// C: `FUN_0202F630(mon, k)`: change it.
    fn fun_0202f630(&mut self, mon: u32, k: u32);
    /// C: `FUN_02034A5C(id)`: unload an overlay.
    fn fun_02034a5c(&mut self, id: u32);
    /// C: `FUN_02034AC4(id)`: load an overlay.
    fn fun_02034ac4(&mut self, id: u32);
    /// C: `FUN_0203F524()`: the 64-bit clock.
    fn fun_0203f524(&mut self) -> u64;
    /// C: `FUN_020891E0(&date, &time, lo, hi)`: split the 64-bit time `hi:lo` into a 12-byte date (days at +0) and a
    /// 12-byte rest (+0 / +4 / +8 compared with 0 / 1 / 1).
    fn fun_020891e0(&mut self, rest: &mut [u32; 3], days: &mut [u32; 3], lo: u32, hi: u32);
    // ---- overlays 10 / 21 / the overlays loaded around the calls
    /// C: `FUN_02159144(f)`: set field-proc flag `f`.
    fn fun_02159144(&mut self, f: u32);
    /// C: `FUN_02159168(f)`: clear field-proc flag `f`.
    fn fun_02159168(&mut self, f: u32);
    /// C: `FUN_02188CB4(fm)`: the player.
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_021A3038(player, &pos)`: the player position.
    fn fun_021a3038(&mut self, player: u32, pos: &mut [u32; 3]);
    /// C: `FUN_021D096C(gamesys)` (ov21): the fld_btl_inst_tool work.
    fn fun_021d096c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_021F3800(gamesys)` (overlay 0x2F).
    fn fun_021f3800(&mut self, gamesys: u32);
    /// C: `FUN_021F385C(fm)` (trainer-card overlay).
    fn fun_021f385c(&mut self, fm: u32);
    /// C: `FUN_021F38BC(gamesys)` (trainer-card overlay): an event.
    fn fun_021f38bc(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_021F39B8(gamesys, mode, &from, &to, frames, x)`: the camera event of command 545.
    fn fun_021f39b8(
        &mut self,
        gamesys: u32,
        mode: u32,
        from: &[u32; 3],
        to: &[u32; 3],
        frames: u32,
        x: u32,
    ) -> u32;
    /// C: `FUN_021F3AA0(gamesys, a, &pos)` (command 546).
    fn fun_021f3aa0(&mut self, gamesys: u32, a: u32, pos: &[u32; 3]);
    /// C: `FUN_021F3B40(gamesys, x)`: an event (command 548).
    fn fun_021f3b40(&mut self, gamesys: u32, x: u32) -> u32;
    /// C: `FUN_021F3B68(gamesys, x)`: an event (command 547).
    fn fun_021f3b68(&mut self, gamesys: u32, x: u32) -> u32;
    /// C: `FUN_021F5260(a, b, c, gamesys, ctx)` (overlay 0x52): the calender event.
    fn fun_021f5260(&mut self, a: u32, b: u32, c: u32, gamesys: u32, ctx: u32) -> u32;
    /// C: `FUN_021F5550(gamesys)` (overlay 0x52): a child event.
    fn fun_021f5550(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_021F558C(fm)` (overlay 0x52).
    fn fun_021f558c(&mut self, fm: u32);
}

// ------------------------------------------------------------------ calender.c, 505

/// C: `FUN_02164778` (command 458): load overlay 0x2F, `FUN_021F3800(gs)`, unload it.
pub fn cmd_458<H: Misc10cHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    h.fun_02034ac4(OVERLAY_458);
    h.fun_021f3800(gs);
    h.fun_02034a5c(OVERLAY_458);
    0
}

/// C: `sub_02164910` (command 467): three script bytes, set field-proc flag 7, load overlay 0x52 and start its event.
pub fn cmd_467<H: Misc10cHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02011330(vm) & 0xFF;
    let b = h.fun_02011330(vm) & 0xFF;
    let c = h.fun_02011330(vm);
    h.fun_02159144(FIELD_PROC_FLAG);
    h.fun_02034ac4(OVERLAY_467);
    let ev = h.fun_021f5260(a, b, c, gs, ctx);
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// C: `sub_02164978` (command 469): `FUN_021F558C(fm)`.
pub fn cmd_469<H: Misc10cHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    h.fun_021f558c(fm);
    0
}

/// C: `sub_02164994` (command 468): start [`event_468`] (no work).
pub fn cmd_468<H: Misc10cHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = h.fun_020122c0(gs, 0, EVENT_468, 0);
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// C: `sub_021649e8` (GMEVENT of command 468): seq 0 runs the child event `FUN_021F5550(gs)`; seq 1 clears field-proc flag 7,
/// unloads overlay 0x52 and ends.
pub fn event_468<H: Misc10cHooks>(h: &mut H, event: u32, seq: u32, _work: u32) -> u32 {
    let gs = h.fun_020124e4(event);
    match h.r32(seq) {
        0 => {
            let child = h.fun_021f5550(gs);
            h.fun_02012374(event, child);
            let s = h.r32(seq);
            h.w32(seq, s.wrapping_add(1));
            0
        }
        1 => {
            h.fun_02159168(FIELD_PROC_FLAG);
            h.fun_02034a5c(OVERLAY_467);
            1
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_10__0216a3f8` (command 505): create the fld_btl_inst_tool work (`FUN_021D096C`) when its slot is empty.
pub fn cmd_505<H: Misc10cHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let slot = h.fun_02012f50(gd);
    if h.r32(slot) == 0 {
        let w = h.fun_021d096c(gs);
        h.w32(slot, w);
    }
    0
}

// ------------------------------------------------------------------ row 10.36: the challenge records

/// C: `FUN_overlay_d_10__0216b86c` (command 534): the value left of the current challenge into a var: kind 0
/// [`goal_left`], kind 1 [`days_left`]; another kind stores the caller's `r5` (0 under the harness's entry state:
/// UNVERIFIED, no such challenge exists).
pub fn cmd_534<H: Misc10cHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gd);
    let s = h.fun_0200c0e8(save);
    let var = h.fun_02159ae8(vm, ctx);
    let c = h.fun_0200c32c(s);
    let v = match chal_kind(c) {
        0 => goal_left(h, save),
        1 => days_left(h, save),
        _ => 0,
    };
    h.w16(var, v);
    0
}

/// C: `FUN_overlay_d_10__0216b8b8`: the state of the current challenge into `out` (u16): kind 0: 1 = reached, 2 = not yet;
/// kind 1: 3 = time is up, 1 = every step done, 4 = not yet; another kind writes nothing.
pub fn challenge_state<H: Misc10cHooks>(h: &mut H, save: u32, out: u32) {
    let t = h.fun_0200e394(save);
    let s = h.fun_0200c0e8(save);
    let c = h.fun_0200c32c(s);
    match chal_kind(c) {
        0 => {
            let v = if goal_reached(h, t, c) { 1 } else { 2 };
            h.w16(out, v);
        }
        1 => {
            let v = if !time_running(h, save) {
                3
            } else if all_steps_done(h, save) {
                1
            } else {
                4
            };
            h.w16(out, v);
        }
        _ => {}
    }
}

/// C: `FUN_overlay_d_10__0216b91c`: the state of challenge `c` into `out` (u16): kind 0: 1 / 2 as
/// [`challenge_state`]; kind 1: 1 = every record full ([`records_full`]), 3 = not; another kind writes nothing.
pub fn challenge_state_of<H: Misc10cHooks>(h: &mut H, save: u32, c: u32, out: u32) {
    let t = h.fun_0200e394(save);
    h.fun_0200c0e8(save);
    match chal_kind(c) {
        0 => {
            let v = if goal_reached(h, t, c) { 1 } else { 2 };
            h.w16(out, v);
        }
        1 => {
            let v = if records_full(h, t, c) { 1 } else { 3 };
            h.w16(out, v);
        }
        _ => {}
    }
}

/// C: `FUN_overlay_d_10__0216b970`: `min(a + b, 999999)` of the two record values of `id` (32-bit wrapping sum, unsigned compare).
pub fn record_sum<H: Misc10cHooks>(h: &mut H, t: u32, id: u32) -> u32 {
    let a = h.fun_0200e458(t, id);
    let b = h.fun_0200e47c(t, id);
    a.wrapping_add(b).min(RECORD_MAX)
}

/// C: `FUN_overlay_d_10__0216b994`: every record of challenge `c` reaches its goal ([`record_sum`] >= the goal byte).
pub fn goal_reached<H: Misc10cHooks>(h: &mut H, t: u32, c: u32) -> bool {
    let mut ids = [0u8; 3];
    chal_ids(c, &mut ids);
    let n = chal_count(c);
    let goal = u32::from(chal_a(c));
    for &id in ids.iter().take(usize::from(n)) {
        if record_sum(h, t, u32::from(id)) < goal {
            return false;
        }
    }
    true
}

/// C: `FUN_overlay_d_10__0216b9d8`: the current challenge's day limit is not above the days elapsed ([`days_elapsed`];
/// signed compare).
pub fn time_running<H: Misc10cHooks>(h: &mut H, save: u32) -> bool {
    h.fun_0200e394(save);
    let s = h.fun_0200c0e8(save);
    let c = h.fun_0200c32c(s);
    let d = days_elapsed(h, save);
    i32::from(chal_b(c & 0xFF)) <= d as i32
}

/// C: `FUN_overlay_d_10__0216ba08`: every step flag `FUN_0200C34C(S, i)` of the current challenge is set.
pub fn all_steps_done<H: Misc10cHooks>(h: &mut H, save: u32) -> bool {
    let s = h.fun_0200c0e8(save);
    let c = h.fun_0200c32c(s);
    let n = chal_count(c);
    for i in 0..n {
        if h.fun_0200c34c(s, u32::from(i)) == 0 {
            return false;
        }
    }
    true
}

/// C: `FUN_overlay_d_10__0216ba40`: every record of challenge `c` is full ([`record_sum`] >= 999999; the goal byte is read
/// and unused).
pub fn records_full<H: Misc10cHooks>(h: &mut H, t: u32, c: u32) -> bool {
    let mut ids = [0u8; 3];
    chal_ids(c, &mut ids);
    let n = chal_count(c);
    let _ = chal_a(c);
    for &id in ids.iter().take(usize::from(n)) {
        if record_sum(h, t, u32::from(id)) < RECORD_FULL {
            return false;
        }
    }
    true
}

/// C: `FUN_overlay_d_10__0216ba88`: the goal of the current challenge minus its smallest record sum (from 0xF694E), as u16.
pub fn goal_left<H: Misc10cHooks>(h: &mut H, save: u32) -> u16 {
    let t = h.fun_0200e394(save);
    let s = h.fun_0200c0e8(save);
    let c = h.fun_0200c32c(s);
    let goal = u32::from(chal_a(c));
    let n = chal_count(c);
    let mut ids = [0u8; 3];
    chal_ids(c, &mut ids);
    let mut m = RECORD_MIN_START;
    for &id in ids.iter().take(usize::from(n)) {
        let v = record_sum(h, t, u32::from(id));
        if v < m {
            m = v;
        }
    }
    goal.wrapping_sub(m) as u16
}

/// C: `FUN_overlay_d_10__0216bae8`: the days left of the current challenge (its day limit minus [`days_elapsed`], 0 when
/// negative), as u16.
pub fn days_left<H: Misc10cHooks>(h: &mut H, save: u32) -> u16 {
    h.fun_0200e394(save);
    let s = h.fun_0200c0e8(save);
    let c = h.fun_0200c32c(s);
    let d = days_elapsed(h, save);
    let r = u32::from(chal_b(c & 0xFF)).wrapping_sub(d);
    if (r as i32) < 0 {
        0
    } else {
        r as u16
    }
}

/// C: `FUN_overlay_d_10__0216bb18`: the days since the challenge started (`FUN_020891E0` of clock - start time): 0 without a
/// challenge, 0x18 when the rest of the split time is not (0, <= 1, <= 1).
pub fn days_elapsed<H: Misc10cHooks>(h: &mut H, save: u32) -> u32 {
    let s = h.fun_0200c0f0(save);
    if h.fun_0200c32c(s) == 0 {
        return 0;
    }
    let start = h.fun_0200c37c(s);
    let now = h.fun_0203f524();
    let d = now.wrapping_sub(start);
    let mut rest = [0u32; 3];
    let mut days = [0u32; 3];
    h.fun_020891e0(&mut rest, &mut days, d as u32, (d >> 32) as u32);
    if rest[0] != 0 || rest[1] > 1 || rest[2] > 1 {
        return DAYS_UNKNOWN;
    }
    days[0] & 0xFFFF
}

/// C: `FUN_overlay_d_10__0216bb70`: the kind byte of challenge `c` (panics past the 47 records, where the original reads the
/// following ROM bytes).
pub fn chal_kind(c: u32) -> u8 {
    CHALLENGES[c as usize][0]
}

/// C: `FUN_overlay_d_10__0216bb80`: the id count of challenge `c`.
pub fn chal_count(c: u32) -> u8 {
    CHALLENGES[c as usize][1]
}

/// C: `FUN_overlay_d_10__0216bb90`: the three ids of challenge `c` into `out`.
pub fn chal_ids(c: u32, out: &mut [u8]) {
    out[..3].copy_from_slice(&CHALLENGES[c as usize][2..5]);
}

/// C: `FUN_overlay_d_10__0216bbb4`: the goal byte of challenge `c`.
pub fn chal_a(c: u32) -> u8 {
    CHALLENGES[c as usize][5]
}

/// C: `FUN_overlay_d_10__0216bbc4`: the day-limit byte of challenge `c`.
pub fn chal_b(c: u32) -> u8 {
    CHALLENGES[c as usize][6]
}

/// fx32 tile position `(a << 16) + 0x8000, b << 15, (c << 16) + 0x8000`.
fn tile_pos(a: u32, b: u32, c: u32) -> [u32; 3] {
    [
        (a << 16).wrapping_add(HALF_TILE),
        b << 15,
        (c << 16).wrapping_add(HALF_TILE),
    ]
}

/// C: `sub_0216bbd4` (command 545): the camera event between the player's position and a tile position (operand order by
/// `mode`: 0 = player -> tile, else tile -> player).
pub fn cmd_545<H: Misc10cHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let mode = h.fun_02159b10(vm, ctx);
    let x = h.fun_02159b10(vm, ctx);
    let y = h.fun_02159b10(vm, ctx);
    let z = h.fun_02159b10(vm, ctx);
    let frames = h.fun_02159b10(vm, ctx);
    let e = h.fun_02159b10(vm, ctx);
    let player = h.fun_02188cb4(fm);
    let mut p = [0u32; 3];
    h.fun_021a3038(player, &mut p);
    let q = tile_pos(x, y, z);
    let ev = if mode == 0 {
        h.fun_021f39b8(gs, mode, &p, &q, frames << 15, e)
    } else {
        h.fun_021f39b8(gs, mode, &q, &p, frames << 15, e)
    };
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// C: `FUN_overlay_d_10__0216bc88` (command 546): `FUN_021F3AA0(gs, a, tile position)`.
pub fn cmd_546<H: Misc10cHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let x = h.fun_02159b10(vm, ctx);
    let y = h.fun_02159b10(vm, ctx);
    let z = h.fun_02159b10(vm, ctx);
    let pos = tile_pos(x, y, z);
    h.fun_021f3aa0(gs, a, &pos);
    0
}

/// C: `sub_0216bce4` (command 547): the event `FUN_021F3B68(gs, x)`.
pub fn cmd_547<H: Misc10cHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let x = h.fun_02159b10(vm, ctx);
    let ev = h.fun_021f3b68(gs, x);
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// C: `sub_0216bd1c` (command 548): the event `FUN_021F3B40(gs, x)`.
pub fn cmd_548<H: Misc10cHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let x = h.fun_02159b10(vm, ctx);
    let ev = h.fun_021f3b40(gs, x);
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// The form / move kind and the move of a species for commands 544 (`(kind, move)`; `None` = not one of them): 0xF3..0xF5
/// -> (2, 0), 0xFB -> (0, 0), 0x1EC -> (7, 0), 0x287 -> (4, 0x224), 0x288 -> (5, 0x223), 0x289 -> (6, 0).
fn kind_544(species: u32) -> Option<(u32, u32)> {
    let s = species as i32;
    Some(match s {
        0xF3..=0xF5 => (2, 0),
        0xFB => (0, 0),
        0x1EC => (7, 0),
        0x287 => (4, 0x224),
        0x288 => (5, 0x223),
        0x289 => (6, 0),
        _ => return None,
    })
}

/// C: `sub_0216bd54` (command 544): look for a party mon of a species with its form / move kind ([`find_mon`]; with the
/// flag operand 0 the move test is off): found into var 2, the slot into var 1; an unknown species stores 0 into var 2.
pub fn cmd_544<H: Misc10cHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let species = h.fun_02159b10(vm, ctx);
    let flag = h.fun_02159b10(vm, ctx);
    let slot_var = h.fun_02159ae8(vm, ctx);
    let found_var = h.fun_02159ae8(vm, ctx);
    let Some((kind, mut mv)) = kind_544(species) else {
        h.w16(found_var, 0);
        return 0;
    };
    if flag == 0 {
        mv = 0;
    }
    let r = find_mon(h, party, species, mv, kind, slot_var);
    h.w16(found_var, r as u16);
    0
}

/// C: `FUN_overlay_d_10__0216be20`: the first party mon of `species` with form / move kind `kind` that does not know the
/// move `mv` (any mon when `mv` = 0): its slot into `var` (u16, 0 when none); 1 when found.
pub fn find_mon<H: Misc10cHooks>(
    h: &mut H,
    party: u32,
    species: u32,
    mv: u32,
    kind: u32,
    var: u32,
) -> u32 {
    let n = h.fun_0201aa18(party) as i32;
    h.w16(var, 0);
    let mut i = 0i32;
    while i < n {
        let mon = h.fun_0201ab28(party, i as u32);
        if h.fun_0202f488(mon, kind) != 0 && h.fun_02017e1c(mon, 5, 0) == species {
            let knows = mv != 0
                && (h.fun_02017e1c(mon, 0x36, 0) == mv
                    || h.fun_02017e1c(mon, 0x37, 0) == mv
                    || h.fun_02017e1c(mon, 0x38, 0) == mv
                    || h.fun_02017e1c(mon, 0x39, 0) == mv);
            if !knows {
                h.w16(var, i as u16);
                return 1;
            }
        }
        i += 1;
    }
    0
}

/// C: `sub_0216bec0` (command 552): change party mon `i` of species 0xF3..0xF5 (kind 2) / 0xFB (kind 0) to its form / move
/// kind when it has it (`FUN_0202F488` / `FUN_0202F630`).
pub fn cmd_552<H: Misc10cHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let species = h.fun_02159b10(vm, ctx);
    let i = h.fun_02159b10(vm, ctx);
    let kind = match species.wrapping_sub(0xF3) {
        0..=2 => 2,
        8 => 0,
        _ => return 0,
    };
    let mon = h.fun_0201ab28(party, i);
    if h.fun_0202f488(mon, kind) != 0 {
        h.fun_0202f630(mon, kind);
    }
    0
}

/// C: `sub_0216bf38` (command 445): `FUN_0200D468(FUN_0200D418(save), i)` (i = u8 operand) into a var.
pub fn cmd_445<H: Misc10cHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gd);
    let x = h.fun_0200d418(save);
    let i = h.fun_02159b10(vm, ctx) & 0xFF;
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_0200d468(x, i);
    h.w16(var, v as u16);
    0
}

/// C: `FUN_overlay_d_10__0216bf74`: for `k` < 8 (signed), `FUN_0200C38C(s, k, FUN_02008530(x) ^ KEYS[k])` (a negative `k`
/// reads before the key table in the original; the port panics: UNVERIFIED, no caller passes one).
pub fn set_key<H: Misc10cHooks>(h: &mut H, s: u32, k: u32, x: u32) {
    if (k as i32) < 8 {
        let v = h.fun_02008530(x);
        h.fun_0200c38c(s, k, v ^ KEYS[k as usize]);
    }
}

// ------------------------------------------------------------------ namein_setup.c / trcard_sys.c tails

/// C: `sub_0216cc30` (command 572): [`set_flag_572`].
pub fn cmd_572<H: Misc10cHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    set_flag_572(h, gs);
    0
}

/// C: `FUN_0216cc40`: event-work flag word 0x34 = 1.
pub fn set_flag_572<H: Misc10cHooks>(h: &mut H, gamesys: u32) {
    let gd = h.fun_020120f4(gamesys);
    let ew = h.fun_02012ec4(gd);
    let p = h.fun_0200b3a4(ew, EW_FLAG_572);
    h.w32(p, 1);
}

/// C: `sub_0216f8b8` (command 590): [`set_flag_590`].
pub fn cmd_590<H: Misc10cHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    set_flag_590(h, gs);
    0
}

/// C: `sub_0216f8c8` (command 591): `FUN_021F385C(fm)`.
pub fn cmd_591<H: Misc10cHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    h.fun_021f385c(fm);
    0
}

/// C: `sub_0216f8dc` (command 592): call the event `FUN_021F38BC(gs)`.
pub fn cmd_592<H: Misc10cHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = h.fun_021f38bc(gs);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `FUN_overlay_d_10__0216f900`: event-work flag word 0x36 = 1.
pub fn set_flag_590<H: Misc10cHooks>(h: &mut H, gamesys: u32) {
    let gd = h.fun_020120f4(gamesys);
    let ew = h.fun_02012ec4(gd);
    let p = h.fun_0200b3a4(ew, EW_FLAG_590);
    h.w32(p, 1);
}

/// Run the command handler at `handler` (Thumb bit allowed) when it is one of this area's; `None` otherwise.
pub fn dispatch<H: Misc10cHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x0216_BF38 => cmd_445(h, vm, ctx),
        0x0216_4778 => cmd_458(h, vm, ctx),
        0x0216_4910 => cmd_467(h, vm, ctx),
        0x0216_4994 => cmd_468(h, vm, ctx),
        0x0216_4978 => cmd_469(h, vm, ctx),
        0x0216_A3F8 => cmd_505(h, vm, ctx),
        0x0216_B86C => cmd_534(h, vm, ctx),
        0x0216_BD54 => cmd_544(h, vm, ctx),
        0x0216_BBD4 => cmd_545(h, vm, ctx),
        0x0216_BC88 => cmd_546(h, vm, ctx),
        0x0216_BCE4 => cmd_547(h, vm, ctx),
        0x0216_BD1C => cmd_548(h, vm, ctx),
        0x0216_BEC0 => cmd_552(h, vm, ctx),
        0x0216_CC30 => cmd_572(h, vm, ctx),
        0x0216_F8B8 => cmd_590(h, vm, ctx),
        0x0216_F8C8 => cmd_591(h, vm, ctx),
        0x0216_F8DC => cmd_592(h, vm, ctx),
        _ => return None,
    })
}

/// Run the GMEVENT function `f(event, seq, work)` (Thumb bit set) when this area installed it; `None` otherwise.
pub fn call_event<H: Misc10cHooks>(
    h: &mut H,
    f: u32,
    event: u32,
    seq: u32,
    work: u32,
) -> Option<u32> {
    (f == EVENT_468).then(|| event_468(h, event, seq, work))
}

#[cfg(test)]
mod tests;
