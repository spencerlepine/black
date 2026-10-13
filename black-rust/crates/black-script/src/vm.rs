//! F7: the script interpreter (`vm.c`, main ARM9 0x020111EC-0x020113D8).
//!
//! Every function is specced in `tools/rom-diff/specs/black-script/vm.spec` (its header documents the VM layout and the
//! step loop `0x02011298`); all 16 are ported here and checked with `tools/check-fn.sh black-script vm`.
//!
//! The VM is a byte-code interpreter over a flat memory block (`0x30 + 4 * (stack + regs)` bytes):
//! one [`step`] per frame reads u16 command ids at the PC and calls `table[cmd](vm, ctx)` through
//! [`CommandDispatch`](crate::hooks::CommandDispatch) until a command yields (returns 1) or the script ends. A command
//! that has to wait for the game (a message, a fade, a key press) installs a wait callback with [`wait`]; the next steps
//! only poll that callback ([`VmHooks::call_wait`]) until it reports 1.

use crate::hooks::{HeapHooks, MemEnv, VmHooks};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_11EC,
    0x0201_121C,
    0x0201_126C,
    0x0201_127C,
    0x0201_1290,
    0x0201_1298,
    0x0201_1324,
    0x0201_1328,
    0x0201_1330,
    0x0201_134C,
    0x0201_137C,
    0x0201_1398,
    0x0201_13B0,
    0x0201_13B4,
    0x0201_13C4,
    0x0201_13D0,
];

/// VM field offsets (see the spec header).
pub const VM_STACK_WORDS: u32 = 0x00;
pub const VM_REG_WORDS: u32 = 0x02;
pub const VM_TABLE: u32 = 0x04;
pub const VM_COUNT: u32 = 0x08;
pub const VM_DEPTH: u32 = 0x0C;
pub const VM_STATUS: u32 = 0x0D;
pub const VM_0E: u32 = 0x0E;
pub const VM_WAIT: u32 = 0x10;
pub const VM_PC: u32 = 0x14;
pub const VM_STACK: u32 = 0x18;
pub const VM_REGS: u32 = 0x1C;
pub const VM_CTX: u32 = 0x20;
pub const VM_HOOK: u32 = 0x24;
pub const VM_HOOK_ARG: u32 = 0x28;
/// Size of the VM header; the regs follow it, then the call stack.
pub const VM_HEADER: u32 = 0x30;

/// VM status byte (+0xD).
pub const STATUS_STOPPED: u8 = 0;
pub const STATUS_RUNNING: u8 = 1;
pub const STATUS_WAITING: u8 = 2;

/// Address of the string `"vm.c"` (the file argument of the allocation in `0x0201121C`).
pub const FILE_VM: u32 = 0x020A_7268;

/// `MI_CpuFill8(dst, v, n)` (`0x02082BCC`), inlined.
fn fill8<M: MemEnv>(m: &mut M, dst: u32, v: u8, n: u32) {
    for i in 0..n {
        m.w8(dst.wrapping_add(i), v);
    }
}

/// C: `FUN_020111ec`: reset: PC, +0xE, status, wait callback and call depth = 0; clear the call stack
/// (`stack words * 4` bytes at +0x18) and the regs (`reg words * 4` bytes at +0x1C); hook and its argument = 0.
pub fn reset<M: MemEnv>(m: &mut M, vm: u32) {
    let words = m.r16(vm + VM_STACK_WORDS) as u32;
    let stack = m.r32(vm + VM_STACK);
    m.w32(vm + VM_PC, 0);
    m.w8(vm + VM_0E, 0);
    m.w8(vm + VM_STATUS, STATUS_STOPPED);
    m.w32(vm + VM_WAIT, 0);
    m.w8(vm + VM_DEPTH, 0);
    fill8(m, stack, 0, words << 2);
    let regs_n = m.r16(vm + VM_REG_WORDS) as u32;
    let regs = m.r32(vm + VM_REGS);
    fill8(m, regs, 0, regs_n << 2);
    m.w32(vm + VM_HOOK, 0);
    m.w32(vm + VM_HOOK_ARG, 0);
}

/// C: `FUN_0201121C`: create a VM: `Heap_AllocDebug(heap, 0x30 + 4 * (regs + stack), 0, "vm.c", 0x60)`, zeroed; the
/// 12-byte init record `{u16 stack words, u16 reg words, command table, command count}` is copied to +0; regs at +0x30
/// (+0x1C), call stack after them (+0x18). Returns the VM (the allocation is not checked).
pub fn create<H: HeapHooks>(h: &mut H, heap: u32, init: &[u8; 12]) -> u32 {
    let stack_words = u16::from_le_bytes([init[0], init[1]]) as u32;
    let reg_words = u16::from_le_bytes([init[2], init[3]]) as u32;
    let regs_end = (reg_words << 2).wrapping_add(VM_HEADER);
    let stack_size = stack_words << 2;
    let size = regs_end.wrapping_add(stack_size);
    let vm = h.heap_alloc(heap, size, 0, FILE_VM, 0x60);
    fill8(h, vm, 0, size);
    for (i, b) in init.iter().enumerate() {
        h.w8(vm.wrapping_add(i as u32), *b);
    }
    let regs = vm.wrapping_add(VM_HEADER);
    h.w32(vm.wrapping_add(VM_REGS), regs);
    h.w32(vm.wrapping_add(VM_STACK), regs.wrapping_add(reg_words << 2));
    vm
}

/// C: `FUN_0201126C`: [`reset`], then ctx (+0x20) = `ctx`.
pub fn init<M: MemEnv>(m: &mut M, vm: u32, ctx: u32) {
    reset(m, vm);
    m.w32(vm + VM_CTX, ctx);
}

/// C: `FUN_0201127C`: [`reset`], PC = `pc`, status running (start a script).
pub fn start<M: MemEnv>(m: &mut M, vm: u32, pc: u32) {
    reset(m, vm);
    m.w32(vm + VM_PC, pc);
    m.w8(vm + VM_STATUS, STATUS_RUNNING);
}

/// C: `FUN_02011290`: stop: status 0, PC 0.
pub fn stop<M: MemEnv>(m: &mut M, vm: u32) {
    m.w8(vm + VM_STATUS, STATUS_STOPPED);
    m.w32(vm + VM_PC, 0);
}

/// C: `FUN_02011298`: run the VM for one frame. Returns 0 when it is (or becomes) stopped, else 1.
///
/// * stopped: 0.
/// * waiting: with a wait callback, call it; 1 clears the callback and sets running; either way return 1 (no command
///   runs this frame). Without a callback: set running and run.
/// * running: loop { PC 0 -> stop, 0; `cmd` = next u16; `cmd >= count` -> stop, 0; a hook that returns 0 -> stop, 0;
///   `table[cmd](vm, ctx) == 1` -> 1 }.
/// * any other status: stop (status 0) and return 1.
pub fn step<H: VmHooks>(h: &mut H, vm: u32) -> u32 {
    match h.r8(vm + VM_STATUS) {
        STATUS_STOPPED => return 0,
        STATUS_RUNNING => {}
        STATUS_WAITING => {
            let f = h.r32(vm + VM_WAIT);
            if f != 0 {
                let ctx = h.r32(vm + VM_CTX);
                if h.call_wait(f, vm, ctx) == 1 {
                    h.w32(vm + VM_WAIT, 0);
                    h.w8(vm + VM_STATUS, STATUS_RUNNING);
                }
                return 1;
            }
            h.w8(vm + VM_STATUS, STATUS_RUNNING);
        }
        _ => {
            h.w8(vm + VM_STATUS, STATUS_STOPPED);
            return 1;
        }
    }
    loop {
        if h.r32(vm + VM_PC) == 0 {
            h.w8(vm + VM_STATUS, STATUS_STOPPED);
            return 0;
        }
        let cmd = read_u16(h, vm);
        if cmd >= h.r32(vm + VM_COUNT) {
            h.w8(vm + VM_STATUS, STATUS_STOPPED);
            return 0;
        }
        let hook = h.r32(vm + VM_HOOK);
        if hook != 0 {
            let ctx = h.r32(vm + VM_CTX);
            let arg = h.r32(vm + VM_HOOK_ARG);
            if h.call_hook(hook, vm, ctx, arg, cmd) == 0 {
                h.w8(vm + VM_STATUS, STATUS_STOPPED);
                return 0;
            }
        }
        let table = h.r32(vm + VM_TABLE);
        let ctx = h.r32(vm + VM_CTX);
        let handler = h.r32(table.wrapping_add(cmd << 2));
        if h.command(handler, vm, ctx) == 1 {
            return 1;
        }
    }
}

/// C: `FUN_02011324`: the ctx (+0x20).
pub fn ctx<M: MemEnv>(m: &mut M, vm: u32) -> u32 {
    m.r32(vm + VM_CTX)
}

/// C: `FUN_02011328`: install the pre-dispatch hook: +0x24 = `hook`, +0x28 = `arg`.
pub fn set_hook<M: MemEnv>(m: &mut M, vm: u32, hook: u32, arg: u32) {
    m.w32(vm + VM_HOOK, hook);
    m.w32(vm + VM_HOOK_ARG, arg);
}

/// C: `FUN_02011330`: the next little-endian u16 of the script at the VM's PC; PC += 2.
pub fn read_u16<M: MemEnv>(m: &mut M, vm: u32) -> u32 {
    let pc = m.r32(vm + VM_PC);
    let lo = m.r8(pc) as u32;
    m.w32(vm + VM_PC, pc.wrapping_add(1));
    let hi = m.r8(pc.wrapping_add(1)) as u32;
    m.w32(vm + VM_PC, pc.wrapping_add(2));
    (lo + (hi << 8)) & 0xFFFF
}

/// C: `FUN_0201134C`: the next little-endian u32 of the script at the VM's PC (any alignment, byte reads); PC += 4
/// (stored after every byte, as the original does).
pub fn read_u32<M: MemEnv>(m: &mut M, vm: u32) -> u32 {
    let pc = m.r32(vm + VM_PC);
    let b0 = m.r8(pc) as u32;
    m.w32(vm + VM_PC, pc.wrapping_add(1));
    let b1 = m.r8(pc.wrapping_add(1)) as u32;
    m.w32(vm + VM_PC, pc.wrapping_add(2));
    let b2 = m.r8(pc.wrapping_add(2)) as u32;
    m.w32(vm + VM_PC, pc.wrapping_add(3));
    let b3 = m.r8(pc.wrapping_add(3)) as u32;
    m.w32(vm + VM_PC, pc.wrapping_add(4));
    (((((b3 << 8) + b2) << 8) + b1) << 8) + b0
}

/// C: `FUN_0201137C`: push `v` on the call stack when `depth + 1 < stack words` (signed compare), then depth += 1;
/// a full stack drops the value silently.
pub fn push<M: MemEnv>(m: &mut M, vm: u32, v: u32) {
    let depth = m.r8(vm + VM_DEPTH) as i32;
    let words = m.r16(vm + VM_STACK_WORDS) as i32;
    if depth + 1 < words {
        let stack = m.r32(vm + VM_STACK);
        m.w32(stack.wrapping_add(depth as u32 * 4), v);
        let d = m.r8(vm + VM_DEPTH);
        m.w8(vm + VM_DEPTH, d.wrapping_add(1));
    }
}

/// C: unlabeled routine at `0x02011398`: pop: depth 0 -> 0; else depth -= 1 and the call stack word at the new depth.
pub fn pop<M: MemEnv>(m: &mut M, vm: u32) -> u32 {
    let d = m.r8(vm + VM_DEPTH);
    if d == 0 {
        return 0;
    }
    m.w8(vm + VM_DEPTH, d - 1);
    let stack = m.r32(vm + VM_STACK);
    let d = m.r8(vm + VM_DEPTH) as u32;
    m.r32(stack.wrapping_add(d << 2))
}

/// C: `FUN_020113B0`: jump: PC = `pc`.
pub fn jump<M: MemEnv>(m: &mut M, vm: u32, pc: u32) {
    m.w32(vm + VM_PC, pc);
}

/// C: `FUN_020113B4`: call: [`push`] the current PC, PC = `pc`.
pub fn call<M: MemEnv>(m: &mut M, vm: u32, pc: u32) {
    let cur = m.r32(vm + VM_PC);
    push(m, vm, cur);
    m.w32(vm + VM_PC, pc);
}

/// C: `FUN_020113c4`: return: PC = [`pop`] (0 on an empty stack, which ends the script at the next step).
pub fn ret<M: MemEnv>(m: &mut M, vm: u32) {
    let pc = pop(m, vm);
    m.w32(vm + VM_PC, pc);
}

/// C: `FUN_020113D0`: wait: status waiting, wait callback (+0x10) = `f` (polled by [`step`]).
pub fn wait<M: MemEnv>(m: &mut M, vm: u32, f: u32) {
    m.w8(vm + VM_STATUS, STATUS_WAITING);
    m.w32(vm + VM_WAIT, f);
}

#[cfg(test)]
mod tests;
