//! `box_savedata.c` (S2): the PC box data model, main ARM9 `0x02007454..0x02007AF4` (33 functions).
//! Verified per function with `tools/check-fn.sh black-storage box_data` (spec
//! `tools/rom-diff/specs/black-storage/box_data.spec`).
//!
//! # Layout
//!
//! The box manager object is 4 bytes (`FUN_02007AA4` allocates it under `"box_savedata.c"` line
//! `0x3B9`): a pointer to the save-data object. `FUN_020074CC(mgr)` and `FUN_02007AF4(mgr, i)` fetch
//! two kinds of save blocks through `FUN_020071CC(savedata, block_id)`:
//!
//! * block 0, the **meta block** ([`META_SIZE`] = `0x3E0` bytes):
//!   * `+0x000` u32 current box (0..23),
//!   * `+0x004 + i * 0x28` the name of box `i`: 20 UTF-16 code units (0x28 bytes), 24 boxes
//!     (default names are message-bank 2/9 strings 6..29, copied by `FUN_02007464`),
//!   * `+0x3C4 + i` u8 wallpaper of box `i` (default `i % 16`),
//!   * `+0x3DC` u8 flags (bit masks; `FUN_02007A68` sets, `FUN_02007A80` tests),
//!   * `+0x3DD` u8 number of unlocked boxes: starts at 8, `FUN_020076B4` adds 8 up to the
//!     maximum of 24 ([`MAX_BOXES`]): the game has 24 boxes, 8 usable at first.
//! * blocks 1..=24, one **box** each ([`TRAY_SIZE`] = `0xFF0` bytes): [`SLOTS`] = 30 slots of
//!   [`MON_SIZE`] = `0x88` bytes (the encrypted BoxPokemon of `black-pokemon` poke_tool, P1). A slot
//!   is empty when its species (`PokeParaGet` id 5) is 0; the counting functions test id `0xA9`.
//!
//! A box index of `0xFFFF_FFFF` means "the current box" in most functions. Functions assert
//! (`FUN_02034714`, never returns) on a box index `>= 24` (or `>=` the unlocked count when putting)
//! and a slot `>= 30`.
//!
//! The port takes memory by address: [`BoxEnv::mem`] gives the bytes from an address to the end of
//! the block it points into, so the ROM's pointer arithmetic and stub calls are reproduced exactly.

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0200_7454,
    0x0200_745C,
    0x0200_7464,
    0x0200_74CC,
    0x0200_74D8,
    0x0200_752C,
    0x0200_75B0,
    0x0200_7610,
    0x0200_7664,
    0x0200_7698,
    0x0200_76A4,
    0x0200_76B4,
    0x0200_76D0,
    0x0200_7774,
    0x0200_77B8,
    0x0200_780C,
    0x0200_781C,
    0x0200_7838,
    0x0200_785C,
    0x0200_7898,
    0x0200_78C4,
    0x0200_791C,
    0x0200_7984,
    0x0200_79A0,
    0x0200_79BC,
    0x0200_7A18,
    0x0200_7A68,
    0x0200_7A80,
    0x0200_7AA4,
    0x0200_7AC8,
    0x0200_7AD0,
    0x0200_7AEC,
    0x0200_7AF4,
];

/// Size of the meta block (`FUN_0200745C`).
pub const META_SIZE: usize = 0x3E0;
/// Size of one box block (`FUN_02007AEC`): `SLOTS * MON_SIZE`.
pub const TRAY_SIZE: usize = 0xFF0;
/// Slots per box.
pub const SLOTS: usize = 30;
/// Size of one stored Pokemon.
pub const MON_SIZE: usize = 0x88;
/// Number of boxes (the maximum of the unlocked count).
pub const MAX_BOXES: u32 = 24;
/// Meta offset of the first box name; each name is [`NAME_SIZE`] bytes.
pub const NAMES_OFS: usize = 4;
/// Size of one box name (20 UTF-16 units).
pub const NAME_SIZE: usize = 0x28;
/// Meta offset of the wallpaper bytes (one per box).
pub const WALLPAPER_OFS: usize = 0x3C4;
/// Meta offset of the flags byte.
pub const FLAGS_OFS: usize = 0x3DC;
/// Meta offset of the unlocked-box count.
pub const COUNT_OFS: usize = 0x3DD;
/// "Current box" in the box-index arguments.
pub const CURRENT: u32 = 0xFFFF_FFFF;
/// C: the `"box_savedata.c"` allocation name, as the ROM passes its address.
pub const NAME_BOX_SAVEDATA: u32 = 0x020A_706C;

/// Undecompiled routines the box code calls, and the other functions of this area (their stubs).
/// Memory is by address: [`BoxEnv::mem`] returns the bytes from `addr` to the end of its block.
pub trait BoxEnv {
    /// C: `FUN_020074CC(mgr)`: the address of the meta block.
    fn meta_addr(&mut self, mgr: u32) -> u32;
    /// C: `FUN_02007AF4(mgr, tray)`: the address of the box block of `tray`.
    fn tray_addr(&mut self, mgr: u32, tray: u32) -> u32;
    /// The bytes from `addr` to the end of the block it points into (not a call).
    fn mem(&mut self, addr: u32) -> &mut [u8];
    /// C: `FUN_020071CC(savedata, block)`: the address of a save block.
    fn save_block(&mut self, savedata: u32, block: u32) -> u32;
    /// C: `FUN_02017E88(mon, id, buf)`: PokeParaGet, returns the value.
    fn poke_get(&mut self, mon: u32, id: u32, buf: u32) -> u32;
    /// C: `FUN_02017550(mon)`: clear one stored Pokemon.
    fn clear_mon(&mut self, mon: u32);
    /// C: `FUN_020181A0(mon)`: post-copy fixup of a stored Pokemon.
    fn fixup_mon(&mut self, mon: u32);
    /// C: `FUN_02007464(meta)`: the meta initialiser.
    fn meta_init(&mut self, meta: u32);
    /// C: `FUN_0200752C(mgr, tray, mon)`: put `mon` in the first free slot of `tray`; 1 or 0.
    fn put_tray(&mut self, mgr: u32, tray: u32, mon: u32) -> u32;
    /// C: `FUN_020078C4(mgr, tray)`: occupied slots of a box.
    fn occupied_count(&mut self, mgr: u32, tray: u32) -> u32;
    /// C: `FUN_0200791C(mgr, tray)`: occupied, non-egg slots of a box.
    fn non_egg_count(&mut self, mgr: u32, tray: u32) -> u32;
    /// C: `FUN_02045924(dst, name)`: copy a box name (0x28 bytes) out to the string `dst`.
    fn name_to_str(&mut self, dst: u32, name: &[u8]);
    /// C: `FUN_02045A38(src, name, n)`: fill a box name (0x28 bytes) from the string `src`.
    fn str_to_name(&mut self, src: u32, name: &mut [u8], n: u32);
    /// C: `FUN_02045F5C(msg, idx, name, n)`: message string `idx` into a box name.
    fn msg_string(&mut self, msg: u32, idx: u32, name: &mut [u8], n: u32);
    /// C: `FUN_02045B38(0, 2, 9, 1)`: open the message bank (0 when missing).
    fn msg_bank(&mut self) -> u32;
    /// C: `FUN_02045C04(msg)`: close the message bank.
    fn msg_free(&mut self, msg: u32);
    /// C: `FUN_02030734(a0, a1, a2, a3, a4)`: heap allocation, the five C arguments in call order.
    fn alloc(&mut self, args: [u32; 5]) -> u32;
    /// C: `FUN_020307B0(ptr)`: heap free.
    fn free(&mut self, ptr: u32);
    /// C: `FUN_02034714(...)`: the assert, never returns.
    fn assert_fail(&mut self) -> !;

    /// C: `FUN_020074CC(mgr)` as bytes: the meta block.
    fn meta(&mut self, mgr: u32) -> &mut [u8] {
        let a = self.meta_addr(mgr);
        self.mem(a)
    }
}

fn rd32(p: &[u8]) -> u32 {
    u32::from_le_bytes(p[..4].try_into().unwrap())
}

fn wr32(p: &mut [u8], v: u32) {
    p[..4].copy_from_slice(&v.to_le_bytes());
}

/// `base + slot * MON_SIZE` (the ROM's `muls`/`adds`, wrapping).
fn slot_addr(base: u32, slot: u32) -> u32 {
    base.wrapping_add(slot.wrapping_mul(MON_SIZE as u32))
}

/// Copy one stored Pokemon from `from` to `to`.
fn copy_mon<E: BoxEnv>(env: &mut E, from: u32, to: u32) {
    let src: [u8; MON_SIZE] = env.mem(from)[..MON_SIZE].try_into().unwrap();
    env.mem(to)[..MON_SIZE].copy_from_slice(&src);
}

/// The box index with `CURRENT` replaced by the meta block's current box (no check).
fn current_or<E: BoxEnv>(env: &mut E, meta: u32, tray: u32) -> u32 {
    if tray == CURRENT {
        rd32(env.mem(meta))
    } else {
        tray
    }
}

/// C: `FUN_0200745C`: the size of the meta block.
pub fn meta_size() -> u32 {
    META_SIZE as u32
}

/// C: `FUN_02007AEC`: the size of one box block.
pub fn tray_size() -> u32 {
    TRAY_SIZE as u32
}

/// C: `FUN_02007454`: tail call of the meta initialiser.
pub fn reset_meta<E: BoxEnv>(env: &mut E, meta: u32) {
    env.meta_init(meta);
}

/// C: `FUN_02007464`: a fresh meta block: wallpapers `i % 16`, no flags, the default names
/// from the message bank (when it opens), the current box 0 and 8 unlocked boxes.
pub fn init_meta<E: BoxEnv>(env: &mut E, meta: u32) {
    for i in 0..MAX_BOXES as usize {
        env.mem(meta)[WALLPAPER_OFS + i] = (i % 16) as u8;
    }
    env.mem(meta)[FLAGS_OFS] = 0;
    let msg = env.msg_bank();
    if msg != 0 {
        for i in 0..MAX_BOXES {
            let mut name = [0u8; NAME_SIZE];
            env.msg_string(msg, i + 6, &mut name, 0x14);
            let ofs = NAMES_OFS + i as usize * NAME_SIZE;
            env.mem(meta)[ofs..ofs + NAME_SIZE].copy_from_slice(&name);
        }
        env.msg_free(msg);
    }
    wr32(env.mem(meta), 0);
    env.mem(meta)[COUNT_OFS] = 8;
}

/// C: `FUN_020074CC`: the meta block (block 0 of the save data).
pub fn meta_block<E: BoxEnv>(env: &mut E, mgr: u32) -> u32 {
    let savedata = rd32(env.mem(mgr));
    env.save_block(savedata, 0)
}

/// C: `FUN_020074D8`: put `mon` in the first box from the current one with room (trying
/// each unlocked box once); when all are full and the count is not 24, unlock 8 more and put it in
/// the first new box.
pub fn put_mon<E: BoxEnv>(env: &mut E, mgr: u32, mon: u32) -> bool {
    let meta = env.meta_addr(mgr);
    let cur = rd32(env.mem(meta));
    let mut tray = cur;
    loop {
        if env.put_tray(mgr, tray, mon) != 0 {
            return true;
        }
        tray += 1;
        let count = env.mem(meta)[COUNT_OFS] as u32;
        if tray >= count {
            tray = 0;
        }
        if tray == cur {
            break;
        }
    }
    let count = env.mem(meta)[COUNT_OFS];
    if count == MAX_BOXES as u8 {
        return false;
    }
    env.mem(meta)[COUNT_OFS] = count.wrapping_add(8);
    env.put_tray(mgr, count as u32, mon);
    true
}

/// C: `FUN_0200752C`: copy `mon` into the first slot of `tray` whose species is 0,
/// then fix it up; false when the box is full. An explicit `tray` must be below the unlocked count.
pub fn put_tray<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32, mon: u32) -> bool {
    let meta = env.meta_addr(mgr);
    let tray = if tray == CURRENT {
        rd32(env.mem(meta))
    } else {
        if tray >= env.mem(meta)[COUNT_OFS] as u32 {
            env.assert_fail();
        }
        tray
    };
    for slot in 0..SLOTS as u32 {
        let base = env.tray_addr(mgr, tray);
        let dst = slot_addr(base, slot);
        if env.poke_get(dst, 5, 0) == 0 {
            copy_mon(env, mon, dst);
            env.fixup_mon(dst);
            return true;
        }
    }
    false
}

/// C: `FUN_020075B0`: copy `mon` into `slot` of `tray` (both checked against
/// the unlocked count and `SLOTS`), then fix it up.
pub fn put_slot<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32, slot: u32, mon: u32) -> bool {
    let meta = env.meta_addr(mgr);
    let tray = current_or(env, meta, tray);
    if tray >= env.mem(meta)[COUNT_OFS] as u32 {
        env.assert_fail();
    }
    if slot >= SLOTS as u32 {
        env.assert_fail();
    }
    let base = env.tray_addr(mgr, tray);
    let dst = slot_addr(base, slot);
    copy_mon(env, mon, dst);
    env.fixup_mon(dst);
    true
}

/// C: `FUN_02007610`: swap two stored Pokemon (no checks).
pub fn swap_mons<E: BoxEnv>(
    env: &mut E,
    mgr: u32,
    tray_a: u32,
    slot_a: u32,
    tray_b: u32,
    slot_b: u32,
) {
    let base_a = env.tray_addr(mgr, tray_a);
    let base_b = env.tray_addr(mgr, tray_b);
    let a = slot_addr(base_a, slot_a);
    let b = slot_addr(base_b, slot_b);
    let tmp: [u8; MON_SIZE] = env.mem(a)[..MON_SIZE].try_into().unwrap();
    copy_mon(env, b, a);
    env.mem(b)[..MON_SIZE].copy_from_slice(&tmp);
}

/// C: `FUN_02007664`: clear a slot (nothing when the slot or box is out of range).
pub fn clear_slot<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32, slot: u32) {
    let meta = env.meta_addr(mgr);
    let tray = current_or(env, meta, tray);
    if slot < SLOTS as u32 && tray < MAX_BOXES {
        let base = env.tray_addr(mgr, tray);
        env.clear_mon(slot_addr(base, slot));
    }
}

/// C: `FUN_02007698`: the current box.
pub fn current_tray<E: BoxEnv>(env: &mut E, mgr: u32) -> u32 {
    let meta = env.meta_addr(mgr);
    rd32(env.mem(meta))
}

/// C: `FUN_020076A4`: the number of unlocked boxes (meta `+0x3DD`).
pub fn unlocked_count<E: BoxEnv>(env: &mut E, mgr: u32) -> u8 {
    env.meta(mgr)[COUNT_OFS]
}

/// C: `FUN_020076B4`: add 8 to the unlocked count unless it is already 24; returns the count.
pub fn unlock_more<E: BoxEnv>(env: &mut E, mgr: u32) -> u8 {
    let meta = env.meta(mgr);
    if meta[COUNT_OFS] != 24 {
        meta[COUNT_OFS] = meta[COUNT_OFS].wrapping_add(8);
    }
    meta[COUNT_OFS]
}

/// C: `FUN_020076D0`: the first free slot (species of id `0xA9` is 0) at or after
/// `(*tray, *slot)`, wrapping over the unlocked boxes; writes both and returns true. When none is
/// free and the count is not 24, the first new box gets it (count += 8).
pub fn find_free<E: BoxEnv>(env: &mut E, mgr: u32, tray_p: u32, slot_p: u32) -> bool {
    let meta = env.meta_addr(mgr);
    if rd32(env.mem(tray_p)) == CURRENT {
        let cur = rd32(env.mem(meta));
        wr32(env.mem(tray_p), cur);
    }
    let start = rd32(env.mem(tray_p));
    let mut tray = start;
    let mut slot = rd32(env.mem(slot_p));
    loop {
        let base = env.tray_addr(mgr, tray);
        while slot < SLOTS as u32 {
            if env.poke_get(slot_addr(base, slot), 0xA9, 0) == 0 {
                wr32(env.mem(tray_p), tray);
                wr32(env.mem(slot_p), slot);
                return true;
            }
            slot += 1;
        }
        tray += 1;
        let count = env.mem(meta)[COUNT_OFS] as u32;
        if tray >= count {
            tray = 0;
        }
        if tray == start {
            break;
        }
        slot = 0;
    }
    let count = env.mem(meta)[COUNT_OFS];
    if count == MAX_BOXES as u8 {
        return false;
    }
    wr32(env.mem(tray_p), count as u32);
    wr32(env.mem(slot_p), 0);
    env.mem(meta)[COUNT_OFS] = count.wrapping_add(8);
    true
}

/// C: `FUN_02007774`: free slots over all 24 boxes.
pub fn count_free_all<E: BoxEnv>(env: &mut E, mgr: u32) -> u32 {
    let mut total = 0u32;
    for tray in 0..MAX_BOXES {
        let base = env.tray_addr(mgr, tray);
        for slot in 0..SLOTS as u32 {
            if env.poke_get(slot_addr(base, slot), 0xA9, 0) == 0 {
                total = total.wrapping_add(1);
            }
        }
    }
    total
}

/// C: `FUN_020077B8`: free slots of one box (`CURRENT` = the current box; `>= 24` asserts).
pub fn count_free_tray<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32) -> u32 {
    let meta = env.meta_addr(mgr);
    let tray = current_or(env, meta, tray);
    if tray >= MAX_BOXES {
        env.assert_fail();
    }
    let base = env.tray_addr(mgr, tray);
    let mut free = 0u32;
    for slot in 0..SLOTS as u32 {
        if env.poke_get(slot_addr(base, slot), 0xA9, 0) == 0 {
            free = free.wrapping_add(1);
        }
    }
    free
}

/// C: `FUN_0200780C`: make `tray` the current box (when below 24).
pub fn set_current<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32) {
    let meta = env.meta_addr(mgr);
    if tray < MAX_BOXES {
        wr32(env.mem(meta), tray);
    }
}

/// C: `FUN_0200785C`: the name of `tray` out to `strbuf` (`>= 24` asserts).
pub fn name_get<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32, strbuf: u32) {
    let meta = env.meta_addr(mgr);
    let tray = current_or(env, meta, tray);
    if tray >= MAX_BOXES {
        env.assert_fail();
    }
    let name_addr = meta.wrapping_add(NAMES_OFS as u32 + tray * NAME_SIZE as u32);
    let name: [u8; NAME_SIZE] = env.mem(name_addr)[..NAME_SIZE].try_into().unwrap();
    env.name_to_str(strbuf, &name);
}

/// C: `FUN_02007898`: set the name of `tray` from `strbuf` (no-op for `>= 24`).
pub fn name_set<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32, strbuf: u32) {
    let meta = env.meta_addr(mgr);
    let tray = current_or(env, meta, tray);
    if tray < MAX_BOXES {
        let name_addr = meta.wrapping_add(NAMES_OFS as u32 + tray * NAME_SIZE as u32);
        let mut name: [u8; NAME_SIZE] = env.mem(name_addr)[..NAME_SIZE].try_into().unwrap();
        env.str_to_name(strbuf, &mut name, 0x14);
        env.mem(name_addr)[..NAME_SIZE].copy_from_slice(&name);
    }
}

/// C: `FUN_020078C4`: occupied slots of `tray` (`>= 24` asserts).
pub fn occupied<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32) -> u32 {
    let meta = env.meta_addr(mgr);
    let tray = current_or(env, meta, tray);
    if tray >= MAX_BOXES {
        env.assert_fail();
    }
    let base = env.tray_addr(mgr, tray);
    let mut n = 0u32;
    for slot in 0..SLOTS as u32 {
        if env.poke_get(slot_addr(base, slot), 0xA9, 0) != 0 {
            n = n.wrapping_add(1);
        }
    }
    n
}

/// C: `FUN_0200791C`: occupied slots of `tray` that are not eggs (`>= 24` asserts).
pub fn non_egg<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32) -> u32 {
    let meta = env.meta_addr(mgr);
    let tray = current_or(env, meta, tray);
    if tray >= MAX_BOXES {
        env.assert_fail();
    }
    let base = env.tray_addr(mgr, tray);
    let mut n = 0u32;
    for slot in 0..SLOTS as u32 {
        let mon = slot_addr(base, slot);
        if env.poke_get(mon, 0xA9, 0) != 0 && env.poke_get(mon, 0x4C, 0) == 0 {
            n = n.wrapping_add(1);
        }
    }
    n
}

/// C: `FUN_02007984`: occupied slots over all 24 boxes.
pub fn sum_occupied<E: BoxEnv>(env: &mut E, mgr: u32) -> u32 {
    let mut total = 0u32;
    for tray in 0..MAX_BOXES {
        total = total.wrapping_add(env.occupied_count(mgr, tray));
    }
    total
}

/// C: `FUN_020079A0`: non-egg occupied slots over all 24 boxes.
pub fn sum_non_egg<E: BoxEnv>(env: &mut E, mgr: u32) -> u32 {
    let mut total = 0u32;
    for tray in 0..MAX_BOXES {
        total = total.wrapping_add(env.non_egg_count(mgr, tray));
    }
    total
}

/// C: `FUN_020079BC`: `PokeParaGet` of one slot; returns its value (tail `bl FUN_02017E88`, r0 untouched up to the
/// `pop`; the field and box callers compare it, V-0022 audit). `tray` `>= 24` (but not `CURRENT`) or `slot >= 30` asserts.
pub fn mon_para_get<E: BoxEnv>(
    env: &mut E,
    mgr: u32,
    tray: u32,
    slot: u32,
    id: u32,
    buf: u32,
) -> u32 {
    let meta = env.meta_addr(mgr);
    if tray != CURRENT && tray >= MAX_BOXES {
        env.assert_fail();
    }
    if slot >= SLOTS as u32 {
        env.assert_fail();
    }
    let tray = current_or(env, meta, tray);
    let base = env.tray_addr(mgr, tray);
    env.poke_get(slot_addr(base, slot), id, buf)
}

/// C: `FUN_02007A18`: the address of a slot (same checks as [`mon_para_get`]).
pub fn slot_ptr<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32, slot: u32) -> u32 {
    let meta = env.meta_addr(mgr);
    if tray != CURRENT && tray >= MAX_BOXES {
        env.assert_fail();
    }
    if slot >= SLOTS as u32 {
        env.assert_fail();
    }
    let tray = current_or(env, meta, tray);
    let base = env.tray_addr(mgr, tray);
    slot_addr(base, slot)
}

/// C: `FUN_0200781C`: the wallpaper of box `tray` (0 for `tray >= 24`; no "current box" shortcut).
pub fn wallpaper<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32) -> u8 {
    let meta = env.meta(mgr);
    if tray < MAX_BOXES {
        meta[WALLPAPER_OFS + tray as usize]
    } else {
        0
    }
}

/// C: `FUN_02007838`: set the wallpaper of box `tray` (`CURRENT` = the current box; `>= 24` ignored).
pub fn set_wallpaper<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32, wallpaper: u8) {
    let meta = env.meta(mgr);
    let tray = if tray == CURRENT {
        u32::from_le_bytes(meta[0..4].try_into().unwrap())
    } else {
        tray
    };
    if tray < MAX_BOXES {
        meta[WALLPAPER_OFS + tray as usize] = wallpaper;
    }
}

/// C: `FUN_02007A68`: meta flags (+0x3DC) |= mask.
pub fn flags_set<E: BoxEnv>(env: &mut E, mgr: u32, mask: u8) {
    env.meta(mgr)[FLAGS_OFS] |= mask;
}

/// C: `FUN_02007A80`: whether any bit of `mask` is set in the flags byte.
pub fn flags_test<E: BoxEnv>(env: &mut E, mgr: u32, mask: u8) -> bool {
    env.meta(mgr)[FLAGS_OFS] & mask != 0
}

/// C: `FUN_02007AA4`: allocate the 4-byte box manager and store `savedata` in it.
pub fn new_mgr<E: BoxEnv>(env: &mut E, heap: u32, savedata: u32) -> u32 {
    let mgr = env.alloc([heap, 4, 0, NAME_BOX_SAVEDATA, 0x3B9]);
    wr32(env.mem(mgr), savedata);
    mgr
}

/// C: `FUN_02007AC8`: free the box manager.
pub fn free_mgr<E: BoxEnv>(env: &mut E, mgr: u32) {
    env.free(mgr);
}

/// C: `FUN_02007AD0`: clear the 30 stored Pokemon of the box block at address `tray_addr`.
pub fn clear_tray<E: BoxEnv>(env: &mut E, tray_addr: u32) {
    for i in 0..SLOTS as u32 {
        env.clear_mon(slot_addr(tray_addr, i));
    }
}

/// C: `FUN_02007AF4`: the save block `tray + 1` of the save data `*mgr`.
pub fn tray_block<E: BoxEnv>(env: &mut E, mgr: u32, tray: u32) -> u32 {
    let savedata = rd32(env.mem(mgr));
    env.save_block(savedata, tray.wrapping_add(1))
}

#[cfg(test)]
mod tests;
