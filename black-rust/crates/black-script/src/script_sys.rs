//! F7: script system glue (`script_sys.c` + `script_work.c`, overlay 10 0x02158804-0x021591FE and 0x02159960-0x02159B48)
//! and the scrcmd ctx getters 0x0215A5CC-0x0215A5FA.
//!
//! Every function of the area is specced in `tools/rom-diff/specs/black-script/script_sys.spec` (its header documents
//! the SYS / SCRIPT_WORK / VM / scrcmd CTX layouts and the command dispatch); all 90 are ported here and checked with
//! `tools/check-fn.sh black-script script_sys`. The interpreter loop itself is `vm.c` in main ([`crate::vm`]); this file
//! reaches it through the `ScriptSysHooks::fun_0201xxxx` methods (they default to the `vm` ports).
//!
//! Objects (all pointers are `u32` addresses in the [`Mem`](crate::hooks::Mem) window):
//! * **SYS** (0x18 bytes, `sys_*`): `+0` u16 heap, `+2` u8 live VM count, `+4/+8/+0xC` VM slots, `+0x10` end callback,
//!   `+0x14` its argument. A script event's work block is `{SW, SYS}`; its event function is [`event_fn`].
//! * **SW** = SCRIPT_WORK (0x14C bytes, `sw_*`): magic 0x3643F, script id, MMDL, heap, GAMESYS, GMEVENT, field block,
//!   message resources, script vars `0x8000..` (u16 at `SW + 0x7C + (id - 0x8000) * 2`), `+0x148` the scrcmd work.
//! * **CTX** = scrcmd ctx (0x24 bytes, created by `0x0215A55C`, F8): `+0` heap, `+4` zone, `+6` script id, `+8` mode,
//!   `+0xC` mode != 0, `+0x10` SW, `+0x20` -> scrcmd work `{SW, GAMESYS, GAMEDATA, ..}` (`ctx_*` getters).
//!
//! Var ids: `< 0x4000` and `>= 0xC000` are literals, `0x4000..0x7FFF` event-work vars (`black_field::event_work`),
//! `0x8000..0xBFFF` script vars in SW.

use crate::hooks::{MemEnv, ScriptSysHooks};
use crate::vm;

mod tables;
pub use tables::{CMD_PERMISSIONS, INIT_PROCS, SCRIPT_RANGES};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_8804,
    0x0215_882C,
    0x0215_886C,
    0x0215_8894,
    0x0215_88A4,
    0x0215_88B0,
    0x0215_88C0,
    0x0215_88D0,
    0x0215_88DC,
    0x0215_8920,
    0x0215_897C,
    0x0215_898C,
    0x0215_89A0,
    0x0215_89D0,
    0x0215_89DC,
    0x0215_89F4,
    0x0215_8A10,
    0x0215_8A20,
    0x0215_8A34,
    0x0215_8A90,
    0x0215_8AC8,
    0x0215_8AD4,
    0x0215_8AE0,
    0x0215_8B00,
    0x0215_8B3C,
    0x0215_8B6C,
    0x0215_8BFC,
    0x0215_8CB4,
    0x0215_8CD0,
    0x0215_8D78,
    0x0215_8D90,
    0x0215_8DB0,
    0x0215_8DBC,
    0x0215_8DE8,
    0x0215_8E38,
    0x0215_8E54,
    0x0215_8E7C,
    0x0215_8F18,
    0x0215_8F50,
    0x0215_8F54,
    0x0215_8F58,
    0x0215_8F6C,
    0x0215_8F74,
    0x0215_8F78,
    0x0215_8F7C,
    0x0215_8F80,
    0x0215_8F84,
    0x0215_8F88,
    0x0215_8F8C,
    0x0215_8F90,
    0x0215_8F94,
    0x0215_8FB4,
    0x0215_8FB8,
    0x0215_8FBC,
    0x0215_8FD0,
    0x0215_8FE0,
    0x0215_901C,
    0x0215_9030,
    0x0215_9058,
    0x0215_909C,
    0x0215_90D4,
    0x0215_90F4,
    0x0215_90FC,
    0x0215_9104,
    0x0215_910C,
    0x0215_9114,
    0x0215_911C,
    0x0215_9124,
    0x0215_9130,
    0x0215_9144,
    0x0215_9168,
    0x0215_9198,
    0x0215_91B4,
    0x0215_9960,
    0x0215_9974,
    0x0215_9984,
    0x0215_9998,
    0x0215_99A8,
    0x0215_9A64,
    0x0215_9AE8,
    0x0215_9B10,
    0x0215_9B38,
    0x0215_A5CC,
    0x0215_A5D0,
    0x0215_A5D4,
    0x0215_A5D8,
    0x0215_A5DC,
    0x0215_A5E4,
    0x0215_A5EC,
    0x0215_A5F4,
];

/// One entry of the script id table `0x021701A8` (see [`SCRIPT_RANGES`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScriptRange {
    pub lo: u16,
    pub hi: u16,
    pub file: u16,
    pub ty: u16,
    pub msg: u16,
}

/// Offset of the script var block in SCRIPT_WORK.
pub const SW_VARS_OFS: u32 = 0x7C;
/// First script var id (stored in SCRIPT_WORK).
pub const SW_VAR_BASE: u32 = 0x8000;
/// First event-work var id.
pub const EVENT_VAR_BASE: u32 = 0x4000;
/// End (exclusive) of the script var ids.
pub const SW_VAR_END: u32 = 0xC000;
/// SCRIPT_WORK size and magic word (+0).
pub const SW_SIZE: u32 = 0x14C;
pub const SW_MAGIC: u32 = 0x0003_643F;
/// SW + 0x148: the scrcmd work (`FUN_0215A514`).
pub const SW_SCRCMD_WORK: u32 = 0x148;
/// SYS size.
pub const SYS_SIZE: u32 = 0x18;
/// Number of VM slots in SYS.
pub const SYS_SLOTS: u32 = 3;
/// Address of the 14-bit init-proc mask (overlay 10 BSS; [`proc_mask_set`] / [`proc_mask_clear`] / [`proc_mask_test`]).
pub const PROC_MASK_ADDR: u32 = 0x0217_2BA0;
/// Number of bits of the init-proc mask (entries of the proc table 0x021705F0).
pub const PROC_MASK_BITS: i32 = 14;

/// Heap id the script events use (`4`, hard-coded in `0x02158920` / `0x02158A34`).
pub const HEAP_SCRIPT: u32 = 4;
/// The VM init record of a script (`0x02158BFC`): 0x100 call-stack words, 0x40 regs, the command table and its count.
pub const VM_STACK_WORDS: u16 = 0x100;
pub const VM_REG_WORDS: u16 = 0x40;
/// Command table (609 handler addresses, index = command id; F8) and its count (the word at `0x021705D8`).
pub const CMD_TABLE: u32 = 0x0217_062C;
pub const CMD_COUNT: u32 = 609;
/// ARC holding the script files.
pub const ARC_SCRIPTS: u32 = 57;

/// Strings passed to `Heap_AllocDebug` / the assertion hook.
pub const FILE_SCRIPT_SYS: u32 = 0x0217_2888;
pub const FILE_SCRIPT_WORK: u32 = 0x0217_289C;
/// The empty string at `0x02172898` (file and expression of the assertion in [`script_lookup`]).
pub const ASSERT_STR: u32 = 0x0217_2898;

/// Code addresses (Thumb bit set) stored as function pointers.
pub const EVENT_FN: u32 = 0x0215_88DD;
pub const END_CALLBACK_VENEER: u32 = 0x0215_889D;
pub const DISPATCH_HOOK: u32 = 0x0215_8DE9;
pub const WAIT_PAD_A: u32 = 0x0215_9961;
pub const WAIT_PAD_B: u32 = 0x0215_9985;
pub const WAIT_311: u32 = 0x0215_99A9;
/// Handlers of commands 49, 50 and 311 (entries of [`CMD_TABLE`]).
pub const CMD_WAIT_PAD_A: u32 = 0x0215_9975;
pub const CMD_WAIT_PAD_B: u32 = 0x0215_9999;
pub const CMD_311: u32 = 0x0215_9A65;

/// `MIi_CpuFill32(v, dst, n)` (`0x02082A7C`), inlined.
fn fill32<H: MemEnv>(h: &mut H, v: u32, dst: u32, n: u32) {
    let mut i = 0;
    while i < n {
        h.w32(dst.wrapping_add(i), v);
        i += 4;
    }
}

/// `MIi_CpuCopy16(src, dst, n)` (`0x02082A60`), inlined.
fn copy16<H: MemEnv>(h: &mut H, src: u32, dst: u32, n: u32) {
    let mut i = 0;
    while i < n {
        let v = h.r16(src.wrapping_add(i));
        h.w16(dst.wrapping_add(i), v);
        i += 2;
    }
}

/// Thumb `lsls rd, rs` (shift by the low byte of `s`).
fn lsl_reg(v: u32, s: u32) -> u32 {
    let s = s & 0xFF;
    if s >= 32 {
        0
    } else {
        v << s
    }
}

/// Thumb `asrs rd, rs` (shift by the low byte of `s`).
fn asr_reg(v: u32, s: u32) -> u32 {
    let s = s & 0xFF;
    if s >= 32 {
        ((v as i32) >> 31) as u32
    } else {
        ((v as i32) >> s) as u32
    }
}

// ======================================================================== script_sys.c: SYS

/// C: `FUN_overlay_d_10__02158804`: create a SYS: `Heap_AllocDebug(heap, 0x18, 1, "script_sys.c", 0xEF)`, u16 heap at
/// +0 (the rest is left as allocated). Returns it.
pub fn sys_new<H: ScriptSysHooks>(h: &mut H, heap: u32) -> u32 {
    let sys = h.heap_alloc(heap, SYS_SIZE, 1, FILE_SCRIPT_SYS, 0xEF);
    h.w16(sys, heap as u16);
    sys
}

/// C: `FUN_overlay_d_10__0215882c`: run every live VM for one frame; a VM that stops is removed from its slot (count
/// -1) and freed ([`vm_free`]). Returns 1 while VMs are left.
pub fn sys_step<H: ScriptSysHooks>(h: &mut H, sys: u32) -> u32 {
    for i in 0..SYS_SLOTS {
        let slot = sys.wrapping_add(i * 4).wrapping_add(4);
        let vm = h.r32(slot);
        if vm != 0 && h.fun_02011298(vm) == 0 {
            h.w32(slot, 0);
            let n = h.r8(sys.wrapping_add(2));
            h.w8(sys.wrapping_add(2), n.wrapping_sub(1));
            vm_free(h, vm);
        }
    }
    (h.r8(sys.wrapping_add(2)) != 0) as u32
}

/// C: `FUN_overlay_d_10__0215886c`: put `vm` into the first free slot (count +1); returns the slot, or 3 when all are taken
/// (the VM is then not stored).
pub fn sys_add_vm<H: MemEnv>(h: &mut H, sys: u32, vm: u32) -> u32 {
    for i in 0..SYS_SLOTS {
        let slot = sys.wrapping_add(i * 4).wrapping_add(4);
        if h.r32(slot) == 0 {
            h.w32(slot, vm);
            let n = h.r8(sys.wrapping_add(2));
            h.w8(sys.wrapping_add(2), n.wrapping_add(1));
            return i;
        }
    }
    SYS_SLOTS
}

/// C: `FUN_overlay_d_10__02158894`: the VM in slot `i` (no range check).
pub fn sys_vm<H: MemEnv>(h: &mut H, sys: u32, i: u32) -> u32 {
    h.r32(sys.wrapping_add(i.wrapping_mul(4)).wrapping_add(4))
}

/// C: `FUN_overlay_d_10__021588a4`: end callback = `0x0215889D` (veneer to main `0x0201235C`), its argument = `arg`.
pub fn sys_set_end_callback<H: MemEnv>(h: &mut H, sys: u32, arg: u32) {
    h.w32(sys.wrapping_add(0x14), arg);
    h.w32(sys.wrapping_add(0x10), END_CALLBACK_VENEER);
}

/// C: `FUN_overlay_d_10__021588b0`: call the end callback as `cb(x, arg)`; returns its result.
pub fn sys_run_end_callback<H: ScriptSysHooks>(h: &mut H, sys: u32, x: u32) -> u32 {
    let arg = h.r32(sys.wrapping_add(0x14));
    let f = h.r32(sys.wrapping_add(0x10));
    h.call_end_callback(f, x, arg)
}

/// C: `FUN_overlay_d_10__021588c0`: 1 if an end callback is set.
pub fn sys_has_end_callback<H: MemEnv>(h: &mut H, sys: u32) -> u32 {
    (h.r32(sys.wrapping_add(0x10)) != 0) as u32
}

/// C: `FUN_overlay_d_10__021588d0`: the current zone id (`FUN_02012AC8(FUN_020120F8(gamesys))`).
pub fn zone_of<H: ScriptSysHooks>(h: &mut H, gamesys: u32) -> u32 {
    let gd = h.fun_020120f8(gamesys);
    h.fun_02012ac8(gd)
}

/// C: `sub_021588dc`: the script event's function (`work` = `{SW, SYS}`). While [`sys_step`] reports live VMs: 0
/// (continue). Then free SW ([`sw_free`]); with an end callback: run it ([`sys_run_end_callback`]`(SYS, event)`), free
/// SYS and return 0 (the callback changed the event); else free SYS and return 1 (event done).
pub fn event_fn<H: ScriptSysHooks>(h: &mut H, event: u32, _seq: u32, work: u32) -> u32 {
    let sys = h.r32(work.wrapping_add(4));
    let sw = h.r32(work);
    if sys_step(h, sys) == 1 {
        return 0;
    }
    sw_free(h, sw);
    if sys_has_end_callback(h, sys) == 1 {
        sys_run_end_callback(h, sys, event);
        h.heap_free(sys);
        return 0;
    }
    h.heap_free(sys);
    1
}

/// C: `FUN_overlay_d_10__02158920`: start a script event for script `id` (talk target `mmdl`, `s0` = SW +0x20):
/// event = `FUN_020122C0(gamesys, 0, 0x021588DD, 8)`; its work = `{SW, SYS}` with SYS = [`sys_new`]`(4)` and SW =
/// [`sw_new`]`(4, gamesys, event, id, s0, 0)`; [`sw_set_mmdl`]; the script's VM is added ([`sw_start_script`] in the
/// current zone, result ignored). Returns the event. `_r1` is not read.
pub fn start_event<H: ScriptSysHooks>(
    h: &mut H,
    gamesys: u32,
    _r1: u32,
    id: u32,
    mmdl: u32,
    s0: u32,
) -> u32 {
    let event = h.fun_020122c0(gamesys, 0, EVENT_FN, 8);
    let work = h.fun_020124e8(event);
    let sys = sys_new(h, HEAP_SCRIPT);
    h.w32(work.wrapping_add(4), sys);
    let sw = sw_new(h, HEAP_SCRIPT, gamesys, event, id, s0, 0);
    h.w32(work, sw);
    sw_set_mmdl(h, sw, mmdl);
    let zone = zone_of(h, gamesys);
    let sw = h.r32(work);
    sw_start_script(h, sw, zone, id);
    event
}

/// C: `FUN_0215897C`: call the child event `child` from the script's event (`FUN_02012374(SW->event, child)`).
pub fn sw_call_child<H: ScriptSysHooks>(h: &mut H, sw: u32, child: u32) {
    let event = sw_event(h, sw);
    h.fun_02012374(event, child);
}

/// C: `FUN_overlay_d_10__0215898c`: set the end callback of the script's SYS (if it has one) with argument `arg`.
pub fn sw_set_end_callback<H: ScriptSysHooks>(h: &mut H, sw: u32, arg: u32) {
    let sys = sw_sys(h, sw);
    if sys != 0 {
        sys_set_end_callback(h, sys, arg);
    }
}

/// C: `FUN_overlay_d_10__021589a0`: start script `id` in another VM of the script's SYS: 3 without a SYS, else
/// [`vm_new`]`(SYS->heap, sw, zone, id, 0)` goes into a free slot ([`sys_add_vm`]; the slot, or 3 when full).
pub fn sw_start_script<H: ScriptSysHooks>(h: &mut H, sw: u32, zone: u32, id: u32) -> u32 {
    let sys = sw_sys(h, sw);
    if sys == 0 {
        return SYS_SLOTS;
    }
    let heap = h.r16(sys) as u32;
    let vm = vm_new(h, heap, sw, zone, id, 0);
    sys_add_vm(h, sys, vm)
}

/// C: `FUN_021589D0`: the SW of a script event (word 0 of its work).
pub fn event_sw<H: ScriptSysHooks>(h: &mut H, event: u32) -> u32 {
    let work = h.fun_020124e8(event);
    h.r32(work)
}

/// C: `FUN_overlay_d_10__021589dc`: the SYS of the script (word 1 of its event's work); 0 when SW has no event.
pub fn sw_sys<H: ScriptSysHooks>(h: &mut H, sw: u32) -> u32 {
    let event = sw_event(h, sw);
    if event == 0 {
        return 0;
    }
    let work = h.fun_020124e8(event);
    h.r32(work.wrapping_add(4))
}

/// C: `FUN_021589f4`: clear the zone-local event state: flags 0..=99 (`FUN_02014368(ew, 0, 99)`) and vars
/// 0x4000..0x401F (`FUN_020143D8(ew, 0x4000, 0x401F)`).
pub fn clear_local_flags<H: ScriptSysHooks>(h: &mut H, ew: u32) {
    h.fun_02014368(ew, 0, 99);
    h.fun_020143d8(ew, EVENT_VAR_BASE, EVENT_VAR_BASE + 0x1F);
}

// ---------------------------------------------------------------- synchronous scripts

/// C: `FUN_overlay_d_10__02158a34`: run script `id` to the end at once (no event): without `sw` a temporary SW is made
/// ([`sw_new`]`(4, gamesys, 0, id, 0, mode)`); the VM ([`vm_new`]`(4, sw, zone, id, mode)`) is stepped until the step
/// returns something other than 1; then the VM and the temporary SW are freed.
pub fn run_sync<H: ScriptSysHooks>(h: &mut H, gamesys: u32, sw: u32, id: u32, mode: u32) {
    let mut sw = sw;
    let mut tmp = 0;
    if sw == 0 {
        tmp = sw_new(h, HEAP_SCRIPT, gamesys, 0, id, 0, mode);
        sw = tmp;
    }
    let zone = zone_of(h, gamesys);
    let vm = vm_new(h, HEAP_SCRIPT, sw, zone, id, mode);
    while h.fun_02011298(vm) == 1 {}
    vm_free(h, vm);
    if tmp != 0 {
        sw_free(h, tmp);
    }
}

/// C: `FUN_02158a10`: [`run_sync`]`(gamesys, 0, 9600, 2)`.
pub fn run_sync_9600<H: ScriptSysHooks>(h: &mut H, gamesys: u32) {
    run_sync(h, gamesys, 0, 9600, 2);
}

/// C: `FUN_overlay_d_10__02158a20`: [`run_sync`]`(gamesys, 0, 9601, 2)`.
pub fn run_sync_9601<H: ScriptSysHooks>(h: &mut H, gamesys: u32) {
    run_sync(h, gamesys, 0, 9601, 2);
}

/// The zone's special-script table (`FUN_02162450(FUN_02012844(gamedata))`).
fn special_table<H: ScriptSysHooks>(h: &mut H, gd: u32) -> u32 {
    let x = h.fun_02012844(gd);
    h.fun_02162450(x)
}

/// C: `FUN_overlay_d_10__02158a90`: run the zone's special script `key` synchronously ([`find_special`] in the zone's
/// table, then [`run_sync`]`(gamesys, 0, id, mode)`): 1, or 0 when the zone has none. `_x` is not read.
pub fn run_special<H: ScriptSysHooks>(
    h: &mut H,
    gamesys: u32,
    _x: u32,
    key: u32,
    mode: u32,
) -> u32 {
    let gd = h.fun_020120f4(gamesys);
    let recs = special_table(h, gd);
    let id = find_special(h, recs, key);
    if id == 0xFFFF {
        return 0;
    }
    run_sync(h, gamesys, 0, id, mode);
    1
}

/// C: `FUN_02158AC8`: [`run_special`]`(gamesys, x, 4, 1)`.
pub fn run_special_4<H: ScriptSysHooks>(h: &mut H, gamesys: u32, x: u32) {
    run_special(h, gamesys, x, 4, 1);
}

/// C: `FUN_02158AD4`: [`run_special`]`(gamesys, x, 3, 1)`.
pub fn run_special_3<H: ScriptSysHooks>(h: &mut H, gamesys: u32, x: u32) {
    run_special(h, gamesys, x, 3, 1);
}

/// C: `FUN_02158AE0`: on entering a zone: [`clear_local_flags`] of the event work, then [`run_special`]`(gamesys, x, 2,
/// 2)`.
pub fn run_special_enter<H: ScriptSysHooks>(h: &mut H, gamesys: u32, x: u32) {
    let gd = h.fun_020120f4(gamesys);
    let ew = h.fun_0201296c(gd);
    clear_local_flags(h, ew);
    run_special(h, gamesys, x, 2, 2);
}

/// C: `FUN_02158B00`: the zone's conditional special script 1 ([`find_conditional`]); none (0xFFFF) -> 0, else
/// `FUN_02158768(gamesys, id, 0, x)`.
pub fn run_conditional<H: ScriptSysHooks>(h: &mut H, gamesys: u32, x: u32) -> u32 {
    let gd = h.fun_020120f4(gamesys);
    let recs = special_table(h, gd);
    let id = find_conditional(h, gd, recs, 1);
    if id == 0xFFFF {
        return 0;
    }
    h.fun_02158768(gamesys, id, 0, x)
}

/// Unaligned little-endian u16 (the original reads the table bytewise).
fn ru16<H: MemEnv>(h: &mut H, p: u32) -> u32 {
    let lo = h.r8(p) as u32;
    let hi = h.r8(p.wrapping_add(1)) as u32;
    (lo + (hi << 8)) & 0xFFFF
}

/// C: `FUN_02158b3c`: script id of the first 6-byte record `{u16 key, u16 id, u16}` whose key equals `key`; 0xFFFF at
/// the key-0 end.
pub fn find_special<H: MemEnv>(h: &mut H, recs: u32, key: u32) -> u32 {
    let mut p = recs;
    loop {
        let k = ru16(h, p);
        if k == 0 {
            return 0xFFFF;
        }
        if key == k {
            return ru16(h, p.wrapping_add(2));
        }
        p = p.wrapping_add(6);
    }
}

/// C: `FUN_overlay_d_10__02158b6c`: conditional script lookup. The record `{u16 key, u32 rel offset}` with key `key`
/// points (record + 6 + offset) at a list of `{u16 var, u16 value, u16 script id}` (var 0 ends); the first entry whose
/// event var equals its value gives the id. 0xFFFF for no record, offset 0, or no match.
pub fn find_conditional<H: ScriptSysHooks>(h: &mut H, gd: u32, recs: u32, key: u32) -> u32 {
    let mut p = recs;
    let ofs = loop {
        let k = ru16(h, p);
        if k == 0 {
            return 0xFFFF;
        }
        if key == k {
            let b2 = h.r8(p.wrapping_add(2)) as u32;
            let b3 = h.r8(p.wrapping_add(3)) as u32;
            let b4 = h.r8(p.wrapping_add(4)) as u32;
            let b5 = h.r8(p.wrapping_add(5)) as u32;
            break (b5 << 24).wrapping_add((b4 << 16).wrapping_add(b2.wrapping_add(b3 << 8)));
        }
        p = p.wrapping_add(6);
    };
    if ofs == 0 {
        return 0xFFFF;
    }
    let mut q = p.wrapping_add(6).wrapping_add(ofs);
    let ew = h.fun_0201296c(gd);
    loop {
        let var = ru16(h, q);
        if var == 0 {
            return 0xFFFF;
        }
        let want = ru16(h, q.wrapping_add(2));
        let a = h.fun_020143cc(ew, var);
        if want == h.r16(a) as u32 {
            return ru16(h, q.wrapping_add(4));
        }
        q = q.wrapping_add(6);
    }
}

// ======================================================================== script_work.c: VM / ctx setup

/// C: `FUN_overlay_d_10__02158bfc`: make the VM of one script: ctx = `FUN_0215A55C({u16 zone, u16 id, mode, mode != 0,
/// sw}, heap)`; VM = `FUN_0201121C(heap, {0x100, 0x40, 0x0217062C, 609})` with that ctx (`FUN_0201126C`); the script's
/// file / message bank ([`script_lookup`]) is loaded into VM + 0x2C ([`load_script_file`]); with mode 0 the message bank
/// is opened (`FUN_0215A760(ctx, type, msg)`; the original's `msg != 0xFFFFFFFF` test compares a u16 and always passes);
/// the VM starts at the file data, PC += index * 4, PC += the u32 there (the file's offset table); the permission hook is
/// installed ([`install_hook`]). Returns the VM.
pub fn vm_new<H: ScriptSysHooks>(
    h: &mut H,
    heap: u32,
    sw: u32,
    zone: u32,
    id: u32,
    mode: u32,
) -> u32 {
    let has_mode = nonzero(mode);
    let mut ci = [0u8; 16];
    ci[0..2].copy_from_slice(&(zone as u16).to_le_bytes());
    ci[2..4].copy_from_slice(&(id as u16).to_le_bytes());
    ci[4..8].copy_from_slice(&mode.to_le_bytes());
    ci[8..12].copy_from_slice(&has_mode.to_le_bytes());
    ci[12..16].copy_from_slice(&sw.to_le_bytes());
    let ctx = h.fun_0215a55c(&ci, heap);
    let mut vi = [0u8; 12];
    vi[0..2].copy_from_slice(&VM_STACK_WORDS.to_le_bytes());
    vi[2..4].copy_from_slice(&VM_REG_WORDS.to_le_bytes());
    vi[4..8].copy_from_slice(&CMD_TABLE.to_le_bytes());
    vi[8..12].copy_from_slice(&CMD_COUNT.to_le_bytes());
    let vm = h.fun_0201121c(heap, &vi);
    h.fun_0201126c(vm, ctx);
    let (mut file, mut ty, mut msg) = (0u16, 0u16, 0u16);
    let idx = script_lookup(h, zone, id, &mut file, &mut ty, &mut msg);
    let data = load_script_file(h, file as u32, heap);
    h.w32(vm.wrapping_add(0x2C), data);
    if has_mode == 0 {
        // `ldrh` vs 0xFFFFFFFF: never equal
        h.fun_0215a760(ctx, ty as u32, msg as u32);
    }
    let data = h.r32(vm.wrapping_add(0x2C));
    h.fun_0201127c(vm, data);
    let pc = h.r32(vm.wrapping_add(vm::VM_PC));
    h.w32(vm.wrapping_add(vm::VM_PC), pc.wrapping_add(idx << 2));
    let ofs = h.fun_0201134c(vm);
    let pc = h.r32(vm.wrapping_add(vm::VM_PC));
    h.w32(vm.wrapping_add(vm::VM_PC), pc.wrapping_add(ofs));
    install_hook(h, vm);
    vm
}

/// C: `FUN_overlay_d_10__02158cb4`: free a script VM: its ctx (`FUN_0215A5A4`), its script file (VM + 0x2C) and the VM.
pub fn vm_free<H: ScriptSysHooks>(h: &mut H, vm: u32) {
    let ctx = h.fun_02011324(vm);
    h.fun_0215a5a4(ctx);
    let data = h.r32(vm.wrapping_add(0x2C));
    h.heap_free(data);
    h.heap_free(vm);
}

/// C: `FUN_overlay_d_10__02158cd0`: where script `id` lives: the first [`SCRIPT_RANGES`] entry with `lo <= id` gives
/// (file, type, msg) and the index `id - lo` (an id past that entry's `hi` hits the assertion `FUN_02034714`, which
/// returns in retail, so the entry is used anyway); ids `1..` below every range are zone-local (file / msg of the zone
/// header, type 3, index `id - 1`); id 0 = file 0x356, type 3, msg 0x9E, index 0. Returns the index (u16).
pub fn script_lookup<H: ScriptSysHooks>(
    h: &mut H,
    zone: u32,
    id: u32,
    file: &mut u16,
    ty: &mut u16,
    msg: &mut u16,
) -> u32 {
    for r in SCRIPT_RANGES.iter() {
        if id >= r.lo as u32 {
            if id > r.hi as u32 {
                h.fun_02034714(ASSERT_STR, 0, ASSERT_STR);
            }
            *file = r.file;
            *ty = r.ty;
            *msg = r.msg;
            return id.wrapping_sub(r.lo as u32) & 0xFFFF;
        }
    }
    if id >= 1 {
        let f = h.fun_02013b5c(zone & 0xFFFF);
        let m = h.fun_02013b68(zone & 0xFFFF);
        *file = f as u16;
        *ty = 3;
        *msg = m as u16;
        return id.wrapping_sub(1) & 0xFFFF;
    }
    *file = 0x356;
    *ty = 3;
    *msg = 0x9E;
    0
}

/// C: `FUN_overlay_d_10__02158d78`: load script file `file` of ARC 57 (`FUN_0204A1BC(57, file, 0, heap)`).
pub fn load_script_file<H: ScriptSysHooks>(h: &mut H, file: u32, heap: u32) -> u32 {
    h.fun_0204a1bc(ARC_SCRIPTS, file, 0, heap)
}

/// C: `FUN_02158D90`: 0 for id 2000, 1 for any other id below 0x291C, else 0.
pub fn is_low_script_id(id: u32) -> u32 {
    if id == 2000 {
        0
    } else {
        (id < 0x291C) as u32
    }
}

/// C: `FUN_overlay_d_10__02158db0`: `x != 0`.
pub fn nonzero(x: u32) -> u32 {
    (x != 0) as u32
}

/// C: `FUN_overlay_d_10__02158dbc`: fatal stop (a command not allowed in the ctx mode): if `FUN_0203CD98()`, run
/// `FUN_0203CF04`, `FUN_0203CEAC(0)`, `FUN_0203D0D0` and poll `FUN_0203CF5C` until `FUN_0203CDBC()`; then
/// `FUN_020053EC` forever.
pub fn fatal<H: ScriptSysHooks>(h: &mut H) -> ! {
    if h.fun_0203cd98() != 0 {
        h.fun_0203cf04();
        h.fun_0203ceac(0);
        h.fun_0203d0d0();
        loop {
            h.fun_0203cf5c();
            if h.fun_0203cdbc() != 0 {
                break;
            }
        }
    }
    loop {
        h.fun_020053ec();
    }
}

/// C: `sub_02158de8`: the VM pre-dispatch hook of scripts (`0x02158DE9`, argument GAMESYS): command `cmd` is allowed
/// when bit `mode` (ctx +8, 0..2) of [`CMD_PERMISSIONS`]`[cmd]` is set (mode >= 3: never); not allowed -> [`fatal`].
/// Returns 1. (`FUN_0201210C(gamesys)` and the ctx's +0xC are read and unused.)
pub fn dispatch_hook<H: ScriptSysHooks>(
    h: &mut H,
    _vm: u32,
    ctx: u32,
    gamesys: u32,
    cmd: u32,
) -> u32 {
    h.fun_0201210c(gamesys);
    ctx_mode_nonzero(h, ctx);
    let mode = ctx_mode(h, ctx);
    let perm = |c: u32| -> u32 {
        // UNVERIFIED for cmd >= 609: the original reads the bytes after the table
        *CMD_PERMISSIONS
            .get(c as usize)
            .expect("command id past the permission table") as u32
    };
    let ok = match mode {
        0 => perm(cmd) & 1,
        1 => (perm(cmd) >> 1) & 1,
        2 => (perm(cmd) >> 2) & 1,
        _ => 0,
    };
    if ok == 0 {
        fatal(h);
    }
    ok
}

/// C: `FUN_overlay_d_10__02158e38`: install [`dispatch_hook`] (`0x02158DE9`) with the ctx's GAMESYS as its argument.
pub fn install_hook<H: ScriptSysHooks>(h: &mut H, vm: u32) {
    let ctx = h.fun_02011324(vm);
    let gamesys = ctx_gamesys(h, ctx);
    h.fun_02011328(vm, DISPATCH_HOOK, gamesys);
}

/// C: `FUN_overlay_d_10__02158e54`: refresh the 8-byte field block at `dst`: `{FUN_02188C54(fm) or 0, fm}` with fm =
/// `FUN_0201210C(gamesys)`.
pub fn field_block_refresh<H: ScriptSysHooks>(h: &mut H, dst: u32, gamesys: u32) {
    let fm = h.fun_0201210c(gamesys);
    fill32(h, 0, dst, 8);
    h.w32(dst.wrapping_add(4), fm);
    if fm != 0 {
        let v = h.fun_02188c54(fm);
        h.w32(dst, v);
    }
}

/// C: `FUN_overlay_d_10__02158e7c`: create a SCRIPT_WORK (`Heap_AllocDebug(heap, 0x14C, 1, "script_work.c", 0xAF)`):
/// magic, u16 heap, GAMESYS, event, u16 script id, MMDL 0, `+0x20` = `a`, `+0x28` = `b`, `+0x24` = `b != 0` (no message
/// resources); else a WORDSET (`FUN_0201EC74(0x1C, 0x40, heap)`) and two 0x500 STRBUFs; the scrcmd work
/// (`FUN_0215A514`); the field block ([`field_block_refresh`]); with `b == 0` the init-proc mask is cleared. Returns SW.
pub fn sw_new<H: ScriptSysHooks>(
    h: &mut H,
    heap: u32,
    gamesys: u32,
    event: u32,
    id: u32,
    a: u32,
    b: u32,
) -> u32 {
    let sw = h.heap_alloc(heap, SW_SIZE, 1, FILE_SCRIPT_WORK, 0xAF);
    h.w32(sw, SW_MAGIC);
    h.w16(sw.wrapping_add(0xC), heap as u16);
    h.w32(sw.wrapping_add(0x10), gamesys);
    h.w32(sw.wrapping_add(0x14), event);
    h.w16(sw.wrapping_add(4), id as u16);
    h.w32(sw.wrapping_add(8), 0);
    h.w32(sw.wrapping_add(0x20), a);
    h.w32(sw.wrapping_add(0x28), b);
    let no_msg = nonzero(b);
    h.w32(sw.wrapping_add(0x24), no_msg);
    if no_msg == 0 {
        let ws = h.fun_0201ec74(0x1C, 0x40, heap);
        h.w32(sw.wrapping_add(0x2C), ws);
        let s1 = h.fun_020457b0(0x500, heap);
        h.w32(sw.wrapping_add(0x30), s1);
        let s2 = h.fun_020457b0(0x500, heap);
        h.w32(sw.wrapping_add(0x34), s2);
    } else {
        h.w32(sw.wrapping_add(0x2C), 0);
        h.w32(sw.wrapping_add(0x30), 0);
        h.w32(sw.wrapping_add(0x34), 0);
    }
    let work = h.fun_0215a514(sw, heap);
    h.w32(sw.wrapping_add(SW_SCRCMD_WORK), work);
    field_block_refresh(h, sw.wrapping_add(0x18), gamesys);
    if h.r32(sw.wrapping_add(0x24)) == 0 {
        proc_mask_reset(h);
    }
    sw
}

/// C: `FUN_overlay_d_10__02158f18`: free a SCRIPT_WORK: magic = 0, the wordset and the two strbufs when set, the scrcmd
/// work (`Heap_Free` through the veneer `0x0215A554`) and SW.
pub fn sw_free<H: ScriptSysHooks>(h: &mut H, sw: u32) {
    h.w32(sw, 0);
    let ws = h.r32(sw.wrapping_add(0x2C));
    if ws != 0 {
        h.fun_0201ed04(ws);
    }
    let s1 = h.r32(sw.wrapping_add(0x30));
    if s1 != 0 {
        h.fun_02045808(s1);
    }
    let s2 = h.r32(sw.wrapping_add(0x34));
    if s2 != 0 {
        h.fun_02045808(s2);
    }
    let work = h.r32(sw.wrapping_add(SW_SCRCMD_WORK));
    h.heap_free(work);
    h.heap_free(sw);
}

// ---------------------------------------------------------------- SW accessors

/// C: `FUN_02158F50`: the GMEVENT (+0x14).
pub fn sw_event<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(0x14))
}

/// C: `FUN_02158F54`: the GAMESYS (+0x10).
pub fn sw_gamesys<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(0x10))
}

/// C: `FUN_overlay_d_10__02158f58`: refresh the field block (+0x18) and return its address.
pub fn sw_field_block<H: ScriptSysHooks>(h: &mut H, sw: u32) -> u32 {
    let gamesys = h.r32(sw.wrapping_add(0x10));
    field_block_refresh(h, sw.wrapping_add(0x18), gamesys);
    sw.wrapping_add(0x18)
}

/// C: `FUN_overlay_d_10__02158f6c`: the scrcmd work (+0x148).
pub fn sw_scrcmd_work<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(SW_SCRCMD_WORK))
}

/// C: `FUN_02158F74`: the WORDSET (+0x2C).
pub fn sw_wordset<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(0x2C))
}

/// C: `FUN_02158F78`: STRBUF 1 (+0x30).
pub fn sw_strbuf1<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(0x30))
}

/// C: `FUN_02158F7C`: STRBUF 2 (+0x34).
pub fn sw_strbuf2<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(0x34))
}

/// C: `FUN_02158F80`: +0x38 = `v`.
pub fn sw_set_38<H: MemEnv>(h: &mut H, sw: u32, v: u32) {
    h.w32(sw.wrapping_add(0x38), v);
}

/// C: `FUN_02158F84`: +0x38.
pub fn sw_get_38<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(0x38))
}

/// C: `FUN_02158f88`: address SW + 0x40.
pub fn sw_addr_40(sw: u32) -> u32 {
    sw.wrapping_add(0x40)
}

/// C: `FUN_02158f8c`: the script id (u16 +4).
pub fn sw_script_id<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r16(sw.wrapping_add(4)) as u32
}

/// C: `FUN_02158f90`: the MMDL (+8).
pub fn sw_mmdl<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(8))
}

/// C: `FUN_overlay_d_10__02158f94`: MMDL (+8) = `mmdl`; a nonzero one also stores its OBJ id (`FUN_0216D588`) into script
/// var 0x8011.
pub fn sw_set_mmdl<H: ScriptSysHooks>(h: &mut H, sw: u32, mmdl: u32) {
    h.w32(sw.wrapping_add(8), mmdl);
    if mmdl != 0 {
        let a = sw_var_addr(sw, 0x8011);
        let id = h.fun_0216d588(mmdl);
        h.w16(a, id as u16);
    }
}

/// C: `FUN_02158FB4`: address SW + 0x3C.
pub fn sw_addr_3c(sw: u32) -> u32 {
    sw.wrapping_add(0x3C)
}

/// C: `FUN_overlay_d_10__02158fb8`: the heap block at +0x3C.
pub fn sw_get_3c<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(0x3C))
}

/// C: `FUN_overlay_d_10__02158fbc`: free the heap block at +0x3C (if any) and clear it.
pub fn sw_free_3c<H: ScriptSysHooks>(h: &mut H, sw: u32) {
    let p = h.r32(sw.wrapping_add(0x3C));
    if p != 0 {
        h.heap_free(p);
        h.w32(sw.wrapping_add(0x3C), 0);
    }
}

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

/// C: `FUN_overlay_d_10__02159030`: store `value` (u16) into var `var` (GAMEDATA of SW's GAMESYS); 1, or 0 when the var
/// has no address.
pub fn set_var<H: ScriptSysHooks>(h: &mut H, sw: u32, var: u32, value: u32) -> u32 {
    let gamesys = h.r32(sw.wrapping_add(0x10));
    let gd = h.fun_020120f4(gamesys);
    let a = var_addr(h, sw, gd, var);
    if a == 0 {
        return 0;
    }
    h.w16(a, value as u16);
    1
}

/// C: `FUN_02159058`: script vars 0x8000..=0x8003 = `a`, `b`, `c`, `d` ([`set_var`]).
pub fn set_vars_8000<H: ScriptSysHooks>(h: &mut H, sw: u32, a: u32, b: u32, c: u32, d: u32) {
    set_var(h, sw, SW_VAR_BASE, a);
    set_var(h, sw, SW_VAR_BASE + 1, b);
    set_var(h, sw, SW_VAR_BASE + 2, c);
    set_var(h, sw, SW_VAR_BASE + 3, d & 0xFFFF);
}

/// C: `FUN_overlay_d_10__0215909c`: a heap copy (`Heap_AllocDebug(SW heap, 0x7E, 1, "script_work.c", 0x21B)`) of the
/// script vars 0x8020..0x805E; returns it.
pub fn save_vars<H: ScriptSysHooks>(h: &mut H, sw: u32) -> u32 {
    let heap = h.r16(sw.wrapping_add(0xC)) as u32;
    let buf = h.heap_alloc(heap, 0x7E, 1, FILE_SCRIPT_WORK, 0x21B);
    let src = sw_var_addr(sw, 0x8020);
    copy16(h, src, buf, 0x7E);
    buf
}

/// C: `FUN_overlay_d_10__021590d4`: restore the script vars 0x8020..0x805E from `buf` ([`save_vars`]) and free it.
pub fn restore_vars<H: ScriptSysHooks>(h: &mut H, sw: u32, buf: u32) {
    let dst = sw_var_addr(sw, 0x8020);
    copy16(h, buf, dst, 0x7E);
    h.heap_free(buf);
}

/// C: `FUN_overlay_d_10__021590f4`: +0x13C = `v`.
pub fn sw_set_13c<H: MemEnv>(h: &mut H, sw: u32, v: u32) {
    h.w32(sw.wrapping_add(0x13C), v);
}

/// C: `FUN_overlay_d_10__021590fc`: +0x13C.
pub fn sw_get_13c<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(0x13C))
}

/// C: `FUN_02159104`: +0x140 = `v`.
pub fn sw_set_140<H: MemEnv>(h: &mut H, sw: u32, v: u32) {
    h.w32(sw.wrapping_add(0x140), v);
}

/// C: `FUN_0215910C`: +0x140.
pub fn sw_get_140<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(0x140))
}

/// C: `FUN_overlay_d_10__02159114`: +0x144 = `v`.
pub fn sw_set_144<H: MemEnv>(h: &mut H, sw: u32, v: u32) {
    h.w32(sw.wrapping_add(0x144), v);
}

/// C: `FUN_overlay_d_10__0215911c`: +0x144.
pub fn sw_get_144<H: MemEnv>(h: &mut H, sw: u32) -> u32 {
    h.r32(sw.wrapping_add(0x144))
}

/// C: `FUN_02159124`: address of the `i`-th 0x1C-byte record at SW + 0x44 (no range check).
pub fn sw_record_addr(sw: u32, i: u32) -> u32 {
    sw.wrapping_add(0x44).wrapping_add(i.wrapping_mul(0x1C))
}

// ---------------------------------------------------------------- init-proc mask (u32 at 0x02172BA0, bits 0..13)

/// C: `FUN_overlay_d_10__02159130`: clear the init-proc mask.
pub fn proc_mask_reset<H: MemEnv>(h: &mut H) {
    fill32(h, 0, PROC_MASK_ADDR, 4);
}

/// C: `FUN_02159144`: set bit `bit` of the init-proc mask when `bit < 14` (signed) and it is not set yet.
pub fn proc_mask_set<H: MemEnv>(h: &mut H, bit: u32) {
    if (bit as i32) < PROC_MASK_BITS && proc_mask_test(h, bit) == 0 {
        let m = h.r32(PROC_MASK_ADDR);
        h.w32(PROC_MASK_ADDR, lsl_reg(1, bit) | m);
    }
}

/// C: `FUN_02159168`: clear bit `bit` of the init-proc mask when `bit < 14` (signed) and it is set.
pub fn proc_mask_clear<H: MemEnv>(h: &mut H, bit: u32) {
    if (bit as i32) < PROC_MASK_BITS && proc_mask_test(h, bit) != 0 {
        let m = h.r32(PROC_MASK_ADDR);
        h.w32(PROC_MASK_ADDR, !lsl_reg(1, bit) & m);
    }
}

/// C: `FUN_overlay_d_10__02159198`: bit `bit` of the init-proc mask; 0 for `bit >= 14` (signed). A negative bit shifts
/// by its low byte (`asrs`), as the original does.
pub fn proc_mask_test<H: MemEnv>(h: &mut H, bit: u32) -> u32 {
    if bit as i32 >= PROC_MASK_BITS {
        return 0;
    }
    let mask = h.r32(PROC_MASK_ADDR);
    asr_reg(mask, bit) & 1
}

/// C: `sub_021591b4`: run the init procs ([`INIT_PROCS`]) from the index at `task + 0x10` on: a 0 entry ends (1); a proc
/// whose mask bit is set is called as `proc(task, task + 0xC)` and a 0 result returns 0 (call again next time); after
/// each entry the index grows, `task + 0xC` = 0, and index >= `task + 0x14` (signed) ends (1). `_a` / `_b` are not read.
pub fn run_init_procs<H: ScriptSysHooks>(h: &mut H, _a: u32, _b: u32, task: u32) -> u32 {
    loop {
        let i = h.r32(task.wrapping_add(0x10));
        // UNVERIFIED for an index past the table: the original reads the words after it
        let f = *INIT_PROCS
            .get(i as usize)
            .expect("init-proc index past the table");
        if f == 0 {
            return 1;
        }
        if proc_mask_test(h, i) != 0 && h.call_init_proc(f, task, task.wrapping_add(0xC)) == 0 {
            return 0;
        }
        let i = h.r32(task.wrapping_add(0x10));
        h.w32(task.wrapping_add(0x10), i.wrapping_add(1));
        h.w32(task.wrapping_add(0xC), 0);
        let i = h.r32(task.wrapping_add(0x10)) as i32;
        let n = h.r32(task.wrapping_add(0x14)) as i32;
        if i >= n {
            return 1;
        }
    }
}

// ======================================================================== script_work.c tail: commands 49, 50, 311

/// C: `sub_02159960`: wait callback of command 49: 1 (done) once `FUN_020362DC() & 3` is nonzero (pad bits 0-1).
pub fn wait_pad_a<H: ScriptSysHooks>(h: &mut H, _vm: u32, _ctx: u32) -> u32 {
    (h.fun_020362dc() & 3 != 0) as u32
}

/// C: `sub_02159974`: command 49: wait on [`wait_pad_a`] (`0x02159961`) and yield.
pub fn cmd_wait_pad_a<H: ScriptSysHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113d0(vm, WAIT_PAD_A);
    1
}

/// C: `sub_02159984`: wait callback of command 50: done once `FUN_020362DC() & 0xF3` is nonzero.
pub fn wait_pad_b<H: ScriptSysHooks>(h: &mut H, _vm: u32, _ctx: u32) -> u32 {
    (h.fun_020362dc() & 0xF3 != 0) as u32
}

/// C: `sub_02159998`: command 50: wait on [`wait_pad_b`] (`0x02159985`) and yield.
pub fn cmd_wait_pad_b<H: ScriptSysHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113d0(vm, WAIT_PAD_B);
    1
}

/// Result of [`wait_311`] for a status above 3: the original returns its caller's r4 there (the register is never set
/// on that path). The differential test enters with r4 = 0; in the game it is the VM step's caller's r4. UNVERIFIED
/// (§8): whether `FUN_02012DD0` ever returns more than 3.
pub const WAIT_311_STALE_R4: u32 = 0;

/// C: `sub_021599a8`: wait callback of command 311. fm = `FUN_02188CCC(FUN_0201210C(GAMESYS))`. Without
/// `FUN_02012E40(GAMEDATA)`: done (1) unless `FUN_021A0F90(fm) == 7` and `FUN_021A153C(fm) == 0`. With it, by the
/// status `FUN_02012DD0(GAMEDATA)`: 0/1 -> 0; 2 -> the var operand ([`cmd_var_addr`]) = 1, 0; 3 -> the var = 0,
/// `FUN_021A1548(fm)` when `FUN_021A0F90(fm) == 7`, 1; above 3 -> [`WAIT_311_STALE_R4`]. When done (1): the object at SW
/// + 0x13C is released (`FUN_021D11D8`, `FUN_021D1194`) and cleared.
pub fn wait_311<H: ScriptSysHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = ctx_gamedata(h, ctx);
    let sw = ctx_script_work(h, ctx);
    let gamesys = ctx_gamesys(h, ctx);
    let fm0 = h.fun_0201210c(gamesys);
    let fm = h.fun_02188ccc(fm0);
    let r = if h.fun_02012e40(gd) == 0 {
        if h.fun_021a0f90(fm) == 7 && h.fun_021a153c(fm) == 0 {
            0
        } else {
            1
        }
    } else {
        match h.fun_02012dd0(gd) {
            0 | 1 => 0,
            2 => {
                let a = cmd_var_addr(h, vm, ctx);
                h.w16(a, 1);
                0
            }
            3 => {
                let a = cmd_var_addr(h, vm, ctx);
                h.w16(a, 0);
                if h.fun_021a0f90(fm) == 7 {
                    h.fun_021a1548(fm);
                }
                1
            }
            _ => WAIT_311_STALE_R4,
        }
    };
    if r == 1 {
        let x = sw_get_13c(h, sw);
        h.fun_021d11d8(x);
        h.fun_021d1194(x);
        sw_set_13c(h, sw, 0);
    }
    r
}

/// C: `sub_02159a64`: command 311: x = `FUN_021D1164(FUN_02188C84(fm), fm)` (fm = the field map), `FUN_021D11B0(x)`;
/// with `FUN_021A0F90(FUN_02188CCC(fm)) == 7` also `FUN_021A1524` / `FUN_021A1530` on it; SW + 0x13C = x;
/// `FUN_02008DF0(FUN_02012F2C(GAMEDATA), 1)`, `FUN_02012DAC(GAMEDATA)`; wait on [`wait_311`] (`0x021599A9`) and yield.
pub fn cmd_311<H: ScriptSysHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let sw = ctx_script_work(h, ctx);
    let gd = ctx_gamedata(h, ctx);
    let gamesys = ctx_gamesys(h, ctx);
    let fm = h.fun_0201210c(gamesys);
    let a = h.fun_02188c84(fm);
    let b = h.fun_02188ccc(fm);
    let x = h.fun_021d1164(a, fm);
    h.fun_021d11b0(x);
    if h.fun_021a0f90(b) == 7 {
        h.fun_021a1524(b);
        h.fun_021a1530(b);
    }
    sw_set_13c(h, sw, x);
    let o = h.fun_02012f2c(gd);
    h.fun_02008df0(o, 1);
    h.fun_02012dac(gd);
    h.fun_020113d0(vm, WAIT_311);
    1
}

/// C: `FUN_02159AE8`: address of the var whose id is the next u16 of the running script
/// (`var_addr(SW, GAMEDATA, FUN_02011330(vm))`); the command handlers store their results through it.
pub fn cmd_var_addr<H: ScriptSysHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gamedata = ctx_gamedata(h, ctx);
    let sw = ctx_script_work(h, ctx);
    let var = h.fun_02011330(vm);
    var_addr(h, sw, gamedata, var)
}

/// C: `FUN_02159B10`: the next u16 of the running script read as a var or a literal ([`var_value`]).
pub fn cmd_var_value<H: ScriptSysHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gamedata = ctx_gamedata(h, ctx);
    let sw = ctx_script_work(h, ctx);
    let var = h.fun_02011330(vm);
    var_value(h, sw, gamedata, var)
}

/// C: `FUN_overlay_d_10__02159b38`: the zone id of the ctx's GAMEDATA (`FUN_02012AC8(FUN_020127A4(GAMEDATA))`).
pub fn ctx_zone_id<H: ScriptSysHooks>(h: &mut H, ctx: u32) -> u32 {
    let gd = ctx_gamedata(h, ctx);
    let blk = h.fun_020127a4(gd);
    h.fun_02012ac8(blk)
}

// ---------------------------------------------------------------- scrcmd ctx getters (scrcmd_work.c, F1 hubs)

/// C: `FUN_0215A5CC`: the ctx's heap id (u16 +0).
pub fn ctx_heap<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    h.r16(ctx) as u32
}

/// C: `FUN_0215A5D0`: the ctx's zone id (u16 +4).
pub fn ctx_zone<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    h.r16(ctx.wrapping_add(4)) as u32
}

/// C: `FUN_overlay_d_10__0215a5d4`: the ctx's `mode != 0` flag (+0xC).
pub fn ctx_mode_nonzero<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    h.r32(ctx.wrapping_add(0xC))
}

/// C: `FUN_overlay_d_10__0215a5d8`: the ctx's mode (+8; selects the command permission bit).
pub fn ctx_mode<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    h.r32(ctx.wrapping_add(8))
}

/// C: `FUN_0215A5DC`: the GAMESYS of a scrcmd ctx (word 1 of the scrcmd work at `ctx + 0x20`).
pub fn ctx_gamesys<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let work = h.r32(ctx.wrapping_add(0x20));
    h.r32(work.wrapping_add(4))
}

/// C: `FUN_0215A5E4`: the GAMEDATA of a scrcmd ctx (word 2 of the scrcmd work).
pub fn ctx_gamedata<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let work = h.r32(ctx.wrapping_add(0x20));
    h.r32(work.wrapping_add(8))
}

/// C: `FUN_0215A5EC`: word 3 of the scrcmd work.
pub fn ctx_work_3<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let work = h.r32(ctx.wrapping_add(0x20));
    h.r32(work.wrapping_add(0xC))
}

/// C: `FUN_overlay_d_10__0215a5f4`: the SCRIPT_WORK of a scrcmd ctx (word 0 of the scrcmd work at `ctx + 0x20`).
pub fn ctx_script_work<H: MemEnv>(h: &mut H, ctx: u32) -> u32 {
    let work = h.r32(ctx.wrapping_add(0x20));
    h.r32(work)
}

// ---------------------------------------------------------------- dispatch helpers for the game side

/// The function pointers of this area that the VM calls back (`VmHooks::call_wait` / `call_hook`,
/// `CommandDispatch::command`), for an implementation that routes addresses to ports: `Some(result)` when `f` is one
/// of them. `args` are the callback's arguments in order (`vm, ctx` for waits and commands; `vm, ctx, arg, cmd` for the
/// hook).
pub fn call_known<H: ScriptSysHooks>(h: &mut H, f: u32, args: &[u32]) -> Option<u32> {
    let a = |i: usize| args.get(i).copied().unwrap_or(0);
    Some(match f {
        WAIT_PAD_A => wait_pad_a(h, a(0), a(1)),
        WAIT_PAD_B => wait_pad_b(h, a(0), a(1)),
        WAIT_311 => wait_311(h, a(0), a(1)),
        CMD_WAIT_PAD_A => cmd_wait_pad_a(h, a(0), a(1)),
        CMD_WAIT_PAD_B => cmd_wait_pad_b(h, a(0), a(1)),
        CMD_311 => cmd_311(h, a(0), a(1)),
        DISPATCH_HOOK => dispatch_hook(h, a(0), a(1), a(2), a(3)),
        EVENT_FN => event_fn(h, a(0), a(1), a(2)),
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
