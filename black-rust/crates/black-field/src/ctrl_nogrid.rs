//! `fieldmap_ctrl_nogrid_work.c` (overlay 21, `0x021A6CB8..0x021A6DC4`; F3b): the MAP CTRL of the free-movement (rail) maps
//! (the callbacks of the no-grid entry of the MAP CTRL table, ROM 0x021D8ADC) and its work.
//!
//! MAP CTRL no-grid work (8, `Heap_AllocDebug(heap, 8, 1, "fieldmap_ctrl_nogrid_work.c", 0x3E)`): `+0` the FIELD_PLAYER.
//! Create: the work, the player at the saved position / rail location, the player as the current RAIL WORK of the no-grid
//! mapper. Frame: pad trigger / held -> `021A3674` (the no-grid player frame through FIELD_PLAYER).
//!
//! Every callee (FIELDMAP getters, FIELD_PLAYER, GAMEDATA, the no-grid mapper, pad, heap) is reached through
//! [`FieldEnv::call`] by address. Verified by `tools/check-fn.sh black-field ctrl_nogrid --per-function` (spec
//! `tools/rom-diff/specs/black-field/ctrl_nogrid.spec`).

use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_6CB8,
    0x021A_6D30,
    0x021A_6D3C,
    0x021A_6D48,
    0x021A_6D58,
    0x021A_6D9C,
    0x021A_6DA4,
    0x021A_6DC0,
];

/// Address of the string `"fieldmap_ctrl_nogrid_work.c"`.
pub const FILE_CTRL_NOGRID: u32 = 0x021D_DF1C;

pub(crate) const FM_CAMERA: u32 = 0x0218_8C58;
pub(crate) const FM_MAPPER: u32 = 0x0218_8C5C;
pub(crate) const FM_GAMESYS: u32 = 0x0218_8C80;
pub(crate) const FM_HEAP: u32 = 0x0218_8C84;
pub(crate) const FM_CTRL_WORK: u32 = 0x0218_8CA4;
pub(crate) const FM_SET_CTRL_WORK: u32 = 0x0218_8CAC;
pub(crate) const FM_PLAYER: u32 = 0x0218_8CB4;
pub(crate) const FM_SET_TARGET: u32 = 0x0218_8D40;
pub(crate) const PL_MMDL: u32 = 0x021A_3074;
pub(crate) const M_POS_PTR: u32 = 0x0216_D888;
pub(crate) const PAD_TRIG: u32 = 0x0203_62DC;
pub(crate) const PAD_HELD: u32 = 0x0203_630C;

/// C: `FUN_021A6CB8(fm, pos, x)`: create the MAP CTRL work and set the player up at `pos` / the saved rail location, facing
/// `x`; the camera target follows the player's MMDL.
pub fn create<E: FieldEnv>(env: &mut E, fm: u32, pos: u32, x: u32) {
    env.callv(FM_CAMERA, &[fm]);
    env.callv(FM_MAPPER, &[fm]);
    let pl = env.callv(FM_PLAYER, &[fm]);
    let heap = env.callv(FM_HEAP, &[fm]);
    let w = work_new(env, fm, heap);
    env.callv(FM_SET_CTRL_WORK, &[fm, w]);
    let m = env.callv(PL_MMDL, &[pl]);
    let p = env.callv(M_POS_PTR, &[m]);
    env.callv(FM_SET_TARGET, &[fm, p]);
    let gs = env.callv(FM_GAMESYS, &[fm]);
    let gd = env.callv(0x0201_20F4, &[gs]);
    let rec = env.callv(0x0201_27A4, &[gd]);
    let loc = env.callv(0x0201_2AB8, &[rec]);
    env.callv(0x021A_36B0, &[pl, loc]);
    env.call(0x021A_36D4, &mut [CArg::V(pl), CArg::In(pos, 12)]);
    env.call(0x021A_3044, &mut [CArg::V(pl), CArg::In(pos, 12)]);
    env.callv(0x021A_305C, &[pl, x]);
}

/// C: `FUN_021A6D30(fm)`: free the MAP CTRL work.
pub fn delete<E: FieldEnv>(env: &mut E, fm: u32) {
    let w = env.callv(FM_CTRL_WORK, &[fm]);
    work_free(env, w);
}

/// C: `FUN_021A6D3C(fm)`: one frame ([`work_frame`]).
pub fn frame<E: FieldEnv>(env: &mut E, fm: u32) {
    let w = env.callv(FM_CTRL_WORK, &[fm]);
    work_frame(env, w);
}

/// C: `FUN_021A6D48(fm)`: the player MMDL's position (pointer).
pub fn pos<E: FieldEnv>(env: &mut E, fm: u32) -> u32 {
    let pl = env.callv(FM_PLAYER, &[fm]);
    let m = env.callv(PL_MMDL, &[pl]);
    env.callv(M_POS_PTR, &[m])
}

/// C: `FUN_021A6D58(fm, heap)`: a new work (the FIELD_PLAYER); the player's no-grid part is created and its rail work
/// becomes the mapper's current one.
pub fn work_new<E: FieldEnv>(env: &mut E, fm: u32, heap: u32) -> u32 {
    let mp = env.callv(FM_MAPPER, &[fm]);
    let w = env.callv(HEAP_ALLOC, &[heap, 8, 1, FILE_CTRL_NOGRID, 0x3E]);
    let pl = env.callv(FM_PLAYER, &[fm]);
    env.w32(w, pl);
    env.callv(0x021A_3664, &[pl, heap]);
    let rw = env.callv(0x021A_36A4, &[pl]);
    env.callv(0x021B_B3AC, &[mp, rw]);
    w
}

/// C: `FUN_021A6D9C(w)`: free (veneer to Heap_Free).
pub fn work_free<E: FieldEnv>(env: &mut E, w: u32) {
    env.callv(HEAP_FREE, &[w]);
}

/// C: `FUN_021A6DA4(w)`: one frame: pad trigger / held to the player (`021A3674`).
pub fn work_frame<E: FieldEnv>(env: &mut E, w: u32) {
    let t = env.callv(PAD_TRIG, &[]);
    let h = env.callv(PAD_HELD, &[]);
    let pl = env.r32(w);
    env.callv(0x021A_3674, &[pl, t, h]);
}

/// C: `FUN_021A6DC0(w)`: the FIELD_PLAYER.
pub fn work_player<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    env.r32(w)
}

#[cfg(test)]
mod tests;
