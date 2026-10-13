//! F8d: overlay-10 script commands of `scrcmd_sodateya.c` (the script side of the daycare), Thumb code 0x0215CE98-0x0215D34C
//! (F1-SPLIT row 10.16 plus the tail of row 10.15; cmd 359 at 0x0215D34C is `scrcmd_musical.c`): commands 235..248 and 593
//! (15 command ids, 18 functions). The daycare itself is overlay-21 `sodateya.c` (F13), reached as `FUN_02188D50(field map)`.
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_sodateya.spec`, ported here, listed in
//! [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_sodateya`. [`COMMANDS`] maps the command ids to their
//! handlers, [`dispatch`] runs one by address and [`call_event`] the GMEVENT function of command 247.
//!
//! Callees outside the area are [`SodateyaHooks`] methods named by address (vm.c, the operand readers, the ctx getters,
//! the GMEVENT calls and the child-event call come from [`CoreHooks`]). Names of the overlay-21 daycare routines are inferred
//! from the command order (UNVERIFIED; F13).

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_CE98,
    0x0215_CEBC,
    0x0215_CEE8,
    0x0215_CF00,
    0x0215_CF54,
    0x0215_CF84,
    0x0215_CFB4,
    0x0215_CFD8,
    0x0215_CFFC,
    0x0215_D034,
    0x0215_D06C,
    0x0215_D0A4,
    0x0215_D0D4,
    0x0215_D110,
    0x0215_D14C,
    0x0215_D198,
    0x0215_D1E4,
    0x0215_D2A4,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (235, 0x0215_CE98),
    (236, 0x0215_CEBC),
    (237, 0x0215_CEE8),
    (238, 0x0215_CFB4),
    (239, 0x0215_CFD8),
    (240, 0x0215_CF54),
    (241, 0x0215_CF84),
    (242, 0x0215_CFFC),
    (243, 0x0215_D034),
    (244, 0x0215_D0A4),
    (245, 0x0215_D0D4),
    (246, 0x0215_D110),
    (247, 0x0215_CF00),
    (248, 0x0215_D06C),
    (593, 0x0215_D14C),
];

/// GMEVENT function of command 247 ([`event_247`], Thumb).
pub const EVENT_247: u32 = 0x0215_D1E5;
/// `__FILE__` string "scrcmd_sodateya.c" passed to `Heap_AllocDebug`.
pub const FILE: u32 = 0x0217_2900;
/// PokeParam ids the daycare commands read (`FUN_02017E1C(mon, id, 0)`).
pub const PP_SPECIES: u32 = 5;
pub const PP_GENDER: u32 = 0x6E;
pub const PP_FORM: u32 = 0x6F;
/// UNVERIFIED: read as "has a nickname" (a Nidoran without one shows no gender mark).
pub const PP_NICKNAMED: u32 = 0x75;
/// Nidoran F / Nidoran M (their species names carry the gender sign).
pub const SPECIES_NIDORAN_F: u32 = 0x1D;
pub const SPECIES_NIDORAN_M: u32 = 0x20;

/// Callees of the scrcmd_sodateya functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the scrcmd ctx getters, `FUN_020120F4` / `FUN_020122C0` / `FUN_020124E8` and the
/// child-event call come from [`CoreHooks`].
pub trait SodateyaHooks: CoreHooks {
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012934(gamedata)` (main): the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)` (main): the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012374(event, child)` (main): call a child event.
    fn fun_02012374(&mut self, event: u32, child: u32);
    /// C: `FUN_0200BDB0(save)` (main; command 247; UNVERIFIED).
    fn fun_0200bdb0(&mut self, save: u32);
    /// C: `FUN_0200CA1C(gamedata)` (main; UNVERIFIED: the Pokedex / a save sub object).
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200CA38(x)` (main; the party-select record of command 247 keeps it at +0x18; UNVERIFIED).
    fn fun_0200ca38(&mut self, x: u32) -> u32;
    /// C: `FUN_0201AA18(party)` (main): the party count.
    fn fun_0201aa18(&mut self, party: u32) -> u32;
    /// C: `FUN_02017E1C(mon, id, 0)` (main): a PokeParam value.
    fn fun_02017e1c(&mut self, mon: u32, id: u32, x: u32) -> u32;
    /// C: `FUN_02030734(heap, size, flag, file, line)` (`Heap_AllocDebug`).
    fn fun_02030734(&mut self, heap: u32, size: u32, flag: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_020144DC(gamesys, field map)` (main; UNVERIFIED: an app event).
    fn fun_020144dc(&mut self, gamesys: u32, fm: u32) -> u32;
    /// C: `FUN_02014910(gamesys, field map, A, B)` (main; UNVERIFIED: the party-list app event).
    fn fun_02014910(&mut self, gamesys: u32, fm: u32, a: u32, b: u32) -> u32;
    /// C: `FUN_020145CC(gamesys)` (main; UNVERIFIED).
    fn fun_020145cc(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02188D50(field map)` (overlay 21): the daycare object.
    fn fun_02188d50(&mut self, fm: u32) -> u32;
    /// C: `FUN_021BE8D4(gamesys, field map, 0, 0)` (overlay 21): the event that leaves the field (UNVERIFIED).
    fn fun_021be8d4(&mut self, gamesys: u32, fm: u32, a: u32, b: u32) -> u32;
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
    /// C: `FUN_021C3204(daycare, party, v)` (overlay-21 sodateya.c; command 240).
    fn fun_021c3204(&mut self, daycare: u32, party: u32, v: u32);
    /// C: `FUN_021C3270(daycare, v, party)` (overlay-21 sodateya.c; command 241).
    fn fun_021c3270(&mut self, daycare: u32, v: u32, party: u32);
    /// C: `FUN_021C32F4(daycare, party)` (overlay-21 sodateya.c; command 236).
    fn fun_021c32f4(&mut self, daycare: u32, party: u32);
    /// C: `FUN_021C335C(daycare)` (overlay-21 sodateya.c; command 237).
    fn fun_021c335c(&mut self, daycare: u32);
    /// C: `FUN_021C3400(daycare)` (overlay-21 sodateya.c; command 235).
    fn fun_021c3400(&mut self, daycare: u32) -> u32;
    /// C: `FUN_021C340C(daycare)` (overlay-21 sodateya.c; command 238).
    fn fun_021c340c(&mut self, daycare: u32) -> u32;
    /// C: `FUN_021C341C(daycare, slot)` (overlay-21 sodateya.c; command 244).
    fn fun_021c341c(&mut self, daycare: u32, slot: u32) -> u32;
    /// C: `FUN_021C3490(daycare, slot)` (overlay-21 sodateya.c; command 245).
    fn fun_021c3490(&mut self, daycare: u32, slot: u32) -> u32;
    /// C: `FUN_021C34B4(daycare, slot)` (overlay-21 sodateya.c; command 246).
    fn fun_021c34b4(&mut self, daycare: u32, slot: u32) -> u32;
    /// C: `FUN_021C34C4(daycare)` (overlay-21 sodateya.c; command 239).
    fn fun_021c34c4(&mut self, daycare: u32) -> u32;
    /// C: `FUN_021C34F8(daycare, slot)` (overlay-21 sodateya.c): the PokeParam of a daycare slot.
    fn fun_021c34f8(&mut self, daycare: u32, slot: u32) -> u32;
}

/// The daycare object of the script's GAMESYS (`FUN_02188D50(FUN_0201210C(GAMESYS))`).
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

/// C: `sub_0215cebc`: command 236: `FUN_021C32F4(daycare, party)`; 0.
pub fn cmd_236<H: SodateyaHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let fm = h.fun_0201210c(gs);
    let d = h.fun_02188d50(fm);
    h.fun_021c32f4(d, party);
    0
}

/// C: `sub_0215cee8`: command 237: `FUN_021C335C(daycare)`; 0.
pub fn cmd_237<H: SodateyaHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let d = daycare(h, ctx);
    h.fun_021c335c(d);
    0
}

/// C: `sub_0215cf00`: command 247: var; `FUN_02012934(GAMEDATA)` (unused), `FUN_0200BDB0(SAVE)`; the party-select event
/// [`party_select_event`]`(GAMESYS, field map, var)` runs as a child event; 1.
pub fn cmd_247<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    h.fun_02012934(gd);
    let save = h.fun_02012ebc(gd);
    h.fun_0200bdb0(save);
    let ev = party_select_event(h, gs, fm, var);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_0215cf54`: command 240: v = value; `FUN_021C3204(daycare, party, v)`; 0.
pub fn cmd_240<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let d = daycare(h, ctx);
    h.fun_021c3204(d, party, v);
    0
}

/// C: `sub_0215cf84`: command 241: v = value; `FUN_021C3270(daycare, v, party)`; 0.
pub fn cmd_241<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let party = h.fun_02012934(gd);
    let d = daycare(h, ctx);
    h.fun_021c3270(d, v, party);
    0
}

/// C: `sub_0215cfb4`: command 238: var = `FUN_021C340C(daycare)`; 0.
pub fn cmd_238<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let d = daycare(h, ctx);
    let v = h.fun_021c340c(d);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0215cfd8`: command 239: var = `FUN_021C34C4(daycare)`; 0.
pub fn cmd_239<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let d = daycare(h, ctx);
    let v = h.fun_021c34c4(d);
    h.w16(var, v as u16);
    0
}

/// Body shared by commands 242 / 243 / 248 (each has its own copy in the original): var, slot; var = PokeParam `id` of the
/// daycare mon in `slot`; 0.
fn slot_param<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32, id: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let slot = h.fun_02159b10(vm, ctx);
    let d = daycare(h, ctx);
    let mon = h.fun_021c34f8(d, slot);
    let v = h.fun_02017e1c(mon, id, 0);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0215cffc`: command 242: var, slot; var = the species of the daycare mon in `slot`; 0.
pub fn cmd_242<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    slot_param(h, vm, ctx, PP_SPECIES)
}

/// C: `sub_0215d034`: command 243: var, slot; var = the form (PokeParam 0x6F) of the daycare mon in `slot`; 0.
pub fn cmd_243<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    slot_param(h, vm, ctx, PP_FORM)
}

/// C: `sub_0215d06c`: command 248: var, slot; var = the gender of the daycare mon in `slot`; 0.
pub fn cmd_248<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    slot_param(h, vm, ctx, PP_GENDER)
}

/// C: `sub_0215d0a4`: command 244: var, slot; var = `FUN_021C341C(daycare, slot)`; 0.
pub fn cmd_244<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let slot = h.fun_02159b10(vm, ctx);
    let d = daycare(h, ctx);
    let v = h.fun_021c341c(d, slot);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0215d0d4`: command 245: var, slot; `FUN_021C34F8(daycare, slot)` (unused); var = `FUN_021C3490(daycare, slot)`; 0.
pub fn cmd_245<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let slot = h.fun_02159b10(vm, ctx);
    let d = daycare(h, ctx);
    h.fun_021c34f8(d, slot);
    let v = h.fun_021c3490(d, slot);
    h.w16(var, v as u16);
    0
}

/// C: `sub_0215d110`: command 246: var, slot; `FUN_021C34F8(daycare, slot)` (unused); var = `FUN_021C34B4(daycare, slot)`; 0.
pub fn cmd_246<H: SodateyaHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let slot = h.fun_02159b10(vm, ctx);
    let d = daycare(h, ctx);
    h.fun_021c34f8(d, slot);
    let v = h.fun_021c34b4(d, slot);
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

/// C: `FUN_overlay_d_10__0215d2a4`: the party-select event of command 247: A = `Heap_AllocDebug(4, 0x98, 0,
/// "scrcmd_sodateya.c", 0x22B)` = {party, .., +0x34 = 0x12, +0x38 = 0, +0x3C = 0} (UNVERIFIED: the party-list app's
/// parameters); B = `alloc(0x28)` (line 0x232) = {party, the word at A+0xC (not initialised here), GAMEDATA, byte +0xC 1, +0xD
/// 0, +0xE party count, +0xF 0, +0x10 0, +0x18 = `FUN_0200CA38(FUN_0200CA1C(GAMEDATA))`}; the GMEVENT `FUN_020122C0(GAMESYS, 0,
/// EVENT_247, 0x14)` gets the work {GAMESYS, field map, A, B, var}; returns the event.
pub fn party_select_event<H: SodateyaHooks>(h: &mut H, gs: u32, fm: u32, var: u32) -> u32 {
    let gd = h.fun_020120f4(gs);
    let party = h.fun_02012934(gd);
    let x = h.fun_0200ca1c(gd);
    let a = h.fun_02030734(4, 0x98, 0, FILE, 0x22B);
    h.w32(a, party);
    h.w32(a + 0x34, 0x12);
    h.w32(a + 0x38, 0);
    h.w32(a + 0x3C, 0);
    let b = h.fun_02030734(4, 0x28, 0, FILE, 0x232);
    let v = h.r32(a);
    h.w32(b, v);
    let v = h.r32(a + 0xC);
    h.w32(b + 4, v);
    h.w32(b + 8, gd);
    h.w8(b + 0xC, 1);
    let n = h.fun_0201aa18(party);
    h.w8(b + 0xE, n as u8);
    h.w8(b + 0xD, 0);
    h.w8(b + 0x10, 0);
    h.w8(b + 0xF, 0);
    let v = h.fun_0200ca38(x);
    h.w32(b + 0x18, v);
    let ev = h.fun_020122c0(gs, 0, EVENT_247, 0x14);
    let w = h.fun_020124e8(ev);
    h.w32(w, gs);
    h.w32(w + 4, fm);
    h.w32(w + 8, a);
    h.w32(w + 0xC, b);
    h.w32(w + 0x10, var);
    ev
}

/// Run the command handler at `handler` (bit 0 ignored) when it is one of [`COMMANDS`]; `None` otherwise.
pub fn dispatch<H: SodateyaHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x0215_CE98 => cmd_235(h, vm, ctx),
        0x0215_CEBC => cmd_236(h, vm, ctx),
        0x0215_CEE8 => cmd_237(h, vm, ctx),
        0x0215_CFB4 => cmd_238(h, vm, ctx),
        0x0215_CFD8 => cmd_239(h, vm, ctx),
        0x0215_CF54 => cmd_240(h, vm, ctx),
        0x0215_CF84 => cmd_241(h, vm, ctx),
        0x0215_CFFC => cmd_242(h, vm, ctx),
        0x0215_D034 => cmd_243(h, vm, ctx),
        0x0215_D0A4 => cmd_244(h, vm, ctx),
        0x0215_D0D4 => cmd_245(h, vm, ctx),
        0x0215_D110 => cmd_246(h, vm, ctx),
        0x0215_CF00 => cmd_247(h, vm, ctx),
        0x0215_D06C => cmd_248(h, vm, ctx),
        0x0215_D14C => cmd_593(h, vm, ctx),
        _ => return None,
    })
}

/// Run the GMEVENT function `f(event, seq, work)` when it is [`EVENT_247`]; `None` otherwise.
pub fn call_event<H: SodateyaHooks>(
    h: &mut H,
    f: u32,
    event: u32,
    seq: u32,
    work: u32,
) -> Option<u32> {
    (f == EVENT_247).then(|| event_247(h, event, seq, work))
}

#[cfg(test)]
mod tests;
