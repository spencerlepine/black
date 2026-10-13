//! F8 (first batch): `scrcmd_work.c`, overlay 10 0x0215A514-0x0215C63A: the SCRCMD WORK / CTX objects and their
//! accessors, the movement-object slots, the choice-menu builder, and 86 commands (system / Pokedex / time / party and
//! Pokemon queries, give Pokemon / egg).
//!
//! Every function of the area (130) is specced in `tools/rom-diff/specs/black-script/scrcmd_work.spec`; its header has
//! the command-id table and the CTX / WORK / MENU layouts. The functions listed in [`PORTED`] are ported here and checked
//! with `tools/check-fn.sh black-script scrcmd_work`; the rest are the F8 port job's. The 8 ctx getters
//! 0x0215A5CC-0x0215A5F4 of this file are ported in [`crate::script_sys`] (`ctx_*`).
//!
//! Pointers are `u32` addresses in the [`Mem`](crate::hooks::Mem) window; the callees are [`WorkHooks`] methods named
//! by address.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_A684,
    0x0215_A690,
    0x0215_A6DC,
    0x0215_AA50,
    0x0215_B464,
];

/// SCRCMD CTX field: -> SCRCMD WORK.
pub const CTX_WORK: u32 = 0x20;
/// SCRCMD WORK field: the 8 movement-object slots (u32 each).
pub const WORK_SLOTS: u32 = 0x14;
/// Number of movement-object slots.
pub const SLOT_COUNT: u32 = 8;
/// SCRCMD WORK field: u16 frame countdown of command 3.
pub const WORK_COUNTDOWN: u32 = 0x106;

/// The u16 table of command 562 (overlay 10 data at 0x02171160, 8 entries; meaning unknown).
pub const TABLE_562: [u16; 8] = [0x0F, 0x37, 0x40, 0x1E, 0x35, 0x2B, 0x9A, 0x0D];
/// ROM address of [`TABLE_562`].
pub const TABLE_562_ADDR: u32 = 0x0217_1160;

/// Callees of the scrcmd_work functions that this area does not port, named by address (meaning per the spec).
pub trait WorkHooks: MemEnv {
    /// C: `FUN_02159AE8` (script_sys `cmd_var_addr`): address of the var whose id is the next script word.
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_02159B10` (script_sys `cmd_var_value`): the next script word read as a var id or a literal.
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32;
}

fn work<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    h.r32(ctx + CTX_WORK)
}

/// C: `FUN_0215A684`: the frame countdown of command 3 (u16 SCRCMD WORK +0x106) = `n`.
pub fn set_countdown<H: MemEnv>(h: &mut H, ctx: u32, n: u32) {
    let w = work(h, ctx);
    h.w16(w + WORK_COUNTDOWN, n as u16);
}

/// C: `FUN_0215A690`: one frame of the countdown: 0 left -> 1 (done); else count down, 0.
pub fn countdown<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let w = work(h, ctx);
    let n = h.r16(w + WORK_COUNTDOWN);
    if n == 0 {
        return 1;
    }
    h.w16(w + WORK_COUNTDOWN, n - 1);
    0
}

/// C: `FUN_0215A6DC`: store `obj` (an overlay-21 movement command object) in the first free of the 8 slots; with none free
/// it is dropped.
pub fn add_slot_object<H: MemEnv>(h: &mut H, ctx: u32, obj: u32) {
    let w = work(h, ctx);
    for i in 0..SLOT_COUNT {
        let a = w + WORK_SLOTS + 4 * i;
        if h.r32(a) == 0 {
            h.w32(a, obj);
            return;
        }
    }
}

/// C: `sub_0215aa50`: command 224: var = 0x15 (UNVERIFIED meaning: a constant, perhaps the game version id).
pub fn cmd_var_set_15<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    h.w16(a, 0x15);
    0
}

/// C: `FUN_overlay_d_10__0215b464`: command 562: var = [`TABLE_562`]`[value]`. An index past the table (the original
/// reads the following ROM bytes) panics.
pub fn cmd_table_562<H: WorkHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let i = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    h.w16(a, TABLE_562[i as usize]);
    0
}

#[cfg(test)]
mod tests;
