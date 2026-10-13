//! `fldmmdl.c` (overlay 10, `0x0216CC58..0x0216E62C`): the field map-model objects (MMDL: NPCs and the player
//! object) and their manager (MMDLSYS): create / delete from 0x24-byte headers, position, direction, move code,
//! status flags, the hooks into the movement and draw procs, and the save-position work.
//!
//! F4a-spec specced all 253 functions of the area (`tools/rom-diff/specs/black-field/fldmmdl.spec`, whose header
//! documents the MMDLSYS (0x48 bytes), MMDL (0x100 bytes), HEADER (0x24 bytes) and save-position layouts; summary in
//! the crate README); this module holds the smoke ports. Records are read through a [`Mem`] window at their
//! original addresses. The draw side (blact / 3D objects) and the movement procs are reached only through the
//! proc tables (`+0x8C` / `+0x90`) and overlay-21 callees: render side (D3, G8/U2) and F4b. Planned next to this
//! module: `fldmmdl_acmd.c` 0x02197D3C-0x0219C552 (overlay 21, 452 fns, movement-type tables at 0x021D4160..; F4b) and
//! `event_trainer_eye.c` 0x021AE454 (F4c); `fldmmdl_blact.c` / `fldmmdl_g3dobj.c` drawing is U2.
//!
//! MMDLSYS: `+0x04 u16` MMDL count, `+0x18` -> `MMDL[count]` (0x100 each). MMDL: `+0x00 u32` status (bit 0 = in
//! use), `+0x08 u16` OBJ id, `+0x0A u16` zone id, `+0x18 u16` display dir, `+0x88` -> MMDLSYS.

use crate::hooks::Mem;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_D4F0,
    0x0216_D55C,
    0x0216_D5F8,
    0x0216_D67C,
    0x0216_DD1C,
    0x0216_E4B0,
];

/// Size of an MMDL record.
pub const MMDL_SIZE: u32 = 0x100;
/// Size of the MMDLSYS record.
pub const SYS_SIZE: u32 = 0x48;
/// Size of a MMDL HEADER (the 36-byte NPC record of ARC 125).
pub const HEADER_SIZE: u32 = 0x24;
/// MMDL status bit: the object is in use.
pub const ST_IN_USE: u32 = 1;
/// "No save slot" answer of [`save_slot`] (and the slot count of the save-position work).
pub const NO_SLOT: u16 = 0x1E;

/// C: `FUN_0216D4F0` (unlabeled routine at `0x0216D4F0`): the MMDL count of a MMDLSYS (`u16 +4`).
pub fn count(mem: &Mem, sys: u32) -> u16 {
    mem.r16(sys.wrapping_add(4))
}

/// C: `FUN_0216D55C` (unlabeled routine at `0x0216D55C`): `status (+0) & mask` of an MMDL.
pub fn status(mem: &Mem, m: u32, mask: u32) -> u32 {
    mem.r32(m) & mask
}

/// C: `FUN_0216D5F8` (unlabeled routine at `0x0216D5F8`): the display direction of an MMDL (`u16 +0x18`).
pub fn dir_disp(mem: &Mem, m: u32) -> u16 {
    mem.r16(m.wrapping_add(0x18))
}

/// C: `FUN_0216D67C` (unlabeled routine at `0x0216D67C`): the MMDLSYS an MMDL belongs to (`+0x88`).
pub fn sys_of(mem: &Mem, m: u32) -> u32 {
    mem.r32(m.wrapping_add(0x88))
}

/// C: `FUN_0216DD1C` (unlabeled routine at `0x0216DD1C`): the "next MMDL in use" iterator. From index `*idx`
/// (`u32` at `idx`) while it is below the count: the index moves on, and the first MMDL in use (status bit 0) is
/// stored to `*out`, returning `true`; none left: `false`. Callers loop `while next_in_use(sys, &m, &i)` with
/// `i = 0`. The index compare is unsigned (an index at or past the count returns `false` at once).
pub fn next_in_use(mem: &mut Mem, sys: u32, out: u32, idx: u32) -> bool {
    let n = count(mem, sys) as u32;
    if mem.r32(idx) >= n {
        return false;
    }
    let mut p = mem
        .r32(sys.wrapping_add(0x18))
        .wrapping_add(mem.r32(idx).wrapping_shl(8));
    loop {
        let i = mem.r32(idx).wrapping_add(1);
        mem.w32(idx, i);
        if status(mem, p, ST_IN_USE) != 0 {
            mem.w32(out, p);
            return true;
        }
        p = p.wrapping_add(MMDL_SIZE);
        if mem.r32(idx) >= n {
            return false;
        }
    }
}

/// ROM table `0x02172A70`: `(zone, OBJ id, save slot)` of the objects whose position is kept in the
/// save-position work (OBJ codes 0x6D and 0x2003..0x2005), then the terminator `(0xFFFF, 0xFFFF, 0x1E)`.
pub const SAVE_SLOTS: [(u16, u16, u16); 27] = [
    (0x13F, 2, 0),
    (0x14D, 0, 1),
    (0x14D, 1, 2),
    (0x14D, 2, 3),
    (0x14E, 0, 4),
    (0x14E, 1, 5),
    (0x14E, 2, 6),
    (0x14F, 0, 7),
    (0x14F, 1, 8),
    (0xE2, 1, 9),
    (0xE0, 1, 10),
    (0xD8, 1, 11),
    (0x172, 0, 12),
    (0x172, 1, 13),
    (0x17A, 0, 14),
    (0x17A, 1, 15),
    (0x183, 0, 16),
    (0x183, 1, 17),
    (0x98, 0xD, 18),
    (0xD0, 0, 19),
    (0xE7, 0, 20),
    (0xE7, 1, 21),
    (0xE7, 3, 22),
    (0xE7, 2, 23),
    (0x17F, 0xA, 24),
    (0x15A, 5, 25),
    (0xFFFF, 0xFFFF, 0x1E),
];

/// C: `FUN_0216E4B0` (unlabeled routine at `0x0216E4B0`): the save slot of the object `id` of
/// `zone` ([`SAVE_SLOTS`]), or [`NO_SLOT`]. Each entry is matched before the end test, so `(0xFFFF, 0xFFFF)`
/// matches the terminator (whose slot is 0x1E anyway). The arguments are compared as whole registers.
pub fn save_slot(zone: u32, id: u32) -> u16 {
    for &(z, i, slot) in SAVE_SLOTS.iter() {
        if z as u32 == zone && i as u32 == id {
            return slot;
        }
        if z == 0xFFFF || i == 0xFFFF {
            break;
        }
    }
    NO_SLOT
}

#[cfg(test)]
mod tests;
