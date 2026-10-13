//! F8 (fourth batch): the overlay-21 script commands without a `__FILE__` anchor: F1-SPLIT rows 21.91 (Thumb
//! 0x021CF7DC-0x021D0140) and 21.97 (0x021D1670-0x021D30B8): distinct-OT counting, the 20-record save block 0x02009470, an OBJ
//! animation proc, field events, lottery / reward numbers, the camera shake of command 328 and the overlay-0x021F3xxx calls
//! (35 command ids, 81 functions; 11 render / UI-flow functions are left out, see the spec header).
//!
//! Every logic function of the area is specced in `tools/rom-diff/specs/black-script/scrcmd_misc21.spec`. The functions listed
//! in [`PORTED`] are ported here and checked with `tools/check-fn.sh black-script scrcmd_misc21`; the rest are the F8 port
//! job's.

use crate::hooks::MemEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_FA1C,
    0x021C_FCF4,
    0x021C_FD88,
    0x021D_1988,
    0x021D_21C0,
];

/// Capacity of the id list of command 569 ({u32 count, u32 ids[0x32]}).
pub const OT_LIST_MAX: u32 = 0x32;
/// Matches kept by `FUN_021CFC80` (the record indices at R+8..).
pub const MATCH_MAX: u32 = 5;
/// The keys of the 21 `{u32 key, u32 flag}` records at 0x021DD614 (overlay 21 ROM).
pub const TABLE_DD614_KEYS: [u32; 21] = [
    0x68A, 0x690, 0x691, 0x692, 0x693, 0x6D5, 0x6D6, 0x6D7, 0x6D8, 0x6E3, 0x6E4, 0x71E, 0x71F,
    0x720, 0x743, 0x746, 0x756, 0x76E, 0x7EF, 0x77E, 0x77F,
];
/// Animation rotation (fx32 degrees) -> OBJ direction (`FUN_0216D60C`), `FUN_021D1988`.
pub const ROT_DIRS: [(u32, u32); 4] = [(0, 1), (0x5A000, 3), (0xB4000, 0), (0x10E000, 2)];

/// Callees of the scrcmd_misc21 functions that this area does not port, named by address (meaning per the spec).
pub trait Misc21Hooks: MemEnv {
    /// C: `FUN_02011610(anime, &t)` (main): the animation's translation (VecFx32).
    fn fun_02011610(&mut self, anime: u32, t: &mut [u8; 12]);
    /// C: `FUN_02011638(anime, &r)` (main): the animation's rotation (VecFx32; r.y = the angle).
    fn fun_02011638(&mut self, anime: u32, r: &mut [u8; 12]);
    /// C: `FUN_0207C9D0(a, b, out)` (main, VEC_Add): `out = a + b`.
    fn fun_0207c9d0(&mut self, a: &[u8; 12], b: &[u8; 12], out: &mut [u8; 12]);
    /// C: `FUN_0216D89C(mmdl, &pos)` (overlay-10 fldmmdl.c): set the object's position.
    fn fun_0216d89c(&mut self, mmdl: u32, pos: &[u8; 12]);
    /// C: `FUN_0216D60C(mmdl, dir)` (overlay-10 fldmmdl.c): set the object's direction.
    fn fun_0216d60c(&mut self, mmdl: u32, dir: u32);
}

/// C: `FUN_overlay_d_21__021cfa1c`: add `v` to the list {count, ids[0x32]} unless it is full or holds `v` already; 1 when added.
pub fn ot_list_add<H: MemEnv>(h: &mut H, v: u32, list: u32) -> bool {
    let n = h.r32(list) as i32;
    if n >= OT_LIST_MAX as i32 {
        return false;
    }
    let mut i = 0;
    while i < n {
        if v == h.r32(list + 4 + 4 * i as u32) {
            return false;
        }
        i += 1;
    }
    h.w32(list.wrapping_add(4 + 4 * i as u32), v);
    let n = h.r32(list);
    h.w32(list, n.wrapping_add(1));
    true
}

/// C: `FUN_overlay_d_21__021cfcf4`: the record index of match `i` (`R[8 + i]` for `i < 5` and `i < R[1]`), else 0xFF.
pub fn match_index<H: MemEnv>(h: &mut H, r: u32, i: u32) -> u8 {
    if i < MATCH_MAX && i < u32::from(h.r8(r + 1)) {
        h.r8(r + 8 + i)
    } else {
        0xFF
    }
}

/// C: `FUN_overlay_d_21__021cfd88`: the message id of a record: base 4 (`g` set) or 0x44, + 0xC m for m < 5, + 6 when `f`,
/// and + 1 + o % 5 when `x` is 0.
pub fn message_id(f: u32, o: u32, m: u32, x: u32, g: u32) -> u32 {
    let mut id: u32 = if g != 0 { 4 } else { 0x44 };
    if m < 5 {
        id += 0xC * m;
    }
    if f != 0 {
        id += 6;
    }
    if x == 0 {
        id += 1 + o % 5;
    }
    id
}

/// C: `FUN_overlay_d_21__021d1988`: move the OBJ `mmdl` to the animation's translation + `vec` (VecFx32 at `vec`) and turn it
/// to the direction of the animation's rotation (0 / 90 / 180 / 270 degrees -> 1 / 3 / 0 / 2; other angles: no turn).
pub fn obj_anim_apply<H: Misc21Hooks>(h: &mut H, anime: u32, mmdl: u32, vec: u32) {
    let mut t = [0u8; 12];
    h.fun_02011610(anime, &mut t);
    let mut r = [0u8; 12];
    h.fun_02011638(anime, &mut r);
    let mut v = [0u8; 12];
    for (k, b) in v.iter_mut().enumerate() {
        *b = h.r8(vec + k as u32);
    }
    let a = t;
    h.fun_0207c9d0(&a, &v, &mut t);
    h.fun_0216d89c(mmdl, &t);
    let angle = u32::from_le_bytes([r[4], r[5], r[6], r[7]]);
    if let Some(&(_, dir)) = ROT_DIRS.iter().find(|&&(a, _)| a == angle) {
        h.fun_0216d60c(mmdl, dir);
    }
}

/// C: `FUN_overlay_d_21__021d21c0`: `v` is one of the [`TABLE_DD614_KEYS`].
pub fn in_table_a(v: u32) -> bool {
    TABLE_DD614_KEYS.contains(&v)
}

#[cfg(test)]
mod tests;
