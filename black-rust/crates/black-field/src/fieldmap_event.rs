//! `fieldmap.c`, second part (overlay 21, `0x02189BD0..0x0218B752`): the field event checks (talk, step,
//! push, bump, signs, warps, hidden items, special tiles) and the 0x40-byte EVENT REQUEST they share.
//!
//! F2a-spec specced all 52 functions (`tools/rom-diff/specs/black-field/fieldmap_event.spec`, whose header
//! documents the EVENT REQUEST); this module holds the smoke port. Records are read through a [`Mem`]
//! window at their original addresses.

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[0x0218_AE48];

/// EVENT REQUEST: `+0x24 u16` facing direction (0 up, 1 down, 2 left, 3 right).
pub const REQ_FACING: u32 = 0x24;
/// EVENT REQUEST: `+0x34` pointer to the player position (`fx32[3]`).
pub const REQ_POS: u32 = 0x34;

/// C: `FUN_0218AE48` (unlabeled routine at `0x0218AE48`): the tile in front of the player: `out` = the
/// three words of `*(req + 0x34)`, then by the facing direction `z -= 1.0` (0), `z += 1.0` (1),
/// `x -= 1.0` (2), `x += 1.0` (3) in fx32 (`0x10000`); other directions leave the position.
pub fn front_position(mem: &mut Mem, req: u32, out: u32) {
    // the original's order (`ldm` 2 words, `stm` them, then the third), so an overlap behaves the same
    let p = mem.r32(req + REQ_POS);
    let (x, y) = (mem.r32(p), mem.r32(p + 4));
    mem.w32(out, x);
    mem.w32(out + 4, y);
    let z = mem.r32(p + 8);
    mem.w32(out + 8, z);
    let step = |mem: &mut Mem, at: u32, up: bool| {
        let v = mem.r32(at);
        mem.w32(
            at,
            if up {
                v.wrapping_add(0x10000)
            } else {
                v.wrapping_sub(0x10000)
            },
        );
    };
    match mem.r16(req + REQ_FACING) {
        0 => step(mem, out + 8, false),
        1 => step(mem, out + 8, true),
        2 => step(mem, out, false),
        3 => step(mem, out, true),
        _ => {}
    }
}

#[cfg(test)]
mod tests;
