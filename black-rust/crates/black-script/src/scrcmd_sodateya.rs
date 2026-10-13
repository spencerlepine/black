//! F8 (third batch): overlay-10 script commands of `scrcmd_sodateya.c` (the script side of the daycare), Thumb code
//! 0x0215CE98-0x0215D34C (F1-SPLIT row 10.16 plus the tail of row 10.15; cmd 359 at 0x0215D34C is `scrcmd_musical.c`):
//! commands 235..248 and 593 (15 command ids, 18 functions). The daycare itself is overlay-21 `sodateya.c` (F13), reached as
//! `FUN_02188D50(field map)`.
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_sodateya.spec`. The functions listed in
//! [`PORTED`] are ported here and checked with `tools/check-fn.sh black-script scrcmd_sodateya`; the rest are the F8 port job's.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_CE98,
    0x0215_CFFC,
    0x0215_D14C,
    0x0215_D198,
    0x0215_D1E4,
];

/// PokeParam ids the daycare commands read (`FUN_02017E1C(mon, id, 0)`).
pub const PP_SPECIES: u32 = 5;
pub const PP_GENDER: u32 = 0x6E;
pub const PP_FORM: u32 = 0x6F;
/// UNVERIFIED: read as "has a nickname" (a Nidoran without one shows no gender mark).
pub const PP_NICKNAMED: u32 = 0x75;
/// Nidoran F / Nidoran M (their species names carry the gender sign).
pub const SPECIES_NIDORAN_F: u32 = 0x1D;
pub const SPECIES_NIDORAN_M: u32 = 0x20;

/// Callees of the scrcmd_sodateya functions that this area does not port, named by address (meaning per the spec).
pub trait SodateyaHooks: MemEnv {
    /// C: `FUN_02159AE8` (script_sys `cmd_var_addr`): the address of the next var operand.
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_02159B10` (script_sys `cmd_var_value`): the next value operand.
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_0215A5DC` (script_sys `ctx_gamesys`).
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02188D50(field map)` (overlay 21): the daycare object.
    fn fun_02188d50(&mut self, fm: u32) -> u32;
    /// C: `FUN_021C3400(daycare)` (overlay-21 sodateya.c; UNVERIFIED meaning).
    fn fun_021c3400(&mut self, daycare: u32) -> u32;
    /// C: `FUN_021C34F8(daycare, slot)` (overlay-21 sodateya.c): the PokeParam of a daycare slot.
    fn fun_021c34f8(&mut self, daycare: u32, slot: u32) -> u32;
    /// C: `FUN_02017E1C(mon, id, 0)` (main): a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_02012374(event, child)` (main): call a child event.
    fn fun_02012374(&mut self, event: u32, child: u32);
    /// C: `FUN_021BE8D4(gamesys, field map, 0, 0)` (overlay 21): the event that leaves the field (UNVERIFIED).
    fn fun_021be8d4(&mut self, gamesys: u32, fm: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_020144DC(gamesys, field map)` (main; UNVERIFIED: an app event).
    fn fun_020144dc(&mut self, gamesys: u32, fm: u32) -> u32;
    /// C: `FUN_02014910(gamesys, field map, A, B)` (main; UNVERIFIED: the party-list app event).
    fn fun_02014910(&mut self, gamesys: u32, fm: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_020145CC(gamesys)` (main; UNVERIFIED).
    fn fun_020145cc(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_021BE92C(gamesys, field map, a, b, c, d, e)` (overlay 21): the event that returns to the field (UNVERIFIED).
    #[allow(clippy::too_many_arguments)]
    fn fun_021be92c(
        &mut self,
        gamesys: u32,
        fm: u32,
        a: u32,
        b: u32,
        c: u32,
        d: u32,
        e: u32,
    ) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
}

/// The daycare object of the script's GAMESYS.
fn daycare<H: SodateyaHooks>(h: &mut H, ctx: u32) -> u32 {
    let gamesys = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gamesys);
    h.fun_02188d50(fm)
}

/// C: `sub_0215ce98`: command 235: var = `FUN_021C3400(daycare)`; 0.
pub fn cmd_235<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let d = daycare(h, ctx);
    let v = h.fun_021c3400(d);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0215cffc`: command 242: var, slot; var = the species of the daycare mon in `slot`; 0.
pub fn cmd_242<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let slot = h.fun_02159b10(vm, ctx);
    let d = daycare(h, ctx);
    let mon = h.fun_021c34f8(d, slot);
    let v = h.fun_02017e1c(mon, PP_SPECIES, 0);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0215d14c`: command 593: var, mode, slot; var = mode 0 ? the [`gender_mark`] of the daycare mon in `slot` : 0; 0.
pub fn cmd_593<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let d = daycare(h, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let mode = h.fun_02159b10(vm, ctx);
    let slot = h.fun_02159b10(vm, ctx);
    let mon = h.fun_021c34f8(d, slot);
    let v = if mode == 0 { gender_mark(h, mon) } else { 0 };
    h.w16(var, v as u16);
    0
}

/// C: `FUN_overlay_d_10__0215d198`: the gender mark after a mon's name: 0 none, 1 male, 2 female (gender 0 / 1; genderless:
/// none). A Nidoran (F / M) whose [`PP_NICKNAMED`] is 0 gets none (its species name has the sign).
pub fn gender_mark<H: SodateyaHooks>(h: &mut H, mon: u32) -> u32 {
    let species = h.fun_02017e1c(mon, PP_SPECIES, 0);
    let gender = h.fun_02017e1c(mon, PP_GENDER, 0);
    if (species == SPECIES_NIDORAN_M || species == SPECIES_NIDORAN_F)
        && h.fun_02017e1c(mon, PP_NICKNAMED, 0) == 0
    {
        return 0;
    }
    match gender {
        0 => 1,
        1 => 2,
        _ => 0,
    }
}

/// C: `FUN_0215D1E4` (unlabeled routine at `0x0215D1E4`): the GMEVENT function of command 247 (`event`, `seq` = address of
/// the sequence word, `w` = {GAMESYS, field map, A, B, u16 *var}): seq 0..4 call the child events (leave the field, an app
/// event, the party-list event with A / B, an app event, return to the field; A+0x3C = B+0xF byte after seq 1), seq 5 stores
/// A+0x3C into var, frees A and B and ends (1); other sequences: nothing (0).
pub fn event_247<H: SodateyaHooks>(h: &mut H, event: u32, seq: u32, w: u32) -> u32 {
    let gamesys = h.r32(w);
    let next = match h.r32(seq) {
        0 => {
            let fm = h.r32(w + 4);
            let ev = h.fun_021be8d4(gamesys, fm, 0, 0);
            h.fun_02012374(event, ev);
            1
        }
        1 => {
            let fm = h.r32(w + 4);
            let ev = h.fun_020144dc(gamesys, fm);
            h.fun_02012374(event, ev);
            let b = h.r32(w + 0xC);
            let v = h.r8(b + 0xF) as u32;
            let a = h.r32(w + 8);
            h.w32(a + 0x3C, v);
            2
        }
        2 => {
            let (fm, a, b) = (h.r32(w + 4), h.r32(w + 8), h.r32(w + 0xC));
            let ev = h.fun_02014910(gamesys, fm, a, b);
            h.fun_02012374(event, ev);
            3
        }
        3 => {
            let ev = h.fun_020145cc(gamesys);
            h.fun_02012374(event, ev);
            4
        }
        4 => {
            let fm = h.r32(w + 4);
            let ev = h.fun_021be92c(gamesys, fm, 0, 0, 1, 0, 0);
            h.fun_02012374(event, ev);
            5
        }
        5 => {
            let a = h.r32(w + 8);
            let v = h.r32(a + 0x3C);
            let var = h.r32(w + 0x10);
            h.w16(var, v as u16);
            let a = h.r32(w + 8);
            h.fun_020307b0(a);
            let b = h.r32(w + 0xC);
            h.fun_020307b0(b);
            return 1;
        }
        _ => return 0,
    };
    h.w32(seq, next);
    0
}

#[cfg(test)]
mod tests;
