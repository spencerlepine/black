//! `symbol_map.c` (overlay 10, `0x021664CC..0x02166F9C`, F1 row 10.27; F5b): the functions of the file that are not
//! script-command handlers (those 17 are black-script `scrcmd_misc10b`): the symbol (Entree Forest) record of the
//! current forest, the symbol-kind predicates, the egg-hatch GMEVENT, the zone remap and the special event flags.
//!
//! Spec: `tools/rom-diff/specs/black-field/symbol_map.spec` (its header has the record bit fields and the hatch work).
//! D9: the Entralink side (the intrude state and comm record of overlay 11, game_beacon) is reached only through
//! [`FieldEnv::call`] by address, like every other callee; the local logic is ported. Not ported (render, U-phase, D3):
//! `FUN_02166A6C` / `FUN_02166B0C` / `FUN_02166B2C`, a V-blank brightness fade through the BG2 registers.

use crate::hooks::{CArg, FieldEnv};

/// Functions of this area that are ported (read by `tools/rom-diff/svflow.py --ported-src`).
pub const PORTED: &[u32] = &[
    0x0216_64CC,
    0x0216_65D0,
    0x0216_65F0,
    0x0216_6618,
    0x0216_662C,
    0x0216_6640,
    0x0216_66A4,
    0x0216_6B58,
    0x0216_6C4C,
    0x0216_6C7C,
    0x0216_6CD0,
];

/// The render-only functions of the file that are not ported (U-phase): the V-blank fade callback, its start and stop.
pub const RENDER_ONLY: &[u32] = &[0x0216_6A6C, 0x0216_6B0C, 0x0216_6B2C];

/// Address of the string `"symbol_map.c"` (the file argument of the allocations in [`symbol_record`]).
pub const FILE_SYMBOL_MAP: u32 = 0x0217_29EC;
/// The hatch GMEVENT proc [`hatch_proc`] (Thumb address).
pub const HATCH_PROC: u32 = 0x0216_6B59;
/// The zone that [`zone_group`] resolves through the current location.
pub const ZONE_FROM_LOCATION: u32 = 0x1A6;
/// ROM table `0x02171F2C`: `(result, key)` pairs of [`zone_group`].
pub const ZONE_REMAP: [(u32, u32); 2] = [(0x88, 0x108), (0xF0, 0xF1)];

/// `FUN_020120F4(gsys)`: the GAMEDATA of the game system.
const GAMEDATA: u32 = 0x0201_20F4;
/// `FUN_02012374(event, child)`: call the child GMEVENT.
const EVENT_CALL: u32 = 0x0201_2374;
/// `FUN_0216649C(x)`: the game-beacon record of `x` (X1).
const BEACON_REC: u32 = 0x0216_649C;
/// `FUN_02013E40(zone)`: zonedata zone group.
const ZONE_GROUP: u32 = 0x0201_3E40;

/// C: `FUN_021664CC`: the symbol record (0x54 bytes on `heap`) of the forest being visited. Own forest (intrude state
/// `FUN_021767A0(I, gd) == 4`): filled by `FUN_0216749C(Y, rec, 20, x, &k)` (whose result goes to `*out`) and `+0x50`
/// packed as in the spec header; another player's forest: a copy of the comm record (`FUN_02176880(FUN_02158344(I))`,
/// 0 when there is none) and `*out = 0x212`. Returns the record or 0.
pub fn symbol_record<E: FieldEnv>(env: &mut E, heap: u32, gsys: u32, out: u32) -> u32 {
    let gd = env.callv(GAMEDATA, &[gsys]);
    let save = env.callv(0x0201_2EBC, &[gd]);
    let y = env.callv(0x0200_E0B0, &[save]);
    let i = env.callv(0x0201_2138, &[gsys]);
    if env.callv(0x0217_67A0, &[i, gd]) == 4 {
        let x = env.callv(0x0201_2FB0, &[gd]);
        let rec = env.callv(0x0203_0734, &[heap, 0x54, 1, FILE_SYMBOL_MAP, 0x1C9]);
        let mut k = [0u8; 1];
        let r = env.call(
            0x0216_749C,
            &mut [
                CArg::V(y),
                CArg::V(rec),
                CArg::V(20),
                CArg::V(x),
                CArg::OutBuf(&mut k),
            ],
        );
        env.w32(out, r);
        let mut t = env.r32(rec + 0x50);
        t = (t & !0x3F) | (k[0] as u32 & 0x3F);
        env.w32(rec + 0x50, t);
        let v = env.callv(0x0216_73C4, &[y]);
        let t = ((v & 0xF) << 6) | (env.r32(rec + 0x50) & 0xFFFF_FC3F);
        env.w32(rec + 0x50, t);
        let w = env.callv(0x0216_73A8, &[y]);
        let mut t = (env.r32(rec + 0x50) & 0xFFFF_C3FF) | ((w & 0xF) << 10);
        t &= 0xFFFC_3FFF;
        t |= 0x1_0000;
        t &= 0xFF03_FFFF;
        t |= (x & 0x3F) << 18;
        env.w32(rec + 0x50, t);
        return rec;
    }
    let p = env.callv(0x0215_8344, &[i]);
    if p == 0 {
        return 0;
    }
    let rec = env.callv(0x0203_0734, &[heap, 0x54, 1, FILE_SYMBOL_MAP, 0x1DC]);
    let src = env.callv(0x0217_6880, &[p]);
    env.mem().copy(rec, src, 0x54);
    env.w32(out, 0x1DC + 0x36);
    rec
}

/// C: `FUN_021665D0`: `FUN_02166324(p, (u8)(a + 1))` of the beacon record `p = FUN_0216649C(x)`, 0x118 when there is
/// none.
pub fn beacon_value<E: FieldEnv>(env: &mut E, x: u32, a: u32) -> u32 {
    let p = env.callv(BEACON_REC, &[x]);
    if p == 0 {
        return 0x118;
    }
    env.callv(0x0216_6324, &[p, a.wrapping_add(1) & 0xFF])
}

/// C: `FUN_021665F0`: `(u8) FUN_021663F4(p, (u8)(a + 1), (u8) b) - 1` of the beacon record `p = FUN_0216649C(x)`
/// (0xFFFFFFFF for 0), 4 when there is none.
pub fn beacon_index<E: FieldEnv>(env: &mut E, x: u32, a: u32, b: u32) -> u32 {
    let p = env.callv(BEACON_REC, &[x]);
    if p == 0 {
        return 4;
    }
    let r = env.callv(0x0216_63F4, &[p, a.wrapping_add(1) & 0xFF, b & 0xFF]);
    (r & 0xFF).wrapping_sub(1)
}

/// C: `FUN_02166618`: symbol kind `x` is 0 (`(u8)(x + 1) == 1`).
pub fn kind_is_0(x: u32) -> u32 {
    (x.wrapping_add(1) & 0xFF == 1) as u32
}

/// C: `FUN_0216662C`: symbol kind `x` is 4 (`(u8)(x + 1) == 5`).
pub fn kind_is_4(x: u32) -> u32 {
    (x.wrapping_add(1) & 0xFF == 5) as u32
}

/// C: `FUN_02166640`: symbol kind `x` is 1..3 (`(u8)((u8)(x + 1) - 2) <= 2`).
pub fn kind_is_1_to_3(x: u32) -> u32 {
    let a = x.wrapping_add(1) & 0xFF;
    (a.wrapping_add(0xFE) & 0xFF <= 2) as u32
}

/// C: `FUN_021666A4`: free the block `*p` points to (if any) and clear `*p`.
pub fn free_ptr<E: FieldEnv>(env: &mut E, p: u32) {
    let b = env.r32(p);
    if b != 0 {
        env.callv(0x0203_07B0, &[b]);
        env.w32(p, 0);
    }
}

/// C: `FUN_02166B58`: the hatch GMEVENT proc ([`HATCH_PROC`]) at step `*seq` of work `w` (steps in the spec): checks the
/// egg (PokeParam 0x4C, 3), fades out (`FUN_021BE8D4`), runs `FUN_020144DC`, the hatch app 0xD7 with `{gd, mon}`,
/// `FUN_020145CC`, fades in (`FUN_021BE92C`) and notifies the beacon of the species. Returns 1 at step 7.
pub fn hatch_proc<E: FieldEnv>(env: &mut E, ev: u32, seq: u32, w: u32) -> u32 {
    let gsys = env.r32(w);
    let gd = env.r32(w + 4);
    let fm = env.r32(w + 8);
    let next = match env.r32(seq) {
        0 => {
            let mon = env.r32(w + 0xC);
            if env.callv(0x0201_7E1C, &[mon, 0x4C, 0]) == 0 {
                7
            } else {
                let mon = env.r32(w + 0xC);
                if env.callv(0x0201_7E1C, &[mon, 3, 0]) == 1 {
                    7
                } else {
                    1
                }
            }
        }
        1 => {
            env.w32(w + 0x10, gd);
            let mon = env.r32(w + 0xC);
            env.w32(w + 0x14, mon);
            2
        }
        2 => {
            let child = env.callv(0x021B_E8D4, &[gsys, fm, 0, 0]);
            env.callv(EVENT_CALL, &[ev, child]);
            3
        }
        3 => {
            let child = env.callv(0x0201_44DC, &[gsys, fm]);
            env.callv(EVENT_CALL, &[ev, child]);
            4
        }
        4 => {
            env.callv(0x0201_20B4, &[gsys, 0xD7, 0x021F_7118, w + 0x10]);
            5
        }
        5 => {
            let child = env.callv(0x0201_45CC, &[gsys]);
            env.callv(EVENT_CALL, &[ev, child]);
            6
        }
        6 => {
            let child = env.callv(0x021B_E92C, &[gsys, fm, 0, 0, 1, 0, 0]);
            env.callv(EVENT_CALL, &[ev, child]);
            let mon = env.r32(w + 0xC);
            let species = env.callv(0x0201_7E1C, &[mon, 5, 0]) & 0xFFFF;
            env.callv(0x0216_559C, &[species]);
            7
        }
        7 => return 1,
        _ => return 0,
    };
    env.w32(seq, next);
    0
}

/// C: `FUN_02166C4C`: the hatch GMEVENT ([`HATCH_PROC`], work 0x18 = `{gsys, gd, fm, mon, ..}`) for Pokemon `mon`.
pub fn hatch_event_new<E: FieldEnv>(env: &mut E, gsys: u32, fm: u32, mon: u32) -> u32 {
    let ev = env.callv(0x0201_22C0, &[gsys, 0, HATCH_PROC, 0x18]);
    let w = env.callv(0x0201_24E8, &[ev]);
    env.w32(w, gsys);
    env.w32(w + 8, fm);
    let gd = env.callv(GAMEDATA, &[gsys]);
    env.w32(w + 4, gd);
    env.w32(w + 0xC, mon);
    ev
}

/// C: `FUN_02166C7C`: the zone group of `zone` (`z = FUN_02013E40(zone)`): [`ZONE_FROM_LOCATION`] -> the group of the
/// current location's zone (`u16` at `FUN_02012894(gd) + 4`); keys of [`ZONE_REMAP`] -> their result; else `z`.
pub fn zone_group<E: FieldEnv>(env: &mut E, gd: u32, zone: u32) -> u32 {
    let z = env.callv(ZONE_GROUP, &[zone]);
    if z == ZONE_FROM_LOCATION {
        let loc = env.callv(0x0201_2894, &[gd]);
        let lz = env.r16(loc + 4) as u32;
        return env.callv(ZONE_GROUP, &[lz]);
    }
    match ZONE_REMAP.iter().find(|e| e.1 == z) {
        Some(&(r, _)) => r & 0xFFFF,
        None => z,
    }
}

/// C: `FUN_02166CD0`: event flag `flag` with two specials: 0xF000 -> `FUN_0216BF9C(FUN_02012F08(gd), 1, MYSTATUS)`,
/// 0xF001 -> `FUN_02009590(FUN_02009470(save), MYSTATUS, 0) != 0`; else `FUN_020142E8(EVENT WORK, flag)`.
pub fn check_flag<E: FieldEnv>(env: &mut E, gd: u32, flag: u32) -> u32 {
    let ew = env.callv(0x0201_296C, &[gd]);
    let x = env.callv(0x0201_2F08, &[gd]);
    let my = env.callv(0x0201_2944, &[gd]);
    let save = env.callv(0x0201_2EBC, &[gd]);
    let s = env.callv(0x0200_9470, &[save]);
    match flag {
        0xF000 => env.callv(0x0216_BF9C, &[x, 1, my]),
        0xF001 => (env.callv(0x0200_9590, &[s, my, 0]) != 0) as u32,
        _ => env.callv(0x0201_42E8, &[ew, flag]),
    }
}

#[cfg(test)]
mod tests;
