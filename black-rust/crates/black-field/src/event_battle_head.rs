//! The head of `event_battle.c` (overlay 10, `0x0216E62C..0x0216E954`): the battle-call GMEVENT proc and its
//! constructors, plus a callback runner and the Burmy form update. F1 counted these 7 functions in its fldmmdl.c
//! row; they are battle-event glue (F5b owns `event_battle.c` from `0x0216E95C`).
//!
//! F4a-spec specced all 7 (`tools/rom-diff/specs/black-field/event_battle_head.spec`, whose header documents the
//! battle-call work); F4a ported them. Every callee (GMEVENT, GAMESYS / GAMEDATA accessors, btl_setup, the
//! battle-event work init `0216F02C` of the next area, overlay-21 field functions) is reached through
//! [`FieldEnv::call`] by address; the works the GMEVENTs hand out are read and written through the [`Mem`] window.
//!
//! BATTLE CALL work (0x10, built by [`battle_call_new`]): `+0` gsys, `+4` BATTLE_SETUP, `+8` flag (1 = end right after
//! the battle), `+0xC`. Child work (0x28, proc `0x0216EB99`, init `0216F02C`): `+0x14`, `+0x18`, `+0x1C`, `+0x20`, `+0x24`,
//! `+8` -> an object whose `+0x18 u16` [`battle_event_new_b`] sets to 0x46B.
//!
//! [`Mem`]: crate::hooks::Mem

use crate::hooks::FieldEnv;

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_E62C,
    0x0216_E680,
    0x0216_E68C,
    0x0216_E7C8,
    0x0216_E818,
    0x0216_E8CC,
    0x0216_E910,
];

/// The battle-call GMEVENT proc (`FUN_0216E68C`, Thumb address).
pub const BATTLE_CALL_PROC: u32 = 0x0216_E68D;
/// The battle child GMEVENT proc (`FUN_0216EB98` of `event_battle.c`, Thumb address).
pub const BATTLE_PROC: u32 = 0x0216_EB99;

/// C: `FUN_0200CA1C(x)`: the pokedex / save object of `x`.
const DEX_OF: u32 = 0x0200_CA1C;
/// C: `FUN_0200CFA0(dex, mon)`: register `mon` in the pokedex.
const DEX_REGISTER: u32 = 0x0200_CFA0;
/// C: `FUN_02008DF0(rec, id)` (thunk to `FUN_02008DAC(rec, id, 1)`): add 1 to record counter `id`.
const RECORD_ADD: u32 = 0x0200_8DF0;
/// C: `FUN_020120F4(gsys)`: the GAMEDATA of the game system.
const GAMEDATA: u32 = 0x0201_20F4;
/// C: `FUN_0201210C(gsys)`: the FIELDMAP of the game system.
const FIELDMAP: u32 = 0x0201_210C;
/// C: `FUN_020122C0(gsys, parent, proc, work size)`: create a GMEVENT.
const EVENT_NEW: u32 = 0x0201_22C0;
/// C: `FUN_0201235C(event, next)`: change the GMEVENT to `next`.
const EVENT_CHANGE: u32 = 0x0201_235C;
/// C: `FUN_02012374(event, child)`: call the child GMEVENT.
const EVENT_CALL: u32 = 0x0201_2374;
/// C: `FUN_020124E8(event)`: the work of a GMEVENT.
const EVENT_WORK: u32 = 0x0201_24E8;
/// C: `FUN_02012A44(gd)`, `FUN_02012F20(gd)`, `FUN_02012F2C(gd)`: GAMEDATA getters.
const GD_A44: u32 = 0x0201_2A44;
const GD_F20: u32 = 0x0201_2F20;
const GD_F2C: u32 = 0x0201_2F2C;
/// C: `FUN_02013090(heap)` (`btl_setup::alloc`) and `FUN_02013144(setup, flags)` (`btl_setup::or_flags`).
const SETUP_ALLOC: u32 = 0x0201_3090;
const SETUP_OR_FLAGS: u32 = 0x0201_3144;
/// C: `FUN_0201313C(setup, 1)`: the party of a BATTLE_SETUP.
const SETUP_PARTY: u32 = 0x0201_313C;
/// C: `FUN_02017988(mon, form)`: set the form of a Pokemon.
const SET_FORM: u32 = 0x0201_7988;
/// C: `FUN_02017E1C(mon, 5, 0)`: PokeParam get (5 = species).
const POKE_GET: u32 = 0x0201_7E1C;
/// C: `FUN_0201AB28(party, i)`: party member `i`.
const PARTY_MEMBER: u32 = 0x0201_AB28;
/// The species that [`burmy_form`] updates (Burmy).
pub const BURMY: u32 = 0x19C;
/// ROM table `0x02172554`: `(battle background, Burmy form)` (25 entries).
pub const BURMY_FORMS: [(u16, u16); 25] = [
    (0, 1),
    (1, 1),
    (2, 2),
    (3, 2),
    (4, 1),
    (5, 0),
    (6, 0),
    (7, 1),
    (8, 0),
    (9, 0),
    (10, 1),
    (11, 1),
    (12, 0),
    (13, 0),
    (14, 2),
    (15, 2),
    (16, 2),
    (17, 2),
    (18, 2),
    (19, 2),
    (20, 2),
    (21, 1),
    (22, 0),
    (23, 0),
    (24, 2),
];

/// C: `FUN_0216E62C` (unlabeled routine at `0x0216E62C`): after a battle on background `bg`: when `mon` is a Burmy
/// and `bg` is in [`BURMY_FORMS`] (the first entry with that background, compared with the whole register), set its
/// form and register it in the pokedex of `x` (`FUN_0200CA1C(x)`). No call reaches this function in the field set.
pub fn burmy_form<E: FieldEnv>(env: &mut E, x: u32, mon: u32, bg: u32) {
    let dex = env.callv(DEX_OF, &[x]);
    let species = env.callv(POKE_GET, &[mon, 5, 0]);
    for &(b, form) in BURMY_FORMS.iter() {
        if bg == b as u32 {
            if species == BURMY {
                env.callv(SET_FORM, &[mon, form as u32]);
                env.callv(DEX_REGISTER, &[dex, mon]);
            }
            return;
        }
    }
}

/// C: `FUN_0216E680` (unlabeled routine at `0x0216E680`): runs the callback `f` when it is not null (with `f` in
/// r0), then returns `!f` (the original's `mvns r0, r4`; the function is referenced from the word at `0x0216EDD8`).
pub fn run_callback<E: FieldEnv>(env: &mut E, f: u32) -> u32 {
    if f != 0 {
        env.callv(f & !1, &[f]);
    }
    !f
}

/// C: `FUN_0216E68C` (unlabeled routine at `0x0216E68C`, the GMEVENT proc [`BATTLE_CALL_PROC`]): the battle call
/// at step `*seq` of the BATTLE CALL work `w`. 0: create the battle child event (`BATTLE_PROC`, 0x28) and init its work
/// (`FUN_0216F02C(work, gsys, setup)`, `+0x14 = 1`), `FUN_021CF5D8(species of party mon 0, w+0xC, setup +4 == 1,
/// fieldmap, work+0x24, *(work+8)+0x18)`, call it; 1: flag 1: `FUN_0202E79C(FUN_02012F20(gd))`, done; else
/// `FUN_0216EFA8(FUN_02012A44(gd), 0) == 1`: the same `0202E79C` call and change to `FUN_0215F6E8(gsys)`, otherwise
/// call `FUN_021BE92C(gsys, fieldmap, 0, 0, 1, 0, 0)`; 2: `FUN_0202E818(FUN_02012F20(gd)) == 1`: call
/// `FUN_02181D8C(gsys, 4)`; 3: `0202E79C` call, done. Steps 0..2 move on (`*seq + 1`) and return 0; done = 1; other
/// steps return 0.
pub fn battle_call_proc<E: FieldEnv>(env: &mut E, event: u32, seq: u32, w: u32) -> u32 {
    let gsys = env.r32(w);
    let gd = env.callv(GAMEDATA, &[gsys]);
    match env.r32(seq) {
        0 => {
            let child = env.callv(EVENT_NEW, &[gsys, 0, BATTLE_PROC, 0x28]);
            let work = env.callv(EVENT_WORK, &[child]);
            let setup = env.r32(w + 4);
            env.callv(0x0216_F02C, &[work, gsys, setup]);
            env.w32(work + 0x14, 1);
            let setup = env.r32(w + 4);
            let party = env.callv(SETUP_PARTY, &[setup, 1]);
            let mon = env.callv(PARTY_MEMBER, &[party, 0]);
            let species = env.callv(POKE_GET, &[mon, 5, 0]);
            let fm = env.callv(FIELDMAP, &[gsys]);
            let setup = env.r32(w + 4);
            let flag = (env.r32(setup + 4) == 1) as u32;
            let x = env.r32(work + 8).wrapping_add(0x18);
            let c = env.r32(w + 0xC);
            env.callv(0x021C_F5D8, &[species, c, flag, fm, work + 0x24, x]);
            env.callv(EVENT_CALL, &[event, child]);
        }
        1 => {
            if env.r32(w + 8) == 1 {
                let r = env.callv(GD_F20, &[gd]);
                env.callv(0x0202_E79C, &[r]);
                return 1;
            }
            let a = env.callv(GD_A44, &[gd]);
            if env.callv(0x0216_EFA8, &[a, 0]) == 1 {
                let r = env.callv(GD_F20, &[gd]);
                env.callv(0x0202_E79C, &[r]);
                let next = env.callv(0x0215_F6E8, &[gsys]);
                env.callv(EVENT_CHANGE, &[event, next]);
            } else {
                let fm = env.callv(FIELDMAP, &[gsys]);
                let child = env.callv(0x021B_E92C, &[gsys, fm, 0, 0, 1, 0, 0]);
                env.callv(EVENT_CALL, &[event, child]);
            }
        }
        2 => {
            let r = env.callv(GD_F20, &[gd]);
            if env.callv(0x0202_E818, &[r]) == 1 {
                let child = env.callv(0x0218_1D8C, &[gsys, 4]);
                env.callv(EVENT_CALL, &[event, child]);
            }
        }
        3 => {
            let r = env.callv(GD_F20, &[gd]);
            env.callv(0x0202_E79C, &[r]);
            return 1;
        }
        _ => return 0,
    }
    let s = env.r32(seq).wrapping_add(1);
    env.w32(seq, s);
    0
}

/// C: `FUN_0216E7C8` (unlabeled routine at `0x0216E7C8`): create the battle-call GMEVENT ([`BATTLE_CALL_PROC`],
/// work 0x10 = `{gsys, setup, flag, z}`), `FUN_0216EE40(setup, 0)`, record counter 5, `FUN_021AAABC(FUN_02188CE8(fm))`;
/// returns the event.
pub fn battle_call_new<E: FieldEnv>(
    env: &mut E,
    gsys: u32,
    fm: u32,
    setup: u32,
    flag: u32,
    z: u32,
) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gsys, 0, BATTLE_CALL_PROC, 0x10]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.w32(w, gsys);
    env.w32(w + 4, setup);
    env.w32(w + 8, flag);
    env.w32(w + 0xC, z);
    env.callv(0x0216_EE40, &[setup, 0]);
    let gd = env.callv(GAMEDATA, &[gsys]);
    let rec = env.callv(GD_F2C, &[gd]);
    env.callv(RECORD_ADD, &[rec, 5]);
    let x = env.callv(0x0218_8CE8, &[fm]);
    env.callv(0x021A_AABC, &[x]);
    ev
}

/// C: `FUN_0216E818` (unlabeled routine at `0x0216E818`): create a battle child GMEVENT ([`BATTLE_PROC`]) with a new
/// BATTLE_SETUP (`btl_setup::alloc(4)`) filled by `FUN_021AA5CC(FUN_02188CE8(fm), setup, a, b, c, d, 4)`;
/// `FUN_0216EE20((u16) c, 0)`, record counters 6 and 0x56, the work init (`+0x14 = 1`; `e & 1`: setup flag 0x100 and
/// `+0x18 = 1`), `FUN_021CF748(c, fm, work+0x24, *(work+8)+0x18)`, `FUN_021AAABC(FUN_02188CE8(fm))`; returns the event.
#[allow(clippy::too_many_arguments)]
pub fn battle_event_new<E: FieldEnv>(
    env: &mut E,
    gsys: u32,
    fm: u32,
    a: u32,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
) -> u32 {
    let ev = env.callv(EVENT_NEW, &[gsys, 0, BATTLE_PROC, 0x28]);
    let w = env.callv(EVENT_WORK, &[ev]);
    let f = env.callv(0x0218_8CE8, &[fm]);
    let gd = env.callv(GAMEDATA, &[gsys]);
    let rec = env.callv(GD_F2C, &[gd]);
    let setup = env.callv(SETUP_ALLOC, &[4]);
    env.callv(0x021A_A5CC, &[f, setup, a, b, c, d, 4]);
    env.callv(0x0216_EE20, &[c & 0xFFFF, 0]);
    env.callv(RECORD_ADD, &[rec, 6]);
    env.callv(RECORD_ADD, &[rec, 0x56]);
    env.callv(0x0216_F02C, &[w, gsys, setup]);
    env.w32(w + 0x14, 1);
    if e & 1 != 0 {
        env.callv(SETUP_OR_FLAGS, &[setup, 0x100]);
        env.w32(w + 0x18, 1);
    }
    let x = env.r32(w + 8).wrapping_add(0x18);
    env.callv(0x021C_F748, &[c, fm, w + 0x24, x]);
    let f = env.callv(0x0218_8CE8, &[fm]);
    env.callv(0x021A_AABC, &[f]);
    ev
}

/// The shared head of [`battle_event_new_a`] / [`battle_event_new_b`]: the child event and its work init with
/// `+0x14 = 0`, `+0x18 = 1`, `+0x24 = 9`.
fn battle_event_with<E: FieldEnv>(env: &mut E, gsys: u32, setup: u32) -> (u32, u32) {
    let ev = env.callv(EVENT_NEW, &[gsys, 0, BATTLE_PROC, 0x28]);
    let w = env.callv(EVENT_WORK, &[ev]);
    env.callv(0x0216_F02C, &[w, gsys, setup]);
    env.w32(w + 0x14, 0);
    (ev, w)
}

/// C: `FUN_0216E8CC` (unlabeled routine at `0x0216E8CC`): a battle child GMEVENT for `setup` with work `+0x14 = 0`,
/// `+0x18 = 1`, `+0x20 = 1`, `+0x24 = 9`; `FUN_021AAABC(FUN_02188CE8(fm))`; returns the event.
pub fn battle_event_new_a<E: FieldEnv>(env: &mut E, gsys: u32, fm: u32, setup: u32) -> u32 {
    let (ev, w) = battle_event_with(env, gsys, setup);
    env.w32(w + 0x18, 1);
    env.w32(w + 0x20, 1);
    env.w32(w + 0x24, 9);
    let f = env.callv(0x0218_8CE8, &[fm]);
    env.callv(0x021A_AABC, &[f]);
    ev
}

/// C: `FUN_0216E910` (unlabeled routine at `0x0216E910`): as [`battle_event_new_a`] with `+0x1C = 1` instead of
/// `+0x20`, and `u16 *(work+8)+0x18 = 0x46B`.
pub fn battle_event_new_b<E: FieldEnv>(env: &mut E, gsys: u32, fm: u32, setup: u32) -> u32 {
    let (ev, w) = battle_event_with(env, gsys, setup);
    env.w32(w + 0x1C, 1);
    env.w32(w + 0x18, 1);
    env.w32(w + 0x24, 9);
    let o = env.r32(w + 8);
    env.w16(o + 0x18, 0x46B);
    let f = env.callv(0x0218_8CE8, &[fm]);
    env.callv(0x021A_AABC, &[f]);
    ev
}

#[cfg(test)]
mod tests;
