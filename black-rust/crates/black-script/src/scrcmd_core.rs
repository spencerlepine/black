//! F8: the scrcmd core, overlay 10 0x02159204-0x0215995E: script commands 0..48 (VM control, the expression stack, VM
//! regs, compares and conditional jumps, var arithmetic, event flags, sub-scripts) and their helpers.
//!
//! Every function of the area (53) is specced in `tools/rom-diff/specs/black-script/scrcmd_core.spec`; its header has the
//! command-id table (id -> handler) and the VM fields the commands touch. All 53 are ported here and listed in
//! [`PORTED`] (checked with `tools/check-fn.sh black-script scrcmd_core`). [`COMMANDS`] maps the command ids to the
//! handler addresses and [`dispatch`] runs a handler by its table address (the F7 `CommandDispatch` seam).
//!
//! Commands are `fn(vm, ctx) -> 0 (next command) / 1 (yield)` (`vm::step` calls `table[cmd](vm, ctx)` through
//! [`CommandDispatch`](crate::hooks::CommandDispatch)). Pointers are `u32` addresses in the [`Mem`](crate::hooks::Mem)
//! window; the callees (vm.c, script_sys, scrcmd_work, main, overlay 21) are [`CoreHooks`] methods named by address.

use crate::hooks::MemEnv;
use crate::vm::{VM_0E, VM_PC, VM_REGS};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_9204,
    0x0215_9240,
    0x0215_9244,
    0x0215_9248,
    0x0215_9254,
    0x0215_9260,
    0x0215_9284,
    0x0215_929C,
    0x0215_92A8,
    0x0215_92C0,
    0x0215_92D8,
    0x0215_92F0,
    0x0215_930C,
    0x0215_932C,
    0x0215_9340,
    0x0215_9370,
    0x0215_9398,
    0x0215_93B8,
    0x0215_93DC,
    0x0215_93F0,
    0x0215_9404,
    0x0215_9420,
    0x0215_942C,
    0x0215_944C,
    0x0215_946C,
    0x0215_948C,
    0x0215_94B0,
    0x0215_94E0,
    0x0215_9570,
    0x0215_95A0,
    0x0215_95CC,
    0x0215_9644,
    0x0215_9658,
    0x0215_9670,
    0x0215_96B4,
    0x0215_96E8,
    0x0215_9708,
    0x0215_9728,
    0x0215_9750,
    0x0215_9778,
    0x0215_97AC,
    0x0215_97CC,
    0x0215_97EC,
    0x0215_980C,
    0x0215_9830,
    0x0215_9854,
    0x0215_9874,
    0x0215_9894,
    0x0215_98AC,
    0x0215_98C8,
    0x0215_98E4,
    0x0215_98EC,
    0x0215_9934,
];

/// Command id -> handler address (Thumb entry, bit 0 clear) of the commands of this area (the words of the command table
/// 0x0217062C at index id; checked against the ROM by the tests).
pub const COMMANDS: &[(u16, u32)] = &[
    (0, 0x0215_9240),
    (1, 0x0215_9244),
    (2, 0x0215_9248),
    (3, 0x0215_9260),
    (4, 0x0215_9284),
    (5, 0x0215_929C),
    (6, 0x0215_92A8),
    (7, 0x0215_92C0),
    (8, 0x0215_93DC),
    (9, 0x0215_93F0),
    (10, 0x0215_9404),
    (11, 0x0215_9420),
    (12, 0x0215_942C),
    (13, 0x0215_944C),
    (14, 0x0215_946C),
    (15, 0x0215_948C),
    (16, 0x0215_94B0),
    (17, 0x0215_94E0),
    (18, 0x0215_9854),
    (19, 0x0215_9874),
    (20, 0x0215_92D8),
    (21, 0x0215_92F0),
    (22, 0x0215_930C),
    (23, 0x0215_9340),
    (24, 0x0215_9370),
    (25, 0x0215_9398),
    (26, 0x0215_93B8),
    (27, 0x0215_9570),
    (28, 0x0215_95CC),
    (29, 0x0215_9644),
    (30, 0x0215_9658),
    (31, 0x0215_9670),
    (32, 0x0215_96B4),
    (33, 0x0215_96E8),
    (34, 0x0215_9708),
    (35, 0x0215_9728),
    (36, 0x0215_9750),
    (37, 0x0215_9778),
    (38, 0x0215_97AC),
    (39, 0x0215_97CC),
    (40, 0x0215_9894),
    (41, 0x0215_98AC),
    (42, 0x0215_98C8),
    (43, 0x0215_97EC),
    (44, 0x0215_980C),
    (45, 0x0215_9830),
    (46, 0x0215_98E4),
    (47, 0x0215_98EC),
    (48, 0x0215_9934),
];

/// The condition table of commands 31 / 32 (overlay 10 data at 0x021705DC, 6 conditions x 3 bytes): `COND[c * 3 + r]` is 1
/// when condition `c` holds for the compare result `r` (0 less, 1 equal, 2 greater; VM +0xE). Conditions: 0 `<`, 1 `==`,
/// 2 `>`, 3 `<=`, 4 `>=`, 5 `!=`.
pub const COND: [u8; 18] = [1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1];
/// ROM address of [`COND`].
pub const COND_ADDR: u32 = 0x0217_05DC;

/// The wait callback command 3 installs (`0x02159255` = [`wait_countdown`]).
pub const WAIT_COUNTDOWN: u32 = 0x0215_9255;
/// The wait callback command 28 installs (`0x021595A1` = [`wait_call_script`]).
pub const WAIT_CALL_SCRIPT: u32 = 0x0215_95A1;
/// The event function of the init-proc event of command 48 (`0x021591B5` = `script_sys::run_init_procs`).
pub const INIT_PROCS_EVENT_FN: u32 = 0x0215_91B5;

/// Callees of the scrcmd core commands that this area does not port, named by address (meaning per the spec). The vm.c
/// routines, the scrcmd ctx getters and the `scrcmd_work` helpers that need nothing but memory default to their ports
/// (each area is verified on its own, so the `scrcmd_core` test overrides them with the spec's stubs).
pub trait CoreHooks: MemEnv {
    // ---- vm.c (main), ported in crate::vm
    /// C: `FUN_02011290` (vm.c `stop`): end the script.
    fn fun_02011290(&mut self, vm: u32)
    where
        Self: Sized,
    {
        crate::vm::stop(self, vm)
    }
    /// C: `FUN_02011330` (vm.c `read_u16`): the next u16 operand of the script; PC += 2.
    fn fun_02011330(&mut self, vm: u32) -> u32
    where
        Self: Sized,
    {
        crate::vm::read_u16(self, vm)
    }
    /// C: `FUN_0201134C` (vm.c `read_u32`): the next u32 operand of the script; PC += 4.
    fn fun_0201134c(&mut self, vm: u32) -> u32
    where
        Self: Sized,
    {
        crate::vm::read_u32(self, vm)
    }
    /// C: `FUN_0201137C` (vm.c `push`): push `v` on the call / expression stack.
    fn fun_0201137c(&mut self, vm: u32, v: u32)
    where
        Self: Sized,
    {
        crate::vm::push(self, vm, v)
    }
    /// C: unlabeled routine at `0x02011398` (vm.c `pop`): pop the call / expression stack (0 when empty).
    fn fun_02011398(&mut self, vm: u32) -> u32
    where
        Self: Sized,
    {
        crate::vm::pop(self, vm)
    }
    /// C: `FUN_020113B0` (vm.c `jump`): PC = `pc`.
    fn fun_020113b0(&mut self, vm: u32, pc: u32)
    where
        Self: Sized,
    {
        crate::vm::jump(self, vm, pc)
    }
    /// C: `FUN_020113B4` (vm.c `call`): push PC, PC = `pc`.
    fn fun_020113b4(&mut self, vm: u32, pc: u32)
    where
        Self: Sized,
    {
        crate::vm::call(self, vm, pc)
    }
    /// C: `FUN_020113c4` (vm.c `ret`): PC = pop.
    fn fun_020113c4(&mut self, vm: u32)
    where
        Self: Sized,
    {
        crate::vm::ret(self, vm)
    }
    /// C: `FUN_020113D0` (vm.c `wait`): wait on the callback `f`.
    fn fun_020113d0(&mut self, vm: u32, f: u32)
    where
        Self: Sized,
    {
        crate::vm::wait(self, vm, f)
    }

    // ---- scrcmd ctx getters (ported in crate::script_sys)
    /// C: `FUN_overlay_d_10__0215a5d4` (`script_sys::ctx_mode_nonzero`): the ctx's `mode != 0` flag.
    fn fun_0215a5d4(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::ctx_mode_nonzero(self, ctx)
    }
    /// C: `FUN_overlay_d_10__0215a5d8` (`script_sys::ctx_mode`): the ctx's mode.
    fn fun_0215a5d8(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::ctx_mode(self, ctx)
    }
    /// C: `FUN_0215A5DC` (`script_sys::ctx_gamesys`): the GAMESYS of the ctx.
    fn fun_0215a5dc(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::ctx_gamesys(self, ctx)
    }
    /// C: `FUN_0215A5E4` (`script_sys::ctx_gamedata`): the GAMEDATA of the ctx.
    fn fun_0215a5e4(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::ctx_gamedata(self, ctx)
    }
    /// C: `FUN_overlay_d_10__0215a5f4` (`script_sys::ctx_script_work`): the SCRIPT_WORK of the ctx.
    fn fun_0215a5f4(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::script_sys::ctx_script_work(self, ctx)
    }

    // ---- script_sys (overlay 10)
    /// C: `FUN_021587E0(sw, slot)`: the VM in slot `slot` of the script's SYS is still running (nonzero) (UNVERIFIED
    /// meaning; outside the script_sys area).
    fn fun_021587e0(&mut self, sw: u32, slot: u32) -> u32;
    /// C: `FUN_0215897C` (`script_sys::sw_call_child`): call the child event `child` from the script's event.
    fn fun_0215897c(&mut self, sw: u32, child: u32);
    /// C: `FUN_overlay_d_10__021589a0` (`script_sys::sw_start_script`): start script `id` in another VM; returns the slot.
    fn fun_021589a0(&mut self, sw: u32, zone: u32, id: u32) -> u32;
    /// C: `FUN_overlay_d_10__02158a34` (`script_sys::run_sync`): run script `id` to the end at once.
    fn fun_02158a34(&mut self, gamesys: u32, sw: u32, id: u32, mode: u32);
    /// C: `FUN_02159AE8` (script_sys `cmd_var_addr`): address of the var whose id is the next script word.
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_02159B10` (script_sys `cmd_var_value`): the next script word read as a var id or a literal.
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_overlay_d_10__02159b38` (`script_sys::ctx_zone_id`): the current zone id.
    fn fun_02159b38(&mut self, ctx: u32) -> u32;

    // ---- scrcmd_work (overlay 10)
    /// C: `FUN_0215A684` (`scrcmd_work::set_countdown`): the frame countdown = `n`.
    fn fun_0215a684(&mut self, ctx: u32, n: u32)
    where
        Self: Sized,
    {
        crate::scrcmd_work::set_countdown(self, ctx, n)
    }
    /// C: `FUN_0215A690` (`scrcmd_work::countdown`): one frame of the countdown; 1 = done.
    fn fun_0215a690(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::scrcmd_work::countdown(self, ctx)
    }
    /// C: `FUN_0215AA24` (`scrcmd_work::save_vars`): save the script vars into the ctx (+0x1C).
    fn fun_0215aa24(&mut self, ctx: u32);
    /// C: `FUN_0215AA34` (`scrcmd_work::restore_vars`): restore them and free the copy.
    fn fun_0215aa34(&mut self, ctx: u32);
    /// C: `FUN_0215AA48` (`scrcmd_work::set_wait_slot`): ctx byte +0x1A = `slot`.
    fn fun_0215aa48(&mut self, ctx: u32, slot: u32)
    where
        Self: Sized,
    {
        crate::scrcmd_work::set_wait_slot(self, ctx, slot)
    }
    /// C: `FUN_0215AA4C` (`scrcmd_work::wait_slot`): ctx byte +0x1A.
    fn fun_0215aa4c(&mut self, ctx: u32) -> u32
    where
        Self: Sized,
    {
        crate::scrcmd_work::wait_slot(self, ctx)
    }

    // ---- main
    /// C: `FUN_020120F4(gamesys)`: the GAMEDATA of the game system.
    fn fun_020120f4(&mut self, gamesys: u32) -> u32;
    /// C: `FUN_020122C0(gamesys, parent, fn, work size)`: create a GMEVENT.
    fn fun_020122c0(&mut self, gamesys: u32, parent: u32, f: u32, size: u32) -> u32;
    /// C: `FUN_020124E8(event)`: the work block of a GMEVENT.
    fn fun_020124e8(&mut self, event: u32) -> u32;
    /// C: `FUN_0201296C(gamedata)`: the EVENT WORK of the game data (`black_field::game_data`).
    fn fun_0201296c(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012A30(gamedata)`: a GAMEDATA sub record (command 47; UNVERIFIED meaning).
    fn fun_02012a30(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_02012B38(gamedata)`: a GAMEDATA sub record (commands 33 / 34; UNVERIFIED meaning).
    fn fun_02012b38(&mut self, gamedata: u32) -> u32;
    /// C: `FUN_020142E8(ew, flag)`: event flag `flag` is set (`black_field::event_work`).
    fn fun_020142e8(&mut self, ew: u32, flag: u32) -> u32;
    /// C: `FUN_02014314(ew, flag)`: set event flag `flag`.
    fn fun_02014314(&mut self, ew: u32, flag: u32);
    /// C: `FUN_0201433C(ew, flag)`: clear event flag `flag`.
    fn fun_0201433c(&mut self, ew: u32, flag: u32);
    /// C: `FUN_02014460(x)`: the u16 value command 34 reads from the `FUN_02012B38` record (UNVERIFIED meaning).
    fn fun_02014460(&mut self, x: u32) -> u32;
    /// C: `FUN_02014474(x, v)`: set that value (command 33; UNVERIFIED meaning).
    fn fun_02014474(&mut self, x: u32, v: u32);
    /// C: `FUN_020288C4(x)`: the condition of command 47 (UNVERIFIED meaning).
    fn fun_020288c4(&mut self, x: u32) -> u32;
    /// C: `FUN_0202972C(gamesys, 0)`: the child event command 47 runs (UNVERIFIED meaning).
    fn fun_0202972c(&mut self, gamesys: u32, a: u32) -> u32;

    // ---- overlay 21
    /// C: `FUN_021B1BB8(vm, ctx)`: the body of command 46 (overlay 21 scrcmd code, another F8 job).
    fn fun_021b1bb8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_021B1BE8(ctx)`: the first step of command 47 (overlay 21, another F8 job).
    fn fun_021b1be8(&mut self, ctx: u32);
}

/// The next script byte (`ldrb [PC]; PC += 1`, inlined by the commands that take byte operands).
fn read_u8<H: MemEnv>(h: &mut H, vm: u32) -> u8 {
    let pc = h.r32(vm + VM_PC);
    let b = h.r8(pc);
    h.w32(vm + VM_PC, pc.wrapping_add(1));
    b
}

/// Address of VM reg `i` (u32 array at `*(vm + 0x1C)`).
fn reg_addr<H: MemEnv>(h: &mut H, vm: u32, i: u8) -> u32 {
    h.r32(vm + VM_REGS).wrapping_add(4 * i as u32)
}

/// `_u32_div_f` (`0x0209C2B0`, real code in the original): `(n / d, n % d)`; division by zero gives `(n, 0)` (as
/// `black_battle::calc::u32_divmod`).
///
/// The shared library-division helpers of black-script ([`u32_divmod`], [`u32_div`], [`s32_div`]): the ARM9 library
/// never traps, a zero divisor returns the dividend (`_u32_div_f`: `cmp r1, 0; bxeq lr`; `_s32_div_f` 0x0209C0A4: skips
/// the loop and restores the sign). Checked under Unicorn on main.sbin (V-0026 / V-0027): `0x12345 / 0` -> 0x12345
/// (r1 0), signed `-0x12345 / 0` -> -0x12345.
pub(crate) fn u32_divmod(n: u32, d: u32) -> (u32, u32) {
    match (n.checked_div(d), n.checked_rem(d)) {
        (Some(q), Some(r)) => (q, r),
        _ => (n, 0),
    }
}

/// `_u32_div_f` (`0x0209C2B0`) quotient: `n / d`; a zero divisor returns `n` (see [`u32_divmod`]).
pub(crate) fn u32_div(n: u32, d: u32) -> u32 {
    u32_divmod(n, d).0
}

/// `_s32_div_f` (`FUN_0209C0A4`) quotient, truncating toward zero (wrapping for `i32::MIN / -1`); a zero divisor
/// returns `n` (see [`u32_divmod`]).
pub(crate) fn s32_div(n: i32, d: i32) -> i32 {
    if d == 0 {
        n
    } else {
        n.wrapping_div(d)
    }
}

/// C: `FUN_overlay_d_10__02159204`: helper of command 48: an event that re-runs the 14 init procs:
/// `event = FUN_020122C0(GAMESYS, 0, 0x021591B5, 0x18)` (event fn `script_sys::run_init_procs`), its work
/// `{+0 GAMESYS, +4 ctx, +8 SW, +0xC 0, +0x10 0, +0x14 15}`; returns the event.
pub fn init_procs_event<H: CoreHooks>(h: &mut H, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gamesys = h.fun_0215a5dc(ctx);
    let event = h.fun_020122c0(gamesys, 0, INIT_PROCS_EVENT_FN, 0x18);
    let w = h.fun_020124e8(event);
    h.w32(w, gamesys);
    h.w32(w + 4, ctx);
    h.w32(w + 8, sw);
    h.w32(w + 0x10, 0);
    h.w32(w + 0xC, 0);
    h.w32(w + 0x14, 15);
    event
}

/// C: `sub_02159240`: command 0: no-op.
pub fn cmd_nop<H: MemEnv>(_h: &mut H, _vm: u32, _ctx: u32) -> u32 {
    0
}

/// C: `sub_02159244`: command 1: no-op (a second copy of command 0).
pub fn cmd_nop_1<H: MemEnv>(_h: &mut H, _vm: u32, _ctx: u32) -> u32 {
    0
}

/// C: `sub_02159248`: command 2: end the script (`vm::stop`) and yield.
pub fn cmd_end<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_02011290(vm);
    1
}

/// C: `sub_02159254`: the wait callback of command 3 (`0x02159255`): one frame of the scrcmd-work countdown
/// (`FUN_0215A690(ctx)`, tail call); 1 = done.
pub fn wait_countdown<H: CoreHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    h.fun_0215a690(ctx)
}

/// C: `sub_02159260`: command 3: wait `n` frames (u16 operand): the countdown = `n` (`FUN_0215A684`), wait on
/// [`WAIT_COUNTDOWN`]; yields.
pub fn cmd_wait_frames<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let n = h.fun_02011330(vm);
    h.fun_0215a684(ctx, n);
    h.fun_020113d0(vm, WAIT_COUNTDOWN);
    1
}

/// C: `sub_02159284`: command 4: call relative: `off` = u32 operand; `vm::call(PC + off)` (PC after the operand).
pub fn cmd_call_rel<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let off = h.fun_0201134c(vm);
    let pc = h.r32(vm + VM_PC);
    h.fun_020113b4(vm, pc.wrapping_add(off));
    0
}

/// C: `sub_0215929c`: command 5: return (`vm::ret`).
pub fn cmd_return<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_020113c4(vm);
    0
}

/// C: `sub_021592a8`: command 6: reads the GAMEDATA (unused) and consumes one value operand (UNVERIFIED: a debug /
/// no-op command).
pub fn cmd_skip_value<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5e4(ctx);
    h.fun_02159b10(vm, ctx);
    0
}

/// C: `sub_021592c0`: command 7: pops the expression stack and consumes one value operand, both discarded
/// (UNVERIFIED role).
pub fn cmd_pop_skip_value<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_02011398(vm);
    h.fun_02159b10(vm, ctx);
    0
}

/// C: `sub_021592d8`: command 20: reg[b0] = b1 (two script bytes).
pub fn cmd_reg_set_byte<H: MemEnv>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let r = read_u8(h, vm);
    let v = read_u8(h, vm);
    let a = reg_addr(h, vm, r);
    h.w32(a, v as u32);
    0
}

/// C: `sub_021592f0`: command 21: reg[b0] = u32 operand.
pub fn cmd_reg_set_u32<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let r = read_u8(h, vm);
    let v = h.fun_0201134c(vm);
    let a = reg_addr(h, vm, r);
    h.w32(a, v);
    0
}

/// C: `sub_0215930c`: command 22: reg[b0] = reg[b1].
pub fn cmd_reg_copy<H: MemEnv>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let d = read_u8(h, vm);
    let s = read_u8(h, vm);
    let sa = reg_addr(h, vm, s);
    let v = h.r32(sa);
    let da = reg_addr(h, vm, d);
    h.w32(da, v);
    0
}

/// C: `FUN_overlay_d_10__0215932c`: compare: 0 if `a < b`, 1 if equal, 2 if `a > b` (unsigned). The compare commands
/// 23..26 store it in VM +0xE for the conditional commands 31 / 32.
pub fn compare(a: u32, b: u32) -> u32 {
    if a < b {
        0
    } else if a == b {
        1
    } else {
        2
    }
}

/// C: `sub_02159340`: command 23: VM +0xE = [`compare`]`(reg[b0] & 0xFF, reg[b1] & 0xFF)`.
pub fn cmd_reg_compare<H: MemEnv>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let r0 = read_u8(h, vm);
    let r1 = read_u8(h, vm);
    let a0 = reg_addr(h, vm, r0);
    let a = h.r32(a0) & 0xFF;
    let a1 = reg_addr(h, vm, r1);
    let b = h.r32(a1) & 0xFF;
    let c = compare(a, b);
    h.w8(vm + VM_0E, c as u8);
    0
}

/// C: `sub_02159370`: command 24: VM +0xE = [`compare`]`(reg[b0] & 0xFF, b1)`.
pub fn cmd_reg_compare_byte<H: MemEnv>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let r = read_u8(h, vm);
    let b = read_u8(h, vm);
    let ra = reg_addr(h, vm, r);
    let a = h.r32(ra) & 0xFF;
    let c = compare(a, b as u32);
    h.w8(vm + VM_0E, c as u8);
    0
}

/// C: `sub_02159398`: command 25: VM +0xE = [`compare`]`(var, u16 literal)`.
pub fn cmd_var_compare_literal<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let va = h.fun_02159ae8(vm, ctx);
    let a = h.r16(va) as u32;
    let b = h.fun_02011330(vm);
    let c = compare(a, b);
    h.w8(vm + VM_0E, c as u8);
    0
}

/// C: `sub_021593b8`: command 26: VM +0xE = [`compare`]`(var a, var b)`.
pub fn cmd_var_compare_var<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let pa = h.fun_02159ae8(vm, ctx);
    let pb = h.fun_02159ae8(vm, ctx);
    let a = h.r16(pa) as u32;
    let b = h.r16(pb) as u32;
    let c = compare(a, b);
    h.w8(vm + VM_0E, c as u8);
    0
}

/// C: `sub_021593dc`: command 8: push the u16 literal operand.
pub fn cmd_push_literal<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let v = h.fun_02011330(vm);
    h.fun_0201137c(vm, v);
    0
}

/// C: `sub_021593f0`: command 9: push the value of a var / literal operand.
pub fn cmd_push_value<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02159b10(vm, ctx);
    h.fun_0201137c(vm, v);
    0
}

/// C: `sub_02159404`: command 10: var = pop (low halfword).
pub fn cmd_pop_to_var<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02011398(vm);
    let a = h.fun_02159ae8(vm, ctx);
    h.w16(a, v as u16);
    0
}

/// C: `sub_02159420`: command 11: pop and discard.
pub fn cmd_pop<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    h.fun_02011398(vm);
    0
}

/// Pop `a` then `b` and push `f(b, a)` (commands 12..15).
fn stack_op<H: CoreHooks>(h: &mut H, vm: u32, f: impl FnOnce(u32, u32) -> u32) -> u32 {
    let a = h.fun_02011398(vm);
    let b = h.fun_02011398(vm);
    h.fun_0201137c(vm, f(b, a));
    0
}

/// C: `sub_0215942c`: command 12: `a = pop, b = pop, push(b + a)`.
pub fn cmd_stack_add<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    stack_op(h, vm, |b, a| b.wrapping_add(a))
}

/// C: `sub_0215944c`: command 13: `a = pop, b = pop, push(b - a)`.
pub fn cmd_stack_sub<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    stack_op(h, vm, |b, a| b.wrapping_sub(a))
}

/// C: `sub_0215946c`: command 14: `a = pop, b = pop, push(b * a)`.
pub fn cmd_stack_mul<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    stack_op(h, vm, |b, a| b.wrapping_mul(a))
}

/// C: `sub_0215948c`: command 15: `a = pop, b = pop, push(b / a)` (`_u32_div_f`: `b` when `a` is 0).
pub fn cmd_stack_div<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    stack_op(h, vm, |b, a| u32_divmod(b, a).0)
}

/// C: `sub_021594b0`: command 16: push whether event flag `value` is set (`FUN_020142E8(event work, value)`).
pub fn cmd_push_flag<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let ew = h.fun_0201296c(gd);
    let flag = h.fun_02159b10(vm, ctx);
    let v = h.fun_020142e8(ew, flag);
    h.fun_0201137c(vm, v);
    0
}

/// C: `sub_021594e0`: command 17: `a = pop, b = pop, op = u16 operand; push(b OP a)` with op 0 `<`, 1 `==`, 2 `>`,
/// 3 `<=`, 4 `>=`, 5 `!=` (unsigned), 6 `b == 1 || a == 1`, 7 `b == 1 && a == 1`; any other op pushes 0.
pub fn cmd_stack_compare<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let a = h.fun_02011398(vm);
    let b = h.fun_02011398(vm);
    let op = h.fun_02011330(vm);
    let r = match op {
        0 => b < a,
        1 => b == a,
        2 => b > a,
        3 => b <= a,
        4 => b >= a,
        5 => b != a,
        6 => b == 1 || a == 1,
        7 => b == 1 && a == 1,
        _ => false,
    };
    h.fun_0201137c(vm, r as u32);
    0
}

/// C: `sub_02159570`: command 27: start script `id` (u16 operand) in a parallel VM of this script event
/// (`FUN_021589A0(SW, zone, id)`) and yield (UNVERIFIED role: "run a sub-script without waiting").
pub fn cmd_start_parallel<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let id = h.fun_02011330(vm);
    let zone = h.fun_02159b38(ctx);
    h.fun_021589a0(sw, zone, id);
    1
}

/// C: `sub_021595a0`: the wait callback of command 28 (`0x021595A1`): once the VM in the ctx's wait slot has ended
/// (`FUN_021587E0(SW, slot) == 0`) restore the saved script vars (`FUN_0215AA34`) and report done (1); else 0.
pub fn wait_call_script<H: CoreHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let slot = h.fun_0215aa4c(ctx);
    if h.fun_021587e0(sw, slot) == 0 {
        h.fun_0215aa34(ctx);
        return 1;
    }
    0
}

/// C: `sub_021595cc`: command 28: call script `id` (u16 operand) and wait. Ctx mode 0: start it in a parallel VM, keep
/// its slot (`FUN_0215AA48`), save the script vars (`FUN_0215AA24`), wait on [`WAIT_CALL_SCRIPT`] and yield; other
/// modes: save the vars, run it synchronously (`FUN_02158A34(GAMESYS, SW, id, mode)`), restore the vars, next command.
pub fn cmd_call_script<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gamesys = h.fun_0215a5dc(ctx);
    let sw = h.fun_0215a5f4(ctx);
    let id = h.fun_02011330(vm);
    if h.fun_0215a5d4(ctx) == 0 {
        let zone = h.fun_02159b38(ctx);
        let slot = h.fun_021589a0(sw, zone, id);
        h.fun_0215aa48(ctx, slot);
        h.fun_0215aa24(ctx);
        h.fun_020113d0(vm, WAIT_CALL_SCRIPT);
        return 1;
    }
    h.fun_0215aa24(ctx);
    let mode = h.fun_0215a5d8(ctx);
    h.fun_02158a34(gamesys, sw, id, mode);
    h.fun_0215aa34(ctx);
    0
}

/// C: `sub_02159644`: command 29: end the script (`vm::stop`; the SW read is unused) and yield.
pub fn cmd_end_2<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_0215a5f4(ctx);
    h.fun_02011290(vm);
    1
}

/// C: `sub_02159658`: command 30: jump relative: `off` = u32 operand; PC = PC + `off` (PC after the operand).
pub fn cmd_jump_rel<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let off = h.fun_0201134c(vm);
    let pc = h.r32(vm + VM_PC);
    h.fun_020113b0(vm, pc.wrapping_add(off));
    0
}

/// C: `sub_02159670`: command 31: conditional jump. `cond` = script byte, `off` = u32 operand; cond 0xFF jumps unless
/// [`pop`](CoreHooks::fun_02011398) gives 1; any other cond jumps when `COND[cond * 3 + VM+0xE] == 1`. The target is
/// the PC after the operands + `off`. A cond past the table (the original reads the bytes after it) panics.
pub fn cmd_if_jump<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let cond = read_u8(h, vm);
    let off = h.fun_0201134c(vm);
    let jump = if cond == 0xFF {
        h.fun_02011398(vm) != 1
    } else {
        let r = h.r8(vm + VM_0E) as usize;
        COND[cond as usize * 3 + r] == 1
    };
    if jump {
        let pc = h.r32(vm + VM_PC);
        h.fun_020113b0(vm, pc.wrapping_add(off));
    }
    0
}

/// C: `sub_021596b4`: command 32: conditional call: `cond` = script byte, `off` = u32 operand; when
/// `COND[cond * 3 + VM+0xE] == 1`, `vm::call(PC + off)`. A cond past the table panics (the original reads on).
pub fn cmd_if_call<H: CoreHooks>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let cond = read_u8(h, vm);
    let off = h.fun_0201134c(vm);
    let r = h.r8(vm + VM_0E) as usize;
    if COND[cond as usize * 3 + r] == 1 {
        let pc = h.r32(vm + VM_PC);
        h.fun_020113b4(vm, pc.wrapping_add(off));
    }
    0
}

/// C: `sub_021596e8`: command 33: `FUN_02014474(FUN_02012B38(GAMEDATA), u16 operand)` (UNVERIFIED: sets a value of a
/// GAMEDATA sub record).
pub fn cmd_33<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let v = h.fun_02011330(vm);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012b38(gd);
    h.fun_02014474(x, v);
    0
}

/// C: `sub_02159708`: command 34: var = `FUN_02014460(FUN_02012B38(GAMEDATA))` (the value command 33 sets).
pub fn cmd_34<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let gd = h.fun_0215a5e4(ctx);
    let x = h.fun_02012b38(gd);
    let v = h.fun_02014460(x);
    h.w16(a, v as u16);
    0
}

/// C: `sub_02159728`: command 35: set event flag `value`.
pub fn cmd_flag_set<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let ew = h.fun_0201296c(gd);
    let flag = h.fun_02159b10(vm, ctx);
    h.fun_02014314(ew, flag);
    0
}

/// C: `sub_02159750`: command 36: clear event flag `value`.
pub fn cmd_flag_clear<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let ew = h.fun_0201296c(gd);
    let flag = h.fun_02159b10(vm, ctx);
    h.fun_0201433c(ew, flag);
    0
}

/// C: `sub_02159778`: command 37: var = event flag `value` (operands: the flag value, then the var).
pub fn cmd_flag_get<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let gd = h.fun_0215a5e4(ctx);
    let ew = h.fun_0201296c(gd);
    let flag = h.fun_02159b10(vm, ctx);
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_020142e8(ew, flag);
    h.w16(a, v as u16);
    0
}

/// `var = f(var, value)` (u16; operands: the var, then the value) for commands 18, 19, 38, 39, 43..45.
fn var_op<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32, f: impl FnOnce(u32, u32) -> u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let x = h.r16(a) as u32;
    h.w16(a, f(x, v) as u16);
    0
}

/// C: `sub_021597ac`: command 38: var += value (u16 wrap).
pub fn cmd_var_add<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    var_op(h, vm, ctx, |x, v| x.wrapping_add(v))
}

/// C: `sub_021597cc`: command 39: var -= value (u16 wrap).
pub fn cmd_var_sub<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    var_op(h, vm, ctx, |x, v| x.wrapping_sub(v))
}

/// C: `sub_021597ec`: command 43: var *= value (u16 wrap).
pub fn cmd_var_mul<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    var_op(h, vm, ctx, |x, v| x.wrapping_mul(v))
}

/// C: `sub_0215980c`: command 44: var /= value (`_u32_div_f`; unchanged when value is 0).
pub fn cmd_var_div<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    var_op(h, vm, ctx, |x, v| u32_divmod(x, v).0)
}

/// C: `sub_02159830`: command 45: var %= value (`_u32_div_f` remainder; 0 when value is 0).
pub fn cmd_var_mod<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    var_op(h, vm, ctx, |x, v| u32_divmod(x, v).1)
}

/// C: `sub_02159854`: command 18: var &= value.
pub fn cmd_var_and<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    var_op(h, vm, ctx, |x, v| x & v)
}

/// C: `sub_02159874`: command 19: var |= value.
pub fn cmd_var_or<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    var_op(h, vm, ctx, |x, v| x | v)
}

/// C: `sub_02159894`: command 40: var = u16 literal.
pub fn cmd_var_set_literal<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02011330(vm);
    h.w16(a, v as u16);
    0
}

/// C: `sub_021598ac`: command 41: var = var2.
pub fn cmd_var_copy<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let d = h.fun_02159ae8(vm, ctx);
    let s = h.fun_02159ae8(vm, ctx);
    let v = h.r16(s);
    h.w16(d, v);
    0
}

/// C: `sub_021598c8`: command 42: var = value (a var or a literal).
pub fn cmd_var_set_value<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    h.w16(a, v as u16);
    0
}

/// C: `sub_021598e4`: command 46: a tail call (`ldr r3, =0x021B1BB9; bx r3`) to the overlay-21 handler
/// `FUN_021B1BB8(vm, ctx)`.
pub fn cmd_46<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    h.fun_021b1bb8(vm, ctx)
}

/// C: `sub_021598ec`: command 47: `FUN_021B1BE8(ctx)`; then when `FUN_020288C4(FUN_02012A30(GAMEDATA))` holds, call the
/// child event `FUN_0202972C(GAMESYS, 0)` (`FUN_0215897C`) and yield (1), else 0 (UNVERIFIED role).
pub fn cmd_47<H: CoreHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    let gamesys = h.fun_0215a5dc(ctx);
    let gd = h.fun_020120f4(gamesys);
    h.fun_021b1be8(ctx);
    let x = h.fun_02012a30(gd);
    if h.fun_020288c4(x) != 0 {
        let ev = h.fun_0202972c(gamesys, 0);
        h.fun_0215897c(sw, ev);
        return 1;
    }
    0
}

/// C: `sub_02159934`: command 48: re-run the init procs: the event [`init_procs_event`]`(ctx)`, when nonzero, becomes a
/// child event of the script (`FUN_0215897C(SW, event)`); yields.
pub fn cmd_48<H: CoreHooks>(h: &mut H, _vm: u32, ctx: u32) -> u32 {
    let sw = h.fun_0215a5f4(ctx);
    h.fun_0215a5dc(ctx);
    let ev = init_procs_event(h, ctx);
    if ev != 0 {
        h.fun_0215897c(sw, ev);
    }
    1
}

/// Run the command handler at `handler` (the command table word; the Thumb bit may be set) when it belongs to this
/// area; `None` for any other address. This is the `scrcmd_core` part of the F7
/// [`CommandDispatch`](crate::hooks::CommandDispatch) seam (see [`crate::scrcmd::dispatch`]).
pub fn dispatch<H: CoreHooks>(h: &mut H, handler: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match handler & !1 {
        0x0215_9240 => cmd_nop(h, vm, ctx),
        0x0215_9244 => cmd_nop_1(h, vm, ctx),
        0x0215_9248 => cmd_end(h, vm, ctx),
        0x0215_9260 => cmd_wait_frames(h, vm, ctx),
        0x0215_9284 => cmd_call_rel(h, vm, ctx),
        0x0215_929C => cmd_return(h, vm, ctx),
        0x0215_92A8 => cmd_skip_value(h, vm, ctx),
        0x0215_92C0 => cmd_pop_skip_value(h, vm, ctx),
        0x0215_93DC => cmd_push_literal(h, vm, ctx),
        0x0215_93F0 => cmd_push_value(h, vm, ctx),
        0x0215_9404 => cmd_pop_to_var(h, vm, ctx),
        0x0215_9420 => cmd_pop(h, vm, ctx),
        0x0215_942C => cmd_stack_add(h, vm, ctx),
        0x0215_944C => cmd_stack_sub(h, vm, ctx),
        0x0215_946C => cmd_stack_mul(h, vm, ctx),
        0x0215_948C => cmd_stack_div(h, vm, ctx),
        0x0215_94B0 => cmd_push_flag(h, vm, ctx),
        0x0215_94E0 => cmd_stack_compare(h, vm, ctx),
        0x0215_9854 => cmd_var_and(h, vm, ctx),
        0x0215_9874 => cmd_var_or(h, vm, ctx),
        0x0215_92D8 => cmd_reg_set_byte(h, vm, ctx),
        0x0215_92F0 => cmd_reg_set_u32(h, vm, ctx),
        0x0215_930C => cmd_reg_copy(h, vm, ctx),
        0x0215_9340 => cmd_reg_compare(h, vm, ctx),
        0x0215_9370 => cmd_reg_compare_byte(h, vm, ctx),
        0x0215_9398 => cmd_var_compare_literal(h, vm, ctx),
        0x0215_93B8 => cmd_var_compare_var(h, vm, ctx),
        0x0215_9570 => cmd_start_parallel(h, vm, ctx),
        0x0215_95CC => cmd_call_script(h, vm, ctx),
        0x0215_9644 => cmd_end_2(h, vm, ctx),
        0x0215_9658 => cmd_jump_rel(h, vm, ctx),
        0x0215_9670 => cmd_if_jump(h, vm, ctx),
        0x0215_96B4 => cmd_if_call(h, vm, ctx),
        0x0215_96E8 => cmd_33(h, vm, ctx),
        0x0215_9708 => cmd_34(h, vm, ctx),
        0x0215_9728 => cmd_flag_set(h, vm, ctx),
        0x0215_9750 => cmd_flag_clear(h, vm, ctx),
        0x0215_9778 => cmd_flag_get(h, vm, ctx),
        0x0215_97AC => cmd_var_add(h, vm, ctx),
        0x0215_97CC => cmd_var_sub(h, vm, ctx),
        0x0215_9894 => cmd_var_set_literal(h, vm, ctx),
        0x0215_98AC => cmd_var_copy(h, vm, ctx),
        0x0215_98C8 => cmd_var_set_value(h, vm, ctx),
        0x0215_97EC => cmd_var_mul(h, vm, ctx),
        0x0215_980C => cmd_var_div(h, vm, ctx),
        0x0215_9830 => cmd_var_mod(h, vm, ctx),
        0x0215_98E4 => cmd_46(h, vm, ctx),
        0x0215_98EC => cmd_47(h, vm, ctx),
        0x0215_9934 => cmd_48(h, vm, ctx),
        _ => return None,
    })
}

/// Run the wait callback `f` (the VM +0x10 word, Thumb bit set) when it is one of this area's (commands 3 and 28);
/// `None` otherwise (the `scrcmd_core` part of `VmHooks::call_wait`).
pub fn call_wait<H: CoreHooks>(h: &mut H, f: u32, vm: u32, ctx: u32) -> Option<u32> {
    Some(match f {
        WAIT_COUNTDOWN => wait_countdown(h, vm, ctx),
        WAIT_CALL_SCRIPT => wait_call_script(h, vm, ctx),
        _ => return None,
    })
}

#[cfg(test)]
mod tests;
