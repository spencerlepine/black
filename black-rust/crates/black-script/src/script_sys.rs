//! F7: script system glue (`script_sys.c` + `script_work.c`, overlay 10 0x02158804-0x021591FE and 0x02159960-0x02159B48)
//! and the scrcmd ctx getters 0x0215A5CC-0x0215A5FA.
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/script_sys.spec` (its header documents
//! the SYS / SCRIPT_WORK / VM / scrcmd CTX layouts and the command dispatch); this file holds the ported ones, checked with
//! `tools/check-fn.sh black-script script_sys`. The interpreter loop itself is `vm.c` in main (0x020111E8-0x02011428).
//!
//! SCRIPT_WORK (`SW`, 0x14C bytes) var block: script vars `0x8000..` are halfwords at `SW + 0x7C + (id - 0x8000) * 2`;
//! ids `0x4000..0x7FFF` are event-work vars (`black_field::event_work`), anything else is a literal.

use crate::hooks::ScriptSysHooks;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_8FD0,
    0x0215_8FE0,
    0x0215_901C,
    0x0215_9198,
    0x0215_9AE8,
    0x0215_A5F4,
];

/// Offset of the script var block in SCRIPT_WORK.
pub const SW_VARS_OFS: u32 = 0x7C;
/// First script var id (stored in SCRIPT_WORK).
pub const SW_VAR_BASE: u32 = 0x8000;
/// First event-work var id.
pub const EVENT_VAR_BASE: u32 = 0x4000;
/// End (exclusive) of the script var ids.
pub const SW_VAR_END: u32 = 0xC000;
/// Address of the 14-bit init-proc mask (overlay 10 BSS; `0x02159144` sets, `0x02159168` clears, `0x02159198` tests).
pub const PROC_MASK_ADDR: u32 = 0x0217_2BA0;
/// Number of bits of the init-proc mask (entries of the proc table 0x021705F0).
pub const PROC_MASK_BITS: i32 = 14;

/// C: `FUN_overlay_d_10__02158fd0`: address of script var `var` in the SCRIPT_WORK at `sw` (no range check; wrapping
/// address arithmetic as in the original).
pub fn sw_var_addr(sw: u32, var: u32) -> u32 {
    sw.wrapping_add(SW_VARS_OFS)
        .wrapping_add(var.wrapping_sub(SW_VAR_BASE).wrapping_mul(2))
}

/// C: `FUN_overlay_d_10__02158fe0`: address of var `var`: `< 0x4000` none (0), `< 0x8000` the event-work var
/// (`FUN_020143CC(FUN_0201296C(gamedata), var)`), `< 0xC000` the script var in `sw`, else none. The event work is looked
/// up first in every case (the original calls `FUN_0201296C` unconditionally).
pub fn var_addr<H: ScriptSysHooks>(h: &mut H, sw: u32, gamedata: u32, var: u32) -> u32 {
    let ew = h.fun_0201296c(gamedata);
    if var < EVENT_VAR_BASE {
        0
    } else if var < SW_VAR_BASE {
        h.fun_020143cc(ew, var)
    } else if var < SW_VAR_END {
        sw_var_addr(sw, var)
    } else {
        0
    }
}

/// C: `FUN_overlay_d_10__0215901c`: value of var `var` (the u16 at [`var_addr`]), or `var` itself when it has no
/// address (a literal operand).
pub fn var_value<H: ScriptSysHooks>(h: &mut H, sw: u32, gamedata: u32, var: u32) -> u32 {
    match var_addr(h, sw, gamedata, var) {
        0 => var,
        p => h.r16(p) as u32,
    }
}

/// C: `FUN_overlay_d_10__02159198`: bit `bit` of the init-proc mask (`mask` = the u32 at [`PROC_MASK_ADDR`]); 0 for
/// `bit >= 14` (signed). A negative bit shifts by its low byte (`asrs`), as the original does.
pub fn proc_mask_test(mask: u32, bit: u32) -> u32 {
    if bit as i32 >= PROC_MASK_BITS {
        return 0;
    }
    let sh = bit & 0xFF;
    let v = if sh < 32 {
        ((mask as i32) >> sh) as u32
    } else if (mask as i32) < 0 {
        u32::MAX
    } else {
        0
    };
    v & 1
}

/// C: `FUN_overlay_d_10__0215a5f4`: the SCRIPT_WORK of a scrcmd ctx (word 0 of the scrcmd work at `ctx + 0x20`).
pub fn ctx_script_work<H: ScriptSysHooks>(h: &mut H, ctx: u32) -> u32 {
    let work = h.r32(ctx.wrapping_add(0x20));
    h.r32(work)
}

/// C: `FUN_overlay_d_10__02159ae8`: address of the var whose id is the next u16 of the running script
/// (`var_addr(SW, GAMEDATA, FUN_02011330(vm))`); the command handlers store their results through it.
pub fn cmd_var_addr<H: ScriptSysHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gamedata = h.fun_0215a5e4(ctx);
    let sw = ctx_script_work(h, ctx);
    let var = h.fun_02011330(vm);
    var_addr(h, sw, gamedata, var)
}

#[cfg(test)]
mod tests;
