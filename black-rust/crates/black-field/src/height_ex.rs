//! `height_ex.c` (overlay 21, `0x021C0598..0x021C06CC`) and the unanchored functions after it in F1 row 21.64
//! (`0x021C06CC..0x021C1024`). HEIGHT_EX (8 bytes, 021C05D4): `+0` record count, `+4` -> records of 0x1C bytes:
//! `+0` tile x, `+4` tile z, `+8` width, `+0xC` depth (tiles), `+0x10` height value, `+0x14` in use, `+0x18` extra;
//! fieldmap (0218DDxx) asks it for the height of a tile inside a rectangle. The rest of the range: GMEVENTs that
//! animate build models (started by script commands 021B2ACC / 021B2B14), a script-callback event (021C06CC), and the
//! boulder (strength) push: the step check 021C0CA4 that field_player 021A2F20 calls and its event 021C0E98.
//!
//! F3a-spec specced all 38 functions (`tools/rom-diff/specs/black-field/height_ex.spec`, header: layouts); F3a ported
//! them. Every callee outside the area (heap, GMEVENT, GAMESYS / GAMEDATA getters, fieldmap, buildmodel, fldmmdl,
//! mmdl_move, the map attribute queries, the player areas, sound, the matrix helpers) is reached through
//! [`FieldEnv::call`] by address (D3: the build models and SE stay addresses); in-area callees are Rust calls. The
//! callers' stack locals (positions, the search rectangle, the rotation matrix) are local byte arrays.
//!
//! Build-model event work (0x38 / 0x3C, `020124E8`): `+0 u16` heap, `+4` FIELDMAP, `+8` VecFx32 model position,
//! `+0x14 u8` models to add (<= 6), `+0x15 u8` models added, `+0x16 u16` frame counter, `+0x18` buildmodel system,
//! `+0x1C` u32 model objects [6], `+0x34` found model id, `+0x38` its object (0x3C variant).
//! Boulder push event work (0x24): `+0` PLAYER GRID, `+4 u16` dir, `+6 u16` fall into a hole, `+8 s16` frame, `+0xC`
//! target height, `+0x10` VecFx32 draw offset, `+0x14` shake, `+0x1C` boulder MMDL, `+0x20` player MMDL.

use crate::hooks::{CArg, FieldEnv, Mem};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x021C_0598,
    0x021C_05D4,
    0x021C_0628,
    0x021C_0674,
    0x021C_0688,
    0x021C_068C,
    0x021C_06A8,
    0x021C_06B4,
    0x021C_06C0,
    0x021C_06CC,
    0x021C_06E0,
    0x021C_0780,
    0x021C_0808,
    0x021C_080C,
    0x021C_0810,
    0x021C_0820,
    0x021C_0844,
    0x021C_0904,
    0x021C_0988,
    0x021C_09A8,
    0x021C_0A10,
    0x021C_0A48,
    0x021C_0A70,
    0x021C_0AA0,
    0x021C_0AC0,
    0x021C_0B3C,
    0x021C_0BD8,
    0x021C_0C48,
    0x021C_0C80,
    0x021C_0C94,
    0x021C_0CA4,
    0x021C_0CE4,
    0x021C_0D08,
    0x021C_0DDC,
    0x021C_0E4C,
    0x021C_0E98,
];

/// Size of a HEIGHT_EX record.
pub const RECORD_SIZE: u32 = 0x1C;

/// Address of the string `"height_ex.c"` (the file argument of the allocations in `FUN_021C05D4`).
pub const FILE_HEIGHT_EX: u32 = 0x021D_E3A0;

/// Address of [`MODEL_OFFSETS`] in the ROM (overlay 21 data).
pub const MODEL_OFFSETS_BASE: u32 = 0x021D_A098;
/// ROM `0x021DA098..0x021DA140`: the two tables of 6 VecFx32 model offsets (0x021DA098 for `FUN_021C09B0`, 0x021DA0E0
/// for `FUN_021C0BE8`) and the first words after them: `FUN_021C09B0` / `FUN_021C0BE8` hand `table + 12 * i` to the
/// matrix multiply, which reads 0x24 bytes there.
pub const MODEL_OFFSETS: [u8; 0xA8] = [
    0x00, 0xC0, 0xFF, 0xFF, 0x00, 0xE0, 0x00, 0x00, 0x00, 0xD0, 0xFF, 0xFF, 0x00, 0x40, 0x00, 0x00,
    0x00, 0xE0, 0x00, 0x00, 0x00, 0xD0, 0xFF, 0xFF, 0x00, 0x90, 0x00, 0x00, 0x00, 0xE0, 0x00, 0x00,
    0x00, 0x40, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0xE0, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00,
    0x00, 0xC0, 0xFF, 0xFF, 0x00, 0xE0, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x70, 0xFF, 0xFF,
    0x00, 0xE0, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0xC0, 0xFF, 0xFF, 0x00, 0xE0, 0x00, 0x00,
    0x00, 0xD0, 0xFF, 0xFF, 0x00, 0x40, 0x00, 0x00, 0x00, 0xE0, 0x00, 0x00, 0x00, 0xD0, 0xFF, 0xFF,
    0x00, 0x90, 0x00, 0x00, 0x00, 0xE0, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00,
    0x00, 0xE0, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0xC0, 0xFF, 0xFF, 0x00, 0xE0, 0x00, 0x00,
    0x00, 0x80, 0x00, 0x00, 0x00, 0x70, 0xFF, 0xFF, 0x00, 0xE0, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00,
    0x19, 0x13, 0x1C, 0x02, 0x6D, 0x14, 0x1C, 0x02, 0xF9, 0x14, 0x1C, 0x02, 0x7D, 0x15, 0x1C, 0x02,
    0x1D, 0x16, 0x1C, 0x02, 0x71, 0x16, 0x1C, 0x02,
];

// other units, named by address (meanings from the spec)
const HEAP_ALLOC: u32 = 0x0203_0734; // Heap_AllocDebug(heap, size, flag, file, line)
const HEAP_FREE: u32 = 0x0203_07B0; // Heap_Free(block)
const SE_PLAY: u32 = 0x0200_61E4; // sound: play SE (id)
const SYS_0200FB38: u32 = 0x0200_FB38; // main: system object (no arguments)
const SYS_0200FBA0: u32 = 0x0200_FBA0; // main (no arguments)
const GSYS_GAMEDATA: u32 = 0x0201_20F4; // GAMESYS -> GAMEDATA
const GSYS_FIELDMAP: u32 = 0x0201_210C; // GAMESYS -> FIELDMAP
const GSYS_SET_FLAG: u32 = 0x0201_2124; // GAMESYS: set flag (gsys, 0)
const GSYS_GET: u32 = 0x0201_2138; // GAMESYS getter
const GSYS_PRED: u32 = 0x0201_21CC; // GAMESYS predicate
const EVENT_CREATE: u32 = 0x0201_22C0; // GMEVENT create (gsys, parent, proc, work size)
const EVENT_CALL: u32 = 0x0201_2374; // GMEVENT call (event, child)
const EVENT_GSYS: u32 = 0x0201_24E4; // GMEVENT -> GAMESYS
const EVENT_WORK: u32 = 0x0201_24E8; // GMEVENT work (event)
const GAMEDATA_GET: u32 = 0x0201_2934; // GAMEDATA getter
const GAMEDATA_BYTE: u32 = 0x0201_2B70; // GAMEDATA byte getter
const SCRIPT_EVENT: u32 = 0x0201_46F8; // script event create (gsys, fieldmap, 21, callbacks, gsys)
const COUNT_0201AA1C: u32 = 0x0201_AA1C; // count (u8)
const PRED_02026454: u32 = 0x0202_6454; // predicate
const OBJ_02026474: u32 = 0x0202_6474; // object or null
const MODE_0202647C: u32 = 0x0202_647C; // mode
const EVENT_02029620: u32 = 0x0202_9620; // event constructor (gsys, 0, 6)
const EVENT_02029830: u32 = 0x0202_9830; // event constructor (gsys, 0x514)
const F_0203CD98: u32 = 0x0203_CD98; // (no arguments)
const F_0203CDBC: u32 = 0x0203_CDBC; // polled (no arguments)
const F_0203CEAC: u32 = 0x0203_CEAC; // (0)
const MTX_ROT: u32 = 0x0205_20FC; // MTX_Rot(x, y, z, &mtx33)
const MTX_MULT_VEC33: u32 = 0x0207_C9D0; // MTX_MultVec33(&v, &m, &out)
const HIT_CHECK: u32 = 0x0216_39C0; // mmdl_move: hit check in dir (m, dir)
const HEIGHT_AT: u32 = 0x0216_3E58; // mmdl_move: height at a position (m, &pos, &y)
const STEP_POS: u32 = 0x0216_4208; // mmdl_move: move a VecFx32 one step in dir (dir, &pos, amount)
const EVDATA_02164FC4: u32 = 0x0216_4FC4; // eventdata_tail: (obj)
const ACMD_SETTABLE: u32 = 0x0216_D3CC; // fldmmdl: MMDL acmd settable
const ACMD_SET: u32 = 0x0216_D410; // fldmmdl: MMDL set acmd (m, code)
const ACMD_DONE: u32 = 0x0216_D454; // fldmmdl: MMDL acmd done
const OBJ_CODE: u32 = 0x0216_D598; // fldmmdl: MMDL OBJ code
const GET_POS: u32 = 0x0216_D88C; // fldmmdl: MMDL position VecFx32 -> out
const SET_POS: u32 = 0x0216_D89C; // fldmmdl: MMDL position VecFx32 = *v
const SET_DRAW_OFS: u32 = 0x0216_D8E0; // fldmmdl: MMDL draw offset VecFx32 = *v
const SIZE_X: u32 = 0x0216_D91C; // fldmmdl: MMDL size x (u8)
const SIZE_Z: u32 = 0x0216_D924; // fldmmdl: MMDL size z (u8)
const SYS_PAUSE: u32 = 0x0216_D9A0; // fldmmdl: MMDLSYS pause all
const SYS_RESUME: u32 = 0x0216_D9E4; // fldmmdl: MMDLSYS resume all
const MOVE_BIT: u32 = 0x0216_DAE0; // fldmmdl: MMDL move bit set/clear (m, on)
const DIR_LOCK: u32 = 0x0216_DB4C; // fldmmdl: MMDL dir lock (m, on)
const SET_POS_DIR: u32 = 0x0216_E014; // fldmmdl: MMDL set position + dir (m, &pos, dir)
const REFRESH: u32 = 0x0216_E57C; // fldmmdl: MMDL refresh (m)
const PUSHABLE: u32 = 0x0216_E5FC; // fldmmdl: OBJ code predicate (pushable boulder?)
const FM_PRED: u32 = 0x0218_8BE8; // fieldmap predicate
const FM_MMDLSYS: u32 = 0x0218_8C7C; // fieldmap +0x38 (MMDLSYS)
const FM_GAMESYS: u32 = 0x0218_8C80; // fieldmap +4 (GAMESYS)
const FM_HEAP: u32 = 0x0218_8C84; // fieldmap heap id
const FM_MAP: u32 = 0x0218_8C98; // fieldmap +0x48 (map draw object)
const FM_PLAYER: u32 = 0x0218_8CB4; // fieldmap +0x8C (FIELD_PLAYER)
const BM_FIND: u32 = 0x0218_C584; // buildmodel: find models in a rect (map, &rect, kind, &n)
const BM_0218C69C: u32 = 0x0218_C69C; // buildmodel (map, id, 0, 0)
const BM_POS: u32 = 0x0218_C6D0; // buildmodel: model position (id, &out)
const BM_0218C724: u32 = 0x0218_C724; // buildmodel (map, id)
const BM_CREATE: u32 = 0x0218_C7F0; // buildmodel: create (map, 0x62, &pos)
const BM_DELETE: u32 = 0x0218_C860; // buildmodel: delete (obj)
const BM_ANIM: u32 = 0x0218_C88C; // buildmodel: animation (obj, i, n)
const BM_ANIM_DONE_1: u32 = 0x0218_C8D8; // buildmodel: animation done (obj)
const BM_ANIM_DONE: u32 = 0x0218_C8FC; // buildmodel: animation done (obj, i)
const MAP_BM_SYS: u32 = 0x0218_DC6C; // map: buildmodel system (draw object)
const ACMD_CODE: u32 = 0x0219_7E7C; // fldmmdl_acmd: acmd code of (dir, base)
const PLAYER_POS: u32 = 0x021A_3038; // player: position (player, &pos)
const CORE_FIELDMAP: u32 = 0x021A_395C; // player_core: FIELDMAP
const CORE_MMDL: u32 = 0x021A_3960; // player_core: the player MMDL
const CORE_GAMESYS: u32 = 0x021A_3990; // player_core: GAMESYS
const CORE_KEY_DIR: u32 = 0x021A_3AB8; // player_core: dir of keys (core, keys)
const GRID_448C: u32 = 0x021A_448C; // player_grid (grid, dir)
const GRID_44BC: u32 = 0x021A_44BC; // player_grid (grid)
const GRID_5408: u32 = 0x021A_5408; // player_grid (grid)
const GRID_CORE: u32 = 0x021A_5728; // player_grid: CORE
const GRID_MMDL: u32 = 0x021A_572C; // player_grid: the player MMDL
const ATTR_AT: u32 = 0x021A_B0D4; // map attribute at a position (map, &pos)
const ATTR_VALUE: u32 = 0x021A_B0F0; // attribute value of an attribute
const ATTR_P_11C: u32 = 0x021A_B11C; // map attribute predicate
const ATTR_P_2DC: u32 = 0x021A_B2DC; // map attribute predicate
const FRONT_MMDL: u32 = 0x021C_19B4; // player_event: MMDL in front (grid, dir)

fn c<E: FieldEnv>(env: &mut E, addr: u32, a: &[u32]) -> u32 {
    env.callv(addr, a)
}

fn rd32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}

fn wr32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

fn record(mem: &Mem, hx: u32, i: u32) -> u32 {
    mem.r32(hx.wrapping_add(4))
        .wrapping_add(i.wrapping_mul(RECORD_SIZE))
}

/// C: `FUN_021C0598` (unlabeled routine at `0x021C0598`): record `rec` is in use (`+0x14 != 0`) and tile `(x, z)`
/// lies inside its rectangle `x0 <= x <= x0 + w - 1`, `z0 <= z <= z0 + d - 1` (signed compares, wrapping sums).
pub fn rect_contains(mem: &Mem, x: u32, z: u32, rec: u32) -> bool {
    if mem.r32(rec.wrapping_add(0x14)) == 0 {
        return false;
    }
    let x0 = mem.r32(rec) as i32;
    let z0 = mem.r32(rec.wrapping_add(4)) as i32;
    let x1 = x0
        .wrapping_add(mem.r32(rec.wrapping_add(8)) as i32)
        .wrapping_sub(1);
    let z1 = z0
        .wrapping_add(mem.r32(rec.wrapping_add(0xC)) as i32)
        .wrapping_sub(1);
    let (x, z) = (x as i32, z as i32);
    x0 <= x && x <= x1 && z0 <= z && z <= z1
}

/// C: `FUN_021C05D4` (unlabeled routine at `0x021C05D4`): create a HEIGHT_EX of `n` records on `heap`:
/// `Heap_AllocDebug(heap, 8, 0, "height_ex.c", 0x5A)`, records `Heap_AllocDebug(heap, n * 0x1C, 0, .., 0x5B)` at
/// `+4`, `+0` = n, every record's `+0x14` (in use) = 0 (u8 counter); returns the HEIGHT_EX.
pub fn create<E: FieldEnv>(env: &mut E, n: u32, heap: u32) -> u32 {
    let hx = c(env, HEAP_ALLOC, &[heap, 8, 0, FILE_HEIGHT_EX, 0x5A]);
    let recs = c(
        env,
        HEAP_ALLOC,
        &[heap, n.wrapping_mul(RECORD_SIZE), 0, FILE_HEIGHT_EX, 0x5B],
    );
    env.w32(hx.wrapping_add(4), recs);
    env.w32(hx, n);
    let mut i: u8 = 0;
    while (i as u32) < n {
        let r = env.r32(hx.wrapping_add(4));
        env.w32(
            r.wrapping_add((i as u32).wrapping_mul(RECORD_SIZE))
                .wrapping_add(0x14),
            0,
        );
        i = i.wrapping_add(1);
    }
    hx
}

/// C: `FUN_021C0628` (unlabeled routine at `0x021C0628`): when `i < count` (signed) and record `i` is not in use:
/// `+0` x, `+4` z, `+8` w, `+0xC` d, `+0x10` h, `+0x18` k, `+0x14` = 1 (no caller in the field set).
#[allow(clippy::too_many_arguments)]
pub fn set_record(mem: &mut Mem, i: u32, x: u32, z: u32, w: u32, d: u32, h: u32, k: u32, hx: u32) {
    if (i as i32) >= (mem.r32(hx) as i32) {
        return;
    }
    let o = i.wrapping_mul(RECORD_SIZE);
    let rec = mem.r32(hx.wrapping_add(4)).wrapping_add(o);
    if mem.r32(rec.wrapping_add(0x14)) != 0 {
        return;
    }
    mem.w32(rec, x);
    for (ofs, v) in [(4, z), (8, w), (0xC, d), (0x10, h), (0x18, k), (0x14, 1)] {
        let r = mem.r32(hx.wrapping_add(4)).wrapping_add(o);
        mem.w32(r.wrapping_add(ofs), v);
    }
}

/// C: `FUN_021C0674` (unlabeled routine at `0x021C0674`): free the records (`+4`) and the HEIGHT_EX.
pub fn delete<E: FieldEnv>(env: &mut E, hx: u32) {
    let r = env.r32(hx.wrapping_add(4));
    c(env, HEAP_FREE, &[r]);
    c(env, HEAP_FREE, &[hx]);
}

/// C: `FUN_021C0688` (unlabeled routine at `0x021C0688`): the record count `+0`.
pub fn count(mem: &Mem, hx: u32) -> u32 {
    mem.r32(hx)
}

/// C: `FUN_021C068C` (unlabeled routine at `0x021C068C`): [`rect_contains`] for record `i` (no index check).
pub fn record_contains(mem: &Mem, x: u32, z: u32, hx: u32, i: u32) -> bool {
    rect_contains(mem, x, z, record(mem, hx, i))
}

/// C: `FUN_021C06A8` (unlabeled routine at `0x021C06A8`): the height value `+0x10` of record `i`.
pub fn height(mem: &Mem, i: u32, hx: u32) -> u32 {
    mem.r32(record(mem, hx, i).wrapping_add(0x10))
}

/// C: `FUN_021C06B4` (unlabeled routine at `0x021C06B4`): the extra value `+0x18` of record `i`.
pub fn extra(mem: &Mem, i: u32, hx: u32) -> u32 {
    mem.r32(record(mem, hx, i).wrapping_add(0x18))
}

/// C: `FUN_021C06C0` (unlabeled routine at `0x021C06C0`): set the height value `+0x10` of record `i` (no caller in
/// the field set).
pub fn set_height(mem: &mut Mem, i: u32, v: u32, hx: u32) {
    let r = record(mem, hx, i);
    mem.w32(r.wrapping_add(0x10), v);
}

/// C: `FUN_021C06CC` (unlabeled routine at `0x021C06CC`): create the script-callback GMEVENT
/// `FUN_020122C0(gsys, 0, FUN_021C06E0, 0)`.
pub fn callback_event_new<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    c(env, EVENT_CREATE, &[gsys, 0, 0x021C_06E1, 0])
}

/// C: `FUN_021C06E0` (unlabeled routine at `0x021C06E0`): the script-callback GMEVENT proc. Step 0: pause the MMDLs
/// (`FUN_0216D9A0`); with mode `FUN_0202647C(FUN_02012138(gsys)) == 2` and the GAMEDATA byte `FUN_02012B70 != 1` the
/// child is the GMEVENT `FUN_021C0780`, else `FUN_0200FB38()` and the script event `FUN_020146F8(gsys, fieldmap, 21,
/// ROM 0x021DA08C, gsys)`; call it (`FUN_02012374`), step 1. Step 1: with a FIELDMAP and `FUN_02188BE8(fm) == 1`
/// resume the MMDLs and answer 1 (done). Other steps answer 0.
pub fn callback_event<E: FieldEnv>(env: &mut E, ev: u32, seq: u32) -> u32 {
    let gsys = c(env, EVENT_GSYS, &[ev]);
    let fm = c(env, GSYS_FIELDMAP, &[gsys]);
    match env.r32(seq) {
        0 => {
            let sys = c(env, FM_MMDLSYS, &[fm]);
            c(env, SYS_PAUSE, &[sys]);
            let g = c(env, GSYS_GET, &[gsys]);
            let script = if c(env, MODE_0202647C, &[g]) == 2 {
                let gd = c(env, GSYS_GAMEDATA, &[gsys]);
                c(env, GAMEDATA_BYTE, &[gd]) == 1
            } else {
                true
            };
            let child = if script {
                c(env, SYS_0200FB38, &[]);
                c(env, SCRIPT_EVENT, &[gsys, fm, 21, 0x021D_A08C, gsys])
            } else {
                c(env, EVENT_CREATE, &[gsys, 0, 0x021C_0781, 0])
            };
            c(env, EVENT_CALL, &[ev, child]);
            let s = env.r32(seq);
            env.w32(seq, s.wrapping_add(1));
            0
        }
        1 => {
            if fm != 0 && c(env, FM_PRED, &[fm]) == 1 {
                let sys = c(env, FM_MMDLSYS, &[fm]);
                c(env, SYS_RESUME, &[sys]);
                return 1;
            }
            0
        }
        _ => 0,
    }
}

/// C: `FUN_021C0780` (unlabeled routine at `0x021C0780`): the child GMEVENT of [`callback_event`]. Step 0:
/// `FUN_0203CD98() != 0`: `FUN_0203CEAC(0)`; step 1. Step 1: `FUN_0203CDBC() == 1`: `FUN_0200FBA0()`,
/// `FUN_02012124(gsys, 0)`, step 2. Step 2: `FUN_020121CC(gsys) == 1`: step 3, else answer 1. Step 3:
/// `o = FUN_02026474(g)` (`g` = `FUN_02012138(gsys)`, left in r0 by the step dispatch); `o` and `FUN_02026454(g)`:
/// `FUN_02164FC4(o)`, answer 1. Otherwise 0.
pub fn callback_child<E: FieldEnv>(env: &mut E, ev: u32, seq: u32) -> u32 {
    let gsys = c(env, EVENT_GSYS, &[ev]);
    let g = c(env, GSYS_GET, &[gsys]);
    let next = |env: &mut E| {
        let s = env.r32(seq);
        env.w32(seq, s.wrapping_add(1));
    };
    match env.r32(seq) {
        0 => {
            if c(env, F_0203CD98, &[]) != 0 {
                c(env, F_0203CEAC, &[0]);
            }
            next(env);
        }
        1 => {
            if c(env, F_0203CDBC, &[]) == 1 {
                c(env, SYS_0200FBA0, &[]);
                c(env, GSYS_SET_FLAG, &[gsys, 0]);
                next(env);
            }
        }
        2 => {
            if c(env, GSYS_PRED, &[gsys]) == 1 {
                next(env);
            } else {
                return 1;
            }
        }
        3 => {
            let o = c(env, OBJ_02026474, &[g]);
            if o != 0 && c(env, PRED_02026454, &[g]) != 0 {
                c(env, EVDATA_02164FC4, &[o]);
                return 1;
            }
        }
        _ => {}
    }
    0
}

/// C: `FUN_021C0808` (unlabeled routine at `0x021C0808`): a callback of ROM 0x021DA08C: answers 1.
pub fn callback_true() -> u32 {
    1
}

/// C: `FUN_021C080C` (unlabeled routine at `0x021C080C`): a callback of ROM 0x021DA08C: answers 1.
pub fn callback_true_2() -> u32 {
    1
}

/// C: `FUN_021C0810` (unlabeled routine at `0x021C0810`): a callback of ROM 0x021DA08C: `FUN_02012124(gsys, 0)`,
/// answers 1.
pub fn callback_end<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    c(env, GSYS_SET_FLAG, &[gsys, 0]);
    1
}

/// C: `FUN_021C0820` (unlabeled routine at `0x021C0820`): create the build-model GMEVENT `FUN_021C0844` (work 0x38)
/// as a child of `parent` and set it up for `n` models ([`models_init`]); returns the event.
pub fn models_event_new<E: FieldEnv>(env: &mut E, gsys: u32, parent: u32, n: u32) -> u32 {
    let ev = c(env, EVENT_CREATE, &[gsys, parent, 0x021C_0845, 0x38]);
    let w = c(env, EVENT_WORK, &[ev]);
    models_init(env, w, gsys, n);
    ev
}

/// C: `FUN_021C0844` (unlabeled routine at `0x021C0844`): the build-model GMEVENT proc. Step 0: every 15th frame add
/// a model ([`models_add`]); all added (`+0x14 <= +0x15`): step 1. Step 1: after 10 frames start the animations
/// ([`models_anim_start`]) and call the event `FUN_02029830(GAMESYS, 0x514)`, step 2. Step 2: animations done
/// ([`models_anim_done`]): [`models_anim_end`], step 3. Step 3: call `FUN_02029620(GAMESYS, 0, 6)`, step 4. Step 4:
/// [`models_delete`], answer 1. The frame counter `+0x16` counts up after every other step.
pub fn models_event<E: FieldEnv>(env: &mut E, ev: u32, seq: u32, w: u32) -> u32 {
    match env.r32(seq) {
        0 => {
            if (env.r16(w.wrapping_add(0x16)) as u32).is_multiple_of(15) {
                // UNVERIFIED port, not in PORTED: through its (in-area) stub
                c(env, 0x021C_09B0, &[w]);
                if env.r8(w.wrapping_add(0x14)) <= env.r8(w.wrapping_add(0x15)) {
                    set_step(env, w, seq, 1);
                }
            }
        }
        1 => {
            if env.r16(w.wrapping_add(0x16)) > 10 {
                models_anim_start(env, w);
                let fm = env.r32(w.wrapping_add(4));
                let g = c(env, FM_GAMESYS, &[fm]);
                let child = c(env, EVENT_02029830, &[g, 0x514]);
                c(env, EVENT_CALL, &[ev, child]);
                set_step(env, w, seq, 2);
            }
        }
        2 => {
            if models_anim_done(env, w) != 0 {
                models_anim_end(env, w);
                set_step(env, w, seq, 3);
            }
        }
        3 => {
            let fm = env.r32(w.wrapping_add(4));
            let g = c(env, FM_GAMESYS, &[fm]);
            let child = c(env, EVENT_02029620, &[g, 0, 6]);
            c(env, EVENT_CALL, &[ev, child]);
            set_step(env, w, seq, 4);
        }
        4 => {
            models_delete(env, w);
            return 1;
        }
        _ => {}
    }
    let f = env.r16(w.wrapping_add(0x16));
    env.w16(w.wrapping_add(0x16), f.wrapping_add(1));
    0
}

/// The search rectangle of [`models_init`] / [`models_init_2`]: +-5 tiles around the player position; finds the
/// models of `kind` there (`FUN_0218C584`), keeps the first id in `+0x34` and its position in `+8`, frees the list.
fn find_models<E: FieldEnv>(env: &mut E, w: u32, kind: u32) {
    let fm = env.r32(w.wrapping_add(4));
    let p = c(env, FM_PLAYER, &[fm]);
    let mut pos = [0u8; 12];
    env.call(PLAYER_POS, &mut [CArg::V(p), CArg::OutBuf(&mut pos)]);
    let d = 5u32 << 16;
    let (x, z) = (rd32(&pos, 0), rd32(&pos, 8));
    let mut rect = [0u8; 16];
    wr32(&mut rect, 0, z.wrapping_sub(d));
    wr32(&mut rect, 4, z.wrapping_add(d));
    wr32(&mut rect, 8, x.wrapping_sub(d));
    wr32(&mut rect, 12, x.wrapping_add(d));
    let bm = env.r32(w.wrapping_add(0x18));
    let mut n = [0u8; 4];
    let list = env.call(
        BM_FIND,
        &mut [
            CArg::V(bm),
            CArg::InBuf(&rect),
            CArg::V(kind),
            CArg::OutBuf(&mut n),
        ],
    );
    if list != 0 {
        let id = env.r32(list);
        env.w32(w.wrapping_add(0x34), id);
        let id = env.r32(list);
        c(env, BM_POS, &[id, w.wrapping_add(8)]);
    }
    c(env, HEAP_FREE, &[list]);
}

/// C: `FUN_021C0904` (unlabeled routine at `0x021C0904`): set up the build-model work `w` for `n` models: `+0` heap,
/// `+4` FIELDMAP, `+0x14` = min(n, 6) (u8), `+0x15` = 0, `+0x18` = the buildmodel system, `+0x34` = 0; then the
/// models of kind 3 around the player (`find_models`).
pub fn models_init<E: FieldEnv>(env: &mut E, w: u32, gsys: u32, n: u32) {
    let fm = c(env, GSYS_FIELDMAP, &[gsys]);
    let heap = c(env, FM_HEAP, &[fm]);
    let map = c(env, FM_MAP, &[fm]);
    env.w16(w, heap as u16);
    env.w8(w.wrapping_add(0x14), n as u8);
    env.w32(w.wrapping_add(4), fm);
    env.w8(w.wrapping_add(0x15), 0);
    let bm = c(env, MAP_BM_SYS, &[map]);
    env.w32(w.wrapping_add(0x18), bm);
    env.w32(w.wrapping_add(0x34), 0);
    if env.r8(w.wrapping_add(0x14)) > 6 {
        env.w8(w.wrapping_add(0x14), 6);
    }
    find_models(env, w, 3);
}

/// C: `FUN_021C0988` (unlabeled routine at `0x021C0988`): delete the `+0x14` model objects `+0x1C[i]`
/// (`FUN_0218C860`).
pub fn models_delete<E: FieldEnv>(env: &mut E, w: u32) {
    let mut i = 0u32;
    while (i as i32) < env.r8(w.wrapping_add(0x14)) as i32 {
        let o = env.r32(w.wrapping_add(0x1C + 4 * i));
        c(env, BM_DELETE, &[o]);
        i += 1;
    }
}

/// C: `FUN_021C09A8` (unlabeled routine at `0x021C09A8`): `*seq = v`, frame counter `+0x16` = 0.
pub fn set_step<E: FieldEnv>(env: &mut E, w: u32, seq: u32, v: u32) {
    env.w32(seq, v);
    env.w16(w.wrapping_add(0x16), 0);
}

/// `FUN_021C09B0` / `FUN_021C0BE8` with the offset table at `table`: model `i = +0x15` while `i < +0x14`:
/// `FUN_020520FC(0, 0, 0, &m)`, `FUN_0207C9D0(w + 8, table + 12 * i, &p)`, `+0x1C[i] = FUN_0218C7F0(+0x18, 0x62,
/// &p)`, `+0x15 += 1`, SE 0x568.
fn add_model<E: FieldEnv>(env: &mut E, w: u32, table: u32) {
    let i = env.r8(w.wrapping_add(0x15)) as u32;
    if env.r8(w.wrapping_add(0x14)) as u32 <= i {
        return;
    }
    let mut m = [0u8; 0x24];
    env.call(
        MTX_ROT,
        &mut [CArg::V(0), CArg::V(0), CArg::V(0), CArg::OutBuf(&mut m)],
    );
    // UNVERIFIED: an index past the 6 entries reads past MODEL_OFFSETS; the port panics there, the original reads
    // the overlay data that follows
    let o = (table + 12 * i - MODEL_OFFSETS_BASE) as usize;
    let rom = &MODEL_OFFSETS[o..o + 0x24];
    let mut p = [0u8; 12];
    env.call(
        MTX_MULT_VEC33,
        &mut [
            CArg::In(w.wrapping_add(8), 12),
            CArg::InBuf(rom),
            CArg::OutBuf(&mut p),
        ],
    );
    let bm = env.r32(w.wrapping_add(0x18));
    let obj = env.call(
        BM_CREATE,
        &mut [CArg::V(bm), CArg::V(0x62), CArg::InBuf(&p)],
    );
    env.w32(w.wrapping_add(0x1C + 4 * i), obj);
    let a = env.r8(w.wrapping_add(0x15));
    env.w8(w.wrapping_add(0x15), a.wrapping_add(1));
    c(env, SE_PLAY, &[0x568]);
}

/// C: `FUN_021C09B0` (unlabeled routine at `0x021C09B0`): UNVERIFIED port of the model adder (see `add_model`;
/// the spec draws `+0x15` past the 6-entry table). Add the next model (`add_model`, offsets ROM 0x021DA098).
pub fn models_add<E: FieldEnv>(env: &mut E, w: u32) {
    add_model(env, w, MODEL_OFFSETS_BASE);
}

/// C: `FUN_021C0A10` (unlabeled routine at `0x021C0A10`): start the animation `FUN_0218C88C(obj, 0, 0)` of the
/// `+0x15` added models; with a found model `+0x34`: `FUN_0218C69C(+0x18, +0x34, 0, 0)`.
pub fn models_anim_start<E: FieldEnv>(env: &mut E, w: u32) {
    let mut i = 0u32;
    while (i as i32) < env.r8(w.wrapping_add(0x15)) as i32 {
        let o = env.r32(w.wrapping_add(0x1C + 4 * i));
        c(env, BM_ANIM, &[o, 0, 0]);
        i += 1;
    }
    let id = env.r32(w.wrapping_add(0x34));
    if id != 0 {
        let bm = env.r32(w.wrapping_add(0x18));
        c(env, BM_0218C69C, &[bm, id, 0, 0]);
    }
}

/// C: `FUN_021C0A48` (unlabeled routine at `0x021C0A48`): `FUN_0218C88C(obj, 0, 3)` for the `+0x15` added models.
pub fn models_anim_end<E: FieldEnv>(env: &mut E, w: u32) {
    let mut i = 0u32;
    while (i as i32) < env.r8(w.wrapping_add(0x15)) as i32 {
        let o = env.r32(w.wrapping_add(0x1C + 4 * i));
        c(env, BM_ANIM, &[o, 0, 3]);
        i += 1;
    }
}

/// C: `FUN_021C0A70` (unlabeled routine at `0x021C0A70`): 1 when `FUN_0218C8FC(obj, 0) == 1` for every added model,
/// else 0.
pub fn models_anim_done<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let mut i = 0u32;
    while (i as i32) < env.r8(w.wrapping_add(0x15)) as i32 {
        let o = env.r32(w.wrapping_add(0x1C + 4 * i));
        if c(env, BM_ANIM_DONE, &[o, 0]) != 1 {
            return 0;
        }
        i += 1;
    }
    1
}

/// C: `FUN_021C0AA0` (unlabeled routine at `0x021C0AA0`): create the second build-model GMEVENT `FUN_021C0AC0`
/// (work 0x3C) as a child of `parent` and set it up ([`models_init_2`]); returns the event.
pub fn models_event_2_new<E: FieldEnv>(env: &mut E, gsys: u32, parent: u32) -> u32 {
    let ev = c(env, EVENT_CREATE, &[gsys, parent, 0x021C_0AC1, 0x3C]);
    let w = c(env, EVENT_WORK, &[ev]);
    models_init_2(env, w, gsys);
    ev
}

/// C: `FUN_021C0AC0` (unlabeled routine at `0x021C0AC0`): the second build-model GMEVENT proc. Step 0: every 20th
/// frame [`models_add_2`]; all added: next step, counter 0. Step 1: after 10 frames [`models_anim_start_2`], next
/// step, counter 0. Step 2: [`models_anim_done_2`]: next step. Step 3: [`models_delete_2`], answer 1. The counter
/// `+0x16` counts up after every other step.
pub fn models_event_2<E: FieldEnv>(env: &mut E, _ev: u32, seq: u32, w: u32) -> u32 {
    let next = |env: &mut E| {
        let s = env.r32(seq);
        env.w32(seq, s.wrapping_add(1));
    };
    match env.r32(seq) {
        0 => {
            if (env.r16(w.wrapping_add(0x16)) as u32).is_multiple_of(20) {
                // UNVERIFIED port, not in PORTED: through its (in-area) stub
                c(env, 0x021C_0BE8, &[w]);
                if env.r8(w.wrapping_add(0x14)) <= env.r8(w.wrapping_add(0x15)) {
                    next(env);
                    env.w16(w.wrapping_add(0x16), 0);
                }
            }
        }
        1 => {
            if env.r16(w.wrapping_add(0x16)) > 10 {
                models_anim_start_2(env, w);
                next(env);
                env.w16(w.wrapping_add(0x16), 0);
            }
        }
        2 => {
            if models_anim_done_2(env, w) != 0 {
                next(env);
            }
        }
        3 => {
            models_delete_2(env, w);
            return 1;
        }
        _ => {}
    }
    let f = env.r16(w.wrapping_add(0x16));
    env.w16(w.wrapping_add(0x16), f.wrapping_add(1));
    0
}

/// C: `FUN_021C0B3C` (unlabeled routine at `0x021C0B3C`): [`models_init`] for the second event: `n` = the count
/// [`party_count`], models of kind 8; with a found model `+0x34`: `+0x38 = FUN_0218C724(+0x18, +0x34)`.
pub fn models_init_2<E: FieldEnv>(env: &mut E, w: u32, gsys: u32) {
    let fm = c(env, GSYS_FIELDMAP, &[gsys]);
    let map = c(env, FM_MAP, &[fm]);
    let heap = c(env, FM_HEAP, &[fm]);
    env.w16(w, heap as u16);
    env.w32(w.wrapping_add(4), fm);
    let n = party_count(env, gsys);
    env.w8(w.wrapping_add(0x14), n as u8);
    env.w8(w.wrapping_add(0x15), 0);
    let bm = c(env, MAP_BM_SYS, &[map]);
    env.w32(w.wrapping_add(0x18), bm);
    env.w32(w.wrapping_add(0x34), 0);
    if env.r8(w.wrapping_add(0x14)) > 6 {
        env.w8(w.wrapping_add(0x14), 6);
    }
    find_models(env, w, 8);
    let id = env.r32(w.wrapping_add(0x34));
    if id != 0 {
        let bm = env.r32(w.wrapping_add(0x18));
        let o = c(env, BM_0218C724, &[bm, id]);
        env.w32(w.wrapping_add(0x38), o);
    }
}

/// C: `FUN_021C0BD8` (unlabeled routine at `0x021C0BD8`): with a found model `+0x34`: delete its object `+0x38`.
pub fn models_delete_2<E: FieldEnv>(env: &mut E, w: u32) {
    if env.r32(w.wrapping_add(0x34)) != 0 {
        let o = env.r32(w.wrapping_add(0x38));
        c(env, BM_DELETE, &[o]);
    }
}

/// C: `FUN_021C0BE8` (unlabeled routine at `0x021C0BE8`): UNVERIFIED port of the second model adder (see
/// `add_model`; the spec draws `+0x15` past the 6-entry table). Add the next model (`add_model`, offsets ROM 0x021DA0E0).
pub fn models_add_2<E: FieldEnv>(env: &mut E, w: u32) {
    add_model(env, w, MODEL_OFFSETS_BASE + 0x48);
}

/// C: `FUN_021C0C48` (unlabeled routine at `0x021C0C48`): `FUN_0218C88C(obj, 0, 2)` for the added models; with a found
/// model: `FUN_0218C88C(+0x38, 0, 0)`.
pub fn models_anim_start_2<E: FieldEnv>(env: &mut E, w: u32) {
    let mut i = 0u32;
    while (i as i32) < env.r8(w.wrapping_add(0x15)) as i32 {
        let o = env.r32(w.wrapping_add(0x1C + 4 * i));
        c(env, BM_ANIM, &[o, 0, 2]);
        i += 1;
    }
    if env.r32(w.wrapping_add(0x34)) != 0 {
        let o = env.r32(w.wrapping_add(0x38));
        c(env, BM_ANIM, &[o, 0, 0]);
    }
}

/// C: `FUN_021C0C80` (unlabeled routine at `0x021C0C80`): with a found model: `FUN_0218C8D8(+0x38)` (animation
/// done); else 1.
pub fn models_anim_done_2<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    if env.r32(w.wrapping_add(0x34)) != 0 {
        let o = env.r32(w.wrapping_add(0x38));
        return c(env, BM_ANIM_DONE_1, &[o]);
    }
    1
}

/// C: `FUN_021C0C94` (unlabeled routine at `0x021C0C94`): `FUN_0201AA1C(FUN_02012934(FUN_020120F4(gsys)))` (a count
/// of the GAMEDATA, the party size by its use).
pub fn party_count<E: FieldEnv>(env: &mut E, gsys: u32) -> u32 {
    let gd = c(env, GSYS_GAMEDATA, &[gsys]);
    let x = c(env, GAMEDATA_GET, &[gd]);
    c(env, COUNT_0201AA1C, &[x])
}

/// C: `FUN_021C0CA4` (unlabeled routine at `0x021C0CA4`): the boulder-push step check of the grid player: dir =
/// `FUN_021A3AB8(core, keys)`; `FUN_021A44BC(grid, dir) == 0`: 0; else [`push_check`]`(grid, dir, flags)` (the event
/// or 0). `a` is unused.
pub fn step_check<E: FieldEnv>(env: &mut E, grid: u32, _a: u32, keys: u32, flags: u32) -> u32 {
    let core = c(env, GRID_CORE, &[grid]);
    c(env, GRID_MMDL, &[grid]);
    let dir = c(env, CORE_KEY_DIR, &[core, keys]);
    // the stub of 021A44BC lists only the grid (r1 = dir is set but not read by the spec's model)
    if env.call(GRID_44BC, &mut [CArg::V(grid), CArg::X(dir)]) == 0 {
        return 0;
    }
    push_check(env, grid, dir, flags)
}

/// C: `FUN_021C0CE4` (unlabeled routine at `0x021C0CE4`): `k = FUN_021A448C(grid, dir)`; with flags bit 0
/// [`push_start`]`(grid, dir, k)`, else 0.
pub fn push_check<E: FieldEnv>(env: &mut E, grid: u32, dir: u32, flags: u32) -> u32 {
    let k = c(env, GRID_448C, &[grid, dir]);
    if flags & 1 != 0 {
        push_start(env, grid, dir, k)
    } else {
        0
    }
}

/// C: `FUN_021C0D08` (unlabeled routine at `0x021C0D08`): the footprint of the boulder `m` one step in `dir`: for
/// each of its size-z rows (`FUN_0216D924`; z decreasing by a tile) and size-x tiles (`FUN_0216D91C`; x increasing),
/// a = `FUN_021AB0D4(map, &p)`: `FUN_021AB11C(a) != 0` (blocked): `FUN_021AB2DC(FUN_021AB0F0(a)) == 0`: answer 1 at
/// once, else a hole; else floor. Answer 3 holes only, 2 holes and floor, 0 otherwise.
pub fn footprint<E: FieldEnv>(env: &mut E, m: u32, dir: u32, map: u32) -> u32 {
    let mut pos = [0u8; 12];
    env.call(GET_POS, &mut [CArg::V(m), CArg::OutBuf(&mut pos)]);
    env.call(
        STEP_POS,
        &mut [CArg::V(dir), CArg::InBuf(&pos), CArg::V(0x10000)],
    );
    let sx = c(env, SIZE_X, &[m]);
    let sz = c(env, SIZE_Z, &[m]);
    let mut floor = false;
    let mut hole = 0u32;
    let mut j = 0u32;
    if sz > 0 {
        loop {
            let mut p = pos;
            let mut i: u8 = 0;
            if sx > 0 {
                loop {
                    let a = env.call(ATTR_AT, &mut [CArg::V(map), CArg::InBuf(&p)]);
                    let v = c(env, ATTR_VALUE, &[a]);
                    if c(env, ATTR_P_11C, &[a]) != 0 {
                        if c(env, ATTR_P_2DC, &[v]) == 0 {
                            return 1;
                        }
                        hole = 1;
                    } else {
                        floor = true;
                    }
                    i = i.wrapping_add(1);
                    let x = rd32(&p, 0).wrapping_add(0x10000);
                    wr32(&mut p, 0, x);
                    if (i as u32) >= sx {
                        break;
                    }
                }
            }
            j = (j + 1) & 0xFF;
            let z = rd32(&pos, 8).wrapping_sub(0x10000);
            wr32(&mut pos, 8, z);
            if j >= sz {
                break;
            }
        }
    }
    if hole == 1 {
        if floor {
            2
        } else {
            3
        }
    } else {
        0
    }
}

/// C: `FUN_021C0DDC` (unlabeled routine at `0x021C0DDC`): with `k == 1`, an MMDL in front (`FUN_021C19B4(grid,
/// dir)`) whose OBJ code is a pushable boulder (`FUN_0216E5FC`): `r` = [`footprint`] on the map; hit bits
/// `FUN_021639C0(m, dir)` without bit 0, and without bit 1 when `r` is 2 or 3; no hit: the push event
/// [`push_event_new`]`(grid, dir, m, r == 3)`. Else 0.
pub fn push_start<E: FieldEnv>(env: &mut E, grid: u32, dir: u32, k: u32) -> u32 {
    if k != 1 {
        return 0;
    }
    let m = c(env, FRONT_MMDL, &[grid, dir]);
    if m == 0 {
        return 0;
    }
    let code = c(env, OBJ_CODE, &[m]);
    if c(env, PUSHABLE, &[code]) != 1 {
        return 0;
    }
    let core = c(env, GRID_CORE, &[grid]);
    let fm = c(env, CORE_FIELDMAP, &[core]);
    let map = c(env, FM_MAP, &[fm]);
    let r = footprint(env, m, dir, map);
    let mut hit = c(env, HIT_CHECK, &[m, dir]) & !1;
    if hit & 2 != 0 && r.wrapping_sub(2) <= 1 {
        hit &= !2;
    }
    if hit != 0 {
        return 0;
    }
    push_event_new(env, grid, dir, m, (r == 3) as u32)
}

/// C: `FUN_021C0E4C` (unlabeled routine at `0x021C0E4C`): create the boulder push GMEVENT `FUN_021C0E98` (work 0x24,
/// cleared): `+0` grid, `+4` dir, `+6` fall, `+0x1C` boulder `m`, `+0x20` the player MMDL; returns the event.
pub fn push_event_new<E: FieldEnv>(env: &mut E, grid: u32, dir: u32, m: u32, fall: u32) -> u32 {
    let core = c(env, GRID_CORE, &[grid]);
    let gsys = c(env, CORE_GAMESYS, &[core]);
    let ev = c(env, EVENT_CREATE, &[gsys, 0, 0x021C_0E99, 0x24]);
    let w = c(env, EVENT_WORK, &[ev]);
    env.mem().fill(w, 0x24, 0);
    env.w32(w, grid);
    env.w16(w.wrapping_add(4), dir as u16);
    env.w16(w.wrapping_add(6), fall as u16);
    env.w32(w.wrapping_add(0x1C), m);
    let pm = c(env, CORE_MMDL, &[core]);
    env.w32(w.wrapping_add(0x20), pm);
    ev
}

/// C: `FUN_021C0E98` (unlabeled routine at `0x021C0E98`): the boulder push GMEVENT proc. Step 0: the boulder's acmd
/// settable and the player's too (or `FUN_021A5408(grid)`): acmd `FUN_02197E7C(dir, 8)` on the boulder,
/// `FUN_02197E7C(dir, 0x20)` on the player, SE 0x674, step 1. Step 1: both acmds done: without a fall step 4; else the
/// height under the boulder (`FUN_02163E58` -> `+0xC`) minus 0xE000 (OBJ code 0x6D) or 0x1A000, dir lock on, move bit
/// off, SE 0x675, step 2. Step 2: move the boulder's y towards `+0xC` by at most 0x4000 per frame; there: shake
/// `+0x14` = 0x2000, step 3. Step 3: draw offset `+0x10`, negate the shake, `+8` += 1, every frame with bit 2 of
/// `+8` the shake shrinks by 0x1000; at 0: set the position and dir again, draw offset, refresh, step 4. Step 4:
/// answer 1.
pub fn push_event<E: FieldEnv>(env: &mut E, _ev: u32, seq: u32, w: u32) -> u32 {
    let next = |env: &mut E| {
        let s = env.r32(seq);
        env.w32(seq, s.wrapping_add(1));
    };
    let m = |env: &mut E| env.r32(w.wrapping_add(0x1C));
    match env.r32(seq) {
        0 => {
            let b = m(env);
            if c(env, ACMD_SETTABLE, &[b]) == 0 {
                return 0;
            }
            let pm = env.r32(w.wrapping_add(0x20));
            if c(env, ACMD_SETTABLE, &[pm]) == 0 {
                let grid = env.r32(w);
                if c(env, GRID_5408, &[grid]) == 0 {
                    return 0;
                }
            }
            let dir = env.r16(w.wrapping_add(4)) as u32;
            let a = c(env, ACMD_CODE, &[dir, 8]);
            let b = m(env);
            c(env, ACMD_SET, &[b, a]);
            let dir = env.r16(w.wrapping_add(4)) as u32;
            let a = c(env, ACMD_CODE, &[dir, 0x20]);
            let pm = env.r32(w.wrapping_add(0x20));
            c(env, ACMD_SET, &[pm, a]);
            c(env, SE_PLAY, &[0x674]);
            next(env);
        }
        1 => {
            let b = m(env);
            if c(env, ACMD_DONE, &[b]) == 0 {
                return 0;
            }
            let pm = env.r32(w.wrapping_add(0x20));
            if c(env, ACMD_DONE, &[pm]) == 0 {
                return 0;
            }
            if env.r16(w.wrapping_add(6)) == 0 {
                env.w32(seq, 4);
                return 0;
            }
            let mut pos = [0u8; 12];
            let b = m(env);
            env.call(GET_POS, &mut [CArg::V(b), CArg::OutBuf(&mut pos)]);
            let b = m(env);
            env.call(
                HEIGHT_AT,
                &mut [
                    CArg::V(b),
                    CArg::InBuf(&pos),
                    CArg::Out(w.wrapping_add(0xC), 4),
                ],
            );
            let b = m(env);
            let code = c(env, OBJ_CODE, &[b]);
            let t = env.r32(w.wrapping_add(0xC));
            let d = if code == 0x6D { 0xE } else { 0x1A } << 12;
            env.w32(w.wrapping_add(0xC), t.wrapping_sub(d));
            let b = m(env);
            c(env, DIR_LOCK, &[b, 1]);
            let b = m(env);
            c(env, MOVE_BIT, &[b, 0]);
            c(env, SE_PLAY, &[0x675]);
            next(env);
        }
        2 => {
            let mut pos = [0u8; 12];
            let b = m(env);
            env.call(GET_POS, &mut [CArg::V(b), CArg::OutBuf(&mut pos)]);
            let y = rd32(&pos, 4) as i32;
            let t = env.r32(w.wrapping_add(0xC)) as i32;
            if t < y {
                let ny = y.wrapping_sub(0x4000);
                wr32(&mut pos, 4, ny as u32);
                if t > ny {
                    wr32(&mut pos, 4, t as u32);
                }
            } else if t > y {
                let ny = y.wrapping_add(0x4000);
                wr32(&mut pos, 4, ny as u32);
                if t < ny {
                    wr32(&mut pos, 4, t as u32);
                }
            }
            let b = m(env);
            env.call(SET_POS, &mut [CArg::V(b), CArg::InBuf(&pos)]);
            if env.r32(w.wrapping_add(0xC)) == rd32(&pos, 4) {
                env.w32(w.wrapping_add(0x14), 0x2000);
                next(env);
            }
        }
        3 => {
            let b = m(env);
            env.call(
                SET_DRAW_OFS,
                &mut [CArg::V(b), CArg::In(w.wrapping_add(0x10), 12)],
            );
            let mut s = env.r32(w.wrapping_add(0x14)).wrapping_neg() as i32;
            env.w32(w.wrapping_add(0x14), s as u32);
            let f = env.r16(w.wrapping_add(8)) as i16;
            env.w16(w.wrapping_add(8), f.wrapping_add(1) as u16);
            let f = env.r16(w.wrapping_add(8)) as i16 as i32;
            if f & 4 != 0 {
                if s < 0 {
                    s = s.wrapping_add(0x1000);
                    env.w32(w.wrapping_add(0x14), s as u32);
                } else if s > 0 {
                    s = s.wrapping_sub(0x1000);
                    env.w32(w.wrapping_add(0x14), s as u32);
                }
            }
            if env.r32(w.wrapping_add(0x14)) != 0 {
                return 0;
            }
            let mut pos = [0u8; 12];
            let b = m(env);
            env.call(GET_POS, &mut [CArg::V(b), CArg::OutBuf(&mut pos)]);
            let dir = env.r16(w.wrapping_add(4)) as u32;
            let b = m(env);
            env.call(
                SET_POS_DIR,
                &mut [CArg::V(b), CArg::InBuf(&pos), CArg::V(dir)],
            );
            let b = m(env);
            env.call(
                SET_DRAW_OFS,
                &mut [CArg::V(b), CArg::In(w.wrapping_add(0x10), 12)],
            );
            let b = m(env);
            c(env, REFRESH, &[b]);
            next(env);
        }
        4 => return 1,
        _ => {}
    }
    0
}

#[cfg(test)]
mod tests;
