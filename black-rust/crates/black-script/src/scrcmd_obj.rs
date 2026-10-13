//! F8 (second batch): overlay-21 field-object / camera script commands, Thumb code 0x021B1668-0x021B2830 (F1-SPLIT row 21.44 from
//! the movement commands on): apply / wait for movements, field-object (MMDL) queries and setters, the player, camera / scroll
//! tasks, the talk lock, and thin wrappers of an overlay at 0x021F3xxx (55 command ids, 63 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_obj.spec` (header: command-id table and
//! roles). The functions listed in [`PORTED`] are ported here and checked with `tools/check-fn.sh black-script scrcmd_obj`; the
//! rest are the F8 port job's. MMDL = the overlay-10 field-object system (fldmmdl.c, F4a).

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_1668,
    0x021B_16DC,
    0x021B_19E0,
    0x021B_1A44,
    0x021B_1F60,
];

/// Object id of the object with MMDL id 0x30 (`FUN_0216DE54(mmdls, 0x30)`; UNVERIFIED: the player's partner).
pub const OBJ_ID_PARTNER: u32 = 0xF2;
/// Object id that stands for the SCRIPT_WORK itself in [`object_of`].
pub const OBJ_ID_SW: u32 = 0xF1;
/// The flag byte of the talk lock (overlay-21 BSS): bit 0 player, 1 the 0x30 object, 2 the talk target, 3 its pair object
/// still moving.
pub const LOCK_FLAGS: u32 = 0x021E_E71C;

/// Callees of the scrcmd_obj functions that this area does not port, named by address (meaning per the spec).
pub trait ObjHooks: MemEnv {
    /// C: `FUN_02159AE8` (script_sys `cmd_var_addr`).
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_02159B10` (script_sys `cmd_var_value`).
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_0215A5EC` (script_sys `ctx_word3`): the MMDL system (word 3 of the scrcmd work).
    fn fun_0215a5ec(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A5F4` (script_sys `ctx_sw`).
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A6FC` (scrcmd_work): 1 while a slotted movement still runs.
    fn fun_0215a6fc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02158F58` (script_sys): the SW field block address (+4 = the field map).
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F90` (script_sys): the talk target MMDL (SW +8).
    fn fun_02158f90(&mut self, sw: u32) -> u32;
    /// C: `FUN_0216DE94(mmdls, id)` (fldmmdl): the MMDL with an OBJ id, or 0.
    fn fun_0216de94(&mut self, mmdls: u32, id: u32) -> u32;
    /// C: `FUN_0216DE54(mmdls, id)` (fldmmdl): the MMDL with an MMDL id, or 0 (UNVERIFIED).
    fn fun_0216de54(&mut self, mmdls: u32, id: u32) -> u32;
    /// C: `FUN_0216D63C(mmdl, n)` (fldmmdl): a u16 property n (0..2) of the MMDL.
    fn fun_0216d63c(&mut self, mmdl: u32, n: u32) -> u32;
    /// C: `FUN_0216D454(mmdl)` (fldmmdl): 1 while it is moving (UNVERIFIED).
    fn fun_0216d454(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216DA5C(mmdl)` (fldmmdl): its pause / move flag (UNVERIFIED).
    fn fun_0216da5c(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_0216DAF8(mmdl)` (fldmmdl): pause it (`flags |= 8`, thunk to `0x0216D... (mmdl, 8)`).
    fn fun_0216daf8(&mut self, mmdl: u32);
    /// C: `FUN_0219BBA4(mmdl)` (overlay 21): the object paired with it, or 0 (UNVERIFIED).
    fn fun_0219bba4(&mut self, mmdl: u32) -> u32;
    /// C: `FUN_02188CB4(field map)` (overlay 21): the player object.
    fn fun_02188cb4(&mut self, fm: u32) -> u32;
    /// C: `FUN_021A3098(player)` (overlay 21): its facing direction (0..3).
    fn fun_021a3098(&mut self, player: u32) -> u32;
    /// C: `FUN_021B1E74(ctx)` (this area, unported): the player MMDL (`FUN_0216DEE0(mmdls)`).
    fn fun_021b1e74(&mut self, ctx: u32) -> u32;
}

/// C: `FUN_overlay_d_21__021b1668`: the field object a script names: [`OBJ_ID_PARTNER`] -> `FUN_0216DE54(mmdls, 0x30)`, else
/// `FUN_0216DE94(mmdls, id)` (0 when there is none). An original bug, kept: for [`OBJ_ID_SW`] the original calls the SW
/// getter but returns its caller's r6, which it never sets (`stale_r6`; command 100 passes its movement offset).
/// UNVERIFIED in the differential test (the spec never generates 0xF1, since the emulator's r6 is the previous run's).
pub fn object_of<H: ObjHooks>(h: &mut H, ctx: u32, id: u32, stale_r6: u32) -> u32 {
    let mmdls = h.fun_0215a5ec(ctx);
    match id {
        OBJ_ID_PARTNER => h.fun_0216de54(mmdls, 0x30),
        OBJ_ID_SW => {
            h.fun_0215a5f4(ctx);
            stale_r6
        }
        _ => h.fun_0216de94(mmdls, id),
    }
}

/// C: `sub_021b16dc`: the wait callback of command 101: done (1) once no slotted movement runs.
pub fn wait_movements<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    (h.fun_0215a6fc(ctx) == 0) as u32
}

/// C: `sub_021b19e0`: command 121: id, n = values; var = `FUN_0216D63C(object id, n)` when n <= 2 and the object exists (else
/// the var is left alone).
pub fn cmd_object_prop<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let id = h.fun_02159b10(vm, ctx);
    let n = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let obj = h.fun_0216de94(mmdls, id);
    if n <= 2 && obj != 0 {
        let v = h.fun_0216d63c(obj, n);
        h.w16(var, v as u16);
    }
    0
}

/// C: `sub_021b1a44`: the wait callback of the talk lock (`0x021B1B08`): every object whose [`LOCK_FLAGS`] bit is set and that
/// has stopped (player: `FUN_0216D454` == 1; the others: `FUN_0216DA5C` == 0; a pair object that is gone counts as stopped) is
/// paused and its bit cleared; done (1) once no bit is left.
pub fn wait_lock<H: ObjHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let target = h.fun_02158f90(sw);
    let player = h.fun_021b1e74(ctx);
    let mmdls = h.fun_0215a5ec(ctx);
    let g = LOCK_FLAGS;
    if h.r8(g) & 1 != 0 && h.fun_0216d454(player) == 1 {
        h.fun_0216daf8(player);
        let f = h.r8(g) & 0xFE;
        h.w8(g, f);
    }
    if h.r8(g) & 4 != 0 && h.fun_0216da5c(target) == 0 {
        h.fun_0216daf8(target);
        let f = h.r8(g) & 0xFB;
        h.w8(g, f);
    }
    if h.r8(g) & 2 != 0 {
        let o = h.fun_0216de54(mmdls, 0x30);
        if h.fun_0216da5c(o) == 0 {
            h.fun_0216daf8(o);
            let f = h.r8(g) & 0xFD;
            h.w8(g, f);
        }
    }
    if h.r8(g) & 8 != 0 {
        let pair = h.fun_0219bba4(target);
        let stopped = if pair == 0 {
            true
        } else if h.fun_0216da5c(pair) == 0 {
            h.fun_0216daf8(pair);
            true
        } else {
            false
        };
        if stopped {
            let f = h.r8(g) & 0xF7;
            h.w8(g, f);
        }
    }
    (h.r8(g) == 0) as u32
}

/// C: `FUN_overlay_d_21__021b1f60`: command 120: var = the player's facing direction (`FUN_021A3098`) for 0..3, else 0.
pub fn cmd_player_dir<H: ObjHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let fb = h.fun_02158f58(sw);
    let fm = h.r32(fb + 4);
    let player = h.fun_02188cb4(fm);
    let var = h.fun_02159ae8(vm, ctx);
    let d = h.fun_021a3098(player);
    h.w16(var, if d <= 3 { d as u16 } else { 0 });
    0
}

#[cfg(test)]
mod tests;
