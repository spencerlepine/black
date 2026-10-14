//! `event_battle.c` after its head (overlay 10, `0x0216E954..0x0216F088`, F1 row 10.39; F5b): the battle child GMEVENT
//! (proc `0x0216EB99`: fade, the battle overlays 0x5D / 0x5C, the return to the field), the capture-demo battle, the
//! special-encounter override of the foe that `0216E818` sets up, the post-battle processing (last result into GAMEDATA,
//! the roamer update, game-beacon notifications, record counter 0x4F) and the battle event work init.
//!
//! The head (`0x0216E62C..0x0216E954`) is the area [`crate::event_battle_head`]. Spec:
//! `tools/rom-diff/specs/black-field/event_battle.spec` (its header documents the work, the setup fields and the ROM
//! tables). Every callee outside this area is reached through [`FieldEnv::call`] by address; callees inside it are Rust
//! calls. The works the GMEVENTs hand out are read and written through the [`Mem`] window.
//!
//! BATTLE EVENT work W (0x28, [`work_init`]): `+0` GAMESYS, `+4` GAMEDATA, `+8` BATTLE_SETUP, `+0xC` / `+0x10` the
//! (GAMEDATA, setup) pair handed to battle overlay 0x5C, `+0x14` 1 = end right after the battle, `+0x18` 1 = never "lost",
//! `+0x1C` 1 = record the battle, `+0x20` 1 = keep the setup, `+0x24` a cookie of `021CF5D8` / `021CF498`.
//!
//! Integrity checks: steps 6 and 7 of [`battle_proc`] call ARM entries (`021882DC` / `02188354` / `02188390`) of an
//! overlay outside the field set with `(0x0216E681, callback, 0)`; a mismatch leaves a sum that starts at 0x013A1AB5
//! not divisible by 0x1933, and the step then calls `020056A0(0x02011D35, 0, 0x7F)` and counts the global
//! `0x0214624C`. Ported as the original does it (the callees are by address).
//!
//! [`Mem`]: crate::hooks::Mem

use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_E95C,
    0x0216_EAB0,
    0x0216_EB98,
    0x0216_EDFC,
    0x0216_EE20,
    0x0216_EE40,
    0x0216_EE7C,
    0x0216_EF84,
    0x0216_EFA8,
    0x0216_EFCC,
    0x0216_EFDC,
    0x0216_EFF0,
    0x0216_F02C,
    0x0216_F054,
    0x0216_F064,
    0x0216_F070,
    0x0216_F07C,
];

/// The battle child GMEVENT proc [`battle_proc`] (Thumb address).
pub const BATTLE_PROC: u32 = 0x0216_EB99;
/// Address of the string `"event_battle.c"` (the file argument of the allocation in [`capture_demo_new`]).
pub const FILE_EVENT_BATTLE: u32 = 0x0217_2B54;

/// `FUN_020120F4(gsys)`: the GAMEDATA of the game system.
const GAMEDATA: u32 = 0x0201_20F4;
/// `FUN_0201210C(gsys)`: the FIELDMAP of the game system.
const FIELDMAP: u32 = 0x0201_210C;
/// `FUN_020122C0(gsys, parent, proc, work size)`: create a GMEVENT.
const EVENT_NEW: u32 = 0x0201_22C0;
/// `FUN_0201235C(event, next)`: change the GMEVENT to `next`.
const EVENT_CHANGE: u32 = 0x0201_235C;
/// `FUN_02012374(event, child)`: call the child GMEVENT.
const EVENT_CALL: u32 = 0x0201_2374;
/// `FUN_02012444(event, overlay, proc data, param)`: run an app proc from the event.
const EVENT_APP: u32 = 0x0201_2444;
/// `FUN_020124E8(event)`: the work of a GMEVENT.
const EVENT_WORK: u32 = 0x0201_24E8;
/// `FUN_02008DF0(rec, id)`: add 1 to record counter `id`.
const RECORD_ADD: u32 = 0x0200_8DF0;
/// `FUN_0201313C(setup, 1)`: the party of a BATTLE_SETUP.
const SETUP_PARTY: u32 = 0x0201_313C;
/// `FUN_020130B4(setup)` (`btl_setup` free).
const SETUP_FREE: u32 = 0x0201_30B4;
/// `FUN_02017E1C(mon, id, x)`: PokeParam get.
const POKE_GET: u32 = 0x0201_7E1C;
/// `FUN_02017E40(mon, id, value)`: PokeParam set.
const POKE_SET: u32 = 0x0201_7E40;
/// `FUN_0201AB28(party, i)`: party member `i`.
const PARTY_MEMBER: u32 = 0x0201_AB28;
/// `Heap_AllocDebug(heap, size, flag, file, line)` and `Heap_Free(block)`.
const HEAP_ALLOC: u32 = 0x0203_0734;
const HEAP_FREE: u32 = 0x0203_07B0;
/// The size of a party Pokemon (`FUN_020173EC()` returns this constant).
const MON_SIZE: u32 = 0xDC;

/// ROM table `0x021725B8` (u8 x 8, read by [`kind_of`]).
pub const KIND_TABLE: [u8; 8] = [0xFF, 0x02, 0xFF, 0x01, 0x01, 0x00, 0x00, 0x00];
/// ROM table `0x021725C0`: the game-beacon notification called by [`notify_trainer`], by class ([`trainer_class`])
/// and column (Thumb addresses in overlay 10's game_beacon code).
pub const NOTIFY_TABLE: [[u32; 2]; 3] = [
    [0x0216_5175, 0x0216_51AD],
    [0x0216_51E5, 0x0216_521D],
    [0x0216_5255, 0x0216_528D],
];
/// ROM table `0x021725D8`: `[result][type]` (7 x 5), 0 = the battle counts as lost ([`lost`]).
pub const LOST_TABLE: [[u8; 5]; 7] = [
    [0, 0, 0, 0, 0],
    [1, 1, 1, 1, 0],
    [0, 0, 0, 0, 0],
    [1, 2, 2, 0, 0],
    [1, 2, 1, 1, 0],
    [1, 2, 2, 2, 0],
    [0, 0, 0, 0, 0],
];

/// C: `FUN_0216E95C`: the capture-demo battle. Builds two one-Pokemon parties on
/// heap `0x8000 | heap & 0x7FFF` (species 0x23C level 7 with moves 1 / 0x2D, species 0x1F8 level 2 with moves 0x21 /
/// 0x2B), a FIELD SITUATION from `FUN_021AA304(&sit, gd, a1)` with `+0 = 0`, `+4 = 5`, the setup
/// (`FUN_020135C0(setup, gd, p1, p2, &sit, heap)` on `btl_setup` alloc(`heap`)), frees the temporaries, and returns a
/// battle child event ([`BATTLE_PROC`], work init, `+0x14 = 1`, `FUN_021CF5D8(0x1F8, 5, 0, a1, W+0x24, *(W+8)+0x18)`,
/// `+0x24 = 0`).
pub fn capture_demo_new<E: FieldEnv>(env: &mut E, gsys: u32, a1: u32, heap: u32) -> u32 {
    let hid = 0x8000 | (heap & 0x7FFF);
    let gd = env.callv(GAMEDATA, &[gsys]);
    let setup = env.callv(0x0201_3090, &[heap]);
    let p1 = env.callv(0x0201_A920, &[hid]);
    let p2 = env.callv(0x0201_A920, &[hid]);
    let mon = env.callv(HEAP_ALLOC, &[hid, MON_SIZE, 1, FILE_EVENT_BATTLE, 0x1DA]);
    for (party, species, level, m1, m2) in [(p1, 0x23C, 7, 1, 0x2D), (p2, 0x1F8, 2, 0x21, 0x2B)] {
        env.callv(0x0201_756C, &[mon, species, level, 0, 0]);
        env.callv(POKE_SET, &[mon, 0x36, m1]);
        env.callv(POKE_SET, &[mon, 0x37, m2]);
        env.callv(POKE_SET, &[mon, 0x38, 0]);
        env.callv(POKE_SET, &[mon, 0x39, 0]);
        env.callv(0x0201_A98C, &[party, mon]);
    }
    let sit = env.mem().push_local(0x10);
    env.call(
        0x021A_A304,
        &mut [CArg::Out(sit, 0x10), CArg::V(gd), CArg::V(a1)],
    );
    env.w32(sit + 4, 5);
    env.w32(sit, 0);
    env.call(
        0x0201_35C0,
        &mut [
            CArg::V(setup),
            CArg::V(gd),
            CArg::V(p1),
            CArg::V(p2),
            CArg::In(sit, 0x10),
            CArg::V(heap),
        ],
    );
    env.mem().pop_local(sit);
    env.callv(HEAP_FREE, &[mon]);
    env.callv(HEAP_FREE, &[p2]);
    env.callv(HEAP_FREE, &[p1]);
    let ev = env.callv(EVENT_NEW, &[gsys, 0, BATTLE_PROC, 0x28]);
    let w = env.callv(EVENT_WORK, &[ev]);
    work_init(env, w, gsys, setup);
    env.w32(w + 0x14, 1);
    let x = env.r32(w + 8).wrapping_add(0x18);
    env.callv(0x021C_F5D8, &[0x1F8, 5, 0, a1, w + 0x24, x]);
    env.w32(w + 0x24, 0);
    ev
}

/// C: `FUN_0216EAB0`: the battle event of `FUN_0216E818(gsys, fm, a, b, c, d, e)`;
/// when entry `f` of the save record `S = FUN_02012FD8(gd)` is present (`FUN_0200E740(S, f)`), its foe (member 0 of
/// the setup's `+0x28` party) is re-initialised from that entry: `FUN_02017588(mon, (u16) FUN_0200E788(S, f, 21),
/// FUN_0200E74C(S, f), FUN_0200E7B4(S, f), 0, -1, FUN_0200E7A4(S, f), 0)`, the entry's item (`FUN_0200E76C` /
/// `FUN_0200E780`, PokeParam 10) and field 0x74 (`FUN_0200E7AC`), the species back (`FUN_020178D8`) and a recalc
/// (`FUN_02017FA8`). Returns the event.
#[allow(clippy::too_many_arguments)]
pub fn special_battle_new<E: FieldEnv>(
    env: &mut E,
    gsys: u32,
    fm: u32,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    f: u32,
) -> u32 {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let s = env.callv(0x0201_2FD8, &[gd]);
    let ev = env.callv(0x0216_E818, &[gsys, fm, a, b, c, d, e]);
    if env.callv(0x0200_E740, &[s, f]) == 0 {
        return ev;
    }
    let w = env.callv(EVENT_WORK, &[ev]);
    let setup = env.r32(w + 8);
    let party = env.r32(setup + 0x28);
    let mon = env.callv(PARTY_MEMBER, &[party, 0]);
    let species = env.callv(POKE_GET, &[mon, 5, 0]);
    let p0 = env.callv(0x0200_E788, &[s, f, 21]);
    let p1 = env.callv(0x0200_E74C, &[s, f]);
    let p2 = env.callv(0x0200_E7B4, &[s, f]);
    let p3 = env.callv(0x0200_E7A4, &[s, f]);
    env.callv(0x0201_7588, &[mon, p0 & 0xFFFF, p1, p2, 0, u32::MAX, p3, 0]);
    if env.callv(0x0200_E76C, &[s, f]) != 0 {
        let item = env.callv(0x0200_E780, &[s, f]);
        env.callv(POKE_SET, &[mon, 10, item]);
    }
    let v = env.callv(0x0200_E7AC, &[s, f]);
    env.callv(POKE_SET, &[mon, 0x74, v]);
    env.callv(0x0201_78D8, &[mon, species & 0xFFFF]);
    env.callv(0x0201_7FA8, &[mon]);
    ev
}

/// The sum the integrity checks of [`battle_proc`] start from (divisible by [`CHECK_MOD`]).
const CHECK_SEED: u32 = 0x013A_1AB5;
/// The modulus of the integrity sum.
const CHECK_MOD: u32 = 0x1933;

/// One integrity check of [`battle_proc`]: `FUN_<at>(0x0216E681, cb, 0) == !cb`.
fn check<E: FieldEnv>(env: &mut E, at: u32, cb: u32) -> bool {
    env.callv(at, &[0x0216_E681, cb, 0]) == !cb
}

/// C: `FUN_0216EB98`: the battle at step `*seq`
/// of the BATTLE EVENT work `w` (steps in the module doc of the spec; every step but 8 / 11 moves on and returns 0
/// through the integrity tail, which calls `FUN_020056A0(0x02011D35, 0, 0x7F)` and counts `0x0214624C` when the sum
/// of this call is not divisible by 0x1933). Returns 1 when the event is done.
pub fn battle_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, w: u32) -> u32 {
    let gsys = env.r32(w);
    let gd = env.r32(w + 4);
    let fm = env.callv(FIELDMAP, &[gsys]);
    let mut sum = CHECK_SEED;
    let step = env.r32(seq);
    let mut next = true;
    match step {
        0 => {
            let setup = env.r32(w + 8);
            let bgm = env.r16(setup + 0x18) as u32;
            let child = env.callv(0x0202_9BCC, &[gsys, bgm]);
            env.callv(EVENT_CALL, &[event, child]);
        }
        1 => {
            let x = env.callv(0x0218_9A08, &[fm]);
            let c = env.r32(w + 0x24);
            env.callv(0x021C_F498, &[x, event, c]);
        }
        2 => {
            let child = env.callv(0x0201_44DC, &[gsys, fm]);
            env.callv(EVENT_CALL, &[event, child]);
        }
        3 => {
            let x = env.callv(0x0201_2B38, &[gd]);
            env.callv(0x0201_447C, &[x, 1]);
            let y = env.callv(0x0201_2F20, &[gd]);
            env.callv(0x0202_E79C, &[y]);
            let setup = env.r32(w + 8);
            env.callv(EVENT_APP, &[event, 0x5D, 0x021E_FDAC, setup]);
        }
        4 => {
            let setup = env.r32(w + 8);
            env.w32(w + 0x10, setup);
            env.w32(w + 0xC, gd);
            env.callv(EVENT_APP, &[event, 0x5C, 0x021B_9D94, w + 0xC]);
        }
        5 => {
            env.callv(0x0203_4AC4, &[0xE6]);
            let x = env.callv(0x0201_2B38, &[gd]);
            env.callv(0x0201_447C, &[x, 0]);
            let y = env.callv(0x0201_2F20, &[gd]);
            env.callv(0x0202_E80C, &[y]);
        }
        6 => {
            if !check(env, 0x0218_82DC, 0x0216_F065) {
                sum = sum.wrapping_add(0x3D1);
            }
            after_battle(env, w, gd);
            if env.r32(w + 0x1C) != 0 {
                let p = env.callv(0x0201_2F50, &[gd]);
                let x = env.r32(p);
                let setup = env.r32(w + 8);
                env.callv(0x0216_8D30, &[x, setup]);
            }
            let s = env.r32(seq).wrapping_add(1);
            env.w32(seq, s);
            if !check(env, 0x0218_8354, 0x0216_F071) {
                sum = sum.wrapping_add(0x5D1);
            }
            next = false;
        }
        7 => {
            if check(env, 0x0218_8390, 0x0216_F07D) {
                sum = sum.wrapping_add(0x9D);
            }
            let child = if env.r32(w + 0x18) == 0 && lost_of(env, w) == 1 {
                env.callv(0x0202_9B28, &[gsys, 30])
            } else {
                env.callv(0x0202_9C00, &[gsys])
            };
            env.callv(EVENT_CALL, &[event, child]);
            env.callv(0x0203_4A5C, &[0xE6]);
        }
        8 => {
            if env.r32(w + 0x18) == 0 && lost_of(env, w) == 1 {
                if env.r32(w + 0x14) == 1 {
                    free_setup(env, w);
                    return 1;
                }
                free_setup(env, w);
                let nx = env.callv(0x0215_F6E8, &[gsys]);
                env.callv(EVENT_CHANGE, &[event, nx]);
                return 0;
            }
            let child = env.callv(0x0201_45CC, &[gsys]);
            env.callv(EVENT_CALL, &[event, child]);
        }
        9 => {
            let child = env.callv(0x0202_9CC0, &[gsys]);
            env.callv(EVENT_CALL, &[event, child]);
        }
        10 => {
            if env.r32(w + 0x14) == 0 {
                let child = env.callv(0x021B_E92C, &[gsys, fm, 0, 0, 1, 0, 0]);
                env.callv(EVENT_CALL, &[event, child]);
            }
        }
        11 => {
            free_setup(env, w);
            return 1;
        }
        _ => next = false,
    }
    if next {
        let s = env.r32(seq).wrapping_add(1);
        env.w32(seq, s);
    }
    if !sum.is_multiple_of(CHECK_MOD) {
        env.callv(0x0200_56A0, &[0x0201_1D35, 0, 0x7F]);
        let n = env.r32(0x0214_624C).wrapping_add(1);
        env.w32(0x0214_624C, n);
    }
    0
}

/// C: `FUN_0216EDFC`: the class of trainer `x`: `k = FUN_0202A3FC(FUN_0202A230(x,
/// 1))`; 12 or 9 -> 0, 2 -> 1, else 2.
pub fn trainer_class<E: FieldEnv>(env: &mut E, x: u32) -> u32 {
    let t = env.callv(0x0202_A230, &[x, 1]);
    match env.callv(0x0202_A3FC, &[t]) {
        12 | 9 => 0,
        2 => 1,
        _ => 2,
    }
}

/// C: `FUN_0216EE20`: the game-beacon notification [`NOTIFY_TABLE`]`[class][i]`
/// for trainer `x` (class [`trainer_class`]), called with `x`.
pub fn notify_trainer<E: FieldEnv>(env: &mut E, x: u32, i: u32) {
    let k = trainer_class(env, x) as usize;
    let f = NOTIFY_TABLE[k][i as usize];
    env.callv(f & !1, &[x]);
}

/// C: `FUN_0216EE40`: the game-beacon notification `i` (0: `FUN_0216505C`, 1:
/// `FUN_02165078`, 2: `FUN_021653F4`; other values: none) with the species of the setup's party member 0.
pub fn notify_species<E: FieldEnv>(env: &mut E, setup: u32, i: u32) {
    let party = env.callv(SETUP_PARTY, &[setup, 1]);
    let mon = env.callv(PARTY_MEMBER, &[party, 0]);
    let species = env.callv(POKE_GET, &[mon, 5, 0]) & 0xFFFF;
    let f = match i {
        0 => 0x0216_505C,
        1 => 0x0216_5078,
        2 => 0x0216_53F4,
        _ => return,
    };
    env.callv(f, &[species]);
}

/// C: `FUN_0216EE7C`: the game-beacon notifications after a battle. Unless the
/// result (`setup +0x8C`) is 0 or 2, for the lead Pokemon (member `FUN_0201AB18(P)` of `P = FUN_02012934(gd)`) with
/// its nickname in a 12-character STRBUF: fainted (`FUN_021654AC`), else HP at most half (`FUN_02165424`); the first
/// of the 4 moves with 0 PP (`FUN_02165468`); a status condition (`FUN_021654F0`). Then by battle type `setup +0`:
/// type 0 with result 3 / 1 -> [`notify_species`] 2 / 1; type 1 with result 1 -> [`notify_trainer`]`((u16)
/// *(setup +0x4C), 1)`.
pub fn after_battle_notify<E: FieldEnv>(env: &mut E, w: u32) {
    let setup = env.r32(w + 8);
    let gd = env.r32(w + 4);
    let party = env.callv(0x0201_2934, &[gd]);
    let i = env.callv(0x0201_AB18, &[party]);
    let mon = env.callv(PARTY_MEMBER, &[party, i]);
    let result = env.r32(setup + 0x8C);
    if result != 0 && result != 2 {
        let s = env.callv(0x0204_57B0, &[12, 0x8004]);
        env.callv(POKE_GET, &[mon, 0x73, s]);
        let hp = env.callv(POKE_GET, &[mon, 0xA0, 0]) & 0xFFFF;
        let max = env.callv(POKE_GET, &[mon, 0xA1, 0]) & 0xFFFF;
        if hp == 0 {
            env.callv(0x0216_54AC, &[s]);
        } else if hp * 2 <= max {
            env.callv(0x0216_5424, &[s]);
        }
        for k in 0..4 {
            if env.callv(POKE_GET, &[mon, 0x36 + k, 0]) != 0
                && env.callv(POKE_GET, &[mon, 0x3A + k, 0]) == 0
            {
                env.callv(0x0216_5468, &[s]);
                break;
            }
        }
        if env.callv(POKE_GET, &[mon, 0x9D, 0]) != 0 {
            env.callv(0x0216_54F0, &[s]);
        }
        env.callv(0x0204_5808, &[s]);
    }
    match env.r32(setup) {
        0 => match env.r32(setup + 0x8C) {
            3 => notify_species(env, setup, 2),
            1 => notify_species(env, setup, 1),
            _ => {}
        },
        1 if env.r32(setup + 0x8C) == 1 => {
            let p = env.r32(setup + 0x4C);
            let id = env.r32(p) & 0xFFFF;
            notify_trainer(env, id, 1);
        }
        _ => {}
    }
}

/// C: `FUN_0216EF84`: a wild battle (`setup +0 == 0`) that ended with result 4
/// adds 1 to record counter 0x4F (`FUN_02008DF0(FUN_02012F2C(gd of W+0), 0x4F)`).
pub fn count_escape<E: FieldEnv>(env: &mut E, w: u32) {
    let setup = env.r32(w + 8);
    let gsys = env.r32(w);
    let gd = env.callv(GAMEDATA, &[gsys]);
    let rec = env.callv(0x0201_2F2C, &[gd]);
    if env.r32(setup) == 0 && env.r32(setup + 0x8C) == 4 {
        env.callv(RECORD_ADD, &[rec, 0x4F]);
    }
}

/// C: `FUN_0216EFA8`: whether battle `result` of type `ty` counts as lost:
/// `ty == 4` -> 0, else [`LOST_TABLE`]`[result][ty] == 0`.
pub fn lost(result: u32, ty: u32) -> u32 {
    if ty == 4 {
        return 0;
    }
    (LOST_TABLE[result as usize][ty as usize] == 0) as u32
}

/// C: `FUN_0216EFCC`: [`KIND_TABLE`]`[i]`, 0xFF read as 1.
pub fn kind_of(i: u32) -> u32 {
    match KIND_TABLE[i as usize] {
        0xFF => 1,
        v => v as u32,
    }
}

/// C: `FUN_0216EFDC`: [`lost`] for the setup of work `w`
/// (`(setup +0x8C, setup +0)`).
pub fn lost_of<E: FieldEnv>(env: &mut E, w: u32) -> u32 {
    let setup = env.r32(w + 8);
    let result = env.r32(setup + 0x8C);
    let ty = env.r32(setup);
    lost(result, ty)
}

/// C: `FUN_0216EFF0`: after the battle: GAMEDATA `+0x1BC` = the result
/// (`FUN_02012A4C`), the roamer update `FUN_0215E5B0(gd, setup)`, [`after_battle_notify`], [`count_escape`],
/// `FUN_0202C83C(gsys, FUN_02012AC8(FUN_020127A4(gd)))`.
pub fn after_battle<E: FieldEnv>(env: &mut E, w: u32, gd: u32) {
    let setup = env.r32(w + 8);
    let result = env.r32(setup + 0x8C);
    env.callv(0x0201_2A4C, &[gd, result]);
    let setup = env.r32(w + 8);
    env.callv(0x0215_E5B0, &[gd, setup]);
    after_battle_notify(env, w);
    count_escape(env, w);
    let p = env.callv(0x0201_27A4, &[gd]);
    let x = env.callv(0x0201_2AC8, &[p]);
    let gsys = env.r32(w);
    env.callv(0x0202_C83C, &[gsys, x]);
}

/// C: `FUN_0216F02C`: clear the BATTLE EVENT work `w` (0x28 bytes) and set `+0`
/// gsys, `+4` its GAMEDATA, `+8` setup (`+0x14` / `+0x18` / `+0x1C` stay 0).
pub fn work_init<E: FieldEnv>(env: &mut E, w: u32, gsys: u32, setup: u32) {
    env.mem().fill(w, 0x28, 0);
    env.w32(w, gsys);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.w32(w + 4, gd);
    env.w32(w + 8, setup);
    env.w32(w + 0x14, 0);
    env.w32(w + 0x18, 0);
    env.w32(w + 0x1C, 0);
}

/// C: `FUN_0216F054`: free the setup (`btl_setup` free) unless `+0x20` is set.
pub fn free_setup<E: FieldEnv>(env: &mut E, w: u32) {
    if env.r32(w + 0x20) == 0 {
        let setup = env.r32(w + 8);
        env.callv(SETUP_FREE, &[setup]);
    }
}

/// C: `FUN_0216F064`, `FUN_0216F070`, `FUN_0216F07C` (unlabeled routines): the callbacks of the integrity checks:
/// `FUN_0203F0A8()` twice.
pub fn check_callback<E: FieldEnv>(env: &mut E) {
    env.callv(0x0203_F0A8, &[]);
    env.callv(0x0203_F0A8, &[]);
}

#[cfg(test)]
mod tests;
