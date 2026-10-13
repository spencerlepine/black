//! F8e: overlay-21 `scrcmd_proc_fld.c`, Thumb code 0x021B4BEC-0x021B5190 (F1-SPLIT row 21.47 plus the tail of row 21.46):
//! commands that start field events and app procs and run them as child events (20 command ids, 21 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_proc_fld.spec` (header: command-id
//! table, roles, why the file starts at 0x021B4BEC), ported here, listed in [`PORTED`] and checked with
//! `tools/check-fn.sh black-script scrcmd_proc_fld`. [`COMMANDS`] maps the command ids to their handlers, [`dispatch`] runs
//! one by address and [`call_callback`] the app-proc result callback of command 335.
//!
//! Callees outside the area are [`ProcFldHooks`] methods named by address (vm.c, the operand readers, the ctx getters and
//! the child-event call come from [`CoreHooks`]). The events / procs the commands start are named by address (UNVERIFIED
//! roles; F10 / G2).

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_4BEC,
    0x021B_4C14,
    0x021B_4C34,
    0x021B_4C54,
    0x021B_4C98,
    0x021B_4CDC,
    0x021B_4D14,
    0x021B_4D60,
    0x021B_4E58,
    0x021B_4E9C,
    0x021B_4EE0,
    0x021B_4F1C,
    0x021B_4F68,
    0x021B_4FB0,
    0x021B_5008,
    0x021B_503C,
    0x021B_5094,
    0x021B_50D0,
    0x021B_50F8,
    0x021B_5128,
    0x021B_5154,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (188, 0x021B_5128),
    (314, 0x021B_4BEC),
    (333, 0x021B_4FB0),
    (335, 0x021B_4D60),
    (338, 0x021B_4F68),
    (342, 0x021B_4EE0),
    (351, 0x021B_4C14),
    (352, 0x021B_4C34),
    (353, 0x021B_4C54),
    (354, 0x021B_4C98),
    (358, 0x021B_4CDC),
    (376, 0x021B_503C),
    (377, 0x021B_5094),
    (380, 0x021B_50D0),
    (381, 0x021B_50F8),
    (411, 0x021B_4F1C),
    (444, 0x021B_5154),
    (536, 0x021B_4E58),
    (537, 0x021B_4E9C),
    (580, 0x021B_5008),
];

/// `__FILE__` string "scrcmd_proc_fld.c" passed to `Heap_AllocDebug`.
pub const FILE: u32 = 0x021D_E220;
/// Procs (code addresses in overlay 10) and event kinds of commands 353 / 354 / 358 (`FUN_020124AC(GAMESYS, kind, proc, ..)`).
pub const PROC_353: u32 = 0x0215_4C21;
pub const KIND_353: u32 = 6;
pub const PROC_354: u32 = 0x0215_42C1;
pub const KIND_354: u32 = 4;
pub const PROC_358: u32 = 0x0215_4695;
pub const KIND_358: u32 = 1;
/// App (overlay id, proc table) of command 335 (`FUN_02014840`) and its result callback ([`cb_335`], Thumb).
pub const APP_335: (u32, u32) = (0xAA, 0x021E_F78C);
pub const CB_335: u32 = 0x021B_4D15;
/// Mode of the proc work of command 335 for which the battle box is used and the result callback writes it back.
pub const MODE_335_BOX: u32 = 4;
/// App (overlay id, proc table) of command 338 (`FUN_020146F8`).
pub const APP_338: (u32, u32) = (0xDB, 0x0218_5F0C);

/// Callees of the scrcmd_proc_fld functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the ctx getters and the child-event call come from [`CoreHooks`].
pub trait ProcFldHooks: CoreHooks {
    /// C: `FUN_0215898C(sw, ev)` (script_sys): switch the script's event to `ev`.
    fn fun_0215898c(&mut self, sw: u32, ev: u32);
    /// C: `FUN_02158F54(sw)` (script_sys).
    fn fun_02158f54(&mut self, sw: u32) -> u32;
    /// C: `FUN_02158F58(sw)` (script_sys): a record whose word +4 is the field map's ... (UNVERIFIED).
    fn fun_02158f58(&mut self, sw: u32) -> u32;
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012138(gamesys)` (main; UNVERIFIED role).
    fn fun_02012138(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_021896F4(x)` (overlay 21; UNVERIFIED role): the value command 314 stores.
    fn fun_021896f4(&mut self, x: u32) -> u32;
    /// C: `FUN_020124AC(gamesys, kind, proc, &rec)` (main): an event built from the 8-byte stack record `rec`.
    fn fun_020124ac(&mut self, gamesys: u32, kind: u32, proc_table: u32, rec: &[u8; 8]) -> u32;
    /// C: `FUN_020124AC(gamesys, kind, proc, param)` (main) with a parameter address instead of a stack record (command
    /// 358 passes the field map itself).
    fn fun_020124ac_at(&mut self, gamesys: u32, kind: u32, proc_table: u32, param: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)` (main): the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201283C(gamedata)` (main).
    fn fun_0201283c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02009CE4(x)` (main): the value command 351 stores.
    fn fun_02009ce4(&mut self, x: u32) -> u32;
    /// C: `FUN_02009A34(x)` (main): the value command 352 stores.
    fn fun_02009a34(&mut self, x: u32) -> u32;
    /// C: `FUN_0201293C(gamedata)` (main).
    fn fun_0201293c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012934(gamedata)` (main): the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0201292C(gamedata)` (main): the bag (UNVERIFIED).
    fn fun_0201292c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012944(gamedata)` (main).
    fn fun_02012944(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200873C(save)` (main).
    fn fun_0200873c(&mut self, save: u32) -> u32;
    /// C: `FUN_0200CA1C(gamedata)` (main): the Pokedex object (UNVERIFIED).
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200CA38(x)` (main).
    fn fun_0200ca38(&mut self, x: u32) -> u32;
    /// C: `FUN_0200BAC8(save)` (main): a save block (UNVERIFIED: the battle box).
    fn fun_0200bac8(&mut self, save: u32) -> u32;
    /// C: `FUN_0200BAF4(block, heap)` (main): a copy of the battle box (UNVERIFIED).
    fn fun_0200baf4(&mut self, block: u32, heap: u32) -> u32;
    /// C: `FUN_0200BBFC(block, heap)` (main; UNVERIFIED).
    fn fun_0200bbfc(&mut self, block: u32, heap: u32) -> u32;
    /// C: `FUN_0200BB58(block, data)` (main; UNVERIFIED: store the proc's result data).
    fn fun_0200bb58(&mut self, block: u32, data: u32);
    /// C: `Heap_AllocDebug(heap, size, from_end, file, line)` (`0x02030734`).
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_02014840(gamesys, fm, ovl, proc table, param, cb, cbwork)` (main): the app event (leave the field, run the app,
    /// come back, call `cb(cbwork)`).
    #[allow(clippy::too_many_arguments)]
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
    /// C: `FUN_020146F8(gamesys, fm, ovl, proc table, param)` (main): an app event.
    fn fun_020146f8(&mut self, gamesys: u32, fm: u32, ovl: u32, proc_table: u32, param: u32)
        -> u32;
    /// C: `FUN_021CD2A4(gamesys, fm, a, mode, var)` (overlay 21): an event.
    fn fun_021cd2a4(&mut self, gamesys: u32, fm: u32, a: u32, mode: u32, var: u32) -> u32;
    /// C: `FUN_0215F8A8(gamesys, v)` (overlay 10): an event.
    fn fun_0215f8a8(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_02009210(save)` (main).
    fn fun_02009210(&mut self, save: u32) -> u32;
    /// C: `FUN_020090D8(x, on)` (main).
    fn fun_020090d8(&mut self, x: u32, on: u32);
    /// C: `FUN_02168BBC(gamesys, v)` (overlay 21): an event.
    fn fun_02168bbc(&mut self, gamesys: u32, v: u32) -> u32;
    /// C: `FUN_021C5BD0(gamesys, flag, var)` (overlay 21): an event.
    fn fun_021c5bd0(&mut self, gamesys: u32, flag: u32, var: u32) -> u32;
    /// C: `FUN_021D30B8(gamesys, var)` (overlay 21, scrcmd_misc21 area's neighbour): an event.
    fn fun_021d30b8(&mut self, gamesys: u32, var: u32) -> u32;
    /// C: `FUN_02188CE8(fm)` (overlay 21).
    fn fun_02188ce8(&mut self, fm: u32) -> u32;
    /// C: `FUN_021AA0A0(x, a, b, c)` (overlay 21): an event.
    fn fun_021aa0a0(&mut self, x: u32, a: u32, b: u32, c: u32) -> u32;
    /// C: `FUN_021BE92C(gamesys, fm, a, b, c, d, e)` (overlay 21): the event that returns to the field (UNVERIFIED).
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
    /// C: `FUN_02012A44(gamedata)` (main).
    fn fun_02012a44(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0216EFCC(x)` (overlay 21): the value command 380 stores.
    fn fun_0216efcc(&mut self, x: u32) -> u32;
    /// C: `FUN_0216E95C(gamesys, fm, n)` (overlay 21): an event.
    fn fun_0216e95c(&mut self, gamesys: u32, fm: u32, n: u32) -> u32;
    /// C: `FUN_021AAB48(x)` (overlay 21): the value command 188 stores.
    fn fun_021aab48(&mut self, x: u32) -> u32;
    /// C: `FUN_0200D418(save)` (main).
    fn fun_0200d418(&mut self, save: u32) -> u32;
    /// C: `FUN_0200D488(x, i)` (main): nonzero when entry `i` is set.
    fn fun_0200d488(&mut self, x: u32, i: u32) -> u32;
    /// C: `FUN_0215E440(gamedata, i)` (overlay 10).
    fn fun_0215e440(&mut self, gamedata: u32, i: u32);
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

/// C: `sub_021b4c14`: command 351: `*var = FUN_02009CE4(FUN_0201283C(GAMEDATA))`; 0.
pub fn cmd_351<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_0201283c(gd);
    let v = h.fun_02009ce4(x);
    h.w16(var, v as u16);
    0
}

/// C: `sub_021b4c34`: command 352: `*var = FUN_02009A34(FUN_0201283C(GAMEDATA))`; 0.
pub fn cmd_352<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let var = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_0201283c(gd);
    let v = h.fun_02009a34(x);
    h.w16(var, v as u16);
    0
}

/// Commands 353 / 354: the event `FUN_020124AC(GAMESYS, kind, proc, &{field map, 0})` runs as a child event; 1.
fn event_fm_rec<H: ProcFldHooks>(h: &mut H, ctx: u32, kind: u32, proc_table: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gamesys = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gamesys);
    let mut rec = [0u8; 8];
    rec[..4].copy_from_slice(&fm.to_le_bytes());
    let ev = h.fun_020124ac(gamesys, kind, proc_table, &rec);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `FUN_overlay_d_21__021b4c54`: command 353: the event `FUN_020124AC(GAMESYS, 6, 0x02154C21, &{field map, 0})` runs as a
/// child event; 1.
pub fn cmd_353<H: ProcFldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    event_fm_rec(h, ctx, KIND_353, PROC_353)
}

/// C: `sub_021b4c98`: command 354: the event `FUN_020124AC(GAMESYS, 4, 0x021542C1, &{field map, 0})` runs as a child event; 1.
pub fn cmd_354<H: ProcFldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    event_fm_rec(h, ctx, KIND_354, PROC_354)
}

/// C: `sub_021b4cdc`: command 358: the event `FUN_020124AC(GAMESYS, 1, 0x02154695, field map)` (the field map itself as the
/// parameter pointer) runs as a child event; 1.
pub fn cmd_358<H: ProcFldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gamesys = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gamesys);
    let ev = h.fun_020124ac_at(gamesys, KIND_358, PROC_358, fm);
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

/// C: `sub_021b4d60`: command 335: var, m; the app 0xAA (proc table 0x021EF78C) runs through the app event `FUN_02014840`
/// with P = `alloc(0x2C)` = {GAMEDATA, `FUN_0201293C` / `FUN_02012934` (party) / `FUN_0201292C` / `FUN_02012944`(GAMEDATA),
/// +0x14 0, +0x18 `FUN_0200873C(SAVE)`, +0x1C `FUN_0200CA38(FUN_0200CA1C(GAMEDATA))`, +0x24 m} (m == 4: P+8 = the battle-box
/// copy `FUN_0200BAF4(S, 4)`, P+0x14 = `FUN_0200BBFC(S, 3)`) and the result callback [`cb_335`] on W = `alloc(0xC)` =
/// {GAMEDATA, var, P}; it runs as a child event; 1.
pub fn cmd_335<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    let save = h.fun_02012ebc(gd);
    let dex = h.fun_0200ca1c(gd);
    let fm = h.fun_0201210c(gs);
    let var = h.fun_02159ae8(vm, ctx);
    let m = h.fun_02159b10(vm, ctx);
    let p = h.fun_02030734(4, 0x2C, 0, FILE, 0x7C);
    h.w32(p, gd);
    let v = h.fun_0201293c(gd);
    h.w32(p + 4, v);
    let v = h.fun_02012934(gd);
    h.w32(p + 8, v);
    let v = h.fun_0201292c(gd);
    h.w32(p + 0xC, v);
    let v = h.fun_02012944(gd);
    h.w32(p + 0x10, v);
    let v = h.fun_0200873c(save);
    h.w32(p + 0x18, v);
    let v = h.fun_0200ca38(dex);
    h.w32(p + 0x1C, v);
    h.w32(p + 0x24, m);
    h.w32(p + 0x14, 0);
    if m == MODE_335_BOX {
        let s = h.fun_0200bac8(save);
        let v = h.fun_0200baf4(s, 4);
        h.w32(p + 8, v);
        let v = h.fun_0200bbfc(s, 3);
        h.w32(p + 0x14, v);
    }
    let w = h.fun_02030734(4, 0xC, 0, FILE, 0x90);
    h.w32(w, gd);
    h.w32(w + 4, var);
    h.w32(w + 8, p);
    let ev = h.fun_02014840(gs, fm, APP_335.0, APP_335.1, p, CB_335, w);
    h.fun_0215897c(sw, ev);
    1
}

/// Commands 536 / 537: var; the event `FUN_021CD2A4(GAMESYS, field map, 0, mode, var)` runs as a child event; 1.
fn cd2a4_event<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32, mode: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let var = h.fun_02159ae8(vm, ctx);
    let ev = h.fun_021cd2a4(gs, fm, 0, mode, var);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b4e58`: command 536: var; the event `FUN_021CD2A4(GAMESYS, field map, 0, 5, var)` runs as a child event; 1.
pub fn cmd_536<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    cd2a4_event(h, vm, ctx, 5)
}

/// C: `sub_021b4e9c`: command 537: var; the event `FUN_021CD2A4(GAMESYS, field map, 0, 6, var)` runs as a child event; 1.
pub fn cmd_537<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    cd2a4_event(h, vm, ctx, 6)
}

/// C: `sub_021b4ee0`: command 342: v; the script's event switches to `FUN_0215F8A8(GAMESYS, v)` (`FUN_0215898C`) and the VM
/// stops (`FUN_02011290`); 1. The field map getter's result is unused.
pub fn cmd_342<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    h.fun_0201210c(gs);
    let v = h.fun_02159b10(vm, ctx);
    let ev = h.fun_0215f8a8(gs, v);
    h.fun_0215898c(sw, ev);
    h.fun_02011290(vm);
    1
}

/// C: `sub_021b4f1c`: command 411: v; `FUN_020090D8(FUN_02009210(SAVE), 1)`; the event `FUN_02168BBC(GAMESYS, v)` runs as a
/// child event; 1.
pub fn cmd_411<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gd);
    let s = h.fun_02009210(save);
    let v = h.fun_02159b10(vm, ctx);
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    h.fun_020090d8(s, 1);
    let ev = h.fun_02168bbc(gs, v);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b4f68`: command 338: the app event `FUN_020146F8(GAMESYS, field map, 0xDB, 0x02185F0C, GAMEDATA)` runs as a
/// child event; 1.
pub fn cmd_338<H: ProcFldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let fm = h.fun_0201210c(gs);
    let gd = h.fun_020120f4(gs);
    let ev = h.fun_020146f8(gs, fm, APP_338.0, APP_338.1, gd);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b4fb0`: command 333: v, var; the event `FUN_021C5BD0(GAMESYS, v == 1, var)` runs as a child event; 1. The
/// GAMEDATA / save / field map getters' results are unused.
pub fn cmd_333<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gs);
    h.fun_02012ebc(gd);
    h.fun_0201210c(gs);
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let ev = h.fun_021c5bd0(gs, (v == 1) as u32, var);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b5008`: command 580: var; the event `FUN_021D30B8(GAMESYS, var)` runs as a child event; 1.
pub fn cmd_580<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let ev = h.fun_021d30b8(gs, var);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b503c`: command 376: a, b, c; `FUN_02158F54(SW)`; R = `FUN_02158F58(SW)`; the event `FUN_021AA0A0(
/// FUN_02188CE8(R+4 word), a, (u8)b, c)` runs as a child event; 1.
pub fn cmd_376<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let a = h.fun_02159b10(vm, ctx);
    let b = h.fun_02159b10(vm, ctx);
    let c = h.fun_02159b10(vm, ctx);
    h.fun_02158f54(sw);
    let r = h.fun_02158f58(sw);
    let x = h.r32(r + 4);
    let x = h.fun_02188ce8(x);
    let ev = h.fun_021aa0a0(x, a, b & 0xFF, c);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b5094`: command 377: the event `FUN_021BE92C(GAMESYS, field map, 0, 0, 1, 0, 0)` runs as a child event; 1.
pub fn cmd_377<H: ProcFldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let ev = h.fun_021be92c(gs, fm, 0, 0, 1, 0, 0);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b50d0`: command 380: `*var = FUN_0216EFCC(FUN_02012A44(GAMEDATA))`; 0.
pub fn cmd_380<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012a44(gd);
    let var = h.fun_02159ae8(vm, ctx);
    let v = h.fun_0216efcc(x);
    h.w16(var, v as u16);
    0
}

/// C: `sub_021b50f8`: command 381: the event `FUN_0216E95C(GAMESYS, field map, 4)` runs as a child event; 1.
pub fn cmd_381<H: ProcFldHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let ev = h.fun_0216e95c(gs, fm, 4);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b5128`: command 188: `*var = FUN_021AAB48(FUN_02188CE8(field map))`; 0.
pub fn cmd_188<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let var = h.fun_02159ae8(vm, ctx);
    let x = h.fun_02188ce8(fm);
    let v = h.fun_021aab48(x);
    h.w16(var, v as u16);
    0
}

/// C: `sub_021b5154`: command 444: i = (u8)value; entry i of `FUN_0200D418(SAVE)` not set (`FUN_0200D488 == 0`) ->
/// `FUN_0215E440(GAMEDATA, i)`; 0.
pub fn cmd_444<H: ProcFldHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let save = h.fun_02012ebc(gd);
    let s = h.fun_0200d418(save);
    let i = h.fun_02159b10(vm, ctx) & 0xFF;
    if h.fun_0200d488(s, i) == 0 {
        h.fun_0215e440(gd, i);
    }
    0
}

/// Run the command handler at `handler` (bit 0 ignored) when it is one of [`COMMANDS`]; `None` otherwise.
pub fn dispatch<H: ProcFldHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x021B_5128 => cmd_188(h, vm, ctx),
        0x021B_4BEC => cmd_314(h, vm, ctx),
        0x021B_4FB0 => cmd_333(h, vm, ctx),
        0x021B_4D60 => cmd_335(h, vm, ctx),
        0x021B_4F68 => cmd_338(h, vm, ctx),
        0x021B_4EE0 => cmd_342(h, vm, ctx),
        0x021B_4C14 => cmd_351(h, vm, ctx),
        0x021B_4C34 => cmd_352(h, vm, ctx),
        0x021B_4C54 => cmd_353(h, vm, ctx),
        0x021B_4C98 => cmd_354(h, vm, ctx),
        0x021B_4CDC => cmd_358(h, vm, ctx),
        0x021B_503C => cmd_376(h, vm, ctx),
        0x021B_5094 => cmd_377(h, vm, ctx),
        0x021B_50D0 => cmd_380(h, vm, ctx),
        0x021B_50F8 => cmd_381(h, vm, ctx),
        0x021B_4F1C => cmd_411(h, vm, ctx),
        0x021B_5154 => cmd_444(h, vm, ctx),
        0x021B_4E58 => cmd_536(h, vm, ctx),
        0x021B_4E9C => cmd_537(h, vm, ctx),
        0x021B_5008 => cmd_580(h, vm, ctx),
        _ => return None,
    })
}

/// Run the app-proc result callback `cb(w)` when it is [`CB_335`]; false otherwise.
pub fn call_callback<H: ProcFldHooks>(h: &mut H, cb: u32, w: u32) -> bool {
    if cb != CB_335 {
        return false;
    }
    cb_335(h, w);
    true
}

#[cfg(test)]
mod tests;
