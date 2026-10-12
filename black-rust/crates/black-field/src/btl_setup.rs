//! `btl_setup.c` (`0x02013090..0x02013A00`): the BATTLE SETUP record (`0xF0` bytes) that the field code
//! fills in before it starts a battle. The layout is documented in the header of
//! `tools/rom-diff/specs/black-field/btl_setup.spec`; every function is verified with
//! `tools/check-fn.sh black-field btl_setup`.
//!
//! A record is addressed by its pointer `p` (a `u32`); fields are read and written through the memory
//! window of the environment ([`crate::hooks::MemEnv`]). The party, game data and trainer data routines
//! it calls are in [`crate::hooks::BtlSetupHooks`].
//!
//! Function `0x020135C0` (`FUN_020135C0`, a variant of `FUN_0201347C` that ends with `+0 = 4`) lies in the
//! range but is not in the spec, so it is not ported.

use crate::hooks::{BtlSetupHooks, FILE_BTL_SETUP};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0201_3090,
    0x0201_30B4,
    0x0201_30C4,
    0x0201_30DC,
    0x0201_3128,
    0x0201_313C,
    0x0201_3144,
    0x0201_3158,
    0x0201_3160,
    0x0201_320C,
    0x0201_3220,
    0x0201_3234,
    0x0201_3270,
    0x0201_3284,
    0x0201_333C,
    0x0201_3360,
    0x0201_347C,
    0x0201_34F4,
    0x0201_3528,
    0x0201_3610,
    0x0201_3644,
    0x0201_3678,
    0x0201_36AC,
    0x0201_36E0,
    0x0201_3734,
    0x0201_3790,
    0x0201_37C0,
    0x0201_37F0,
    0x0201_3810,
    0x0201_3864,
    0x0201_3894,
    0x0201_38C4,
    0x0201_3930,
    0x0201_394C,
    0x0201_3980,
];

/// Size of the BATTLE SETUP record.
pub const SETUP_SIZE: u32 = 0xF0;
/// Size of the trainer data block (`FUN_02013234`).
pub const TRAINER_BLOCK_SIZE: u32 = 0x28;
/// Offset of the party object pointers (4 of them).
pub const PARTY_OFS: u32 = 0x24;
/// Offset of the 4-byte object pointers (slot 0 is the game data block, set by [`init_from_game`]).
pub const OBJ4_OFS: u32 = 0x34;
/// Offset of the trainer data block pointers (4 of them).
pub const TRAINER_OFS: u32 = 0x48;
/// Offset of the pointer to the `0xC00`-byte block.
pub const BLOCK_OFS: u32 = 0x94;
/// Offset of the `u16` flags (`or_flags` / `test_flags`).
pub const FLAGS_OFS: u32 = 0x82;
/// Event flag tested by [`init_from_game`] (sets setup flag 2).
pub const EVENT_FLAG_96D: u32 = 0x96D;

fn at(p: u32, ofs: u32) -> u32 {
    p.wrapping_add(ofs)
}

/// `p + base + 4 * i` (the original shifts the index without a range check).
fn slot(p: u32, base: u32, i: u32) -> u32 {
    p.wrapping_add(base).wrapping_add(i.wrapping_shl(2))
}

/// C: `FUN_02013090`: allocate (`"btl_setup.c"`, line 0x31) and initialise a setup record.
pub fn alloc<E: BtlSetupHooks>(env: &mut E, heap: u32) -> u32 {
    let p = env.heap_alloc(heap, SETUP_SIZE, 0, FILE_BTL_SETUP, 0x31);
    init(env, p);
    p
}

/// C: `FUN_020130B4`: release everything the record holds, then free it.
pub fn free<E: BtlSetupHooks>(env: &mut E, p: u32) {
    release(env, p);
    env.heap_free(p);
}

/// C: `FUN_020130C4`: zero the record and set the default config at `+8`.
pub fn init<E: BtlSetupHooks>(env: &mut E, p: u32) {
    env.mem().fill(p, SETUP_SIZE, 0);
    init_config(env, at(p, 8));
}

/// C: `FUN_020130DC`: free the party objects, trainer blocks, the 4-byte objects (not slot 0) and the
/// `0xC00` block, then zero the record.
pub fn release<E: BtlSetupHooks>(env: &mut E, p: u32) {
    for i in 0..4 {
        let party = env.r32(slot(p, PARTY_OFS, i));
        if party != 0 {
            env.heap_free(party);
        }
        let tr = env.r32(slot(p, TRAINER_OFS, i));
        if tr != 0 {
            free_trainer(env, tr);
        }
        if i != 0 {
            let o = env.r32(slot(p, OBJ4_OFS, i));
            if o != 0 {
                env.heap_free(o);
            }
        }
    }
    let blk = env.r32(at(p, BLOCK_OFS));
    if blk != 0 {
        env.heap_free(blk);
    }
    env.mem().fill(p, SETUP_SIZE, 0);
}

/// C: `FUN_02013128`: `FUN_0201AC2C(x, party i)`.
pub fn copy_to_party<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, i: u32) {
    let party = env.r32(slot(p, PARTY_OFS, i));
    env.fun_0201ac2c(x, party);
}

/// C: `FUN_0201313C`: party object pointer `i`.
pub fn party<E: BtlSetupHooks>(env: &mut E, p: u32, i: u32) -> u32 {
    env.r32(slot(p, PARTY_OFS, i))
}

/// C: `FUN_02013144`: or `mask` (low 16 bits) into the flags.
pub fn or_flags<E: BtlSetupHooks>(env: &mut E, p: u32, mask: u32) {
    let f = env.r16(at(p, FLAGS_OFS));
    env.w16(at(p, FLAGS_OFS), f | mask as u16);
}

/// C: `FUN_02013158`: flags `& mask`.
pub fn test_flags<E: BtlSetupHooks>(env: &mut E, p: u32, mask: u32) -> u32 {
    u32::from(env.r16(at(p, FLAGS_OFS))) & mask
}

/// C: `FUN_02013160`: copy the clock (`+0x74`/`+0x76`, minutes * 60) and the 7 bytes at `+0x78` from `x`,
/// set the battle type flags from field `0x12` of `x`, and run `FUN_0201A30C` / `FUN_0201A2A8` on every
/// party object.
pub fn read_from_save<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, y: u32) {
    let v = env.fun_0200ae54(x, 0x10);
    env.w16(at(p, 0x76), v.wrapping_mul(60) as u16);
    let v = env.fun_0200ae54(x, 0xF);
    env.w16(at(p, 0x74), v.wrapping_mul(60) as u16);
    env.fun_0200b040(x, at(p, 0x78));
    let kind = env.fun_0200ae54(x, 0x12);
    if matches!(kind, 1 | 3) {
        or_flags(env, p, 0x800);
    }
    if matches!(kind, 2 | 3) {
        or_flags(env, p, 0x400);
    }
    for i in 0..4 {
        let party = env.r32(slot(p, PARTY_OFS, i));
        if party != 0 {
            env.fun_0201a30c(x, party, y);
            let party = env.r32(slot(p, PARTY_OFS, i));
            env.fun_0201a2a8(x, party);
        }
    }
}

/// C: `FUN_0201320C`: config record `{9, 0, ...}`.
pub fn init_config_alt<E: BtlSetupHooks>(env: &mut E, q: u32) {
    env.w32(q, 9);
    env.w32(at(q, 4), 0);
    env.w8(at(q, 8), 0);
    env.w16(at(q, 0xA), 0);
    env.w8(at(q, 0xC), 0);
    env.w8(at(q, 0xD), 0);
    env.w8(at(q, 9), 0);
}

/// C: `FUN_02013220`: default config record (`+0xC = 10`).
pub fn init_config<E: BtlSetupHooks>(env: &mut E, q: u32) {
    env.w32(q, 0);
    env.w32(at(q, 4), 0);
    env.w8(at(q, 9), 0);
    env.w8(at(q, 8), 0);
    env.w16(at(q, 0xA), 0);
    env.w8(at(q, 0xC), 10);
    env.w8(at(q, 0xD), 0);
}

/// C: `FUN_02013234`: allocate a trainer data block (`"btl_setup.c"`, line 0xEF) with a `StrBuf` at
/// `+0x14` and two sub records at `+0x18` and `+0x20`.
pub fn new_trainer<E: BtlSetupHooks>(env: &mut E, heap: u32) -> u32 {
    let q = env.heap_alloc(heap, TRAINER_BLOCK_SIZE, 1, FILE_BTL_SETUP, 0xEF);
    let s = env.fun_020457b0(0x10, heap);
    env.w32(at(q, 0x14), s);
    env.w32(q, 0);
    env.fun_0202428c(at(q, 0x18));
    env.fun_0202428c(at(q, 0x20));
    q
}

/// C: `FUN_02013270`: free a trainer data block and its `StrBuf`.
pub fn free_trainer<E: BtlSetupHooks>(env: &mut E, q: u32) {
    let s = env.r32(at(q, 0x14));
    env.fun_02045808(s);
    env.heap_free(q);
}

/// C: `FUN_02013284`: make sure the party object at `pq` and trainer block `i` exist; with `y != 0`
/// load them from `y` and pick the message id at `+0x1A` from the trainer type.
pub fn load_trainer<E: BtlSetupHooks>(
    env: &mut E,
    p: u32,
    x: u32,
    i: u32,
    pq: u32,
    y: u32,
    heap: u32,
) {
    if env.r32(pq) == 0 {
        let party = env.fun_0201a920(heap);
        env.w32(pq, party);
    }
    let sl = slot(p, TRAINER_OFS, i);
    if env.r32(sl) == 0 {
        let tr = new_trainer(env, heap);
        env.w32(sl, tr);
    }
    if y != 0 {
        let tr = env.r32(sl);
        env.fun_0202a440(y, tr, heap);
        let party = env.r32(pq);
        env.fun_0202a4b4(y, party, heap);
        let v = env.fun_02012984(x);
        let party = env.r32(pq);
        env.fun_0216aae4(x, party, v);
        let tr = env.r32(sl);
        let arg = env.r32(at(tr, 4));
        let t = env.fun_0202a3fc(arg);
        let id = match t {
            2 | 3 => 0x47E,
            4 | 0xB => 0x480,
            6 | 9 => 0x491,
            7 | 8 => 0x47F,
            _ => 0x47D,
        };
        env.w16(at(p, 0x1A), id);
    }
}

/// C: `FUN_0201333C`: `FUN_020127A4(x)`, allocate party object 0 and `FUN_0201AC2C(FUN_02012934(x), it)`.
pub fn init_party0<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, heap: u32) {
    env.fun_020127a4(x);
    let party = env.fun_0201a920(heap);
    env.w32(at(p, PARTY_OFS), party);
    let a = env.fun_02012934(x);
    let party = env.r32(at(p, PARTY_OFS));
    env.fun_0201ac2c(a, party);
}

/// C: `FUN_02013360`: initialise the record around the game data `x` and the 16-byte config `q`.
pub fn init_from_game<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, q: &[u8]) {
    env.w16(at(p, 0x84), 0xFFFF);
    env.w8(at(p, 0x20), 0);
    env.w8(at(p, 0x21), 0);
    env.w8(at(p, 0x22), 0);
    env.w32(at(p, BLOCK_OFS), 0);
    env.w8(at(p, 0x23), 0);
    let f = env.r8(at(p, 0xC1));
    env.w32(at(p, 0x1C), 0);
    env.w8(at(p, 0xC1), f & !1);
    let f = env.r8(at(p, 0xC1));
    env.w32(at(p, 0x58), x);
    env.w8(at(p, 0xC1), f & !2);
    for i in 0..4 {
        env.w32(slot(p, PARTY_OFS, i), 0);
        env.w32(slot(p, OBJ4_OFS, i), 0);
    }
    for i in 0..6 {
        env.w8(at(p, 0xC2 + i), 0);
    }
    env.w8(at(p, 0x7F), 1);
    for i in 0..7 {
        env.w8(at(p, 0x78 + i), 0);
    }
    let v = env.fun_02012944(x);
    env.w32(at(p, 0x34), v);
    let v = env.fun_0201292c(x);
    env.w32(at(p, 0x60), v);
    let v = env.fun_02012924(x);
    env.w32(at(p, 0x64), v);
    let v = env.fun_0200ca1c(x);
    env.w32(at(p, 0x68), v);
    let v = env.fun_02012f20(x);
    env.w32(at(p, 0x70), v);
    let v = env.fun_02012f2c(x);
    env.w32(at(p, 0x6C), v);
    let r7 = env.fun_02012ebc(x);
    let clock = env.fun_0200c0e8();
    let v = env.fun_0200873c(r7);
    env.w32(at(p, 0x5C), v);
    let v = env.fun_0200c258(clock);
    env.w8(at(p, 0x80), v as u8);
    env.mem().write(at(p, 8), &q[..0x10]);
    env.w16(at(p, 0x18), 0x468);
    env.w16(at(p, 0x1A), 0x47C);
    env.w32(at(p, 0x8C), 1);
    let ew = env.fun_0201296c(x);
    if env.fun_020142e8(ew, EVENT_FLAG_96D) != 0 {
        or_flags(env, p, 2);
    }
}

/// C: `FUN_0201347C`: set up a setup record (kind `kind`, optional copy of the party `src` into party 1).
/// `heap` is truncated to 16 bits before it is passed on (the original reads it with `ldrh`).
pub fn setup_with_party<E: BtlSetupHooks>(
    env: &mut E,
    p: u32,
    x: u32,
    src: u32,
    q: &[u8],
    kind: u32,
    heap: u32,
) {
    let heap = heap & 0xFFFF;
    init(env, p);
    init_from_game(env, p, x, q);
    init_party0(env, p, x, heap);
    let party = env.fun_0201a920(heap);
    env.w32(at(p, 0x28), party);
    if src != 0 {
        env.fun_0201ac2c(src, party);
    }
    env.w32(at(p, 4), kind);
    env.w32(p, 0);
    let p0 = env.r32(at(p, PARTY_OFS));
    let a = env.fun_0201aa18(p0);
    let p0 = env.r32(at(p, PARTY_OFS));
    let b = env.fun_0201aa14(p0);
    if a == b {
        let v = env.fun_0201293c(x);
        let (mut r, mut s) = (0, 0);
        if env.fun_020076d0(v, &mut r, &mut s) == 0 {
            or_flags(env, p, 0x40);
        }
    }
}

/// C: `FUN_020134F4`: set up a record of kind `kind`. `heap` is truncated to 16 bits (`ldrh`).
pub fn setup_kind<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, kind: u32, q: &[u8], heap: u32) {
    let heap = heap & 0xFFFF;
    init(env, p);
    init_from_game(env, p, x, q);
    init_party0(env, p, x, heap);
    env.w32(p, 1);
    env.w32(at(p, 4), kind);
    env.w16(at(p, 0x1A), 0x47D);
}

/// C: `FUN_02013528`: set up a multi battle record (`+0 = 3`). `d` and `e` are stored as bytes.
#[allow(clippy::too_many_arguments)]
pub fn setup_multi<E: BtlSetupHooks>(
    env: &mut E,
    p: u32,
    x: u32,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    heap: u32,
) {
    init(env, p);
    // local config: `FUN_0201320C` then `+0 = 9`, `+4 = 1`; the last two bytes of the 16 are never
    // written by the original (uninitialised stack), UNVERIFIED: modelled as 0
    let mut q = [0u8; 16];
    q[0..4].copy_from_slice(&9u32.to_le_bytes());
    q[4..8].copy_from_slice(&1u32.to_le_bytes());
    init_from_game(env, p, x, &q);
    init_party0(env, p, x, heap);
    env.w32(p, 3);
    env.w32(at(p, 4), a);
    env.w32(at(p, 0x1C), c);
    env.w8(at(p, 0x20), d as u8);
    env.w8(at(p, 0x22), b as u8);
    env.w8(at(p, 0x21), e as u8);
    env.w16(at(p, 0x84), 0xFF);
    env.w16(at(p, 0x1A), 0x47D);
    for i in 1..4 {
        if env.r32(slot(p, OBJ4_OFS, i)) == 0 {
            let o = env.fun_0200846c(heap);
            env.w32(slot(p, OBJ4_OFS, i), o);
        }
        if env.r32(slot(p, PARTY_OFS, i)) == 0 {
            let o = env.fun_0201a920(heap);
            env.w32(slot(p, PARTY_OFS, i), o);
        }
    }
}

/// `setup_kind(kind)` then load trainer 1 from `y` (the shape of `FUN_02013610` / `02013644` / `02013678` /
/// `020136AC`).
fn setup_single<E: BtlSetupHooks>(
    env: &mut E,
    p: u32,
    x: u32,
    kind: u32,
    q: &[u8],
    y: u32,
    heap: u32,
) {
    let heap = heap & 0xFFFF;
    setup_kind(env, p, x, kind, q, heap);
    load_trainer(env, p, x, 1, at(p, 0x28), y, heap);
}

/// C: `FUN_02013610`: `setup_kind(0)` + trainer 1.
pub fn setup_single_0<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, q: &[u8], y: u32, heap: u32) {
    setup_single(env, p, x, 0, q, y, heap);
}

/// C: `FUN_02013644`: `setup_kind(1)` + trainer 1.
pub fn setup_single_1<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, q: &[u8], y: u32, heap: u32) {
    setup_single(env, p, x, 1, q, y, heap);
}

/// C: `FUN_02013678`: `setup_kind(2)` + trainer 1.
pub fn setup_single_2<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, q: &[u8], y: u32, heap: u32) {
    setup_single(env, p, x, 2, q, y, heap);
}

/// C: `FUN_020136AC`: `setup_kind(3)` + trainer 1.
pub fn setup_single_3<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, q: &[u8], y: u32, heap: u32) {
    setup_single(env, p, x, 3, q, y, heap);
}

/// C: `FUN_020136E0`: `setup_kind(1)`, trainers 1 and 3 (from `y` and `z`), `+0x22 = 4` (double battle).
/// `heap` is read with `ldrh` for the first call and the first trainer, and as 16 bits again for the second.
pub fn setup_double<E: BtlSetupHooks>(
    env: &mut E,
    p: u32,
    x: u32,
    q: &[u8],
    y: u32,
    z: u32,
    heap: u32,
) {
    let heap = heap & 0xFFFF;
    setup_kind(env, p, x, 1, q, heap);
    load_trainer(env, p, x, 1, at(p, 0x28), y, heap);
    load_trainer(env, p, x, 3, at(p, 0x30), z, heap);
    env.w8(at(p, 0x22), 4);
}

/// C: `FUN_02013734`: `setup_kind(1)`, trainers 2, 1 and 3 (from `y2`, `y1`, `y3`), `+0x22 = 3`.
/// Unlike [`setup_double`] the full `heap` goes to the trainer loads (only `setup_kind` truncates it).
#[allow(clippy::too_many_arguments)]
pub fn setup_triple<E: BtlSetupHooks>(
    env: &mut E,
    p: u32,
    x: u32,
    q: &[u8],
    y2: u32,
    y1: u32,
    y3: u32,
    heap: u32,
) {
    setup_kind(env, p, x, 1, q, heap);
    load_trainer(env, p, x, 2, at(p, 0x2C), y2, heap);
    load_trainer(env, p, x, 1, at(p, 0x28), y1, heap);
    load_trainer(env, p, x, 3, at(p, 0x30), y3, heap);
    env.w8(at(p, 0x22), 3);
}

/// `FUN_0203D1A4(a)` then `setup_multi(kind, 0, a, b, ret, heap)` (the shape of `FUN_02013790` /
/// `020137C0` / `02013864` / `02013894`).
fn setup_multi_a<E: BtlSetupHooks>(
    env: &mut E,
    p: u32,
    x: u32,
    kind: u32,
    a: u32,
    b: u32,
    heap: u32,
) {
    let r = env.fun_0203d1a4(a);
    setup_multi(env, p, x, kind, 0, a, b, r, heap & 0xFFFF);
}

/// C: `FUN_02013790`: `setup_multi` with kind 0.
pub fn setup_multi_0<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, a: u32, b: u32, heap: u32) {
    setup_multi_a(env, p, x, 0, a, b, heap);
}

/// C: `FUN_020137C0`: `setup_multi` with kind 1.
pub fn setup_multi_1<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, a: u32, b: u32, heap: u32) {
    setup_multi_a(env, p, x, 1, a, b, heap);
}

/// C: `FUN_02013864`: `setup_multi` with kind 2.
pub fn setup_multi_2<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, a: u32, b: u32, heap: u32) {
    setup_multi_a(env, p, x, 2, a, b, heap);
}

/// C: `FUN_02013894`: `setup_multi` with kind 3.
pub fn setup_multi_3<E: BtlSetupHooks>(env: &mut E, p: u32, x: u32, a: u32, b: u32, heap: u32) {
    setup_multi_a(env, p, x, 3, a, b, heap);
}

/// C: `FUN_020137F0`: `setup_multi(1, 1, a, b, c as u8, heap as u16)`.
pub fn setup_multi_b<E: BtlSetupHooks>(
    env: &mut E,
    p: u32,
    x: u32,
    a: u32,
    b: u32,
    c: u32,
    heap: u32,
) {
    setup_multi(env, p, x, 1, 1, a, b, c & 0xFF, heap & 0xFFFF);
}

/// C: `FUN_02013810`: `setup_multi(1, 2, a, b, c as u8, heap as u16)`, `+0 = 1`, trainers 1 and 3 from
/// `y1` and `y2`.
#[allow(clippy::too_many_arguments)]
pub fn setup_multi_c<E: BtlSetupHooks>(
    env: &mut E,
    p: u32,
    x: u32,
    a: u32,
    b: u32,
    c: u32,
    y1: u32,
    y2: u32,
    heap: u32,
) {
    let heap = heap & 0xFFFF;
    setup_multi(env, p, x, 1, 2, a, b, c & 0xFF, heap);
    env.w32(p, 1);
    load_trainer(env, p, x, 1, at(p, 0x28), y1, heap);
    load_trainer(env, p, x, 3, at(p, 0x30), y2, heap);
}

/// C: `FUN_020138C4`: build a full record around the game data `w`: zero it, allocate the four party
/// objects, 4-byte objects and trainer blocks, then the `0xC00` block.
pub fn build_all<E: BtlSetupHooks>(env: &mut E, p: u32, w: u32, heap: u32) {
    env.mem().fill(p, SETUP_SIZE, 0);
    for i in 0..4 {
        let v = env.fun_0201a920(heap);
        env.w32(slot(p, PARTY_OFS, i), v);
        let v = env.fun_0200846c(heap);
        env.w32(slot(p, OBJ4_OFS, i), v);
        let v = new_trainer(env, heap);
        env.w32(slot(p, TRAINER_OFS, i), v);
    }
    env.w32(at(p, 0x58), w);
    let t = env.fun_02012ebc(w);
    let r4 = env.fun_0200873c(t);
    let r1 = env.fun_020082ec(heap);
    env.w32(at(p, 0x5C), r1);
    env.fun_02008314(r4, r1);
    let v = env.fun_0200ca1c(w);
    env.w32(at(p, 0x68), v);
    env.w8(at(p, 0x23), 1);
    alloc_block(env, p, heap);
}

/// C: `FUN_02013930`: free the object at `+0x5C`, then [`release`].
pub fn free_all<E: BtlSetupHooks>(env: &mut E, p: u32) {
    let o = env.r32(at(p, 0x5C));
    if o != 0 {
        env.heap_free(o);
        env.w32(at(p, 0x5C), 0);
    }
    release(env, p);
}

/// C: `FUN_0201394C`: when `+0 == 1`, set it to 2 and pick the message ids from the trainer type of
/// trainer block 1.
pub fn start_battle_ids<E: BtlSetupHooks>(env: &mut E, p: u32) {
    if env.r32(p) == 1 {
        env.w32(p, 2);
        let tr = env.r32(at(p, 0x4C));
        let arg = env.r32(at(tr, 4));
        let t = env.fun_0202a3fc(arg);
        if t != 0xB {
            env.w16(at(p, 0x18), 0x46B);
            env.w16(at(p, 0x1A), 0x47D);
        } else {
            env.w16(at(p, 0x18), 0x470);
            env.w16(at(p, 0x1A), 0x480);
        }
    }
}

/// C: `FUN_02013980`: allocate the `0xC00`-byte block at `+0x94` if there is none.
pub fn alloc_block<E: BtlSetupHooks>(env: &mut E, p: u32, heap: u32) {
    if env.r32(at(p, BLOCK_OFS)) == 0 {
        let b = env.heap_alloc(heap, 0xC00, 0, FILE_BTL_SETUP, 0x39A);
        env.w32(at(p, BLOCK_OFS), b);
    }
}

#[cfg(test)]
mod tests;
