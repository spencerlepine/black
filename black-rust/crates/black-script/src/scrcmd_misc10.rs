//! F8 (third batch): the unanchored overlay-10 script commands: F1-SPLIT rows 10.33 (Thumb 0x0216A420-0x0216AC40: commands
//! 506..520, 549 / 550 object moves, 559, the seasonal-form helper 0x0216AAE4) and 10.35 (0x0216B504-0x0216B86C: commands
//! 528..543, save-block accessors), plus command 359 at 0x0215D34C (the first function of `scrcmd_musical.c`, X2; specced
//! only). 30 command ids, 33 functions.
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_misc10.spec`. The functions listed in
//! [`PORTED`] are ported here and checked with `tools/check-fn.sh black-script scrcmd_misc10`; the rest are the F8 port job's.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_A420,
    0x0216_A838,
    0x0216_AAE4,
    0x0216_B538,
    0x0216_B5AC,
];

/// Species with seasonal forms: Deerling, Sawsbuck.
pub const SPECIES_DEERLING: u32 = 0x249;
pub const SPECIES_SAWSBUCK: u32 = 0x24A;
/// PokeParam ids (`FUN_02017E1C(mon, id, 0)`).
pub const PP_SPECIES: u32 = 5;
pub const PP_FORM: u32 = 0x6F;

/// Callees of the scrcmd_misc10 functions that this area does not port, named by address (meaning per the spec).
pub trait Misc10Hooks: MemEnv {
    /// C: `FUN_02159AE8` (script_sys `cmd_var_addr`): the address of the next var operand.
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_0215A5DC` (script_sys `ctx_gamesys`).
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A5E4` (script_sys `ctx_gamedata`).
    fn fun_0215a5e4(&mut self, ctx: u32) -> u32;
    /// C: `FUN_0215A5F4` (script_sys `ctx_sw`).
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32;
    /// C: `FUN_020120F4(gamesys)` (main): the GAMEDATA.
    fn fun_020120f4(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)` (main): the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012F50(gamedata)` (main): a GAMEDATA record (UNVERIFIED).
    fn fun_02012f50(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_021D0A48(gamesys, record)` (overlay 21; UNVERIFIED).
    fn fun_021d0a48(&mut self, gamesys: u32, rec: u32);
    /// C: `FUN_0200C0F0(save)` (main, a save-block getter thunk: block 0x34).
    fn fun_0200c0f0(&mut self, save: u32) -> u32;
    /// C: `FUN_0200C224(block)` (main; UNVERIFIED: a level / count 0..).
    fn fun_0200c224(&mut self, block: u32) -> u32;
    /// C: `FUN_0200C22C(block, n)` (main; UNVERIFIED: set it).
    fn fun_0200c22c(&mut self, block: u32, n: u32);
    /// C: `FUN_0200C33C(block, i)` (main; UNVERIFIED).
    fn fun_0200c33c(&mut self, block: u32, i: u32) -> u32;
    /// C: `FUN_02027138(n)` (main; UNVERIFIED).
    fn fun_02027138(&mut self, n: u32);
    /// C: `FUN_0216D89C(mmdl, &pos)` (overlay-10 fldmmdl.c): set the object's position (VecFx32).
    fn fun_0216d89c(&mut self, mmdl: u32, pos: &[u8; 12]);
    /// C: `FUN_0200CA1C(x)` (main): the Pokedex object.
    fn fun_0200ca1c(&mut self, x: u32) -> u32;
    /// C: `FUN_0201AA18(party)` (main): the party count.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_0201AB28(party, i)` (main): the PokeParam of party slot `i`.
    fn fun_0201ab28(&mut self, party: u32, i: u32) -> u32;
    /// C: `FUN_02017E1C(mon, id, 0)` (main): a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_02017988(mon, form)` (main): set the form.
    fn fun_02017988(&mut self, mon: u32, form: u32);
    /// C: `FUN_0200CFA0(dex, mon)` (main): register the mon in the Pokedex.
    fn fun_0200cfa0(&mut self, dex: u32, mon: u32);
}

/// C: `sub_0216a420`: command 506: `FUN_021D0A48(GAMESYS, FUN_02012F50(GAMEDATA))`; 0.
pub fn cmd_506<H: Misc10Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gamesys = h.fun_0215a5dc(ctx);
    let gamedata = h.fun_020120f4(gamesys);
    let rec = h.fun_02012f50(gamedata);
    h.fun_021d0a48(gamesys, rec);
    0
}

/// C: `sub_0216a838`: the GMEVENT function of command 549 (`w` = {s32 dist, u16 frames, u16 done, MMDL, VecFx32 start}): one
/// frame of a vertical move: done += 1; y = start.y + dist * done / frames (`_s32_div_f`, truncating); set the object's
/// position; finished (1) once done >= frames.
pub fn event_549<H: Misc10Hooks>(h: &mut H, _event: u32, _seq: u32, w: u32) -> u32 {
    let done = h.r16(w + 6).wrapping_add(1);
    h.w16(w + 6, done);
    let mut pos = [0u8; 12];
    for (i, b) in pos.iter_mut().enumerate() {
        *b = h.r8(w + 0xC + i as u32);
    }
    let prod = (h.r16(w + 6) as u32).wrapping_mul(h.r32(w));
    let frames = h.r16(w + 4) as i32;
    let q = (prod as i32).wrapping_div(frames);
    let y = u32::from_le_bytes(pos[4..8].try_into().unwrap()).wrapping_add(q as u32);
    pos[4..8].copy_from_slice(&y.to_le_bytes());
    let mmdl = h.r32(w + 8);
    h.fun_0216d89c(mmdl, &pos);
    (h.r16(w + 6) >= h.r16(w + 4)) as u32
}

/// C: `FUN_0216AAE4`: the seasonal forms (`season` 0..3, anything else = 0): every party Deerling / Sawsbuck
/// whose form differs gets the season's form, and the first one of each species changed is registered in the Pokedex of `x`.
pub fn update_season_forms<H: Misc10Hooks>(h: &mut H, x: u32, party: u32, season: u32) {
    let dex = h.fun_0200ca1c(x);
    let form = if season <= 3 { season } else { 0 };
    let (mut seen_deerling, mut seen_sawsbuck) = (false, false);
    let n = h.fun_0201aa18(party) as i32;
    let mut i = 0i32;
    while i < n {
        let mon = h.fun_0201ab28(party, i as u32);
        let species = h.fun_02017e1c(mon, PP_SPECIES, 0);
        let f = h.fun_02017e1c(mon, PP_FORM, 0);
        let seen = if species == SPECIES_DEERLING && f != form {
            Some(&mut seen_deerling)
        } else if species == SPECIES_SAWSBUCK && f != form {
            Some(&mut seen_sawsbuck)
        } else {
            None
        };
        if let Some(seen) = seen {
            h.fun_02017988(mon, form & 0xFFFF);
            if !*seen {
                h.fun_0200cfa0(dex, mon);
                *seen = true;
            }
        }
        i += 1;
    }
}

/// C: `sub_0216b538`: command 543: with the save block 0x34 `s`: n = `FUN_0200C224(s)`; n < 5 (signed) -> `FUN_0200C22C(s,
/// n + 1)` and `FUN_02027138((n + 1) & 0xFF)`; 0.
pub fn cmd_543<H: Misc10Hooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gamedata = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gamedata);
    let s = h.fun_0200c0f0(save);
    let n = h.fun_0200c224(s);
    if (n as i32) < 5 {
        h.fun_0200c22c(s, n.wrapping_add(1));
        h.fun_02027138(n.wrapping_add(1) & 0xFF);
    }
    0
}

/// C: `sub_0216b5ac`: command 531: three vars = `FUN_0200C33C(s, 0 / 1 / 2)` of the save block 0x34; 0.
pub fn cmd_531<H: Misc10Hooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    let gamedata = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gamedata);
    let s = h.fun_0200c0f0(save);
    let vars = [
        h.fun_02159ae8(vm, ctx),
        h.fun_02159ae8(vm, ctx),
        h.fun_02159ae8(vm, ctx),
    ];
    for (i, var) in vars.into_iter().enumerate() {
        let v = h.fun_0200c33c(s, i as u32);
        h.w16(var, v as u16);
    }
    0
}

#[cfg(test)]
mod tests;
