//! F7: the script interpreter (`vm.c`, main ARM9 0x020111EC-0x020113D8).
//!
//! Every function is specced in `tools/rom-diff/specs/black-script/vm.spec` (its header documents the VM layout and the
//! step loop `0x02011298`); this file holds the ported ones, checked with `tools/check-fn.sh black-script vm`.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0201_1330, 0x0201_137C];

/// VM field offsets (see the spec header).
pub const VM_STACK_WORDS: u32 = 0x00;
pub const VM_DEPTH: u32 = 0x0C;
pub const VM_PC: u32 = 0x14;
pub const VM_STACK: u32 = 0x18;

/// C: `FUN_02011330`: the next little-endian u16 of the script at the VM's PC; PC += 2.
pub fn read_u16<M: MemEnv>(m: &mut M, vm: u32) -> u32 {
    let pc = m.r32(vm + VM_PC);
    let lo = m.r8(pc) as u32;
    m.w32(vm + VM_PC, pc.wrapping_add(1));
    let hi = m.r8(pc.wrapping_add(1)) as u32;
    m.w32(vm + VM_PC, pc.wrapping_add(2));
    (lo + (hi << 8)) & 0xFFFF
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

#[cfg(test)]
mod tests;
