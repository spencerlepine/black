//! `fieldmap.c`, first part (overlay 21, `0x02187F00..0x02189BD0`): the FIELDMAP proc (create, the
//! setup sequence, the per-frame update and the teardown), its accessors, the zone-change check and the
//! map-setup record. The event checks of the same file are [`crate::fieldmap_event`].
//!
//! F2a-spec specced the 101 logic functions of the range (`tools/rom-diff/specs/black-field/fieldmap.spec`;
//! its header documents the 0x154-byte FIELDMAP record, the MAP CTRL and MAP SETUP tables and the proc table);
//! F2a ported them. Records are read through the [`Mem`] window of the environment at their original
//! addresses. Every callee outside the area goes through [`FieldEnv::call`]: the field subsystems of other F
//! items, the save / game-data accessors, and the draw side (D3): the 17 render-only functions of this range
//! (crate README) and the 2D / 3D engine calls are rewritten in G8, never ported here, so a logic function
//! reaches them by address only.

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0218_7F00,
    0x0218_7FD0,
    0x0218_7FE8,
    0x0218_8030,
    0x0218_80A4,
    0x0218_81D8,
    0x0218_8758,
    0x0218_8910,
    0x0218_8B58,
    0x0218_8BD8,
    0x0218_8BE8,
    0x0218_8BF8,
    0x0218_8C54,
    0x0218_8C58,
    0x0218_8C5C,
    0x0218_8C60,
    0x0218_8C64,
    0x0218_8C68,
    0x0218_8C6C,
    0x0218_8C70,
    0x0218_8C7C,
    0x0218_8C80,
    0x0218_8C84,
    0x0218_8C88,
    0x0218_8C90,
    0x0218_8C98,
    0x0218_8C9C,
    0x0218_8CA4,
    0x0218_8CAC,
    0x0218_8CB4,
    0x0218_8CBC,
    0x0218_8CCC,
    0x0218_8CD0,
    0x0218_8CD8,
    0x0218_8CE0,
    0x0218_8CE8,
    0x0218_8CF0,
    0x0218_8CFC,
    0x0218_8D20,
    0x0218_8D24,
    0x0218_8D2C,
    0x0218_8D30,
    0x0218_8D38,
    0x0218_8D40,
    0x0218_8D48,
    0x0218_8D4C,
    0x0218_8D50,
    0x0218_8D58,
    0x0218_8E30,
    0x0218_9004,
    0x0218_9204,
    0x0218_92B4,
    0x0218_92CC,
    0x0218_932C,
    0x0218_9370,
    0x0218_93C0,
    0x0218_94E0,
    0x0218_9518,
    0x0218_9560,
    0x0218_958C,
    0x0218_95BC,
    0x0218_95D0,
    0x0218_95DC,
    0x0218_9614,
    0x0218_9684,
    0x0218_96F4,
    0x0218_9734,
    0x0218_9758,
    0x0218_9764,
    0x0218_97A0,
    0x0218_97BC,
    0x0218_99F0,
    0x0218_99F8,
    0x0218_9A00,
    0x0218_9A08,
    0x0218_9A10,
    0x0218_9A18,
    0x0218_9A1C,
    0x0218_9A24,
    0x0218_9A2C,
    0x0218_9A38,
    0x0218_9A48,
    0x0218_9A50,
    0x0218_9A58,
    0x0218_9A60,
    0x0218_9A68,
    0x0218_9A70,
    0x0218_9A78,
    0x0218_9A80,
    0x0218_9ACC,
    0x0218_9AE8,
    0x0218_9B04,
    0x0218_9B30,
    0x0218_9BA8,
    0x0218_9BBC,
];

/// Size of the FIELDMAP record (`Heap_AllocDebug(heap, 0x154, 1, "fieldmap.c", 0x209)` in `0x02188030`).
pub const FIELDMAP_SIZE: u32 = 0x154;
/// Address of the string `"fieldmap.c"`.
pub const FILE_FIELDMAP: u32 = 0x021D_D9C0;
/// FIELDMAP: `+0x118` the SLOT `{s32 id (-1 = free), block}`.
pub const SLOT_OFS: u32 = 0x118;
/// FIELDMAP: `+0x108` -> MAP CTRL.
pub const CTRL_OFS: u32 = 0x108;

/// ROM `0x021D35A8`: the kind of a MAP CTRL (`+0` of the record) -> the map kind (0 grid, 1 rail, 2 hybrid). Only the
/// first 3 bytes belong to the table; the rest is what follows it in the ROM (read by out-of-range kinds).
pub const CTRL_KIND: [u8; 12] = [
    0x00, 0x01, 0x02, 0x00, 0x60, 0x36, 0x1D, 0x02, 0x04, 0x00, 0x00, 0x00,
];

/// Address of the MAP SETUP table (8 records of 0x48 bytes, indexed by the zone kind `FUN_02013B50(zone)`).
pub const MAP_SETUP: u32 = 0x021D_3700;
/// Size of a MAP SETUP record.
pub const MAP_SETUP_SIZE: u32 = 0x48;
/// ROM `0x021D3700` as words: `+0..+0x3B` the record `FUN_02189B30` copies, `+0x3C` -> MAP CTRL (ROM 0x021D88F8 grid,
/// 0x021D8AD8 rail), `+0x40` "use the matrix size", `+0x44` the heap size of `FUN_02187F00`.
pub const MAP_SETUP_DATA: [u32; 144] = [
    0x0020_0000,
    0x0040_0000,
    0x0002_0002,
    0x0000_0001,
    0x0000_0008,
    0x0001_0001,
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0002,
    0x0001_B008,
    0x0000_0001,
    0x021D_88F8,
    0x0000_0001,
    0x0013_0400,
    0x0080_0000,
    0x0080_0000,
    0x0003_0002,
    0x0000_0000,
    0x0000_0008,
    0x0003_0002,
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0xFFFF_FFFF,
    0xFFFF_FFFF,
    0x0001_5004,
    0x0000_0001,
    0x021D_8AD8,
    0x0000_0001,
    0x0013_0400,
    0x0020_0000,
    0x0040_0000,
    0x0002_0002,
    0x0000_0000,
    0x0000_0008,
    0x0002_0002,
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0xFFFF_FFFF,
    0xFFFF_FFFF,
    0x0001_5004,
    0x0000_0000,
    0x021D_8AD8,
    0x0000_0001,
    0x0013_0400,
    0x0020_0000,
    0x0040_0000,
    0x0002_0003,
    0x0000_0002,
    0x0000_0008,
    0x0001_0001,
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0002,
    0x0001_5004,
    0x0000_0000,
    0x021D_88F8,
    0x0000_0001,
    0x0013_0400,
    0x0020_0000,
    0x0040_0000,
    0x0002_0002,
    0x0000_0000,
    0x0000_0008,
    0x0001_0001,
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0002,
    0x0001_5004,
    0x0000_0001,
    0x021D_88F8,
    0x0000_0001,
    0x0013_0400,
    0x0040_0000,
    0x0040_0000,
    0x0001_0001,
    0x0000_0000,
    0x0000_0008,
    0x0001_0001,
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0002,
    0x0005_4010,
    0x0000_0000,
    0x021D_88F8,
    0x0000_0001,
    0x0013_0400,
    0x0020_0000,
    0x0040_0000,
    0x0001_0001,
    0x0000_0000,
    0x0000_0008,
    0x0001_0001,
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0002,
    0x0001_5004,
    0x0000_0000,
    0x021D_88F8,
    0x0000_0001,
    0x0012_8400,
    0x0020_0000,
    0x0040_0000,
    0x0003_0002,
    0x0000_0001,
    0x0000_0008,
    0x0001_0001,
    0x0000_0001,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0002,
    0x0001_5004,
    0x0000_0001,
    0x021D_8AD8,
    0x0000_0001,
    0x0013_0400,
];

fn setup_word(k: u32, ofs: u32) -> u32 {
    MAP_SETUP_DATA[((k.wrapping_mul(MAP_SETUP_SIZE).wrapping_add(ofs)) / 4) as usize]
}

/// C: `FUN_02013B50(zone)` (through the veneer `FUN_02189B28`): the zone kind, index of the MAP SETUP table.
const ZONE_KIND: u32 = 0x0201_3B50;

macro_rules! getter {
    ($(#[$m:meta])* $name:ident, $ofs:expr) => {
        $(#[$m])*
        pub fn $name(mem: &Mem, fm: u32) -> u32 {
            mem.r32(fm + $ofs)
        }
    };
}

macro_rules! setter {
    ($(#[$m:meta])* $name:ident, $ofs:expr) => {
        $(#[$m])*
        pub fn $name(mem: &mut Mem, fm: u32, v: u32) {
            mem.w32(fm + $ofs, v);
        }
    };
}

/// C: `FUN_02187F00` (unlabeled routine at `0x02187F00`): the FIELDMAP proc's init (`*seq` 0): set the player record's
/// zone (`FUN_02012AC8`), create the field heaps (`FUN_0203064C` / `FUN_02030678`: 0x15 sized by [`heap_size_of`],
/// 0x70, 0x50, 0x89 at 0x021DE700, 0x92, 0x93, 0x96), the proc work (`FUN_0203159C(proc, 4, 0x15)`) holding the new
/// FIELDMAP ([`create`]), register it (`FUN_02012110`) and go on (0). `*seq` 1: unless `FUN_0201214C(gamesys)`,
/// release the comm state (`FUN_02026400`) when `FUN_02026454` is 1 or 2; done (1). Else 0.
pub fn proc_init<E: FieldEnv>(env: &mut E, proc_: u32, seq: u32, gs: u32) -> u32 {
    match env.r32(seq) {
        0 => {
            let p = env.callv(0x0201_20F8, &[gs]);
            let z = env.callv(0x0201_2AC8, &[p]);
            let n = heap_size_of(env, z);
            env.callv(0x0203_064C, &[1, 0x15, n.wrapping_sub(0xD000)]);
            env.callv(0x0203_064C, &[1, 0x70, 0xD000]);
            env.callv(0x0203_064C, &[0x15, 0x50, 0xC000]);
            env.callv(0x0203_0678, &[0x021D_E700, 0x1_0000, 0x89]);
            env.callv(0x0203_064C, &[0x89, 0x92, 0x6400]);
            env.callv(0x0203_064C, &[0x89, 0x93, 0x7400]);
            env.callv(0x0203_064C, &[0x89, 0x96, 0x500]);
            let w = env.callv(0x0203_159C, &[proc_, 4, 0x15]);
            let fm = create(env, gs, 0x15);
            env.w32(w, fm);
            env.callv(0x0201_2110, &[gs, fm]);
            let v = env.r32(seq).wrapping_add(1);
            env.w32(seq, v);
            0
        }
        1 => {
            if env.callv(0x0201_214C, &[gs]) == 0 {
                let x = env.callv(0x0201_2138, &[gs]);
                let k = env.callv(0x0202_6454, &[x]);
                if k == 1 || k == 2 {
                    // the spec's stub lists no arguments
                    env.call(0x0202_6400, &mut [CArg::X(x)]);
                }
            }
            1
        }
        _ => 0,
    }
}

/// C: `FUN_02187FD0` (unlabeled routine at `0x02187FD0`): the FIELDMAP proc's main: 1 (done) when [`proc_step`] says 1.
pub fn proc_main<E: FieldEnv>(env: &mut E, _proc: u32, _seq: u32, gs: u32, work: u32) -> u32 {
    let fm = env.r32(work);
    (proc_step(env, gs, fm) == 1) as u32
}

/// C: `FUN_02187FE8` (unlabeled routine at `0x02187FE8`): the FIELDMAP proc's end: [`destroy`], unregister
/// (`FUN_02012110(gamesys, 0)`), free the proc work (`FUN_020315D4`) and delete the field heaps (`FUN_020306F0`);
/// done (1).
pub fn proc_end<E: FieldEnv>(env: &mut E, proc_: u32, _seq: u32, gs: u32, work: u32) -> u32 {
    let fm = env.r32(work);
    destroy(env, fm);
    env.callv(0x0201_2110, &[gs, 0]);
    env.callv(0x0203_15D4, &[proc_]);
    for h in [0x96, 0x92, 0x93, 0x89, 0x50, 0x70, 0x15] {
        env.callv(0x0203_06F0, &[h]);
    }
    1
}

/// C: `FUN_02188030` (unlabeled routine at `0x02188030`): allocate the FIELDMAP (0x154 bytes, line 0x209): heap id,
/// state 0, game system, GAMEDATA, the setup zone (`+0xD0`, the player record's), the area data
/// (`FUN_0201407C(heap, FUN_02013B38(zone), season)`), the MAP CTRL of the zone ([`map_ctrl_of`]); returns it.
pub fn create<E: FieldEnv>(env: &mut E, gs: u32, heap: u32) -> u32 {
    let fm = env.callv(HEAP_ALLOC, &[heap, FIELDMAP_SIZE, 1, FILE_FIELDMAP, 0x209]);
    env.w16(fm, heap as u16);
    env.w32(fm + 0xC0, 0);
    env.w32(fm + 4, gs);
    let gd = env.callv(0x0201_20F4, &[gs]);
    env.w32(fm + 8, gd);
    let p = env.callv(0x0201_20F8, &[gs]);
    let z = env.callv(0x0201_2AC8, &[p]);
    env.w16(fm + 0xD0, z as u16);
    let z = env.r16(fm + 0xD0) as u32;
    let a = env.callv(0x0201_3B38, &[z]);
    let season = env.callv(0x0202_E83C, &[gs]);
    let ad = env.callv(0x0201_407C, &[heap, a, season]);
    env.w32(fm + 0xC, ad);
    let z = env.r16(fm + 0xD0) as u32;
    let c = map_ctrl_of(env, z);
    env.w32(fm + CTRL_OFS, c);
    fm
}

/// C: `FUN_021880A4` (unlabeled routine at `0x021880A4`): free the area data (`FUN_020140B4`) and the FIELDMAP,
/// unregister it (`FUN_02012110(gamesys, 0)`), `FUN_020371AC(0, 0)`.
pub fn destroy<E: FieldEnv>(env: &mut E, fm: u32) {
    let (ad, gs) = (env.r32(fm + 0xC), env.r32(fm + 4));
    env.callv(0x0201_40B4, &[ad]);
    env.callv(HEAP_FREE, &[fm]);
    env.callv(0x0201_2110, &[gs, 0]);
    env.callv(0x0203_71AC, &[0, 0]);
}

/// C: `FUN_021880C8` (unlabeled routine at `0x021880C8`, proc step 0): graphics init (D3: the 2D/3D engine calls go
/// to G8): `FUN_020433E0(0x021D3688)`, the render setup `FUN_02188D5C(fm)`, `FUN_0204A48C`, display layers, the
/// graphics objects, `FUN_02188EE4(fm)`; then the logic side: the map draw object (`+0x48 = FUN_0218C9E8(heap,
/// ..)`), the VBlank task (`+0xAC`), the task system (`+0x110` / `+0x114`, 0x20 tasks, line 0x26B), `+0x134`, `+0x14C`,
/// the overlay (0x13 for the zone kinds of `FUN_02013CC8` / `FUN_02013CE0`, else 0x12), `FUN_02035660(0x1E)`; 1.
///
/// UNVERIFIED port of `FUN_021880C8`: not in [`PORTED`]. The spec's stub `02188d5c args=v,v,v,v` logs r1..r3, which
/// the one-argument render function does not read; r2 and r3 are this function's entry registers (the proc driver's
/// or the harness's), so no port reproduces the hash. The driver [`proc_step`] reaches it through its stub.
pub fn proc_graphics_init<E: FieldEnv>(env: &mut E, _gs: u32, fm: u32) -> u32 {
    let heap = env.r16(fm) as u32;
    env.callv(0x0204_33E0, &[0x021D_3688]);
    env.call(
        0x0218_8D5C,
        &mut [CArg::V(fm), CArg::V(fm), CArg::V(0), CArg::V(0)],
    );
    env.callv(0x0204_A48C, &[0x021D_3604, 0x021D_3688, 0x89]);
    env.callv(0x0204_34DC, &[0x10, 1]);
    env.callv(0x0204_3598, &[0x10, 1]);
    env.callv(0x0204_5088, &[heap]);
    env.callv(0x0201_DD60, &[]);
    let h2 = env.r16(fm) as u32;
    env.callv(0x0204_6440, &[0, 3, 0, 1, 0x1000, h2, 0x0218_8E51]);
    env.callv(0x0218_8EE4, &[fm]);
    let gd = env.r32(fm + 8);
    let x = env.callv(0x0201_2984, &[gd]);
    let h2 = env.r16(fm) as u32;
    let d = env.callv(0x0218_C9E8, &[h2, x]);
    env.w32(fm + 0x48, d);
    let t = env.callv(0x0200_56A0, &[0x0218_9109, fm, 0]);
    env.w32(fm + 0xAC, t);
    let n = env.callv(0x0203_0CB0, &[0x20]);
    let b = env.callv(HEAP_ALLOC, &[heap, n, 0, FILE_FIELDMAP, 0x26B]);
    env.w32(fm + 0x114, b);
    let t = env.callv(0x0203_0CBC, &[0x20, b]);
    env.w32(fm + 0x110, t);
    let h2 = env.r16(fm) as u32;
    let v = env.callv(0x021C_6D10, &[10, h2]);
    env.w32(fm + 0x134, v);
    let h2 = env.r16(fm) as u32;
    let v = env.callv(0x021D_317C, &[h2]);
    env.w32(fm + 0x14C, v);
    let z = env.r16(fm + 0xD0) as u32;
    let ov = if env.callv(0x0201_3CC8, &[z]) == 1 || {
        let z = env.r16(fm + 0xD0) as u32;
        env.callv(0x0201_3CE0, &[z]) == 1
    } {
        19
    } else {
        18
    };
    env.callv(0x0203_4AC4, &[ov]);
    env.callv(0x0203_5660, &[0x1E]);
    1
}

/// C: `FUN_021881D8` (unlabeled routine at `0x021881D8`, proc step 1): the setup sequence, one sub-step (`+0xC8`) per
/// call: 0 the subsystem objects of the map (3D handler, scene area, ...), 1 the event / MAP SETUP / weather side,
/// 2 the player LOCATION and the zone texture set, 3 the MMDL system ([`mmdl_setup`]) and `+0x94`, 4 the player
/// (`FUN_021A2E50`), the MAP CTRL setup callback, the light / scene objects, 5 `+0x24`, 6 the encounter object, the
/// FIELDMAP FUNC system (`FUN_021BB890`, fieldmap_func) and the rest. Every call advances `+0xC8` and clears the
/// draw mode (`+0x12C`); returns 1 after sub-step 6, else 0.
pub fn proc_setup<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let gd = env.callv(0x0201_20F4, &[gs]);
    let mut running = true;
    let zone = |env: &mut E| env.r16(fm + 0xD0) as u32;
    let heap = |env: &mut E| env.r16(fm) as u32;
    match env.r8(fm + 0xC8) {
        0 => {
            let z = zone(env);
            // the spec's stub lists no arguments
            env.call(0x0216_447C, &mut [CArg::X(gd), CArg::X(z)]);
            let (h, a4) = (heap(env), env.r32(fm + 0xA4));
            let v = env.callv(0x0219_01EC, &[h, a4]);
            env.w32(fm + 0x2C, 0);
            env.w32(fm + 0x28, v);
            // the third word is r2, still the result of `FUN_021901EC` (the spec's stub logs it)
            let v = env.callv(0x021B_C388, &[gs, 0x93, v]);
            env.w32(fm + 0x30, v);
            let gd8 = env.r32(fm + 8);
            let x = env.callv(0x0201_2B38, &[gd8]);
            let y = env.callv(0x0201_4470, &[x]);
            let h = heap(env);
            let v = env.callv(0x021C_7F70, &[y, h]);
            env.w32(fm + 0xA0, v);
            let z = zone(env);
            let t = env.callv(0x0201_3BBC, &[z]);
            let (h, a4) = (heap(env), env.r32(fm + 0xA4));
            let v = env.callv(0x0218_E018, &[t, 0, a4, fm + 0xF4, h]);
            env.w32(fm + 0x10, v);
            let h = heap(env);
            let v = env.callv(0x021A_B5F4, &[h, v, fm]);
            env.w32(fm + 0x40, v);
            let h = heap(env);
            let v = env.callv(0x021A_B7E4, &[h]);
            env.w32(fm + 0x44, v);
        }
        1 => {
            let (h, a, b, c) = (
                heap(env),
                env.r32(fm + 0x10),
                env.r32(fm + 0x40),
                env.r32(fm + 0x44),
            );
            let v = env.callv(0x021B_B1F4, &[h, a, b, c]);
            env.w32(fm + 0x3C, v);
            let (h, z) = (heap(env), zone(env));
            let v = env.callv(0x021D_33A8, &[h, gs, fm, z]);
            env.w32(fm + 0x9C, v);
            let d = env.r32(fm + 0x48);
            let w = env.callv(0x0218_DC6C, &[d]);
            let gd8 = env.r32(fm + 8);
            let mtx = env.callv(0x0201_2974, &[gd8]);
            let (z, ad) = (zone(env), env.r32(fm + 0xC));
            map_setup(env, z, ad, fm + 0x50, mtx);
            let gs4 = env.r32(fm + 4);
            env.callv(0x0201_2138, &[gs4]);
            let o = env.r32(fm + 0x9C);
            let e = env.callv(0x021D_33F8, &[o]);
            let (z, ad) = (zone(env), env.r32(fm + 0xC));
            env.callv(0x0218_B8AC, &[w, z, ad, e]);
            let o = env.r32(fm + 0x130);
            env.callv(0x021C_6534, &[o, e]);
            zone_light(env, gd, fm);
        }
        2 => {
            let (z, x, y, w) = (
                zone(env),
                env.r32(fm + 0xF4),
                env.r32(fm + 0xF8),
                env.r32(fm + 0xFC),
            );
            env.call(
                0x0201_4110,
                &mut [
                    CArg::Out(fm + 0xD4, 0x1C),
                    CArg::V(z),
                    CArg::V(0),
                    CArg::V(0),
                    CArg::V(0),
                    CArg::V(x),
                    CArg::V(y),
                    CArg::V(w),
                ],
            );
            let gs4 = env.r32(fm + 4);
            env.callv(0x0201_2138, &[gs4]);
            let o = env.r32(fm + 0x9C);
            let e = env.callv(0x021D_33F8, &[o]);
            let d = env.r32(fm + 0x48);
            env.callv(0x0218_CEE4, &[d, fm + 0x50, e]);
            let z = zone(env);
            let w = env.callv(0x0201_3C9C, &[z]);
            if w != u32::MAX {
                let (h, o) = (heap(env), env.r32(fm + 0x3C));
                // the fourth word is r3 (0 from the LOCATION call); the callee's use of it is UNVERIFIED
                env.callv(0x021B_B2F8, &[o, w, h, 0]);
            }
            let z = zone(env);
            let v = zone_494(env, z);
            env.w32(fm + 0x104, v);
            let (z, h) = (zone(env), heap(env));
            // `FUN_021895E8` through its stub (UNVERIFIED port, see `zone_change_10_at`)
            env.callv(0x0218_95E8, &[fm, z, h]);
        }
        3 => {
            mmdl_setup(env, fm);
            let h = heap(env);
            let o = env.callv(0x021A_BD50, &[fm, 0x18, h]);
            env.w32(fm + 0x94, o);
            env.callv(0x021A_BF68, &[o, 0x5E]);
            let o9c = env.r32(fm + 0x9C);
            let e = env.callv(0x021D_33F8, &[o9c]);
            let o = env.r32(fm + 0x94);
            env.callv(0x021A_C1EC, &[o, e]);
            let o = env.r32(fm + 0x94);
            // ROM word 0x021D8F00 (0x14) and table 0x021D8F10
            env.callv(0x021A_BE30, &[o, 0x021D_8F10, 0x14]);
            let z = zone(env);
            if env.callv(0x0201_3CC8, &[z]) == 1 || {
                let z = zone(env);
                env.callv(0x0201_3CE0, &[z]) == 1
            } {
                let h = heap(env);
                let v = env.callv(0x0217_F8C0, &[h]);
                env.w32(fm + 0x98, v);
            }
        }
        4 => {
            let p = env.callv(0x0201_20F8, &[gs]);
            let x = env.callv(0x0201_2AE4, &[p]);
            if env.callv(0x0201_2B28, &[p]) == 1 {
                let z = zone(env);
                if env.callv(0x0201_3C20, &[z]) == 0 {
                    env.callv(0x0201_2B2C, &[p, 0]);
                }
            }
            let a = env.callv(0x0201_2944, &[gd]);
            let b = env.callv(0x0200_8550, &[a]);
            let h = heap(env);
            let pl = env.callv(0x021A_2E50, &[p, fm, p + 4, b, h]);
            env.w32(fm + 0x8C, pl);
            for i in 0..3 {
                let v = env.r32(p + 4 + 4 * i);
                env.w32(fm + 0xF4 + 4 * i, v);
            }
            let ctrl = env.r32(fm + CTRL_OFS);
            let f = env.r32(ctrl + 4);
            env.callv(f & !1, &[fm, fm + 0xF4, x]);
            let d = env.r32(fm + 0x48);
            env.callv(0x0218_D158, &[d, fm + 0xF4]);
            slot_init(env.mem(), fm + SLOT_OFS);
            let h = heap(env);
            let v = env.callv(0x021B_E2A8, &[0x3C, 0x23, h]);
            env.w32(fm + 0x34, v);
            let z = crate::fieldmap::zone(env.mem(), fm) as u32;
            let ad = env.r32(fm + 0xC);
            env.callv(0x0218_9154, &[ad, z]);
            let h = heap(env);
            let v = env.callv(0x021A_0104, &[h]);
            env.w32(fm + 0x18, v);
            let h = heap(env);
            let v = env.callv(0x021A_02C4, &[h]);
            env.w32(fm + 0x1C, v);
            let z = zone(env);
            let a = env.callv(0x0201_3EEC, &[z]);
            let z = zone(env);
            let b = env.callv(0x0201_3F2C, &[z]);
            let o = env.r32(fm + 0x1C);
            env.callv(0x021A_0338, &[o, a, b, 1]);
            let ad = env.r32(fm + 0xC);
            let t = env.callv(0x0201_40DC, &[ad]);
            let rtc = env.callv(0x0203_F4FC, &[]);
            let season = env.callv(0x0202_E83C, &[gs]);
            let (a8, h, o18) = (env.r32(fm + 0xA8), heap(env), env.r32(fm + 0x18));
            let v = env.call(
                0x021A_03D0,
                &mut [
                    CArg::V(t),
                    CArg::V(rtc),
                    CArg::V(season),
                    CArg::V(o18),
                    CArg::V(a8),
                    CArg::X(h),
                ],
            );
            env.w32(fm + 0x14, v);
            let gd8 = env.r32(fm + 8);
            let w = env.callv(0x0201_2A30, &[gd8]);
            let season = env.callv(0x0202_E83C, &[gs]);
            let (a, b, c, d) = (
                env.r32(fm + 0x10),
                env.r32(fm + 0x14),
                env.r32(fm + 0x18),
                env.r32(fm + 0x1C),
            );
            let v = env.callv(0x021A_19E8, &[a, b, c, d, w, season, 0x92]);
            env.w32(fm + 0x20, v);
            let gd8 = env.r32(fm + 8);
            let k = env.callv(0x0201_2994, &[gd8]) & 0xFFFF;
            let o = env.r32(fm + 0x20);
            env.callv(0x021A_1B74, &[o, k, 0x92]);
            env.callv(0x021B_E1DC, &[fm]);
        }
        5 => {
            let x = env.callv(0x0201_29AC, &[gd]);
            let h = heap(env);
            let v = env.callv(0x021A_0CF4, &[h, fm, x]);
            env.w32(fm + 0x24, v);
        }
        6 => {
            let v = env.callv(0x021A_9ED8, &[fm]);
            env.w32(fm + 0x90, v);
            let x = env.callv(0x0201_2138, &[gs]);
            env.callv(0x0202_6388, &[x, fm]);
            let h = heap(env);
            let v = env.callv(0x021B_B890, &[fm, h, 0x20]);
            env.w32(fm + 0xBC, v);
            let v = env.callv(0x021C_1A08, &[0x50]);
            env.w32(fm + 0x120, v);
            // the second word is r1, still the result of `FUN_021C1A08` (the spec's stub logs it)
            let v1 = env.r32(fm + 0x120);
            let v = env.callv(0x021C_1AF4, &[0x50, v1]);
            env.w32(fm + 0x124, v);
            let h = heap(env);
            let v = env.callv(0x021C_F45C, &[h, fm]);
            env.w32(fm + 0x13C, v);
            let gd8 = env.r32(fm + 8);
            let save = env.callv(0x0201_2EBC, &[gd8]);
            let b = env.callv(0x0200_BDB0, &[save]);
            let h = heap(env);
            let v = env.callv(0x021C_31D4, &[h, fm, b]);
            env.w32(fm + 0x128, v);
            let z = zone(env);
            zone_change_scene(env, fm, z);
            running = false;
        }
        _ => {}
    }
    let s = env.r8(fm + 0xC8).wrapping_add(1);
    env.w8(fm + 0xC8, s);
    env.w32(fm + 0x12C, 0);
    (!running) as u32
}

/// C: `FUN_021885E8` (unlabeled routine at `0x021885E8`, proc step 2): the start: sub-step 0 settles the MMDLs, the light
/// and the first frame's map work ([`frame_map_work`], the draw functions) and goes on into sub-step 1: wait while
/// a hold zone ([`is_hold_zone`]) is pending (0), the script start (`FUN_02158AC8` / `FUN_02158AD4`), install the
/// event check of the map kind (`FUN_0201209C(gamesys, check, fm)`: `fieldmap_event` 0x0218A0E5 comm zones, 0x0218A241
/// rail, 0x0218A4BD hybrid, 0x0218A075 grid), running (`+0xC0 = 1`), `FUN_020129DC(gd, 1)`; 1.
///
/// UNVERIFIED port of `FUN_021885E8`: not in [`PORTED`]. Its call of the render function `FUN_02189084(fm)` (spec stub
/// `args=v,v,v,v`) logs r3, which comes from this function's entry registers (see [`proc_graphics_init`]).
pub fn proc_start<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let sub = env.r8(fm + 0xC8);
    if sub > 1 {
        return 1;
    }
    if sub == 0 {
        let d = env.r32(fm + 0x48);
        env.callv(0x0218_CA78, &[d]);
        if env.r32(fm + 0x38) != 0 {
            loop {
                let m = env.r32(fm + 0x38);
                env.callv(0x0216_CD64, &[m]);
                let m = env.r32(fm + 0x38);
                env.callv(0x0219_C624, &[m]);
                let m = env.r32(fm + 0x38);
                if env.callv(0x0219_C630, &[m]) != 1 {
                    break;
                }
            }
        }
        loop {
            let o = env.r32(fm + 0x20);
            if env.callv(0x021A_1B50, &[o]) == 0 {
                break;
            }
            let o = env.r32(fm + 0x20);
            env.callv(0x021A_1AF4, &[o, 0x92]);
        }
        frame_map_work(env, fm);
        env.call(
            0x0218_9084,
            &mut [CArg::V(fm), CArg::V(0), CArg::V(0), CArg::V(0)],
        );
        env.callv(0x0218_90AC, &[fm]);
        env.callv(0x0204_A600, &[]);
        if env.r32(fm + 0x28) != 0 {
            let o = env.r32(fm + 0x28);
            env.callv(0x0219_03E0, &[o]);
        }
        let o = env.r32(fm + 0x94);
        env.callv(0x021A_BE04, &[o]);
        let o = env.r32(fm + 0xBC);
        env.callv(0x021B_B948, &[o]);
        let s = env.r8(fm + 0xC8).wrapping_add(1);
        env.w8(fm + 0xC8, s);
    }
    let gd8 = env.r32(fm + 8);
    let x = env.callv(0x0201_2B38, &[gd8]);
    if env.callv(0x0201_4460, &[x]) == 1
        && is_hold_zone(env.r16(fm + 0xD0) as u32)
        && env.callv(0x0200_5C64, &[]) != 0
    {
        return 0;
    }
    let gd8 = env.r32(fm + 8);
    let x = env.callv(0x0201_2B38, &[gd8]);
    let (gs4, heap) = (env.r32(fm + 4), env.r16(fm) as u32);
    if env.callv(0x0201_4458, &[x]) != 0 {
        env.call(0x0215_8AC8, &mut [CArg::V(gs4), CArg::X(heap)]);
        let gd8 = env.r32(fm + 8);
        let x = env.callv(0x0201_2B38, &[gd8]);
        env.callv(0x0201_4454, &[x, 0]);
    } else {
        env.call(0x0215_8AD4, &mut [CArg::V(gs4), CArg::X(heap)]);
    }
    let gd8 = env.r32(fm + 8);
    let x = env.callv(0x0201_2B38, &[gd8]);
    env.callv(0x0201_445C, &[x, 0]);
    let d = env.r32(fm + 0x48);
    env.callv(0x0218_CA78, &[d]);
    let z = env.r16(fm + 0xD0) as u32;
    let check = if env.callv(0x0201_3CC8, &[z]) == 1 || {
        let z = env.r16(fm + 0xD0) as u32;
        env.callv(0x0201_3CE0, &[z]) == 1
    } {
        0x0218_A0E5
    } else if ctrl_kind(env.mem(), fm) == 1 {
        0x0218_A241
    } else if ctrl_kind(env.mem(), fm) == 2 {
        0x0218_A4BD
    } else {
        0x0218_A075
    };
    env.callv(0x0201_209C, &[gs, check, fm]);
    env.w32(fm + 0xC0, 1);
    let gd = env.callv(0x0201_20F4, &[gs]);
    env.callv(0x0201_29DC, &[gd, 1]);
    1
}

/// C: `FUN_02188758` (unlabeled routine at `0x02188758`, proc step 3 half 0): the frame update: the pending-script
/// flag (`FUN_0201447C`), nothing while `FUN_02012E40(gd)`; with `+0x140` only the draw step; else the zone-change
/// check ([`zone_check`]), the MAP CTRL frame callback and `FUN_02168EEC(gd)` (unless `FUN_020120D8(gamesys)`), the
/// per-frame updates of the subsystems (the comm zones' `FUN_021737A4`, else `FUN_0217AE20`; the MMDLs with EXMEMCNT
/// toggled; the FIELDMAP FUNC system's task A; the task system), the position copy from `+0xF0`, the map work
/// ([`frame_map_work`]), `FUN_021C8660(..)`, and the draw step (`FUN_021888C8`, through its stub); 0.
pub fn proc_frame<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let gs4 = env.r32(fm + 4);
    if env.callv(0x0201_2104, &[gs4]) == 0 {
        let gd8 = env.r32(fm + 8);
        let x = env.callv(0x0201_2B38, &[gd8]);
        env.callv(0x0201_447C, &[x, 0]);
    }
    let gd8 = env.r32(fm + 8);
    if env.callv(0x0201_2E40, &[gd8]) != 0 {
        return 0;
    }
    if env.r32(fm + 0x140) != 0 {
        // `FUN_021888C8` through its stub (UNVERIFIED port, see `draw_step`)
        env.callv(0x0218_88C8, &[fm]);
        env.callv(0x0204_A600, &[]);
        return 0;
    }
    env.callv(0x0203_5648, &[]);
    zone_check(env, fm);
    if env.callv(0x0201_20D8, &[gs]) == 0 {
        let ctrl = env.r32(fm + CTRL_OFS);
        let f = env.r32(ctrl + 8);
        env.callv(f & !1, &[fm, fm + 0xF4]);
        let gd8 = env.r32(fm + 8);
        env.callv(0x0216_8EEC, &[gd8]);
    }
    if env.r32(fm + 0x30) != 0 {
        let o = env.r32(fm + 0x30);
        env.callv(0x021B_C484, &[o]);
    }
    let o = env.r32(fm + 0xA0);
    env.callv(0x021C_7FAC, &[o]);
    let o = env.r32(fm + 0x8C);
    env.callv(0x021A_2EB8, &[o]);
    let z = env.r16(fm + 0xD0) as u32;
    // r1 after the branch: fm (comm zones) or the game system (else); the overlay-18 call below logs it
    let mut r1 = if env.callv(0x0201_3CC8, &[z]) == 1 || {
        let z = env.r16(fm + 0xD0) as u32;
        env.callv(0x0201_3CE0, &[z]) == 1
    } {
        let x = env.callv(0x0201_2138, &[gs]);
        env.call(0x0217_37A4, &mut [CArg::V(x), CArg::X(fm)]);
        fm
    } else {
        let (gs4, pl) = (env.r32(fm + 4), env.r32(fm + 0x8C));
        env.callv(0x0217_AE20, &[fm, gs4, pl]);
        gs4
    };
    let o = env.r32(fm + 0x24);
    env.callv(0x021A_0D68, &[o]);
    if env.r32(fm + 0x38) != 0 {
        exmem_arm7(env);
        let m = env.r32(fm + 0x38);
        env.callv(0x0216_CD64, &[m]);
        r1 = env.r16(EXMEMCNT) as u32;
        exmem_arm9(env);
    }
    env.callv(0x021B_E258, &[fm]);
    let o = env.r32(fm + 0x94);
    env.callv(0x021A_BE04, &[o]);
    if env.r32(fm + 0x98) != 0 {
        let o = env.r32(fm + 0x98);
        // the second word is r1 (not an argument of the callee; the spec's stub logs it)
        env.call(0x0217_F914, &mut [CArg::P(o), CArg::V(r1)]);
    }
    let o = env.r32(fm + 0xBC);
    env.callv(0x021B_B948, &[o]);
    if env.r32(fm + 0x28) != 0 {
        let o = env.r32(fm + 0x28);
        env.callv(0x0219_03E0, &[o]);
    }
    let t = env.r32(fm + 0x110);
    env.callv(0x0203_0D28, &[t]);
    let o = env.r32(fm + 0x134);
    env.callv(0x021C_6D90, &[o]);
    let src = env.r32(fm + 0xF0);
    if src != 0 {
        let (a, b) = (env.r32(src), env.r32(src + 4));
        env.w32(fm + 0xF4, a);
        env.w32(fm + 0xF8, b);
        let c = env.r32(src + 8);
        env.w32(fm + 0xFC, c);
    }
    let d = env.r32(fm + 0x48);
    env.callv(0x0218_D158, &[d, fm + 0xF4]);
    frame_map_work(env, fm);
    let gd8 = env.r32(fm + 8);
    let rec = env.callv(0x0201_2ED0, &[gd8]);
    let w = env.r32(rec);
    env.callv(0x021C_8660, &[w]);
    env.callv(0x0204_A600, &[]);
    env.callv(0x0218_88C8, &[fm]);
    0
}

/// C: `FUN_021888C8` (unlabeled routine at `0x021888C8`): the draw step: `FUN_02047CE4()`, the 3D handler's key update
/// `FUN_0218E104(+0x10, trigger mask)`, the camera distance (`+0x100`, [`camera_distance`]) and the map draw
/// `FUN_02189084(fm)` (render, G8) with EXMEMCNT toggled.
///
/// UNVERIFIED port of `FUN_021888C8`: not in [`PORTED`]. The render call's spec stub (`args=v,v,v,v`) logs r3, this
/// function's entry r3.
pub fn draw_step<E: FieldEnv>(env: &mut E, fm: u32) {
    env.callv(0x0204_7CE4, &[]);
    let k = env.callv(0x0203_630C, &[]) & 0xFFFF;
    let h = env.r32(fm + 0x10);
    env.callv(0x0218_E104, &[h, k]);
    let d = camera_distance(env, fm);
    env.w32(fm + 0x100, d);
    let old = env.r16(EXMEMCNT) as u32;
    exmem_arm7(env);
    env.call(
        0x0218_9084,
        &mut [CArg::V(fm), CArg::V(old), CArg::V(0), CArg::V(0)],
    );
    exmem_arm9(env);
}

/// C: `FUN_02188910` (unlabeled routine at `0x02188910`, proc step 3 half 1): the draw half: unless
/// `FUN_02012E40(gd)`, the map draw `FUN_0218CC34(+0x48)`, `FUN_021A0E58(+0x24)`, `FUN_021903E0(+0x28)`,
/// `FUN_021BC4F0(+0x30)` and the render function `FUN_021890AC(fm)` (EXMEMCNT toggled around the draws); in both
/// cases `FUN_020129F0(gd)`; 0.
pub fn proc_draw<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let gd8 = env.r32(fm + 8);
    if env.callv(0x0201_2E40, &[gd8]) == 0 {
        exmem_arm7(env);
        let d = env.r32(fm + 0x48);
        env.callv(0x0218_CC34, &[d]);
        exmem_arm9(env);
        let o = env.r32(fm + 0x24);
        env.callv(0x021A_0E58, &[o]);
        if env.r32(fm + 0x28) != 0 {
            let o = env.r32(fm + 0x28);
            env.callv(0x0219_03E0, &[o]);
        }
        if env.r32(fm + 0x30) != 0 {
            let o = env.r32(fm + 0x30);
            env.callv(0x021B_C4F0, &[o]);
        }
        exmem_arm7(env);
        env.callv(0x0218_90AC, &[fm]);
        exmem_arm9(env);
    }
    let gd = env.callv(0x0201_20F4, &[gs]);
    env.callv(0x0201_29F0, &[gd]);
    0
}

/// C: `FUN_02188990` (unlabeled routine at `0x02188990`, proc step 4): the teardown of the subsystems (the reverse of
/// [`proc_setup`]), the MAP CTRL teardown callback, [`clear_ctrl_work`], [`mmdl_delete`]; 1.
///
/// UNVERIFIED port of `FUN_02188990`: not in [`PORTED`]. The spec's stubs `021c1c04` / `021c1ad0 args=v,v,v,v` log r2
/// and r3, this function's entry registers.
pub fn proc_teardown<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let o = env.r32(fm + 0x13C);
    env.callv(0x021C_F480, &[o]);
    let o = env.r32(fm + 0x124);
    env.callv(0x021C_1C04, &[o, fm, 0, 0]);
    let o = env.r32(fm + 0x120);
    env.callv(0x021C_1AD0, &[o, fm, 0, 0]);
    env.callv(0x0201_209C, &[gs, 0, 0]);
    let o = env.r32(fm + 0x8C);
    env.callv(0x021A_2EB8, &[o]);
    let o = env.r32(fm + 0xBC);
    env.callv(0x021B_B90C, &[o]);
    let x = env.callv(0x0201_2138, &[gs]);
    env.callv(0x0202_63B0, &[x, fm]);
    if env.r32(fm + 0x30) != 0 {
        let o = env.r32(fm + 0x30);
        env.callv(0x021B_C424, &[o]);
    }
    env.callv(0x021B_E218, &[fm]);
    let o = env.r32(fm + 0x34);
    env.callv(0x021B_E2E0, &[o]);
    let o = env.r32(fm + 0x90);
    env.callv(0x021A_9F28, &[o]);
    let gd = env.callv(0x0201_20F4, &[gs]);
    let o = env.r32(fm + 0x24);
    let v = env.callv(0x021A_0D44, &[o]);
    env.callv(0x0201_29B8, &[gd, v]);
    let o = env.r32(fm + 0x94);
    env.callv(0x021A_BDE8, &[o]);
    if env.r32(fm + 0x98) != 0 {
        let o = env.r32(fm + 0x98);
        env.callv(0x0217_F8EC, &[o]);
        env.w32(fm + 0x98, 0);
    }
    let o = env.r32(fm + 0x20);
    env.callv(0x021A_1AAC, &[o]);
    env.w32(fm + 0x20, 0);
    for (f, ofs) in [
        (0x021A_0448, 0x14),
        (0x021A_02F4, 0x1C),
        (0x021A_0138, 0x18),
        (0x021D_33E0, 0x9C),
        (0x021B_B240, 0x3C),
        (0x021A_B81C, 0x44),
        (0x021A_B624, 0x40),
        (0x0218_E0E4, 0x10),
    ] {
        let o = env.r32(fm + ofs);
        env.callv(f, &[o]);
    }
    let ctrl = env.r32(fm + CTRL_OFS);
    let f = env.r32(ctrl + 0xC);
    env.callv(f & !1, &[fm]);
    clear_ctrl_work(env.mem(), fm);
    mmdl_delete(env, fm);
    let o = env.r32(fm + 0x8C);
    env.callv(0x021A_2E90, &[o]);
    let d = env.r32(fm + 0x48);
    env.callv(0x0218_D05C, &[d]);
    if env.r32(fm + 0x2C) != 0 {
        let o = env.r32(fm + 0x2C);
        env.call(
            0x0219_0728,
            // r2 = 0 (from `FUN_0201209C(gs, 0, 0)`), r3 = the target `FUN_021A0138`'s veneer left (not read)
            &mut [CArg::V(o), CArg::P(0), CArg::V(0), CArg::V(0x0203_07B0)],
        );
    }
    if env.r32(fm + 0x28) != 0 {
        let o = env.r32(fm + 0x28);
        env.callv(0x0219_02BC, &[o]);
    }
    let gd = env.callv(0x0201_20F4, &[gs]);
    env.callv(0x0201_29DC, &[gd, 0]);
    env.callv(0x0203_5660, &[0x3C]);
    let o = env.r32(fm + 0x128);
    env.callv(0x021C_31FC, &[o]);
    let o = env.r32(fm + 0xA0);
    env.callv(0x021C_7F9C, &[o]);
    1
}

/// C: `FUN_02188AD4` (unlabeled routine at `0x02188AD4`, proc step 5): graphics exit: free the task system, `+0x14C`,
/// `+0x134`, the VBlank task, the map draw object, the render teardown `FUN_021890C8(fm)` (G8), `FUN_0204A5AC()`,
/// `FUN_020450C8()`, `FUN_02188E38(fm)` (G8), unload the overlay of [`proc_graphics_init`]; 2 (finished).
///
/// UNVERIFIED port of `FUN_02188AD4`: not in [`PORTED`]. The render stubs `021890c8 args=v,v,v,v` /
/// `02188e38 args=p,v,v,v` log registers the functions do not read (this function's entry r2 / r3).
pub fn proc_graphics_exit<E: FieldEnv>(env: &mut E, _gs: u32, fm: u32) -> u32 {
    let t = env.r32(fm + 0x110);
    env.call(0x0203_0DA4, &mut [CArg::X(t)]);
    let b = env.r32(fm + 0x114);
    env.callv(HEAP_FREE, &[b]);
    let o = env.r32(fm + 0x14C);
    env.callv(0x021D_31A4, &[o]);
    env.w32(fm + 0x14C, 0);
    let o = env.r32(fm + 0x134);
    env.callv(0x021C_6D5C, &[o]);
    let t = env.r32(fm + 0xAC);
    env.callv(0x0203_0EAC, &[t]);
    let d = env.r32(fm + 0x48);
    env.callv(0x0218_CA4C, &[d]);
    env.callv(0x0218_90C8, &[fm, 0, 0, 0]);
    env.callv(0x0204_A5AC, &[]);
    env.callv(0x0204_50C8, &[]);
    env.call(
        0x0218_8E38,
        &mut [CArg::P(fm), CArg::V(0), CArg::V(0), CArg::V(0)],
    );
    let z = env.r16(fm + 0xD0) as u32;
    let ov = if env.callv(0x0201_3CC8, &[z]) == 1 || {
        let z = env.r16(fm + 0xD0) as u32;
        env.callv(0x0201_3CE0, &[z]) == 1
    } {
        19
    } else {
        18
    };
    env.callv(0x0203_4A5C, &[ov]);
    2
}

/// The FIELDMAP proc steps (ROM `0x021D36B8`, 6 steps x {half 0, half 1}); a step answers 0 (switch half), 1 (next
/// step) or 2 (finished).
pub const PROC_TABLE: [[u32; 2]; 6] = [
    [0x0218_80C9, 0x0218_80C9],
    [0x0218_81D9, 0x0218_81D9],
    [0x0218_85E9, 0x0218_85E9],
    [0x0218_8759, 0x0218_8911],
    [0x0218_8991, 0x0218_8991],
    [0x0218_8AD5, 0x0218_8AD5],
];

/// Run the proc step function at `f` (a [`PROC_TABLE`] entry): the ported ones as Rust, the UNVERIFIED ports through
/// their stubs.
fn run_proc_fn<E: FieldEnv>(env: &mut E, f: u32, gs: u32, fm: u32) -> u32 {
    match f & !1 {
        0x0218_81D8 => proc_setup(env, gs, fm),
        0x0218_8758 => proc_frame(env, gs, fm),
        0x0218_8910 => proc_draw(env, gs, fm),
        // the graphics steps do not read the game system
        a @ (0x0218_80C8 | 0x0218_8AD4) => env.call(a, &mut [CArg::P(gs), CArg::V(fm)]),
        a => env.callv(a, &[gs, fm]),
    }
}

/// C: `FUN_02188B58` (unlabeled routine at `0x02188B58`): the proc driver: count the frame (`+0xCC`), run the step
/// `PROC_TABLE[+0xC4][+0xC9]`: 0 toggles the half (`+0xC9`), 1 moves to the next step (sub-step and half 0), 2
/// returns 1 (finished); else 0.
pub fn proc_step<E: FieldEnv>(env: &mut E, gs: u32, fm: u32) -> u32 {
    let c = env.r32(fm + 0xCC).wrapping_add(1);
    env.w32(fm + 0xCC, c);
    let step = env.r32(fm + 0xC4) as usize;
    let half = env.r8(fm + 0xC9) as usize;
    let f = PROC_TABLE[step][half];
    let r = if f != 0 {
        run_proc_fn(env, f, gs, fm)
    } else {
        0
    };
    match r {
        0 => {
            let h = (env.r8(fm + 0xC9) == 0) as u8;
            env.w8(fm + 0xC9, h);
        }
        1 => {
            let s = env.r32(fm + 0xC4).wrapping_add(1);
            env.w32(fm + 0xC4, s);
            env.w8(fm + 0xC8, 0);
            env.w8(fm + 0xC9, 0);
        }
        2 => return 1,
        _ => {}
    }
    0
}

/// C: `FUN_02189004` (unlabeled routine at `0x02189004`): the map work of a frame: `FUN_020504D8` on `+0xB0` / `+0xB4`
/// / `+0xB8`, `FUN_021BB36C(+0x3C)`, the scene area at the position (`FUN_021AB660`), `FUN_021A030C(+0x1C, heap)`,
/// `FUN_021A1AF4(+0x20, 0x92)`, `FUN_021A0140(+0x18)`, `FUN_021A0458(+0x14, RTC seconds)`, the map draw's load step
/// `FUN_0218CB34(+0x48)` (EXMEMCNT toggled) and, when it finished something, the sound `FUN_0203D12C(2)`.
pub fn frame_map_work<E: FieldEnv>(env: &mut E, fm: u32) {
    for o in [0xB0, 0xB4, 0xB8] {
        let r = env.r32(fm + o);
        env.callv(0x0205_04D8, &[r]);
    }
    let o = env.r32(fm + 0x3C);
    env.callv(0x021B_B36C, &[o]);
    let o = env.r32(fm + 0x40);
    env.callv(0x021A_B660, &[o, fm + 0xF4]);
    let (o, h) = (env.r32(fm + 0x1C), env.r16(fm) as u32);
    env.callv(0x021A_030C, &[o, h]);
    let o = env.r32(fm + 0x20);
    env.callv(0x021A_1AF4, &[o, 0x92]);
    let o = env.r32(fm + 0x18);
    env.callv(0x021A_0140, &[o]);
    let t = env.callv(0x0203_F4FC, &[]);
    let o = env.r32(fm + 0x14);
    env.callv(0x021A_0458, &[o, t]);
    exmem_arm7(env);
    let d = env.r32(fm + 0x48);
    let r = env.callv(0x0218_CB34, &[d]);
    exmem_arm9(env);
    if r != 0 {
        env.callv(0x0203_D12C, &[2]);
    }
}

/// C: `FUN_02189204` (unlabeled routine at `0x02189204`): create the MMDL system: `+0x38 = FUN_02012964(gd)`,
/// `FUN_0216CCDC(m, heap, gd, fm, +0x48, +0x3C)`, `FUN_0219C554(m, +0xB0, 0x20)`, `FUN_0219C680(m)`, the player's
/// MMDL form `FUN_0219C658(m, FUN_021A31A4(x, 0))` (`x` from `FUN_0217C0A8(..)` outside the comm zones, else the
/// player's `FUN_02008550(..)`), `FUN_0219D448(m, +0x130)`, `FUN_0216CDB8(m, FUN_0218ED50(+0x10))`,
/// `FUN_0216D2F8(m)`, `FUN_0216D974(m, FUN_02189AE8(setup zone))`.
pub fn mmdl_setup<E: FieldEnv>(env: &mut E, fm: u32) {
    let gs = env.r32(fm + 4);
    let gd = env.callv(0x0201_20F4, &[gs]);
    let m = env.callv(0x0201_2964, &[gd]);
    env.w32(fm + 0x38, m);
    let (d, o, h) = (env.r32(fm + 0x48), env.r32(fm + 0x3C), env.r16(fm) as u32);
    env.callv(0x0216_CCDC, &[m, h, gd, fm, d, o]);
    let (m2, b0) = (env.r32(fm + 0x38), env.r32(fm + 0xB0));
    env.callv(0x0219_C554, &[m2, b0, 0x20]);
    let m2 = env.r32(fm + 0x38);
    env.callv(0x0219_C680, &[m2]);
    let z = env.r16(fm + 0xD8) as u32;
    let x = if env.callv(0x0201_3CC8, &[z]) == 0 && env.callv(0x0201_3CE0, &[z]) == 0 {
        let gs4 = env.r32(fm + 4);
        let y = env.callv(0x0201_2138, &[gs4]);
        env.callv(0x0217_C0A8, &[y])
    } else {
        let a = env.callv(0x0201_2944, &[gd]);
        env.callv(0x0200_8550, &[a])
    };
    let f = env.callv(0x021A_31A4, &[x, 0]);
    env.callv(0x0219_C658, &[m, f]);
    let (m2, o) = (env.r32(fm + 0x38), env.r32(fm + 0x130));
    env.callv(0x0219_D448, &[m2, o]);
    let h = env.r32(fm + 0x10);
    let v = env.callv(0x0218_ED50, &[h]);
    let m2 = env.r32(fm + 0x38);
    env.callv(0x0216_CDB8, &[m2, v]);
    let m2 = env.r32(fm + 0x38);
    env.callv(0x0216_D2F8, &[m2]);
    let z = env.r16(fm + 0xD0) as u32;
    let k = zone_not_3f(env, z);
    env.callv(0x0216_D974, &[m, k]);
}

/// C: `FUN_02188BD8` (unlabeled routine at `0x02188BD8`): stop: state (`+0xC0`) 2, proc step (`+0xC4`) 4 (teardown).
pub fn request_end(mem: &mut Mem, fm: u32) {
    mem.w32(fm + 0xC0, 2);
    mem.w32(fm + 0xC4, 4);
}

/// C: `FUN_02188BE8` (unlabeled routine at `0x02188BE8`): the FIELDMAP is running (state `+0xC0` == 1).
pub fn is_running(mem: &Mem, fm: u32) -> bool {
    mem.r32(fm + 0xC0) == 1
}

/// C: `FUN_02188BF8` (unlabeled routine at `0x02188BF8`): end the player's special state: `r = FUN_021A3098(player)`;
/// 1: `FUN_021A2F48(player, 1)`; 0: sound `FUN_020061E4(0x55F)`, `FUN_021A2F48(player, 2)`, `FUN_02008DF0(save x, 3)`;
/// in both cases `FUN_021A2F54(player)` and true; else false.
pub fn end_player_state<E: FieldEnv>(env: &mut E, fm: u32) -> bool {
    let pl = env.r32(fm + 0x8C);
    let r = env.callv(0x021A_3098, &[pl]);
    let done = match r {
        1 => {
            let pl = env.r32(fm + 0x8C);
            // the spec's stub lists the first argument
            env.call(0x021A_2F48, &mut [CArg::V(pl), CArg::X(1)]);
            true
        }
        0 => {
            env.call(0x0200_61E4, &mut [CArg::X(0x55F)]);
            let pl = env.r32(fm + 0x8C);
            env.call(0x021A_2F48, &mut [CArg::V(pl), CArg::X(2)]);
            let gd = env.r32(fm + 8);
            let x = env.callv(0x0201_2F2C, &[gd]);
            env.callv(0x0200_8DF0, &[x, 3]);
            true
        }
        _ => false,
    };
    if done {
        let pl = env.r32(fm + 0x8C);
        env.callv(0x021A_2F54, &[pl]);
    }
    done
}

getter!(
    /// C: `FUN_02188C54` (unlabeled routine at `0x02188C54`): `+0x28`.
    get_28, 0x28
);
getter!(
    /// C: `FUN_02188C58` (unlabeled routine at `0x02188C58`): `+0x10` (the map 3D handler).
    get_10, 0x10
);
getter!(
    /// C: `FUN_02188C5C` (unlabeled routine at `0x02188C5C`): `+0x3C`.
    get_3c, 0x3C
);
getter!(
    /// C: `FUN_02188C60` (unlabeled routine at `0x02188C60`): `+0x14`.
    get_14, 0x14
);
getter!(
    /// C: `FUN_02188C64` (unlabeled routine at `0x02188C64`): `+0x18`.
    get_18, 0x18
);
getter!(
    /// C: `FUN_02188C68` (unlabeled routine at `0x02188C68`): `+0x1C`.
    get_1c, 0x1C
);
getter!(
    /// C: `FUN_02188C6C` (unlabeled routine at `0x02188C6C`): `+0x20`.
    get_20, 0x20
);

/// C: `FUN_02188C70` (unlabeled routine at `0x02188C70`): `FUN_0202C794(gamesys)` (a tail call).
pub fn gamesys_0202c794<E: FieldEnv>(env: &mut E, fm: u32) -> u32 {
    let gs = env.r32(fm + 4);
    env.callv(0x0202_C794, &[gs])
}

getter!(
    /// C: `FUN_02188C7C` (unlabeled routine at `0x02188C7C`): `+0x38` (the MMDL system).
    mmdl_sys, 0x38
);
getter!(
    /// C: `FUN_02188C80` (unlabeled routine at `0x02188C80`): `+4` (the game system).
    gamesys, 4
);

/// C: `FUN_02188C84` (unlabeled routine at `0x02188C84`): the heap id (`u16 +0`).
pub fn heap(mem: &Mem, fm: u32) -> u16 {
    mem.r16(fm)
}

getter!(
    /// C: `FUN_02188C88` (unlabeled routine at `0x02188C88`): `+0xB4`.
    get_b4, 0xB4
);
getter!(
    /// C: `FUN_02188C90` (unlabeled routine at `0x02188C90`): `+0xB8`.
    get_b8, 0xB8
);
getter!(
    /// C: `FUN_02188C98` (unlabeled routine at `0x02188C98`): `+0x48` (the map draw object).
    get_48, 0x48
);

/// C: `FUN_02188C9C` (unlabeled routine at `0x02188C9C`): the current zone id (`u16 +0xD8`).
pub fn zone(mem: &Mem, fm: u32) -> u16 {
    mem.r16(fm + 0xD8)
}

getter!(
    /// C: `FUN_02188CA4` (unlabeled routine at `0x02188CA4`): `+0x10C` (the MAP CTRL work).
    ctrl_work, 0x10C
);
setter!(
    /// C: `FUN_02188CAC` (unlabeled routine at `0x02188CAC`): `+0x10C = v`.
    set_ctrl_work, 0x10C
);
getter!(
    /// C: `FUN_02188CB4` (unlabeled routine at `0x02188CB4`): `+0x8C` (the player).
    player, 0x8C
);

/// C: `FUN_02188CBC` (unlabeled routine at `0x02188CBC`): there is a player (`+0x8C != 0`).
pub fn has_player(mem: &Mem, fm: u32) -> bool {
    mem.r32(fm + 0x8C) != 0
}

getter!(
    /// C: `FUN_02188CCC` (unlabeled routine at `0x02188CCC`): `+0x24`.
    get_24, 0x24
);
getter!(
    /// C: `FUN_02188CD0` (unlabeled routine at `0x02188CD0`): `+0x94`.
    get_94, 0x94
);
getter!(
    /// C: `FUN_02188CD8` (unlabeled routine at `0x02188CD8`): `+0x130`.
    get_130, 0x130
);
getter!(
    /// C: `FUN_02188CE0` (unlabeled routine at `0x02188CE0`): `+0x98`.
    get_98, 0x98
);
getter!(
    /// C: `FUN_02188CE8` (unlabeled routine at `0x02188CE8`): `+0x90` (the encounter object).
    encount, 0x90
);

/// C: `FUN_02188CF0` (unlabeled routine at `0x02188CF0`): the MAP CTRL kind (`u32 *(+0x108)`).
pub fn ctrl_kind(mem: &Mem, fm: u32) -> u32 {
    mem.r32(mem.r32(fm + CTRL_OFS))
}

/// C: `FUN_02188CFC` (unlabeled routine at `0x02188CFC`): the map kind of the MAP CTRL ([`CTRL_KIND`]); a hybrid map
/// (2) answers `FUN_021A6DC0(ctrl work)` instead (grid or rail at the player's place).
pub fn map_kind<E: FieldEnv>(env: &mut E, fm: u32) -> u32 {
    let ctrl = env.r32(fm + CTRL_OFS);
    let k = CTRL_KIND[env.r32(ctrl) as usize] as u32;
    if k == 2 {
        let w = env.r32(fm + 0x10C);
        return env.callv(0x021A_6DC0, &[w]);
    }
    k
}

getter!(
    /// C: `FUN_02188D20` (unlabeled routine at `0x02188D20`): `+0x30`.
    get_30, 0x30
);
getter!(
    /// C: `FUN_02188D24` (unlabeled routine at `0x02188D24`): `+0xBC` (the FIELDMAP FUNC system).
    func_sys, 0xBC
);
getter!(
    /// C: `FUN_02188D2C` (unlabeled routine at `0x02188D2C`): `+0x34`.
    get_34, 0x34
);
getter!(
    /// C: `FUN_02188D30` (unlabeled routine at `0x02188D30`): `+0x110` (the task system).
    tasks, 0x110
);
getter!(
    /// C: `FUN_02188D38` (unlabeled routine at `0x02188D38`): `+0x134`.
    get_134, 0x134
);
setter!(
    /// C: `FUN_02188D40` (unlabeled routine at `0x02188D40`): `+0xF0 = p` (the position source).
    set_position_src, 0xF0
);
getter!(
    /// C: `FUN_02188D48` (unlabeled routine at `0x02188D48`): `+0x2C`.
    get_2c, 0x2C
);
setter!(
    /// C: `FUN_02188D4C` (unlabeled routine at `0x02188D4C`): `+0x2C = v`.
    set_2c, 0x2C
);
getter!(
    /// C: `FUN_02188D50` (unlabeled routine at `0x02188D50`): `+0x128`.
    get_128, 0x128
);
getter!(
    /// C: `FUN_02188D58` (unlabeled routine at `0x02188D58`): `+0xC` (the area data).
    area, 0xC
);
getter!(
    /// C: `FUN_02188E30` (unlabeled routine at `0x02188E30`): `+0x14C`.
    get_14c, 0x14C
);

/// EXMEMCNT (`REG_EXMEMCNT`, 0x04000204): the original clears bit 15 (ARM9 owns the card bus) around archive reads
/// of the zone data and sets it again after; the differential specs own the register as a `global`.
pub const EXMEMCNT: u32 = 0x0400_0204;

fn exmem_arm7<E: FieldEnv>(env: &mut E) {
    let v = env.r16(EXMEMCNT) & 0x7FFF;
    env.w16(EXMEMCNT, v);
}

fn exmem_arm9<E: FieldEnv>(env: &mut E) {
    let v = env.r16(EXMEMCNT) | 0x8000;
    env.w16(EXMEMCNT, v);
}

/// C: `FUN_021892B4` (unlabeled routine at `0x021892B4`): delete the MMDL system (`FUN_0216D2AC`, `FUN_0216CD50`) and
/// clear `+0x38`.
pub fn mmdl_delete<E: FieldEnv>(env: &mut E, fm: u32) {
    let m = env.r32(fm + 0x38);
    env.callv(0x0216_D2AC, &[m]);
    let m = env.r32(fm + 0x38);
    env.callv(0x0216_CD50, &[m]);
    env.w32(fm + 0x38, 0);
}

/// C: `FUN_021892CC` (unlabeled routine at `0x021892CC`): the zone-change check: the player moved ([`player_moved`])
/// into another zone ([`zone_changed`]): [`zone_change`] and true. Otherwise, unless `FUN_02162458(zone events)`,
/// reload the zone events `FUN_02162438(events, zone, FUN_02012984(gd))` (EXMEMCNT toggled around it); false.
pub fn zone_check<E: FieldEnv>(env: &mut E, fm: u32) -> bool {
    if player_moved(env, fm) && zone_changed(env, fm) {
        zone_change(env, fm);
        return true;
    }
    let gd = env.r32(fm + 8);
    let zev = env.callv(0x0201_2844, &[gd]);
    if env.callv(0x0216_2458, &[zev]) == 0 {
        exmem_arm7(env);
        let gd = env.r32(fm + 8);
        let x = env.callv(0x0201_2984, &[gd]);
        let z = env.r16(fm + 0xD8) as u32;
        env.callv(0x0216_2438, &[zev, z, x]);
        exmem_arm9(env);
    }
    false
}

/// The LOCATION position (`FUN_020141E0(fm + 0xD4, &pos)`, fx32 x, y, z) into a local.
fn location_pos<E: FieldEnv>(env: &mut E, fm: u32) -> [u32; 3] {
    let mut pos = [0u8; 12];
    env.call(
        0x0201_41E0,
        &mut [CArg::V(fm + 0xD4), CArg::OutBuf(&mut pos)],
    );
    let w = |i: usize| u32::from_le_bytes(pos[4 * i..4 * i + 4].try_into().unwrap());
    [w(0), w(1), w(2)]
}

/// C: `FUN_0218932C` (unlabeled routine at `0x0218932C`): the player record's position (`FUN_02012AA0(..)`) differs in x
/// or z from the LOCATION: copy it in (`FUN_020141F0(loc, p)`) and true.
pub fn player_moved<E: FieldEnv>(env: &mut E, fm: u32) -> bool {
    let gs = env.r32(fm + 4);
    let gd = env.callv(0x0201_20F4, &[gs]);
    let a = env.callv(0x0201_27A4, &[gd]);
    let p = env.callv(0x0201_2AA0, &[a]);
    let pos = location_pos(env, fm);
    if env.r32(p) != pos[0] || env.r32(p + 8) != pos[2] {
        env.callv(0x0201_41F0, &[fm + 0xD4, p]);
        return true;
    }
    false
}

/// C: `FUN_02189370` (unlabeled routine at `0x02189370`): the LOCATION is inside the map matrix and its zone
/// (`FUN_02159BD4`, map_matrix) is a zone (not 0xFFFF) other than the current one (`s16 +0xD8`).
pub fn zone_changed<E: FieldEnv>(env: &mut E, fm: u32) -> bool {
    let gd = env.r32(fm + 8);
    let mtx = env.callv(0x0201_2974, &[gd]);
    let pos = location_pos(env, fm);
    if env.callv(0x0215_9C48, &[mtx, pos[0], pos[2]]) != 1 {
        return false;
    }
    let z = env.callv(0x0215_9BD4, &[mtx, pos[0], pos[2]]);
    z != 0xFFFF && z != env.r16(fm + 0xD8) as i16 as i32 as u32
}

/// C: `FUN_021893C0` (unlabeled routine at `0x021893C0`): change to the zone at the LOCATION: sound `FUN_0203D12C(2)`,
/// `FUN_021AA8EC(encount)`, reload the zone events (`FUN_0216DF1C`, `FUN_021623DC`, `FUN_0216242C`; EXMEMCNT toggled),
/// `FUN_021643AC(gd, fm)`, the zone helpers [`zone_change_save`], [`zone_change_1c`], [`zone_change_20`],
/// [`zone_change_10`], [`zone_change_record`], [`zone_change_scene`], `FUN_021BC550(+0x30, z)` (unless
/// `FUN_020120D8(gamesys)`), store the zone, `FUN_0202735C(z, gd)`, the fly point flag (`FUN_0215A014`, map_matrix),
/// `FUN_02158AE0(gamesys, 4)`, [`zone_change_mmdl`] (EXMEMCNT toggled) and `FUN_0215874C(..)`.
pub fn zone_change<E: FieldEnv>(env: &mut E, fm: u32) {
    let gs = env.r32(fm + 4);
    let gd = env.callv(0x0201_20F4, &[gs]);
    let zev = env.callv(0x0201_2844, &[gd]);
    let m = env.r32(fm + 0x38);
    let gd8 = env.r32(fm + 8);
    let mtx = env.callv(0x0201_2974, &[gd8]);
    env.callv(0x0203_D12C, &[2]);
    let e = env.r32(fm + 0x90);
    env.callv(0x021A_A8EC, &[e]);
    let pos = location_pos(env, fm);
    let z = env.callv(0x0215_9BD4, &[mtx, pos[0], pos[2]]);
    exmem_arm7(env);
    env.callv(0x0216_DF1C, &[m]);
    env.callv(0x0216_23DC, &[zev]);
    let x = env.callv(0x0201_2984, &[gd]);
    // the spec's stub lists the first two arguments
    env.call(
        0x0216_242C,
        &mut [CArg::V(zev), CArg::V(z & 0xFFFF), CArg::X(x)],
    );
    exmem_arm9(env);
    env.callv(0x0216_43AC, &[gd, fm]);
    zone_change_save(env, fm, z);
    zone_change_1c(env, fm, z);
    zone_change_20(env, fm, z);
    // `FUN_021895D0` -> `FUN_021895E8` through its stub (UNVERIFIED port, see `zone_change_10_at`)
    zone_change_10(env, fm, z);
    zone_change_record(env, gd, z);
    zone_change_scene(env, fm, z);
    if env.r32(fm + 0x30) != 0 {
        let gs = env.r32(fm + 4);
        if env.callv(0x0201_20D8, &[gs]) == 0 {
            let o = env.r32(fm + 0x30);
            env.callv(0x021B_C550, &[o, z]);
        }
    }
    env.w16(fm + 0xD8, z as u16);
    env.callv(0x0202_735C, &[z & 0xFFFF, gd]);
    env.callv(0x0215_A014, &[gd, z]);
    let gs = env.r32(fm + 4);
    env.callv(0x0215_8AE0, &[gs, 4]);
    exmem_arm7(env);
    zone_change_mmdl(env, gd, m, zev, z);
    exmem_arm9(env);
    let gs = env.r32(fm + 4);
    let x = env.callv(0x0201_2138, &[gs]);
    env.callv(0x0215_874C, &[x]);
}

/// C: `FUN_021894E0` (unlabeled routine at `0x021894E0`): `FUN_0216CEAC(m, FUN_02162740(p), z, e, event work)` for the
/// zone's MMDL list `e = FUN_02162744(p)`, if any.
pub fn zone_change_mmdl<E: FieldEnv>(env: &mut E, gd: u32, m: u32, p: u32, z: u32) {
    let e = env.callv(0x0216_2744, &[p]);
    if e != 0 {
        let ev = env.callv(0x0201_296C, &[gd]);
        let x = env.callv(0x0216_2740, &[p]);
        env.callv(0x0216_CEAC, &[m, x, z, e, ev]);
    }
}

/// C: `FUN_02189518` (unlabeled routine at `0x02189518`): `FUN_0202B544(x, z)` and, unless the player record kind
/// (`FUN_02012B28`) is 2, `FUN_02029CF0(w, gd, z)`.
pub fn zone_change_save<E: FieldEnv>(env: &mut E, fm: u32, z: u32) {
    let gs = env.r32(fm + 4);
    let gd = env.callv(0x0201_20F4, &[gs]);
    let gs = env.r32(fm + 4);
    let x = env.callv(0x0201_2134, &[gs]);
    let w = env.callv(0x0201_2A30, &[gd]);
    let a = env.callv(0x0201_27A4, &[gd]);
    let k = env.callv(0x0201_2B28, &[a]);
    env.callv(0x0202_B544, &[x, z & 0xFFFF]);
    if k != 2 {
        env.callv(0x0202_9CF0, &[w, gd, z & 0xFFFF]);
    }
}

/// C: `FUN_02189560` (unlabeled routine at `0x02189560`): `FUN_0202C83C(gamesys, z)`; unless `FUN_02012994(gd)` is 0xF,
/// `FUN_021A1BDC(+0x20, ..)` (a thunk to `FUN_021A1C34`; r1 still holds that value).
pub fn zone_change_20<E: FieldEnv>(env: &mut E, fm: u32, z: u32) {
    let s = get_20(env.mem(), fm);
    let gs = env.r32(fm + 4);
    env.callv(0x0202_C83C, &[gs, z]);
    let gd = env.r32(fm + 8);
    let k = env.callv(0x0201_2994, &[gd]) & 0xFFFF;
    if k != 0xF {
        env.callv(0x021A_1BDC, &[s, k]);
    }
}

/// C: `FUN_0218958C` (unlabeled routine at `0x0218958C`): `FUN_021A0358(+0x1C)`, then `FUN_021A0338(+0x1C,
/// FUN_02013EEC(z), FUN_02013F2C(z), 1)`.
pub fn zone_change_1c<E: FieldEnv>(env: &mut E, fm: u32, z: u32) {
    let o = get_1c(env.mem(), fm);
    env.callv(0x021A_0358, &[o]);
    let a = env.callv(0x0201_3EEC, &[z & 0xFFFF]);
    let b = env.callv(0x0201_3F2C, &[z & 0xFFFF]);
    let o = env.r32(fm + 0x1C);
    env.callv(0x021A_0338, &[o, a, b, 1]);
}

/// C: `FUN_021895BC` (unlabeled routine at `0x021895BC`): `FUN_02012AC4(FUN_020127A4(gd), z)` (the player record's
/// zone).
pub fn zone_change_record<E: FieldEnv>(env: &mut E, gd: u32, z: u32) {
    let a = env.callv(0x0201_27A4, &[gd]);
    env.callv(0x0201_2AC4, &[a, z & 0xFFFF]);
}

/// C: `FUN_021895D0` (unlabeled routine at `0x021895D0`): `FUN_021895E8(fm, z, heap)` (a tail call; through its stub,
/// see [`zone_change_10_at`]).
pub fn zone_change_10<E: FieldEnv>(env: &mut E, fm: u32, z: u32) {
    let heap = env.r16(fm) as u32;
    env.callv(0x0218_95E8, &[fm, z, heap]);
}

/// C: `FUN_021895DC` (unlabeled routine at `0x021895DC`): `+0x10C = 0`.
pub fn clear_ctrl_work(mem: &mut Mem, fm: u32) {
    mem.w32(fm + 0x10C, 0);
}

/// C: `FUN_021895E8` (unlabeled routine at `0x021895E8`): the zone's map texture set `w = FUN_02013BD0(z)`: load it
/// (`FUN_0218F14C(+0x10, w, heap)`) or, for 0xFFFF, clear (`FUN_0218F134(+0x10)`).
///
/// UNVERIFIED port of `FUN_021895E8`: not in [`PORTED`]. The spec's stub `0218f134 args=v,p,p,v` logs r3, which the
/// one-argument callee does not read; r3 is this function's entry r3 (the caller's), so the hash depends on
/// whoever called it. Callers reach it through its stub.
pub fn zone_change_10_at<E: FieldEnv>(env: &mut E, fm: u32, z: u32, heap: u32) {
    let w = env.callv(0x0201_3BD0, &[z & 0xFFFF]);
    let h = env.r32(fm + 0x10);
    if w != 0xFFFF {
        env.callv(0x0218_F14C, &[h, w, heap]);
    } else {
        env.call(
            0x0218_F134,
            &mut [CArg::V(h), CArg::P(w), CArg::P(heap), CArg::V(0)],
        );
    }
}

/// C: `FUN_02189614` (unlabeled routine at `0x02189614`): in zones where `FUN_02013DD0(zone)` holds, set up the map
/// draw's weather / light object (`FUN_0218DC74(+0x48, x, k)` with `x = FUN_020127D0(gd, k ? 0 : 1)`, `k =
/// FUN_0201444C(..)`) and `FUN_021939C0` / `FUN_02193ABC` / `FUN_02193AFC` on `FUN_0218DC70(+0x48)`.
pub fn zone_light<E: FieldEnv>(env: &mut E, gd: u32, fm: u32) {
    let s = env.callv(0x0201_2B38, &[gd]);
    let z = env.r16(fm + 0xD0) as u32;
    if env.callv(0x0201_3DD0, &[z]) == 0 {
        return;
    }
    let k = env.callv(0x0201_444C, &[s]);
    let gd8 = env.r32(fm + 8);
    // C: `FUN_020127C4(gd)` is the thunk `FUN_020127D0(gd, 1)`
    let x = if k != 0 {
        env.callv(0x0201_27D0, &[gd8, 0])
    } else {
        env.callv(0x0201_27C4, &[gd8])
    };
    let d = env.r32(fm + 0x48);
    env.callv(0x0218_DC74, &[d, x, k]);
    let d = env.r32(fm + 0x48);
    let e = env.callv(0x0218_DC70, &[d]);
    let zev = env.callv(0x0201_2844, &[gd]);
    let heap = env.r16(fm) as u32;
    env.callv(0x0219_39C0, &[e, zev, k, heap]);
    let (heap, h) = (env.r16(fm) as u32, env.r32(fm + 0x10));
    env.callv(0x0219_3ABC, &[e, h, heap]);
    let gd8 = env.r32(fm + 8);
    let mm = env.callv(0x0201_2964, &[gd8]);
    env.callv(0x0219_3AFC, &[e, mm]);
}

/// C: `FUN_02189684` (unlabeled routine at `0x02189684`): the zone's scene-area set `w = FUN_02013FA8(z)`: load it
/// (`FUN_021AB650(+0x40)`, `FUN_021AB868(+0x44)`, `FUN_021AB840(+0x44, 1, w, 0x96)`, `FUN_021AB634(+0x40, ..)` with
/// the three values of `FUN_021AB880` / `884` / `888`); none: unless `FUN_02013C80(z)`, just the two resets.
pub fn zone_change_scene<E: FieldEnv>(env: &mut E, fm: u32, z: u32) {
    let w = env.callv(0x0201_3FA8, &[z & 0xFFFF]);
    if w != 0xFFFF {
        let a = env.r32(fm + 0x40);
        env.callv(0x021A_B650, &[a]);
        let b = env.r32(fm + 0x44);
        env.callv(0x021A_B868, &[b]);
        let b = env.r32(fm + 0x44);
        env.callv(0x021A_B840, &[b, 1, w, 0x96]);
        let b = env.r32(fm + 0x44);
        let x = env.callv(0x021A_B880, &[b]);
        let b = env.r32(fm + 0x44);
        let y = env.callv(0x021A_B884, &[b]);
        let b = env.r32(fm + 0x44);
        // the spec's stub lists no arguments
        let zz = env.call(0x021A_B888, &mut [CArg::X(b)]);
        let a = env.r32(fm + 0x40);
        env.callv(0x021A_B634, &[a, x, y, zz]);
        return;
    }
    if env.callv(0x0201_3C80, &[z & 0xFFFF]) == 0 {
        let a = env.r32(fm + 0x40);
        env.callv(0x021A_B650, &[a]);
        let b = env.r32(fm + 0x44);
        env.callv(0x021A_B868, &[b]);
    }
}

/// C: `FUN_021896F4` (unlabeled routine at `0x021896F4`): the comm mode of `x`: `k = FUN_02026454(x)`; 0: `FUN_0202647C(x)`
/// when `FUN_0200FB24()` is set (else 0); 1, 2, 5: `k`; 3, 4 and above 5: 2.
pub fn comm_mode<E: FieldEnv>(env: &mut E, x: u32) -> u32 {
    let k = env.callv(0x0202_6454, &[x]);
    match k {
        0 => {
            if env.callv(0x0200_FB24, &[]) != 0 {
                env.callv(0x0202_647C, &[x])
            } else {
                0
            }
        }
        1 | 2 | 5 => k,
        _ => 2,
    }
}

/// C: `FUN_02189734` (unlabeled routine at `0x02189734`): 1 when `FUN_02026454(x) == 0`; else `FUN_02026400(x)` unless
/// `FUN_02026460(x)`, and 0.
pub fn comm_idle<E: FieldEnv>(env: &mut E, x: u32) -> u32 {
    if env.callv(0x0202_6454, &[x]) == 0 {
        return 1;
    }
    if env.callv(0x0202_6460, &[x]) == 0 {
        // the spec's stub lists no arguments
        env.call(0x0202_6400, &mut [CArg::X(x)]);
    }
    0
}

/// C: `FUN_02189758` (unlabeled routine at `0x02189758`): clear a SLOT (`id = -1`, block 0).
pub fn slot_init(mem: &mut Mem, slot: u32) {
    mem.w32(slot, u32::MAX);
    mem.w32(slot + 4, 0);
}

/// C: `FUN_02189764` (unlabeled routine at `0x02189764`): take the SLOT for `id` with a new block of `size` bytes (line
/// 0xD7C); 0 when `id` is -1 or the slot is in use.
pub fn slot_alloc<E: FieldEnv>(env: &mut E, fm: u32, id: u32, heap: u32, size: u32) -> u32 {
    let slot = fm + SLOT_OFS;
    if id == u32::MAX || env.r32(slot) != u32::MAX {
        return 0;
    }
    env.w32(slot, id);
    let b = env.callv(HEAP_ALLOC, &[heap, size, 1, FILE_FIELDMAP, 0xD7C]);
    env.w32(slot + 4, b);
    b
}

/// C: `FUN_021897A0` (unlabeled routine at `0x021897A0`): release the SLOT (free its block, `id = -1`) unless `id` is -1
/// (the id is not compared with the slot's).
pub fn slot_free<E: FieldEnv>(env: &mut E, fm: u32, id: u32) {
    let slot = fm + SLOT_OFS;
    if id != u32::MAX {
        let b = env.r32(slot + 4);
        env.callv(HEAP_FREE, &[b]);
        env.w32(slot, u32::MAX);
    }
}

/// C: `FUN_021897BC` (unlabeled routine at `0x021897BC`): the SLOT's block when it belongs to `id` (not -1), else 0.
pub fn slot_block(mem: &Mem, fm: u32, id: u32) -> u32 {
    let slot = fm + SLOT_OFS;
    if id == u32::MAX || id != mem.r32(slot) {
        return 0;
    }
    mem.r32(slot + 4)
}

getter!(
    /// C: `FUN_021899F0` (unlabeled routine at `0x021899F0`): the draw mode (`+0x12C`).
    draw_mode, 0x12C
);
setter!(
    /// C: `FUN_021899F8` (unlabeled routine at `0x021899F8`): `+0x12C = v`.
    set_draw_mode, 0x12C
);
getter!(
    /// C: `FUN_02189A00` (unlabeled routine at `0x02189A00`): `+0x124`.
    get_124, 0x124
);
getter!(
    /// C: `FUN_02189A08` (unlabeled routine at `0x02189A08`): `+0x13C`.
    get_13c, 0x13C
);
getter!(
    /// C: `FUN_02189A10` (unlabeled routine at `0x02189A10`): `+0xA0`.
    get_a0, 0xA0
);
getter!(
    /// C: `FUN_02189A18` (unlabeled routine at `0x02189A18`): `+0x40` (the scene area).
    get_40, 0x40
);
setter!(
    /// C: `FUN_02189A1C` (unlabeled routine at `0x02189A1C`): `+0x138 = v`.
    set_138, 0x138
);
getter!(
    /// C: `FUN_02189A24` (unlabeled routine at `0x02189A24`): `+0x138`.
    get_138, 0x138
);

/// C: `FUN_02189A2C` (unlabeled routine at `0x02189A2C`): `FUN_020120D8(gamesys)` (a tail call).
pub fn gamesys_020120d8<E: FieldEnv>(env: &mut E, fm: u32) -> u32 {
    let gs = env.r32(fm + 4);
    env.callv(0x0201_20D8, &[gs])
}

getter!(
    /// C: `FUN_02189A48` (unlabeled routine at `0x02189A48`): `+0x144`.
    get_144, 0x144
);
setter!(
    /// C: `FUN_02189A50` (unlabeled routine at `0x02189A50`): `+0x144 = v`.
    set_144, 0x144
);
setter!(
    /// C: `FUN_02189A58` (unlabeled routine at `0x02189A58`): `+0x140 = v`.
    set_140, 0x140
);

/// C: `FUN_02189A60` (unlabeled routine at `0x02189A60`): address of `+0x148`.
pub fn addr_148(fm: u32) -> u32 {
    fm.wrapping_add(0x148)
}

setter!(
    /// C: `FUN_02189A68` (unlabeled routine at `0x02189A68`): `+0x150 = v`.
    set_150, 0x150
);
getter!(
    /// C: `FUN_02189A70` (unlabeled routine at `0x02189A70`): `+0x150`.
    get_150, 0x150
);
getter!(
    /// C: `FUN_02189A78` (unlabeled routine at `0x02189A78`): `+0x100` (the camera distance of [`camera_distance`]).
    get_100, 0x100
);

/// C: `FUN_02189A38` (unlabeled routine at `0x02189A38`): `FUN_020113D8(FUN_02012984(gd))`.
///
/// The spec's stub `020113d8 args=v,v,v,v` logs r1..r3, which the one-argument callee does not read; they are this
/// function's entry registers, 0 in every run of the harness (nothing in the function writes them), so the port
/// passes zeros there. A real implementation ignores them.
pub fn gamedata_020113d8<E: FieldEnv>(env: &mut E, fm: u32) -> u32 {
    let gd = env.r32(fm + 8);
    let x = env.callv(0x0201_2984, &[gd]);
    env.callv(0x0201_13D8, &[x, 0, 0, 0])
}

/// `FX_SinCosTable_` entry `i` (ROM `0x020A1B38`, 4096 `(sin, cos)` pairs of fx16): `round(4096 * cos(2 pi i / 4096))`.
/// Computed instead of embedded; F2a checked it against all 4096 ROM entries (the nearest tie is 5e-4 away, far
/// beyond `f64` error).
pub fn fx_cos(i: u32) -> i16 {
    (((i as f64) * std::f64::consts::TAU / 4096.0).cos() * 4096.0).round() as i16
}

/// C: `FUN_02189A80` (unlabeled routine at `0x02189A80`): the camera distance term from the camera angle `a` (`u16
/// *FUN_0218ED3C(+0x10)`): `max(((|cos(a)| << 14) + 0x800) >> 12 - 0x4000, -0x2000)` (fx32; the shift in 64 bits).
pub fn camera_distance<E: FieldEnv>(env: &mut E, fm: u32) -> u32 {
    let h = env.r32(fm + 0x10);
    let p = env.callv(0x0218_ED3C, &[h]);
    let a = env.r16(p) as u32;
    let c = (fx_cos(a >> 4) as i32).unsigned_abs() as i64;
    let d = (((c << 14) + 0x800) >> 12) as i32 - 0x4000;
    d.max(-0x2000) as u32
}

/// C: `FUN_02189ACC` (unlabeled routine at `0x02189ACC`): 494 when `FUN_02013BA8(zone) == 1`, else 310.
pub fn zone_494<E: FieldEnv>(env: &mut E, zone: u32) -> u32 {
    if env.callv(0x0201_3BA8, &[zone]) == 1 {
        494
    } else {
        310
    }
}

/// C: `FUN_02189AE8` (unlabeled routine at `0x02189AE8`): 0 for zone 0x3F or when `FUN_02013CE0(zone) == 1`, else 1.
pub fn zone_not_3f<E: FieldEnv>(env: &mut E, zone: u32) -> u32 {
    if zone == 0x3F || env.callv(0x0201_3CE0, &[zone]) == 1 {
        0
    } else {
        1
    }
}

/// C: `FUN_02189B04` (unlabeled routine at `0x02189B04`): zone `z` is one of `0x3F`, `0x8C`, `0x8D`,
/// `0x8E` (the start step `0x021885E8` returns early for these zones while `0x02005C64()` is set;
/// meaning of the zone group UNVERIFIED).
pub fn is_hold_zone(z: u32) -> bool {
    matches!(z, 0x3F | 0x8C | 0x8D | 0x8E)
}

/// C: `FUN_02189B30` (unlabeled routine at `0x02189B30`): fill the MAP SETUP record `out` (0x3C bytes) of zone `z`: the
/// table record of its kind, then (when the table says so) the matrix size and land table of `mtx`
/// (`FUN_02159BF4` / `BF8` / `BFC` / `C20`, map_matrix), `+0x20 = 1`, `+0x24 = 0xE`, and three fields of the area
/// data (`FUN_020140BC` / `C0` / `CC`).
pub fn map_setup<E: FieldEnv>(env: &mut E, z: u32, area: u32, out: u32, mtx: u32) {
    let k = env.callv(ZONE_KIND, &[z]);
    for i in 0..15 {
        env.w32(out + 4 * i, setup_word(k, 4 * i));
    }
    if setup_word(k, 0x40) != 0 {
        let w = env.callv(0x0215_9BF4, &[mtx]);
        env.w16(out + 0x14, w as u16);
        let h = env.callv(0x0215_9BF8, &[mtx]);
        env.w16(out + 0x16, h as u16);
        let n = env.callv(0x0215_9BFC, &[mtx]);
        env.w32(out + 0x18, n);
        let l = env.callv(0x0215_9C20, &[mtx]);
        env.w32(out + 0x1C, l);
    }
    env.w32(out + 0x20, 1);
    env.w32(out + 0x24, 0xE);
    let v = env.callv(0x0201_40BC, &[area]);
    env.w32(out + 0x28, v);
    let v = env.callv(0x0201_40C0, &[area]);
    env.w32(out + 0x2C, v);
    let v = env.callv(0x0201_40CC, &[area]);
    env.w32(out + 0x30, v);
}

/// C: `FUN_02189BA8` (unlabeled routine at `0x02189BA8`): the MAP CTRL of zone `z` (MAP SETUP `+0x3C`).
pub fn map_ctrl_of<E: FieldEnv>(env: &mut E, z: u32) -> u32 {
    let k = env.callv(ZONE_KIND, &[z]);
    setup_word(k, 0x3C)
}

/// C: `FUN_02189BBC` (unlabeled routine at `0x02189BBC`): the heap size of zone `z` (MAP SETUP `+0x44`).
pub fn heap_size_of<E: FieldEnv>(env: &mut E, z: u32) -> u32 {
    let k = env.callv(ZONE_KIND, &[z]);
    setup_word(k, 0x44)
}

#[cfg(test)]
mod tests;
