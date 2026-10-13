//! F8e: overlay-21 `scrcmd_encount.c`, Thumb code 0x021B5190-0x021B5700 (F1-SPLIT row 21.48 plus its static helper
//! 0x021B5190): the encounter-table query 561 and the field effect object commands 319..328 / 418..420 (14 command ids, 19
//! functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_encount.spec`, ported here, listed in
//! [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_encount`. [`COMMANDS`] maps the command ids to their
//! handlers, [`dispatch`] runs one by address, [`call_event`] the GMEVENT functions of commands 319 / 325 and [`init_proc`]
//! the init proc of `script_sys::INIT_PROCS` slot 0.
//!
//! Callees outside the area are [`EncountHooks`] methods named by address (vm.c, the operand readers, the ctx getters and
//! the GMEVENT calls come from [`CoreHooks`]). The field effect object E (`FUN_02188C58(field map)`) and its 0x0218Exxx /
//! 0x0218Fxxx methods are the fieldmap camera / effect code (overlay 21, F2 / F10; roles UNVERIFIED).

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_5190,
    0x021B_51BC,
    0x021B_52A4,
    0x021B_52E0,
    0x021B_52FC,
    0x021B_536C,
    0x021B_538C,
    0x021B_53A4,
    0x021B_53BC,
    0x021B_542C,
    0x021B_5498,
    0x021B_54D4,
    0x021B_54FC,
    0x021B_5528,
    0x021B_55C8,
    0x021B_55F0,
    0x021B_5610,
    0x021B_5650,
    0x021B_568C,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (319, 0x021B_52FC),
    (320, 0x021B_536C),
    (321, 0x021B_538C),
    (322, 0x021B_53A4),
    (323, 0x021B_53BC),
    (324, 0x021B_5498),
    (325, 0x021B_54FC),
    (326, 0x021B_542C),
    (327, 0x021B_54D4),
    (328, 0x021B_5528),
    (418, 0x021B_5610),
    (419, 0x021B_5650),
    (420, 0x021B_568C),
    (561, 0x021B_51BC),
];

/// Init proc of `script_sys::INIT_PROCS` slot 0 ([`init_proc`], Thumb).
pub const INIT_PROC: u32 = 0x021B_52E1;
/// GMEVENT function of command 319 ([`event_319`], Thumb).
pub const EVENT_319: u32 = 0x021B_55C9;
/// GMEVENT function of command 325 ([`wait_325`], Thumb).
pub const EVENT_325: u32 = 0x021B_55F1;
/// `__FILE__` string "scrcmd_encount.c" passed to `Heap_AllocDebug`.
pub const FILE: u32 = 0x021D_E234;
/// Heap id and ARC id of command 561.
pub const HEAP_561: u32 = 0x8015;
pub const ARC_561: u32 = 0x7E;
/// ARC (163) of the records command 326 loads.
pub const ARC_326: u32 = 0xA3;
/// The 6 u16 ids of the table 0x021D98D0 (overlay 21 ROM) that command 561 queries.
pub const TABLE_561_ADDR: u32 = 0x021D_98D0;
pub const TABLE_561: [u16; 6] = [0xFF, 0x13D, 0x141, 0x144, 0x172, 0x1A7];
/// Proc id and the 4 proc tables (overlay 0x021F3xxx) of command 420 (values 1..4).
pub const PROC_420: u32 = 0x38;
pub const TABLES_420: [u32; 4] = [0x021F_4344, 0x021F_4374, 0x021F_438C, 0x021F_435C];

/// Callees of the scrcmd_encount functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the ctx getters and the GMEVENT calls come from [`CoreHooks`].
pub trait EncountHooks: CoreHooks {
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020124E4(event)` (main): the event's GAMESYS.
    fn fun_020124e4(&mut self, event: u32) -> u32;
    /// C: `FUN_02012984(gamedata)` (main): the zone id.
    fn fun_02012984(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_020490F4(arc, heap)` (main): open an ARC handle.
    fn fun_020490f4(&mut self, arc: u32, heap: u32) -> u32;
    /// C: `FUN_02049238(handle)` (main): close an ARC handle.
    fn fun_02049238(&mut self, handle: u32) -> u32;
    /// C: `FUN_02048F4C(&t, arc, member)` (main): load ARC `arc` member `member` into the 0x2C-byte record `t`.
    fn fun_02048f4c(&mut self, t: &mut [u8; 0x2C], arc: u32, member: u32);
    /// C: `Heap_AllocDebug(heap, size, from_end, file, line)` (`0x02030734`).
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_02005728(n)` (main RNG): a random number below `n`.
    fn fun_02005728(&mut self, n: u32) -> u32;
    /// C: `FUN_0215E2B8(&rec, arc, id, zone)` (overlay 10): nonzero when the encounter record `id` of the zone was loaded
    /// into the 0xE8-byte record `rec` (rec[5] / rec[6]: the id lists at +0xC0 / +0xD4 are valid).
    fn fun_0215e2b8(&mut self, rec: &mut [u8; 0xE8], arc: u32, id: u32, zone: u32) -> u32;
    /// C: `FUN_02159144(x)` (overlay 10).
    fn fun_02159144(&mut self, x: u32);
    /// C: `FUN_02159168(x)` (overlay 10).
    fn fun_02159168(&mut self, x: u32);
    /// C: `FUN_02188C58(fm)` (overlay 21): the field effect / camera object E.
    fn fun_02188c58(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188C5C(fm)` (overlay 21).
    fn fun_02188c5c(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188CFC(fm)` (overlay 21): a field-map mode (1 -> `FUN_021BB260`).
    fn fun_02188cfc(&mut self, fm: u32) -> u32;
    /// C: `FUN_02188D24(fm)` (overlay 21).
    fn fun_02188d24(&mut self, fm: u32) -> u32;
    /// C: `FUN_021BB260(x, on)` (overlay 21).
    fn fun_021bb260(&mut self, x: u32, on: u32);
    /// C: `FUN_021BB990(proc, x, table)` (overlay 21): load the proc `proc` with `table`.
    fn fun_021bb990(&mut self, proc_id: u32, x: u32, table: u32) -> u32;
    /// C: `FUN_0218E17C(E, x)` (overlay 21).
    fn fun_0218e17c(&mut self, e: u32, x: u32);
    /// C: `FUN_0218EC54(E)` (overlay 21).
    fn fun_0218ec54(&mut self, e: u32);
    /// C: `FUN_0218EC60(E)` (overlay 21).
    fn fun_0218ec60(&mut self, e: u32);
    /// C: `FUN_0218F05C(E)` (overlay 21).
    fn fun_0218f05c(&mut self, e: u32);
    /// C: `FUN_0218F0A8(E)` (overlay 21).
    fn fun_0218f0a8(&mut self, e: u32);
    /// C: `FUN_0218F0E8(E)` (overlay 21): nonzero while E is still busy.
    fn fun_0218f0e8(&mut self, e: u32) -> u32;
    /// C: `FUN_0218F0F8(E)` (overlay 21): nonzero when E is on.
    fn fun_0218f0f8(&mut self, e: u32) -> u32;
    /// C: `FUN_0218F818(E)` (overlay 21).
    fn fun_0218f818(&mut self, e: u32);
    /// C: `FUN_0218F878(E)` (overlay 21).
    fn fun_0218f878(&mut self, e: u32);
    /// C: `FUN_0218F900(E, &rec, n)` (overlay 21): start a move with the 0x48-byte record `rec`.
    fn fun_0218f900(&mut self, e: u32, rec: &[u8; 0x48], n: u32);
    /// C: `FUN_0218F9C4(E, &rec, n)` (overlay 21): the same with a 0x18-byte record.
    fn fun_0218f9c4(&mut self, e: u32, rec: &[u8; 0x18], n: u32);
    /// C: `FUN_0218FA40(E, n)` (overlay 21).
    fn fun_0218fa40(&mut self, e: u32, n: u32);
    /// C: `FUN_0218FE50(E)` (overlay 21): nonzero while E is still busy.
    fn fun_0218fe50(&mut self, e: u32) -> u32;
    /// C: `FUN_021D2D50(gamesys, &rec)` (overlay 21, scrcmd_misc21): the camera-shake event of the 0x18-byte record `rec`.
    fn fun_021d2d50(&mut self, gamesys: u32, rec: &[u8; 0x18]) -> u32;
    /// C: `FUN_021F36C0(gamesys, fm, n)` (overlay at 0x021F3xxx, outside the field set): an event.
    fn fun_021f36c0(&mut self, gamesys: u32, fm: u32, n: u32) -> u32;
    /// C: `FUN_021F36C8(n)` (overlay at 0x021F3xxx).
    fn fun_021f36c8(&mut self, n: u32) -> u32;
}

/// C: `FUN_overlay_d_21__021b5190`: `list[0..n)` (u16) already holds `v` -> `n`; else append `v` -> `n + 1` (u16). The helper
/// of command 561 (unique ids).
pub fn add_unique<H: crate::hooks::MemEnv>(h: &mut H, list: u32, n: u32, v: u32) -> u16 {
    let mut i: i32 = 0;
    while i < n as i32 {
        if v == u32::from(h.r16(list + 2 * i as u32)) {
            return n as u16;
        }
        i += 1;
    }
    h.w16(list.wrapping_add(n.wrapping_mul(2)), v as u16);
    n.wrapping_add(1) as u16
}

/// C: `sub_021b51bc`: command 561: var; for each id of [`TABLE_561`] whose record `FUN_0215E2B8(&rec, ARC 126, id, zone)`
/// loads, the 5 ids at rec+0xC0 (if rec[5]) and the 5 at rec+0xD4 (if rec[6]) (each `& 0x7FF`) go into a 0x78-byte list
/// without duplicates ([`add_unique`]); `*var` = the list entry `FUN_02005728(n)` (random below the count); 0.
pub fn cmd_561<H: EncountHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let zone = h.fun_02012984(gd);
    let arc = h.fun_020490f4(ARC_561, HEAP_561);
    let list = h.fun_02030734(HEAP_561, 0x78, 1, FILE, 0xF8);
    let mut n: u16 = 0;
    // the stack record (filled by FUN_0215E2B8 only when it answers nonzero)
    let mut rec = [0u8; 0xE8];
    for &id in &TABLE_561 {
        if h.fun_0215e2b8(&mut rec, arc, u32::from(id), zone) == 0 {
            continue;
        }
        if rec[5] != 0 {
            for i in 0..5 {
                let v = u16::from_le_bytes([rec[0xC0 + 4 * i], rec[0xC1 + 4 * i]]) & 0x7FF;
                n = add_unique(h, list, u32::from(n), u32::from(v));
            }
        }
        if rec[6] != 0 {
            for i in 0..5 {
                let v = u16::from_le_bytes([rec[0xD4 + 4 * i], rec[0xD5 + 4 * i]]) & 0x7FF;
                n = add_unique(h, list, u32::from(n), u32::from(v));
            }
        }
    }
    let k = h.fun_02005728(u32::from(n));
    let v = h.r16(list.wrapping_add(k.wrapping_mul(2)));
    h.w16(var, v);
    h.fun_020307b0(list);
    h.fun_02049238(arc);
    0
}

/// C: `FUN_overlay_d_21__021b52a4`: turn the field effect object off: `FUN_0218F878(E)`; `FUN_0218F0F8(E)` ->
/// `FUN_0218F05C(E)`; field-map mode 1 -> `FUN_021BB260(FUN_02188C5C(fm), 1)`; `FUN_02159168(0)`.
pub fn effect_off<H: EncountHooks>(h: &mut H, fm: u32, e: u32) {
    h.fun_0218f878(e);
    if h.fun_0218f0f8(e) != 0 {
        h.fun_0218f05c(e);
    }
    if h.fun_02188cfc(fm) == 1 {
        let x = h.fun_02188c5c(fm);
        h.fun_021bb260(x, 1);
    }
    h.fun_02159168(0);
}

/// C: `sub_021b52e0`: the init proc of `script_sys::INIT_PROCS` slot 0 (`(task, task + 0xC)`, task +0 = GAMESYS):
/// [`effect_off`]`(fm, E)`; 1 (done).
pub fn init_proc<H: EncountHooks>(h: &mut H, task: u32, _x: u32) -> u32 {
    let gs = h.r32(task);
    let fm = h.fun_0201210c(gs);
    let e = h.fun_02188c58(fm);
    effect_off(h, fm, e);
    1
}

/// C: `sub_021b52fc`: command 319: `FUN_02159144(0)`; E on -> `FUN_0218F0A8(E)`, the GMEVENT [`EVENT_319`] runs as a child
/// event, field-map mode 1 -> `FUN_021BB260(FUN_02188C5C(fm), 0)`; E off -> `FUN_0218F818(E)`; 1.
pub fn cmd_319<H: EncountHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let e = h.fun_02188c58(fm);
    h.fun_02159144(0);
    if h.fun_0218f0f8(e) != 0 {
        h.fun_0218f0a8(e);
        let sw = h.fun_0215a5f4(ctx);
        let ev = h.fun_020122c0(gs, 0, EVENT_319, 0);
        h.fun_0215897c(sw, ev);
        if h.fun_02188cfc(fm) == 1 {
            let x = h.fun_02188c5c(fm);
            h.fun_021bb260(x, 0);
        }
    } else {
        h.fun_0218f818(e);
    }
    1
}

/// C: `sub_021b536c`: command 320: [`effect_off`]`(fm, E)`; 0.
pub fn cmd_320<H: EncountHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let e = h.fun_02188c58(fm);
    effect_off(h, fm, e);
    0
}

/// The field effect object E of the script's GAMESYS (`FUN_02188C58(FUN_0201210C(GAMESYS))`).
fn effect<H: EncountHooks>(h: &mut H, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    h.fun_02188c58(fm)
}

/// C: `FUN_overlay_d_21__021b538c`: command 321: `FUN_0218EC54(E)`; 0.
pub fn cmd_321<H: EncountHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let e = effect(h, ctx);
    h.fun_0218ec54(e);
    0
}

/// C: `sub_021b53a4`: command 322: `FUN_0218EC60(E)`; 0.
pub fn cmd_322<H: EncountHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let e = effect(h, ctx);
    h.fun_0218ec60(e);
    0
}

/// C: `sub_021b53bc`: command 323: the 0x48-byte record {+0x24 u16 op, +0x26 u16 op, +0x28 u32 op, +0xC / +0x10 / +0x14 u32
/// ops, +0x30 0, +0x34 / +0x38 / +0x3C 1, +0x40 0, +0x44 1} goes to `FUN_0218F900(E, &rec, u16 op)`; 0. The other words of the
/// stack record are never set (0 here, the Z19 test state; stack garbage in the original).
pub fn cmd_323<H: EncountHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let e = effect(h, ctx);
    let mut rec = [0u8; 0x48];
    let a = h.fun_02011330(vm) as u16;
    rec[0x24..0x26].copy_from_slice(&a.to_le_bytes());
    let b = h.fun_02011330(vm) as u16;
    rec[0x26..0x28].copy_from_slice(&b.to_le_bytes());
    for o in [0x28, 0xC, 0x10, 0x14] {
        let v = h.fun_0201134c(vm);
        rec[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }
    for (o, v) in [
        (0x34, 1u32),
        (0x38, 1),
        (0x3C, 1),
        (0x44, 1),
        (0x30, 0),
        (0x40, 0),
    ] {
        rec[o..o + 4].copy_from_slice(&v.to_le_bytes());
    }
    let n = h.fun_02011330(vm);
    h.fun_0218f900(e, &rec, n);
    0
}

/// C: `sub_021b542c`: command 326: a, b = u16 ops; t = ARC 163 member a (`FUN_02048F4C`, 0x2C bytes); the 0x48-byte record
/// {+0x24 t.u16[0], +0x26 t.u16[1], +0x28 t[1], +0xC..+0x18 t[2..5], +0x30..+0x48 t[5..11]} goes to `FUN_0218F900(E, &rec,
/// b)`; 0. The other words of the stack record are never set (0 here, the Z19 test state).
pub fn cmd_326<H: EncountHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let e = effect(h, ctx);
    let a = h.fun_02011330(vm);
    let b = h.fun_02011330(vm);
    let mut t = [0u8; 0x2C];
    h.fun_02048f4c(&mut t, ARC_326, a);
    let mut rec = [0u8; 0x48];
    rec[0x24..0x28].copy_from_slice(&t[0..4]);
    rec[0x28..0x2C].copy_from_slice(&t[4..8]);
    rec[0xC..0x18].copy_from_slice(&t[8..0x14]);
    rec[0x30..0x48].copy_from_slice(&t[0x14..0x2C]);
    h.fun_0218f900(e, &rec, b);
    0
}

/// C: `sub_021b5498`: command 324: `FUN_0218F9C4(E, &{6 words of 1}, u16 op)`; 0.
pub fn cmd_324<H: EncountHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let e = effect(h, ctx);
    let n = h.fun_02011330(vm);
    let mut rec = [0u8; 0x18];
    for c in rec.chunks_mut(4) {
        c.copy_from_slice(&1u32.to_le_bytes());
    }
    h.fun_0218f9c4(e, &rec, n);
    0
}

/// C: `sub_021b54d4`: command 327: `FUN_0218FA40(E, u16 op)`; 0.
pub fn cmd_327<H: EncountHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let e = effect(h, ctx);
    let n = h.fun_02011330(vm);
    h.fun_0218fa40(e, n);
    0
}

/// C: `sub_021b54fc`: command 325: the GMEVENT [`EVENT_325`] runs as a child event; 1.
pub fn cmd_325<H: EncountHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let ev = h.fun_020122c0(gs, 0, EVENT_325, 0);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b5528`: command 328: values v0..v7; the shake event `FUN_021D2D50(GAMESYS, &rec)` (rec, 0x18 bytes = {u16 v0,
/// u16 v1, u16 v4, u16 v5, u16 v6, u16 0, u16 v7, u8 v3, u8 v3, u32 v2, u32 0}) runs as a child event; 1.
pub fn cmd_328<H: EncountHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let mut v = [0u32; 8];
    for x in v.iter_mut() {
        *x = h.fun_02159b10(vm, ctx);
    }
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let mut rec = [0u8; 0x18];
    for (o, x) in [
        (0, v[0]),
        (2, v[1]),
        (4, v[4]),
        (6, v[5]),
        (8, v[6]),
        (0xA, 0),
        (0xC, v[7]),
    ] {
        rec[o..o + 2].copy_from_slice(&(x as u16).to_le_bytes());
    }
    rec[0xE] = v[3] as u8;
    rec[0xF] = v[3] as u8;
    rec[0x10..0x14].copy_from_slice(&v[2].to_le_bytes());
    let ev = h.fun_021d2d50(gs, &rec);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b55c8`: the GMEVENT function of command 319: E no longer busy (`FUN_0218F0E8(E) == 0`) -> `FUN_0218F818(E)`,
/// done (1); else 0.
pub fn event_319<H: EncountHooks>(h: &mut H, event: u32) -> u32 {
    let gs = h.fun_020124e4(event);
    let fm = h.fun_0201210c(gs);
    let e = h.fun_02188c58(fm);
    if h.fun_0218f0e8(e) != 0 {
        return 0;
    }
    h.fun_0218f818(e);
    1
}

/// C: `sub_021b55f0`: the GMEVENT function of command 325: done (1) once `FUN_0218FE50(E)` answers 0, else 0 (wait).
pub fn wait_325<H: EncountHooks>(h: &mut H, event: u32) -> u32 {
    let gamesys = h.fun_020124e4(event);
    let fm = h.fun_0201210c(gamesys);
    let e = h.fun_02188c58(fm);
    if h.fun_0218fe50(e) != 0 {
        0
    } else {
        1
    }
}

/// C: `sub_021b5610`: command 418: v; the event `FUN_021F36C0(GAMESYS, fm, (u16)(v - 1))` runs as a child event; 1.
pub fn cmd_418<H: EncountHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let v = h.fun_02159b10(vm, ctx);
    let ev = h.fun_021f36c0(gs, fm, v.wrapping_sub(1) & 0xFFFF);
    h.fun_0215897c(sw, ev);
    1
}

/// C: `sub_021b5650`: command 419: v; `FUN_0218E17C(E, FUN_021F36C8((u16)(v - 1)))`; 0.
pub fn cmd_419<H: EncountHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let v = h.fun_02159b10(vm, ctx);
    let e = h.fun_02188c58(fm);
    let x = h.fun_021f36c8(v.wrapping_sub(1) & 0xFFFF);
    h.fun_0218e17c(e, x);
    0
}

/// C: `sub_021b568c`: command 420: v = value 1..4 -> `FUN_021BB990(0x38, FUN_02188D24(fm), TABLES_420[v - 1])`; other values
/// do nothing; 0.
pub fn cmd_420<H: EncountHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gs = h.fun_0215a5dc(ctx);
    let fm = h.fun_0201210c(gs);
    let x = h.fun_02188d24(fm);
    let v = h.fun_02159b10(vm, ctx);
    if (1..=4).contains(&v) {
        h.fun_021bb990(PROC_420, x, TABLES_420[v as usize - 1]);
    }
    0
}

/// Run the command handler at `handler` (bit 0 ignored) when it is one of [`COMMANDS`]; `None` otherwise.
pub fn dispatch<H: EncountHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x021B_51BC => cmd_561(h, vm, ctx),
        0x021B_52FC => cmd_319(h, vm, ctx),
        0x021B_536C => cmd_320(h, vm, ctx),
        0x021B_538C => cmd_321(h, vm, ctx),
        0x021B_53A4 => cmd_322(h, vm, ctx),
        0x021B_53BC => cmd_323(h, vm, ctx),
        0x021B_5498 => cmd_324(h, vm, ctx),
        0x021B_54FC => cmd_325(h, vm, ctx),
        0x021B_542C => cmd_326(h, vm, ctx),
        0x021B_54D4 => cmd_327(h, vm, ctx),
        0x021B_5528 => cmd_328(h, vm, ctx),
        0x021B_5610 => cmd_418(h, vm, ctx),
        0x021B_5650 => cmd_419(h, vm, ctx),
        0x021B_568C => cmd_420(h, vm, ctx),
        _ => return None,
    })
}

/// Run the GMEVENT function `f(event, seq, work)` when it is [`EVENT_319`] or [`EVENT_325`]; `None` otherwise.
pub fn call_event<H: EncountHooks>(
    h: &mut H,
    f: u32,
    event: u32,
    _seq: u32,
    _work: u32,
) -> Option<u32> {
    match f {
        EVENT_319 => Some(event_319(h, event)),
        EVENT_325 => Some(wait_325(h, event)),
        _ => None,
    }
}

/// Run the init proc `f(task, x)` when it is [`INIT_PROC`] (`script_sys::INIT_PROCS` slot 0); `None` otherwise.
pub fn call_init_proc<H: EncountHooks>(h: &mut H, f: u32, task: u32, x: u32) -> Option<u32> {
    (f == INIT_PROC).then(|| init_proc(h, task, x))
}

#[cfg(test)]
mod tests;
