//! `field_buildmodel.c` (overlay 21, `0x0218B754..0x0218C9E8`, F1 row 21.01; F2b): the map building models of a zone. A BM
//! loads the zone's model list (one 0x24-byte RECORD per model kind) with its texture and animation resources, keeps a 400-byte
//! map model id -> list index, places the models of each map block (0x120 placed-model ENTRIES: a field_g3d_map handle, an
//! index and the placed model's state word), runs the models' animations (4 kinds by the record's animation header: none, loop
//! all, scripted states, by time of day), and makes DOOR objects (7 slots) for placed models found by position, with their open /
//! close animations and sounds.
//!
//! Logic only (D3): the G3D resources and objects (g3d_system.c), field_g3d_map.c / field_g3d_mapper.c, the FX matrix / vector
//! library and the sound player are reached through [`FieldEnv::call`] by address; `FX_SinCosTable_` is computed
//! ([`crate::field_light::fx_sin`], [`crate::fieldmap::fx_cos`]).
//!
//! Spec: `tools/rom-diff/specs/black-field/buildmodel.spec` (its header has the BM / LIST / RECORD / SET / OBJECT / DOOR
//! layouts). Verified by `tools/check-fn.sh black-field buildmodel --per-function`.

use crate::field_light::fx_sin;
use crate::fieldmap::fx_cos;
use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0218_B754,
    0x0218_B7F8,
    0x0218_B83C,
    0x0218_B88C,
    0x0218_B8AC,
    0x0218_B910,
    0x0218_B924,
    0x0218_B930,
    0x0218_B974,
    0x0218_B9FC,
    0x0218_BA04,
    0x0218_BA48,
    0x0218_BA70,
    0x0218_BA98,
    0x0218_BACC,
    0x0218_BB04,
    0x0218_BB0C,
    0x0218_BB64,
    0x0218_BB78,
    0x0218_BBD0,
    0x0218_BBF4,
    0x0218_BC00,
    0x0218_BC10,
    0x0218_BC1C,
    0x0218_BC40,
    0x0218_BC6C,
    0x0218_BC8C,
    0x0218_BCAC,
    0x0218_BCB0,
    0x0218_BCE4,
    0x0218_BCF4,
    0x0218_BD20,
    0x0218_BD24,
    0x0218_BD28,
    0x0218_BD94,
    0x0218_BDD8,
    0x0218_BE84,
    0x0218_BEE8,
    0x0218_BF84,
    0x0218_BFB4,
    0x0218_C02C,
    0x0218_C040,
    0x0218_C08C,
    0x0218_C0D8,
    0x0218_C124,
    0x0218_C150,
    0x0218_C154,
    0x0218_C1A4,
    0x0218_C1C4,
    0x0218_C23C,
    0x0218_C290,
    0x0218_C2E4,
    0x0218_C350,
    0x0218_C388,
    0x0218_C3B0,
    0x0218_C3E0,
    0x0218_C438,
    0x0218_C43C,
    0x0218_C47C,
    0x0218_C490,
    0x0218_C4A0,
    0x0218_C4EC,
    0x0218_C518,
    0x0218_C530,
    0x0218_C54C,
    0x0218_C574,
    0x0218_C584,
    0x0218_C638,
    0x0218_C654,
    0x0218_C674,
    0x0218_C69C,
    0x0218_C6D0,
    0x0218_C6F4,
    0x0218_C724,
    0x0218_C7D8,
    0x0218_C7F0,
    0x0218_C860,
    0x0218_C88C,
    0x0218_C89C,
    0x0218_C8B0,
    0x0218_C8D8,
    0x0218_C8FC,
    0x0218_C910,
    0x0218_C924,
    0x0218_C934,
    0x0218_C944,
    0x0218_C990,
    0x0218_C9A4,
    0x0218_C9C4,
];

/// Address of the string `"field_buildmodel.c"` (the file argument of this file's allocations).
pub const FILE_BUILDMODEL: u32 = 0x021D_D9CC;
/// Size of the BM object.
pub const BM_SIZE: u32 = 0x13F8;
/// BM fields.
pub const BM_HEAP: u32 = 0x0;
pub const BM_G: u32 = 0x4;
pub const BM_TOD: u32 = 0x8;
pub const BM_LIST: u32 = 0x18;
pub const BM_MAP: u32 = 0x1C;
pub const BM_COUNT: u32 = 0x1AC;
pub const BM_S: u32 = 0x1B0;
pub const BM_NSETS: u32 = 0x1C8;
pub const BM_SETS: u32 = 0x1CC;
pub const BM_TEX: u32 = 0x1D0;
pub const BM_NOBJS: u32 = 0x1D4;
pub const BM_OBJS: u32 = 0x1D8;
pub const BM_DOORS: u32 = 0x1DC;
pub const BM_ENTRIES: u32 = 0x1F8;
/// Number of door slots, placed-model entries, map bytes.
pub const N_DOORS: u32 = 7;
pub const N_ENTRIES: u32 = 0x120;
pub const MAP_LEN: u32 = 400;
/// The map byte of a model id without a list entry.
pub const MAP_NONE: u8 = 0x80;
/// Size of an animation resource SET / an animation OBJECT.
pub const SET_SIZE: u32 = 0x18;
/// Size of a DOOR object.
pub const DOOR_SIZE: u32 = 0x60;
/// Door slot codes of [`door_code`] / [`door_by_code`]: `DOOR_CODE | slot`.
pub const DOOR_CODE: u32 = 0xFFF0;

/// ROM `0x021D3D50`: the animation index of time zones 0..7.
pub const TOD_ANIM: [u8; 8] = [0, 1, 2, 3, 3, 0, 0, 0];
/// ROM `0x021D3DC4`: the door kind of model kinds 0..15.
pub const DOOR_KIND: [u8; 16] = [0, 1, 1, 1, 2, 0, 3, 4, 5, 6, 7, 8, 9, 1, 1, 1];
/// ROM `0x021D3D88`: per door model kind the sounds of the door states 0..3.
pub const DOOR_SE: [(u16, [u16; 4]); 6] = [
    (1, [0x685, 0x686, 0, 0]),
    (2, [0x687, 0x687, 0, 0]),
    (5, [0x687, 0x687, 0, 0]),
    (0xD, [0x776, 0x776, 0, 0]),
    (0xE, [0x77A, 0x77B, 0, 0]),
    (0xF, [0x687, 0x687, 0, 0]),
];

/// g3d_system.c (main; D3): texture resource / upload / release / delete, animation resource size / init, model object, animation,
/// animated object, its animations (count, i, delete, model), delete object / model, animation on / rewind / advance / advance
/// once / set frame / off / frame, the animation resource of an animation, draw.
const G3D_TEX_NEW: u32 = 0x0204_6DB8;
const G3D_TEX_UPLOAD: u32 = 0x0204_70A8;
const G3D_TEX_RELEASE: u32 = 0x0204_70D0;
const G3D_TEX_DELETE: u32 = 0x0204_6EFC;
const G3D_ANMRES_SIZE: u32 = 0x0204_6C94;
const G3D_ANMRES_INIT: u32 = 0x0204_6CE0;
const G3D_MODEL_NEW: u32 = 0x0204_7400;
const G3D_ANIM_NEW: u32 = 0x0204_7564;
const G3D_OBJ_NEW: u32 = 0x0204_767C;
const G3D_OBJ_COUNT: u32 = 0x0204_779C;
const G3D_OBJ_ANIM: u32 = 0x0204_7790;
const G3D_ANIM_DELETE: u32 = 0x0204_762C;
const G3D_OBJ_MODEL: u32 = 0x0204_7788;
const G3D_OBJ_DELETE: u32 = 0x0204_7738;
const G3D_MODEL_DELETE: u32 = 0x0204_7504;
const G3D_ANIM_ON: u32 = 0x0204_77A4;
const G3D_ANIM_REWIND: u32 = 0x0204_7804;
const G3D_ANIM_ADVANCE: u32 = 0x0204_7910;
const G3D_ANIM_ADVANCE_ONCE: u32 = 0x0204_78B8;
const G3D_ANIM_SET_FRAME: u32 = 0x0204_784C;
const G3D_ANIM_RES: u32 = 0x0204_7674;
const G3D_ANIM_OFF: u32 = 0x0204_77D4;
const G3D_ANIM_FRAME: u32 = 0x0204_7828;
const G3D_DRAW: u32 = 0x0204_7A60;
/// Archives: open, member count, load a member, close.
const ARC_OPEN: u32 = 0x0204_90F4;
const ARC_COUNT: u32 = 0x0204_961C;
const ARC_LOAD: u32 = 0x0204_9250;
const ARC_CLOSE: u32 = 0x0204_9238;
/// field_g3d_map.c (ov10, G8): place a model, its state word, remove it, its block position.
const MAP_PLACE: u32 = 0x0216_93D4;
const MAP_STATE: u32 = 0x0216_9444;
const MAP_REMOVE: u32 = 0x0216_940C;
const MAP_POSITION: u32 = 0x0216_9510;
/// field_g3d_mapper.c (G8): the texture hook, the map offset.
const MAPPER_TEX: u32 = 0x0218_D0D8;
const MAPPER_OFFSET: u32 = 0x0218_DC5C;
/// FX library: VEC_Add, MTX_RotY33.
const VEC_ADD: u32 = 0x0207_C9D0;
const MTX_ROT_Y33: u32 = 0x0207_AF1C;
/// The time zone of a season (main).
const TIME_ZONE: u32 = 0x0201_13D8;
/// zonedata: the model set of a zone; the archive choice predicate.
const ZONE_MODEL_SET: u32 = 0x0201_3B38;
const ARC_CHOICE: u32 = 0x0201_40D8;
/// Sound: the SE player of an SE, play.
const SE_PLAYER: u32 = 0x0200_6148;
const SE_PLAY: u32 = 0x0200_6224;

fn heap<E: FieldEnv>(env: &mut E, bm: u32) -> u32 {
    env.r16(bm + BM_HEAP) as u32
}

/// C: `FUN_0218B754(heap, g, season)`: a new BM (0x13F8 bytes on `heap`): `+4 = g`, the time of day of `season`, no doors, all
/// entries free, no lists (`+0x1D0` is left as allocated).
pub fn new<E: FieldEnv>(env: &mut E, heap: u32, g: u32, season: u32) -> u32 {
    let bm = env.callv(HEAP_ALLOC, &[heap, BM_SIZE, 0, FILE_BUILDMODEL, 0x193]);
    env.w16(bm + BM_HEAP, heap as u16);
    env.w32(bm + BM_G, g);
    tod_init(env, bm + BM_TOD, season & 0xFF);
    for i in 0..N_DOORS {
        env.w32(bm + BM_DOORS + 4 * i, 0);
    }
    for i in 0..N_ENTRIES {
        entry_clear(env, bm + BM_ENTRIES + 16 * i);
    }
    for o in [
        0x1B0u32, 0x1B4, 0x1B8, 0x1BC, 0x1C0, 0x1C4, 0x1C8, 0x1CC, 0x1D4, 0x1D8,
    ] {
        env.w32(bm + o, 0);
    }
    bm
}

/// C: `FUN_0218B7F8(bm)`: delete the doors, the animation objects and resources, the model list and the textures; free the BM.
pub fn delete<E: FieldEnv>(env: &mut E, bm: u32) {
    for i in 0..N_DOORS {
        let d = env.r32(bm + BM_DOORS + 4 * i);
        if d != 0 {
            door_delete(env, d);
        }
    }
    free_objs(env, bm, bm + BM_S);
    free_sets(env, bm);
    free_list(env, bm);
    free_textures(env, bm);
    env.callv(HEAP_FREE, &[bm]);
}

/// C: `FUN_0218B83C(bm)`: one frame: the time of day, then a frame of every door's and every model's animation object.
pub fn update<E: FieldEnv>(env: &mut E, bm: u32) {
    tod_update(env, bm + BM_TOD);
    for i in 0..N_DOORS {
        let d = env.r32(bm + BM_DOORS + 4 * i);
        if d != 0 {
            obj_frame(env, bm, d + 0xC);
        }
    }
    let mut i = 0u32;
    while i < env.r32(bm + BM_NOBJS) {
        let o = env.r32(bm + BM_OBJS).wrapping_add(i.wrapping_mul(SET_SIZE));
        obj_frame(env, bm, o);
        i += 1;
    }
}

/// C: `FUN_0218B88C(bm)`: draw every door.
pub fn draw<E: FieldEnv>(env: &mut E, bm: u32) {
    for i in 0..N_DOORS {
        let d = env.r32(bm + BM_DOORS + 4 * i);
        if d != 0 {
            door_draw(env, d);
        }
    }
}

/// C: `FUN_0218B8AC(bm, zone, x, h2)`: load the zone's models: the model set of the zone ([`member_of`]`(02013B38(zone))`) picks
/// the members, `020140D8(x)` the archives ((0xE5, 0xB0) with map key 0x15, else (0xE6, 0xB1) with 0x14): the model list, the
/// textures, the id map, the animation resources and objects.
pub fn load<E: FieldEnv>(env: &mut E, bm: u32, zone: u32, x: u32, h2: u32) {
    let set = env.callv(ZONE_MODEL_SET, &[zone]);
    let n = member_of(set);
    let (arc1, arc2, k) = if env.callv(ARC_CHOICE, &[x]) != 0 {
        (0xE5, 0xB0, 0x15)
    } else {
        (0xE6, 0xB1, 0x14)
    };
    load_list(env, bm, arc1, n);
    load_textures(env, bm, arc2, n, h2);
    build_map(env, bm, k);
    load_sets(env, bm, h2);
    make_objs(env, bm, bm + BM_S);
}

/// C: `FUN_0218B910(bm, x)`: the record of model id `x`.
pub fn record_of<E: FieldEnv>(env: &mut E, bm: u32, x: u32) -> u32 {
    let v = index_of(env, bm, x);
    let l = env.r32(bm + BM_LIST);
    list_entry(env, l, v)
}

/// C: `FUN_0218B924(bm, i)` (a thunk to [`list_entry`] on the model list): record `i`.
pub fn record<E: FieldEnv>(env: &mut E, bm: u32, i: u32) -> u32 {
    let l = env.r32(bm + BM_LIST);
    list_entry(env, l, i)
}

/// The linked model of model id `x`: `None` when its record has none, else (its list index, its offset).
fn linked_vals<E: FieldEnv>(env: &mut E, bm: u32, x: u32) -> Option<(u32, [u32; 3])> {
    let r = record_of(env, bm, x);
    let l = env.r16(r + 4) as u32;
    if l == 0xFFFF {
        return None;
    }
    let idx = index_of(env, bm, l);
    let off = [6u32, 8, 0xA].map(|o| ((env.r16(r + o) as i16 as i32) << 12) as u32);
    Some((idx, off))
}

/// C: `FUN_0218B930(bm, x, pos, idx)`: 1 when model id `x` has a linked model (`*idx` = its list index, `pos` = its offset), else 0.
pub fn linked<E: FieldEnv>(env: &mut E, bm: u32, x: u32, pos: u32, idx: u32) -> u32 {
    match linked_vals(env, bm, x) {
        None => 0,
        Some((i, p)) => {
            env.w32(idx, i);
            env.w32(pos + 8, p[2]);
            env.w32(pos, p[0]);
            env.w32(pos + 4, p[1]);
            1
        }
    }
}

/// The big-endian model id at `+0xE` of a block entry.
fn entry_id<E: FieldEnv>(env: &mut E, ent: u32) -> u32 {
    (env.r8(ent + 0xF) as u32 + ((env.r8(ent + 0xE) as u32) << 8)) & 0xFFFF
}

/// C: `FUN_0218B974(bm, h, list, n)`: place the `n` models of a map block (16-byte entries: x, y, z, u16 rotation, big-endian u16
/// model id), each followed by its linked model (at the model's position plus the link offset); answers the number of placements
/// (counting a linked model twice).
pub fn place_block<E: FieldEnv>(env: &mut E, bm: u32, h: u32, list: u32, n: u32) -> u32 {
    let mut c = 0u32;
    let mut j = 0u32;
    if (n as i32) > 0 {
        loop {
            let ent = list.wrapping_add(j << 4);
            place_one(env, bm, h, ent, c);
            let id = entry_id(env, ent);
            if let Some((idx, p)) = linked_vals(env, bm, id) {
                let rot = env.r16(ent + 0xC);
                c += 1;
                let x = p[0].wrapping_add(env.r32(ent));
                let y = p[1].wrapping_add(env.r32(ent + 4));
                let z = p[2].wrapping_sub(env.r32(ent + 8));
                let d = env.mem().push_local(0x14);
                env.w32(d, idx);
                env.w32(d + 4, x);
                env.w32(d + 8, y);
                env.w32(d + 0xC, z);
                env.w16(d + 0x10, rot);
                place(env, bm, h, d, c);
                env.mem().pop_local(d);
            }
            c += 1;
            j += 1;
            if (j as i32) >= (n as i32) {
                break;
            }
        }
    }
    c
}

/// C: `FUN_0218B9FC(bm, h)`: a thunk to [`remove`].
pub fn remove_thunk<E: FieldEnv>(env: &mut E, bm: u32, h: u32) {
    remove(env, bm, h);
}

/// C: `FUN_0218BA04(bm, h, ent, c)`: place the model of block entry `ent` as placement `c` (z negated); answers the entry.
pub fn place_one<E: FieldEnv>(env: &mut E, bm: u32, h: u32, ent: u32, c: u32) -> u32 {
    let id = entry_id(env, ent);
    let idx = index_of(env, bm, id);
    let x = env.r32(ent);
    let y = env.r32(ent + 4);
    let z = 0u32.wrapping_sub(env.r32(ent + 8));
    let rot = env.r16(ent + 0xC);
    let d = env.mem().push_local(0x14);
    env.w32(d, idx);
    env.w32(d + 4, x);
    env.w32(d + 8, y);
    env.w32(d + 0xC, z);
    env.w16(d + 0x10, rot);
    let r = place(env, bm, h, d, c);
    env.mem().pop_local(d);
    r
}

/// C: `FUN_0218BA48(bm, d)`: clear the door slot holding `d`.
pub fn door_slot_clear<E: FieldEnv>(env: &mut E, bm: u32, d: u32) {
    for i in 0..N_DOORS {
        if env.r32(bm + BM_DOORS + 4 * i) == d {
            env.w32(bm + BM_DOORS + 4 * i, 0);
            return;
        }
    }
}

/// C: `FUN_0218BA70(bm, d)`: put `d` in the first free door slot (nothing when all are taken).
pub fn door_slot_put<E: FieldEnv>(env: &mut E, bm: u32, d: u32) {
    for i in 0..N_DOORS {
        if env.r32(bm + BM_DOORS + 4 * i) == 0 {
            env.w32(bm + BM_DOORS + 4 * i, d);
            return;
        }
    }
}

/// C: `FUN_0218BA98(bm, code)`: the door of slot code `0xFFF0 | i` (`i < 7`), else 0.
pub fn door_by_code<E: FieldEnv>(env: &mut E, bm: u32, code: u32) -> u32 {
    let i = code & 0xF;
    if code & 0xFFF0 != DOOR_CODE {
        return 0;
    }
    if i >= N_DOORS {
        return 0;
    }
    env.r32(bm + BM_DOORS + 4 * i)
}

/// C: `FUN_0218BACC(bm, d)`: the slot code `0xFFF0 | i` of door `d`, 0 when `d` is null or not in a slot.
pub fn door_code<E: FieldEnv>(env: &mut E, bm: u32, d: u32) -> u32 {
    if d == 0 {
        return 0;
    }
    for i in 0..N_DOORS {
        if env.r32(bm + BM_DOORS + 4 * i) == d {
            return DOOR_CODE | i;
        }
    }
    0
}

/// C: `FUN_0218BB04(bm)`: the address of `S` (`bm + 0x1B0`).
pub fn s_ptr(bm: u32) -> u32 {
    bm + BM_S
}

/// C: `FUN_0218BB0C(bm, arc, member)`: load member `member` (0 when past the archive's count) of archive `arc` as the model list;
/// the model count is the list's byte 2 / 2 (0x80 when above 0x190).
pub fn load_list<E: FieldEnv>(env: &mut E, bm: u32, arc: u32, member: u32) {
    let hp = heap(env, bm);
    let h = env.callv(ARC_OPEN, &[arc, (hp & 0x7FFF) | 0x8000]);
    let mut m = member;
    if env.callv(ARC_COUNT, &[h]) <= m {
        m = 0;
    }
    let hp = heap(env, bm);
    let l = env.callv(ARC_LOAD, &[h, m, hp]);
    env.w32(bm + BM_LIST, l);
    let c = (env.r8(l + 2) >> 1) as u16;
    env.w16(bm + BM_COUNT, c);
    if env.r16(bm + BM_COUNT) > 0x190 {
        env.w16(bm + BM_COUNT, 0x80);
    }
    env.callv(ARC_CLOSE, &[h]);
}

/// C: `FUN_0218BB64(bm)`: free the model list; no models.
pub fn free_list<E: FieldEnv>(env: &mut E, bm: u32) {
    let l = env.r32(bm + BM_LIST);
    env.callv(HEAP_FREE, &[l]);
    env.w16(bm + BM_COUNT, 0);
}

/// C: `FUN_0218BB78(bm, k)`: build the id map: `map[record(i).id] = i` for every model (u8 `i`), and the ids without a model map
/// to `map[k]`.
pub fn build_map<E: FieldEnv>(env: &mut E, bm: u32, k: u32) {
    env.mem().fill(bm + BM_MAP, MAP_LEN, MAP_NONE);
    let mut i = 0u32;
    if env.r16(bm + BM_COUNT) as i32 > 0 {
        loop {
            let r = record(env, bm, i);
            let id = env.r16(r) as u32;
            env.w8(bm + id + BM_MAP, i as u8);
            i = (i + 1) & 0xFF;
            if (i as i32) >= env.r16(bm + BM_COUNT) as i32 {
                break;
            }
        }
    }
    let v = env.r8(bm + k + BM_MAP);
    for j in 0..MAP_LEN {
        if env.r8(bm + BM_MAP + j) == MAP_NONE {
            env.w8(bm + BM_MAP + j, v);
        }
    }
}

/// C: `FUN_0218BBD0(bm, x)`: the list index of model id `x` (`x >= 400` reads id 0; 0 when the map byte is 0x80.. or past the
/// model count).
pub fn index_of<E: FieldEnv>(env: &mut E, bm: u32, x: u32) -> u32 {
    let x = if x >= MAP_LEN { 0 } else { x };
    let v = env.r8(bm + x + BM_MAP) as u32;
    if v >= 0x80 || (v as i32) >= env.r16(bm + BM_COUNT) as i32 {
        return 0;
    }
    v
}

/// C: `FUN_0218BBF4(list, i)`: entry `i` of a list (`list + offset[i]`).
pub fn list_entry<E: FieldEnv>(env: &mut E, list: u32, i: u32) -> u32 {
    let o = env.r32(list.wrapping_add(i << 2).wrapping_add(4));
    list.wrapping_add(o)
}

/// C: `FUN_0218BC00(list, i)`: the animation data of model `i` (the offsets after the records).
pub fn list_entry2<E: FieldEnv>(env: &mut E, list: u32, i: u32) -> u32 {
    let n = (env.r8(list + 2) >> 1) as u32;
    let o = env.r32(list.wrapping_add((n.wrapping_add(i)) << 2).wrapping_add(4));
    list.wrapping_add(o)
}

/// C: `FUN_0218BC10(hdr, i)`: animation `i` of an animation header (`hdr + offset[i]`).
pub fn hdr_entry<E: FieldEnv>(env: &mut E, hdr: u32, i: u32) -> u32 {
    let o = env.r32(hdr.wrapping_add(i << 2).wrapping_add(4));
    hdr.wrapping_add(o)
}

/// C: `FUN_0218BC1C(a)`: the archive member of model set `a`: `a - 210` from 210 on, `(a - 2) / 4` from 2 on, else 0.
pub fn member_of(a: u32) -> u32 {
    if a >= 210 {
        return a.wrapping_sub(210) & 0xFFFF;
    }
    if a < 2 {
        return 0;
    }
    let r = a.wrapping_sub(2);
    let bias = ((r as i32 >> 1) as u32) >> 30;
    (r.wrapping_add(bias) << 14) >> 16
}

/// C: `FUN_0218BC40(bm, arc, n, h2)`: load the textures (member `n` of `arc`), hand them to field_g3d_mapper with `h2` (when set)
/// and upload them; answers the upload's result.
pub fn load_textures<E: FieldEnv>(env: &mut E, bm: u32, arc: u32, n: u32, h2: u32) -> u32 {
    let t = env.callv(G3D_TEX_NEW, &[arc, n]);
    env.w32(bm + BM_TEX, t);
    if h2 != 0 {
        env.callv(MAPPER_TEX, &[t, h2]);
    }
    let t = env.r32(bm + BM_TEX);
    env.callv(G3D_TEX_UPLOAD, &[t])
}

/// C: `FUN_0218BC6C(bm)`: release and delete the textures, if any.
pub fn free_textures<E: FieldEnv>(env: &mut E, bm: u32) {
    let t = env.r32(bm + BM_TEX);
    if t != 0 {
        env.callv(G3D_TEX_RELEASE, &[t]);
        let t = env.r32(bm + BM_TEX);
        env.callv(G3D_TEX_DELETE, &[t]);
        env.w32(bm + BM_TEX, 0);
    }
}

/// C: `FUN_0218BC8C(hdr)`: the number of animations of a header (offsets that are not -1).
pub fn hdr_count<E: FieldEnv>(env: &mut E, hdr: u32) -> u32 {
    (0..4u32)
        .filter(|&i| env.r32(hdr + 4 + 4 * i) != 0xFFFF_FFFF)
        .count() as u32
}

/// C: `FUN_0218BCAC(r)`: the animation header of a record (`r + 0x10`).
pub fn hdr_of(r: u32) -> u32 {
    r.wrapping_add(0x10)
}

/// C: `FUN_0218BCB0(r)`: the door kind of a record's model kind ([`DOOR_KIND`], 0 from 16 on).
pub fn door_kind<E: FieldEnv>(env: &mut E, r: u32) -> u32 {
    let k = env.r16(r + 2) as usize;
    if k >= 16 {
        return 0;
    }
    DOOR_KIND[k] as u32
}

/// C: `FUN_0218BCE4(t, season)`: init the time of day TOD (`+0 = 5`, the season at `+0xD`) and update it.
pub fn tod_init<E: FieldEnv>(env: &mut E, t: u32, season: u32) {
    env.w32(t, 5);
    env.w8(t + 0xD, season as u8);
    tod_update(env, t);
}

/// C: `FUN_0218BCF4(t)`: the time zone of the season now (`020113D8`): `+4` the previous one, `+8` changed, `+0xC` its
/// animation index ([`TOD_ANIM`]; UNVERIFIED: a zone from 8 on reads past the ROM table in the original, this port panics).
pub fn tod_update<E: FieldEnv>(env: &mut E, t: u32) {
    let old = env.r32(t);
    env.w32(t + 4, old);
    let s = env.r8(t + 0xD) as u32;
    let z = env.callv(TIME_ZONE, &[s]);
    let prev = env.r32(t + 4);
    env.w32(t, z);
    env.w32(t + 8, (z != prev) as u32);
    let z = env.r32(t);
    env.w8(t + 0xC, TOD_ANIM[z as usize]);
}

/// C: `FUN_0218BD20(t)`: the time zone changed in the last update.
pub fn tod_changed<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    env.r32(t + 8)
}

/// C: `FUN_0218BD24(t)`: the animation index of the time zone.
pub fn tod_anim<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    env.r8(t + 0xC) as u32
}

/// C: `FUN_0218BD28(bm, h2)`: an animation resource set per model (`h2` is passed on to [`load_set`], which does not use it).
pub fn load_sets<E: FieldEnv>(env: &mut E, bm: u32, _h2: u32) {
    let n = env.r16(bm + BM_COUNT) as u32;
    if n == 0 {
        return;
    }
    env.w32(bm + BM_NSETS, n);
    let hp = heap(env, bm);
    let a = env.callv(
        HEAP_ALLOC,
        &[hp, 24u32.wrapping_mul(n), 1, FILE_BUILDMODEL, 0x45F],
    );
    env.w32(bm + BM_SETS, a);
    let mut i = 0u32;
    while i < env.r32(bm + BM_NSETS) {
        let s = env.r32(bm + BM_SETS).wrapping_add(SET_SIZE * i);
        load_set(env, bm, s, i);
        i = (i + 1) & 0xFF;
    }
}

/// C: `FUN_0218BD94(bm)`: free the animation resource sets, if any.
pub fn free_sets<E: FieldEnv>(env: &mut E, bm: u32) {
    if env.r32(bm + BM_SETS) == 0 {
        return;
    }
    let mut i = 0u32;
    while i < env.r32(bm + BM_NSETS) {
        let s = env.r32(bm + BM_SETS).wrapping_add(SET_SIZE.wrapping_mul(i));
        free_set(env, s);
        i += 1;
    }
    let a = env.r32(bm + BM_SETS);
    env.callv(HEAP_FREE, &[a]);
    env.w32(bm + BM_NSETS, 0);
    env.w32(bm + BM_SETS, 0);
}

/// C: `FUN_0218BDD8(bm, s)`: an animation object per model (`+0x1D8`, on the sets `+0x1CC`), and `s` = {array of {object, 0}, count}.
pub fn make_objs<E: FieldEnv>(env: &mut E, bm: u32, s: u32) {
    let n = env.r16(bm + BM_COUNT) as u32;
    if n == 0 {
        return;
    }
    env.w32(s + 4, n);
    let hp = heap(env, bm);
    let a = env.callv(HEAP_ALLOC, &[hp, n << 3, 1, FILE_BUILDMODEL, 0x490]);
    env.w32(s, a);
    env.w32(bm + BM_NOBJS, n);
    let hp = heap(env, bm);
    let o = env.callv(
        HEAP_ALLOC,
        &[hp, 24u32.wrapping_mul(n), 1, FILE_BUILDMODEL, 0x494],
    );
    env.w32(bm + BM_OBJS, o);
    let mut i = 0u32;
    while i < env.r32(bm + BM_NOBJS) {
        let objs = env.r32(bm + BM_OBJS);
        let sets = env.r32(bm + BM_SETS);
        let off = SET_SIZE.wrapping_mul(i);
        make_obj(env, bm, objs.wrapping_add(off), sets.wrapping_add(off));
        let w = env.r32(objs.wrapping_add(off));
        let arr = env.r32(s);
        env.w32(arr.wrapping_add(i << 3), w);
        let arr = env.r32(s);
        env.w32(arr.wrapping_add(i << 3).wrapping_add(4), 0);
        i += 1;
    }
}

/// C: `FUN_0218BE84(bm, s)`: free `s`'s blocks and the animation objects.
pub fn free_objs<E: FieldEnv>(env: &mut E, bm: u32, s: u32) {
    let p = env.r32(s);
    if p != 0 {
        env.callv(HEAP_FREE, &[p]);
        env.w32(s, 0);
    }
    let p = env.r32(s + 8);
    if p != 0 {
        env.callv(HEAP_FREE, &[p]);
        env.w32(s + 8, 0);
    }
    if env.r32(bm + BM_OBJS) == 0 {
        return;
    }
    let mut i = 0u32;
    while i < env.r32(bm + BM_NOBJS) {
        let o = env.r32(bm + BM_OBJS).wrapping_add(SET_SIZE.wrapping_mul(i));
        obj_delete(env, o);
        i += 1;
    }
    let a = env.r32(bm + BM_OBJS);
    env.callv(HEAP_FREE, &[a]);
    env.w32(bm + BM_NOBJS, 0);
    env.w32(bm + BM_OBJS, 0);
}

/// C: `FUN_0218BEE8(bm, set, i, h2)`: the animation resource set of model `i`: its record, its model resource and up to 4 animation
/// resources (`h2` is unused).
pub fn load_set<E: FieldEnv>(env: &mut E, bm: u32, set: u32, i: u32) {
    let r = record(env, bm, i);
    env.w32(set, r);
    let size = env.callv(G3D_ANMRES_SIZE, &[]);
    let hp = heap(env, bm);
    let a = env.callv(HEAP_ALLOC, &[hp, size, 1, FILE_BUILDMODEL, 0x4D0]);
    env.w32(set + 4, a);
    let l = env.r32(bm + BM_LIST);
    let data = list_entry2(env, l, i);
    let a = env.r32(set + 4);
    env.callv(G3D_ANMRES_INIT, &[a, data]);
    let r = env.r32(set);
    let hdr = hdr_of(r);
    let cnt = hdr_count(env, hdr);
    for j in 0..4u32 {
        if (j as i32) < (cnt as i32) {
            let size = env.callv(G3D_ANMRES_SIZE, &[]);
            let hp = heap(env, bm);
            let a = env.callv(HEAP_ALLOC, &[hp, size, 1, FILE_BUILDMODEL, 0x4DB]);
            env.w32(set + 8 + 4 * j, a);
            let data = hdr_entry(env, hdr, j);
            let a = env.r32(set + 8 + 4 * j);
            env.callv(G3D_ANMRES_INIT, &[a, data]);
        } else {
            env.w32(set + 8 + 4 * j, 0);
        }
    }
}

/// C: `FUN_0218BF84(set)`: free the animation resources and the model resource of a set.
pub fn free_set<E: FieldEnv>(env: &mut E, set: u32) {
    for j in 0..4u32 {
        let p = env.r32(set + 8 + 4 * j);
        if p != 0 {
            env.callv(HEAP_FREE, &[p]);
            env.w32(set + 8 + 4 * j, 0);
        }
    }
    let p = env.r32(set + 4);
    if p != 0 {
        env.callv(HEAP_FREE, &[p]);
        env.w32(set + 4, 0);
    }
}

/// The animation kind of an object (byte 0 of its set's record header).
fn obj_kind<E: FieldEnv>(env: &mut E, obj: u32) -> (u32, u32) {
    let set = env.r32(obj + 4);
    let r = env.r32(set);
    let hdr = hdr_of(r);
    (hdr, env.r8(hdr) as u32)
}

/// C: `FUN_0218BFB4(bm, obj, set)`: the animation object of a set (G3D model with the textures, its animations) and the init
/// proc of the animation kind (ROM table `0x021D3D58`: [`init_off`] / [`init_all`] / [`init_off`] / [`init_tod`]).
pub fn make_obj<E: FieldEnv>(env: &mut E, bm: u32, obj: u32, set: u32) {
    let r = env.r32(set);
    let hdr = hdr_of(r);
    let res = env.r32(set + 4);
    let tex = env.r32(bm + BM_TEX);
    let m = env.callv(G3D_MODEL_NEW, &[res, 0, tex]);
    let mut anims = [0u8; 16];
    for j in 0..4usize {
        let p = env.r32(set + 8 + 4 * j as u32);
        let a = if p != 0 {
            env.callv(G3D_ANIM_NEW, &[m, p, 0])
        } else {
            0
        };
        anims[4 * j..4 * j + 4].copy_from_slice(&a.to_le_bytes());
    }
    let o = env.call(
        G3D_OBJ_NEW,
        &mut [CArg::V(m), CArg::InBuf(&anims), CArg::V(4)],
    );
    env.w32(obj, o);
    env.w32(obj + 4, set);
    match env.r8(hdr) {
        0 | 2 => init_off(env, obj),
        1 => init_all(env, obj),
        3 => init_tod(env, bm, obj),
        // UNVERIFIED: the original calls a word past the 4-kind table
        k => panic!("animation kind {k} outside the table"),
    }
}

/// C: `FUN_0218C02C(bm, obj)` (init, kinds 0 / 2): all states off.
pub fn init_off<E: FieldEnv>(env: &mut E, obj: u32) {
    for j in 0..4u32 {
        env.w32(obj + 8 + 4 * j, 0);
    }
}

/// C: `FUN_0218C040(bm, obj)` (init, kind 1): the first `min(4, frames per state)` animations on and looping, the rest off.
pub fn init_all<E: FieldEnv>(env: &mut E, obj: u32) {
    let set = env.r32(obj + 4);
    let r = env.r32(set);
    let c = env.r8(hdr_of(r) + 2) as u32;
    let mut i = 0u32;
    while i < 4 && i < c {
        let g = env.r32(obj);
        env.callv(G3D_ANIM_ON, &[g, i & 0xFFFF]);
        let g = env.r32(obj);
        env.callv(G3D_ANIM_REWIND, &[g, i & 0xFFFF]);
        env.w32(obj + 8 + 4 * i, 1);
        i += 1;
    }
    while i < 4 {
        env.w32(obj + 8 + 4 * i, 0);
        i += 1;
    }
}

/// C: `FUN_0218C08C(bm, obj)` (init, kind 3): only the animation of the time of day on.
pub fn init_tod<E: FieldEnv>(env: &mut E, bm: u32, obj: u32) {
    let set = env.r32(obj + 4);
    let r = env.r32(set);
    let _ = hdr_of(r);
    let z = tod_anim(env, bm + BM_TOD);
    for i in 0..4u32 {
        if i == z {
            let g = env.r32(obj);
            env.callv(G3D_ANIM_ON, &[g, i]);
            let g = env.r32(obj);
            env.callv(G3D_ANIM_REWIND, &[g, i]);
            env.w32(obj + 8 + 4 * i, 1);
        } else {
            env.w32(obj + 8 + 4 * i, 0);
        }
    }
}

/// C: `FUN_0218C0D8(obj)`: delete an animation object (its animations, the object, its model), if any.
pub fn obj_delete<E: FieldEnv>(env: &mut E, obj: u32) {
    let g = env.r32(obj);
    if g == 0 {
        return;
    }
    let n = env.callv(G3D_OBJ_COUNT, &[g]);
    let mut i = 0u32;
    while (i as i32) < (n as i32) {
        let g = env.r32(obj);
        let a = env.callv(G3D_OBJ_ANIM, &[g, i & 0xFFFF]);
        if a != 0 {
            env.callv(G3D_ANIM_DELETE, &[a]);
        }
        i += 1;
    }
    let g = env.r32(obj);
    let m = env.callv(G3D_OBJ_MODEL, &[g]);
    let g = env.r32(obj);
    env.callv(G3D_OBJ_DELETE, &[g]);
    env.w32(obj, 0);
    env.w32(obj + 4, 0);
    env.callv(G3D_MODEL_DELETE, &[m]);
}

/// C: `FUN_0218C124(bm, obj)`: a frame of an animation object (when it exists): the frame proc of its kind (ROM table
/// `0x021D3D5C`: [`frame_none`] / [`frame_all`] / [`frame_states`] / [`frame_tod`]).
pub fn obj_frame<E: FieldEnv>(env: &mut E, bm: u32, obj: u32) {
    let (_, k) = obj_kind(env, obj);
    if env.r32(obj) == 0 {
        return;
    }
    match k {
        0 => frame_none(),
        1 => frame_all(env, obj),
        2 => frame_states(env, obj),
        3 => frame_tod(env, bm, obj),
        // UNVERIFIED: the original calls a word past the 4-kind table
        k => panic!("animation kind {k} outside the table"),
    }
}

/// C: `FUN_0218C150()` (frame, kind 0): nothing.
pub fn frame_none() {}

/// C: `FUN_0218C154(bm, obj)` (frame, kind 3): on a time zone change only the zone's animation, rewound; else it advances.
pub fn frame_tod<E: FieldEnv>(env: &mut E, bm: u32, obj: u32) {
    let z = tod_anim(env, bm + BM_TOD);
    if tod_changed(env, bm + BM_TOD) != 0 {
        all_off(env, obj);
        let g = env.r32(obj);
        env.callv(G3D_ANIM_ON, &[g, z & 0xFFFF]);
        let g = env.r32(obj);
        env.callv(G3D_ANIM_REWIND, &[g, z & 0xFFFF]);
        env.w32(obj.wrapping_add(z << 2).wrapping_add(8), 1);
    } else {
        let g = env.r32(obj);
        env.callv(G3D_ANIM_ADVANCE, &[g, z & 0xFFFF, 0x1000]);
    }
}

/// C: `FUN_0218C1A4(bm, obj)` (frame, kind 1): all 4 animations advance.
pub fn frame_all<E: FieldEnv>(env: &mut E, obj: u32) {
    for j in 0..4u32 {
        let g = env.r32(obj);
        env.callv(G3D_ANIM_ADVANCE, &[g, j, 0x1000]);
    }
}

/// C: `FUN_0218C1C4(bm, obj)` (frame, kind 2): looping animations advance; playing ones advance once and stop at their end
/// (backward ones also reset their frame).
pub fn frame_states<E: FieldEnv>(env: &mut E, obj: u32) {
    for j in 0..4u32 {
        let st = obj + 8 + 4 * j;
        match env.r32(st) {
            1 => {
                let g = env.r32(obj);
                env.callv(G3D_ANIM_ADVANCE, &[g, j, 0x1000]);
            }
            3 => {
                let g = env.r32(obj);
                if env.callv(G3D_ANIM_ADVANCE_ONCE, &[g, j, 0x1000]) == 0 {
                    env.w32(st, 2);
                }
            }
            4 => {
                let g = env.r32(obj);
                if env.callv(G3D_ANIM_ADVANCE_ONCE, &[g, j, 0xFFFF_F000]) == 0 {
                    let g = env.r32(obj);
                    env.callv(G3D_ANIM_SET_FRAME, &[g, j, 0]);
                    env.w32(st, 2);
                }
            }
            _ => {}
        }
    }
}

/// The animations of state `k` (`k * c + j` for `j < c`, `c` = frames per state) on, rewound, in state `st`.
fn play_states<E: FieldEnv>(env: &mut E, obj: u32, k: u32, st: u32) {
    let set = env.r32(obj + 4);
    let hdr = hdr_of(env.r32(set));
    let c = env.r8(hdr + 2) as u32;
    let base = c.wrapping_mul(k);
    let mut j = 0u32;
    if c != 0 {
        loop {
            let s = base.wrapping_add(j);
            let g = env.r32(obj);
            env.callv(G3D_ANIM_ON, &[g, s & 0xFFFF]);
            let g = env.r32(obj);
            env.callv(G3D_ANIM_REWIND, &[g, s & 0xFFFF]);
            env.w32(obj.wrapping_add(base << 2).wrapping_add(4 * j + 8), st);
            j += 1;
            if (j as i32) >= env.r8(hdr + 2) as i32 {
                break;
            }
        }
    }
}

/// C: `FUN_0218C23C(obj, k)`: play the animations of state `k` forward once (state 3).
pub fn play_forward<E: FieldEnv>(env: &mut E, obj: u32, k: u32) {
    play_states(env, obj, k, 3);
}

/// C: `FUN_0218C290(obj, k)`: loop the animations of state `k` (state 1).
pub fn play_loop<E: FieldEnv>(env: &mut E, obj: u32, k: u32) {
    play_states(env, obj, k, 1);
}

/// C: `FUN_0218C2E4(obj, k)`: play the animations of state `k` backward once from their last frame (state 4).
pub fn play_backward<E: FieldEnv>(env: &mut E, obj: u32, k: u32) {
    let set = env.r32(obj + 4);
    let hdr = hdr_of(env.r32(set));
    let c = env.r8(hdr + 2) as u32;
    let base = c.wrapping_mul(k);
    let mut j = 0u32;
    if c != 0 {
        loop {
            let s = base.wrapping_add(j);
            let g = env.r32(obj);
            env.callv(G3D_ANIM_ON, &[g, s & 0xFFFF]);
            let g = env.r32(obj);
            let a = env.callv(G3D_OBJ_ANIM, &[g, s & 0xFFFF]);
            let res = env.callv(G3D_ANIM_RES, &[a]);
            let p = env.r32(res + 8);
            let v = (env.r16(p + 4) as u32) << 12;
            let g = env.r32(obj);
            env.call(
                G3D_ANIM_SET_FRAME,
                &mut [
                    CArg::V(g),
                    CArg::V(s & 0xFFFF),
                    CArg::InBuf(&v.to_le_bytes()),
                ],
            );
            env.w32(obj.wrapping_add(base << 2).wrapping_add(4 * j + 8), 4);
            j += 1;
            if (j as i32) >= env.r8(hdr + 2) as i32 {
                break;
            }
        }
    }
}

/// C: `FUN_0218C350(obj, k)`: stop the animations of state `k` (the ones not off go to state 2).
pub fn stop<E: FieldEnv>(env: &mut E, obj: u32, k: u32) {
    let set = env.r32(obj + 4);
    let hdr = hdr_of(env.r32(set));
    let c = env.r8(hdr + 2) as u32;
    let base = obj.wrapping_add(c.wrapping_mul(k) << 2);
    let mut j = 0u32;
    if c != 0 {
        loop {
            let st = base.wrapping_add(j << 2).wrapping_add(8);
            if env.r32(st) != 0 {
                env.w32(st, 2);
            }
            j += 1;
            if (j as i32) >= env.r8(hdr + 2) as i32 {
                break;
            }
        }
    }
}

/// C: `FUN_0218C388(obj)`: all animations that are not off: off.
pub fn all_off<E: FieldEnv>(env: &mut E, obj: u32) {
    for j in 0..4u32 {
        let st = obj + 8 + 4 * j;
        if env.r32(st) != 0 {
            let g = env.r32(obj);
            env.callv(G3D_ANIM_OFF, &[g, j]);
            env.w32(st, 0);
        }
    }
}

/// C: `FUN_0218C3B0(obj, k, mode)`: set state `k` (`k & 0xFF`, 0 from 4 on) to `mode` through the state proc of the animation
/// kind (ROM table `0x021D3D60`: [`state_none`] / [`state_none`] / [`state_proc`] / [`state_none`]).
pub fn set_state<E: FieldEnv>(env: &mut E, obj: u32, k: u32, mode: u32) {
    let (_, kind) = obj_kind(env, obj);
    let k = if k & 0xFF >= 4 { 0 } else { k & 0xFF };
    match kind {
        0 | 1 | 3 => state_none(),
        2 => state_proc(env, obj, k, mode),
        // UNVERIFIED: the original calls a word past the 4-kind table
        k => panic!("animation kind {k} outside the table"),
    }
}

/// C: `FUN_0218C3E0(obj, k, mode)` (state proc, kind 2): mode 0 play forward, 1 play backward, 2 loop (each after all off), 3 stop,
/// 4 all off, others nothing.
pub fn state_proc<E: FieldEnv>(env: &mut E, obj: u32, k: u32, mode: u32) {
    match mode {
        0 => {
            all_off(env, obj);
            play_forward(env, obj, k);
        }
        1 => {
            all_off(env, obj);
            play_backward(env, obj, k);
        }
        2 => {
            all_off(env, obj);
            play_loop(env, obj, k);
        }
        3 => stop(env, obj, k),
        4 => all_off(env, obj),
        _ => {}
    }
}

/// C: `FUN_0218C438()` (state proc, kinds 0 / 1 / 3): nothing.
pub fn state_none() {}

/// C: `FUN_0218C43C(obj, k)`: 1 when animation `k` (`k & 0xFF`, 0 from 4 on) is off or stopped (state 0 / 2), else 0.
pub fn stopped<E: FieldEnv>(env: &mut E, obj: u32, k: u32) -> u32 {
    let k = if k & 0xFF >= 4 { 0 } else { k & 0xFF };
    match env.r32(obj + 8 + 4 * k) {
        0 | 2 => 1,
        _ => 0,
    }
}

/// C: `FUN_0218C47C(obj, j)`: the frame of animation `j` (`02047828(obj+0, j, &v)`; UNVERIFIED: when the callee does not write
/// `v` the original answers its caller's r3, this port 0).
pub fn frame_of<E: FieldEnv>(env: &mut E, obj: u32, j: u32) -> u32 {
    let g = env.r32(obj);
    let mut v = [0u8; 4];
    env.call(
        G3D_ANIM_FRAME,
        &mut [CArg::V(g), CArg::V(j & 0xFFFF), CArg::OutBuf(&mut v)],
    );
    u32::from_le_bytes(v)
}

/// C: `FUN_0218C490(bm, e)`: clear a placed-model entry (free, no saved state).
pub fn entry_clear<E: FieldEnv>(env: &mut E, e: u32) {
    env.w32(e, 0);
    env.w32(e + 4, 0xFFFF_FFFF);
    env.w16(e + 8, 0);
    env.w16(e + 0xA, 0);
    env.w32(e + 0xC, 0);
}

/// C: `FUN_0218C4A0(bm, h, desc, idx)`: place a model (`desc` = {list index, x, y, z, u16 rotation}) in the first free entry:
/// field_g3d_map places it (`021693D4(h, desc, idx)`), the entry keeps `h`, `idx` and the placed model's state word; answers the
/// entry, 0 when none is free.
pub fn place<E: FieldEnv>(env: &mut E, bm: u32, h: u32, desc: u32, idx: u32) -> u32 {
    for i in 0..N_ENTRIES {
        let e = bm + BM_ENTRIES + 16 * i;
        if env.r32(e) == 0 {
            env.call(
                MAP_PLACE,
                &mut [CArg::V(h), CArg::In(desc, 18), CArg::V(idx)],
            );
            env.w32(e, h);
            env.w32(e + 4, 0xFFFF_FFFF);
            env.w16(e + 8, idx as u16);
            env.w16(e + 0xA, 1);
            let s = env.callv(MAP_STATE, &[h, idx]);
            env.w32(e + 0xC, s);
            return e;
        }
    }
    0
}

/// C: `FUN_0218C4EC(bm, h)`: remove every placed model of handle `h`.
pub fn remove<E: FieldEnv>(env: &mut E, bm: u32, h: u32) {
    for i in 0..N_ENTRIES {
        let e = bm + BM_ENTRIES + 16 * i;
        if env.r32(e) == h {
            entry_remove(env, bm, e);
        }
    }
}

/// C: `FUN_0218C518(bm, e)`: remove a placed model from field_g3d_map and clear its entry.
pub fn entry_remove<E: FieldEnv>(env: &mut E, _bm: u32, e: u32) {
    let h = env.r32(e);
    let i = env.r16(e + 8) as u32;
    env.callv(MAP_REMOVE, &[h, i]);
    entry_clear(env, e);
}

/// C: `FUN_0218C530(e)`: the animation state of a placed model: its state word, or the saved state when the word is -1 (low byte).
pub fn entry_state<E: FieldEnv>(env: &mut E, e: u32) -> u32 {
    let p = env.r32(e + 0xC);
    let w = env.r32(p);
    if w == 0xFFFF_FFFF {
        return env.r32(e + 4) & 0xFF;
    }
    w & 0xFF
}

/// C: `FUN_0218C54C(e, f)`: `f != 0`: the saved state goes back to the state word (flag 1); else the state word is saved and set
/// to -1 (flag 0).
pub fn entry_swap<E: FieldEnv>(env: &mut E, e: u32, f: u32) {
    if f != 0 {
        let v = env.r32(e + 4);
        let p = env.r32(e + 0xC);
        env.w32(p, v);
        env.w32(e + 4, 0xFFFF_FFFF);
        env.w16(e + 0xA, 1);
    } else {
        let p = env.r32(e + 0xC);
        let v = env.r32(p);
        env.w32(e + 4, v);
        env.w32(p, 0xFFFF_FFFF);
        env.w16(e + 0xA, 0);
    }
}

/// C: `FUN_0218C574(bm, e, x)`: the state word of a placed model = the list index of model id `x`.
pub fn entry_set_state<E: FieldEnv>(env: &mut E, bm: u32, e: u32, x: u32) {
    let v = index_of(env, bm, x);
    let p = env.r32(e + 0xC);
    env.w32(p, v);
}

/// C: `FUN_0218C584(bm, rect, x, n)`: a new list (0x80 bytes) of the placed models inside `rect` (`{z0, z1, x0, x1}`, inclusive,
/// signed; position = block position + offset) whose door kind is `x` (any when 0); `*n` = the count (the original writes past
/// the list for more than 32).
pub fn find_in_box<E: FieldEnv>(env: &mut E, bm: u32, rect: u32, x: u32, n: u32) -> u32 {
    let hp = heap(env, bm);
    let l = env.callv(HEAP_ALLOC, &[hp, 0x80, 1, FILE_BUILDMODEL, 0x749]);
    let mut c = 0u32;
    for i in 0..N_ENTRIES {
        let e = bm + BM_ENTRIES + 16 * i;
        if env.r32(e) == 0 {
            continue;
        }
        if x != 0 && entry_door_kind(env, bm, e) != x {
            continue;
        }
        let p = entry_pos(env, e);
        let (px, pz) = (p[0] as i32, p[2] as i32);
        let inside = env.r32(rect + 8) as i32 <= px
            && env.r32(rect + 0xC) as i32 >= px
            && env.r32(rect) as i32 <= pz
            && env.r32(rect + 4) as i32 >= pz;
        if inside {
            env.w32(l.wrapping_add(c << 2), e);
            c += 1;
        }
    }
    env.w32(n, c);
    l
}

/// The position of a placed model: its block position (`02169510`) + the offset at its state word + 4 (`VEC_Add`).
fn entry_pos<E: FieldEnv>(env: &mut E, e: u32) -> [u32; 3] {
    let h = env.r32(e);
    let mut p = [0u8; 12];
    env.call(MAP_POSITION, &mut [CArg::V(h), CArg::OutBuf(&mut p)]);
    let t = env.r32(e + 0xC).wrapping_add(4);
    let pin = p;
    env.call(
        VEC_ADD,
        &mut [CArg::In(t, 12), CArg::InBuf(&pin), CArg::OutBuf(&mut p)],
    );
    [0, 1, 2].map(|i| u32::from_le_bytes(p[4 * i..4 * i + 4].try_into().unwrap()))
}

/// C: `FUN_0218C638(bm, x, rect)`: the first placed model of [`find_in_box`] (the list's first word also when it found none).
pub fn first_in_box<E: FieldEnv>(env: &mut E, bm: u32, x: u32, rect: u32) -> u32 {
    let n = env.mem().push_local(4);
    let l = find_in_box(env, bm, rect, x, n);
    env.mem().pop_local(n);
    let r = env.r32(l);
    env.callv(HEAP_FREE, &[l]);
    r
}

/// C: `FUN_0218C654(bm, x, pos)`: the first placed model of door kind `x` in the box around `pos` ([`box_around`]).
pub fn first_near<E: FieldEnv>(env: &mut E, bm: u32, x: u32, pos: u32) -> u32 {
    let rect = env.mem().push_local(16);
    box_around(env, rect, pos);
    let r = first_in_box(env, bm, x, rect);
    env.mem().pop_local(rect);
    r
}

/// The list index of a placed model's state (0 when 0x80.. or past the model count).
fn entry_index<E: FieldEnv>(env: &mut E, bm: u32, e: u32) -> u32 {
    let v = entry_state(env, e);
    if v >= 0x80 || (v as i32) >= env.r16(bm + BM_COUNT) as i32 {
        return 0;
    }
    v
}

/// C: `FUN_0218C674(bm, e)`: the door kind of a placed model's model.
pub fn entry_door_kind<E: FieldEnv>(env: &mut E, bm: u32, e: u32) -> u32 {
    let v = entry_index(env, bm, e);
    let r = record(env, bm, v);
    door_kind(env, r)
}

/// C: `FUN_0218C69C(bm, e, k, mode)`: [`set_state`] on the animation object of a placed model's model.
pub fn entry_set_anim<E: FieldEnv>(env: &mut E, bm: u32, e: u32, k: u32, mode: u32) {
    let v = entry_index(env, bm, e);
    let o = env.r32(bm + BM_OBJS).wrapping_add(SET_SIZE.wrapping_mul(v));
    set_state(env, o, k, mode);
}

/// C: `FUN_0218C6D0(e, out)`: the position of a placed model.
pub fn entry_position<E: FieldEnv>(env: &mut E, e: u32, out: u32) {
    let h = env.r32(e);
    let mut p = [0u8; 12];
    env.call(MAP_POSITION, &mut [CArg::V(h), CArg::OutBuf(&mut p)]);
    let t = env.r32(e + 0xC).wrapping_add(4);
    env.call(
        VEC_ADD,
        &mut [CArg::In(t, 12), CArg::InBuf(&p), CArg::Out(out, 12)],
    );
}

/// C: `FUN_0218C6F4(bm, e, j)`: [`frame_of`] on the animation object of a placed model's model.
pub fn entry_frame<E: FieldEnv>(env: &mut E, bm: u32, e: u32, j: u32) -> u32 {
    let v = entry_index(env, bm, e);
    let o = env.r32(bm + BM_OBJS).wrapping_add(SET_SIZE.wrapping_mul(v));
    frame_of(env, o, j)
}

/// C: `FUN_0218C724(bm, e)`: a new DOOR for placed model `e`: its rotation (the state's u16 `+0x10`), its position (block
/// position + offset + the map offset of `G`), scale 1, an animation object on resource set `*(e+0xC)`; in a door slot; the
/// placed model's state is saved ([`entry_swap`] 0).
pub fn door_new<E: FieldEnv>(env: &mut E, bm: u32, e: u32) -> u32 {
    let t = env.r32(e + 0xC);
    let hp = heap(env, bm);
    let d = env.callv(HEAP_ALLOC, &[hp, DOOR_SIZE, 0, FILE_BUILDMODEL, 0x7E9]);
    env.w32(d, bm);
    env.w32(d + 8, e);
    env.w32(d + 4, 0xFFFF);
    let i = (env.r16(t + 0x10) as u32) >> 4;
    let s = fx_sin(i) as i32 as u32;
    let c = fx_cos(i) as i32 as u32;
    env.call(
        MTX_ROT_Y33,
        &mut [CArg::Out(d + 0x3C, 0x24), CArg::V(s), CArg::V(c)],
    );
    let h = env.r32(e);
    env.call(MAP_POSITION, &mut [CArg::V(h), CArg::Out(d + 0x24, 12)]);
    env.call(
        VEC_ADD,
        &mut [
            CArg::In(d + 0x24, 12),
            CArg::In(t + 4, 12),
            CArg::Out(d + 0x24, 12),
        ],
    );
    let g = env.r32(bm + BM_G);
    let mut off = [0u8; 12];
    env.call(MAPPER_OFFSET, &mut [CArg::V(g), CArg::OutBuf(&mut off)]);
    env.call(
        VEC_ADD,
        &mut [
            CArg::In(d + 0x24, 12),
            CArg::InBuf(&off),
            CArg::Out(d + 0x24, 12),
        ],
    );
    env.w32(d + 0x30, 0x1000);
    env.w32(d + 0x34, 0x1000);
    env.w32(d + 0x38, 0x1000);
    let k = env.r32(t);
    let set = env.r32(bm + BM_SETS).wrapping_add(SET_SIZE.wrapping_mul(k));
    make_obj(env, bm, d + 0xC, set);
    door_slot_put(env, bm, d);
    let e = env.r32(d + 8);
    entry_swap(env, e, 0);
    d
}

/// C: `FUN_0218C7D8(bm, x, pos)`: a DOOR for the first placed model of door kind `x` near `pos`, 0 when there is none.
pub fn door_new_near<E: FieldEnv>(env: &mut E, bm: u32, x: u32, pos: u32) -> u32 {
    let e = first_near(env, bm, x, pos);
    if e == 0 {
        return 0;
    }
    door_new(env, bm, e)
}

/// C: `FUN_0218C7F0(bm, x, src)`: a new DOOR without a placed model: its position / scale / rotation copied from `src` (0x3C
/// bytes), an animation object on the resource set of model id `x`; in a door slot.
pub fn door_new_at<E: FieldEnv>(env: &mut E, bm: u32, x: u32, src: u32) -> u32 {
    let hp = heap(env, bm);
    let d = env.callv(HEAP_ALLOC, &[hp, DOOR_SIZE, 0, FILE_BUILDMODEL, 0x825]);
    env.w32(d, bm);
    env.w32(d + 8, 0);
    env.w32(d + 4, 0xFFFF);
    env.mem().copy(d + 0x24, src, 0x3C);
    let v = index_of(env, bm, x);
    let set = env.r32(bm + BM_SETS).wrapping_add(SET_SIZE.wrapping_mul(v));
    make_obj(env, bm, d + 0xC, set);
    door_slot_put(env, bm, d);
    d
}

/// C: `FUN_0218C860(d)`: delete a DOOR (when not null): out of its slot, its animation object, its placed model's state restored.
pub fn door_delete<E: FieldEnv>(env: &mut E, d: u32) {
    if d == 0 {
        return;
    }
    let bm = env.r32(d);
    door_slot_clear(env, bm, d);
    obj_delete(env, d + 0xC);
    let e = env.r32(d + 8);
    if e != 0 {
        entry_swap(env, e, 1);
    }
    env.callv(HEAP_FREE, &[d]);
}

/// C: `FUN_0218C88C(d, s, mode)`: door state `s` (kept in `+4`) set to `mode` (when `d` is not null).
pub fn door_set<E: FieldEnv>(env: &mut E, d: u32, s: u32, mode: u32) {
    if d == 0 {
        return;
    }
    env.w32(d + 4, s);
    set_state(env, d + 0xC, s, mode);
}

/// C: `FUN_0218C89C(d, mode)`: the current door state set to `mode` (when `d` is not null).
pub fn door_mode<E: FieldEnv>(env: &mut E, d: u32, mode: u32) {
    if d == 0 {
        return;
    }
    let s = env.r32(d + 4);
    set_state(env, d + 0xC, s, mode);
}

/// C: `FUN_0218C8B0(d, s)`: when door animation `s` has stopped, stop state `s` ([`door_set`] mode 3) and 1; 0 while it plays; 1
/// for a null door.
pub fn door_start<E: FieldEnv>(env: &mut E, d: u32, s: u32) -> u32 {
    if d == 0 {
        return 1;
    }
    if door_stopped_s(env, d, s) == 1 {
        door_set(env, d, s, 3);
        return 1;
    }
    0
}

/// C: `FUN_0218C8D8(d)`: [`door_start`] for the current door state.
pub fn door_restart<E: FieldEnv>(env: &mut E, d: u32) -> u32 {
    if d == 0 {
        return 1;
    }
    if door_stopped(env, d) == 1 {
        door_mode(env, d, 3);
        return 1;
    }
    0
}

/// C: `FUN_0218C8FC(d, s)`: [`stopped`] for door animation `s` (0 for a null door).
pub fn door_stopped_s<E: FieldEnv>(env: &mut E, d: u32, s: u32) -> u32 {
    if d == 0 {
        return 0;
    }
    stopped(env, d + 0xC, s)
}

/// C: `FUN_0218C910(d)`: [`stopped`] for the current door state (0 for a null door).
pub fn door_stopped<E: FieldEnv>(env: &mut E, d: u32) -> u32 {
    if d == 0 {
        return 0;
    }
    let s = env.r32(d + 4);
    stopped(env, d + 0xC, s)
}

/// C: `FUN_0218C924(d)`: the model kind of the door's model (u16 `+2` of its record; 0 for a null door).
pub fn door_model_kind<E: FieldEnv>(env: &mut E, d: u32) -> u32 {
    if d == 0 {
        return 0;
    }
    let set = env.r32(d + 0x10);
    let r = env.r32(set);
    env.r16(r + 2) as u32
}

/// C: `FUN_0218C934(d)`: draw a door (when not null).
pub fn door_draw<E: FieldEnv>(env: &mut E, d: u32) {
    if d == 0 {
        return;
    }
    let g = env.r32(d + 0xC);
    env.call(G3D_DRAW, &mut [CArg::V(g), CArg::In(d + 0x24, 0x3C)]);
}

/// The sound of door state `k` for door model kind `v` ([`DOOR_SE`]): `Some(se)` when the kind is in the table.
fn door_se_value(v: u32, k: u32) -> Option<u16> {
    DOOR_SE
        .iter()
        .find(|(key, _)| *key as u32 == v)
        .map(|(_, se)| se[k as usize])
}

/// C: `FUN_0218C944(d, k, out)`: `*out` (u16) = the sound of door state `k` for the door's model kind and 1; 0 when `d` is null
/// (`*out` untouched), `k >= 4` or the kind has no sounds (`*out = 0`).
pub fn door_se_of<E: FieldEnv>(env: &mut E, d: u32, k: u32, out: u32) -> u32 {
    if d == 0 {
        return 0;
    }
    let v = door_model_kind(env, d);
    env.w16(out, 0);
    if k >= 4 {
        return 0;
    }
    match door_se_value(v, k) {
        Some(se) => {
            env.w16(out, se);
            1
        }
        None => 0,
    }
}

/// C: `FUN_0218C990(d, out)`: [`door_se_of`] for the current door state (0 for a null door).
pub fn door_se<E: FieldEnv>(env: &mut E, d: u32, out: u32) -> u32 {
    if d == 0 {
        return 0;
    }
    let s = env.r32(d + 4);
    door_se_of(env, d, s, out)
}

/// C: `FUN_0218C9A4(d)`: play the sound of the current door state; answers the player's answer (0 when there is none).
pub fn door_play_se<E: FieldEnv>(env: &mut E, d: u32) -> u32 {
    let se = env.mem().push_local(4);
    let r = door_se(env, d, se);
    let v = env.r16(se) as u32;
    env.mem().pop_local(se);
    if r == 0 {
        return 0;
    }
    let p = env.callv(SE_PLAYER, &[v]);
    env.callv(SE_PLAY, &[p])
}

/// C: `FUN_0218C9C4(rect, pos)`: the box around a position: `{z - 0x30000, z + 0x30000, x - 0x20000, x + 0x20000}`.
pub fn box_around<E: FieldEnv>(env: &mut E, rect: u32, pos: u32) {
    let z = env.r32(pos + 8);
    env.w32(rect + 4, z.wrapping_add(0x30000));
    let x = env.r32(pos);
    env.w32(rect, z.wrapping_sub(0x30000));
    env.w32(rect + 8, x.wrapping_sub(0x20000));
    env.w32(rect + 0xC, x.wrapping_add(0x20000));
}

#[cfg(test)]
mod tests;
