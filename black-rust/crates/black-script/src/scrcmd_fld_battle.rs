//! F8e: overlay-21 `scrcmd_fld_battle.c`, Thumb code 0x021B5700-0x021B5A18 (F1-SPLIT row 21.49): regulation counts and the
//! party selection before a facility battle (commands 441 / 442, 6 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_fld_battle.spec`, ported here, listed in
//! [`PORTED`] and checked with `tools/check-fn.sh black-script scrcmd_fld_battle`. [`COMMANDS`] maps the command ids to their
//! handlers, [`dispatch`] runs one by address and [`call_event`] the party-select GMEVENT of command 442.
//!
//! Callees outside the area are [`FldBattleHooks`] methods named by address (vm.c, the operand readers, the ctx getters and
//! the GMEVENT calls come from [`CoreHooks`]). The regulation / party-select roles are read from the code (UNVERIFIED).

use crate::scrcmd_core::CoreHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021B_5700,
    0x021B_574C,
    0x021B_57CC,
    0x021B_5824,
    0x021B_58B0,
    0x021B_59B0,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[(441, 0x021B_5700), (442, 0x021B_574C)];

/// GMEVENT function of command 442's party selection ([`select_event`], Thumb).
pub const EVENT_SELECT: u32 = 0x021B_59B1;
/// `__FILE__` string "scrcmd_fld_battle.c" passed to `Heap_AllocDebug`.
pub const FILE: u32 = 0x021D_E248;
/// Ids 0..`COLLECT_IDS` are tested against the record's bit field (T + 0xA).
pub const COLLECT_IDS: u32 = 0x28B;
/// At most this many ids are collected (the list is 0x40 bytes).
pub const COLLECT_MAX: u16 = 0x20;
/// The selection result when the party-select app was cancelled (also: both checks passed).
pub const SELECT_CANCEL: u16 = 3;
/// Size of the party-select event work.
pub const SELECT_WORK_SIZE: u32 = 0x1C;

/// Callees of the scrcmd_fld_battle functions outside the area, named by address (meaning per the spec; UNVERIFIED unless
/// stated). vm.c, the operand readers, the ctx getters and the GMEVENT calls come from [`CoreHooks`].
pub trait FldBattleHooks: CoreHooks {
    /// C: `FUN_0215A5CC(ctx)` (script_sys): the heap id of the script.
    fn fun_0215a5cc(&mut self, ctx: u32) -> u32;
    /// C: `FUN_02158F74(sw)` (script_sys): the script's word set (UNVERIFIED: the list command 441 fills).
    fn fun_02158f74(&mut self, sw: u32) -> u32;
    /// C: `FUN_0201EE60(list, i, v)` (main): store entry `i` of the word-set list.
    fn fun_0201ee60(&mut self, list: u32, i: u32, v: u32);
    /// C: `FUN_0201A354(v, heap)` (main): a new regulation record for `v` (freed by the caller).
    fn fun_0201a354(&mut self, v: u32, heap: u32) -> u32;
    /// C: `FUN_0200CA1C(gamedata)` (main): the Pokedex object (UNVERIFIED).
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200CED4(dex, id)` (main): nonzero when `id` passes (UNVERIFIED: seen / caught).
    fn fun_0200ced4(&mut self, dex: u32, id: u32) -> u32;
    /// C: `Heap_AllocDebug(heap, size, from_end, file, line)` (`0x02030734`).
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_0201210C(gamesys)` (main): the field map.
    fn fun_0201210c(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_02012934(gamedata)` (main): the party.
    fn fun_02012934(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012EBC(gamedata)` (main): the save data.
    fn fun_02012ebc(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200BAC8(save)` (main): the battle-box save block (UNVERIFIED).
    fn fun_0200bac8(&mut self, save: u32) -> u32;
    /// C: `FUN_0200BBA8(block)` (main): 1 when the battle box is in use (UNVERIFIED).
    fn fun_0200bba8(&mut self, block: u32) -> u32;
    /// C: `FUN_0200BAF4(block, heap)` (main): a party built from the battle box (or null; freed by the caller).
    fn fun_0200baf4(&mut self, block: u32, heap: u32) -> u32;
    /// C: `FUN_02019ED4(reg, party)` (main): the regulation check of the level-limited rules (v 0x14..0x16).
    fn fun_02019ed4(&mut self, reg: u32, party: u32) -> u32;
    /// C: `FUN_0201A090(reg, party, &x)` (main): the regulation check (`x` is the caller's stack word, a passed-through out).
    fn fun_0201a090(&mut self, reg: u32, party: u32, x: u32) -> u32;
    /// C: `FUN_021CC720(fm, party, box, a, b, h)` (party_select_list.c): open the party selection (UNVERIFIED roles).
    fn fun_021cc720(&mut self, fm: u32, party: u32, bx: u32, a: u32, b: u32, heap: u32) -> u32;
    /// C: `FUN_021CC810(sel)`: nonzero once the selection has finished.
    fn fun_021cc810(&mut self, sel: u32) -> u32;
    /// C: `FUN_021CC7B4(sel, &cancel)`: the chosen value; `cancel` (u32) is 1 when cancelled.
    fn fun_021cc7b4(&mut self, sel: u32, cancel: &mut [u8; 4]) -> u32;
}

/// C: `sub_021b5700`: command 441: L = `FUN_02158F74(SW)`; v; var; `*var = `[`count_mons`]`(GAMEDATA, L, v, heap)`; 0.
pub fn cmd_441<H: FldBattleHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gd = h.fun_0215a5e4(ctx);
    let list = h.fun_02158f74(sw);
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let heap = h.fun_0215a5cc(ctx);
    let n = count_mons(h, gd, list, v, heap);
    h.w16(var, n);
    0
}

/// C: `sub_021b574c`: command 442: v, var; P = `FUN_0201A354(v, heap)`; ev = [`regulation_event`]`(GAMESYS, (u16)(v - 0x14)
/// <= 2, P, var, heap)`; P is freed; ev == 0 -> 0, else ev runs as a child event, 1.
pub fn cmd_442<H: FldBattleHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gs = h.fun_0215a5dc(ctx);
    let v = h.fun_02159b10(vm, ctx);
    let var = h.fun_02159ae8(vm, ctx);
    let heap = h.fun_0215a5cc(ctx);
    let p = h.fun_0201a354(v, heap);
    let flag = (v.wrapping_add(0xFFEC) & 0xFFFF) <= 2;
    let ev = regulation_event(h, gs, flag as u32, p, var, heap);
    h.fun_020307b0(p);
    if ev == 0 {
        return 0;
    }
    h.fun_0215897c(sw, ev);
    1
}

/// C: `FUN_overlay_d_21__021b57cc`: T = `FUN_0201A354(v, heap)`; M = [`collect`]`(T, GAMEDATA, &cnt, heap)`; entry i of
/// `list` = M[i] for i < cnt (`FUN_0201EE60`); M and T are freed; returns cnt.
pub fn count_mons<H: FldBattleHooks>(h: &mut H, gd: u32, list: u32, v: u32, heap: u32) -> u16 {
    let t = h.fun_0201a354(v, heap);
    let (m, cnt) = collect_list(h, t, gd, heap);
    let mut i: u16 = 0;
    while i < cnt {
        let x = h.r16(m + 2 * u32::from(i));
        h.fun_0201ee60(list, u32::from(i), u32::from(x));
        i += 1;
    }
    h.fun_020307b0(m);
    h.fun_020307b0(t);
    cnt
}

/// C: `FUN_overlay_d_21__021b5824`: the ids `i < 0x28B` whose bit is set in the record `t` (bit field at t + 0xA) and that
/// `FUN_0200CED4(FUN_0200CA1C(GAMEDATA), i)` accepts, at most 0x20, into a new u16 list `Heap_AllocDebug((h & 0x7FFF) |
/// 0x8000, 0x40, 1, "scrcmd_fld_battle.c", 0xAC)`; `*cnt` (u16) = their count; returns the list.
pub fn collect<H: FldBattleHooks>(h: &mut H, t: u32, gamedata: u32, cnt: u32, heap: u32) -> u32 {
    h.w16(cnt, 0);
    let (list, n) = collect_list(h, t, gamedata, heap);
    h.w16(cnt, n);
    list
}

/// [`collect`] with the count returned instead of stored (its caller [`count_mons`] keeps it in a stack word): (list, count).
fn collect_list<H: FldBattleHooks>(h: &mut H, t: u32, gamedata: u32, heap: u32) -> (u32, u16) {
    let dex = h.fun_0200ca1c(gamedata);
    let list = h.fun_02030734((heap & 0x7FFF) | 0x8000, 0x40, 1, FILE, 0xAC);
    let mut n: u16 = 0;
    for i in 0..COLLECT_IDS {
        if h.r8(t + 0xA + (i >> 3)) & (1 << (i & 7)) == 0 {
            continue;
        }
        if h.fun_0200ced4(dex, i) == 0 {
            continue;
        }
        if n >= COLLECT_MAX {
            break;
        }
        h.w16(list + 2 * u32::from(n), i as u16);
        n += 1;
    }
    (list, n)
}

/// C: `FUN_overlay_d_21__021b58b0`: the regulation check before a facility battle: ok1 = the party passes (`FUN_02019ED4`
/// when `flag`, else `FUN_0201A090`); without a battle box in use (`FUN_0200BBA8(S) != 1`) `*var` = ok1 ? 3 : 0 and no event
/// (0); else B = `FUN_0200BAF4(S, heap)` and ok2 = B passes; both pass -> B is freed (if nonzero), `*var` = 3, 0; else the
/// GMEVENT [`select_event`] with the work {GAMESYS, field map, -, party, B, u8 ok1 != 0, u8 ok2 != 0, u16 heap, var}.
pub fn regulation_event<H: FldBattleHooks>(
    h: &mut H,
    gs: u32,
    flag: u32,
    reg: u32,
    var: u32,
    heap: u32,
) -> u32 {
    // the stack word handed to FUN_0201A090 (UNVERIFIED role); the original passes &x, the stub ignores it
    let x = 0;
    let gd = h.fun_020120f4(gs);
    let party = h.fun_02012934(gd);
    let ok1 = if flag != 0 {
        h.fun_02019ed4(reg, party)
    } else {
        h.fun_0201a090(reg, party, x)
    };
    let save = h.fun_02012ebc(gd);
    let s = h.fun_0200bac8(save);
    if h.fun_0200bba8(s) != 1 {
        h.w16(var, if ok1 != 0 { SELECT_CANCEL } else { 0 });
        return 0;
    }
    let b = h.fun_0200baf4(s, heap & 0xFFFF);
    let ok2 = if flag != 0 {
        h.fun_02019ed4(reg, b)
    } else {
        h.fun_0201a090(reg, b, x)
    };
    if ok1 != 0 && ok2 != 0 {
        if b != 0 {
            h.fun_020307b0(b);
        }
        h.w16(var, SELECT_CANCEL);
        return 0;
    }
    let ev = h.fun_020122c0(gs, 0, EVENT_SELECT, SELECT_WORK_SIZE);
    let w = h.fun_020124e8(ev);
    h.w32(w, gs);
    let fm = h.fun_0201210c(gs);
    h.w32(w + 4, fm);
    h.w32(w + 0x18, var);
    h.w16(w + 0x16, heap as u16);
    h.w32(w + 0xC, party);
    h.w32(w + 0x10, b);
    h.w8(w + 0x14, (ok1 != 0) as u8);
    h.w8(w + 0x15, (ok2 != 0) as u8);
    ev
}

/// C: `sub_021b59b0`: the GMEVENT function of the party selection (`w` = {GAMESYS, fm, sel, party, box, u8 a, u8 b, u16 heap,
/// u16 *var}): seq 0 opens it (`FUN_021CC720`), seq 1 waits for `FUN_021CC810`, stores the choice (3 when cancelled) in `*var`,
/// frees the box copy and ends (1).
pub fn select_event<H: FldBattleHooks>(h: &mut H, _event: u32, seq: u32, w: u32) -> u32 {
    match h.r32(seq) {
        0 => {
            let (fm, party, bx) = (h.r32(w + 4), h.r32(w + 0xC), h.r32(w + 0x10));
            let (a, b, heap) = (
                u32::from(h.r8(w + 0x14)),
                u32::from(h.r8(w + 0x15)),
                u32::from(h.r16(w + 0x16)),
            );
            let sel = h.fun_021cc720(fm, party, bx, a, b, heap);
            h.w32(w + 8, sel);
            let s = h.r32(seq);
            h.w32(seq, s.wrapping_add(1));
            0
        }
        1 => {
            let sel = h.r32(w + 8);
            if h.fun_021cc810(sel) == 0 {
                return 0;
            }
            let sel = h.r32(w + 8);
            let mut cancel = [0u8; 4];
            let v = h.fun_021cc7b4(sel, &mut cancel);
            let var = h.r32(w + 0x18);
            h.w16(var, v as u16);
            if u32::from_le_bytes(cancel) == 1 {
                let var = h.r32(w + 0x18);
                h.w16(var, SELECT_CANCEL);
            }
            let bx = h.r32(w + 0x10);
            h.fun_020307b0(bx);
            1
        }
        _ => 0,
    }
}

/// Run the command handler at `handler` (bit 0 ignored) when it is one of [`COMMANDS`]; `None` otherwise.
pub fn dispatch<H: FldBattleHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x021B_5700 => cmd_441(h, vm, ctx),
        0x021B_574C => cmd_442(h, vm, ctx),
        _ => return None,
    })
}

/// Run the GMEVENT function `f(event, seq, work)` when it is [`EVENT_SELECT`]; `None` otherwise.
pub fn call_event<H: FldBattleHooks>(
    h: &mut H,
    f: u32,
    event: u32,
    seq: u32,
    work: u32,
) -> Option<u32> {
    (f == EVENT_SELECT).then(|| select_event(h, event, seq, work))
}

#[cfg(test)]
mod tests;
