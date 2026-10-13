//! F8 (second batch): overlay-21 field script commands, Thumb code 0x021AE948-0x021B0084 (the first part of F1-SPLIT row
//! 21.42): trainer-eye / trainer-battle starts, field events and sounds, the WORDSET commands 76..99 / 538 / 539, name / string
//! queries and warps (80 command ids, 97 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_fld.spec`; its header has the command-id
//! table and a one-line role per function. The functions listed in [`PORTED`] are ported here and checked with
//! `tools/check-fn.sh black-script scrcmd_fld`; the rest are the F8 port job's.
//!
//! Pointers are `u32` addresses in the [`Mem`](crate::hooks::Mem) window; the callees are [`FldHooks`] methods named by address.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_E9CC,
    0x021A_EBB0,
    0x021A_ECC0,
    0x021A_EEE0,
    0x021A_F1E8,
    0x021A_F21C,
];

/// SCRIPT_WORK field (`FUN_02159124`): the 0x1C-byte trainer-eye records start at SW + 0x44; record +0x14 is read by command
/// 131.
pub const TRAINER_REC_14: u32 = 0x14;

/// Callees of the scrcmd_fld functions that this area does not port, named by address (meaning per the spec).
pub trait FldHooks: MemEnv {
    /// C: `FUN_02159AE8` (script_sys `cmd_var_addr`): address of the var whose id is the next script word.
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_02159B10` (script_sys `cmd_var_value`): the next script word read as a var id or a literal.
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_0215A5DC` (script_sys `ctx_gamesys`).
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A5F4` (script_sys `ctx_sw`): the SCRIPT_WORK.
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02159124` (script_sys `sw_record`): address of the `i`-th 0x1C-byte record at SW + 0x44.
    fn fun_02159124(&mut self, sw: u32, i: u32) -> u32;
    /// C: `FUN_02158F8C` (script_sys): the script id (u16 SW + 4).
    fn fun_02158f8c(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F88` (script_sys): address SW + 0x40 (a byte of sound flags, bits 0..4).
    fn fun_02158f88(&mut self, sw: u32) -> u32;
    /// C: `FUN_0215A460(id)` (overlay 10, event_poke_status / scrcmd area 10.13; meaning UNVERIFIED).
    fn fun_0215a460(&mut self, id: u32) -> u32;
    /// C: `FUN_0215A4A0(id)` (overlay 10; UNVERIFIED).
    fn fun_0215a4a0(&mut self, id: u32) -> u32;
    /// C: `FUN_0215A4B4(id)` (overlay 10; UNVERIFIED).
    fn fun_0215a4b4(&mut self, id: u32) -> u32;
    /// C: `FUN_0202A230(v, k)` (main; a property test of v; UNVERIFIED).
    fn fun_0202a230(&mut self, v: u32, k: u32) -> u32;
    /// C: `FUN_02005F38()` (main; nonzero while something (a sound?) is busy; UNVERIFIED).
    fn fun_02005f38(&mut self) -> u32;
    /// C: `FUN_02006224(i)` (main sound; 0 once sound slot i finished; UNVERIFIED).
    fn fun_02006224(&mut self, i: u32) -> u32;
    /// C: `FUN_02006148(x)` (main sound; the slot / player of sound x; UNVERIFIED).
    fn fun_02006148(&mut self, x: u32) -> u32;
}

/// C: `sub_021ae9cc`: command 131: var = the u16 at trainer-eye record 0 +0x14 (`FUN_02159124(SW, 0)`).
pub fn cmd_trainer_rec_14<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let rec = h.fun_02159124(sw, 0);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.r32(rec + TRAINER_REC_14);
    h.w16(var, v as u16);
    0
}

/// C: `sub_021aebb0`: command 136: three vars = (0, 2, 0x18) when `FUN_0215A4B4(FUN_0215A460(script id))` is 0 (u16), else
/// (3, 5, 6) when `FUN_0215A4A0(script id)` is 0 (u16), else (7, 9, 0xA).
pub fn cmd_three_vars<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let id = h.fun_02158f8c(sw);
    let a = h.fun_02159ae8(vm, ctx);
    let b = h.fun_02159ae8(vm, ctx);
    let c = h.fun_02159ae8(vm, ctx);
    let x = h.fun_0215a460(id);
    let (va, vb, vc) = if h.fun_0215a4b4(x) & 0xFFFF == 0 {
        (0, 2, 0x18)
    } else if h.fun_0215a4a0(id) & 0xFFFF == 0 {
        (3, 5, 6)
    } else {
        (7, 9, 0xA)
    };
    h.w16(a, va);
    h.w16(b, vb);
    h.w16(c, vc);
    0
}

/// C: `sub_021aecc0`: command 146: v = value; var = `FUN_0202A230(v, 9)` ? 1 : `FUN_0202A230(v, 0xB)` ? 2 : 0.
pub fn cmd_classify_146<H: FldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let r = if h.fun_0202a230(v, 9) != 0 {
        1
    } else if h.fun_0202a230(v, 0xB) != 0 {
        2
    } else {
        0
    };
    h.w16(var, r);
    0
}

/// C: `sub_021aeee0`: wait callback of command 160: done (1) once `FUN_02005F38()` is 0. (The GAMESYS getter result is
/// unused, as in the original.)
pub fn wait_160<H: FldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5dc(ctx);
    h.fun_0215a5f4(ctx);
    (h.fun_02005f38() == 0) as u32
}

/// C: `FUN_overlay_d_21__021af1e8`: for every bit i < 5 set in the sound-flag byte at SW +0x40: `FUN_02006224(i) == 0` (that
/// sound finished) clears it.
pub fn sound_flags_update<H: FldHooks>(h: &mut H, sw: u32) {
    let p = h.fun_02158f88(sw);
    for i in 0..5u32 {
        let bit = 1u32 << i;
        if h.r8(p) as u32 & bit != 0 && h.fun_02006224(i) == 0 {
            let v = h.r8(p) & !(bit as u8);
            h.w8(p, v);
        }
    }
}

/// C: `FUN_overlay_d_21__021af21c`: set bit `FUN_02006148(x)` of the sound-flag byte at SW +0x40 (a shift of 32 or more sets
/// nothing, as Thumb `lsls` by register).
pub fn sound_flags_add<H: FldHooks>(h: &mut H, sw: u32, x: u32) {
    let p = h.fun_02158f88(sw);
    let n = h.fun_02006148(x) & 0xFF;
    let bit = if n < 32 { 1u32 << n } else { 0 };
    let v = h.r8(p) | bit as u8;
    h.w8(p, v);
}

#[cfg(test)]
mod tests;
