//! `game_data.c` / `game_event.c` / the game system record (`0x02011F24..0x02013090`): the GAMEDATA
//! record, a `0x6B0`-byte hub that holds pointers to the subsystems (save data, event work, player
//! records, ...), the `0x34`-byte game system record and the `0x14`-byte EVENT task records. The layout
//! is documented in the header of `tools/rom-diff/specs/black-field/game_data.spec`; every function is
//! verified per function with `tools/check-fn.sh black-field game_data` (159 functions).
//!
//! Pointers stay `u32` addresses; records are read and written through [`MemEnv`] and the unported
//! callees are the methods of [`GameDataHooks`], named after their address. Offsets are named only
//! where the code shows the meaning; the rest are `gd_get_<addr>` / `gd_set_<addr>` accessors named
//! after the original routine.
//!
//! Not in the spec (UI/hardware glue that only sequences unported subsystems): the boot, main loop and
//! shutdown functions of the game system, the function-pointer dispatchers of `game_event.c`, and the
//! GAMEDATA constructor and destructor.

use crate::hooks::{GameDataHooks, MemEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_1F24,
    0x0201_1F28,
    0x0201_205C,
    0x0201_209C,
    0x0201_20A4,
    0x0201_20B4,
    0x0201_20C0,
    0x0201_20D8,
    0x0201_20F0,
    0x0201_20F4,
    0x0201_20F8,
    0x0201_2104,
    0x0201_2108,
    0x0201_210C,
    0x0201_2110,
    0x0201_2114,
    0x0201_2124,
    0x0201_212C,
    0x0201_2134,
    0x0201_2138,
    0x0201_213C,
    0x0201_2144,
    0x0201_214C,
    0x0201_2158,
    0x0201_2164,
    0x0201_21CC,
    0x0201_2204,
    0x0201_2210,
    0x0201_2220,
    0x0201_2248,
    0x0201_2258,
    0x0201_2264,
    0x0201_22A4,
    0x0201_22B4,
    0x0201_2314,
    0x0201_2374,
    0x0201_23C4,
    0x0201_24E4,
    0x0201_24E8,
    0x0201_24EC,
    0x0201_24F0,
    0x0201_2798,
    0x0201_27A4,
    0x0201_27AC,
    0x0201_27B8,
    0x0201_27C4,
    0x0201_27D0,
    0x0201_27E0,
    0x0201_27F0,
    0x0201_2800,
    0x0201_281C,
    0x0201_283C,
    0x0201_2844,
    0x0201_284C,
    0x0201_2854,
    0x0201_2870,
    0x0201_2878,
    0x0201_2894,
    0x0201_289C,
    0x0201_28B8,
    0x0201_28C0,
    0x0201_28DC,
    0x0201_28E4,
    0x0201_2900,
    0x0201_2914,
    0x0201_2924,
    0x0201_292C,
    0x0201_2934,
    0x0201_293C,
    0x0201_2944,
    0x0201_2950,
    0x0201_295C,
    0x0201_2964,
    0x0201_296C,
    0x0201_2974,
    0x0201_297C,
    0x0201_2984,
    0x0201_298C,
    0x0201_2994,
    0x0201_29A0,
    0x0201_29AC,
    0x0201_29B8,
    0x0201_29C4,
    0x0201_29D0,
    0x0201_29DC,
    0x0201_29F0,
    0x0201_29FC,
    0x0201_2A24,
    0x0201_2A30,
    0x0201_2A38,
    0x0201_2A44,
    0x0201_2A4C,
    0x0201_2A54,
    0x0201_2A60,
    0x0201_2A90,
    0x0201_2AA0,
    0x0201_2AA4,
    0x0201_2AB8,
    0x0201_2ABC,
    0x0201_2AC0,
    0x0201_2AC4,
    0x0201_2AC8,
    0x0201_2ACC,
    0x0201_2AD0,
    0x0201_2AD4,
    0x0201_2AE4,
    0x0201_2B28,
    0x0201_2B2C,
    0x0201_2B30,
    0x0201_2B34,
    0x0201_2B38,
    0x0201_2B40,
    0x0201_2B4C,
    0x0201_2B58,
    0x0201_2B64,
    0x0201_2B70,
    0x0201_2B78,
    0x0201_2B80,
    0x0201_2B8C,
    0x0201_2B98,
    0x0201_2BA4,
    0x0201_2BB0,
    0x0201_2BC8,
    0x0201_2BE0,
    0x0201_2BF8,
    0x0201_2C0C,
    0x0201_2E40,
    0x0201_2EBC,
    0x0201_2EC0,
    0x0201_2EC4,
    0x0201_2ED0,
    0x0201_2ED8,
    0x0201_2EE4,
    0x0201_2EF0,
    0x0201_2EFC,
    0x0201_2F08,
    0x0201_2F14,
    0x0201_2F20,
    0x0201_2F2C,
    0x0201_2F38,
    0x0201_2F44,
    0x0201_2F50,
    0x0201_2F5C,
    0x0201_2F68,
    0x0201_2F74,
    0x0201_2F80,
    0x0201_2F8C,
    0x0201_2F98,
    0x0201_2FA4,
    0x0201_2FB0,
    0x0201_2FBC,
    0x0201_2FCC,
    0x0201_2FD8,
    0x0201_2FE4,
    0x0201_2FF0,
    0x0201_2FFC,
    0x0201_3020,
    0x0201_3050,
    0x0201_3080,
];

/// Size of the GAMEDATA record (`FUN_020124F4` allocates it).
pub const GAMEDATA_SIZE: usize = 0x6B0;
/// Offset of the pointer to the event work object (flags and vars, see [`crate::event_work`]).
pub const EVENT_WORK_PTR_OFS: usize = 0x1AC;

/// Address of the global that holds a pointer to a record whose first word is a handle.
pub const GLOBAL_HANDLE_PTR: u32 = 0x020A_7298;
/// Callback address `FUN_02012248` passes to `FUN_02035138` (the event dispatcher).
pub const SYS_PROC_A: u32 = 0x0201_2221;
/// Callback address `FUN_020122A4` passes to `FUN_02035064`.
pub const SYS_PROC_B: u32 = 0x0201_2265;
/// Argument of `FUN_020120C0` (a `0x021D359C` table in overlay space) to `FUN_02031348`.
pub const SYS_TABLE_021D359C: u32 = 0x021D_359C;

/// ROM table at `0x0209DFA8` (17 halfwords, the last is the first word of the next table): the first four (`0, 0x8000, 0x4000, 0xC000`) are facing
/// angles, the rest is the neighbouring data (`FUN_02012AD4` has no bounds check).
const TABLE_0209DFA8: [u16; 17] = [
    0x0000, 0x8000, 0x4000, 0xC000, 0x0103, 0xFCFE, 0x0006, 0x0000, 0x00F5, 0x00F6, 0x00F7, 0x00F8,
    0x01A8, 0x0000, 0x0126, 0x0127, 0x003E,
];
/// ROM table at `0x0209DFB0` (signed bytes, 5 entries).
const TABLE_0209DFB0: [i8; 5] = [3, 1, -2, -4, 6];

// ---------------------------------------------------------------------------------------------
// handwritten functions
// ---------------------------------------------------------------------------------------------

/// C: `FUN_02011F24`: size of the game system record.
pub fn sys_size() -> u32 {
    0x34
}

/// C: `FUN_02011F28`: fill the game system record `sys` (`heap` is the heap id, `a2` an opaque argument).
///
/// The original stores the heap id as a halfword and passes the stored halfword to three of the
/// constructors; the unmodified argument goes to `FUN_0202B4BC` and `FUN_020139B0`.
pub fn sys_init<H: GameDataHooks>(h: &mut H, sys: u32, heap: u32, a2: u32) {
    h.w16(sys, heap as u16);
    let id = h.r16(sys) as u32;
    h.w32(sys + 4, a2);
    let v = h.fun_02031324(id);
    h.w32(sys + 8, v);
    let id = h.r16(sys) as u32;
    for ofs in [0xC, 0x10, 0x14, 0x18] {
        h.w32(sys + ofs, 0);
    }
    let gd = h.fun_020124f4(id);
    let id = h.r16(sys) as u32;
    h.w32(sys + 0x1C, gd);
    h.w32(sys + 0x20, 0);
    let v = h.fun_020261e4(id, gd);
    h.w32(sys + 0x24, v);
    let gd = h.r32(sys + 0x1C);
    h.w32(sys + 0x28, 0);
    let v = h.fun_0202b4bc(gd, heap);
    h.w32(sys + 0x2C, v);
    h.w8(sys + 0x32, 0);
    let b = h.r8(sys + 0x33);
    h.w8(sys + 0x33, b & !1);
    sys_register_a(h, sys);
    sys_register_b(h, sys);
    h.fun_020139b0(heap);
}

/// C: `FUN_0201209C`: callback pair `+0x10 = a`, `+0x14 = b`.
pub fn sys_set_callbacks<E: MemEnv>(env: &mut E, sys: u32, a: u32, b: u32) {
    env.w32(sys + 0x10, a);
    env.w32(sys + 0x14, b);
}

/// C: `FUN_020120A4`: `+0x31` = whether an event is queued.
pub fn sys_cache_event_active<E: MemEnv>(env: &mut E, sys: u32) {
    let v = event_queued(env, sys);
    env.w8(sys + 0x31, v as u8);
}

/// C: `FUN_020120B4`: tail call `FUN_02031348(*(sys + 8), a, b, c)`.
pub fn sys_call_02031348<H: GameDataHooks>(h: &mut H, sys: u32, a: u32, b: u32, c: u32) -> u32 {
    let x = h.r32(sys + 8);
    h.fun_02031348(x, a, b, c)
}

/// C: `FUN_020120C0`: `FUN_020120B4(sys, 0x15, 0x021D359C, sys)`.
pub fn sys_call_021d359c<H: GameDataHooks>(h: &mut H, sys: u32) -> u32 {
    sys_call_02031348(h, sys, 0x15, SYS_TABLE_021D359C, sys)
}

/// C: `FUN_020120D8`: 1 if an event is queued, else the cached byte at `+0x31`.
pub fn sys_event_active<E: MemEnv>(env: &mut E, sys: u32) -> u8 {
    if event_queued(env, sys) {
        1
    } else {
        env.r8(sys + 0x31)
    }
}

/// C: `FUN_020123C4`: whether `+0x18` (the current event) is set.
pub fn event_queued<E: MemEnv>(env: &mut E, sys: u32) -> bool {
    sys_get_event(env, sys) != 0
}

/// C: `FUN_020120F8`: tail call `FUN_020127A4(*(sys + 0x1C))`: the 0x20-byte block of the GAMEDATA.
pub fn sys_gamedata_block<E: MemEnv>(env: &mut E, sys: u32) -> u32 {
    let gd = env.r32(sys + 0x1C);
    gd_addr_020127a4(gd)
}

/// C: `FUN_02012114`: whether `+0x20` is set.
pub fn sys_has_20<E: MemEnv>(env: &mut E, sys: u32) -> bool {
    sys_get_20(env, sys) != 0
}

/// C: `FUN_0201214C`: tail call `FUN_02012BB0(*(sys + 0x1C))`.
pub fn sys_flag_198<H: GameDataHooks>(h: &mut H, sys: u32) -> bool {
    let gd = h.r32(sys + 0x1C);
    gd_flag_198(h, gd)
}

/// C: `FUN_02012158`: tail call `FUN_02012BC8(*(sys + 0x1C), flag)`.
pub fn sys_set_flag_198<H: GameDataHooks>(h: &mut H, sys: u32, flag: u32) {
    let gd = h.r32(sys + 0x1C);
    gd_set_flag_198(h, gd, flag);
}

/// C: `FUN_02012164`: 1 when nothing blocks the field (no event, not in a blocked zone, ...).
///
/// The first callee (`FUN_02013D34`, a zone id predicate) is given the halfword at `+0` of the
/// GAMEDATA block, as in the original.
pub fn sys_field_free<H: GameDataHooks>(h: &mut H, sys: u32) -> bool {
    let block = sys_gamedata_block(h, sys);
    let id = pl_get_00(h, block) as u32;
    if sys_get_32(h, sys) != 0 {
        return false;
    }
    if h.fun_02013d34(id) != 0 {
        return false;
    }
    if h.fun_0200fb24() != 0 {
        return false;
    }
    if !sys_flag_198(h, sys) {
        return false;
    }
    if h.fun_02158318(sys) != 1 {
        return false;
    }
    if h.fun_02035390() == 1 && h.fun_020353b0() == 1 {
        return false;
    }
    let gd = h.r32(sys + 0x1C);
    h.fun_0200921c(gd);
    h.fun_020090e0() != 0
}

/// C: `FUN_020121CC`: start the field task `sys[0x24]` when nothing blocks; 1 if started.
pub fn sys_try_start<H: GameDataHooks>(h: &mut H, sys: u32) -> bool {
    let u = sys_get_24(h, sys);
    if h.fun_0203cd98() == 0 && h.fun_02026454(u) == 0 && sys_field_free(h, sys) {
        h.fun_020263d8(u, 1, sys);
        return true;
    }
    false
}

/// C: `FUN_0201205C`: no event queued, `+0x20` set and `FUN_02188BE8(+0x20) == 1`: `FUN_020121CC`.
pub fn sys_poll<H: GameDataHooks>(h: &mut H, sys: u32) {
    if sys_event_active(h, sys) == 0 && sys_has_20(h, sys) {
        let v = sys_get_20(h, sys);
        if h.fun_02188be8(v) == 1 {
            sys_try_start(h, sys);
        }
    }
}

/// C: `FUN_02012204`: bit 0 of `+0x33`.
pub fn sys_get_33_bit0<E: MemEnv>(env: &mut E, sys: u32) -> u8 {
    env.r8(sys + 0x33) & 1
}

/// C: `FUN_02012210`: clear bit 0 of `+0x33`.
pub fn sys_clear_33_bit0<E: MemEnv>(env: &mut E, sys: u32) {
    let b = env.r8(sys + 0x33);
    env.w8(sys + 0x33, b & !1);
}

/// C: `FUN_02012220`: when `FUN_0201214C`: `FUN_02012158(sys, 0)` and set bit 0 of `+0x33`.
pub fn sys_suspend<H: GameDataHooks>(h: &mut H, sys: u32) {
    if sys_flag_198(h, sys) {
        sys_set_flag_198(h, sys, 0);
        let b = h.r8(sys + 0x33);
        h.w8(sys + 0x33, (b & !1) | 1);
    }
}

/// C: `FUN_02012248`: `FUN_02035138(0x02012221, sys)`.
pub fn sys_register_a<H: GameDataHooks>(h: &mut H, sys: u32) {
    h.fun_02035138(SYS_PROC_A, sys);
}

/// C: `FUN_02012258`: `FUN_02035138(0, 0)`.
pub fn sys_unregister_a<H: GameDataHooks>(h: &mut H) {
    h.fun_02035138(0, 0);
}

/// C: `FUN_02012264`: 1 if `FUN_02026454(sys[0x24])` or `FUN_0203DA4C()`; 0 if `FUN_020353B0()`; else
/// `FUN_0201214C(sys)`.
pub fn sys_busy<H: GameDataHooks>(h: &mut H, sys: u32) -> bool {
    let u = sys_get_24(h, sys);
    if h.fun_02026454(u) != 0 {
        return true;
    }
    if h.fun_0203da4c() != 0 {
        return true;
    }
    if h.fun_020353b0() != 0 {
        return false;
    }
    sys_flag_198(h, sys)
}

/// C: `FUN_020122A4`: `FUN_02035064(0x02012265, sys)`.
pub fn sys_register_b<H: GameDataHooks>(h: &mut H, sys: u32) {
    h.fun_02035064(SYS_PROC_B, sys);
}

/// C: `FUN_020122B4`: `FUN_02035064(0, 0)`.
pub fn sys_unregister_b<H: GameDataHooks>(h: &mut H) {
    h.fun_02035064(0, 0);
}

/// C: `FUN_02012314`: free the data block (`+0xC`) when set, then the EVENT record.
pub fn ev_free<H: GameDataHooks>(h: &mut H, ev: u32) {
    let data = h.r32(ev + 0xC);
    if data != 0 {
        h.heap_free(data);
    }
    h.heap_free(ev);
}

/// C: `FUN_02012374`: `*p = ev`, then the owner's current event becomes `p`.
pub fn ev_publish<E: MemEnv>(env: &mut E, ev: u32, p: u32) {
    env.w32(p, ev);
    let owner = env.r32(ev + 0x10);
    sys_set_event(env, owner, p);
}

/// C: `FUN_02012798`: address of PLAYER record `i` (`gd + 4 + i * 0x44`, no bounds check).
pub fn gd_player(gd: u32, i: u32) -> u32 {
    gd.wrapping_add(4).wrapping_add(i.wrapping_mul(0x44))
}

/// C: `FUN_02012950`: address `gd + 0x24 + i * 0x44` (PLAYER record `i` + 0x20, no bounds check).
pub fn gd_player_24(gd: u32, i: u32) -> u32 {
    gd.wrapping_add(0x24).wrapping_add(i.wrapping_mul(0x44))
}

/// C: `FUN_020127AC`: address of the 0x10-byte record `i` (`gd + 0x1D4 + i * 0x10`, no bounds check).
pub fn gd_record_10(gd: u32, i: u32) -> u32 {
    gd.wrapping_add(0x1D4).wrapping_add(i.wrapping_mul(0x10))
}

/// C: `FUN_020127B8`: `FUN_020127AC(gd, 3)`.
pub fn gd_record_10_3(gd: u32) -> u32 {
    gd_record_10(gd, 3)
}

/// C: `FUN_020127D0`: address of the 0x1E8-byte record `i` (`gd + 0x214 + i * 0x1E8`, no bounds check).
pub fn gd_record_1e8(gd: u32, i: u32) -> u32 {
    gd.wrapping_add(0x214).wrapping_add(i.wrapping_mul(0x1E8))
}

/// C: `FUN_020127C4`: `FUN_020127D0(gd, 1)`.
pub fn gd_record_1e8_1(gd: u32) -> u32 {
    gd_record_1e8(gd, 1)
}

/// C: `FUN_020127E0`: `FUN_0200A890(FUN_0200A864(*gd))`.
pub fn gd_save_chain_a<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    let x = h.fun_0200a864(save);
    h.fun_0200a890(x)
}

/// C: `FUN_020127F0`: `FUN_0200A898(FUN_0200A864(*gd))`.
pub fn gd_save_chain_b<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    let x = h.fun_0200a864(save);
    h.fun_0200a898(x)
}

/// C: `FUN_02012800`: copy 0x1E8 bytes from `src` to the record at `gd + 0x214`, then `FUN_02161D64`.
pub fn gd_set_record_1e8_0<H: GameDataHooks>(h: &mut H, gd: u32, src: u32) {
    h.mem().copy(gd + 0x214, src, 0x1E8);
    h.fun_02161d64(gd + 0x214);
}

/// C: `FUN_0201281C`: zero the record at `gd + 0x214`, then `FUN_02161D64(gd + 0x3FC)`.
pub fn gd_clear_record_1e8_0<H: GameDataHooks>(h: &mut H, gd: u32) {
    h.mem().fill(gd + 0x214, 0x1E8, 0);
    h.fun_02161d64(gd + 0x3FC);
}

/// C: `FUN_020128E4`: copy 0x1C bytes from `src` to the inline struct at `gd + 0x170`.
pub fn gd_copy_in_020128e4<E: MemEnv>(env: &mut E, gd: u32, src: u32) {
    env.mem().copy(gd + 0x170, src, 0x1C);
}

/// C: `FUN_02012900`: `*(u16 *)FUN_02008850(*(gd + 0x15C)) = v`.
pub fn gd_set_halfword_15c<H: GameDataHooks>(h: &mut H, gd: u32, v: u16) {
    let x = h.r32(gd + 0x15C);
    let p = h.fun_02008850(x);
    h.w16(p, v);
}

/// C: `FUN_02012914`: `*(u16 *)FUN_02008850(*(gd + 0x15C))`.
pub fn gd_get_halfword_15c<H: GameDataHooks>(h: &mut H, gd: u32) -> u16 {
    let x = h.r32(gd + 0x15C);
    let p = h.fun_02008850(x);
    h.r16(p)
}

/// C: `FUN_02012944`: `FUN_020127A4(gd) + 0x20` (`gd + 0x134`).
pub fn gd_addr_02012944(gd: u32) -> u32 {
    gd_addr_020127a4(gd).wrapping_add(0x20)
}

/// C: `FUN_020129DC`: `+0x1C9 = v`; a nonzero `v` also clears the counter at `+0x1C8`.
pub fn gd_set_020129dc<E: MemEnv>(env: &mut E, gd: u32, v: u8) {
    env.w8(gd + 0x1C9, v);
    if v != 0 {
        env.w8(gd + 0x1C8, 0);
    }
}

/// C: `FUN_020129F0`: `+0x1C8 = 0`.
pub fn gd_clear_1c8<E: MemEnv>(env: &mut E, gd: u32) {
    env.w8(gd + 0x1C8, 0);
}

/// C: `FUN_020129FC`: when `+0x1C9` is set, return the counter at `+0x1C8` and increment it (u8); else 0.
pub fn gd_next_1c8<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    if env.r8(gd + 0x1C9) == 0 {
        return 0;
    }
    let old = env.r8(gd + 0x1C8);
    env.w8(gd + 0x1C8, old.wrapping_add(1));
    old
}

/// C: `FUN_02012A60`: initialise the PLAYER record `rec` (`+0x1C = 0xFFFF`, the rest zero).
pub fn pl_init<E: MemEnv>(env: &mut E, rec: u32) {
    env.w8(rec + 0x3D, 0);
    env.w8(rec + 0x3A, 0);
    env.w16(rec, 0);
    env.w32(rec + 4, 0);
    env.w32(rec + 8, 0);
    env.w32(rec + 0xC, 0);
    env.w16(rec + 0x18, 0);
    env.w8(rec + 0x1B, 0);
    env.w32(rec + 0x30, 0);
    env.w8(rec + 0x12, 0);
    env.w8(rec + 0x13, 0);
    env.w16(rec + 0x10, 0);
    env.w16(rec + 0x16, 0);
    env.w16(rec + 0x14, 0);
    env.w16(rec + 0x1C, 0xFFFF);
}

/// C: `FUN_02012A90`: copy 12 bytes from `src` to `rec + 4`.
pub fn pl_copy_in_04<E: MemEnv>(env: &mut E, rec: u32, src: u32) {
    env.mem().copy(rec + 4, src, 12);
}

/// C: `FUN_02012AA4`: `rec[+0x10, +0x12, +0x14, +0x16]` = the four halfwords at `src`.
pub fn pl_copy_in_10<E: MemEnv>(env: &mut E, rec: u32, src: u32) {
    for k in 0..4 {
        let v = env.r16(src + 2 * k);
        env.w16(rec + 0x10 + 2 * k, v);
    }
}

/// C: `FUN_02012AD4`: `rec[+0x18]` = halfword `i` of the ROM table at `0x0209DFA8`.
///
/// UNVERIFIED: the original does not check `i`; indices past the 17 halfwords kept here read whatever
/// follows the table, this port returns 0 for them.
pub fn pl_set_18_from_table<E: MemEnv>(env: &mut E, rec: u32, i: u32) {
    let v = TABLE_0209DFA8.get(i as usize).copied().unwrap_or(0);
    env.w16(rec + 0x18, v);
}

/// C: `FUN_02012AE4`: classify the angle at `+0x18`: `(0x2000, 0x6000)` is 2, `[0x6000, 0xA000]` is 1,
/// `(0xA000, 0xE000)` is 3, anything else 0.
pub fn pl_direction<E: MemEnv>(env: &mut E, rec: u32) -> u32 {
    let a = env.r16(rec + 0x18);
    if a > 0x2000 && a < 0x6000 {
        2
    } else if (0x6000..=0xA000).contains(&a) {
        1
    } else if a > 0xA000 && a < 0xE000 {
        3
    } else {
        0
    }
}

/// C: `FUN_02012B98`: byte `i` of the array at `gd + 0x1D0` (no bounds check).
pub fn gd_get_byte_1d0<E: MemEnv>(env: &mut E, gd: u32, i: u32) -> u8 {
    env.r8(gd.wrapping_add(i).wrapping_add(0x1D0))
}

/// C: `FUN_02012BA4`: byte `i` of the array at `gd + 0x1D0` = `v`.
pub fn gd_set_byte_1d0<E: MemEnv>(env: &mut E, gd: u32, i: u32, v: u8) {
    env.w8(gd.wrapping_add(i).wrapping_add(0x1D0), v);
}

/// C: `FUN_02012BB0`: `FUN_02008448(*(gd + 0x198)) == 1`.
pub fn gd_flag_198<H: GameDataHooks>(h: &mut H, gd: u32) -> bool {
    let x = h.r32(gd + 0x198);
    h.fun_02008448(x) == 1
}

/// C: `FUN_02012BC8`: tail call `FUN_02008450(*(gd + 0x198), flag != 0)`.
pub fn gd_set_flag_198<H: GameDataHooks>(h: &mut H, gd: u32, flag: u32) {
    let x = h.r32(gd + 0x198);
    h.fun_02008450(x, (flag != 0) as u32);
}

/// C: `FUN_02012BE0`: `FUN_0200C418(FUN_0200C40C(*gd), a, b)`.
pub fn gd_call_0200c418<H: GameDataHooks>(h: &mut H, gd: u32, a: u32, b: u32) {
    let save = gd_get_02012ebc(h, gd);
    let x = h.fun_0200c40c(save);
    h.fun_0200c418(x, a, b);
}

/// C: `FUN_02012BF8`: `FUN_0200C44C(FUN_0200C404(*gd), a)`, whose 0 / 1 it returns (tail `bl` then `pop`; ~27 callers
/// in overlays 68 / 70 / 111 test or keep it, V-0022 audit).
pub fn gd_call_0200c44c<H: GameDataHooks>(h: &mut H, gd: u32, a: u32) -> u32 {
    let save = gd_get_02012ec0(h, gd);
    let x = h.fun_0200c404(save);
    h.fun_0200c44c(x, a)
}

/// C: `FUN_02012FBC(gd, b)`: tail call `FUN_020085D4(*(gd + 0x6AC), b)`; r1 passes through unchanged (the
/// play-time update `FUN_02021464` adds its seconds this way; F12).
pub fn gd_call_020085d4<H: GameDataHooks>(h: &mut H, gd: u32, b: u32) -> u32 {
    let x = h.r32(gd + 0x6AC);
    h.fun_020085d4(x, b)
}

/// C: `FUN_02012FF0`: the global `0x020A7298` = `v`.
pub fn global_handle_set<E: MemEnv>(env: &mut E, v: u32) {
    env.w32(GLOBAL_HANDLE_PTR, v);
}

/// C: `FUN_02013080`: signed byte `min(i, 4)` of the ROM table at `0x0209DFB0` (returned as a byte).
pub fn table_0209dfb0(i: u32) -> u8 {
    TABLE_0209DFB0[(i.min(4)) as usize] as u8
}

/// `FUN_0200873C(*0x020A7298)` then `FUN_02008374` (shared by the three handle functions).
fn handle_value<H: GameDataHooks>(h: &mut H) -> u32 {
    let handle = h.r32(GLOBAL_HANDLE_PTR);
    let x = h.fun_0200873c(handle);
    h.fun_02008374(x)
}

/// C: `FUN_02012FFC`: table `0x0209DFB0` at `min(FUN_02008374(FUN_0200873C(*0x020A7298)), 4)`.
pub fn handle_table_direct<H: GameDataHooks>(h: &mut H) -> u8 {
    let v = handle_value(h);
    table_0209dfb0(v)
}

/// C: `FUN_02013020`: the same with the value mapped `{0: 1, 1: 2, 2: 3, else v}`.
pub fn handle_table_plus_one<H: GameDataHooks>(h: &mut H) -> u8 {
    let v = handle_value(h);
    let m = match v {
        0 => 1,
        1 => 2,
        2 => 3,
        v => v,
    };
    table_0209dfb0(m)
}

/// C: `FUN_02013050`: the same with the value mapped `{0: 4, 1: 0, 2: 1, else v}`.
pub fn handle_table_rotated<H: GameDataHooks>(h: &mut H) -> u8 {
    let v = handle_value(h);
    let m = match v {
        0 => 4,
        1 => 0,
        2 => 1,
        v => v,
    };
    table_0209dfb0(m)
}

// ---------------------------------------------------------------------------------------------
// accessors
// ---------------------------------------------------------------------------------------------

/// C: `FUN_020120F0`: u32 at +0xc of the game system record.
pub fn sys_get_0c<E: MemEnv>(env: &mut E, sys: u32) -> u32 {
    env.r32(sys.wrapping_add(0xc))
}

/// C: `FUN_020120F4`: u32 at +0x1c of the game system record.
pub fn sys_get_gamedata<E: MemEnv>(env: &mut E, sys: u32) -> u32 {
    env.r32(sys.wrapping_add(0x1c))
}

/// C: `FUN_02012104`: u32 at +0x18 of the game system record.
pub fn sys_get_event<E: MemEnv>(env: &mut E, sys: u32) -> u32 {
    env.r32(sys.wrapping_add(0x18))
}

/// C: `FUN_0201210C`: u32 at +0x20 of the game system record.
pub fn sys_get_20<E: MemEnv>(env: &mut E, sys: u32) -> u32 {
    env.r32(sys.wrapping_add(0x20))
}

/// C: `FUN_02012134`: u32 at +0x2c of the game system record.
pub fn sys_get_2c<E: MemEnv>(env: &mut E, sys: u32) -> u32 {
    env.r32(sys.wrapping_add(0x2c))
}

/// C: `FUN_02012138`: u32 at +0x24 of the game system record.
pub fn sys_get_24<E: MemEnv>(env: &mut E, sys: u32) -> u32 {
    env.r32(sys.wrapping_add(0x24))
}

/// C: `FUN_0201212C`: u8 at +0x30 of the game system record.
pub fn sys_get_30<E: MemEnv>(env: &mut E, sys: u32) -> u8 {
    env.r8(sys.wrapping_add(0x30))
}

/// C: `FUN_02012144`: u8 at +0x32 of the game system record.
pub fn sys_get_32<E: MemEnv>(env: &mut E, sys: u32) -> u8 {
    env.r8(sys.wrapping_add(0x32))
}

/// C: `FUN_02012108`: u32 at +0x18 of the game system record = v.
pub fn sys_set_event<E: MemEnv>(env: &mut E, sys: u32, v: u32) {
    env.w32(sys.wrapping_add(0x18), v);
}

/// C: `FUN_02012110`: u32 at +0x20 of the game system record = v.
pub fn sys_set_20<E: MemEnv>(env: &mut E, sys: u32, v: u32) {
    env.w32(sys.wrapping_add(0x20), v);
}

/// C: `FUN_02012124`: u8 at +0x30 of the game system record = v.
pub fn sys_set_30<E: MemEnv>(env: &mut E, sys: u32, v: u8) {
    env.w8(sys.wrapping_add(0x30), v);
}

/// C: `FUN_0201213C`: u8 at +0x32 of the game system record = v.
pub fn sys_set_32<E: MemEnv>(env: &mut E, sys: u32, v: u8) {
    env.w8(sys.wrapping_add(0x32), v);
}

/// C: `FUN_020124E4`: u32 at +0x10 of the EVENT record.
pub fn ev_owner<E: MemEnv>(env: &mut E, ev: u32) -> u32 {
    env.r32(ev.wrapping_add(0x10))
}

/// C: `FUN_020124E8`: u32 at +0xc of the EVENT record.
pub fn ev_data<E: MemEnv>(env: &mut E, ev: u32) -> u32 {
    env.r32(ev.wrapping_add(0xc))
}

/// C: `FUN_020124F0`: u32 at +0x0 of the EVENT record.
pub fn ev_func<E: MemEnv>(env: &mut E, ev: u32) -> u32 {
    env.r32(ev.wrapping_add(0x0))
}

/// C: `FUN_020124EC`: address ev + 0x8 (EVENT record state word).
pub fn ev_state_addr(ev: u32) -> u32 {
    ev.wrapping_add(0x8)
}

/// C: `FUN_0201283C`: u32 at +0x1a0 of GAMEDATA.
pub fn gd_get_0201283c<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x1a0))
}

/// C: `FUN_02012844`: u32 at +0x158 of GAMEDATA.
pub fn gd_get_02012844<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x158))
}

/// C: `FUN_0201284C`: u32 at +0x160 of GAMEDATA.
pub fn gd_get_0201284c<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x160))
}

/// C: `FUN_02012870`: u32 at +0x164 of GAMEDATA.
pub fn gd_get_02012870<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x164))
}

/// C: `FUN_02012894`: u32 at +0x168 of GAMEDATA.
pub fn gd_get_02012894<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x168))
}

/// C: `FUN_020128B8`: u32 at +0x16c of GAMEDATA.
pub fn gd_get_020128b8<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x16c))
}

/// C: `FUN_02012924`: u32 at +0x18c of GAMEDATA.
pub fn gd_get_02012924<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x18c))
}

/// C: `FUN_0201292C`: u32 at +0x190 of GAMEDATA.
pub fn gd_get_0201292c<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x190))
}

/// C: `FUN_02012934`: u32 at +0x194 of GAMEDATA.
pub fn gd_get_02012934<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x194))
}

/// C: `FUN_0201293C`: u32 at +0x19c of GAMEDATA.
pub fn gd_get_0201293c<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x19c))
}

/// C: `FUN_0201295C`: u32 at +0x1a4 of GAMEDATA.
pub fn gd_get_0201295c<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x1a4))
}

/// C: `FUN_02012964`: u32 at +0x1a8 of GAMEDATA.
pub fn gd_get_02012964<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x1a8))
}

/// C: `FUN_0201296C`: u32 at +0x1ac of GAMEDATA.
pub fn event_work_ptr<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x1ac))
}

/// C: `FUN_02012974`: u32 at +0x1b0 of GAMEDATA.
pub fn gd_get_02012974<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x1b0))
}

/// C: `FUN_0201297C`: u32 at +0x690 of GAMEDATA.
pub fn gd_get_0201297c<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x690))
}

/// C: `FUN_02012A30`: u32 at +0x1b4 of GAMEDATA.
pub fn gd_get_02012a30<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x1b4))
}

/// C: `FUN_02012A38`: u32 at +0x5e4 of GAMEDATA.
pub fn gd_get_02012a38<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x5e4))
}

/// C: `FUN_02012A44`: u32 at +0x1bc of GAMEDATA.
pub fn gd_get_02012a44<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x1bc))
}

/// C: `FUN_02012A54`: u32 at +0x68c of GAMEDATA.
pub fn gd_get_02012a54<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x68c))
}

/// C: `FUN_02012B38`: u32 at +0x1b8 of GAMEDATA.
pub fn gd_get_02012b38<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x1b8))
}

/// C: `FUN_02012EBC`: u32 at +0x0 of GAMEDATA.
pub fn gd_get_02012ebc<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x0))
}

/// C: `FUN_02012EC0`: u32 at +0x0 of GAMEDATA.
pub fn gd_get_02012ec0<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x0))
}

/// C: `FUN_02012ED8`: u32 at +0x688 of GAMEDATA.
pub fn gd_get_02012ed8<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x688))
}

/// C: `FUN_02012F20`: u32 at +0x684 of GAMEDATA.
pub fn gd_get_02012f20<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x684))
}

/// C: `FUN_02012F80`: u32 at +0x6a8 of GAMEDATA.
pub fn gd_get_02012f80<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x6a8))
}

/// C: `FUN_02012FCC`: u32 at +0x6ac of GAMEDATA.
pub fn gd_get_02012fcc<E: MemEnv>(env: &mut E, gd: u32) -> u32 {
    env.r32(gd.wrapping_add(0x6ac))
}

/// C: `FUN_02012A4C`: u32 at +0x1bc of GAMEDATA = v.
pub fn gd_set_02012a4c<E: MemEnv>(env: &mut E, gd: u32, v: u32) {
    env.w32(gd.wrapping_add(0x1bc), v);
}

/// C: `FUN_02012EE4`: u32 at +0x688 of GAMEDATA = v.
pub fn gd_set_02012ee4<E: MemEnv>(env: &mut E, gd: u32, v: u32) {
    env.w32(gd.wrapping_add(0x688), v);
}

/// C: `FUN_02012F74`: u32 at +0x6a8 of GAMEDATA = v.
pub fn gd_set_02012f74<E: MemEnv>(env: &mut E, gd: u32, v: u32) {
    env.w32(gd.wrapping_add(0x6a8), v);
}

/// C: `FUN_02012984`: u8 at +0x1c4 of GAMEDATA.
pub fn gd_get_02012984<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1c4))
}

/// C: `FUN_0201298C`: u8 at +0x1c4 of GAMEDATA = v.
pub fn gd_set_0201298c<E: MemEnv>(env: &mut E, gd: u32, v: u8) {
    env.w8(gd.wrapping_add(0x1c4), v);
}

/// C: `FUN_02012994`: u8 at +0x1c5 of GAMEDATA.
pub fn gd_get_02012994<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1c5))
}

/// C: `FUN_020129A0`: u8 at +0x1c5 of GAMEDATA = v.
pub fn gd_set_020129a0<E: MemEnv>(env: &mut E, gd: u32, v: u8) {
    env.w8(gd.wrapping_add(0x1c5), v);
}

/// C: `FUN_020129AC`: u8 at +0x1c6 of GAMEDATA.
pub fn gd_get_020129ac<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1c6))
}

/// C: `FUN_020129B8`: u8 at +0x1c6 of GAMEDATA = v.
pub fn gd_set_020129b8<E: MemEnv>(env: &mut E, gd: u32, v: u8) {
    env.w8(gd.wrapping_add(0x1c6), v);
}

/// C: `FUN_020129C4`: u8 at +0x1c7 of GAMEDATA.
pub fn gd_get_020129c4<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1c7))
}

/// C: `FUN_020129D0`: u8 at +0x1c7 of GAMEDATA = v.
pub fn gd_set_020129d0<E: MemEnv>(env: &mut E, gd: u32, v: u8) {
    env.w8(gd.wrapping_add(0x1c7), v);
}

/// C: `FUN_02012B40`: u8 at +0x1ca of GAMEDATA.
pub fn gd_get_02012b40<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1ca))
}

/// C: `FUN_02012B4C`: u8 at +0x1ca of GAMEDATA = v.
pub fn gd_set_02012b4c<E: MemEnv>(env: &mut E, gd: u32, v: u8) {
    env.w8(gd.wrapping_add(0x1ca), v);
}

/// C: `FUN_02012B58`: u8 at +0x1cb of GAMEDATA.
pub fn gd_get_02012b58<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1cb))
}

/// C: `FUN_02012B64`: u8 at +0x1cb of GAMEDATA = v.
pub fn gd_set_02012b64<E: MemEnv>(env: &mut E, gd: u32, v: u8) {
    env.w8(gd.wrapping_add(0x1cb), v);
}

/// C: `FUN_02012B70`: u8 at +0x1cc of GAMEDATA.
pub fn gd_get_02012b70<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1cc))
}

/// C: `FUN_02012B78`: u8 at +0x1cc of GAMEDATA = v.
pub fn gd_set_02012b78<E: MemEnv>(env: &mut E, gd: u32, v: u8) {
    env.w8(gd.wrapping_add(0x1cc), v);
}

/// C: `FUN_02012B80`: u8 at +0x1cd of GAMEDATA.
pub fn gd_get_02012b80<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1cd))
}

/// C: `FUN_02012B8C`: u8 at +0x1cd of GAMEDATA = v.
pub fn gd_set_02012b8c<E: MemEnv>(env: &mut E, gd: u32, v: u8) {
    env.w8(gd.wrapping_add(0x1cd), v);
}

/// C: `FUN_02012FB0`: u8 at +0x1cf of GAMEDATA.
pub fn gd_get_02012fb0<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1cf))
}

/// C: `FUN_02012FA4`: u8 at +0x1cf of GAMEDATA = v.
pub fn gd_set_02012fa4<E: MemEnv>(env: &mut E, gd: u32, v: u8) {
    env.w8(gd.wrapping_add(0x1cf), v);
}

/// C: `FUN_02012A24`: u8 at +0x1c9 of GAMEDATA.
pub fn gd_get_02012a24<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1c9))
}

/// C: `FUN_02012E40`: u8 at +0x1ce of GAMEDATA.
pub fn gd_get_02012e40<E: MemEnv>(env: &mut E, gd: u32) -> u8 {
    env.r8(gd.wrapping_add(0x1ce))
}

/// C: `FUN_020127A4`: address gd + 0x114 (the 0x20-byte block).
pub fn gd_addr_020127a4(gd: u32) -> u32 {
    gd.wrapping_add(0x114)
}

/// C: `FUN_020128DC`: address gd + 0x170 (inline 0x1C-byte struct).
pub fn gd_addr_020128dc(gd: u32) -> u32 {
    gd.wrapping_add(0x170)
}

/// C: `FUN_02012C0C`: address gd + 0x5e8 (inline block).
pub fn gd_addr_02012c0c(gd: u32) -> u32 {
    gd.wrapping_add(0x5e8)
}

/// C: `FUN_02012EC4`: address gd + 0x5ec (inline block).
pub fn gd_addr_02012ec4(gd: u32) -> u32 {
    gd.wrapping_add(0x5ec)
}

/// C: `FUN_02012ED0`: address gd + 0x680 (inline block).
pub fn gd_addr_02012ed0(gd: u32) -> u32 {
    gd.wrapping_add(0x680)
}

/// C: `FUN_02012F38`: address gd + 0x694 (inline block).
pub fn gd_addr_02012f38(gd: u32) -> u32 {
    gd.wrapping_add(0x694)
}

/// C: `FUN_02012F50`: address gd + 0x6a4 (inline block).
pub fn gd_addr_02012f50(gd: u32) -> u32 {
    gd.wrapping_add(0x6a4)
}

/// C: `FUN_02012AC0`: u8 at +0x1b of a PLAYER record.
pub fn pl_get_1b<E: MemEnv>(env: &mut E, rec: u32) -> u8 {
    env.r8(rec.wrapping_add(0x1b))
}

/// C: `FUN_02012AC8`: u16 at +0x0 of a PLAYER record.
pub fn pl_get_00<E: MemEnv>(env: &mut E, rec: u32) -> u16 {
    env.r16(rec.wrapping_add(0x0))
}

/// C: `FUN_02012AD0`: u16 at +0x18 of a PLAYER record.
pub fn pl_get_18<E: MemEnv>(env: &mut E, rec: u32) -> u16 {
    env.r16(rec.wrapping_add(0x18))
}

/// C: `FUN_02012B28`: u32 at +0x40 of a PLAYER record.
pub fn pl_get_40<E: MemEnv>(env: &mut E, rec: u32) -> u32 {
    env.r32(rec.wrapping_add(0x40))
}

/// C: `FUN_02012B34`: u16 at +0x1c of a PLAYER record.
pub fn pl_get_1c<E: MemEnv>(env: &mut E, rec: u32) -> u16 {
    env.r16(rec.wrapping_add(0x1c))
}

/// C: `FUN_02012ABC`: u8 at +0x1b of a PLAYER record = v.
pub fn pl_set_1b<E: MemEnv>(env: &mut E, rec: u32, v: u8) {
    env.w8(rec.wrapping_add(0x1b), v);
}

/// C: `FUN_02012AC4`: u16 at +0x0 of a PLAYER record = v.
pub fn pl_set_00<E: MemEnv>(env: &mut E, rec: u32, v: u16) {
    env.w16(rec.wrapping_add(0x0), v);
}

/// C: `FUN_02012ACC`: u16 at +0x18 of a PLAYER record = v.
pub fn pl_set_18<E: MemEnv>(env: &mut E, rec: u32, v: u16) {
    env.w16(rec.wrapping_add(0x18), v);
}

/// C: `FUN_02012B2C`: u32 at +0x40 of a PLAYER record = v.
pub fn pl_set_40<E: MemEnv>(env: &mut E, rec: u32, v: u32) {
    env.w32(rec.wrapping_add(0x40), v);
}

/// C: `FUN_02012B30`: u16 at +0x1c of a PLAYER record = v.
pub fn pl_set_1c<E: MemEnv>(env: &mut E, rec: u32, v: u16) {
    env.w16(rec.wrapping_add(0x1c), v);
}

/// C: `FUN_02012AA0`: address rec + 0x4 (PLAYER record +4).
pub fn pl_addr_04(rec: u32) -> u32 {
    rec.wrapping_add(0x4)
}

/// C: `FUN_02012AB8`: address rec + 0x10 (PLAYER record +0x10).
pub fn pl_addr_10(rec: u32) -> u32 {
    rec.wrapping_add(0x10)
}

/// C: `FUN_02012EF0`: save block 0x3a of the save data `*gd` (tail call `FUN_020071CC`).
pub fn gd_save_block_02012ef0<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_020071cc(save, 0x3a)
}

/// C: `FUN_02012EFC`: save block 0x3b of the save data `*gd` (tail call `FUN_020071CC`).
pub fn gd_save_block_02012efc<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_020071cc(save, 0x3b)
}

/// C: `FUN_02012F2C`: save block 0x26 of the save data `*gd` (tail call `FUN_020071CC`).
pub fn gd_save_block_02012f2c<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_020071cc(save, 0x26)
}

/// C: `FUN_02012F44`: save block 0x2a of the save data `*gd` (tail call `FUN_020071CC`).
pub fn gd_save_block_02012f44<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_020071cc(save, 0x2a)
}

/// C: `FUN_02012F68`: save block 0x42 of the save data `*gd` (tail call `FUN_020071CC`).
pub fn gd_save_block_02012f68<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_020071cc(save, 0x42)
}

/// C: `FUN_02012FD8`: save block 0x43 of the save data `*gd` (tail call `FUN_020071CC`).
pub fn gd_save_block_02012fd8<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_020071cc(save, 0x43)
}

/// C: `FUN_02012FE4`: save block 0x2D of the save data `save` (tail call `FUN_020071CC`).
pub fn save_block_2d<H: GameDataHooks>(h: &mut H, save: u32) -> u32 {
    h.fun_020071cc(save, 0x2D)
}

/// C: `FUN_02012F08`: tail call `FUN_0200C0F0(*gd)`.
pub fn gd_save_call_02012f08<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_0200c0f0(save)
}

/// C: `FUN_02012F14`: tail call `FUN_02009EC8(*gd)`.
pub fn gd_save_call_02012f14<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_02009ec8(save)
}

/// C: `FUN_02012F5C`: tail call `FUN_020089C0(*gd)`.
pub fn gd_save_call_02012f5c<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_020089c0(save)
}

/// C: `FUN_02012F8C`: tail call `FUN_02009448(*gd)`.
pub fn gd_save_call_02012f8c<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_02009448(save)
}

/// C: `FUN_02012F98`: tail call `FUN_02007B44(*gd)`.
pub fn gd_save_call_02012f98<H: GameDataHooks>(h: &mut H, gd: u32) -> u32 {
    let save = h.r32(gd);
    h.fun_02007b44(save)
}

/// C: `FUN_02012854`: copy 0x1C bytes from `src` to the struct `*(gd + 0x160)` points to.
pub fn gd_copy_in_02012854<E: MemEnv>(env: &mut E, gd: u32, src: u32) {
    let dst = env.r32(gd + 0x160);
    env.mem().copy(dst, src, 0x1C);
}

/// C: `FUN_02012878`: copy 0x1C bytes from `src` to the struct `*(gd + 0x164)` points to.
pub fn gd_copy_in_02012878<E: MemEnv>(env: &mut E, gd: u32, src: u32) {
    let dst = env.r32(gd + 0x164);
    env.mem().copy(dst, src, 0x1C);
}

/// C: `FUN_0201289C`: copy 0x1C bytes from `src` to the struct `*(gd + 0x168)` points to.
pub fn gd_copy_in_0201289c<E: MemEnv>(env: &mut E, gd: u32, src: u32) {
    let dst = env.r32(gd + 0x168);
    env.mem().copy(dst, src, 0x1C);
}

/// C: `FUN_020128C0`: copy 0x1C bytes from `src` to the struct `*(gd + 0x16c)` points to.
pub fn gd_copy_in_020128c0<E: MemEnv>(env: &mut E, gd: u32, src: u32) {
    let dst = env.r32(gd + 0x16c);
    env.mem().copy(dst, src, 0x1C);
}

#[cfg(test)]
mod tests;
