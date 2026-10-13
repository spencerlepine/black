//! `effect_encount.c` (overlay 21, `0x021AA77C..0x021AB0D4`, the logic part of F1 row 21.30): the effect encounters
//! (shaking grass, dust clouds, rippling water, bird shadows): the step trigger, the candidate search around the
//! player, the placement, the stepped-on check and the items of dust clouds and bird shadows. The effect objects
//! themselves (fldeff_encount.c, F1 row 21.38) are drawn by U2; this code reaches them only by address (D3).
//!
//! F5a-spec specced all 29 functions (`tools/rom-diff/specs/black-field/effect_encount.spec`, whose header
//! documents the trigger, the search window, the EFFECT ENCOUNT work (0x5C0) and the effect state in the ENCOUNT
//! WORK); F5a ported all of them.

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};
use black_core::softfloat;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_A77C,
    0x021A_A794,
    0x021A_A7AC,
    0x021A_A7CC,
    0x021A_A7DC,
    0x021A_A80C,
    0x021A_A8EC,
    0x021A_A900,
    0x021A_A944,
    0x021A_AA4C,
    0x021A_AABC,
    0x021A_AAF0,
    0x021A_AB1C,
    0x021A_AB48,
    0x021A_AB54,
    0x021A_AB74,
    0x021A_ABC4,
    0x021A_ABEC,
    0x021A_ABF4,
    0x021A_ACD4,
    0x021A_AE34,
    0x021A_AEC0,
    0x021A_AF54,
    0x021A_AFA0,
    0x021A_AFC0,
    0x021A_AFFC,
    0x021A_B018,
    0x021A_B070,
    0x021A_B0AC,
];

/// `FUN_02005728(max)`: the global LCG's next value scaled below `max` (`MathRandContext32::next_scaled`).
const RAND_SCALED: u32 = 0x0200_5728;
/// GAMEDATA -> ENCOUNT WORK (0x20 bytes).
const ENCOUNT_WORK: u32 = 0x0201_2A38;
/// GAMEDATA -> map matrix.
const MAP_MATRIX: u32 = 0x0201_2974;
/// GAMEDATA getter (the argument of `0200C234`).
const GAMEDATA_F08: u32 = 0x0201_2F08;
/// `(GAMEDATA getter result, 0)` predicate: effect encounters on.
const EFFECTS_ON: u32 = 0x0200_C234;
/// map_matrix: the position is in bounds `(matrix, x, z)`.
const MATRIX_IN_BOUNDS: u32 = 0x0215_9C48;
/// map_matrix: the zone id at a position `(matrix, x, z)`.
const MATRIX_ZONE: u32 = 0x0215_9BD4;
/// FIELDMAP zone id.
const FIELDMAP_ZONE: u32 = 0x0218_8C9C;
/// FIELDMAP +0x8C (the field player).
const FIELDMAP_PLAYER: u32 = 0x0218_8CB4;
/// FIELDMAP -> field effect control.
const FIELDMAP_FX: u32 = 0x0218_8CD0;
/// FIELDMAP getter (map mode).
const FIELDMAP_MODE: u32 = 0x0218_8CFC;
/// FIELDMAP -> map mapper.
const FIELDMAP_MAPPER: u32 = 0x0218_8C98;
/// FIELDMAP -> MMDLSYS.
const FIELDMAP_MMDLSYS: u32 = 0x0218_8C7C;
/// FIELDMAP heap id.
const FIELDMAP_HEAP: u32 = 0x0218_8C84;
/// The field player -> its MMDL.
const PLAYER_MMDL: u32 = 0x021A_3074;
/// Field player predicate.
const PLAYER_3624: u32 = 0x021A_3624;
/// Field player call (after the predicate).
const PLAYER_3644: u32 = 0x021A_3644;
/// MMDL position VecFx32 -> out.
const MMDL_POS: u32 = 0x0216_D88C;
/// MMDL grid x / y / z (s16 x 3) -> out.
const MMDL_GRID: u32 = 0x0216_D828;
/// MMDL grid x.
const MMDL_GX: u32 = 0x0216_D83C;
/// MMDL grid z.
const MMDL_GZ: u32 = 0x0216_D86C;
/// MMDL in use.
const MMDL_IN_USE: u32 = 0x0216_D454;
/// MMDL iterator `(sys, &m, &i)`.
const MMDL_NEXT: u32 = 0x0216_DD1C;
/// MMDL OBJ data.
const MMDL_OBJ: u32 = 0x0216_E3C4;
/// mapper: block / grid handle.
const MAPPER_HANDLE: u32 = 0x0218_D168;
/// mapper: grid ready `(mapper, h)`.
const MAPPER_READY: u32 = 0x0218_DBD4;
/// mapper: attribute at a position `(mapper, h, &pos, &out)`.
const MAPPER_ATTR: u32 = 0x0218_DBAC;
/// map_attr `021AB504`: effect-encounter kind of an attribute word.
const ATTR_EFFECT_KIND: u32 = 0x021A_B504;
/// fldeff_encount: create the effect object `(enc, ctrl, x, z, height, kind)`.
const FLDEFF_CREATE: u32 = 0x021A_DAAC;
/// fldeff_encount: animation `(obj, v)`.
const FLDEFF_ANIM: u32 = 0x021A_DB4C;
/// fldeff_encount `(obj, v)`.
const FLDEFF_B5C: u32 = 0x021A_DB5C;
/// Field effect delete `(obj)`.
const FLDEFF_DELETE: u32 = 0x021A_C154;
/// field_encount `021A9F40(enc, kind)`: the wild encounter check.
const STEP_CHECK: u32 = 0x021A_9F40;
/// ov10 script call `(gsys, script, 0, heap)`.
const SCRIPT_CALL: u32 = 0x0215_8768;

/// Address of the string `"effect_encount.c"`.
pub const FILE_EFFECT_ENCOUNT: u32 = 0x021D_DFB4;
/// EFFECT ENCOUNT `+0x5B4`: the effect object.
pub const EFF_OBJECT: u32 = 0x5B4;

/// EFFECT ENCOUNT `+0x5B8 u16`: the step interval.
pub const EFF_INTERVAL: u32 = 0x5B8;
/// EFFECT ENCOUNT `+0x5BA u16`: the probability (per mille / 10).
pub const EFF_PROBABILITY: u32 = 0x5BA;
/// EFFECT ENCOUNT `+0x5BC u16`: the item of the last dust cloud / bird shadow.
pub const EFF_ITEM: u32 = 0x5BC;

/// ROM `0x021D8E70`: (interval, probability) per effect kind 0..2 (the table's byte 7 & 0x7F).
pub const KIND_PARAMS: [[u8; 2]; 3] = [[20, 10], [20, 10], [5, 15]];

/// C: `FUN_021AA7DC` (unlabeled routine at `0x021AA7DC`): the interval and probability of the zone's effect kind:
/// `k = table byte 7 & 0x7F` (`enc +0xC`), more than 2 counts as 0; `eff +0x5B8 / +0x5BA` = [`KIND_PARAMS`]`[k]`.
pub fn set_params(mem: &mut Mem, enc: u32, eff: u32) {
    let table = mem.r32(enc.wrapping_add(0xC));
    let mut k = (mem.r8(table.wrapping_add(7)) & 0x7F) as usize;
    if k > 2 {
        k = 0;
    }
    mem.w16(eff.wrapping_add(EFF_INTERVAL), KIND_PARAMS[k][0] as u16);
    mem.w16(eff.wrapping_add(EFF_PROBABILITY), KIND_PARAMS[k][1] as u16);
}

/// C: `FUN_overlay_d_21__021aab48` (unlabeled routine at `0x021AAB48`): the item of the last dust cloud / bird
/// shadow (`u16 +0x5BC` of the EFFECT ENCOUNT work `enc +0x10`).
pub fn item(mem: &Mem, enc: u32) -> u16 {
    mem.r16(mem.r32(enc.wrapping_add(0x10)).wrapping_add(EFF_ITEM))
}

/// C: `FUN_overlay_d_21__021aabc4` (unlabeled routine at `0x021AABC4`): count a step of the effect counter
/// (`ewk +0xC`, unless it is 0xFFFFFFFF) and tell whether it reached the interval `eff +0x5B8`.
pub fn step_due(mem: &mut Mem, ewk: u32, eff: u32) -> bool {
    let a = ewk.wrapping_add(0xC);
    let c = mem.r32(a);
    if c != u32::MAX {
        mem.w32(a, c + 1);
    }
    mem.r32(a) >= mem.r16(eff.wrapping_add(EFF_INTERVAL)) as u32
}

/// C: `FUN_overlay_d_21__021aabec` (unlabeled routine at `0x021AABEC`): restart the effect counter (`ewk +0xC = 0`).
pub fn reset_counter(mem: &mut Mem, ewk: u32) {
    mem.w32(ewk.wrapping_add(0xC), 0);
}

/// C: `FUN_overlay_d_21__021ab070` (unlabeled routine at `0x021AB070`): the item of a bird shadow:
/// `FUN_02005728(1000) < 900` (as u16): a wing `565 + FUN_02005728(600) / 100` (565..570), else 571.
pub fn shadow_item<E: FieldEnv>(env: &mut E) -> u32 {
    if env.callv(RAND_SCALED, &[1000]) & 0xFFFF < 900 {
        let w = (env.callv(RAND_SCALED, &[600]) / 100) & 0xFFFF;
        (w + 0x235) & 0xFFFF
    } else {
        0x23B
    }
}

/// The ENCOUNT WORK of the FIELD ENCOUNT object (`FUN_02012A38(enc +8)`).
fn encount_work<E: FieldEnv>(env: &mut E, enc: u32) -> u32 {
    let gd = env.r32(enc.wrapping_add(8));
    env.callv(ENCOUNT_WORK, &[gd])
}

/// The effect is active (low nibble of the state byte `ewk +0x1D`).
fn active<E: FieldEnv>(env: &mut E, ewk: u32) -> bool {
    env.r8(ewk.wrapping_add(0x1D)) & 0xF != 0
}

/// C: `FUN_overlay_d_21__021aa77c` (unlabeled routine at `0x021AA77C`): allocate the EFFECT ENCOUNT work
/// (`Heap_AllocDebug(heap, 0x5C0, 1, "effect_encount.c", 0x6E)`).
pub fn create<E: FieldEnv>(env: &mut E, heap: u32) -> u32 {
    env.callv(HEAP_ALLOC, &[heap, 0x5C0, 1, FILE_EFFECT_ENCOUNT, 0x6E])
}

/// C: `FUN_overlay_d_21__021aa794` (unlabeled routine at `0x021AA794`): zero the EFFECT ENCOUNT work and free it.
pub fn delete<E: FieldEnv>(env: &mut E, eff: u32) {
    env.mem().fill(eff, 0x5C0, 0);
    env.callv(HEAP_FREE, &[eff]);
}

/// C: `FUN_overlay_d_21__021aa7ac` (unlabeled routine at `0x021AA7AC`): set up the EFFECT ENCOUNT work: `eff +0` = the
/// field effect control of the FIELDMAP (`FUN_02188CD0`), [`set_params`], [`restore`].
pub fn init<E: FieldEnv>(env: &mut E, enc: u32, eff: u32) {
    let fm = env.r32(enc);
    let ctrl = env.callv(FIELDMAP_FX, &[fm]);
    env.w32(eff, ctrl);
    set_params(env.mem(), enc, eff);
    restore(env, enc, eff);
}

/// C: `FUN_overlay_d_21__021aa7cc` (unlabeled routine at `0x021AA7CC`): tear down: [`mark_restore`], then
/// [`delete_object`].
pub fn exit<E: FieldEnv>(env: &mut E, enc: u32, eff: u32) {
    mark_restore(env, enc);
    delete_object(env, eff);
}

/// C: `FUN_overlay_d_21__021aa80c` (unlabeled routine at `0x021AA80C`): the step trigger of the effect encounters: the
/// ENCOUNT TABLE must be loaded, the FIELDMAP and the map matrix (`FUN_02012974`) present, the player's position
/// (`FUN_0216D88C`, a VecFx32 local) in bounds (`FUN_02159C48`) and in the current zone (`FUN_02159BD4` equal to the
/// FIELDMAP zone, neither 0xFFFF), effect encounters on (`FUN_0200C234(FUN_02012F08(gd), 0)`), `FUN_02188CFC(fieldmap)`
/// 0 and no effect active; then [`step_due`]: `FUN_02005728(1000) < 10 * probability` searches the candidates
/// ([`search`]) and places one ([`place`]) when there are any; otherwise the counter restarts ([`reset_counter`]).
pub fn step<E: FieldEnv>(env: &mut E, enc: u32) {
    let eff = env.r32(enc.wrapping_add(0x10));
    let ewk = encount_work(env, enc);
    if enc == 0 {
        return;
    }
    let tab = env.r32(enc.wrapping_add(0xC));
    if tab == 0 || env.r8(tab.wrapping_add(7)) & 0x80 == 0 {
        return;
    }
    let gd = env.r32(enc.wrapping_add(8));
    let mm = env.callv(MAP_MATRIX, &[gd]);
    let fm = env.r32(enc);
    if fm == 0 || mm == 0 {
        return;
    }
    let zone = env.callv(FIELDMAP_ZONE, &[fm]);
    let fm = env.r32(enc);
    let player = env.callv(FIELDMAP_PLAYER, &[fm]);
    let m = env.callv(PLAYER_MMDL, &[player]);
    let pos = env.mem().push_local(12);
    env.call(MMDL_POS, &mut [CArg::V(m), CArg::Out(pos, 12)]);
    let (x, z) = (env.r32(pos), env.r32(pos.wrapping_add(8)));
    env.mem().pop_local(pos);
    if env.callv(MATRIX_IN_BOUNDS, &[mm, x, z]) == 0 {
        return;
    }
    let z2 = env.callv(MATRIX_ZONE, &[mm, x, z]);
    if zone == 0xFFFF || z2 == 0xFFFF || zone != z2 {
        return;
    }
    let gd = env.r32(enc.wrapping_add(8));
    let g = env.callv(GAMEDATA_F08, &[gd]);
    if env.callv(EFFECTS_ON, &[g, 0]) == 0 {
        return;
    }
    let fm = env.r32(enc);
    if env.callv(FIELDMAP_MODE, &[fm]) != 0 {
        return;
    }
    if active(env, ewk) {
        return;
    }
    if !step_due(env.mem(), ewk, eff) {
        return;
    }
    let r = env.callv(RAND_SCALED, &[1000]);
    let p = env.r16(eff.wrapping_add(EFF_PROBABILITY)) as u32;
    if r >= p.wrapping_mul(10) {
        reset_counter(env.mem(), ewk);
        return;
    }
    search(env, enc, eff);
    if env.r16(eff.wrapping_add(4)) != 0 {
        place(env, enc, eff);
    }
}

/// C: `FUN_overlay_d_21__021aa8ec` (unlabeled routine at `0x021AA8EC`): [`end_effect`]`(enc, enc +0x10)` (a tail jump).
pub fn end<E: FieldEnv>(env: &mut E, enc: u32) {
    let eff = env.r32(enc.wrapping_add(0x10));
    end_effect(env, enc, eff);
}

/// C: `FUN_overlay_d_21__021aa900` (unlabeled routine at `0x021AA900`): is the active effect at `pos` (three s16: x, y,
/// z; compared x, z, y with the state `+0x12` / `+0x16` / `+0x14`)?
pub fn at<E: FieldEnv>(env: &mut E, enc: u32, pos: u32) -> bool {
    let w = encount_work(env, enc);
    if !active(env, w) {
        return false;
    }
    env.r16(w.wrapping_add(0x12)) as i16 == env.r16(pos) as i16
        && env.r16(w.wrapping_add(0x16)) as i16 == env.r16(pos.wrapping_add(4)) as i16
        && env.r16(w.wrapping_add(0x14)) as i16 == env.r16(pos.wrapping_add(2)) as i16
}

/// The player's MMDL grid position (`FUN_0216D828` into a 6-byte local): (x, y, z) as s16.
fn player_grid<E: FieldEnv>(env: &mut E, enc: u32) -> (i16, i16, i16) {
    let fm = env.r32(enc);
    let player = env.callv(FIELDMAP_PLAYER, &[fm]);
    let m = env.callv(PLAYER_MMDL, &[player]);
    mmdl_grid(env, m)
}

fn mmdl_grid<E: FieldEnv>(env: &mut E, m: u32) -> (i16, i16, i16) {
    let g = env.mem().push_local(8);
    env.call(MMDL_GRID, &mut [CArg::V(m), CArg::Out(g, 6)]);
    let r = (
        env.r16(g) as i16,
        env.r16(g.wrapping_add(2)) as i16,
        env.r16(g.wrapping_add(4)) as i16,
    );
    env.mem().pop_local(g);
    r
}

/// C: `FUN_overlay_d_21__021aa944` (unlabeled routine at `0x021AA944`): the player stepped on the active effect (its grid
/// x / z / y equal the state's); returns the battle event or 0. `r = FUN_02005728(1000)` (u16): a bird shadow (kind 7)
/// with `r >= 200` gives [`shadow_item`], a dust cloud (kind 4) with `r >= 400` [`dust_item`], by the script
/// [`give_item`] (then the effect ends); else the kind-1 encounter `FUN_021A9F40(enc, 1)`: none ends the effect and
/// gives 0; a battle: kinds 5 / 6 [`anim_b`]`(enc, 1)`, state bits 6..7 = 1 (stepped on), `FUN_021A3644(player)` when
/// `FUN_021A3624(player)`.
pub fn stepped_on<E: FieldEnv>(env: &mut E, enc: u32) -> u32 {
    let eff = env.r32(enc.wrapping_add(0x10));
    let w = encount_work(env, enc);
    let s = w.wrapping_add(0x10);
    if !active(env, w) {
        return 0;
    }
    let (gx, gy, gz) = player_grid(env, enc);
    let on = env.r16(s.wrapping_add(2)) as i16 == gx
        && env.r16(s.wrapping_add(6)) as i16 == gz
        && env.r16(s.wrapping_add(4)) as i16 == gy;
    if !on {
        return 0;
    }
    let r = env.callv(RAND_SCALED, &[1000]) & 0xFFFF;
    let kind = env.r8(s.wrapping_add(0xC));
    let mut item_path = false;
    let ev;
    if kind == 7 && r >= 200 {
        let it = shadow_item(env);
        ev = give_item(env, enc, it);
        item_path = true;
    } else if kind == 4 && r >= 400 {
        let it = dust_item(env);
        ev = give_item(env, enc, it);
        item_path = true;
    } else {
        ev = env.callv(STEP_CHECK, &[enc, 1]);
    }
    if ev == 0 {
        end_effect(env, enc, eff);
        return 0;
    }
    if item_path {
        end_effect(env, enc, eff);
        return ev;
    }
    if env.r8(s.wrapping_add(0xC)).wrapping_sub(5) <= 1 {
        anim_b(env, enc, 1);
    }
    let b = env.r8(s.wrapping_add(0xD));
    env.w8(s.wrapping_add(0xD), (b & !0xC0) | 0x40);
    let fm = env.r32(enc);
    let player = env.callv(FIELDMAP_PLAYER, &[fm]);
    if env.callv(PLAYER_3624, &[player]) != 0 {
        env.callv(PLAYER_3644, &[player]);
    }
    ev
}

/// C: `FUN_overlay_d_21__021aaa4c` (unlabeled routine at `0x021AAA4C`): the distance of the player to the active effect:
/// `*d` (u16) = max(|state x - gx|, |state z - gz|) (16-bit differences), returns 1; no effect: `*d` = 0, 0.
pub fn distance<E: FieldEnv>(env: &mut E, enc: u32, d: u32) -> u32 {
    let w = encount_work(env, enc);
    let s = w.wrapping_add(0x10);
    if !active(env, w) {
        env.w16(d, 0);
        return 0;
    }
    let (gx, _, gz) = player_grid(env, enc);
    let mut dx = (env.r16(s.wrapping_add(2)) as i16 as i32).wrapping_sub(gx as i32) as i16 as i32;
    if dx < 0 {
        dx = dx.wrapping_mul(-1);
    }
    let mut dz = (env.r16(s.wrapping_add(6)) as i16 as i32).wrapping_sub(gz as i32) as i16 as i32;
    if dz < 0 {
        dz = dz.wrapping_mul(-1);
    }
    env.w16(d, if dx > dz { dx } else { dz } as u16);
    1
}

/// C: `FUN_overlay_d_21__021aaabc` (unlabeled routine at `0x021AAABC`): with an active effect: state bits 6..7 = 1 and,
/// with an effect object, `FUN_021ADB4C(obj, 1)`.
pub fn mark_stepped<E: FieldEnv>(env: &mut E, enc: u32) {
    let w = encount_work(env, enc);
    let s = w.wrapping_add(0x10);
    let b = env.r8(s.wrapping_add(0xD));
    if b & 0xF == 0 {
        return;
    }
    env.w8(s.wrapping_add(0xD), (b & !0xC0) | 0x40);
    let eff = env.r32(enc.wrapping_add(0x10));
    let obj = env.r32(eff.wrapping_add(EFF_OBJECT));
    if obj != 0 {
        env.callv(FLDEFF_ANIM, &[obj, 1]);
    }
}

/// C: `FUN_overlay_d_21__021aaaf0` (unlabeled routine at `0x021AAAF0`): with an active effect and an effect object:
/// `FUN_021ADB4C(obj, v)`.
pub fn anim_a<E: FieldEnv>(env: &mut E, enc: u32, v: u32) {
    let w = encount_work(env, enc);
    let eff = env.r32(enc.wrapping_add(0x10));
    if !active(env, w) {
        return;
    }
    let obj = env.r32(eff.wrapping_add(EFF_OBJECT));
    if obj != 0 {
        env.callv(FLDEFF_ANIM, &[obj, v]);
    }
}

/// C: `FUN_overlay_d_21__021aab1c` (unlabeled routine at `0x021AAB1C`): with an active effect and an effect object:
/// `FUN_021ADB5C(obj, v)`.
pub fn anim_b<E: FieldEnv>(env: &mut E, enc: u32, v: u32) {
    let w = encount_work(env, enc);
    let eff = env.r32(enc.wrapping_add(0x10));
    if !active(env, w) {
        return;
    }
    let obj = env.r32(eff.wrapping_add(EFF_OBJECT));
    if obj != 0 {
        env.callv(FLDEFF_B5C, &[obj, v]);
    }
}

/// C: `FUN_overlay_d_21__021aab54` (unlabeled routine at `0x021AAB54`): with an active effect: state bits 4..5 = 1 (the
/// effect object must be made again, [`restore`]).
pub fn mark_restore<E: FieldEnv>(env: &mut E, enc: u32) {
    let w = encount_work(env, enc);
    let s = w.wrapping_add(0x10);
    let b = env.r8(s.wrapping_add(0xD));
    if b & 0xF != 0 {
        env.w8(s.wrapping_add(0xD), (b & !0x30) | 0x10);
    }
}

/// C: `FUN_overlay_d_21__021aab74` (unlabeled routine at `0x021AAB74`): with an active effect marked by
/// [`mark_restore`]: clear the mark; in another zone (FIELDMAP zone != state zone) or stepped on (bits 6..7) the state
/// is cleared (0x10 bytes), else the effect object is made again ([`make_object`]).
pub fn restore<E: FieldEnv>(env: &mut E, enc: u32, eff: u32) {
    let w = encount_work(env, enc);
    let s = w.wrapping_add(0x10);
    let b = env.r8(s.wrapping_add(0xD));
    if b & 0xF == 0 || (b >> 4) & 3 == 0 {
        return;
    }
    env.w8(s.wrapping_add(0xD), b & !0x30);
    let fm = env.r32(enc);
    let zone = env.callv(FIELDMAP_ZONE, &[fm]);
    if zone != env.r16(s) as u32 || env.r8(s.wrapping_add(0xD)) >> 6 != 0 {
        env.mem().fill(s, 0x10, 0);
        return;
    }
    make_object(env, enc, eff, s);
}

/// Window offsets ROM `0x021D8E76` (5 x 4 signed bytes: x0, x1, z0, z1): entry 0 for kind 2, entries 1..4 the half
/// windows.
pub const WINDOW_OFFSETS: [[i8; 4]; 5] = [
    [-5, 5, -5, 5],
    [0, 5, -5, 5],
    [-5, 0, -5, 5],
    [-5, 5, -5, 0],
    [-5, 5, 0, 5],
];

/// C: `FUN_overlay_d_21__021aabf4` (unlabeled routine at `0x021AABF4`): the search window (12 s16 at `w`): `w[0]` /
/// `w[1]` = player gx / gz, `w[2]` / `w[3]` = their tile in the 32-tile block (signed remainder), `w[4]` / `w[5]` = the
/// block (signed quotient), `w[6]` / `w[7]` = the block origin; offsets [`WINDOW_OFFSETS`]`[0]` for kind 2, else
/// `[FUN_02005728(4) + 1]`; `x0 = max(w[2] + o[0], 0)`, `x1 = min(w[2] + o[1], 31)`, `z0` / `z1` likewise from `w[3]`;
/// `w[8]` / `w[9]` / `w[10]` / `w[11]` = origin + `x0` / `z0` / `x1` / `z1`. `eff` is not read.
pub fn window<E: FieldEnv>(env: &mut E, enc: u32, _eff: u32, w: u32) {
    let fm = env.r32(enc);
    let player = env.callv(FIELDMAP_PLAYER, &[fm]);
    let m = env.callv(PLAYER_MMDL, &[player]);
    let gx = env.callv(MMDL_GX, &[m]);
    env.w16(w, gx as u16);
    let gz = env.callv(MMDL_GZ, &[m]);
    env.w16(w.wrapping_add(2), gz as u16);
    let x = env.r16(w) as i16 as i32;
    env.w16(w.wrapping_add(4), (x % 32) as u16);
    let z = env.r16(w.wrapping_add(2)) as i16 as i32;
    env.w16(w.wrapping_add(6), (z % 32) as u16);
    env.w16(w.wrapping_add(8), (x / 32) as u16);
    env.w16(w.wrapping_add(0xA), (z / 32) as u16);
    let tx = env.r16(w.wrapping_add(4)) as i16 as i32;
    env.w16(w.wrapping_add(0xC), x.wrapping_sub(tx) as u16);
    let tz = env.r16(w.wrapping_add(6)) as i16 as i32;
    env.w16(w.wrapping_add(0xE), z.wrapping_sub(tz) as u16);
    let tab = env.r32(enc.wrapping_add(0xC));
    let o = if env.r8(tab.wrapping_add(7)) & 0x7F != 2 {
        let r = env.callv(RAND_SCALED, &[4]);
        WINDOW_OFFSETS[r.wrapping_add(1) as usize]
    } else {
        WINDOW_OFFSETS[0]
    };
    let tx = env.r16(w.wrapping_add(4)) as i16 as i32;
    let mut x0 = tx.wrapping_add(o[0] as i32) as i16 as i32;
    if x0 < 0 {
        x0 = 0;
    }
    let mut x1 = tx.wrapping_add(o[1] as i32) as i16 as i32;
    if x1 >= 0x20 {
        x1 = 0x1F;
    }
    let tz = env.r16(w.wrapping_add(6)) as i16 as i32;
    let mut z0 = tz.wrapping_add(o[2] as i32) as i16 as i32;
    if z0 < 0 {
        z0 = 0;
    }
    let mut z1 = tz.wrapping_add(o[3] as i32) as i16 as i32;
    if z1 >= 0x20 {
        z1 = 0x1F;
    }
    let ox = env.r16(w.wrapping_add(0xC)) as i32;
    env.w16(w.wrapping_add(0x10), x0.wrapping_add(ox) as u16);
    env.w16(w.wrapping_add(0x14), x1.wrapping_add(ox) as u16);
    let oz = env.r16(w.wrapping_add(0xE)) as i32;
    env.w16(w.wrapping_add(0x12), z0.wrapping_add(oz) as u16);
    env.w16(w.wrapping_add(0x16), z1.wrapping_add(oz) as u16);
}

/// The fx32 centre of tile `t` as the original computes it with the soft-float helpers: `(t << 16)` as a single,
/// `+ 0.5` (`t << 4 > 0`) or `- 0.5`, converted back (toward zero), `+ 0x8000`.
fn tile_centre(t: i32) -> u32 {
    let f = softfloat::i2f(t.wrapping_shl(16));
    let g = if t.wrapping_shl(4) > 0 {
        softfloat::fadd(0x3F00_0000, f)
    } else {
        softfloat::fsub(f, 0x3F00_0000)
    };
    (softfloat::f2i(g) as u32).wrapping_add(0x8000)
}

/// C: `FUN_overlay_d_21__021aacd4` (unlabeled routine at `0x021AACD4`): find the candidates: zero the candidate count
/// and list (`eff +4`, 0x5B0 bytes); with a ready grid (`FUN_0218DBD4(mapper, FUN_0218D168(mapper))`) and the
/// [`window`], every tile `z = w[9]..=w[11]`, `x = w[8]..=w[10]` (positions: [`tile_centre`], stepping by 0x10000) whose
/// attribute (`FUN_0218DBAC(mapper, h, &pos, &out)`, `out +8`) has an effect kind (`021AB504` != 8) is appended: kind,
/// x, z (u16) and the height `out +0xC`.
pub fn search<E: FieldEnv>(env: &mut E, enc: u32, eff: u32) {
    let fm = env.r32(enc);
    let mapper = env.callv(FIELDMAP_MAPPER, &[fm]);
    env.mem().fill(eff.wrapping_add(4), 0x5B0, 0);
    let pos = env.mem().push_local(12);
    let h = env.callv(MAPPER_HANDLE, &[mapper]);
    if env.callv(MAPPER_READY, &[mapper, h]) == 0 {
        env.mem().pop_local(pos);
        return;
    }
    let out = env.mem().push_local(0x10);
    let w = env.mem().push_local(0x18);
    window(env, enc, eff, w);
    let mut z = env.r16(w.wrapping_add(0x12)) as i16 as i32;
    env.w32(pos.wrapping_add(8), tile_centre(z));
    let x_start = tile_centre(env.r16(w.wrapping_add(0x10)) as i16 as i32);
    while z <= env.r16(w.wrapping_add(0x16)) as i16 as i32 {
        env.w32(pos, x_start);
        let mut x = env.r16(w.wrapping_add(0x10)) as i16 as i32;
        while x <= env.r16(w.wrapping_add(0x14)) as i16 as i32 {
            let z16 = z as u32 & 0xFFFF;
            env.call(
                MAPPER_ATTR,
                &mut [
                    CArg::V(mapper),
                    CArg::V(h),
                    CArg::In(pos, 12),
                    CArg::Out(out, 0x10),
                ],
            );
            let a = env.r32(out.wrapping_add(8));
            let k = env.callv(ATTR_EFFECT_KIND, &[a]);
            if k != 8 {
                let n = env.r16(eff.wrapping_add(4)) as u32;
                let c = eff.wrapping_add(0xC * n);
                env.w8(c.wrapping_add(8), k as u8);
                env.w16(c.wrapping_add(0xA), x as u16);
                env.w16(c.wrapping_add(0xE), z16 as u16);
                env.w16(eff.wrapping_add(4), (n + 1) as u16);
                let ht = env.r32(out.wrapping_add(0xC));
                env.w32(c.wrapping_add(0x10), ht);
            }
            let px = env.r32(pos);
            env.w32(pos, px.wrapping_add(0x10000));
            x = (x + 1) as i16 as i32;
        }
        let pz = env.r32(pos.wrapping_add(8));
        env.w32(pos.wrapping_add(8), pz.wrapping_add(0x10000));
        z = (z + 1) as i16 as i32;
    }
    env.mem().pop_local(w);
    env.mem().pop_local(out);
    env.mem().pop_local(pos);
}

/// C: `FUN_overlay_d_21__021aae34` (unlabeled routine at `0x021AAE34`): place the effect on a random candidate
/// (`FUN_02005728(count)`, u16): its y (`+4`) = height / 16 / 4096 (each rounded toward zero); when no MMDL stands there
/// ([`mmdl_on`] with the player included is 0) the state becomes (FIELDMAP zone, x, y, z, height, kind), active, the
/// counter restarts ([`reset_counter`]) and the object is made ([`make_object`]).
pub fn place<E: FieldEnv>(env: &mut E, enc: u32, eff: u32) {
    let n = env.r16(eff.wrapping_add(4)) as u32;
    let k = env.callv(RAND_SCALED, &[n]) & 0xFFFF;
    let c = eff.wrapping_add(8).wrapping_add(0xC * k);
    let w = encount_work(env, enc);
    let h = (env.r32(c.wrapping_add(8)) as i32) >> 4;
    env.w16(c.wrapping_add(4), (h / 4096) as u16);
    let x = env.r16(c.wrapping_add(2)) as i16 as i32 as u32;
    let y = env.r16(c.wrapping_add(4)) as i16 as i32 as u32;
    let z = env.r16(c.wrapping_add(6)) as i16 as i32 as u32;
    let s = w.wrapping_add(0x10);
    if mmdl_on(env, enc, x, y, z, 0) {
        return;
    }
    for o in [2, 4, 6] {
        let v = env.r16(c.wrapping_add(o));
        env.w16(s.wrapping_add(o), v);
    }
    let ht = env.r32(c.wrapping_add(8));
    env.w32(s.wrapping_add(8), ht);
    let kind = env.r8(c);
    env.w8(s.wrapping_add(0xC), kind);
    let fm = env.r32(enc);
    let zone = env.callv(FIELDMAP_ZONE, &[fm]);
    env.w16(s, zone as u16);
    let b = env.r8(s.wrapping_add(0xD));
    env.w8(s.wrapping_add(0xD), (b & !0xF) | 1);
    reset_counter(env.mem(), w);
    make_object(env, enc, eff, s);
}

/// C: `FUN_overlay_d_21__021aaec0` (unlabeled routine at `0x021AAEC0`): does an MMDL in use stand on `(x, y, z)` (the
/// fifth argument `skip_player`, a stack word, leaves the player out)? Every MMDL of the iterator `FUN_0216DD1C(sys, &m,
/// &i)` (locals) in use (`FUN_0216D454`): OBJ data `d` (`FUN_0216E3C4`), grid `(gx, gy, gz)` (`FUN_0216D828`); `y ==
/// gy`, `gx <= x < gx + d[0xB]`, `gz - d[0xC] < z <= gz` (signed).
pub fn mmdl_on<E: FieldEnv>(
    env: &mut E,
    enc: u32,
    x: u32,
    y: u32,
    z: u32,
    skip_player: u32,
) -> bool {
    let fm = env.r32(enc);
    let player = env.callv(FIELDMAP_PLAYER, &[fm]);
    let pm = env.callv(PLAYER_MMDL, &[player]);
    let fm = env.r32(enc);
    let sys = env.callv(FIELDMAP_MMDLSYS, &[fm]);
    let i = env.mem().push_local(4);
    let mp = env.mem().push_local(4);
    let found = loop {
        let more = env.call(MMDL_NEXT, &mut [CArg::V(sys), CArg::Out(mp, 4), CArg::P(i)]);
        if more == 0 {
            break false;
        }
        let m = env.r32(mp);
        if env.callv(MMDL_IN_USE, &[m]) == 0 {
            continue;
        }
        if m == pm && skip_player != 0 {
            continue;
        }
        let d = env.callv(MMDL_OBJ, &[m]);
        let (gx, gy, gz) = mmdl_grid(env, m);
        if y as i32 != gy as i32 {
            continue;
        }
        if (x as i32) < gx as i32 {
            continue;
        }
        if x as i32 >= (gx as i32).wrapping_add(env.r8(d.wrapping_add(0xB)) as i32) {
            continue;
        }
        if z as i32 > gz as i32 {
            continue;
        }
        if z as i32 <= (gz as i32).wrapping_sub(env.r8(d.wrapping_add(0xC)) as i32) {
            continue;
        }
        break true;
    };
    env.mem().pop_local(mp);
    env.mem().pop_local(i);
    found
}

/// C: `FUN_overlay_d_21__021aaf54` (unlabeled routine at `0x021AAF54`): an MMDL other than the player on the active
/// effect ([`mmdl_on`]) ends it ([`end_effect`]) and gives 1; else 0.
pub fn mmdl_on_effect<E: FieldEnv>(env: &mut E, enc: u32) -> u32 {
    let w = encount_work(env, enc);
    let s = w.wrapping_add(0x10);
    if !active(env, w) {
        return 0;
    }
    let x = env.r16(s.wrapping_add(2)) as i16 as i32 as u32;
    let y = env.r16(s.wrapping_add(4)) as i16 as i32 as u32;
    let z = env.r16(s.wrapping_add(6)) as i16 as i32 as u32;
    if mmdl_on(env, enc, x, y, z, 1) {
        let eff = env.r32(enc.wrapping_add(0x10));
        end_effect(env, enc, eff);
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_21__021aafa0` (unlabeled routine at `0x021AAFA0`): end the effect: [`delete_object`], zero the
/// state (0x10 bytes at `ewk +0x10`).
pub fn end_effect<E: FieldEnv>(env: &mut E, enc: u32, eff: u32) {
    let w = encount_work(env, enc);
    delete_object(env, eff);
    env.mem().fill(w.wrapping_add(0x10), 0x10, 0);
}

/// C: `FUN_overlay_d_21__021aafc0` (unlabeled routine at `0x021AAFC0`): without an effect object, make it:
/// `FUN_021ADAAC(enc, eff +0, rec x (u16 +2), z (u16 +6), height +8, kind +0xC)`; when that fails (0) the state `rec`
/// is zeroed (0x10 bytes).
pub fn make_object<E: FieldEnv>(env: &mut E, enc: u32, eff: u32, rec: u32) {
    if env.r32(eff.wrapping_add(EFF_OBJECT)) != 0 {
        return;
    }
    let ht = env.r32(rec.wrapping_add(8));
    let kind = env.r8(rec.wrapping_add(0xC)) as u32;
    let x = env.r16(rec.wrapping_add(2)) as u32;
    let z = env.r16(rec.wrapping_add(6)) as u32;
    let ctrl = env.r32(eff);
    let obj = env.callv(FLDEFF_CREATE, &[enc, ctrl, x, z, ht, kind]);
    env.w32(eff.wrapping_add(EFF_OBJECT), obj);
    if obj == 0 {
        env.mem().fill(rec, 0x10, 0);
    }
}

/// C: `FUN_overlay_d_21__021aaffc` (unlabeled routine at `0x021AAFFC`): delete the effect object (`FUN_021AC154`) if
/// there is one and clear `eff +0x5B4`.
pub fn delete_object<E: FieldEnv>(env: &mut E, eff: u32) {
    let obj = env.r32(eff.wrapping_add(EFF_OBJECT));
    if obj != 0 {
        env.callv(FLDEFF_DELETE, &[obj]);
        env.w32(eff.wrapping_add(EFF_OBJECT), 0);
    }
}

/// ROM `0x021D8E8A`: the rare dust-cloud items (10 u16: evolution stones 80..85, gems 107..110).
pub const DUST_RARE: [u16; 10] = [80, 81, 82, 83, 84, 85, 107, 108, 109, 110];

/// C: `FUN_overlay_d_21__021ab018` (unlabeled routine at `0x021AB018`): the item of a dust cloud: `r =
/// FUN_02005728(1000)` (u16): below 100 [`DUST_RARE`]`[FUN_02005728(1000) / 100]`; below 950 `548 + FUN_02005728(1700) /
/// 100` (548..564); else 229 (Everstone).
pub fn dust_item<E: FieldEnv>(env: &mut E) -> u32 {
    let r = env.callv(RAND_SCALED, &[1000]) & 0xFFFF;
    if r < 100 {
        let k = (env.callv(RAND_SCALED, &[1000]) / 100) & 0xFFFF;
        return DUST_RARE[k as usize] as u32;
    }
    if r < 950 {
        let k = (env.callv(RAND_SCALED, &[0x6A4]) / 100) & 0xFFFF;
        return (k + 0x224) & 0xFFFF;
    }
    0xE5
}

/// C: `FUN_overlay_d_21__021ab0ac` (unlabeled routine at `0x021AB0AC`): give `item` by script: `eff +0x5BC` = item, then
/// `FUN_02158768(gsys, 0x2795, 0, FIELDMAP heap)` (returns its result, the event).
pub fn give_item<E: FieldEnv>(env: &mut E, enc: u32, item: u32) -> u32 {
    let eff = env.r32(enc.wrapping_add(0x10));
    env.w16(eff.wrapping_add(EFF_ITEM), item as u16);
    let fm = env.r32(enc);
    let heap = env.callv(FIELDMAP_HEAP, &[fm]);
    let gsys = env.r32(enc.wrapping_add(4));
    env.callv(SCRIPT_CALL, &[gsys, 0x2795, 0, heap])
}

#[cfg(test)]
mod tests;
