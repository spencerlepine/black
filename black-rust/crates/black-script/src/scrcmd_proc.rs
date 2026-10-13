//! F8d: overlay-10 script commands of `scrcmd_proc.c`, Thumb code 0x0215C63C-0x0215CE98 (F1-SPLIT row 10.15 minus its tail,
//! which is `scrcmd_sodateya.c`): commands that run an application proc and wait for it (332..341, 355..357, 370, 489) and
//! the overlay-37 wrappers 383 / 386 / 390 / 399 / 406 (17 command ids, 22 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_proc.spec` (header: command-id table,
//! roles, the proc-work layout), ported here, listed in [`PORTED`] and checked with `tools/check-fn.sh black-script
//! scrcmd_proc`. [`COMMANDS`] maps the command ids to their handlers; [`dispatch`] / [`call_wait`] run a handler / the
//! wait callback by address and [`call_callback`] the result callbacks the commands hand to the app-proc code.
//!
//! Callees outside the area are [`ProcHooks`] methods named by address (vm.c, the operand readers, the ctx getters and the
//! child-event call come from [`CoreHooks`]).
//!
//! # App-proc requests (proposed `ScriptHost` mapping, not in `host.rs` yet)
//!
//! The original starts an application (a separate screen running in its own overlay) in two ways; which application each
//! overlay id is (0x46, 0x47, 0x88, 0xAD, 0xAE, 0xDD..0xE0) is not identified (G2 / U jobs), so the mapping stays at the
//! level of the call sites:
//!
//! * **proc call + wait** ([`proc_call_wait`], commands 334 / 337 / 339 / 341 / 370): `FUN_020120B4(GAMESYS, ovl, proc table,
//!   param)` starts the app with the parameter block `param` (allocated and filled by the command); the VM then polls
//!   [`PROC_WAIT`] every frame, which is done once `FUN_020120F0(GAMESYS)` is 0 (no app running) and then hands the proc work
//!   to the result callback (copy the app's results from `param` into script vars, free the blocks). A game-side
//!   implementation of [`ProcHooks::fun_020120b4`] would issue `ScriptHost::app_start(ovl, proc table, param)` and
//!   [`ProcHooks::fun_020120f0`] answer `ScriptHost::app_running()`; the parameter / result layouts are the commands' (see
//!   each command's doc), the app itself is the rewrite's.
//! * **app event** (commands 336 / 489, and `scrcmd_musical.c` 359): `FUN_02014840` / `FUN_02014808(GAMESYS, field map, ovl,
//!   proc table, param, result callback, callback work)` (main) build a GMEVENT that leaves the field, runs the app, returns
//!   and calls the callback ([`cb_336`] / [`cb_489`], routed by [`call_callback`]); the script only waits for the child event
//!   (`FUN_0215897C`). The same `app_start` / `app_running` pair covers it once the event code (main, F10 / G2) is ported;
//!   until then [`ProcHooks::fun_02014840`] / [`ProcHooks::fun_02014808`] are the seam.
//!
//! Commands 340 (`FUN_02161B90`, event_field_proclink.c), 356 / 357 (`FUN_020124AC` events) and the overlay-37 wrappers
//! 383..406 are events / calls of other code, not host requests.

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_C63C,
    0x0215_C69C,
    0x0215_C6F0,
    0x0215_C700,
    0x0215_C72C,
    0x0215_C7C4,
    0x0215_C7E0,
    0x0215_C870,
    0x0215_C89C,
    0x0215_C95C,
    0x0215_CA64,
    0x0215_CAB4,
    0x0215_CB08,
    0x0215_CBF0,
    0x0215_CC38,
    0x0215_CC4C,
    0x0215_CCE8,
    0x0215_CD3C,
    0x0215_CD80,
    0x0215_CDB8,
    0x0215_CDF0,
    0x0215_CE70,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (332, 0x0215_C6F0),
    (334, 0x0215_C72C),
    (336, 0x0215_C7E0),
    (337, 0x0215_CB08),
    (339, 0x0215_CBF0),
    (340, 0x0215_CA64),
    (341, 0x0215_CAB4),
    (355, 0x0215_CC38),
    (356, 0x0215_CC4C),
    (357, 0x0215_CCE8),
    (370, 0x0215_C95C),
    (383, 0x0215_CD3C),
    (386, 0x0215_CD80),
    (390, 0x0215_CDB8),
    (399, 0x0215_CDF0),
    (406, 0x0215_CE70),
    (489, 0x0215_C89C),
];

/// Wait callback of [`proc_call_wait`] ([`proc_wait`], `0x0215C69C`, Thumb).
pub const PROC_WAIT: u32 = 0x0215_C69D;
/// Result callbacks the commands pass (Thumb): command 334 ([`cb_334`], through [`proc_call_wait`]), 336 ([`cb_336`]) and 489
/// ([`cb_489`], through the app events).
pub const CB_334: u32 = 0x0215_C701;
pub const CB_336: u32 = 0x0215_C7C5;
pub const CB_489: u32 = 0x0215_C871;
/// `__FILE__` string "scrcmd_proc.c" passed to `Heap_AllocDebug`.
pub const FILE: u32 = 0x0217_28F0;
/// Heap of the proc work blocks.
pub const HEAP: u32 = 4;
/// Size of the proc work W: {param, cbwork, cb}.
pub const PROC_WORK_SIZE: u32 = 0xC;
/// Proc of command 356 and its event kind (`FUN_020124AC(GAMESYS, 2, proc, &rec)`).
pub const PROC_356: u32 = 0x0215_4439;
/// Proc of command 357 and its event kind (`FUN_020124AC(GAMESYS, 3, proc, &rec)`).
pub const PROC_357: u32 = 0x0215_43BD;
/// The overlay the commands 383 / 386 / 390 / 399 / 406 load around their overlay-37 call (`FUN_02034AC4` / `FUN_02034A5C`).
pub const OVL_37: u32 = 0x25;

/// App procs of the commands: (overlay id, proc table) as passed to `FUN_020120B4` / the app events (UNVERIFIED which
/// application each is).
pub const APP_334: (u32, u32) = (0x46, 0x021B_B96C);
pub const APP_336: (u32, u32) = (0x88, 0x021C_78A4);
pub const APP_337_0: (u32, u32) = (0xDD, 0x021B_A1A4);
pub const APP_337_1: (u32, u32) = (0xDE, 0x021B_A250);
pub const APP_337_2: (u32, u32) = (0xE0, 0x021B_9BF8);
pub const APP_339: (u32, u32) = (0xDF, 0x021B_C8C4);
pub const APP_341: (u32, u32) = (0xAE, 0x021B_74A0);
pub const APP_370: (u32, u32) = (0x47, 0x021B_A758);
pub const APP_489: (u32, u32) = (0xAD, 0x021B_81D0);

/// Callees of the scrcmd_proc functions outside the area, named by address (meaning per the spec; names of main /
/// overlay routines are inferred, UNVERIFIED unless stated). vm.c, the operand readers, the scrcmd ctx getters and the
/// child-event call come from [`CoreHooks`].
#[allow(clippy::too_many_arguments)]
pub trait ProcHooks: CoreHooks {
    /// C: `FUN_02158F50` (`script_sys::sw_event`): the script's GMEVENT.
    fn fun_02158f50(&mut self, sw: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::sw_event(self, sw)
    }
    /// C: `FUN_02158FB4` (`script_sys::sw_addr_3c`): address of the proc-work slot SW + 0x3C.
    fn fun_02158fb4(&mut self, sw: u32) -> u32 {
        crate::script_sys::sw_addr_3c(sw)
    }
    /// C: `FUN_02158FB8` (`script_sys::sw_get_3c`): the proc work in the SW + 0x3C slot.
    fn fun_02158fb8(&mut self, sw: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::sw_get_3c(self, sw)
    }
    /// C: `FUN_02158FBC` (`script_sys::sw_free_3c`): free the proc work and clear the slot.
    fn fun_02158fbc(&mut self, sw: u32);
    /// C: `(W+8)(W)` at `0x0215C6CC`: the result callback `cb` of a finished proc (`W` = the proc work); [`call_callback`]
    /// routes the ported ones ([`CB_334`]).
    fn call_proc_callback(&mut self, cb: u32, w: u32);
    /// C: `Heap_AllocDebug(heap, size, from_end, file, line)` (`0x02030734`).
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_0216F940(block)` (overlay 10, a thunk to `Heap_Free`).
    fn fun_0216f940(&mut self, block: u32);
    /// C: `FUN_020120B4(gamesys, ovl, proc, param)` (main): start an app proc (proposed `ScriptHost::app_start`; UNVERIFIED role).
    fn fun_020120b4(&mut self, gamesys: u32, ovl: u32, proc_table: u32, param: u32);
    /// C: `FUN_020120F0(gamesys)` (main): nonzero while an app proc runs (proposed `ScriptHost::app_running`; UNVERIFIED).
    fn fun_020120f0(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012138(gamesys)` (main; command 370 passes it to `FUN_02158344`).
    fn fun_02012138(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020124AC(gamesys, kind, proc, &rec)` (main): an event built from the stack record `rec` (8 bytes for command
    /// 357, 16 for 356).
    fn fun_020124ac(&mut self, gamesys: u32, kind: u32, proc_table: u32, rec: &[u8]) -> u32;
    /// C: `FUN_02012934(gamedata)` (main): the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)` (main; UNVERIFIED: a GAMEDATA sub record the apps of 337 / 489 get).
    fn fun_02012944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012B58(gamedata)` (main; command 370 compares it with the comm record's low 3 bits; UNVERIFIED).
    fn fun_02012b58(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)` (main): the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200BE48(save)` (main; command 370, result unused; UNVERIFIED).
    fn fun_0200be48(&mut self, save: u32);
    /// C: `FUN_0201AB28(party, i)` (main): the PokeParam of party slot `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_0202E474(gamedata, 0, m, 4)` (main): the parameter block of command 334's app (UNVERIFIED).
    fn fun_0202e474(&mut self, gamedata: u32, a: u32, m: u32, heap: u32) -> u32;
    /// C: `FUN_02014808(gamesys, field map, ovl, proc, param, cb, cbwork)` (main): the app event of command 489 (and 359).
    fn fun_02014808(
        &mut self,
        gamesys: u32,
        fm: u32,
        ovl: u32,
        proc_table: u32,
        param: u32,
        cb: u32,
        cbwork: u32,
    ) -> u32;
    /// C: `FUN_02014840(gamesys, field map, ovl, proc, param, cb, cbwork)` (main): the app event of command 336.
    fn fun_02014840(
        &mut self,
        gamesys: u32,
        fm: u32,
        ovl: u32,
        proc_table: u32,
        param: u32,
        cb: u32,
        cbwork: u32,
    ) -> u32;
    /// C: `FUN_02034AC4(ovl)` (main): load an overlay.
    fn fun_02034ac4(&mut self, ovl: u32);
    /// C: `FUN_02034A5C(ovl)` (main): unload it.
    fn fun_02034a5c(&mut self, ovl: u32);
    /// C: `FUN_02158344(x)` (overlay 10, `delivery_irc` side): the comm state record, or 0 (UNVERIFIED).
    fn fun_02158344(&mut self, x: u32) -> u32;
    /// C: `FUN_0215840C(x, i)` (overlay 10): the 20-halfword record `i` of the comm state (UNVERIFIED).
    fn fun_0215840c(&mut self, x: u32, i: u32) -> u32;
    /// C: `FUN_0215841C(gamedata)` (overlay 10): an index 0..3 into the comm state (UNVERIFIED).
    fn fun_0215841c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02161B90(gamesys, event, a, b, 0)` (overlay 10, event_field_proclink.c): the event of command 340.
    fn fun_02161b90(&mut self, gamesys: u32, event: u32, a: u32, b: u32, c: u32) -> u32;
    /// C: `FUN_02165344()` (overlay 10; command 337 calls it for modes other than 2; UNVERIFIED).
    fn fun_02165344(&mut self);
    /// C: `FUN_0216F918(heap)` (overlay 10): allocate the 0x1C-byte parameter block of command 489 (UNVERIFIED size).
    fn fun_0216f918(&mut self, heap: u32) -> u32;
    /// C: `FUN_0216F948(mon, heap)` (overlay 10; command 489 keeps the result at P + 0x10 and frees it in [`cb_489`]).
    fn fun_0216f948(&mut self, mon: u32, heap: u32) -> u32;
    /// C: `FUN_02175648(gamedata, out)` (overlay 11): fills the 16 bytes at `out` (command 370's fallback; UNVERIFIED).
    fn fun_02175648(&mut self, gamedata: u32, out: u32);
    /// C: `FUN_02175FB4(x)` (overlay 11): 1 = the comm record entry `x` is usable (UNVERIFIED).
    fn fun_02175fb4(&mut self, x: u32) -> u32;
    /// C: `FUN_021F36C0(gamesys, a, b)` (overlay 37; command 383).
    fn fun_021f36c0(&mut self, gamesys: u32, a: u32, b: u32);
    /// C: `FUN_021F36E8(gamesys, v)` (overlay 37; command 386).
    fn fun_021f36e8(&mut self, gamesys: u32, v: u32);
    /// C: `FUN_021F3724(gamesys, v)` (overlay 37; command 390).
    fn fun_021f3724(&mut self, gamesys: u32, v: u32);
    /// C: `FUN_021F3768(gamesys, &vars)` (overlay 37; command 399): the 8 var addresses as a 32-byte stack array.
    fn fun_021f3768(&mut self, gamesys: u32, vars: &[u8; 32]);
    /// C: `FUN_021F3800(gamesys)` (overlay 37; command 406).
    fn fun_021f3800(&mut self, gamesys: u32);
}

// ---- the app-proc request

/// C: `FUN_overlay_d_10__0215c63c` (proc_call_wait): the app-proc request of commands 334 / 337 / 339 / 341 / 370. The proc work
/// W = `Heap_AllocDebug(4, 0xC, 1, "scrcmd_proc.c", 0x8B)` = {param, cbwork, cb} goes into the SW + 0x3C slot, the proc `ovl` /
/// `proc_table` starts with `param` and the VM waits on [`PROC_WAIT`] (which calls `cb(W)` or frees param / cbwork when the proc
/// ends). `param`, `cb`, `cbwork` are the stack arguments 5..7.
#[allow(clippy::too_many_arguments)]
pub fn proc_call_wait<H: ProcHooks>(
    h: &mut H,
    vm: u32,
    ctx: u32,
    ovl: u32,
    proc_table: u32,
    param: u32,
    cb: u32,
    cbwork: u32,
) {
    let gamesys = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let slot = h.fun_02158fb4(sw);
    let w = h.fun_02030734(HEAP, PROC_WORK_SIZE, 1, FILE, 0x8B);
    h.w32(w, param);
    h.w32(w + 4, cbwork);
    h.w32(w + 8, cb);
    h.fun_020120b4(gamesys, ovl, proc_table, param);
    h.w32(slot, w);
    h.fun_020113d0(vm, PROC_WAIT);
}

/// C: `sub_0215c69c`: the wait callback of [`proc_call_wait`]: 0 while `FUN_020120F0(GAMESYS)` (an app proc runs) is nonzero;
/// then the proc work W (SW + 0x3C): `cb(W)` when W.cb is set, else `Heap_Free` of W.param / W.cbwork (each when nonzero);
/// the slot is freed (`FUN_02158FBC`); 1 (done).
pub fn proc_wait<H: ProcHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gamesys = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let w = h.fun_02158fb8(sw);
    if h.fun_020120f0(gamesys) != 0 {
        return 0;
    }
    let cb = h.r32(w + 8);
    if cb != 0 {
        h.call_proc_callback(cb, w);
    } else {
        let param = h.r32(w);
        if param != 0 {
            h.fun_020307b0(param);
        }
        let cbwork = h.r32(w + 4);
        if cbwork != 0 {
            h.fun_020307b0(cbwork);
        }
    }
    h.fun_02158fbc(sw);
    1
}

/// C: `sub_0215c6f0`: command 332: free the proc work at SW + 0x3C; 1 (yield).
pub fn cmd_332<H: ProcHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_02158fbc(sw);
    1
}

/// C: `sub_0215c700`: the result callback of command 334 (`w` = the proc work): with R = param and O = cbwork = {u16 *a,
/// u16 *b}: `*a = R+0x34 != 0`, `*b = R+0x38`; then O and R are freed.
pub fn cb_334<H: ProcHooks>(h: &mut H, w: u32) {
    let r = h.r32(w);
    let o = h.r32(w + 4);
    let flag = (h.r32(r + 0x34) != 0) as u16;
    let a = h.r32(o);
    h.w16(a, flag);
    let v = h.r32(r + 0x38);
    let b = h.r32(o + 4);
    h.w16(b, v as u16);
    let o = h.r32(w + 4);
    h.fun_020307b0(o);
    let r = h.r32(w);
    h.fun_020307b0(r);
}

/// C: `sub_0215c72c`: command 334: m = value, vars a, b; R = `FUN_0202E474(GAMEDATA, 0, m 0 -> 4 / 1 -> 5 / else 0, 4)`; O =
/// `alloc(8)` = {a, b}; [`proc_call_wait`] with [`APP_334`], param R, callback [`CB_334`] (cbwork O); 1.
pub fn cmd_334<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5dc(ctx);
    h.fun_0215a5f4(ctx);
    let m = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let b = h.fun_02159ae8(vm, ctx);
    let m = match m {
        0 => 4,
        1 => 5,
        _ => 0,
    };
    let gd = h.fun_0215a5e4(ctx);
    let r = h.fun_0202e474(gd, 0, m, 4);
    let o = h.fun_02030734(HEAP, 8, 1, FILE, 0x118);
    h.w32(o, a);
    h.w32(o + 4, b);
    proc_call_wait(h, vm, ctx, APP_334.0, APP_334.1, r, CB_334, o);
    1
}

/// C: `sub_0215c7c4`: the result callback of command 336 (`w` = {A, u16 *var}): `*var = A+4 == 1`; free A (a tail call).
pub fn cb_336<H: ProcHooks>(h: &mut H, w: u32) {
    let a = h.r32(w);
    let v = (h.r32(a + 4) == 1) as u16;
    let var = h.r32(w + 4);
    h.w16(var, v);
    let a = h.r32(w);
    h.fun_020307b0(a);
}

/// C: `sub_0215c7e0`: command 336: var; A = `alloc(8)` = {GAMEDATA}, B = `alloc(8)` = {A, var}; the app event
/// `FUN_02014840(GAMESYS, field map, APP_336, A, CB_336, B)` runs as a child event; 1.
pub fn cmd_336<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let fm = h.fun_0201210c(gs);
    let a = h.fun_02030734(HEAP, 8, 0, FILE, 335);
    let b = h.fun_02030734(HEAP, 8, 0, FILE, 336);
    h.w32(a, gd);
    h.w32(b, a);
    let var = h.fun_02159ae8(vm, ctx);
    h.w32(b + 4, var);
    let ev = h.fun_02014840(gs, fm, APP_336.0, APP_336.1, a, CB_336, b);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0215c870`: the result callback of command 489 (`w` = {P, u16 *var}): `*var = P+0x1A byte == 1`; free the block at
/// P+0x10, then P (through the thunk `0x0216F940`).
pub fn cb_489<H: ProcHooks>(h: &mut H, w: u32) {
    let p = h.r32(w);
    let v = (h.r8(p + 0x1A) == 1) as u16;
    let var = h.r32(w + 4);
    h.w16(var, v);
    let p = h.r32(w);
    let blk = h.r32(p + 0x10);
    h.fun_020307b0(blk);
    let p = h.r32(w);
    h.fun_0216f940(p);
}

/// C: `sub_0215c89c`: command 489: var, i = value; mon = party slot i; P = `FUN_0216F918(4)` = {mon, `FUN_02012944(GAMEDATA)`,
/// 0, GAMESYS, `FUN_0216F948(mon, 4)`, .., byte +0x19 = 1}; W = `alloc(8)` = {P, var}; the app event
/// `FUN_02014808(GAMESYS, field map, APP_489, P, CB_489, W)` runs as a child event; 1.
pub fn cmd_489<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let fm = h.fun_0201210c(gs);
    let var = h.fun_02159ae8(vm, ctx);
    let i = h.fun_02159b10(vm, ctx);
    let party = h.fun_02012934(gd);
    let mon = h.fun_0201ab28(party, i);
    let x = h.fun_02012944(gd);
    let y = h.fun_0216f948(mon, 4);
    let p = h.fun_0216f918(4);
    h.w32(p, mon);
    h.w32(p + 4, x);
    h.w32(p + 8, 0);
    h.w32(p + 0xC, gs);
    h.w32(p + 0x10, y);
    h.w8(p + 0x19, 1);
    let w = h.fun_02030734(HEAP, 8, 0, FILE, 411);
    h.w32(w, p);
    h.w32(w + 4, var);
    let ev = h.fun_02014808(gs, fm, APP_489.0, APP_489.1, p, CB_489, w);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0215c95c`: command 370: x = `FUN_02158344(FUN_02012138(GAMESYS))` (the comm state record, or 0); W = `alloc(0x40)`.
/// When x is set, `(x+0x86 u16) & 7 != FUN_02012B58(GAMEDATA)`, `FUN_02175FB4(x + 0x1AC + 0x28 * FUN_0215841C(GAMEDATA)) == 1`
/// and the byte x+0x922 is 1: i = `FUN_0215841C(GAMEDATA)`, W+4 = the 20 halfwords of `FUN_0215840C(x, i)`, W+0x2C = the 16
/// bytes at x+0x914, byte W+0x3D = 1. Otherwise `FUN_0200BE48(SAVE)`, `FUN_02175648(GAMEDATA, W+0x2C)`, W+0x3D = 0, i = 0.
/// Then W[0] = GAMESYS, byte W+0x3C = i; [`proc_call_wait`] with [`APP_370`], param W, no callback; 1.
pub fn cmd_370<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let c = h.fun_02012138(gs);
    let gd = h.fun_020120f4(gs);
    h.fun_0215a5f4(ctx);
    let x = h.fun_02158344(c);
    let w = h.fun_02030734(HEAP, 0x40, 1, FILE, 0x1BC);
    let mut usable = false;
    if x != 0 {
        let v = h.fun_02012b58(gd);
        if (h.r16(x + 0x86) as u32) & 7 != v {
            let k = h.fun_0215841c(gd);
            let e = x.wrapping_add(0x1AC).wrapping_add(k.wrapping_mul(0x28));
            usable = h.fun_02175fb4(e) == 1 && h.r8(x + 0x922) == 1;
        }
    }
    let i = if usable {
        let i = h.fun_0215841c(gd);
        let src = h.fun_0215840c(x, i);
        for k in 0..0x14 {
            let v = h.r16(src + 2 * k);
            h.w16(w + 4 + 2 * k, v);
        }
        for k in 0..4 {
            let v = h.r32(x + 0x914 + 4 * k);
            h.w32(w + 0x2C + 4 * k, v);
        }
        h.w8(w + 0x3D, 1);
        i
    } else {
        let save = h.fun_02012ebc(gd);
        h.fun_0200be48(save);
        h.fun_02175648(gd, w + 0x2C);
        h.w8(w + 0x3D, 0);
        0
    };
    h.w32(w, gs);
    h.w8(w + 0x3C, i as u8);
    proc_call_wait(h, vm, ctx, APP_370.0, APP_370.1, w, 0, 0);
    1
}

/// C: `sub_0215ca64`: command 340: a, b = values; the event `FUN_02161B90(GAMESYS, the script's GMEVENT, a, b, 0)`
/// (event_field_proclink.c) runs as a child event; 1.
pub fn cmd_340<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let parent = h.fun_02158f50(sw);
    let ev = h.fun_02161b90(gs, parent, a, b, 0);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0215cab4`: command 341: v = value; W = `alloc(8)` = {u16 v, GAMEDATA (+4)}; [`proc_call_wait`] with [`APP_341`],
/// param W, no callback; 1.
pub fn cmd_341<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let w = h.fun_02030734(HEAP, 8, 1, FILE, 0x205);
    h.w16(w, v as u16);
    let gd = h.fun_0215a5e4(ctx);
    h.w32(w + 4, gd);
    proc_call_wait(h, vm, ctx, APP_341.0, APP_341.1, w, 0, 0);
    1
}

/// C: `FUN_overlay_d_10__0215cb08`: command 337: m, v = values; m != 2 calls `FUN_02165344()` first. m 0 / 1: W = `alloc(8)` =
/// {GAMEDATA, v}, [`proc_call_wait`] with [`APP_337_0`] / [`APP_337_1`]; m 2: W = `alloc(4)` = {`FUN_02012944(GAMEDATA)`},
/// [`APP_337_2`]; other m: no proc. No callback; 1.
pub fn cmd_337<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let x = h.fun_02012944(gd);
    let m = h.fun_02159b10(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    if m != 2 {
        h.fun_02165344();
    }
    let (w, app) = match m {
        0 | 1 => {
            let (line, app) = if m == 0 {
                (0x22B, APP_337_0)
            } else {
                (0x233, APP_337_1)
            };
            let w = h.fun_02030734(HEAP, 8, 1, FILE, line);
            h.w32(w, gd);
            h.w32(w + 4, v);
            (w, app)
        }
        2 => {
            let w = h.fun_02030734(HEAP, 4, 1, FILE, 0x23C);
            h.w32(w, x);
            (w, APP_337_2)
        }
        _ => return 1,
    };
    proc_call_wait(h, vm, ctx, app.0, app.1, w, 0, 0);
    1
}

/// C: `sub_0215cbf0`: command 339: var; W = `alloc(8)` = {var}; [`proc_call_wait`] with [`APP_339`], param W, no callback; 1.
pub fn cmd_339<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let w = h.fun_02030734(HEAP, 8, 1, FILE, 0x254);
    h.w32(w, var);
    proc_call_wait(h, vm, ctx, APP_339.0, APP_339.1, w, 0, 0);
    1
}

/// C: `sub_0215cc38`: command 355: reads the SW and the GAMESYS (unused); 1 (yield).
pub fn cmd_355<H: ProcHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    h.fun_0215a5dc(ctx);
    1
}

/// C: `sub_0215cc4c`: command 356: t, s = values; the event `FUN_020124AC(GAMESYS, 2, 0x02154439, &rec)` with the 16-byte record
/// {field map, 1, s (0 / 1), t - 15 (t 15..19)} runs as a child event; 1. An original quirk, kept: for s > 1 the third word is
/// the caller's r6 and for t outside 15..19 the fourth the caller's r4, which the function never sets (`stale_r6` /
/// `stale_r4`; [`dispatch`] passes 0, the Z19 entry value the differential test runs with).
pub fn cmd_356<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32, stale_r4: u32, stale_r6: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let t = h.fun_02159b10(vm, ctx);
    let s = h.fun_02159b10(vm, ctx);
    let k = t.wrapping_sub(15);
    let r4 = if k <= 4 { k } else { stale_r4 };
    let r6 = match s {
        0 => 0,
        1 => 1,
        _ => stale_r6,
    };
    let fm = h.fun_0201210c(gs);
    let mut rec = [0u8; 16];
    rec[..4].copy_from_slice(&fm.to_le_bytes());
    rec[4..8].copy_from_slice(&1u32.to_le_bytes());
    rec[8..12].copy_from_slice(&r6.to_le_bytes());
    rec[12..].copy_from_slice(&r4.to_le_bytes());
    let ev = h.fun_020124ac(gs, 2, PROC_356, &rec);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0215cce8`: command 357: `v` = value; the event `FUN_020124AC(GAMESYS, 3, 0x021543BD, &{field map, v == 0 ? 1 :
/// 2})` runs as a child event; 1.
pub fn cmd_357<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gamesys = h.fun_0215a5dc(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let kind: u32 = if v == 0 { 1 } else { 2 };
    let fm = h.fun_0201210c(gamesys);
    let mut rec = [0u8; 8];
    rec[..4].copy_from_slice(&fm.to_le_bytes());
    rec[4..].copy_from_slice(&kind.to_le_bytes());
    let ev = h.fun_020124ac(gamesys, 3, PROC_357, &rec);
    h.fun_0215897c(sw, ev);
    1
}

// ---- overlay-37 wrappers

/// C: `sub_0215cd3c`: command 383: vars a, b; with overlay 0x25 loaded: `FUN_021F36C0(GAMESYS, *a, *b)`; 0.
pub fn cmd_383<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let b = h.fun_02159ae8(vm, ctx);
    h.fun_02034ac4(OVL_37);
    let (va, vb) = (h.r16(a) as u32, h.r16(b) as u32);
    h.fun_021f36c0(gs, va, vb);
    h.fun_02034a5c(OVL_37);
    0
}

/// C: `sub_0215cd80`: command 386: v = value; with overlay 0x25 loaded: `FUN_021F36E8(GAMESYS, v)`; 0.
pub fn cmd_386<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let v = h.fun_02159b10(vm, ctx);
    h.fun_02034ac4(OVL_37);
    h.fun_021f36e8(gs, v);
    h.fun_02034a5c(OVL_37);
    0
}

/// C: `sub_0215cdb8`: command 390: v = value; with overlay 0x25 loaded: `FUN_021F3724(GAMESYS, v)`; 0.
pub fn cmd_390<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let v = h.fun_02159b10(vm, ctx);
    h.fun_02034ac4(OVL_37);
    h.fun_021f3724(gs, v);
    h.fun_02034a5c(OVL_37);
    0
}

/// C: `sub_0215cdf0`: command 399: 8 vars (their addresses in a stack array); with overlay 0x25 loaded:
/// `FUN_021F3768(GAMESYS, &array)`; 0.
pub fn cmd_399<H: ProcHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let mut vars = [0u8; 32];
    for k in 0..8 {
        let a = h.fun_02159ae8(vm, ctx);
        vars[4 * k..4 * k + 4].copy_from_slice(&a.to_le_bytes());
    }
    h.fun_02034ac4(OVL_37);
    h.fun_021f3768(gs, &vars);
    h.fun_02034a5c(OVL_37);
    0
}

/// C: `sub_0215ce70`: command 406: with overlay 0x25 loaded: `FUN_021F3800(GAMESYS)`; 0.
pub fn cmd_406<H: ProcHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    h.fun_02034ac4(OVL_37);
    h.fun_021f3800(gs);
    h.fun_02034a5c(OVL_37);
    0
}

// ---- dispatch

/// Run the command handler at `handler` (bit 0 ignored) when it is one of [`COMMANDS`]; `None` otherwise. Command 356's
/// unset-register words are passed as 0 (see [`cmd_356`]).
pub fn dispatch<H: ProcHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x0215_C6F0 => cmd_332(h, vm, ctx),
        0x0215_C72C => cmd_334(h, vm, ctx),
        0x0215_C7E0 => cmd_336(h, vm, ctx),
        0x0215_CB08 => cmd_337(h, vm, ctx),
        0x0215_CBF0 => cmd_339(h, vm, ctx),
        0x0215_CA64 => cmd_340(h, vm, ctx),
        0x0215_CAB4 => cmd_341(h, vm, ctx),
        0x0215_CC38 => cmd_355(h, vm, ctx),
        0x0215_CC4C => cmd_356(h, vm, ctx, 0, 0),
        0x0215_CCE8 => cmd_357(h, vm, ctx),
        0x0215_C95C => cmd_370(h, vm, ctx),
        0x0215_CD3C => cmd_383(h, vm, ctx),
        0x0215_CD80 => cmd_386(h, vm, ctx),
        0x0215_CDB8 => cmd_390(h, vm, ctx),
        0x0215_CDF0 => cmd_399(h, vm, ctx),
        0x0215_CE70 => cmd_406(h, vm, ctx),
        0x0215_C89C => cmd_489(h, vm, ctx),
        _ => return None,
    })
}

/// Run the wait callback `f` (VM +0x10, Thumb bit set) when it is [`PROC_WAIT`]; `None` otherwise.
pub fn call_wait<H: ProcHooks>(h: &mut H, f: u32, vm: u32, ctx: u32) -> Option<u32> {
    (f == PROC_WAIT).then(|| proc_wait(h, vm, ctx))
}

/// Run the result callback `cb(w)` (Thumb bit set) when it is one of this area's ([`CB_334`] from the proc wait, [`CB_336`] /
/// [`CB_489`] from the app events); false otherwise.
pub fn call_callback<H: ProcHooks>(h: &mut H, cb: u32, w: u32) -> bool {
    match cb {
        CB_334 => cb_334(h, w),
        CB_336 => cb_336(h, w),
        CB_489 => cb_489(h, w),
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests;
