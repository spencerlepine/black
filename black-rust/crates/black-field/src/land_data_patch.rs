//! `land_data_patch.c` (overlay 21, `0x021C939C..0x021C94BC`): attribute / building patches applied to a map
//! block (ARC 156).
//!
//! F2a-spec specced all 6 functions (`tools/rom-diff/specs/black-field/land_data_patch.spec`, whose header
//! documents the patch file); F2a ported them. The patch object (`+0` -> file) and the blocks it patches are
//! read through the [`Mem`] window of the environment at their original addresses; heap, archive and
//! building-model callees go through [`FieldEnv::call`].
//!
//! PATCH FILE: `+0 u16` offset of the ATTRIBUTE section, `+2 u16` offset of the BUILDING section. ATTRIBUTE
//! section: `+0 u16` row width (cells), cells of 0x18 bytes from `+4`. BUILDING section: `+0 u32` count,
//! records of 16 bytes `{x, y, z, ..}` from `+4`. The cell and record meanings are UNVERIFIED (read from the
//! copy arithmetic only).

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_939C,
    0x021C_93D0,
    0x021C_93E4,
    0x021C_9450,
    0x021C_94AC,
    0x021C_94B4,
];

/// Address of the string `"land_data_patch.c"` (the file argument of the allocation in `FUN_021C939C`).
pub const FILE_LAND_DATA_PATCH: u32 = 0x021D_E558;
/// Archive of the patch files (ARC 156).
pub const PATCH_ARC: u32 = 0x9C;
/// Size of one attribute cell.
pub const CELL: u32 = 0x18;
/// C: `FUN_0204A1BC(arc, member, flag, heap)`: load an archive member into a new heap block (returns it).
const LOAD_MEMBER: u32 = 0x0204_A1BC;
/// C: `FUN_0218BA04(a, b, record, index)` (field_buildmodel, F2b): place one building record.
const PLACE_BUILDING: u32 = 0x0218_BA04;

/// C: `FUN_021C939C` (unlabeled routine at `0x021C939C`): create a patch object for ARC 156 member `id`:
/// `p = Heap_AllocDebug(heap, 4, 1, "land_data_patch.c", 0x4F)`, `p+0 = FUN_0204A1BC(0x9C, id, 0, heap)`;
/// returns `p`.
pub fn create<E: FieldEnv>(env: &mut E, id: u32, heap: u32) -> u32 {
    let p = env.call(
        HEAP_ALLOC,
        &mut [
            CArg::V(heap),
            CArg::V(4),
            CArg::V(1),
            CArg::V(FILE_LAND_DATA_PATCH),
            CArg::V(0x4F),
        ],
    );
    let file = env.call(
        LOAD_MEMBER,
        &mut [CArg::V(PATCH_ARC), CArg::V(id), CArg::V(0), CArg::V(heap)],
    );
    env.w32(p, file);
    p
}

/// C: `FUN_021C93D0` (unlabeled routine at `0x021C93D0`): free the file and the object.
pub fn free<E: FieldEnv>(env: &mut E, p: u32) {
    let file = env.r32(p);
    env.call(HEAP_FREE, &mut [CArg::V(file)]);
    env.call(HEAP_FREE, &mut [CArg::V(p)]);
}

/// C: `FUN_021C93E4` (unlabeled routine at `0x021C93E4`): copy a `w` x `h` cell rectangle of the attribute
/// section, from `(sx, sy)`, into the attribute block `dst` (same layout) at `(dx, dy)`, one row per
/// `MI_CpuCopy8`. Both row widths are re-read for every row (as the original does); no bounds checks.
#[allow(clippy::too_many_arguments)]
pub fn copy_attributes<E: FieldEnv>(
    env: &mut E,
    p: u32,
    dst: u32,
    sx: u32,
    sy: u32,
    dx: u32,
    dy: u32,
    w: u32,
    h: u32,
) {
    let src = attr_section(env.mem(), p);
    let len = w.wrapping_mul(CELL);
    let mut i = 0u32;
    while i < h {
        let sw = env.r16(src) as u32;
        let from = src.wrapping_add(4).wrapping_add(
            sx.wrapping_add(sy.wrapping_add(i).wrapping_mul(sw))
                .wrapping_mul(CELL),
        );
        let dw = env.r16(dst) as u32;
        let to = dst.wrapping_add(4).wrapping_add(
            dx.wrapping_add(dy.wrapping_add(i).wrapping_mul(dw))
                .wrapping_mul(CELL),
        );
        env.mem().copy(to, from, len);
        i += 1;
    }
}

/// C: `FUN_021C9450` (unlabeled routine at `0x021C9450`): hand every building record of the patch to
/// `FUN_0218BA04(a, b, &rec, base + i)` as a copy with `x += dx << 16` and `z -= dz << 16` (fx32 tiles);
/// returns `base + count` (the next free index).
pub fn place_buildings<E: FieldEnv>(
    env: &mut E,
    p: u32,
    a: u32,
    b: u32,
    base: u32,
    dx: u32,
    dz: u32,
) -> u32 {
    let sec = build_section(env.mem(), p);
    let recs = sec.wrapping_add(4);
    let mut i = 0u32;
    while i < env.r32(sec) {
        let mut rec = [0u8; 16];
        for (j, v) in rec.iter_mut().enumerate() {
            *v = env.r8(recs.wrapping_add(i << 4).wrapping_add(j as u32));
        }
        let x = u32::from_le_bytes(rec[0..4].try_into().unwrap()).wrapping_add(dx << 16);
        rec[0..4].copy_from_slice(&x.to_le_bytes());
        let z = u32::from_le_bytes(rec[8..12].try_into().unwrap()).wrapping_sub(dz << 16);
        rec[8..12].copy_from_slice(&z.to_le_bytes());
        env.call(
            PLACE_BUILDING,
            &mut [
                CArg::V(a),
                CArg::V(b),
                CArg::InBuf(&rec),
                CArg::V(base.wrapping_add(i)),
            ],
        );
        i += 1;
    }
    base.wrapping_add(env.r32(sec))
}

/// C: `FUN_021C94AC` (unlabeled routine at `0x021C94AC`): address of the attribute section of the patch
/// file: `file + u16 file[0]` (`file` = `*p`).
pub fn attr_section(mem: &Mem, p: u32) -> u32 {
    let file = mem.r32(p);
    file.wrapping_add(mem.r16(file) as u32)
}

/// C: `FUN_021C94B4` (unlabeled routine at `0x021C94B4`): address of the building section of the patch
/// file: `file + u16 file[2]`.
pub fn build_section(mem: &Mem, p: u32) -> u32 {
    let file = mem.r32(p);
    file.wrapping_add(mem.r16(file.wrapping_add(2)) as u32)
}

#[cfg(test)]
mod tests;
