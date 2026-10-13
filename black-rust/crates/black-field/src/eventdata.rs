//! `eventdata_system.c` (overlay 10, `0x0216237C..0x02163010`): the zone event data of ARC 125 (one member per
//! zone: background events (signs, hidden items), NPC headers, warps and coordinate triggers) and the lookups
//! by position (grid `VecFx32` or rail location), the warp exits and the trigger checks.
//!
//! F6a-spec specced all 56 functions (`tools/rom-diff/specs/black-field/eventdata.spec`, whose header documents
//! the EVENTDATA_SYSTEM (0xAA8 bytes) and the four record layouts; summary in the crate README); F6a ported them.
//! Records are read through a [`Mem`] window at their original addresses. Callees outside the area (heap,
//! archives, zonedata, field_encount_st, the LOCATION constructors, the event-work var / flag accessors, `FX_Mul`,
//! the hidden-item flag of `eventdata_tail`) are reached through [`FieldEnv::call`] by address. The rest of F1
//! row 10.23 is the areas `wfbc_data`, `mmdl_move` and `eventdata_tail`.
//!
//! EVENTDATA_SYSTEM: `+0x04 / +0x08 / +0x0C` ARC 125 / 126 / 57 handles, `+0x10 u16` zone, `+0x12..+0x18 u16` bg /
//! npc / warp / trigger counts, `+0x1C..+0x28` the record arrays, `+0x2C..+0x32` copies of the counts, `+0x34`
//! field_encount_st ready, `+0x38` field_encount_st work, `+0x120` -> the tail after the records, `+0x124` the
//! loaded ARC 125 member. WARP (20 bytes): `+0 u16` destination zone, `+2 u16` destination warp, `+4 u8` exit
//! direction, `+5 u8` door kind, `+6 u16` position kind (0 grid), grid `+8 / +0xA / +0xC s16` x / y / z in
//! `fx32 >> 12` units (16 per tile), `+0xE / +0x10 u16` width / depth in tiles; rail `+8` line, `+0xA` front,
//! `+0xC s16` side, `+0xE` / `+0x10` lengths, `+0x12` flip mode. BG (20): `+0` script, `+2` kind, `+4` facing
//! rule, `+6` position kind, grid `+8 / +0xC u32` tile x / z, `+0x10 s32` y. TRIGGER (22): `+0` script, `+2`
//! value, `+4` var, `+6` kind, `+8` position kind, grid `+0xA / +0xC` tile x / z, `+0xE / +0x10` width / depth,
//! `+0x12 s16` y. RAIL LOCATION (8): `+0 u16` line, `+2 u8` type, `+3 u8` key, `+4 s16` side, `+6 u16` front.

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_237C,
    0x0216_23BC,
    0x0216_23DC,
    0x0216_2408,
    0x0216_242C,
    0x0216_2438,
    0x0216_2450,
    0x0216_2458,
    0x0216_245C,
    0x0216_2478,
    0x0216_24B0,
    0x0216_2530,
    0x0216_255C,
    0x0216_2560,
    0x0216_2564,
    0x0216_2598,
    0x0216_25CC,
    0x0216_25E8,
    0x0216_25FC,
    0x0216_2694,
    0x0216_26B8,
    0x0216_26E8,
    0x0216_26EC,
    0x0216_2708,
    0x0216_2740,
    0x0216_2744,
    0x0216_2748,
    0x0216_2774,
    0x0216_2788,
    0x0216_279C,
    0x0216_27C8,
    0x0216_282C,
    0x0216_2890,
    0x0216_28E4,
    0x0216_2928,
    0x0216_2940,
    0x0216_2958,
    0x0216_2970,
    0x0216_2A54,
    0x0216_2A5C,
    0x0216_2A74,
    0x0216_2AA4,
    0x0216_2B68,
    0x0216_2BCC,
    0x0216_2C54,
    0x0216_2C84,
    0x0216_2CE8,
    0x0216_2D2C,
    0x0216_2D90,
    0x0216_2E00,
    0x0216_2E48,
    0x0216_2E9C,
    0x0216_2ECC,
    0x0216_2F0C,
    0x0216_2F74,
    0x0216_2FC8,
];

/// Size of the EVENTDATA_SYSTEM record (`Heap_AllocDebug(heap, 0xAA8, ..)` in `FUN_0216237C`).
pub const SYS_SIZE: u32 = 0xAA8;
/// Size of a background-event record.
pub const BG_SIZE: u32 = 0x14;
/// Size of an NPC record (= the fldmmdl MMDL HEADER).
pub const NPC_SIZE: u32 = 0x24;
/// Size of a warp record.
pub const WARP_SIZE: u32 = 0x14;
/// Size of a trigger record.
pub const TRIGGER_SIZE: u32 = 0x16;
/// Bytes cleared after the counts of the loaded member (`FUN_02162530`).
pub const MEMBER_DATA_SIZE: u32 = 0x880;
/// "Not found" index / script id of the lookups.
pub const NONE: u32 = 0xFFFF;

/// Address of the string `"eventdata_system.c"` (the file argument of the allocation in `FUN_0216237C`).
pub const FILE_EVENTDATA_SYSTEM: u32 = 0x0217_299C;

/// `FUN_020490F4(arc, heap)`: open an archive, returns its handle.
const ARC_OPEN: u32 = 0x0204_90F4;
/// `FUN_02049238(handle)`: close an archive.
const ARC_CLOSE: u32 = 0x0204_9238;
/// `FUN_020493D4(handle, member)`: size of an archive member.
const ARC_SIZE: u32 = 0x0204_93D4;
/// `FUN_0204944C(handle, member, offset, size, dst)`: read an archive member.
const ARC_READ: u32 = 0x0204_944C;
/// `FUN_02013ECC(zone)` (zonedata): the ARC 125 member of a zone.
const ZONE_EVENTS_MEMBER: u32 = 0x0201_3ECC;
/// `FUN_0215E2B8(work, arc126, zone, x)`: field_encount_st init.
const ENCOUNT_INIT: u32 = 0x0215_E2B8;
/// `FUN_020140F8(loc)`: LOCATION clear.
const LOC_CLEAR: u32 = 0x0201_40F8;
/// `FUN_02014110(loc, zone, warp, dir, code, x, y, z)`: LOCATION init, grid form.
const LOC_INIT_GRID: u32 = 0x0201_4110;
/// `FUN_02014134(loc, zone, warp, dir, code, line, front, side)`: LOCATION init, rail form.
const LOC_INIT_RAIL: u32 = 0x0201_4134;
/// `FUN_0201416C(loc, zone, warp, x)`: LOCATION set.
const LOC_SET: u32 = 0x0201_416C;
/// `FUN_0201441C(dir)`: the rail key of a direction (a byte).
const RAIL_KEY: u32 = 0x0201_441C;
/// `FUN_020143CC(ew, var)`: address of an event-work var.
const VAR_ADDR: u32 = 0x0201_43CC;
/// `FUN_020142E8(ew, flag)`: event-work flag test.
const CHECK_FLAG: u32 = 0x0201_42E8;
/// `FUN_02164708(script)` (`eventdata_tail`): the event flag of a hidden item.
const HIDDEN_ITEM_FLAG: u32 = 0x0216_4708;
/// `FX_Mul(a, b)`.
const FX_MUL: u32 = 0x0207_C768;

/// `(v >> 4) / 4096` toward zero: the tile of an fx32 coordinate (16 units per tile).
fn tile(v: u32) -> i32 {
    ((v as i32) >> 4) / 4096
}

/// `v >> 12`: an fx32 coordinate in units.
fn unit(v: u32) -> i32 {
    (v as i32) >> 12
}

fn s16(mem: &Mem, a: u32) -> i32 {
    mem.r16(a) as i16 as i32
}

/// C: `FUN_0216237C` (unlabeled routine at `0x0216237C`): allocate the EVENTDATA_SYSTEM
/// (`Heap_AllocDebug(heap, 0xAA8, 1, "eventdata_system.c", 0xAF)`) and open ARC 125 (`+4`), 126 (`+8`) and 57
/// (`+0xC`) on `heap`; returns it.
pub fn create<E: FieldEnv>(env: &mut E, heap: u32) -> u32 {
    let sys = env.callv(
        HEAP_ALLOC,
        &[heap, SYS_SIZE, 1, FILE_EVENTDATA_SYSTEM, 0xAF],
    );
    for (arc, o) in [(0x7D, 4), (0x7E, 8), (0x39, 0xC)] {
        let h = env.callv(ARC_OPEN, &[arc, heap]);
        env.w32(sys.wrapping_add(o), h);
    }
    sys
}

/// C: `FUN_021623BC` (unlabeled routine at `0x021623BC`): close the three archives and free the record.
pub fn delete<E: FieldEnv>(env: &mut E, sys: u32) {
    for o in [4, 8, 0xC] {
        let h = env.r32(sys.wrapping_add(o));
        env.callv(ARC_CLOSE, &[h]);
    }
    env.callv(HEAP_FREE, &[sys]);
}

/// C: `FUN_021623DC` (unlabeled routine at `0x021623DC`): [`clear`]; `+0x120 = sys + 0x2328` (past the record; the
/// load replaces it), `+0x34 = 0`, and bit 7 of the byte `+0x3F` (in the field_encount_st work) cleared.
pub fn reset(mem: &mut Mem, sys: u32) {
    clear(mem, sys);
    mem.w32(sys.wrapping_add(0x120), sys.wrapping_add(0x2328));
    mem.w32(sys.wrapping_add(0x34), 0);
    let a = sys.wrapping_add(0x3F);
    let b = mem.r8(a);
    mem.w8(a, b & 0x7F);
}

/// C: `FUN_02162408` (unlabeled routine at `0x02162408`): [`reset`], [`set_zone`] (loads the zone's events) and
/// [`encount_init`].
pub fn load<E: FieldEnv>(env: &mut E, sys: u32, zone: u32, x: u32) {
    reset(env.mem(), sys);
    set_zone(env, sys, zone);
    encount_init(env, sys, zone, x);
}

/// C: `FUN_0216242C` (unlabeled routine at `0x0216242C`): `+0x10 = zone` (u16), then [`load_member`].
pub fn set_zone<E: FieldEnv>(env: &mut E, sys: u32, zone: u32) {
    env.w16(sys.wrapping_add(0x10), zone as u16);
    load_member(env, sys, zone);
}

/// C: `FUN_02162438` (unlabeled routine at `0x02162438`): `FUN_0215E2B8(sys + 0x38, +8 (ARC 126), zone, x)`
/// (field_encount_st init); `+0x34 = 1`.
pub fn encount_init<E: FieldEnv>(env: &mut E, sys: u32, zone: u32, x: u32) {
    let arc = env.r32(sys.wrapping_add(8));
    env.callv(ENCOUNT_INIT, &[sys.wrapping_add(0x38), arc, zone, x]);
    env.w32(sys.wrapping_add(0x34), 1);
}

/// C: `FUN_02162450` (unlabeled routine at `0x02162450`): the tail after the records (`+0x120`).
pub fn tail(mem: &Mem, sys: u32) -> u32 {
    mem.r32(sys.wrapping_add(0x120))
}

/// C: `FUN_02162458` (unlabeled routine at `0x02162458`): `+0x34` (field_encount_st ready).
pub fn encount_ready(mem: &Mem, sys: u32) -> u32 {
    mem.r32(sys.wrapping_add(0x34))
}

/// C: `FUN_0216245C` (unlabeled routine at `0x0216245C`): for `i` below the warp count, disables warp `i`
/// (`+0` and `+2` = 0xFFFF).
pub fn warp_disable(mem: &mut Mem, sys: u32, i: u32) {
    let warps = mem.r32(sys.wrapping_add(0x24));
    if (mem.r16(sys.wrapping_add(0x16)) as u32) > i {
        let w = warps.wrapping_add(i.wrapping_mul(WARP_SIZE));
        mem.w16(w, 0xFFFF);
        mem.w16(w.wrapping_add(2), 0xFFFF);
    }
}

/// C: `FUN_02162478` (unlabeled routine at `0x02162478`): is there a trigger of kind 6 that contains `pos`
/// ([`trigger_contains`])? No caller in the field set.
pub fn kind6_trigger_at(mem: &Mem, sys: u32, pos: u32) -> bool {
    let mut t = mem.r32(sys.wrapping_add(0x28));
    if t == 0 {
        return false;
    }
    let n = mem.r16(sys.wrapping_add(0x18));
    let mut i: u16 = 0;
    while i < n {
        if trigger_contains(mem, t, pos) && mem.r16(t.wrapping_add(6)) == 6 {
            return true;
        }
        i = i.wrapping_add(1);
        t = t.wrapping_add(TRIGGER_SIZE);
    }
    false
}

/// C: `FUN_021624B0` (unlabeled routine at `0x021624B0`): read the zone's ARC 125 member (`FUN_02013ECC(zone)`)
/// whole to `+0x124`; `+0x120` = `sys + 0x128 + size` (the tail); the four count bytes at `+0x128` go to
/// `+0x12..+0x18` and `+0x2C..+0x32`; the record arrays follow from `+0x12C` (bg, npc, warp, trigger).
pub fn load_member<E: FieldEnv>(env: &mut E, sys: u32, zone: u32) {
    let m = env.callv(ZONE_EVENTS_MEMBER, &[zone]);
    let h = env.r32(sys.wrapping_add(4));
    let n = env.callv(ARC_SIZE, &[h, m]);
    let h = env.r32(sys.wrapping_add(4));
    let dst = sys.wrapping_add(0x124);
    env.call(
        ARC_READ,
        &mut [
            CArg::V(h),
            CArg::V(m),
            CArg::V(0),
            CArg::V(n),
            CArg::Out(dst, 0x10),
        ],
    );
    let m = env.mem();
    let base = sys.wrapping_add(0x128);
    let size = m.r32(dst);
    m.w32(sys.wrapping_add(0x120), base.wrapping_add(size));
    let recs = base.wrapping_add(4);
    let nb = m.r8(base) as u32;
    m.w16(sys.wrapping_add(0x12), nb as u16);
    let o_npc = nb * BG_SIZE;
    m.w32(sys.wrapping_add(0x1C), recs);
    let nn = m.r8(base.wrapping_add(1)) as u32;
    m.w32(sys.wrapping_add(0x20), recs.wrapping_add(o_npc));
    m.w16(sys.wrapping_add(0x14), nn as u16);
    let o_warp = o_npc + nn * NPC_SIZE;
    let nw = m.r8(base.wrapping_add(2)) as u32;
    m.w32(sys.wrapping_add(0x24), recs.wrapping_add(o_warp));
    m.w16(sys.wrapping_add(0x16), nw as u16);
    let o_trig = o_warp + nw * WARP_SIZE;
    let nt = m.r8(base.wrapping_add(3));
    m.w16(sys.wrapping_add(0x18), nt as u16);
    m.w32(sys.wrapping_add(0x28), recs.wrapping_add(o_trig));
    for o in [0x12, 0x14, 0x16, 0x18] {
        let c = m.r16(sys.wrapping_add(o));
        m.w16(sys.wrapping_add(o + 0x1A), c);
    }
}

/// C: `FUN_02162530` (unlabeled routine at `0x02162530`): counts (`+0x2C..+0x32`, `+0x12..+0x18`) and record
/// pointers (`+0x1C..+0x28`) = 0; `MI_CpuFill8(sys + 0x128, 0, 0x880)`.
pub fn clear(mem: &mut Mem, sys: u32) {
    for o in [0x2C, 0x2E, 0x30, 0x32, 0x12, 0x14, 0x16, 0x18] {
        mem.w16(sys.wrapping_add(o), 0);
    }
    for o in [0x1C, 0x20, 0x24, 0x28] {
        mem.w32(sys.wrapping_add(o), 0);
    }
    mem.fill(sys.wrapping_add(0x128), MEMBER_DATA_SIZE, 0);
}

/// C: `FUN_0216255C` (unlabeled routine at `0x0216255C`): the field_encount_st work (`sys + 0x38`).
pub fn encount_work(sys: u32) -> u32 {
    sys.wrapping_add(0x38)
}

/// C: `FUN_02162560` (unlabeled routine at `0x02162560`): the warp count (`u16 +0x16`). No caller in the field
/// set.
pub fn warp_count(mem: &Mem, sys: u32) -> u32 {
    mem.r16(sys.wrapping_add(0x16)) as u32
}

/// C: `FUN_02162564` (unlabeled routine at `0x02162564`): index of the first warp that contains the grid position
/// `pos` ([`warp_contains`]), else 0xFFFF.
pub fn warp_index_at(mem: &Mem, sys: u32, pos: u32) -> u32 {
    let mut w = mem.r32(sys.wrapping_add(0x24));
    if mem.r16(sys.wrapping_add(0x16)) != 0 {
        let mut i: i32 = 0;
        loop {
            if warp_contains(mem, w, pos) {
                return i as u32;
            }
            i += 1;
            w = w.wrapping_add(WARP_SIZE);
            if i >= mem.r16(sys.wrapping_add(0x16)) as i32 {
                break;
            }
        }
    }
    NONE
}

/// C: `FUN_02162598` (unlabeled routine at `0x02162598`): index of the first warp that contains the rail location
/// `rail` ([`warp_contains_rail`]), else 0xFFFF.
pub fn warp_index_at_rail(mem: &Mem, sys: u32, rail: u32) -> u32 {
    let mut w = mem.r32(sys.wrapping_add(0x24));
    if mem.r16(sys.wrapping_add(0x16)) != 0 {
        let mut i: i32 = 0;
        loop {
            if warp_contains_rail(mem, w, rail) {
                return i as u32;
            }
            i += 1;
            w = w.wrapping_add(WARP_SIZE);
            if i >= mem.r16(sys.wrapping_add(0x16)) as i32 {
                break;
            }
        }
    }
    NONE
}

/// C: `FUN_021625CC` (unlabeled routine at `0x021625CC`): warp record `i` of the zone, or 0 when there are no
/// warp records (`+0x24` null) or `i` is not below the warp count (`u16 +0x16`, unsigned compare).
pub fn warp_at(mem: &Mem, sys: u32, i: u32) -> u32 {
    let warps = mem.r32(sys.wrapping_add(0x24));
    if warps == 0 || i >= mem.r16(sys.wrapping_add(0x16)) as u32 {
        return 0;
    }
    warps.wrapping_add(i.wrapping_mul(WARP_SIZE))
}

/// C: `FUN_021625E8` (unlabeled routine at `0x021625E8`): is the warp's destination warp (`u16 +2`) 0x100?
pub fn warp_dest_is_0x100(mem: &Mem, warp: u32) -> bool {
    mem.r16(warp.wrapping_add(2)) == 0x100
}

/// C: `FUN_021625FC` (unlabeled routine at `0x021625FC`): fill the LOCATION `loc` with the exit of warp `i` and
/// the exit code `code`: 0 when there is no warp `i` ([`warp_at`]); else LOCATION clear, `loc +0` = 3 when the
/// warp's `+2` is 0x100 else 2, then the grid form (`FUN_02014110(loc, zone, (s16) i, dir byte +4, code, x, y,
/// z)` with the position of [`warp_exit_pos`]) or the rail form (`FUN_02014134(loc, zone, (s16) i, +4, code,
/// line, front, side)` with [`warp_exit_rail`]); returns 1.
pub fn warp_exit_location<E: FieldEnv>(env: &mut E, sys: u32, loc: u32, i: u32, code: u32) -> u32 {
    let w = warp_at(env.mem(), sys, i);
    if w == 0 {
        return 0;
    }
    env.callv(LOC_CLEAR, &[loc]);
    let kind = if env.r16(w.wrapping_add(2)) == 0x100 {
        3
    } else {
        2
    };
    env.w32(loc, kind);
    let idx = i as u16 as i16 as i32 as u32;
    if env.r16(w.wrapping_add(6)) == 0 {
        let v = env.mem().push_local(12);
        warp_exit_pos(env, w, code, v);
        let (x, y, z) = (env.r32(v), env.r32(v + 4), env.r32(v + 8));
        env.mem().pop_local(v);
        let zone = env.r16(sys.wrapping_add(0x10)) as u32;
        let dir = env.r8(w.wrapping_add(4)) as u32;
        let mut a = [
            CArg::Out(loc, 0x1C),
            CArg::V(zone),
            CArg::V(idx),
            CArg::V(dir),
            CArg::V(code),
            CArg::V(x),
            CArg::V(y),
            CArg::V(z),
        ];
        env.call(LOC_INIT_GRID, &mut a);
    } else {
        let r = env.mem().push_local(8);
        warp_exit_rail(env, w, code, r);
        let line = env.r16(r) as u32;
        let front = env.r16(r + 6) as u32;
        let side = env.r16(r + 4) as i16 as i32 as u32;
        env.mem().pop_local(r);
        let zone = env.r16(sys.wrapping_add(0x10)) as u32;
        let dir = env.r8(w.wrapping_add(4)) as u32;
        let mut a = [
            CArg::Out(loc, 0x1C),
            CArg::V(zone),
            CArg::V(idx),
            CArg::V(dir),
            CArg::V(code),
            CArg::V(line),
            CArg::V(front),
            CArg::V(side),
        ];
        env.call(LOC_INIT_RAIL, &mut a);
    }
    1
}

/// C: `FUN_02162694` (unlabeled routine at `0x02162694`): LOCATION set `FUN_0201416C(loc, r +0, r +2, x)` from the
/// two halfwords at `r` (a warp: destination zone and warp).
pub fn location_set<E: FieldEnv>(env: &mut E, r: u32, loc: u32, x: u32) {
    let a = env.r16(r) as u32;
    let b = env.r16(r.wrapping_add(2)) as u32;
    env.callv(LOC_SET, &[loc, a, b, x]);
}

/// C: `FUN_021626B8` (unlabeled routine at `0x021626B8`): does the facing `dir` match the warp's exit byte `+4`:
/// (dir, byte) one of (0, 2), (1, 1), (2, 4), (3, 3)?
pub fn warp_faces(mem: &Mem, warp: u32, dir: u32) -> bool {
    let b = mem.r8(warp.wrapping_add(4));
    matches!((dir, b), (0, 2) | (1, 1) | (2, 4) | (3, 3))
}

/// C: `FUN_021626E8` (unlabeled routine at `0x021626E8`): the warp's door kind (`u8 +5`).
pub fn warp_door_kind(mem: &Mem, warp: u32) -> u32 {
    mem.r8(warp.wrapping_add(5)) as u32
}

/// C: `FUN_021626EC` (unlabeled routine at `0x021626EC`): is the warp disabled (`+0` or `+2` is 0xFFFF)?
pub fn warp_is_disabled(mem: &Mem, warp: u32) -> bool {
    mem.r16(warp) == 0xFFFF || mem.r16(warp.wrapping_add(2)) == 0xFFFF
}

/// C: `FUN_02162708` (unlabeled routine at `0x02162708`): for `i <= warp count` (sic) and a grid warp `i`, move it
/// to the tile `(x, y, z)`: `+8 = 16 x + 8`, `+0xA = 16 y`, `+0xC = 16 z + 8` (halfwords).
pub fn warp_set_tile(mem: &mut Mem, sys: u32, i: u32, x: u32, y: u32, z: u16) {
    if (mem.r16(sys.wrapping_add(0x16)) as u32) < i {
        return;
    }
    let warps = mem.r32(sys.wrapping_add(0x24));
    if warps == 0 {
        return;
    }
    let w = warps.wrapping_add(i.wrapping_mul(WARP_SIZE));
    if mem.r16(w.wrapping_add(6)) != 0 {
        return;
    }
    mem.w16(w.wrapping_add(8), (x << 4).wrapping_add(8) as u16);
    mem.w16(w.wrapping_add(0xA), (y << 4) as u16);
    mem.w16(
        w.wrapping_add(0xC),
        ((z as u32) << 4).wrapping_add(8) as u16,
    );
}

/// C: `FUN_02162740` (unlabeled routine at `0x02162740`): the NPC records (`+0x20`).
pub fn npcs(mem: &Mem, sys: u32) -> u32 {
    mem.r32(sys.wrapping_add(0x20))
}

/// C: `FUN_02162744` (unlabeled routine at `0x02162744`): the NPC count (`u16 +0x14`).
pub fn npc_count(mem: &Mem, sys: u32) -> u32 {
    mem.r16(sys.wrapping_add(0x14)) as u32
}

/// C: `FUN_02162748` (unlabeled routine at `0x02162748`): for `i` below the NPC count and a grid NPC `i` (`u32
/// +0x18 == 0`): dir `+0xC`, x `+0x1C`, y `+0x20` (u32), z `+0x1E`.
pub fn npc_set_pos(mem: &mut Mem, sys: u32, i: u32, dir: u32, x: u32, y: u32, z: u16) {
    if i >= mem.r16(sys.wrapping_add(0x14)) as u32 {
        return;
    }
    let n = mem
        .r32(sys.wrapping_add(0x20))
        .wrapping_add(i.wrapping_mul(NPC_SIZE));
    if mem.r32(n.wrapping_add(0x18)) != 0 {
        return;
    }
    mem.w16(n.wrapping_add(0xC), dir as u16);
    mem.w16(n.wrapping_add(0x1C), x as u16);
    mem.w32(n.wrapping_add(0x20), y);
    mem.w16(n.wrapping_add(0x1E), z);
}

/// C: `FUN_02162774` (unlabeled routine at `0x02162774`): for `i` below the NPC count, NPC `i`'s OBJ code (`+2`) =
/// `code`.
pub fn npc_set_obj_code(mem: &mut Mem, sys: u32, i: u32, code: u32) {
    if i < mem.r16(sys.wrapping_add(0x14)) as u32 {
        let n = mem
            .r32(sys.wrapping_add(0x20))
            .wrapping_add(i.wrapping_mul(NPC_SIZE));
        mem.w16(n.wrapping_add(2), code as u16);
    }
}

/// C: `FUN_02162788` (unlabeled routine at `0x02162788`): for `i` below the NPC count, NPC `i`'s event id (script,
/// `+0xA`) = `id`.
pub fn npc_set_script(mem: &mut Mem, sys: u32, i: u32, id: u32) {
    if i < mem.r16(sys.wrapping_add(0x14)) as u32 {
        let n = mem
            .r32(sys.wrapping_add(0x20))
            .wrapping_add(i.wrapping_mul(NPC_SIZE));
        mem.w16(n.wrapping_add(0xA), id as u16);
    }
}

/// C: `FUN_0216279C` (unlabeled routine at `0x0216279C`): the trigger kind of a facing: 0 / 1 / 2 / 3 -> 1 / 2 / 3 /
/// 4, else 0.
pub fn dir_trigger_kind(dir: u32) -> u32 {
    match dir {
        0..=3 => dir + 1,
        _ => 0,
    }
}

/// The trigger loop of `021627C8` / `0216282C` / `02162890`: the first trigger that `hit` accepts, whose kind
/// (`+6`) is `kind` and whose var (`+4`) is 0 or holds the value `+2`.
fn find_trigger<E: FieldEnv>(
    env: &mut E,
    sys: u32,
    ew: u32,
    kind: u32,
    hit: impl Fn(&Mem, u32) -> bool,
) -> u32 {
    let mut t = env.r32(sys.wrapping_add(0x28));
    if t == 0 {
        return 0;
    }
    let n = env.r16(sys.wrapping_add(0x18));
    let mut i: u16 = 0;
    while i < n {
        if hit(env.mem(), t) && env.r16(t.wrapping_add(6)) as u32 == kind {
            let var = env.r16(t.wrapping_add(4)) as u32;
            if var == 0 {
                return t;
            }
            let p = env.callv(VAR_ADDR, &[ew, var]);
            if env.r16(p) == env.r16(t.wrapping_add(2)) {
                return t;
            }
        }
        i = i.wrapping_add(1);
        t = t.wrapping_add(TRIGGER_SIZE);
    }
    0
}

/// C: `FUN_021627C8` (unlabeled routine at `0x021627C8`): the first trigger that contains `pos`
/// ([`trigger_contains`]), whose kind is the facing's ([`dir_trigger_kind`]) and whose var condition holds (var 0,
/// or `*FUN_020143CC(ew, var) == value`); 0 when none.
pub fn trigger_at<E: FieldEnv>(env: &mut E, sys: u32, ew: u32, pos: u32, dir: u32) -> u32 {
    if env.r32(sys.wrapping_add(0x28)) == 0 {
        return 0;
    }
    let k = dir_trigger_kind(dir);
    find_trigger(env, sys, ew, k, |m, t| trigger_contains(m, t, pos))
}

/// C: `FUN_0216282C` (unlabeled routine at `0x0216282C`): [`trigger_at`] with [`trigger_contains_2d`] (no height
/// check).
pub fn trigger_at_2d<E: FieldEnv>(env: &mut E, sys: u32, ew: u32, pos: u32, dir: u32) -> u32 {
    if env.r32(sys.wrapping_add(0x28)) == 0 {
        return 0;
    }
    let k = dir_trigger_kind(dir);
    find_trigger(env, sys, ew, k, |m, t| trigger_contains_2d(m, t, pos))
}

/// C: `FUN_02162890` (unlabeled routine at `0x02162890`): [`trigger_at`] for the triggers of kind 6.
pub fn trigger6_at<E: FieldEnv>(env: &mut E, sys: u32, ew: u32, pos: u32) -> u32 {
    find_trigger(env, sys, ew, 6, |m, t| trigger_contains(m, t, pos))
}

/// C: `FUN_021628E4` (unlabeled routine at `0x021628E4`): the first trigger that contains the rail location
/// ([`trigger_contains_rail`]) and whose var holds its value (`*FUN_020143CC(ew, var) == value`, var 0
/// included); 0 when none.
pub fn trigger_at_rail<E: FieldEnv>(env: &mut E, sys: u32, ew: u32, rail: u32) -> u32 {
    let mut t = env.r32(sys.wrapping_add(0x28));
    if t == 0 {
        return 0;
    }
    let n = env.r16(sys.wrapping_add(0x18));
    let mut i: u16 = 0;
    while i < n {
        if trigger_contains_rail(env.mem(), t, rail) {
            let var = env.r16(t.wrapping_add(4)) as u32;
            let p = env.callv(VAR_ADDR, &[ew, var]);
            if env.r16(p) == env.r16(t.wrapping_add(2)) {
                return t;
            }
        }
        i = i.wrapping_add(1);
        t = t.wrapping_add(TRIGGER_SIZE);
    }
    0
}

fn script_or_none<E: FieldEnv>(env: &mut E, t: u32) -> u32 {
    if t == 0 {
        NONE
    } else {
        env.r16(t) as u32
    }
}

/// C: `FUN_02162928` (unlabeled routine at `0x02162928`): the script of [`trigger_at`], or 0xFFFF.
pub fn trigger_script_at<E: FieldEnv>(env: &mut E, sys: u32, ew: u32, pos: u32, dir: u32) -> u32 {
    let t = trigger_at(env, sys, ew, pos, dir);
    script_or_none(env, t)
}

/// C: `FUN_02162940` (unlabeled routine at `0x02162940`): the script of [`trigger6_at`], or 0xFFFF.
pub fn trigger6_script_at<E: FieldEnv>(env: &mut E, sys: u32, ew: u32, pos: u32) -> u32 {
    let t = trigger6_at(env, sys, ew, pos);
    script_or_none(env, t)
}

/// C: `FUN_02162958` (unlabeled routine at `0x02162958`): the script of [`trigger_at_rail`], or 0xFFFF.
pub fn trigger_script_at_rail<E: FieldEnv>(env: &mut E, sys: u32, ew: u32, rail: u32) -> u32 {
    let t = trigger_at_rail(env, sys, ew, rail);
    script_or_none(env, t)
}

/// C: `FUN_02162970` (unlabeled routine at `0x02162970`): the script of the first background event at `pos`
/// (grid records [`bg_contains`], rail records [`bg_contains_rail`], `pos` read as either) that applies: a hidden
/// item (kind 2) whose flag (`FUN_02164708(script)`, `FUN_020142E8(ew, ..)`) is clear, or another kind whose
/// facing rule (`+4`) accepts `dir` (0: dir 0, 1: dir 3, 2: dir 2, 3: dir 1, 4: any, 5: dir 2 / 3, 6: dir 0 / 1);
/// else 0xFFFF.
pub fn bg_script_at<E: FieldEnv>(env: &mut E, sys: u32, ew: u32, pos: u32, dir: u32) -> u32 {
    let mut b = env.r32(sys.wrapping_add(0x1C));
    if b == 0 {
        return NONE;
    }
    let n = env.r16(sys.wrapping_add(0x12));
    let d2 = dir.wrapping_add(0xFFFE) & 0xFFFF;
    let mut i: u16 = 0;
    while i < n {
        let hit = if env.r16(b.wrapping_add(6)) == 0 {
            bg_contains(env.mem(), b, pos)
        } else {
            bg_contains_rail(env.mem(), b, pos)
        };
        if hit {
            if env.r16(b.wrapping_add(2)) == 2 {
                let s = env.r16(b) as u32;
                let f = env.callv(HIDDEN_ITEM_FLAG, &[s]);
                if env.callv(CHECK_FLAG, &[ew, f]) == 0 {
                    return env.r16(b) as u32;
                }
            } else {
                let ok = match env.r16(b.wrapping_add(4)) {
                    0 => dir == 0,
                    1 => dir == 3,
                    2 => dir == 2,
                    3 => dir == 1,
                    4 => true,
                    5 => d2 <= 1,
                    6 => dir <= 1,
                    _ => false,
                };
                if ok {
                    return env.r16(b) as u32;
                }
            }
        }
        i = i.wrapping_add(1);
        b = b.wrapping_add(BG_SIZE);
    }
    NONE
}

/// C: `FUN_02162A54` (unlabeled routine at `0x02162A54`): [`bg_script_at`].
pub fn bg_script_at_a<E: FieldEnv>(env: &mut E, sys: u32, ew: u32, pos: u32, dir: u32) -> u32 {
    bg_script_at(env, sys, ew, pos, dir)
}

/// C: `FUN_02162A5C` (unlabeled routine at `0x02162A5C`): [`bg_script_at`].
pub fn bg_script_at_b<E: FieldEnv>(env: &mut E, sys: u32, ew: u32, pos: u32, dir: u32) -> u32 {
    bg_script_at(env, sys, ew, pos, dir)
}

/// C: `FUN_02162A74` (unlabeled routine at `0x02162A74`): for `i <= bg count` (sic) and a grid record `i`: tile x
/// `+8 = x`, tile z `+0xC = z`, `+0x10 = y << 4`.
pub fn bg_set_pos(mem: &mut Mem, sys: u32, i: u32, x: u32, y: u32, z: u16) {
    if (mem.r16(sys.wrapping_add(0x12)) as u32) < i {
        return;
    }
    let bgs = mem.r32(sys.wrapping_add(0x1C));
    if bgs == 0 {
        return;
    }
    let b = bgs.wrapping_add(i.wrapping_mul(BG_SIZE));
    if mem.r16(b.wrapping_add(6)) != 0 {
        return;
    }
    mem.w32(b.wrapping_add(8), x);
    mem.w32(b.wrapping_add(0xC), z as u32);
    mem.w32(b.wrapping_add(0x10), y << 4);
}

/// C: `FUN_02162AA4` (unlabeled routine at `0x02162AA4`): map the door offset of the exit code `code` (offset
/// `code & 0xF`, width `(code >> 4) & 0xF`, direction `(code >> 8) & 0xF`) onto a door of width `n`: 0 for code 0;
/// the offset is mirrored (`w - 1 - o`) when (direction, `dir`) is (0, 3), (1, 2), (3, 0) or (2, 1), and again when
/// `flip` and `mode` 0 with direction 2 / 3, mode 1 with direction 0 / 1, or mode 2; then re-centred from width `w`
/// to `n` and clamped to `0..n - 1`.
pub fn door_offset(code: u32, dir: u32, flip: u32, mode: u32, n: u16) -> u32 {
    if code == 0 {
        return 0;
    }
    let mut o = code & 0xF;
    let w = (code & 0xF0) >> 4;
    let d = (code & 0xF00) >> 8;
    if (d == 0 && dir == 3) || (d == 1 && dir == 2) || (d == 3 && dir == 0) || (d == 2 && dir == 1)
    {
        o = w.wrapping_sub(1).wrapping_sub(o);
    }
    if flip != 0 {
        let m = match mode {
            0 => d.wrapping_sub(2) <= 1,
            1 => d <= 1,
            2 => true,
            _ => false,
        };
        if m {
            o = w.wrapping_sub(1).wrapping_sub(o);
        }
    }
    let n = n as u32;
    let mut half = w >> 1;
    let nh = n >> 1;
    let diff = n.wrapping_sub(w) as i32;
    if diff != 0 {
        if diff % 2 == 0 {
            o = nh.wrapping_add(o.wrapping_sub(half));
        } else if !n.is_multiple_of(2) {
            if o < half {
                half = half.wrapping_sub(1);
            }
            o = nh.wrapping_add(o.wrapping_sub(half));
        } else if o > half {
            o = nh.wrapping_add(o.wrapping_sub(half));
        } else {
            o = nh.wrapping_sub(1).wrapping_add(o.wrapping_sub(half));
        }
    }
    if (o as i32) < 0 {
        o = 0;
    } else if (o as i32) >= n as i32 {
        o = n.wrapping_sub(1);
    }
    o & 0xFFFF
}

/// C: `FUN_02162B68` (unlabeled routine at `0x02162B68`): the exit code of a grid warp for the position `pos`:
/// with `k` = [`warp_exit_dir`] `<< 8`: width `+0xE > 1`: `w << 4 | ((pos.x >> 12) - x) / 16 | k`; else depth
/// `+0x10 > 1`: the same with z and the depth; else `k | 0x10` (halfword).
pub fn warp_exit_code(mem: &Mem, warp: u32, pos: u32) -> u32 {
    let k = warp_exit_dir(mem, warp);
    for (lo, p, c) in [(0xE, 0, 8), (0x10, 8, 0xC)] {
        let len = mem.r16(warp.wrapping_add(lo)) as u32;
        if len > 1 {
            let dlt =
                unit(mem.r32(pos.wrapping_add(p))).wrapping_sub(s16(mem, warp.wrapping_add(c)));
            let off = (dlt / 16) as u32;
            return ((len << 4) | off | (k << 8)) & 0xFFFF;
        }
    }
    ((k << 8) | 0x10) & 0xFFFF
}

/// C: `FUN_02162BCC` (unlabeled routine at `0x02162BCC`): the exit code of a rail warp for the rail location
/// `rail`: the flip mode `+0x12` mirrors the front (0, 2) and the side (1, 2) offsets; side length `+0x10 > 1`:
/// `len << 4 | rail.side - side start | k`; else front length `+0xE > 1`: the same with the front; else `k | 0x10`.
pub fn warp_exit_code_rail(mem: &Mem, warp: u32, rail: u32) -> u32 {
    let k = warp_exit_dir(mem, warp);
    let (flip_front, flip_side) = match mem.r16(warp.wrapping_add(0x12)) {
        0 => (true, false),
        1 => (false, true),
        2 => (true, true),
        _ => (false, false),
    };
    let side_len = mem.r16(warp.wrapping_add(0x10)) as u32;
    if side_len > 1 {
        let mut o = (s16(mem, rail.wrapping_add(4)) - s16(mem, warp.wrapping_add(0xC))) as u32;
        if flip_side {
            o = side_len.wrapping_sub(1).wrapping_sub(o);
        }
        return ((side_len << 4) | o | (k << 8)) & 0xFFFF;
    }
    let front_len = mem.r16(warp.wrapping_add(0xE)) as u32;
    if front_len > 1 {
        let mut o = (mem.r16(rail.wrapping_add(6)) as u32)
            .wrapping_sub(mem.r16(warp.wrapping_add(0xA)) as u32);
        if flip_front {
            o = front_len.wrapping_sub(1).wrapping_sub(o);
        }
        return ((front_len << 4) | o | (k << 8)) & 0xFFFF;
    }
    ((k << 8) | 0x10) & 0xFFFF
}

/// C: `FUN_02162C54` (unlabeled routine at `0x02162C54`): the exit direction of a warp: its byte `+4` 1 / 2 / 3 /
/// 4 is direction 0 / 1 / 2 / 3; any other value (0 included) is 1.
pub fn warp_exit_dir(mem: &Mem, warp: u32) -> u32 {
    match mem.r8(warp.wrapping_add(4)) {
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        _ => 1,
    }
}

/// C: `FUN_02162C84` (unlabeled routine at `0x02162C84`): the exit position (`VecFx32` at `out`) of a grid warp
/// for the exit code `code`: `(x, y, z) << 12`; width `> 1`: `out.x += door_offset(code, dir, 0, 0, w) << 16`;
/// else depth `> 1`: `out.z += ..` with the depth.
pub fn warp_exit_pos<E: FieldEnv>(env: &mut E, warp: u32, code: u32, out: u32) {
    let m = env.mem();
    let dir = warp_exit_dir(m, warp);
    for (i, o) in [8u32, 0xA, 0xC].into_iter().enumerate() {
        let v = (s16(m, warp.wrapping_add(o)) << 12) as u32;
        m.w32(out.wrapping_add(4 * i as u32), v);
    }
    for (lo, p) in [(0xE, 0), (0x10, 8)] {
        let len = m.r16(warp.wrapping_add(lo));
        if len > 1 {
            let d = door_offset(code, dir, 0, 0, len);
            let a = out.wrapping_add(p);
            let v = m.r32(a).wrapping_add(d << 16);
            m.w32(a, v);
            return;
        }
    }
}

/// C: `FUN_02162CE8` (unlabeled routine at `0x02162CE8`): the centre (`VecFx32` at `out`) of a grid warp: `(x, y,
/// z) << 12`; width `> 1`: `out.x += (w >> 1) << 16`; else depth `> 1`: `out.z += (d >> 1) << 16`.
pub fn warp_centre(mem: &mut Mem, warp: u32, out: u32) {
    let x = (s16(mem, warp.wrapping_add(8)) << 12) as u32;
    mem.w32(out, x);
    let y = (s16(mem, warp.wrapping_add(0xA)) << 12) as u32;
    mem.w32(out.wrapping_add(4), y);
    let z = (s16(mem, warp.wrapping_add(0xC)) << 12) as u32;
    mem.w32(out.wrapping_add(8), z);
    let w = mem.r16(warp.wrapping_add(0xE)) as u32;
    if w > 1 {
        mem.w32(out, x.wrapping_add((w >> 1) << 16));
        return;
    }
    let d = mem.r16(warp.wrapping_add(0x10)) as u32;
    if d > 1 {
        mem.w32(out.wrapping_add(8), z.wrapping_add((d >> 1) << 16));
    }
}

/// C: `FUN_02162D2C` (unlabeled routine at `0x02162D2C`): does the grid warp contain the position `pos`
/// (`VecFx32`)? A rail warp (`+6 == 1`) never does. With `p = pos >> 12` (arithmetic), the warp covers
/// `x <= p.x <= x + 16 (w - 1)`, `y - 2 <= p.y <= y + 2` and `z <= p.z <= z + 16 (d - 1)` (signed, the
/// sums wrap as in the original; a width of 0 makes the range empty).
pub fn warp_contains(mem: &Mem, warp: u32, pos: u32) -> bool {
    if mem.r16(warp.wrapping_add(6)) == 1 {
        return false;
    }
    let px = (mem.r32(pos) as i32) >> 12;
    let py = (mem.r32(pos.wrapping_add(4)) as i32) >> 12;
    let pz = (mem.r32(pos.wrapping_add(8)) as i32) >> 12;
    let x = mem.r16(warp.wrapping_add(8)) as i16 as i32;
    let y = mem.r16(warp.wrapping_add(0xA)) as i16 as i32;
    let z = mem.r16(warp.wrapping_add(0xC)) as i16 as i32;
    let w = mem.r16(warp.wrapping_add(0xE)) as i32;
    let d = mem.r16(warp.wrapping_add(0x10)) as i32;
    x <= px
        && x.wrapping_add((w - 1).wrapping_shl(4)) >= px
        && y - 2 <= py
        && y + 2 >= py
        && z <= pz
        && z.wrapping_add((d - 1).wrapping_shl(4)) >= pz
}

/// C: `FUN_02162D90` (unlabeled routine at `0x02162D90`): the exit rail location (8 bytes at `out`) of a rail warp
/// for the exit code `code`: line `+8`, type 1, key `FUN_0201441C(dir)`, side `+0xC`, front `+0xA`; side length
/// `> 1`: `side += door_offset(code, dir, 1, flip mode, side length)`; else front length `> 1`: `front += ..`.
pub fn warp_exit_rail<E: FieldEnv>(env: &mut E, warp: u32, code: u32, out: u32) {
    let dir = warp_exit_dir(env.mem(), warp);
    let line = env.r16(warp.wrapping_add(8));
    env.w16(out, line);
    env.w8(out.wrapping_add(2), 1);
    let key = env.callv(RAIL_KEY, &[dir]);
    env.w8(out.wrapping_add(3), key as u8);
    let m = env.mem();
    let mode = m.r16(warp.wrapping_add(0x12)) as u32;
    let side = m.r16(warp.wrapping_add(0xC));
    m.w16(out.wrapping_add(4), side);
    let front = m.r16(warp.wrapping_add(0xA));
    m.w16(out.wrapping_add(6), front);
    let sl = m.r16(warp.wrapping_add(0x10));
    if sl > 1 {
        let d = door_offset(code, dir, 1, mode, sl);
        let v = m.r16(out.wrapping_add(4)).wrapping_add(d as u16);
        m.w16(out.wrapping_add(4), v);
        return;
    }
    let fl = m.r16(warp.wrapping_add(0xE));
    if fl > 1 {
        let d = door_offset(code, dir, 1, mode, fl);
        let v = m.r16(out.wrapping_add(6)).wrapping_add(d as u16);
        m.w16(out.wrapping_add(6), v);
    }
}

/// C: `FUN_02162E00` (unlabeled routine at `0x02162E00`): does the rail warp contain the rail location `rail`? A
/// grid warp (`+6 == 0`) never does; same line (`+8`), `front start <= rail.front < start + front length`
/// (unsigned) and `side start <= rail.side < start + side length` (signed).
pub fn warp_contains_rail(mem: &Mem, warp: u32, rail: u32) -> bool {
    if mem.r16(warp.wrapping_add(6)) == 0 {
        return false;
    }
    if mem.r16(warp.wrapping_add(8)) != mem.r16(rail) {
        return false;
    }
    let f = mem.r16(rail.wrapping_add(6)) as u32;
    let f0 = mem.r16(warp.wrapping_add(0xA)) as u32;
    if !(f0 <= f && f < f0 + mem.r16(warp.wrapping_add(0xE)) as u32) {
        return false;
    }
    let s = s16(mem, rail.wrapping_add(4));
    let s0 = s16(mem, warp.wrapping_add(0xC));
    s0 <= s && s < s0 + mem.r16(warp.wrapping_add(0x10)) as i32
}

/// C: `FUN_02162E48` (unlabeled routine at `0x02162E48`): is the grid background event at `pos`? A record with
/// position kind 1 never is; tile x (`+8`) and z (`+0xC`) equal the position's tiles and `y - 2 <= pos.y >> 12 <
/// y + 2` (`+0x10`).
pub fn bg_contains(mem: &Mem, bg: u32, pos: u32) -> bool {
    if mem.r16(bg.wrapping_add(6)) == 1 {
        return false;
    }
    let tz = tile(mem.r32(pos.wrapping_add(8)));
    let py = unit(mem.r32(pos.wrapping_add(4)));
    let tx = tile(mem.r32(pos));
    if mem.r32(bg.wrapping_add(8)) as i32 != tx || mem.r32(bg.wrapping_add(0xC)) as i32 != tz {
        return false;
    }
    let y = mem.r32(bg.wrapping_add(0x10)) as i32;
    y.wrapping_sub(2) <= py && py < y.wrapping_add(2)
}

/// C: `FUN_02162E9C` (unlabeled routine at `0x02162E9C`): is the rail background event at the rail location? A grid
/// record (`+6 == 0`) never is; line, front and side equal.
pub fn bg_contains_rail(mem: &Mem, bg: u32, rail: u32) -> bool {
    if mem.r16(bg.wrapping_add(6)) == 0 {
        return false;
    }
    mem.r16(bg.wrapping_add(8)) == mem.r16(rail)
        && mem.r16(bg.wrapping_add(0xA)) == mem.r16(rail.wrapping_add(6))
        && mem.r16(bg.wrapping_add(0xC)) == mem.r16(rail.wrapping_add(4))
}

/// C: `FUN_02162ECC` (unlabeled routine at `0x02162ECC`): the centre (`VecFx32` at `out`) of a grid trigger:
/// `out.z = (z << 16) + FX_Mul(depth << 16, 0x2000)`, `out.x = (x << 16) + FX_Mul(width << 16, 0x2000)` (in this
/// call order), `out.y = y << 12`.
pub fn trigger_centre<E: FieldEnv>(env: &mut E, t: u32, out: u32) {
    let d = env.r16(t.wrapping_add(0x10)) as u32;
    let hz = env.callv(FX_MUL, &[d << 16, 0x2000]);
    let z = ((env.r16(t.wrapping_add(0xC)) as u32) << 16).wrapping_add(hz);
    let y = (s16(env.mem(), t.wrapping_add(0x12)) << 12) as u32;
    let w = env.r16(t.wrapping_add(0xE)) as u32;
    let hx = env.callv(FX_MUL, &[w << 16, 0x2000]);
    let x = ((env.r16(t.wrapping_add(0xA)) as u32) << 16).wrapping_add(hx);
    env.w32(out.wrapping_add(8), z);
    env.w32(out, x);
    env.w32(out.wrapping_add(4), y);
}

/// The tile rectangle of a grid trigger: `x <= tile x < x + w` and `z <= tile z < z + d`.
fn trigger_rect(mem: &Mem, t: u32, pos: u32) -> bool {
    let tx = tile(mem.r32(pos));
    let tz = tile(mem.r32(pos.wrapping_add(8)));
    let x = mem.r16(t.wrapping_add(0xA)) as i32;
    let z = mem.r16(t.wrapping_add(0xC)) as i32;
    x <= tx
        && x + mem.r16(t.wrapping_add(0xE)) as i32 > tx
        && z <= tz
        && z + mem.r16(t.wrapping_add(0x10)) as i32 > tz
}

/// C: `FUN_02162F0C` (unlabeled routine at `0x02162F0C`): does the grid trigger contain `pos`? Position kind 1
/// never does; the tile rectangle (`+0xA..+0x10`) and `y - 2 <= pos.y >> 12 < y + 2` (`s16 +0x12`).
pub fn trigger_contains(mem: &Mem, t: u32, pos: u32) -> bool {
    if mem.r16(t.wrapping_add(8)) == 1 {
        return false;
    }
    if !trigger_rect(mem, t, pos) {
        return false;
    }
    let py = unit(mem.r32(pos.wrapping_add(4)));
    let y = s16(mem, t.wrapping_add(0x12));
    y - 2 <= py && y + 2 > py
}

/// C: `FUN_02162F74` (unlabeled routine at `0x02162F74`): [`trigger_contains`] without the height check.
pub fn trigger_contains_2d(mem: &Mem, t: u32, pos: u32) -> bool {
    if mem.r16(t.wrapping_add(8)) == 1 {
        return false;
    }
    trigger_rect(mem, t, pos)
}

/// C: `FUN_02162FC8` (unlabeled routine at `0x02162FC8`): does the rail trigger contain the rail location? A grid
/// trigger (`+8 == 0`) never does; same line (`+0xA`), `front <= rail.front < front + len` (`+0xC`, `+0x10`,
/// unsigned) and `side <= rail.side < side + len` (`s16 +0xE`, `+0x12`).
pub fn trigger_contains_rail(mem: &Mem, t: u32, rail: u32) -> bool {
    if mem.r16(t.wrapping_add(8)) == 0 {
        return false;
    }
    if mem.r16(t.wrapping_add(0xA)) != mem.r16(rail) {
        return false;
    }
    let f = mem.r16(rail.wrapping_add(6)) as u32;
    let f0 = mem.r16(t.wrapping_add(0xC)) as u32;
    if !(f0 <= f && f < f0 + mem.r16(t.wrapping_add(0x10)) as u32) {
        return false;
    }
    let s = s16(mem, rail.wrapping_add(4));
    let s0 = s16(mem, t.wrapping_add(0xE));
    s0 <= s && s < s0 + mem.r16(t.wrapping_add(0x12)) as i32
}

#[cfg(test)]
mod tests;
