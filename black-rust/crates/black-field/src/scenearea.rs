//! `fld_scenearea.c` (overlay 21, `0x021AB5F4..0x021AB7E4`, the first part of F1 row 21.31; F2b): the SCENE AREA system of a
//! map. A map has a list of scene areas (0x48-byte records loaded by [`crate::scenearea_loader`]); every frame [`update`] tests
//! the player position against each area with the area's CHECK callback, and when the area the player is in changes it runs
//! the EXIT callback of the old area and the ENTER callback of the new one (else the UPDATE callback of the current one). The
//! callbacks (camera set-ups, [`crate::scenearea_loader`]) come from a callback table descriptor CB.
//!
//! Spec: `tools/rom-diff/specs/black-field/scenearea.spec` (its header has the SA / AREA / CB layouts). Verified by
//! `tools/check-fn.sh black-field scenearea --per-function`. The callbacks are reached through [`FieldEnv::call`] by address.

use crate::hooks::{CArg, FieldEnv, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021A_B5F4,
    0x021A_B624,
    0x021A_B634,
    0x021A_B650,
    0x021A_B660,
    0x021A_B75C,
    0x021A_B760,
    0x021A_B764,
    0x021A_B768,
    0x021A_B780,
    0x021A_B798,
    0x021A_B7C0,
];

/// Address of the string `"fld_scenearea.c"` (the file argument of the allocation in [`new`]).
pub const FILE_SCENEAREA: u32 = 0x021D_DFC8;
/// Size of the SA object.
pub const SA_SIZE: u32 = 0x20;
/// Size of an AREA record.
pub const AREA_SIZE: u32 = 0x48;
/// "No area" (`+0x14`).
pub const NO_AREA: u32 = 0xFFFF_FFFF;

/// SA fields.
pub const SA_FIELDSYS: u32 = 0x0;
pub const SA_FIELDMAP: u32 = 0x4;
pub const SA_AREAS: u32 = 0x8;
pub const SA_CB: u32 = 0xC;
pub const SA_COUNT: u32 = 0x10;
pub const SA_CURRENT: u32 = 0x14;
pub const SA_ENABLED: u32 = 0x18;
pub const SA_LAST_ID: u32 = 0x1C;
/// AREA fields: the callback ids.
pub const AREA_CHECK: u32 = 0x40;
pub const AREA_UPDATE: u32 = 0x42;
pub const AREA_ENTER: u32 = 0x44;
pub const AREA_EXIT: u32 = 0x46;
/// CB fields.
pub const CB_CHECKS: u32 = 0x0;
pub const CB_UPDATES: u32 = 0x4;
pub const CB_NCHECK: u32 = 0x8;
pub const CB_NUPDATE: u32 = 0xA;

/// C: `FUN_021AB5F4(heap, a, b)`: a new SA (0x20 bytes on `heap & 0xFFFF`): `+0 = a`, `+4 = b`, no current area, enabled. The
/// list fields `+8..+0x10` and `+0x1C` are left as allocated (callers set the list with [`set_list`]).
pub fn new<E: FieldEnv>(env: &mut E, heap: u32, a: u32, b: u32) -> u32 {
    let sa = env.callv(
        HEAP_ALLOC,
        &[heap & 0xFFFF, SA_SIZE, 1, FILE_SCENEAREA, 0x51],
    );
    env.w32(sa + SA_FIELDSYS, a);
    env.w32(sa + SA_FIELDMAP, b);
    env.w32(sa + SA_CURRENT, NO_AREA);
    env.w32(sa + SA_ENABLED, 1);
    sa
}

/// C: `FUN_021AB624(sa)`: clear the list and free the SA.
pub fn delete<E: FieldEnv>(env: &mut E, sa: u32) {
    clear_list(env, sa);
    env.callv(HEAP_FREE, &[sa]);
}

/// C: `FUN_021AB634(sa, areas, count, cb)`: set the area list (`count` AREA records at `areas`) and the callback table
/// descriptor; no current area.
pub fn set_list<E: FieldEnv>(env: &mut E, sa: u32, areas: u32, count: u32, cb: u32) {
    clear_list(env, sa);
    env.w32(sa + SA_AREAS, areas);
    env.w32(sa + SA_CB, cb);
    env.w32(sa + SA_COUNT, count);
    env.w32(sa + SA_CURRENT, NO_AREA);
}

/// C: `FUN_021AB650(sa)`: no area list, no callback table, no current area.
pub fn clear_list<E: FieldEnv>(env: &mut E, sa: u32) {
    env.w32(sa + SA_AREAS, 0);
    env.w32(sa + SA_CB, 0);
    env.w32(sa + SA_COUNT, 0);
    env.w32(sa + SA_CURRENT, NO_AREA);
}

/// C: `FUN_021AB660(sa, pos)`: one frame: the area `pos` is in (the last one whose check callback answers nonzero) becomes the
/// current one, running the old area's exit callback and the new area's enter callback when it changes, else the current
/// area's update callback; `+0x1C` is the id of the last callback run (0xFFFF none). Answers the current area (-1 none; -1 at
/// once while disabled).
pub fn update<E: FieldEnv>(env: &mut E, sa: u32, pos: u32) -> u32 {
    if env.r32(sa + SA_ENABLED) == 0 {
        return NO_AREA;
    }
    env.w32(sa + SA_LAST_ID, 0xFFFF);
    let mut found = NO_AREA;
    let mut i = 0u32;
    while i < env.r32(sa + SA_COUNT) {
        let area = env
            .r32(sa + SA_AREAS)
            .wrapping_add(i.wrapping_mul(AREA_SIZE));
        let cb = env.r32(sa + SA_CB);
        let id = env.r16(area + AREA_CHECK) as u32;
        if run_check(env, cb, id, sa, area, pos) != 0 {
            found = i;
        }
        i += 1;
    }
    let cur = env.r32(sa + SA_CURRENT);
    if found != cur {
        if cur != NO_AREA {
            let area = env
                .r32(sa + SA_AREAS)
                .wrapping_add(cur.wrapping_mul(AREA_SIZE));
            let cb = env.r32(sa + SA_CB);
            let id = env.r16(area + AREA_EXIT) as u32;
            run_update(env, cb, id, sa, area, pos);
            let cur = env.r32(sa + SA_CURRENT);
            let area = env
                .r32(sa + SA_AREAS)
                .wrapping_add(cur.wrapping_mul(AREA_SIZE));
            let id = env.r16(area + AREA_EXIT);
            env.w32(sa + SA_LAST_ID, id as u32);
        }
        if found != NO_AREA {
            let area = env
                .r32(sa + SA_AREAS)
                .wrapping_add(found.wrapping_mul(AREA_SIZE));
            let cb = env.r32(sa + SA_CB);
            let id = env.r16(area + AREA_ENTER) as u32;
            run_update(env, cb, id, sa, area, pos);
            let area = env
                .r32(sa + SA_AREAS)
                .wrapping_add(found.wrapping_mul(AREA_SIZE));
            let id = env.r16(area + AREA_ENTER);
            env.w32(sa + SA_LAST_ID, id as u32);
        }
        env.w32(sa + SA_CURRENT, found);
    } else if cur != NO_AREA {
        let area = env
            .r32(sa + SA_AREAS)
            .wrapping_add(cur.wrapping_mul(AREA_SIZE));
        let cb = env.r32(sa + SA_CB);
        let id = env.r16(area + AREA_UPDATE) as u32;
        run_update(env, cb, id, sa, area, pos);
        let cur = env.r32(sa + SA_CURRENT);
        let area = env
            .r32(sa + SA_AREAS)
            .wrapping_add(cur.wrapping_mul(AREA_SIZE));
        let id = env.r16(area + AREA_UPDATE);
        env.w32(sa + SA_LAST_ID, id as u32);
    }
    env.r32(sa + SA_CURRENT)
}

/// C: `FUN_021AB75C(sa, v)`: enable (`v != 0`) or disable [`update`].
pub fn set_enabled<E: FieldEnv>(env: &mut E, sa: u32, v: u32) {
    env.w32(sa + SA_ENABLED, v);
}

/// C: `FUN_021AB760(sa)`: `+0` (the field system the callbacks hand to the camera).
pub fn fieldsys<E: FieldEnv>(env: &mut E, sa: u32) -> u32 {
    env.r32(sa + SA_FIELDSYS)
}

/// C: `FUN_021AB764(sa)`: `+4` (the fieldmap).
pub fn fieldmap<E: FieldEnv>(env: &mut E, sa: u32) -> u32 {
    env.r32(sa + SA_FIELDMAP)
}

/// C: `FUN_021AB768(cb, id)`: 1 when `id` is below the check count and its check table entry is set.
pub fn has_check<E: FieldEnv>(env: &mut E, cb: u32, id: u32) -> u32 {
    if (env.r16(cb + CB_NCHECK) as u32) > id {
        let t = env.r32(cb + CB_CHECKS);
        if env.r32(t.wrapping_add(id << 2)) != 0 {
            return 1;
        }
    }
    0
}

/// C: `FUN_021AB780(cb, id)`: 1 when `id` is below the update count and its update table entry is set.
pub fn has_update<E: FieldEnv>(env: &mut E, cb: u32, id: u32) -> u32 {
    if (env.r16(cb + CB_NUPDATE) as u32) > id {
        let t = env.r32(cb + CB_UPDATES);
        if env.r32(t.wrapping_add(id << 2)) != 0 {
            return 1;
        }
    }
    0
}

/// C: `FUN_021AB798(cb, id, sa, area, pos)`: the answer of check callback `id` (`f(sa, area, pos)`), 0 when there is none.
pub fn run_check<E: FieldEnv>(env: &mut E, cb: u32, id: u32, sa: u32, area: u32, pos: u32) -> u32 {
    if has_check(env, cb, id) == 0 {
        return 0;
    }
    let t = env.r32(cb + CB_CHECKS);
    let f = env.r32(t.wrapping_add(id << 2));
    env.call(f & !1, &mut [CArg::V(sa), CArg::V(area), CArg::V(pos)])
}

/// C: `FUN_021AB7C0(cb, id, sa, area, pos)`: run update callback `id` (`f(sa, area, pos)`) and pass on its answer; 0 when there
/// is none.
pub fn run_update<E: FieldEnv>(env: &mut E, cb: u32, id: u32, sa: u32, area: u32, pos: u32) -> u32 {
    if has_update(env, cb, id) == 0 {
        return 0;
    }
    let t = env.r32(cb + CB_UPDATES);
    let f = env.r32(t.wrapping_add(id << 2));
    env.call(f & !1, &mut [CArg::V(sa), CArg::V(area), CArg::V(pos)])
}

#[cfg(test)]
mod tests;
