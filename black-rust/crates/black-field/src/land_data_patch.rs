//! `land_data_patch.c` (overlay 21, `0x021C939C..0x021C94BC`): attribute / building patches applied to a map
//! block (ARC 156).
//!
//! F2a-spec specced all 6 functions (`tools/rom-diff/specs/black-field/land_data_patch.spec`, whose header
//! documents the patch file); this module holds the smoke port. The patch object (`+0` -> file) is read
//! through a [`Mem`] window at its original address.

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x021C_94AC];

/// C: `FUN_021C94AC` (unlabeled routine at `0x021C94AC`): address of the attribute section of the patch
/// file: `file + u16 file[0]` (`file` = `*p`).
pub fn attr_section(mem: &Mem, p: u32) -> u32 {
    let file = mem.r32(p);
    file.wrapping_add(mem.r16(file) as u32)
}

#[cfg(test)]
mod tests;
