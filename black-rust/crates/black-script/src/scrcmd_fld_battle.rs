//! F8 (fourth batch): overlay-21 `scrcmd_fld_battle.c`, Thumb code 0x021B5700-0x021B5A18 (F1-SPLIT row 21.49): regulation
//! counts and the party selection before a facility battle (commands 441 / 442, 6 functions).
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_fld_battle.spec`. The functions listed
//! in [`PORTED`] are ported here and checked with `tools/check-fn.sh black-script scrcmd_fld_battle`; the rest are the F8 port
//! job's.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x021B_5824, 0x021B_59B0];

/// `__FILE__` string "scrcmd_fld_battle.c" passed to `Heap_AllocDebug`.
pub const FILE: u32 = 0x021D_E248;
/// Ids 0..`COLLECT_IDS` are tested against the record's bit field (T + 0xA).
pub const COLLECT_IDS: u32 = 0x28B;
/// At most this many ids are collected (the list is 0x40 bytes).
pub const COLLECT_MAX: u16 = 0x20;
/// The selection result when the party-select app was cancelled.
pub const SELECT_CANCEL: u16 = 3;

/// Callees of the scrcmd_fld_battle functions that this area does not port, named by address (meaning per the spec).
pub trait FldBattleHooks: MemEnv {
    /// C: `FUN_0200CA1C(gamedata)` (main): the Pokedex object (UNVERIFIED).
    fn fun_0200ca1c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_0200CED4(dex, id)` (main): nonzero when `id` passes (UNVERIFIED: seen / caught).
    fn fun_0200ced4(&mut self, dex: u32, id: u32) -> u32;
    /// C: `Heap_AllocDebug(heap, size, from_end, file, line)` (`0x02030734`).
    fn fun_02030734(&mut self, heap: u32, size: u32, from_end: u32, file: u32, line: u32) -> u32;
    /// C: `Heap_Free(block)` (`0x020307B0`).
    fn fun_020307b0(&mut self, block: u32);
    /// C: `FUN_021CC720(fm, party, box, a, b, h)` (party_select_list.c): open the party selection (UNVERIFIED roles).
    fn fun_021cc720(&mut self, fm: u32, party: u32, bx: u32, a: u32, b: u32, heap: u32) -> u32;
    /// C: `FUN_021CC810(sel)`: nonzero once the selection has finished.
    fn fun_021cc810(&mut self, sel: u32) -> u32;
    /// C: `FUN_021CC7B4(sel, &cancel)`: the chosen value; `cancel` (u32) is 1 when cancelled.
    fn fun_021cc7b4(&mut self, sel: u32, cancel: &mut [u8; 4]) -> u32;
}

/// C: `FUN_overlay_d_21__021b5824`: the ids `i < 0x28B` whose bit is set in the record `t` (bit field at t + 0xA) and that
/// `FUN_0200CED4(FUN_0200CA1C(GAMEDATA), i)` accepts, at most 0x20, into a new u16 list `Heap_AllocDebug((h & 0x7FFF) |
/// 0x8000, 0x40, 1, "scrcmd_fld_battle.c", 0xAC)`; `*cnt` (u16) = their count; returns the list.
pub fn collect<H: FldBattleHooks>(h: &mut H, t: u32, gamedata: u32, cnt: u32, heap: u32) -> u32 {
    h.w16(cnt, 0);
    let dex = h.fun_0200ca1c(gamedata);
    let list = h.fun_02030734((heap & 0x7FFF) | 0x8000, 0x40, 1, FILE, 0xAC);
    for i in 0..COLLECT_IDS {
        if h.r8(t + 0xA + (i >> 3)) & (1 << (i & 7)) == 0 {
            continue;
        }
        if h.fun_0200ced4(dex, i) == 0 {
            continue;
        }
        let n = h.r16(cnt);
        if n >= COLLECT_MAX {
            break;
        }
        h.w16(cnt, n + 1);
        h.w16(list + 2 * u32::from(n), i as u16);
    }
    list
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

#[cfg(test)]
mod tests;
