//! F8 (first batch): the scrcmd core, overlay 10 0x02159204-0x0215995E: script commands 0..48 (VM control, the
//! expression stack, VM regs, compares and conditional jumps, var arithmetic, event flags, sub-scripts) and their
//! helpers.
//!
//! Every function of the area (53) is specced in `tools/rom-diff/specs/black-script/scrcmd_core.spec`; its header has the
//! command-id table (id -> handler) and the VM fields the commands touch. The functions listed in [`PORTED`] are ported
//! here and checked with `tools/check-fn.sh black-script scrcmd_core`; the rest are the F8 port job's.
//!
//! Commands are `fn(vm, ctx) -> 0 (next command) / 1 (yield)` (`vm::step` calls `table[cmd](vm, ctx)` through
//! [`CommandDispatch`](crate::hooks::CommandDispatch)). Pointers are `u32` addresses in the [`Mem`](crate::hooks::Mem)
//! window; the callees (vm.c, script_sys, other areas) are [`CoreHooks`] methods named by address.

use crate::hooks::MemEnv;
use crate::vm::{VM_0E, VM_PC, VM_REGS};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_92D8,
    0x0215_932C,
    0x0215_9340,
    0x0215_9670,
    0x0215_97AC,
];

/// The condition table of commands 31 / 32 (overlay 10 data at 0x021705DC, 6 conditions x 3 bytes): `COND[c * 3 + r]` is 1
/// when condition `c` holds for the compare result `r` (0 less, 1 equal, 2 greater; VM +0xE). Conditions: 0 `<`, 1 `==`,
/// 2 `>`, 3 `<=`, 4 `>=`, 5 `!=`.
pub const COND: [u8; 18] = [1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 1];
/// ROM address of [`COND`].
pub const COND_ADDR: u32 = 0x0217_05DC;

/// Callees of the scrcmd core commands that this area does not port, named by address (meaning per the spec).
pub trait CoreHooks: MemEnv {
    /// C: `FUN_0201134C` (vm.c `read_u32`): the next u32 operand of the script; PC += 4.
    fn fun_0201134c(&mut self, vm: u32) -> u32;
    /// C: unlabeled routine at `0x02011398` (vm.c `pop`): pop the call / expression stack (0 when empty).
    fn fun_02011398(&mut self, vm: u32) -> u32;
    /// C: `FUN_020113B0` (vm.c `jump`): PC = `pc`.
    fn fun_020113b0(&mut self, vm: u32, pc: u32);
    /// C: `FUN_02159AE8` (script_sys `cmd_var_addr`): address of the var whose id is the next script word.
    fn fun_02159ae8(&mut self, vm: u32, ctx: u32) -> u32;
    /// C: `FUN_02159B10` (script_sys `cmd_var_value`): the next script word read as a var id or a literal.
    fn fun_02159b10(&mut self, vm: u32, ctx: u32) -> u32;
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

/// C: `sub_021592d8`: command 20: reg[b0] = b1 (two script bytes).
pub fn cmd_reg_set_byte<H: MemEnv>(h: &mut H, vm: u32, _ctx: u32) -> u32 {
    let r = read_u8(h, vm);
    let v = read_u8(h, vm);
    let a = reg_addr(h, vm, r);
    h.w32(a, v as u32);
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

/// C: `sub_021597ac`: command 38: var += value (u16 wrap).
pub fn cmd_var_add<H: CoreHooks>(h: &mut H, vm: u32, ctx: u32) -> u32 {
    let a = h.fun_02159ae8(vm, ctx);
    let v = h.fun_02159b10(vm, ctx);
    let x = h.r16(a);
    h.w16(a, x.wrapping_add(v as u16));
    0
}

#[cfg(test)]
mod tests;
