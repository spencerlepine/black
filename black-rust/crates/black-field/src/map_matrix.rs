//! `map_matrix.c` + `map_replace.c` (overlay 10, `0x02159B48..0x0215A07A`): the map matrix (the grid of
//! map blocks of a zone group, ARC 9), the matrix replacements of ARC 10 and the fly-point table after them.
//!
//! F2a-spec specced all 33 functions (`tools/rom-diff/specs/black-field/map_matrix.spec`, whose header
//! documents the MAP MATRIX (0x152C bytes) and MAP REPLACE (0x2C bytes) layouts); F2a ported them. Records
//! are read through the [`Mem`] window of the environment at their original addresses; heap, archive, event
//! work and LOCATION callees go through [`FieldEnv::call`]. The two ROM tables the code reads
//! (`0x02170FB0`, `0x02170FEC`) are embedded.
//!
//! MAP MATRIX: `+0x08 u16` width, `+0x0A u16` height (blocks), `+0x10 u32` cell count, `+0x14 u16 zone[900]`,
//! `+0x71C u32 land[900]` (cell = `x + y * width`). See the crate README for the full table.

use crate::hooks::{CArg, FieldEnv, Mem, HEAP_ALLOC, HEAP_FREE};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0215_9B48,
    0x0215_9B94,
    0x0215_9BC4,
    0x0215_9BD4,
    0x0215_9BF4,
    0x0215_9BF8,
    0x0215_9BFC,
    0x0215_9C00,
    0x0215_9C20,
    0x0215_9C2C,
    0x0215_9C48,
    0x0215_9C68,
    0x0215_9CC4,
    0x0215_9D04,
    0x0215_9D68,
    0x0215_9D84,
    0x0215_9DC8,
    0x0215_9DCC,
    0x0215_9DEC,
    0x0215_9E00,
    0x0215_9E60,
    0x0215_9E8C,
    0x0215_9F04,
    0x0215_9F28,
    0x0215_9F54,
    0x0215_9F80,
    0x0215_9F84,
    0x0215_9F94,
    0x0215_9FA8,
    0x0215_9FE0,
    0x0215_A014,
    0x0215_A048,
    0x0215_A064,
];

/// Offset of the zone table (`u16` per cell).
pub const ZONE_OFS: u32 = 0x14;
/// Offset of the land (map container) table (`u32` per cell).
pub const LAND_OFS: u32 = 0x71C;
/// Size of the MAP MATRIX record.
pub const MATRIX_SIZE: u32 = 0x152C;
/// Most cells a matrix holds (30 x 30).
pub const MAX_CELLS: u32 = 900;
/// MAP REPLACE: offset of the 13-byte STATE.
pub const REP_STATE: u32 = 0xC;
/// MAP REPLACE: offset of the current 16-byte replacement RECORD.
pub const REP_RECORD: u32 = 0x1A;

/// Address of the string `"map_matrix.c"`.
pub const FILE_MAP_MATRIX: u32 = 0x0217_28AC;
/// Address of the string `"map_replace.c"`.
pub const FILE_MAP_REPLACE: u32 = 0x0217_28BC;

/// Address of the ROM table of the event-var switches (10 entries of 6 bytes).
pub const VAR_TABLE: u32 = 0x0217_0FB0;
/// ROM `0x02170FB0`: 10 x `{u8 selector key, u8 key, u16 event var, u16 value}` (the 0x4031..0x403A event vars
/// that switch matrix replacements; `FUN_02159E60` matches byte 0, `FUN_02159F04` byte 1).
pub const VAR_TABLE_DATA: [u8; 60] = [
    0x03, 0x00, 0x31, 0x40, 0x20, 0x12, 0x04, 0x01, 0x32, 0x40, 0x24, 0x02, 0x05, 0x02, 0x33, 0x40,
    0x06, 0x02, 0x06, 0x03, 0x34, 0x40, 0x09, 0x12, 0x07, 0x04, 0x35, 0x40, 0x16, 0x03, 0x08, 0x05,
    0x36, 0x40, 0x27, 0x12, 0x09, 0x06, 0x37, 0x40, 0x30, 0x08, 0x0A, 0x07, 0x38, 0x40, 0x08, 0x04,
    0x0B, 0x08, 0x39, 0x40, 0x20, 0x01, 0x0C, 0x09, 0x3A, 0x40, 0x28, 0x10,
];

/// Address of the ROM table of the fly points (17 entries of 0x14 bytes).
pub const FLY_TABLE: u32 = 0x0217_0FEC;
/// ROM `0x02170FEC`: 17 x 0x14-byte fly points: `+0 u16` flag id, `+2 u16` bits (bit 0: the zone at `+4` counts
/// for `FUN_02159FE0`, bit 1: the zone at `+8` sets the flag in `FUN_0215A014`), `+4 u16` zone, `+6 u8` x,
/// `+7 u8` y, `+8 u16` zone, then more fields this area does not read.
pub const FLY_TABLE_DATA: [u8; 340] = [
    0xB0, 0x09, 0x03, 0x00, 0x86, 0x01, 0x06, 0x08, 0x85, 0x01, 0x0E, 0x03, 0xEC, 0x02, 0x85, 0x01,
    0x0E, 0x03, 0xEC, 0x02, 0xB1, 0x09, 0x03, 0x00, 0x8E, 0x01, 0x07, 0x0C, 0x8D, 0x01, 0x17, 0x03,
    0x92, 0x02, 0x8D, 0x01, 0x17, 0x03, 0x92, 0x02, 0xB2, 0x09, 0x03, 0x00, 0x08, 0x00, 0x07, 0x0C,
    0x06, 0x00, 0x18, 0x04, 0x6B, 0x00, 0x06, 0x00, 0x18, 0x04, 0x6B, 0x00, 0xB3, 0x09, 0x01, 0x00,
    0x14, 0x00, 0x07, 0x0C, 0x10, 0x00, 0x1D, 0x05, 0x84, 0x00, 0x10, 0x00, 0x1D, 0x05, 0x84, 0x00,
    0xB4, 0x09, 0x03, 0x00, 0x29, 0x00, 0x07, 0x0C, 0x1C, 0x00, 0x8A, 0x05, 0xEB, 0x00, 0x1C, 0x00,
    0x8A, 0x05, 0xEB, 0x00, 0xB5, 0x09, 0x01, 0x00, 0x41, 0x00, 0x07, 0x0C, 0x3E, 0x00, 0x11, 0x05,
    0x27, 0x01, 0x3E, 0x00, 0x11, 0x05, 0x27, 0x01, 0xB6, 0x09, 0x01, 0x00, 0x63, 0x00, 0x07, 0x0C,
    0x60, 0x00, 0xCF, 0x04, 0xEE, 0x00, 0x60, 0x00, 0xCF, 0x04, 0xEE, 0x00, 0xB7, 0x09, 0x03, 0x00,
    0x6D, 0x00, 0x07, 0x0C, 0x6B, 0x00, 0xB9, 0x04, 0xB8, 0x01, 0x6B, 0x00, 0xB9, 0x04, 0xB8, 0x01,
    0xB8, 0x09, 0x03, 0x00, 0x73, 0x00, 0x07, 0x0C, 0x71, 0x00, 0xB9, 0x04, 0xB8, 0x01, 0x71, 0x00,
    0xB9, 0x04, 0xB8, 0x01, 0xB9, 0x09, 0x01, 0x00, 0x7A, 0x00, 0x07, 0x0C, 0x78, 0x00, 0xB9, 0x04,
    0xB8, 0x01, 0x78, 0x00, 0xB9, 0x04, 0xB8, 0x01, 0xBA, 0x09, 0x03, 0x00, 0x97, 0x01, 0x07, 0x0C,
    0x96, 0x01, 0x08, 0x04, 0x07, 0x01, 0x96, 0x01, 0x08, 0x04, 0x07, 0x01, 0xBB, 0x09, 0x03, 0x00,
    0x92, 0x00, 0x07, 0x0C, 0x88, 0x00, 0x11, 0x00, 0x11, 0x00, 0x88, 0x00, 0x11, 0x00, 0x11, 0x00,
    0xBD, 0x09, 0x03, 0x00, 0x9D, 0x01, 0x07, 0x0C, 0x9C, 0x01, 0x64, 0x00, 0x64, 0x00, 0x9C, 0x01,
    0x64, 0x00, 0x64, 0x00, 0xBE, 0x09, 0x02, 0x00, 0x93, 0x00, 0x8B, 0xEC, 0x93, 0x00, 0x8B, 0x01,
    0xEC, 0x02, 0x93, 0x00, 0x8B, 0x01, 0xEC, 0x02, 0xBC, 0x09, 0x03, 0x00, 0x01, 0x00, 0x07, 0x0C,
    0x00, 0x00, 0x64, 0x00, 0x64, 0x00, 0x00, 0x00, 0x64, 0x00, 0x64, 0x00, 0xBC, 0x09, 0x03, 0x00,
    0xA9, 0x01, 0x07, 0x0C, 0xA8, 0x01, 0x64, 0x00, 0x64, 0x00, 0xA8, 0x01, 0x64, 0x00, 0x64, 0x00,
    0xBF, 0x09, 0x02, 0x00, 0xD6, 0x00, 0x00, 0x00, 0xD6, 0x00, 0x00, 0x00, 0x00, 0x00, 0xD6, 0x00,
    0x00, 0x00, 0x00, 0x00,
];

/// `u16` of the ROM table at `addr` (`base`/`data` = one of the two embedded tables). Panics outside it.
fn rom_u16(base: u32, data: &[u8], addr: u32) -> u16 {
    let o = addr.wrapping_sub(base) as usize;
    u16::from_le_bytes([data[o], data[o + 1]])
}

fn var_u8(i: u32, ofs: u32) -> u32 {
    VAR_TABLE_DATA[(i * 6 + ofs) as usize] as u32
}

fn var_u16(i: u32, ofs: u32) -> u32 {
    rom_u16(VAR_TABLE, &VAR_TABLE_DATA, VAR_TABLE + i * 6 + ofs) as u32
}

fn fly_u16(i: u32, ofs: u32) -> u32 {
    rom_u16(
        FLY_TABLE,
        &FLY_TABLE_DATA,
        FLY_TABLE
            .wrapping_add(i.wrapping_mul(0x14))
            .wrapping_add(ofs),
    ) as u32
}

/// C: `FUN_02048F60(arc, member, heap, heap2)`: load an archive member into a new heap block (returns it).
const LOAD_MEMBER: u32 = 0x0204_8F60;
/// C: `FUN_020490F4(arc, heap)`: open an archive, returns its handle.
const ARC_OPEN: u32 = 0x0204_90F4;
/// C: `FUN_020493D4(handle, member)`: size of an archive member.
const ARC_SIZE: u32 = 0x0204_93D4;
/// C: `FUN_0204944C(handle, member, offset, size, dst)`: read part of an archive member.
const ARC_READ: u32 = 0x0204_944C;
/// C: `FUN_02049238(handle)`: close an archive.
const ARC_CLOSE: u32 = 0x0204_9238;
/// C: `FUN_020120F4(gamesys)`: the GAMEDATA of the game system.
const GAMEDATA: u32 = 0x0201_20F4;
/// C: `FUN_0201296C(gamedata)`: the event work (`game_data::event_work_ptr`).
const EVENT_WORK: u32 = 0x0201_296C;
/// C: `FUN_02012138(gamesys)`.
const SYS_02012138: u32 = 0x0201_2138;
/// C: `FUN_0202E83C(gamesys)`: the season (0..3).
const SEASON: u32 = 0x0202_E83C;
/// C: `FUN_0215843C(x)` (overlay 10).
const OV10_0215843C: u32 = 0x0215_843C;
/// C: `FUN_020143CC(ev, id)`: address of event var `id` (`event_work::var_addr`).
const VAR_ADDR: u32 = 0x0201_43CC;
/// C: `FUN_02014314(ev, id)`: set event flag `id` (`event_work::set_flag`).
const SET_FLAG: u32 = 0x0201_4314;
/// C: `FUN_02014184(loc, zone, kind, x, y, z)`: set a LOCATION (`zonedata`).
const SET_LOCATION: u32 = 0x0201_4184;

/// C: `FUN_02159B48` (unlabeled routine at `0x02159B48`): allocate a MAP MATRIX (0x152C bytes, line 0x4B), store
/// the heap id, fill the land table with `0xFFFFFFFF` and the zone table with the words `0x0000FFFF`
/// (`MIi_CpuFill32(0xFFFF, ..)`: every other zone halfword is 0); returns it.
pub fn create<E: FieldEnv>(env: &mut E, heap: u32) -> u32 {
    let m = env.call(
        HEAP_ALLOC,
        &mut [
            CArg::V(heap),
            CArg::V(MATRIX_SIZE),
            CArg::V(1),
            CArg::V(FILE_MAP_MATRIX),
            CArg::V(0x4B),
        ],
    );
    env.w16(m, heap as u16);
    for i in 0..0xE10 / 4 {
        env.w32(m + LAND_OFS + 4 * i, 0xFFFF_FFFF);
    }
    for i in 0..0x708 / 4 {
        env.w32(m + ZONE_OFS + 4 * i, 0xFFFF);
    }
    m
}

/// C: `FUN_02159B94` (unlabeled routine at `0x02159B94`): load matrix `id` of ARC 9 into `m` ([`load_from`]
/// with the default zone `zone`) and free the file.
pub fn load<E: FieldEnv>(env: &mut E, m: u32, id: u32, zone: u32, heap: u32) {
    let data = env.call(
        LOAD_MEMBER,
        &mut [CArg::V(9), CArg::V(id), CArg::V(heap), CArg::V(heap)],
    );
    load_from(env, m, data, id, zone);
    env.call(HEAP_FREE, &mut [CArg::V(data)]);
}

/// C: `FUN_02159BC4` (unlabeled routine at `0x02159BC4`): zone id of block `(x, y)`: `u16` at
/// `m + 0x14 + 2 * (x + y * width)`. No bounds check (the arithmetic wraps like the original's).
pub fn zone_at(mem: &Mem, m: u32, x: u32, y: u32) -> u16 {
    let w = mem.r16(m + 8) as u32;
    let cell = x.wrapping_add(y.wrapping_mul(w));
    mem.r16(m.wrapping_add(ZONE_OFS).wrapping_add(cell.wrapping_mul(2)))
}

/// C: `FUN_02159BD4` (unlabeled routine at `0x02159BD4`): zone id at the world position `(px, pz)` (fx32).
pub fn zone_at_position(mem: &Mem, m: u32, px: i32, pz: i32) -> u16 {
    zone_at(mem, m, block_of(px) as u32, block_of(pz) as u32)
}

/// C: `FUN_02159BF4` (unlabeled routine at `0x02159BF4`): width in blocks.
pub fn width(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 8)
}

/// C: `FUN_02159BF8` (unlabeled routine at `0x02159BF8`): height in blocks.
pub fn height(mem: &Mem, m: u32) -> u16 {
    mem.r16(m + 0xA)
}

/// C: `FUN_02159BFC` (unlabeled routine at `0x02159BFC`): cell count.
pub fn cell_count(mem: &Mem, m: u32) -> u32 {
    mem.r32(m + 0x10)
}

/// C: `FUN_02159C00` (unlabeled routine at `0x02159C00`): set the size to `w` x `h` blocks when `w <= 30`,
/// `h <= 30` and `w * h <= 900` (unsigned, the product wrapping); otherwise nothing.
pub fn set_size(mem: &mut Mem, m: u32, w: u32, h: u32) {
    let n = w.wrapping_mul(h);
    if w <= 30 && h <= 30 && n <= MAX_CELLS {
        mem.w16(m + 8, w as u16);
        mem.w16(m + 0xA, h as u16);
        mem.w32(m + 0x10, n);
    }
}

/// C: `FUN_02159C20` (unlabeled routine at `0x02159C20`): address of the land table.
pub fn land_table(m: u32) -> u32 {
    m.wrapping_add(LAND_OFS)
}

/// C: `FUN_02159C2C` (unlabeled routine at `0x02159C2C`): block `(x, y)` is inside the matrix (signed compares).
pub fn contains(mem: &Mem, m: u32, x: i32, y: i32) -> bool {
    y >= 0 && y < mem.r16(m + 0xA) as i32 && x >= 0 && x < mem.r16(m + 8) as i32
}

/// C: `FUN_02159C48` (unlabeled routine at `0x02159C48`): the world position `(px, pz)` (fx32) is inside the
/// matrix.
pub fn contains_position(mem: &Mem, m: u32, px: i32, pz: i32) -> bool {
    contains(mem, m, block_of(px), block_of(pz))
}

/// C: `FUN_02159C68` (unlabeled routine at `0x02159C68`): apply the current replacement record of `rep` to `m`:
/// kind 2 ([`replacement`]) reloads the matrix `new` (default zone `u16 m+2`), kind 1 changes every land id
/// equal to `old` into `new`.
pub fn apply_replacement<E: FieldEnv>(env: &mut E, m: u32, rep: u32, heap: u32) {
    let (k, old, new) = replacement(env, rep);
    match k {
        2 => {
            let zone = env.r16(m + 2) as u32;
            load(env, m, new & 0xFFFF, zone, heap);
        }
        1 => {
            let mut i = 0u32;
            while i < env.r32(m + 0x10) {
                let at = m + LAND_OFS + 4 * i;
                if env.r32(at) == old {
                    env.w32(at, new);
                }
                i += 1;
            }
        }
        _ => {}
    }
}

/// C: `FUN_02159CC4` (unlabeled routine at `0x02159CC4`): apply every ARC 10 replacement record whose matrix id
/// is the matrix id of `m` (`u32 m+4`), in record order.
pub fn apply_replacements<E: FieldEnv>(env: &mut E, m: u32, gamesys: u32, heap: u32) {
    let rep = replace_create(env, heap, gamesys);
    let n = replace_count(env.mem(), rep) as i32;
    let mut i = 0i32;
    while i < n {
        let id = replace_read(env, rep, i as u32) as u32;
        if id == env.r32(m + 4) {
            apply_replacement(env, m, rep, heap);
        }
        i += 1;
    }
    replace_free(env, rep);
}

/// C: `FUN_02159D04` (unlabeled routine at `0x02159D04`): fill `m` from the matrix file `data`: id, default zone,
/// size, layout flag, the land table (`MIi_CpuCopy32` of `w * h` words); flag 1: the zone table is the second
/// plane of the file (`u32` per cell, stored as `u16`), else every zone halfword is `zone`.
pub fn load_from<E: FieldEnv>(env: &mut E, m: u32, data: u32, id: u32, zone: u32) {
    let w = env.r16(data + 4) as u32;
    let h = env.r16(data + 6) as u32;
    env.w32(m + 4, id);
    let n = w.wrapping_mul(h);
    env.w16(m + 0xA, h as u16);
    let flag = env.r16(data);
    env.w32(m + 0x10, n);
    env.w16(m + 0xC, flag);
    env.w16(m + 2, zone as u16);
    env.w16(m + 8, w as u16);
    let src = data + 8;
    for i in 0..(n.wrapping_mul(4) / 4) {
        let v = env.r32(src + 4 * i);
        env.w32(m + LAND_OFS + 4 * i, v);
    }
    if env.r16(m + 0xC) == 1 {
        let zones = src.wrapping_add(env.r32(m + 0x10).wrapping_mul(4));
        let mut i = 0u32;
        while i < env.r32(m + 0x10) {
            let v = env.r32(zones + 4 * i);
            env.w16(m + ZONE_OFS + 2 * i, v as u16);
            i += 1;
        }
    } else {
        for i in 0..0x708 / 2 {
            env.w16(m + ZONE_OFS + 2 * i, zone as u16);
        }
    }
}

/// C: `FUN_02159D68` (unlabeled routine at `0x02159D68`): fx32 world coordinate -> 32-tile block
/// index: `((v / 4096) / 16) / 32`, each a signed division rounding toward zero (the `asrs/lsrs/adds`
/// sign fix of the original, so `-1` gives block 0).
pub fn block_of(v: i32) -> i32 {
    ((v / 4096) / 16) / 32
}

/// C: `FUN_02159D84` (unlabeled routine at `0x02159D84`): create the MAP REPLACE object (0x2C bytes, line 0x77):
/// heap id, the ARC 10 handle, the record count (`u16 size of member 0 >> 4`) and the STATE ([`replace_state`]).
pub fn replace_create<E: FieldEnv>(env: &mut E, heap: u32, gamesys: u32) -> u32 {
    let r = env.call(
        HEAP_ALLOC,
        &mut [
            CArg::V(heap),
            CArg::V(0x2C),
            CArg::V(1),
            CArg::V(FILE_MAP_REPLACE),
            CArg::V(0x77),
        ],
    );
    env.w16(r, heap as u16);
    let h = env.call(ARC_OPEN, &mut [CArg::V(10), CArg::V(heap)]);
    env.w32(r + 4, h);
    let size = env.call(ARC_SIZE, &mut [CArg::V(h), CArg::V(0)]);
    env.w32(r + 8, (size & 0xFFFF) >> 4);
    replace_state(env, r + REP_STATE, gamesys);
    r
}

/// C: `FUN_02159DC8` (unlabeled routine at `0x02159DC8`): record count.
pub fn replace_count(mem: &Mem, rep: u32) -> u32 {
    mem.r32(rep + 8)
}

/// C: `FUN_02159DCC` (unlabeled routine at `0x02159DCC`): read record `i` (16 bytes) into `rep + 0x1A`; returns
/// its matrix id.
pub fn replace_read<E: FieldEnv>(env: &mut E, rep: u32, i: u32) -> u16 {
    let h = env.r32(rep + 4);
    env.call(
        ARC_READ,
        &mut [
            CArg::V(h),
            CArg::V(0),
            CArg::V(i << 4),
            CArg::V(0x10),
            CArg::Out(rep + REP_RECORD, 0x10),
        ],
    );
    env.r16(rep + REP_RECORD)
}

/// C: `FUN_02159DEC` (unlabeled routine at `0x02159DEC`): close the archive and free the object.
pub fn replace_free<E: FieldEnv>(env: &mut E, rep: u32) {
    let h = env.r32(rep + 4);
    env.call(ARC_CLOSE, &mut [CArg::V(h)]);
    env.call(HEAP_FREE, &mut [CArg::V(rep)]);
}

/// The body of `FUN_02159E00`: `(kind, old, new)` of the current record. The selector (record `+3`) picks a
/// STATE byte: 0 season, 1 `+1`, 2 `+2`, else `STATE[3 + k]` for the var-table entry `k` whose byte 0 is the
/// selector (`k = -1`: `+2`); that byte indexes `value[]` (`u16` at record `+4`). `old = value[0]`; kind 0 when
/// `old == new`, else 2 for record kind (`+2`) 1, 1 for kind 0, 0 otherwise.
fn replacement<E: FieldEnv>(env: &mut E, rep: u32) -> (u32, u32, u32) {
    let rec = rep + REP_RECORD;
    let st = rep + REP_STATE;
    let sel = env.r8(rec + 3) as u32;
    let old = env.r16(rec + 4) as u32;
    let s = match sel {
        0 => env.r8(st),
        1 => env.r8(st + 1),
        2 => env.r8(st + 2),
        _ => {
            let k = var_key_index(sel);
            env.r8(st.wrapping_add(k).wrapping_add(3))
        }
    } as u32;
    let new = env.r16(rec + (s << 1) + 4) as u32;
    if old == new {
        return (0, old, new);
    }
    let k = match env.r8(rec + 2) {
        1 => 2,
        0 => 1,
        _ => 0,
    };
    (k, old, new)
}

/// C: `FUN_02159E00` (unlabeled routine at `0x02159E00`): `*pold` / `*pnew` (`u32`) = the replaced and the new
/// value of the current record; returns its kind (0 nothing, 1 replace land ids, 2 reload a matrix).
pub fn replacement_at<E: FieldEnv>(env: &mut E, rep: u32, pold: u32, pnew: u32) -> u32 {
    let (k, old, new) = replacement(env, rep);
    env.w32(pold, old);
    env.w32(pnew, new);
    k
}

/// C: `FUN_02159E60` (unlabeled routine at `0x02159E60`): index of the first var-table entry whose byte 0 is
/// `key`, else -1.
pub fn var_key_index(key: u32) -> u32 {
    (0..10).find(|&k| var_u8(k, 0) == key).unwrap_or(u32::MAX)
}

/// C: `FUN_02159E8C` (unlabeled routine at `0x02159E8C`): fill the 13-byte STATE: `+0` season, `+1`
/// `FUN_0215843C(..) != 0x15`, `+2` season + 1 when `+1` is set (else 0), `+3 + k` event var `VAR(k) == VALUE(k)`
/// for the 10 var-table entries.
pub fn replace_state<E: FieldEnv>(env: &mut E, st: u32, gamesys: u32) {
    let gd = env.call(GAMEDATA, &mut [CArg::V(gamesys)]);
    let ev = env.call(EVENT_WORK, &mut [CArg::V(gd)]);
    let x = env.call(SYS_02012138, &mut [CArg::V(gamesys)]);
    let season = env.call(SEASON, &mut [CArg::V(gamesys)]);
    env.w8(st, season as u8);
    let on = env.call(OV10_0215843C, &mut [CArg::V(x)]) != 0x15;
    env.w8(st + 1, on as u8);
    let v = if env.r8(st + 1) != 0 { season + 1 } else { 0 };
    env.w8(st + 2, v as u8);
    for k in 0..10 {
        let p = env.call(VAR_ADDR, &mut [CArg::V(ev), CArg::V(var_u16(k, 2))]);
        let eq = env.r16(p) as u32 == var_u16(k, 4);
        env.w8(st + 3 + k, eq as u8);
    }
}

/// C: `FUN_02159F04` (unlabeled routine at `0x02159F04`): address of the first var-table entry whose byte 1 is
/// `key`, else 0 (the first argument is not read).
pub fn var_entry(_x: u32, key: u32) -> u32 {
    (0..10)
        .find(|&k| var_u8(k, 1) == key)
        .map_or(0, |k| VAR_TABLE + 6 * k)
}

/// C: `FUN_02159F28` (unlabeled routine at `0x02159F28`): set (`on`: the entry's value, else 0) the event var
/// of the var-table entry with key `key`, if there is one.
pub fn var_switch_set<E: FieldEnv>(env: &mut E, gd: u32, key: u32, on: u32) {
    let e = var_entry(gd, key);
    if e != 0 {
        let ev = env.call(EVENT_WORK, &mut [CArg::V(gd)]);
        let id = rom_u16(VAR_TABLE, &VAR_TABLE_DATA, e + 2) as u32;
        let p = env.call(VAR_ADDR, &mut [CArg::V(ev), CArg::V(id)]);
        let v = if on != 0 {
            rom_u16(VAR_TABLE, &VAR_TABLE_DATA, e + 4)
        } else {
            0
        };
        env.w16(p, v);
    }
}

/// C: `FUN_02159F54` (unlabeled routine at `0x02159F54`): the event var of the var-table entry with key `key`
/// holds the entry's value (false when there is no entry).
pub fn var_switch_on<E: FieldEnv>(env: &mut E, gd: u32, key: u32) -> bool {
    let e = var_entry(gd, key);
    if e == 0 {
        return false;
    }
    let ev = env.call(EVENT_WORK, &mut [CArg::V(gd)]);
    let id = rom_u16(VAR_TABLE, &VAR_TABLE_DATA, e + 2) as u32;
    let p = env.call(VAR_ADDR, &mut [CArg::V(ev), CArg::V(id)]);
    rom_u16(VAR_TABLE, &VAR_TABLE_DATA, e + 4) == env.r16(p)
}

/// C: `FUN_02159F80` (unlabeled routine at `0x02159F80`): 1 (the fallback fly point).
pub fn fly_default() -> u32 {
    1
}

/// C: `FUN_02159F84` (unlabeled routine at `0x02159F84`): `1 <= i <= 17` (a fly point number).
pub fn fly_valid(i: u32) -> bool {
    (i as i32) > 0 && i <= 17
}

/// C: `FUN_02159F94` (unlabeled routine at `0x02159F94`): the zone (`+8`) of fly point `i`.
pub fn fly_zone(i: u32) -> u16 {
    fly_u16(fly_index(i), 8) as u16
}

/// C: `FUN_02159FA8` (unlabeled routine at `0x02159FA8`): set the LOCATION `loc` to fly point `i` (zone `+4`,
/// block `x` / `y` = bytes `+6` / `+7`).
pub fn fly_location<E: FieldEnv>(env: &mut E, i: u32, loc: u32) {
    let k = fly_index(i);
    let xy = fly_u16(k, 6);
    let zone = fly_u16(k, 4);
    set_location(env, loc, zone, 0, xy & 0xFF, (xy >> 8) & 0xFF);
}

/// C: `FUN_02159FE0` (unlabeled routine at `0x02159FE0`): fly point number (1-based) of the first entry whose zone
/// `+4` is `zone` and whose bit 0 is set, else 0.
pub fn fly_point_of_zone(zone: u32) -> u32 {
    (0..17)
        .find(|&k| fly_u16(k, 4) == zone && fly_u16(k, 2) & 1 != 0)
        .map_or(0, |k| k + 1)
}

/// C: `FUN_0215A014` (unlabeled routine at `0x0215A014`): set the flag (`+0`) of the first entry whose zone `+8`
/// is `zone` and whose bit 1 is set (none: nothing).
pub fn fly_arrive<E: FieldEnv>(env: &mut E, gd: u32, zone: u32) {
    if let Some(k) = (0..17).find(|&k| fly_u16(k, 8) == zone && fly_u16(k, 2) & 2 != 0) {
        let ev = env.call(EVENT_WORK, &mut [CArg::V(gd)]);
        env.call(SET_FLAG, &mut [CArg::V(ev), CArg::V(fly_u16(k, 0))]);
    }
}

/// C: `FUN_0215A048` (unlabeled routine at `0x0215A048`): `FUN_02014184(loc, zone, 1, x << 16, 0, (u16) y << 16)`
/// (the third argument is not read).
pub fn set_location<E: FieldEnv>(env: &mut E, loc: u32, zone: u32, _unused: u32, x: u32, y: u32) {
    env.call(
        SET_LOCATION,
        &mut [
            CArg::V(loc),
            CArg::V(zone),
            CArg::V(1),
            CArg::V(x << 16),
            CArg::V(0),
            CArg::V((y & 0xFFFF) << 16),
        ],
    );
}

/// C: `FUN_0215A064` (unlabeled routine at `0x0215A064`): 0-based table index of fly point `i` (an invalid
/// number gives the default point's).
pub fn fly_index(i: u32) -> u32 {
    let i = if fly_valid(i) { i } else { fly_default() };
    i.wrapping_sub(1)
}

#[cfg(test)]
mod tests;
