//! F8 (fourth batch): overlay-21 `scrcmd_proc_fld.c`, Thumb code 0x021B4BEC-0x021B5190 (F1-SPLIT row 21.47 plus the tail of row
//! 21.46): commands that start field events and app procs and run them as child events (20 command ids, 21 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_proc_fld.spec` (header: command-id
//! table, roles, why the file starts at 0x021B4BEC). The functions listed in [`PORTED`] are ported here and checked with
//! `tools/check-fn.sh black-script scrcmd_proc_fld`; the rest are the F8 port job's.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x021B_4BEC, 0x021B_4C54, 0x021B_4D14];

/// Proc of command 353 and its event kind (`FUN_020124AC(GAMESYS, 6, proc, &rec)`).
pub const PROC_353: u32 = 0x0215_4C21;
pub const KIND_353: u32 = 6;
/// Mode of the proc work of command 335 for which the result callback writes the box data back.
pub const MODE_335_BOX: u32 = 4;

/// Callees of the scrcmd_proc_fld functions that this area does not port, named by address (meaning per the spec).
pub trait ProcFldHooks: MemEnv {
    /// C: `FUN_02159AE8` (script_sys `cmd_var_addr`): the address of the next var operand.
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_0215A5DC` (script_sys `ctx_gamesys`).
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A5F4` (script_sys `ctx_sw`).
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215897C` (script_sys): call `child` as a child event of the script's event.
    fn fun_0215897c(&mut self, sw: u32, child: u32);
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012138(gamesys)` (main; UNVERIFIED role).
    fn fun_02012138(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_021896F4(x)` (overlay 21; UNVERIFIED role): the value command 314 stores.
    fn fun_021896f4(&mut self, x: u32) -> u32;
    /// C: `FUN_020124AC(gamesys, kind, proc, &rec)` (main): an event built from the 8-byte stack record `rec`.
    fn fun_020124ac(&mut self, gamesys: u32, kind: u32, proc_table: u32, rec: &[u8; 8]) -> u32;
    /// C: `FUN_02012EBC(gamedata)` (main): the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200BAC8(save)` (main): a save block (UNVERIFIED: the battle box).
    fn fun_0200bac8(&mut self, save: u32) -> u32;
    /// C: `FUN_0200BB58(block, data)` (main; UNVERIFIED: store the proc's result data).
    fn fun_0200bb58(&mut self, block: u32, data: u32);
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
}

/// C: `sub_021b4bec`: command 314: `*var = FUN_021896F4(FUN_02012138(GAMESYS))`; 1.
pub fn cmd_314<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gamesys = h.fun_0215a5dc(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let x = h.fun_02012138(gamesys);
    let v = h.fun_021896f4(x);
    h.w16(var, v as u16);
    1
}

/// C: `FUN_overlay_d_21__021b4c54`: command 353: the event `FUN_020124AC(GAMESYS, 6, 0x02154C21, &{field map, 0})` runs as a
/// child event; 1.
pub fn cmd_353<H: ProcFldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gamesys = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gamesys);
    let mut rec = [0u8; 8];
    rec[..4].copy_from_slice(&fm.to_le_bytes());
    let ev = h.fun_020124ac(gamesys, KIND_353, PROC_353, &rec);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b4d14`: the result callback of command 335 (`w` = {GAMEDATA, u16 *var, P}): mode P+0x24 == 4 -> the data at P+8
/// goes back into the save block (`FUN_0200BB58(FUN_0200BAC8(SAVE), P+8)`) and is freed; `*var = P+0x28 == 1`; P is freed.
pub fn cb_335<H: ProcFldHooks>(h: &mut H, w: u32) {
    let p = h.r32(w + 8);
    if h.r32(p + 0x24) == MODE_335_BOX {
        let gd = h.r32(w);
        let save = h.fun_02012ebc(gd);
        let blk = h.fun_0200bac8(save);
        let p = h.r32(w + 8);
        let data = h.r32(p + 8);
        h.fun_0200bb58(blk, data);
    }
    let p = h.r32(w + 8);
    if h.r32(p + 0x24) == MODE_335_BOX {
        let data = h.r32(p + 8);
        h.fun_020307b0(data);
    }
    let p = h.r32(w + 8);
    let done = (h.r16(p + 0x28) == 1) as u16;
    let var = h.r32(w + 4);
    h.w16(var, done);
    let p = h.r32(w + 8);
    h.fun_020307b0(p);
}

#[cfg(test)]
mod tests;
