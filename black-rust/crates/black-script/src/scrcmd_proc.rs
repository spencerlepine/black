//! F8 (third batch): overlay-10 script commands of `scrcmd_proc.c`, Thumb code 0x0215C63C-0x0215CE98 (F1-SPLIT row 10.15 minus
//! its tail, which is `scrcmd_sodateya.c`): commands that run an application proc and wait for it (332..341, 355..357, 370,
//! 489) and the overlay-37 wrappers 383 / 386 / 390 / 399 / 406 (17 command ids, 22 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_proc.spec` (header: command-id table,
//! roles, the proc-work layout). The functions listed in [`PORTED`] are ported here and checked with `tools/check-fn.sh
//! black-script scrcmd_proc`; the rest are the F8 port job's. The app-proc start (`FUN_020120B4`) and the proc-running poll
//! (`FUN_020120F0`) are what a game-side `ScriptHost` app request would model (not in `host.rs` yet; see the crate README).

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_C63C,
    0x0215_C6F0,
    0x0215_C700,
    0x0215_C7C4,
    0x0215_C870,
    0x0215_CCE8,
];

/// Wait callback of [`proc_call_wait`] (`0x0215C69C`, Thumb).
pub const PROC_WAIT: u32 = 0x0215_C69D;
/// `__FILE__` string "scrcmd_proc.c" passed to `Heap_AllocDebug`.
pub const FILE: u32 = 0x0217_28F0;
/// Heap of the proc work blocks.
pub const HEAP: u32 = 4;
/// Size of the proc work W: {param, cbwork, cb}.
pub const PROC_WORK_SIZE: u32 = 0xC;
/// Proc of command 357 and its event kind (`FUN_020124AC(GAMESYS, 3, proc, &rec)`).
pub const PROC_357: u32 = 0x0215_43BD;

/// Callees of the scrcmd_proc functions that this area does not port, named by address (meaning per the spec).
pub trait ProcHooks: MemEnv {
    /// C: `FUN_020113D0` (vm.c `wait`): install a wait callback.
    fn fun_020113d0(&mut self, vm: u32, cb: u32);
    /// C: `FUN_02159B10` (script_sys `cmd_var_value`): the next value operand.
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_0215A5DC` (script_sys `ctx_gamesys`).
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A5F4` (script_sys `ctx_sw`).
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02158FB4` (script_sys `sw_addr_3c`): address of the proc-work slot SW + 0x3C.
    fn fun_02158fb4(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158FBC` (script_sys `sw_free_3c`): free the proc work and clear the slot.
    fn fun_02158fbc(&mut self, sw: u32);
    /// C: `FUN_0215897C` (script_sys): call `child` as a child event of the script's event.
    fn fun_0215897c(&mut self, sw: u32, child: u32);
    /// C: `Heap_AllocDebug(heap, size, from_end, file, line)` (`0x02030734`).
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_0216F940(block)` (overlay 10, a thunk to `Heap_Free`).
    fn fun_0216f940(&mut self, block: u32);
    /// C: `FUN_020120B4(gamesys, ovl, proc, param)` (main): start an app proc (UNVERIFIED role).
    fn fun_020120b4(&mut self, gamesys: u32, ovl: u32, proc_table: u32, param: u32);
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020124AC(gamesys, kind, proc, &rec)` (main): an event built from the 8-byte stack record `rec`.
    fn fun_020124ac(&mut self, gamesys: u32, kind: u32, proc_table: u32, rec: &[u8; 8]) -> u32;
}

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

/// C: `sub_0215c7c4`: the result callback of command 336 (`w` = {A, u16 *var}): `*var = A+4 == 1`; free A (a tail call).
pub fn cb_336<H: ProcHooks>(h: &mut H, w: u32) {
    let a = h.r32(w);
    let v = (h.r32(a + 4) == 1) as u16;
    let var = h.r32(w + 4);
    h.w16(var, v);
    let a = h.r32(w);
    h.fun_020307b0(a);
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

#[cfg(test)]
mod tests;
