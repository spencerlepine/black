//! `btl_client.c` part 3 (0x021CF5B0-0x021D10B8): move-selection checks, the trainer-AI switch heuristics, the AI action step,
//! the switch-in steps and their party/target helpers (B13-spec1 / B13-spec2; ported B13a).
//!
//! Specs: `tools/rom-diff/svflow_specs/client3.spec` (`nowork`, the client object is `arg buf 0x1C0`, a step's state word
//! `arg buf 0x10`). The ports take the object as `c` (with its address `c_addr` where the original stores an address into it).
//!
//! The AI heuristics draw from a 64-bit LCG kept in the object ([`lcg_next`]: state `+0xF4`, multiplier `+0xFC`, increment
//! `+0x104`; the original steps it through the 64-bit multiply `0209c054`). Callees: battle-view calls and the view-side object
//! (`c+0x34`) through [`ClientUi`] (D3 / D7); BattleMon accessors, the battle server's getters, the waza-data accessors
//! (`0201xxxx`), the action-record helpers (`021d8xxx`), the overlay-96 trainer AI, the veneers and the callees in other client
//! areas (client2 `021ce068 021ce08c 021ce0e0 021ce1b0 021cf470 021cf524 021cf58c`, client5 `021d3dc4`) through
//! [`FlowEnv::call`].
//!
//! Pointers the adapter returns (`021eec20`, the switch-request list of [`switch_requests`]) are resolved in the block given as
//! `list` (at address `list_addr`); the specs place that block in an extra buffer argument.

use super::{rd32, wr32, ClientUi};
use crate::server_flow::env::{Arg, FlowEnv, Lend};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021C_F5B0,
    0x021C_F81C,
    0x021C_F89C,
    0x021C_F8D4,
    0x021C_F918,
    0x021C_F9B4,
    0x021C_FAC4,
    0x021C_FADC,
    0x021C_FB8C,
    0x021C_FBA4,
    0x021C_FBBC,
    0x021C_FBE0,
    0x021C_FBF0,
    0x021C_FD50,
    0x021C_FDCC,
    0x021C_FE08,
    0x021C_FE98,
    0x021C_FFAC,
    0x021D_0100,
    0x021D_0248,
    0x021D_0304,
    0x021D_04E4,
    0x021D_055C,
    0x021D_05C4,
    0x021D_0644,
    0x021D_06C8,
    0x021D_0770,
    0x021D_0980,
    0x021D_09E0,
    0x021D_0B10,
    0x021D_0B6C,
    0x021D_0BB0,
    0x021D_0C78,
    0x021D_0CC4,
    0x021D_0D38,
    0x021D_0D44,
    0x021D_0D50,
    0x021D_0E74,
    0x021D_0E7C,
    0x021D_102C,
    0x021D_1058,
];

const OFS_SERVER: usize = 0x00;
const OFS_POKECON: usize = 0x04;
const OFS_MON: usize = 0x08;
const OFS_REC: usize = 0x0C;
const OFS_VIEW_OBJ: usize = 0x34;
const OFS_ADAPTER: usize = 0x50;
const OFS_VIEW: usize = 0x54;
const OFS_RESULT_PTR: usize = 0xD8;
const OFS_RESULT_SIZE: usize = 0xDC;
const OFS_AI: usize = 0xF0;
const OFS_LCG_STATE: usize = 0xF4;
const OFS_LCG_MUL: usize = 0xFC;
const OFS_LCG_INC: usize = 0x104;
const OFS_SLOTS: usize = 0x10C;
const SLOT_COUNT: usize = 6;
const OFS_PARTY: usize = 0x118;
const OFS_PARTY_COUNT: usize = 0x11C;
const OFS_CUR: usize = 0x11D;
const OFS_ANSWERS: usize = 0x124;
const OFS_PARTY_BLOCK: usize = 0x184;
const OFS_PARTY_SEL: usize = 0x190;
const OFS_ID: usize = 0x1A6;
const OFS_1A9: usize = 0x1A9;
const OFS_YESNO: usize = 0x1AC;
const OFS_1AF: usize = 0x1AF;
const OFS_REQ_COUNT: usize = 0x1B3;
const OFS_REQ_LIST: usize = 0x1B5;

/// The struggle move id.
const MOVE_STRUGGLE: u32 = 0xA5;

/// The absorb-ability table at `0x021EFFC2` (ROM data): 4 records `{u8 type, pad, u16 abilities[4]}`.
const ABSORB_TABLE: [(u8, [u16; 4]); 4] = [
    (0x0A, [0x0B, 0x72, 0x57, 0]),
    (0x0C, [0x0A, 0x4E, 0x1F, 0]),
    (0x0B, [0x41, 0, 0, 0]),
    (0x09, [0x12, 0, 0, 0]),
];

fn view(c: &[u8]) -> u32 {
    rd32(c, OFS_VIEW)
}

fn st(s: &[u8]) -> u32 {
    rd32(s, 0)
}

fn set_st(s: &mut [u8], v: u32) {
    wr32(s, 0, v);
}

fn rd64(b: &[u8], ofs: usize) -> u64 {
    rd32(b, ofs) as u64 | (rd32(b, ofs + 4) as u64) << 32
}

/// One step of the AI's 64-bit LCG (`state = mul * state + inc`, all 64-bit, wrapping); returns the new high word.
pub fn lcg_next(c: &mut [u8]) -> u32 {
    let s = rd64(c, OFS_LCG_MUL)
        .wrapping_mul(rd64(c, OFS_LCG_STATE))
        .wrapping_add(rd64(c, OFS_LCG_INC));
    wr32(c, OFS_LCG_STATE, s as u32);
    wr32(c, OFS_LCG_STATE + 4, (s >> 32) as u32);
    (s >> 32) as u32
}

/// `(hi * n) >> 32`: a draw below `n` from the LCG's high word.
fn scaled(hi: u32, n: u32) -> u32 {
    ((hi as u64 * n as u64) >> 32) as u32
}

/// `021b9c00(party +0x118, i)`: the mon in party slot `i`.
fn party_mon<E: FlowEnv>(env: &mut E, c: &[u8], i: u32) -> u32 {
    env.callr(
        0x021B_9C00,
        &mut [Arg::V(rd32(c, OFS_PARTY)), Arg::V(i & 0xFF)],
        c,
    )
}

/// `021d5b68(mon)`: is the mon fainted?
fn fainted<E: FlowEnv>(env: &mut E, mon: u32) -> bool {
    env.call(0x021D_5B68, &mut [Arg::V(mon)]) != 0
}

/// `021d62c4(mon, flag)`.
fn flag<E: FlowEnv>(env: &mut E, mon: u32, f: u32) -> bool {
    env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(f)]) != 0
}

/// `BattleMon_Get(mon, field)`.
fn mon_get<E: FlowEnv>(env: &mut E, mon: u32, field: u32) -> u32 {
    env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(field)])
}

/// `021d5440(mon)`: the mon's battle position / name id.
fn mon_id<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    env.call(0x021D_5440, &mut [Arg::V(mon)])
}

/// `021efce4(view object, k)`: a battle-rule query on the view-side object.
fn rule<E: ClientUi>(env: &mut E, c: &[u8], k: u32) -> bool {
    env.view_obj(0x021E_FCE4, &mut [Arg::V(rd32(c, OFS_VIEW_OBJ)), Arg::V(k)]) != 0
}

/// The first party slot the client may send out: `021d7f2c(battle type, c[0x11C])`.
fn first_slot<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    let t = env.callr(0x021B_8588, &mut [Arg::V(rd32(c, OFS_SERVER))], c);
    env.callr(
        0x021D_7F2C,
        &mut [Arg::V(t), Arg::V(c[OFS_PARTY_COUNT] as u32)],
        c,
    )
}

/// `021b9b94(party)`: the party size.
fn party_size<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    env.callr(0x021B_9B94, &mut [Arg::V(rd32(c, OFS_PARTY))], c)
}

/// A "move cannot be chosen" message: `021eaf18(buf, kind, id)`, `021eaf50(buf, getter(mon))` (`getter` = `021d5440` or
/// `021d5b40`), `021eaf50(buf, b)`.
fn blocked_msg<E: ClientUi>(env: &mut E, kind: u32, id: u32, getter: u32, mon: u32, b: u32) {
    env.view(0x021E_AF18, &mut [Arg::P, Arg::V(kind), Arg::V(id)]);
    let a = env.call(getter, &mut [Arg::V(mon)]);
    env.view(0x021E_AF50, &mut [Arg::P, Arg::V(a)]);
    env.view(0x021E_AF50, &mut [Arg::P, Arg::V(b)]);
}

/// C: `FUN_overlay_d_93__021cf5b0`: can `mon` not choose move `mv`? Struggle (0xA5) never is blocked. In order: flag 0x1B with
/// [`can_use`] and `m = 021e8b80(021d6314(mon, 0x1B))` usable (`021d5750(mon, m)`) and `m != mv` (message 0x63 with
/// `021d5b40(mon)` and `m`); flag 0x17 with `m = 021e8b80(021d6314(mon, 0x17)) != mv` (0x64, the mon and `m`); flag 0xB and
/// `!0201c314(mv)` (0x23B); flag 0xC and `021d67b8(mon) == mv` (0x244); flag 0xD and `021d62d8(mon, 0xD) == mv` (0x253);
/// flag 0xF and `0201beb8(mv, 0xC)` (0x37A); rule 3 and `021efc1c(view object, pokecon, mon, mv)` (0x24D); rule 2 and
/// `0201beb8(mv, 9)` (0x43E). A blocked move answers 1 and, with a message buffer `buf != 0`, fills it (kind 1 for the first
/// two, else 2; the mon (`021d5440`) and the move). Else 0.
pub fn move_blocked<E: ClientUi>(env: &mut E, c: &[u8], mon: u32, mv: u32, buf: u32) -> u32 {
    if mv == MOVE_STRUGGLE {
        return 0;
    }
    if can_use(env, c, mon) != 0 && flag(&mut Lend::r(env, c), mon, 0x1B) {
        let w = env.callr(0x021D_6314, &mut [Arg::V(mon), Arg::V(0x1B)], c);
        let m = env.callr(0x021E_8B80, &mut [Arg::V(w)], c);
        if env.callr(0x021D_5750, &mut [Arg::V(mon), Arg::V(m)], c) != 0 && m != mv {
            if buf != 0 {
                blocked_msg(&mut Lend::r(env, c), 1, 0x63, 0x021D_5B40, mon, m);
            }
            return 1;
        }
    }
    if flag(&mut Lend::r(env, c), mon, 0x17) {
        let w = env.callr(0x021D_6314, &mut [Arg::V(mon), Arg::V(0x17)], c);
        let m = env.callr(0x021E_8B80, &mut [Arg::V(w)], c);
        if mv != m {
            if buf != 0 {
                blocked_msg(&mut Lend::r(env, c), 1, 0x64, 0x021D_5440, mon, m);
            }
            return 1;
        }
    }
    if flag(&mut Lend::r(env, c), mon, 0xB) && env.callr(0x0201_C314, &mut [Arg::V(mv)], c) == 0 {
        if buf != 0 {
            blocked_msg(&mut Lend::r(env, c), 2, 0x23B, 0x021D_5440, mon, mv);
        }
        return 1;
    }
    if flag(&mut Lend::r(env, c), mon, 0xC) && env.callr(0x021D_67B8, &mut [Arg::V(mon)], c) == mv {
        if buf != 0 {
            blocked_msg(&mut Lend::r(env, c), 2, 0x244, 0x021D_5440, mon, mv);
        }
        return 1;
    }
    if flag(&mut Lend::r(env, c), mon, 0xD)
        && env.callr(0x021D_62D8, &mut [Arg::V(mon), Arg::V(0xD)], c) == mv
        && mv != MOVE_STRUGGLE
    {
        if buf != 0 {
            blocked_msg(&mut Lend::r(env, c), 2, 0x253, 0x021D_5440, mon, mv);
        }
        return 1;
    }
    if flag(&mut Lend::r(env, c), mon, 0xF)
        && env.callr(0x0201_BEB8, &mut [Arg::V(mv), Arg::V(0xC)], c) != 0
    {
        if buf != 0 {
            blocked_msg(&mut Lend::r(env, c), 2, 0x37A, 0x021D_5440, mon, mv);
        }
        return 1;
    }
    if rule(env, c, 3)
        && env.view_obj(
            0x021E_FC1C,
            &mut [
                Arg::V(rd32(c, OFS_VIEW_OBJ)),
                Arg::V(rd32(c, OFS_POKECON)),
                Arg::V(mon),
                Arg::V(mv),
            ],
        ) != 0
    {
        if buf != 0 {
            blocked_msg(&mut Lend::r(env, c), 2, 0x24D, 0x021D_5440, mon, mv);
        }
        return 1;
    }
    if rule(env, c, 2) && env.callr(0x0201_BEB8, &mut [Arg::V(mv), Arg::V(9)], c) != 0 {
        if buf != 0 {
            blocked_msg(&mut Lend::r(env, c), 2, 0x43E, 0x021D_5440, mon, mv);
        }
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021cf81c`: the usable-move mask of `mon` into `out`: for `j < 021d5448(mon)`, `out[j]` = the move
/// has PP (`021d5584(mon, j)`) and is not blocked ([`move_blocked`]); `out[j] = 0` up to 4. Returns the first usable `j`
/// (4 = none).
pub fn usable_moves<E: ClientUi>(env: &mut E, c: &[u8], mon: u32, out: &mut [u8]) -> u32 {
    let n = env.callr(0x021D_5448, &mut [Arg::V(mon)], c);
    let mut first = 4u32;
    let mut j = 0u32;
    while j < n {
        if env.callr(0x021D_5584, &mut [Arg::V(mon), Arg::V(j)], c) != 0 {
            let mv = env.callr(0x021D_54C0, &mut [Arg::V(mon), Arg::V(j)], c);
            out[j as usize] = (move_blocked(env, c, mon, mv, 0) == 0) as u8;
            if first == 4 && out[j as usize] != 0 {
                first = j;
            }
        } else {
            out[j as usize] = 0;
        }
        j = (j + 1) & 0xFF;
    }
    while j < 4 {
        out[j as usize] = 0;
        j += 1;
    }
    first
}

/// C: `FUN_overlay_d_93__021cf89c`: can `mon` use its held item / ability effects here? 0 under rule 7 (`021efce4(view object,
/// 7)`), with flag 0x13, or with ability 0x67 (`BattleMon_Get(mon, 0x11)`); else 1.
pub fn can_use<E: ClientUi>(env: &mut E, c: &[u8], mon: u32) -> u32 {
    if rule(env, c, 7) {
        return 0;
    }
    if flag(&mut Lend::r(env, c), mon, 0x13) {
        return 0;
    }
    (mon_get(&mut Lend::r(env, c), mon, 0x11) != 0x67) as u32
}

/// C: `FUN_overlay_d_93__021cf8d4`: can `mon` switch out? [`can_use`] and item 0x127 (`021d5b40`) -> 4 (free). Else
/// [`trap_check`]; its answer 4 sets `*outb = 0x1F`, `*outh = 0`. Returns the answer.
pub fn escape_check<E: ClientUi>(
    env: &mut E,
    c: &[u8],
    mon: u32,
    outb: &mut [u8],
    outh: &mut [u8],
) -> u32 {
    if can_use(env, c, mon) != 0 && env.callr(0x021D_5B40, &mut [Arg::V(mon)], c) == 0x127 {
        return 4;
    }
    let r = trap_check(env, c, mon, outb, outh);
    if r == 4 {
        outb[0] = 0x1F;
        outh[..2].copy_from_slice(&0u16.to_le_bytes());
        return 4;
    }
    r
}

/// C: `FUN_overlay_d_93__021cf918`: can the client run? Any live party mon (of the `c[0x11C]`) with [`can_use`] and item 0xE4,
/// or ability 0x32, -> 4. Else the first [`trap_check`] answer that is not 4; else 4.
pub fn run_check<E: ClientUi>(env: &mut E, c: &[u8], outb: &mut [u8], outh: &mut [u8]) -> u32 {
    let mut i = 0u32;
    while i < c[OFS_PARTY_COUNT] as u32 {
        let mon = party_mon(env, c, i);
        if !fainted(&mut Lend::r(env, c), mon) {
            if can_use(env, c, mon) != 0 && env.callr(0x021D_5B40, &mut [Arg::V(mon)], c) == 0xE4 {
                return 4;
            }
            if mon_get(&mut Lend::r(env, c), mon, 0x11) == 0x32 {
                return 4;
            }
        }
        i += 1;
    }
    let mut i = 0u32;
    while i < c[OFS_PARTY_COUNT] as u32 {
        let mon = party_mon(env, c, i);
        let r = trap_check(env, c, mon, outb, outh);
        if r != 4 {
            return r;
        }
        i += 1;
    }
    4
}

/// C: `FUN_overlay_d_93__021cf9b4`: is `mon` trapped? The opponents (`n = 021b8d38(server, pokecon, 021b8d8c(server, pokecon,
/// 021d5440(mon)) | 0x100, list)`, each `m = 021b9934(pokecon, list[i])`): ability (`BattleMon_Get(m, 0x11)`) 0x17 with
/// [`not_ability_17`]`(mon)`, 0x47 with [`arena_trap_test`]`(mon)` or 0x2A with [`flag8_test`]`(mon)` -> `*outb =
/// 021d5440(m)`, `*outh` = the ability, 0. Then flag 0x16, 8 or 0x15 on `mon` -> `*outb = 021d5440(mon)`, `*outh = 0`, 3.
/// Else 4.
pub fn trap_check<E: ClientUi>(
    env: &mut E,
    c: &[u8],
    mon: u32,
    outb: &mut [u8],
    outh: &mut [u8],
) -> u32 {
    let (server, pokecon) = (rd32(c, OFS_SERVER), rd32(c, OFS_POKECON));
    let id = mon_id(&mut Lend::r(env, c), mon);
    let side = env.callr(
        0x021B_8D8C,
        &mut [Arg::V(server), Arg::V(pokecon), Arg::V(id)],
        c,
    );
    let mut list = [0u8; 4];
    let n = env.callr(
        0x021B_8D38,
        &mut [
            Arg::V(server),
            Arg::V(pokecon),
            Arg::V((side | 0x100) & 0xFFFF),
            Arg::Out(&mut list),
        ],
        c,
    );
    let mut i = 0u32;
    while i < n {
        let m = env.callr(
            0x021B_9934,
            &mut [Arg::V(pokecon), Arg::V(list[i as usize] as u32)],
            c,
        );
        let ab = mon_get(&mut Lend::r(env, c), m, 0x11) & 0xFFFF;
        mon_id(&mut Lend::r(env, c), m);
        let hit = match ab {
            0x17 => not_ability_17(&mut Lend::r(env, c), mon) != 0,
            0x47 => arena_trap_test(env, c, mon) != 0,
            0x2A => flag8_test(&mut Lend::r(env, c), mon) != 0,
            _ => false,
        };
        if hit {
            outb[0] = mon_id(&mut Lend::r(env, c), m) as u8;
            outh[..2].copy_from_slice(&(ab as u16).to_le_bytes());
            return 0;
        }
        i = (i + 1) & 0xFF;
    }
    if flag(&mut Lend::r(env, c), mon, 0x16)
        || flag(&mut Lend::r(env, c), mon, 8)
        || flag(&mut Lend::r(env, c), mon, 0x15)
    {
        outb[0] = mon_id(&mut Lend::r(env, c), mon) as u8;
        outh[..2].copy_from_slice(&0u16.to_le_bytes());
        return 3;
    }
    4
}

/// C: `FUN_overlay_d_93__021cfac4`: `BattleMon_Get(mon, 0x11) != 0x17` (the original's first argument is unused).
pub fn not_ability_17<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    (mon_get(env, mon, 0x11) != 0x17) as u32
}

/// C: `FUN_overlay_d_93__021cfadc`: is `mon` grounded for an arena-trap opponent? `u = `[`can_use`]; rule 2 -> 1; flag 0x1F
/// or 0x15 -> 1; `u` and item 0x116 -> 1; ability 0x1A -> 0; `021d5814(mon, 2)` -> 0; flag 0x1E or 0x20 -> 0; `u` and item
/// 0x21D -> 0; else 1.
pub fn arena_trap_test<E: ClientUi>(env: &mut E, c: &[u8], mon: u32) -> u32 {
    let u = can_use(env, c, mon) != 0;
    if rule(env, c, 2) {
        return 1;
    }
    if flag(&mut Lend::r(env, c), mon, 0x1F) || flag(&mut Lend::r(env, c), mon, 0x15) {
        return 1;
    }
    if u && env.callr(0x021D_5B40, &mut [Arg::V(mon)], c) == 0x116 {
        return 1;
    }
    if mon_get(&mut Lend::r(env, c), mon, 0x11) == 0x1A {
        return 0;
    }
    if env.callr(0x021D_5814, &mut [Arg::V(mon), Arg::V(2)], c) != 0 {
        return 0;
    }
    if flag(&mut Lend::r(env, c), mon, 0x1E) || flag(&mut Lend::r(env, c), mon, 0x20) {
        return 0;
    }
    if u && env.callr(0x021D_5B40, &mut [Arg::V(mon)], c) == 0x21D {
        return 0;
    }
    1
}

/// C: `FUN_overlay_d_93__021cfb8c`: `021d5814(mon, 8) != 0` (the original's first argument is unused).
pub fn flag8_test<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    (env.call(0x021D_5814, &mut [Arg::V(mon), Arg::V(8)]) != 0) as u32
}

/// C: `FUN_overlay_d_93__021cfba4`: the six slot bytes at `+0x10C` = 0xFF (empty).
pub fn clear_slots(c: &mut [u8]) {
    c[OFS_SLOTS..OFS_SLOTS + SLOT_COUNT].fill(0xFF);
}

/// C: `FUN_overlay_d_93__021cfbbc`: is `id` one of the six signed bytes at `+0x10C` (0xFF = empty slot, i.e. -1).
pub fn slot_holds(c: &[u8], id: i32) -> bool {
    (0..SLOT_COUNT).any(|i| c[OFS_SLOTS + i] as i8 as i32 == id)
}

/// C: `FUN_overlay_d_93__021cfbe0`: `c[0x10C + i]` = `v` for `i < 6`.
pub fn set_slot(c: &mut [u8], i: u32, v: u32) {
    if i < SLOT_COUNT as u32 {
        c[OFS_SLOTS + i as usize] = v as u8;
    }
}

/// C: `FUN_overlay_d_93__021cfbf0`: trainer AI "switch `mon` out?" (`idx` = its slot index for [`set_slot`], `out` = the
/// chosen party slot). Only when [`escape_check`] says free (4) and the party (`021b9b98`) is larger than `c[0x11C]`; the
/// target is a random opponent ([`random_opponent`] of `021b8d8c(server, pokecon, 021d5440(mon))`, none -> 0), replaced by
/// `021b937c(server, pokecon, target)` when `021d6c24(target)`. The first test that fires among [`counter_test`],
/// [`wonder_guard_test`], [`super_effective_test`], [`encore_test`], [`absorb_switch`], [`natural_cure_switch`],
/// [`stat_boost_switch`] (the last three may pick the slot) decides: no slot picked -> the first free slot of the
/// [`live_slots`] list sorted by [`sort_by_power`]; a slot -> [`set_slot`]`(c, idx, slot)`, `*out = slot`, 1. Else 0.
pub fn ai_switch<E: ClientUi>(
    env: &mut E,
    c: &mut [u8],
    mon: u32,
    idx: u32,
    out: &mut [u8],
) -> u32 {
    let (mut b, mut h) = ([0u8; 1], [0u8; 2]);
    let mut choice = [6u8; 1];
    if escape_check(env, c, mon, &mut b, &mut h) != 4 {
        return 0;
    }
    if env.callh(0x021B_9B98, &mut [Arg::V(rd32(c, OFS_PARTY))], c) <= c[OFS_PARTY_COUNT] as u32 {
        return 0;
    }
    let id = mon_id(&mut Lend::h(env, c), mon);
    let side = env.callh(
        0x021B_8D8C,
        &mut [
            Arg::V(rd32(c, OFS_SERVER)),
            Arg::V(rd32(c, OFS_POKECON)),
            Arg::V(id),
        ],
        c,
    );
    let mut target = random_opponent(env, c, side);
    if target == 0 {
        return 0;
    }
    if env.callh(0x021D_6C24, &mut [Arg::V(target)], c) != 0 {
        target = env.callh(
            0x021B_937C,
            &mut [
                Arg::V(rd32(c, OFS_SERVER)),
                Arg::V(rd32(c, OFS_POKECON)),
                Arg::V(target),
            ],
            c,
        );
    }
    let fired = counter_test(&mut Lend::h(env, c), mon) != 0
        || wonder_guard_test(env, c, mon, target) != 0
        || super_effective_test(env, c, mon, target) != 0
        || encore_test(env, c, mon, target) != 0
        || absorb_switch(env, c, mon, target, &mut choice) != 0
        || natural_cure_switch(env, c, mon, &mut choice) != 0
        || stat_boost_switch(env, c, mon, target, &mut choice) != 0;
    if !fired {
        return 0;
    }
    if choice[0] == 6 {
        let mut list = [0u8; 8];
        let k = live_slots(env, c, Some(&mut list));
        sort_by_power(env, c, &mut list, k & 0xFF, target);
        for &slot in list.iter().take(k as usize) {
            if !slot_holds(c, slot as i32) {
                choice[0] = slot;
                break;
            }
        }
    }
    if choice[0] == 6 {
        return 0;
    }
    set_slot(c, idx, choice[0] as u32);
    out[0] = choice[0];
    1
}

/// C: `FUN_overlay_d_93__021cfd50`: a random opponent of `side`: `n = 021b8d38(server, pokecon, side | 0x100, list)`; none ->
/// 0; else one LCG step ([`lcg_next`]) and `021b9934(pokecon, list[(hi * n) >> 32])`.
pub fn random_opponent<E: FlowEnv>(env: &mut E, c: &mut [u8], side: u32) -> u32 {
    let mut list = [0u8; 4];
    let n = env.callh(
        0x021B_8D38,
        &mut [
            Arg::V(rd32(c, OFS_SERVER)),
            Arg::V(rd32(c, OFS_POKECON)),
            Arg::V((side | 0x100) & 0xFFFF),
            Arg::Out(&mut list),
        ],
        c,
    );
    if n == 0 {
        return 0;
    }
    let hi = lcg_next(c);
    let k = scaled(hi, n) & 0xFF;
    env.callh(
        0x021B_9934,
        &mut [
            Arg::V(rd32(c, OFS_POKECON)),
            Arg::V(list[k as usize] as u32),
        ],
        c,
    )
}

/// C: `FUN_overlay_d_93__021cfdcc`: flag 0x14 and its counter `021d631c(mon, 0x14) + 1` equals `021e8afc(021d6314(mon, 0x14))`
/// (the original's first argument is unused).
pub fn counter_test<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    if !flag(env, mon, 0x14) {
        return 0;
    }
    let w = env.call(0x021D_6314, &mut [Arg::V(mon), Arg::V(0x14)]);
    let lim = env.call(0x021E_8AFC, &mut [Arg::V(w)]);
    let n = env.call(0x021D_631C, &mut [Arg::V(mon), Arg::V(0x14)]);
    (n.wrapping_add(1) == lim) as u32
}

/// C: `FUN_overlay_d_93__021cfe08`: battle type 0, the target's ability 0x19, `mon` cannot hit it (`!`[`can_hit`]`(mon, target,
/// 4)`) and some free party mon can ([`any_hitter`]`(target, 4)`): one LCG step, `(hi * 3) >> 32 < 2`.
pub fn wonder_guard_test<E: ClientUi>(env: &mut E, c: &mut [u8], mon: u32, target: u32) -> u32 {
    if env.callh(0x021B_8588, &mut [Arg::V(rd32(c, OFS_SERVER))], c) != 0 {
        return 0;
    }
    if mon_get(&mut Lend::h(env, c), target, 0x11) != 0x19 {
        return 0;
    }
    if can_hit(env, c, mon, target, 4) != 0 {
        return 0;
    }
    if any_hitter(env, c, target, 4) == 0 {
        return 0;
    }
    (scaled(lcg_next(c), 3) < 2) as u32
}

/// C: `FUN_overlay_d_93__021cfe98`: `mon`'s damaging moves (`0201c314`) are all resisted by the target (`021d799c(0201c1c8(mv),
/// 021d57fc(target)) == 0`; any other -> 0) and there are at least two: [`any_hitter`]`(target, 4)` -> LCG `(hi * 3) >> 32 < 2`;
/// else [`any_hitter`]`(target, 3)` -> LCG high bit clear; else 0.
pub fn super_effective_test<E: ClientUi>(env: &mut E, c: &mut [u8], mon: u32, target: u32) -> u32 {
    let t = env.callh(0x021D_57FC, &mut [Arg::V(target)], c);
    let n = env.callh(0x021D_5448, &mut [Arg::V(mon)], c);
    let mut cnt = 0u32;
    let mut j = 0u32;
    while j < n {
        let mv = env.callh(0x021D_54C0, &mut [Arg::V(mon), Arg::V(j & 0xFF)], c);
        if env.callh(0x0201_C314, &mut [Arg::V(mv)], c) != 0 {
            let ty = env.callh(0x0201_C1C8, &mut [Arg::V(mv)], c);
            if env.callh(0x021D_799C, &mut [Arg::V(ty), Arg::V(t)], c) != 0 {
                return 0;
            }
            cnt += 1;
        }
        j += 1;
    }
    if cnt < 2 {
        return 0;
    }
    if any_hitter(env, c, target, 4) != 0 {
        return (scaled(lcg_next(c), 3) < 2) as u32;
    }
    if any_hitter(env, c, target, 3) != 0 {
        return (lcg_next(c) >> 31 == 0) as u32;
    }
    0
}

/// C: `FUN_overlay_d_93__021cffac`: `mon` locked into a move (flag 0x1B, `m = 021e8b80(021d6314(mon, 0x1B))`, none -> 0): a
/// damaging `m` (`0201c314`) the target is immune to (`021d799c(0201c1c8(m), 021d57fc(target)) == 0`): [`any_hitter`]`(4)`
/// -> LCG `(hi * 3) >> 32 < 2`, [`any_hitter`]`(3)` -> LCG high bit clear, else 0; a non-damaging `m`: LCG high bit clear;
/// else 0.
pub fn encore_test<E: ClientUi>(env: &mut E, c: &mut [u8], mon: u32, target: u32) -> u32 {
    if !flag(&mut Lend::h(env, c), mon, 0x1B) {
        return 0;
    }
    let w = env.callh(0x021D_6314, &mut [Arg::V(mon), Arg::V(0x1B)], c);
    let m = env.callh(0x021E_8B80, &mut [Arg::V(w)], c);
    if m == 0 {
        return 0;
    }
    let d = env.callh(0x0201_C314, &mut [Arg::V(m)], c);
    let ty = env.callh(0x0201_C1C8, &mut [Arg::V(m)], c);
    let t = env.callh(0x021D_57FC, &mut [Arg::V(target)], c);
    let eff = env.callh(0x021D_799C, &mut [Arg::V(ty), Arg::V(t)], c);
    if eff == 0 && d != 0 {
        if any_hitter(env, c, target, 4) != 0 {
            return (scaled(lcg_next(c), 3) < 2) as u32;
        }
        if any_hitter(env, c, target, 3) != 0 {
            return (lcg_next(c) >> 31 == 0) as u32;
        }
    }
    if d != 0 {
        return 0;
    }
    (lcg_next(c) >> 31 == 0) as u32
}

/// C: `FUN_overlay_d_93__021d0100`: switch to an absorbing ability: `mon` can hit the target ([`can_hit`]`(4)`) and the LCG
/// gives `(hi * 3) >> 32 == 0` -> 0. Then for each record `k = 0, 1, ...` of `021d6940(mon, 1, k, rec)` while it answers: the
/// entries of the ROM table 0x021EFFC2 with the record's type `rec[4]`: each ability id (up to the first 0) that
/// [`find_ability_slot`] finds, LCG high bit clear -> `*out` = the slot, 1. Else 0.
pub fn absorb_switch<E: ClientUi>(
    env: &mut E,
    c: &mut [u8],
    mon: u32,
    target: u32,
    out: &mut [u8],
) -> u32 {
    if can_hit(env, c, mon, target, 4) != 0 && scaled(lcg_next(c), 3) == 0 {
        return 0;
    }
    let mut k = 0u32;
    loop {
        let mut rec = [0u8; 8];
        let turn = k;
        k = (k + 1) & 0xFF;
        if env.callh(
            0x021D_6940,
            &mut [Arg::V(mon), Arg::V(1), Arg::V(turn), Arg::Out(&mut rec)],
            c,
        ) == 0
        {
            return 0;
        }
        for (ty, abilities) in ABSORB_TABLE.iter() {
            if rec[4] != *ty {
                continue;
            }
            for &ab in abilities.iter() {
                if ab == 0 {
                    break;
                }
                let mut slot = [0u8; 1];
                if find_ability_slot(env, c, ab as u32, &mut slot) != 0 && lcg_next(c) >> 31 == 0 {
                    out[0] = slot[0];
                    return 1;
                }
            }
        }
    }
}

/// C: `FUN_overlay_d_93__021d0248`: natural-cure switch: ability 0x1E, status flag 2 or 3, `021d5c44(mon) >= 0x32000`
/// (signed): with a move record (`021d6940(mon, 1, 0, rec)`), [`find_resist_slot`]`(rec[4], 2)` -> `*out` = the slot, 1,
/// else 0; without one, LCG high bit clear -> 1. Else 0.
pub fn natural_cure_switch<E: ClientUi>(
    env: &mut E,
    c: &mut [u8],
    mon: u32,
    out: &mut [u8],
) -> u32 {
    if mon_get(&mut Lend::h(env, c), mon, 0x11) != 0x1E {
        return 0;
    }
    if !flag(&mut Lend::h(env, c), mon, 2) && !flag(&mut Lend::h(env, c), mon, 3) {
        return 0;
    }
    if (env.callh(0x021D_5C44, &mut [Arg::V(mon)], c) as i32) < 0x32000 {
        return 0;
    }
    let mut rec = [0u8; 8];
    if env.callh(
        0x021D_6940,
        &mut [Arg::V(mon), Arg::V(1), Arg::V(0), Arg::Out(&mut rec)],
        c,
    ) != 0
    {
        let mut slot = [0u8; 1];
        if find_resist_slot(env, c, rec[4] as u32, 2, &mut slot) == 0 {
            return 0;
        }
        out[0] = slot[0];
        return 1;
    }
    (lcg_next(c) >> 31 == 0) as u32
}

/// C: `FUN_overlay_d_93__021d0304`: stat-boost switch: `mon` can hit the target ([`can_hit`]`(4)`) and the LCG gives
/// `(hi * 10) >> 32 != 0` -> 0; the sum of `BattleMon_Get(mon, 1..=7)` >= 4 -> 0; a move record (`021d6940(mon, 1, 0, rec)`, none -> 0):
/// for the free party slots from [`first_slot`] below the party size whose mon can hit the target: effectiveness
/// `021d799c(rec[4], 021d57fc(m))` 0 -> LCG high bit clear picks it; below 3 -> LCG `(hi * 3) >> 32 == 0` picks it (`*out` =
/// slot, 1). Else 0.
pub fn stat_boost_switch<E: ClientUi>(
    env: &mut E,
    c: &mut [u8],
    mon: u32,
    target: u32,
    out: &mut [u8],
) -> u32 {
    if can_hit(env, c, mon, target, 4) != 0 && scaled(lcg_next(c), 10) != 0 {
        return 0;
    }
    let mut sum = 0u32;
    for k in 1..=7 {
        sum = sum.wrapping_add(mon_get(&mut Lend::h(env, c), mon, k));
    }
    if sum >= 4 {
        return 0;
    }
    let mut rec = [0u8; 8];
    if env.callh(
        0x021D_6940,
        &mut [Arg::V(mon), Arg::V(1), Arg::V(0), Arg::Out(&mut rec)],
        c,
    ) == 0
    {
        return 0;
    }
    let n = party_size(env, c);
    let mut i = first_slot(env, c);
    while i < n {
        if !slot_holds(c, (i & 0xFF) as i32) {
            let m = party_mon(env, c, i);
            if can_hit(env, c, m, target, 4) != 0 {
                let t = env.callh(0x021D_57FC, &mut [Arg::V(m)], c);
                let eff = env.callh(0x021D_799C, &mut [Arg::V(rec[4] as u32), Arg::V(t)], c) as i32;
                if eff == 0 {
                    if lcg_next(c) >> 31 == 0 {
                        out[0] = i as u8;
                        return 1;
                    }
                } else if eff < 3 && scaled(lcg_next(c), 3) == 0 {
                    out[0] = i as u8;
                    return 1;
                }
            }
        }
        i += 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021d04e4`: the first free party slot (from [`first_slot`] below the party size) with a live mon whose
/// ability (`BattleMon_Get(m, 0x11)`) is `ability` -> `*out` = the slot, 1; else 0.
pub fn find_ability_slot<E: FlowEnv>(env: &mut E, c: &[u8], ability: u32, out: &mut [u8]) -> u32 {
    let n = party_size(env, c);
    let mut i = first_slot(env, c);
    while i < n {
        if !slot_holds(c, (i & 0xFF) as i32) {
            let m = party_mon(env, c, i);
            if !fainted(&mut Lend::r(env, c), m)
                && ability == mon_get(&mut Lend::r(env, c), m, 0x11)
            {
                out[0] = i as u8;
                return 1;
            }
        }
        i += 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021d055c`: some free party slot (from [`first_slot`] below the party size) whose mon can hit the
/// target with effectiveness >= `power` ([`can_hit`]).
pub fn any_hitter<E: ClientUi>(env: &mut E, c: &[u8], target: u32, power: u32) -> u32 {
    let n = party_size(env, c);
    let mut i = first_slot(env, c);
    while i < n {
        if !slot_holds(c, (i & 0xFF) as i32) {
            let m = party_mon(env, c, i);
            if can_hit(env, c, m, target, power) != 0 {
                return 1;
            }
        }
        i += 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021d05c4`: the first free party slot (from [`first_slot`] below the party size) whose mon is usable
/// (`021d5b7c`) and resists type `ty` (`021d799c(ty, 021d57fc(m)) <= max`, signed) -> `*out` = the slot, 1; else 0.
pub fn find_resist_slot<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    ty: u32,
    max: u32,
    out: &mut [u8],
) -> u32 {
    let n = party_size(env, c);
    let mut i = first_slot(env, c);
    while i < n {
        if !slot_holds(c, (i & 0xFF) as i32) {
            let m = party_mon(env, c, i);
            if env.callr(0x021D_5B7C, &mut [Arg::V(m)], c) != 0 {
                let t = env.callr(0x021D_57FC, &mut [Arg::V(m)], c);
                let eff = env.callr(0x021D_799C, &mut [Arg::V(ty), Arg::V(t)], c);
                if eff as i32 <= max as i32 {
                    out[0] = i as u8;
                    return 1;
                }
            }
        }
        i += 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021d0644`: a live `mon` has a move with PP (`021d5584`), damaging (`0201c314`), not blocked
/// ([`move_blocked`]) and with effectiveness `021d799c(0201c1c8(mv), 021d57fc(target)) >= min` (signed).
pub fn can_hit<E: ClientUi>(env: &mut E, c: &[u8], mon: u32, target: u32, min: u32) -> u32 {
    if fainted(&mut Lend::r(env, c), mon) {
        return 0;
    }
    let t = env.callr(0x021D_57FC, &mut [Arg::V(target)], c);
    let n = env.callr(0x021D_5448, &mut [Arg::V(mon)], c);
    let mut j = 0u32;
    while j < n {
        let mv = env.callr(0x021D_54C0, &mut [Arg::V(mon), Arg::V(j & 0xFF)], c);
        if env.callr(0x021D_5584, &mut [Arg::V(mon), Arg::V(j & 0xFF)], c) != 0
            && env.callr(0x0201_C314, &mut [Arg::V(mv)], c) != 0
            && move_blocked(env, c, mon, mv, 0) == 0
        {
            let ty = env.callr(0x0201_C1C8, &mut [Arg::V(mv)], c);
            if env.callr(0x021D_799C, &mut [Arg::V(ty), Arg::V(t)], c) as i32 >= min as i32 {
                return 1;
            }
        }
        j += 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021d06c8`: a random target side for a double battle: the list `{1}` plus 2 / 3 when the party mon of
/// side `021b9ab0(2)` / `021b9ab0(3)` is alive; one entry -> 1, else one LCG step and `list[(hi * n) >> 32]`.
pub fn random_partner_side<E: FlowEnv>(env: &mut E, c: &mut [u8]) -> u32 {
    let mut list = [1u8, 0, 0, 0];
    let mut n = 1usize;
    let s = env.callh(0x021B_9AB0, &mut [Arg::V(2)], c);
    let m = party_mon(env, c, s);
    if !fainted(&mut Lend::h(env, c), m) {
        n += 1;
        list[1] = 2;
    }
    let s = env.callh(0x021B_9AB0, &mut [Arg::V(3)], c);
    let m = party_mon(env, c, s);
    if !fainted(&mut Lend::h(env, c), m) {
        list[n] = 3;
        n += 1;
    }
    if n <= 1 {
        return 1;
    }
    let hi = lcg_next(c);
    list[scaled(hi, n as u32) as usize] as u32
}

/// C: `sub_021d0770`: request 3, mode 1, the trainer-AI action step (a 4-way jump table on the state).
///
/// * 0: [`clear_slots`], index `+0x11D` = 0, `c[0x1AF] = 0`, state 1, then as 1.
/// * 1: for mon `c[0x11D]` (`+8` = party slot `i`, `+0xC` = its record): fainted -> "skip"; already answered (`021cf470`,
///   client2); a trainer item (`021d3dc4(c, mon, party)`, client5) -> `021d8358(rec, item, i, 0)`; `021b86b8(server)` and a
///   switch ([`ai_switch`]) -> `021d8394(rec, i, slot)`; else (battle type 3: a partner side [`random_partner_side`] != 1 ->
///   `021d83fc(rec, side)`, `+8` = that side's mon, `c[0x1AF] = 1`, `+0xC` += 4) flag 0x19 -> `021d82d0(rec, m, 021d81f8(server,
///   pokecon, mon, m, c+0xF4))`; else (`021cf524` stuck -> state 3) the usable-move mask ([`usable_moves`]) has a move ->
///   the AI `021880bc(ai, mask, 021b8d8c(server, pokecon, 021d5440(mon)))`, state 2; else struggle (`021cf58c`). Each case
///   that answers -> state 3.
/// * 2: once `02187fb0(ai) == 0`: `k = 02188134(ai)`; 4 -> `021d8414(rec)`; else `021d82d0(rec, 021d54c0(mon, k),
///   02188140(ai))`; state 3.
/// * 3: index + 1; more -> state 1; else result = `&c[0x124]`, size `4 * (count + c[0x1AF])`, `c[0x1AF] = 0`, 1.
pub fn ai_step<E: ClientUi>(env: &mut E, c: &mut [u8], c_addr: u32, state: &mut [u8]) -> u32 {
    match st(state) {
        0 | 1 => {
            if st(state) == 0 {
                clear_slots(c);
                c[OFS_CUR] = 0;
                c[OFS_1AF] = 0;
                set_st(state, 1);
            }
            if c[OFS_CUR] >= c[OFS_PARTY_COUNT] {
                return 0;
            }
            let i = c[OFS_CUR] as u32;
            let mon = party_mon(env, c, i);
            wr32(c, OFS_MON, mon);
            wr32(c, OFS_REC, c_addr.wrapping_add(OFS_ANSWERS as u32 + 4 * i));
            let next = ai_choose(env, c, state);
            set_st(state, next);
            0
        }
        2 => {
            let ai = rd32(c, OFS_AI);
            if env.callh(0x0218_7FB0, &mut [Arg::V(ai)], c) != 0 {
                return 0;
            }
            let k = env.callh(0x0218_8134, &mut [Arg::V(rd32(c, OFS_AI))], c);
            if k == 4 {
                env.callh(0x021D_8414, &mut [Arg::PAt(rd32(c, OFS_REC))], c);
            } else {
                let t = env.callh(0x0218_8140, &mut [Arg::V(rd32(c, OFS_AI))], c);
                let mv = env.callh(0x021D_54C0, &mut [Arg::V(rd32(c, OFS_MON)), Arg::V(k)], c);
                env.callh(
                    0x021D_82D0,
                    &mut [Arg::PAt(rd32(c, OFS_REC)), Arg::V(mv), Arg::V(t)],
                    c,
                );
            }
            set_st(state, 3);
            0
        }
        3 => {
            c[OFS_CUR] = c[OFS_CUR].wrapping_add(1);
            let count = c[OFS_PARTY_COUNT];
            if c[OFS_CUR] < count {
                set_st(state, 1);
                return 0;
            }
            let n = count as u32 + c[OFS_1AF] as u32;
            wr32(c, OFS_RESULT_PTR, c_addr.wrapping_add(OFS_ANSWERS as u32));
            wr32(c, OFS_RESULT_SIZE, n << 2);
            c[OFS_1AF] = 0;
            1
        }
        _ => 0,
    }
}

/// The answer part of state 1 of [`ai_step`]; returns the next state (2 or 3).
fn ai_choose<E: ClientUi>(env: &mut E, c: &mut [u8], state: &mut [u8]) -> u32 {
    let i = c[OFS_CUR] as u32;
    let mon = rd32(c, OFS_MON);
    if fainted(&mut Lend::h(env, c), mon) {
        env.callh(0x021D_8434, &mut [Arg::PAt(rd32(c, OFS_REC))], c);
        return 3;
    }
    if env.callh(
        0x021C_F470,
        &mut [Arg::P, Arg::V(mon), Arg::PAt(rd32(c, OFS_REC))],
        c,
    ) != 0
    {
        return 3;
    }
    let item = env.callh(
        0x021D_3DC4,
        &mut [Arg::P, Arg::V(mon), Arg::V(rd32(c, OFS_PARTY))],
        c,
    );
    if item != 0 {
        env.callh(
            0x021D_8358,
            &mut [
                Arg::PAt(rd32(c, OFS_REC)),
                Arg::V(item),
                Arg::V(i),
                Arg::V(0),
            ],
            c,
        );
        return 3;
    }
    if env.callh(0x021B_86B8, &mut [Arg::V(rd32(c, OFS_SERVER))], c) != 0 {
        let mut slot = [0u8; 1];
        if ai_switch(env, c, mon, i, &mut slot) != 0 {
            env.callh(
                0x021D_8394,
                &mut [
                    Arg::PAt(rd32(c, OFS_REC)),
                    Arg::V(c[OFS_CUR] as u32),
                    Arg::V(slot[0] as u32),
                ],
                c,
            );
            return 3;
        }
    }
    if env.callh(0x021B_8588, &mut [Arg::V(rd32(c, OFS_SERVER))], c) == 3 {
        let side = random_partner_side(env, c);
        if side != 1 {
            let s = env.callh(0x021B_9AB0, &mut [Arg::V(side)], c);
            env.callh(
                0x021D_83FC,
                &mut [Arg::PAt(rd32(c, OFS_REC)), Arg::V(side)],
                c,
            );
            let m = env.callh(0x021B_9C00, &mut [Arg::V(rd32(c, OFS_PARTY)), Arg::V(s)], c);
            wr32(c, OFS_MON, m);
            c[OFS_1AF] = 1;
            wr32(c, OFS_REC, rd32(c, OFS_REC).wrapping_add(4));
        }
    }
    let mon = rd32(c, OFS_MON);
    if flag(&mut Lend::h(env, c), mon, 0x19) {
        let w = env.callh(0x021D_6314, &mut [Arg::V(mon), Arg::V(0x19)], c);
        let m = env.callh(0x021E_8B80, &mut [Arg::V(w)], c);
        // the saved RNG context buffer at c + 0xF4
        let sv = c[0xF4..].as_ptr() as usize;
        let t = env.callh(
            0x021D_81F8,
            &mut [
                Arg::V(rd32(c, OFS_SERVER)),
                Arg::V(rd32(c, OFS_POKECON)),
                Arg::V(mon),
                Arg::V(m),
                Arg::PHost(sv),
            ],
            c,
        );
        env.callh(
            0x021D_82D0,
            &mut [Arg::PAt(rd32(c, OFS_REC)), Arg::V(m), Arg::V(t)],
            c,
        );
        return 3;
    }
    if env.callh(
        0x021C_F524,
        &mut [Arg::P, Arg::V(mon), Arg::PAt(rd32(c, OFS_REC))],
        c,
    ) != 0
    {
        set_st(state, 3);
    }
    env.callh(0x021D_5448, &mut [Arg::V(rd32(c, OFS_MON))], c);
    let mut mask = [0u8; 4];
    if usable_moves(env, c, rd32(c, OFS_MON), &mut mask) != 4 {
        let id = mon_id(env, rd32(c, OFS_MON));
        let side = env.callh(
            0x021B_8D8C,
            &mut [
                Arg::V(rd32(c, OFS_SERVER)),
                Arg::V(rd32(c, OFS_POKECON)),
                Arg::V(id),
            ],
            c,
        );
        env.callh(
            0x0218_80BC,
            &mut [Arg::V(rd32(c, OFS_AI)), Arg::PBuf(&mut mask), Arg::V(side)],
            c,
        );
        return 2;
    }
    // (rec, c, mon)
    env.callh(
        0x021C_F58C,
        &mut [Arg::PAt(rd32(c, OFS_REC)), Arg::P, Arg::V(rd32(c, OFS_MON))],
        c,
    );
    3
}

/// C: `FUN_overlay_d_93__021d0980`: the party slots (from [`first_slot`] below the party size) whose mon `021d5b7c` accepts,
/// stored into `out` (if given); returns their number (a byte).
pub fn live_slots<E: FlowEnv>(env: &mut E, c: &[u8], mut out: Option<&mut [u8]>) -> u32 {
    let n = party_size(env, c);
    let mut i = first_slot(env, c);
    let mut k = 0u32;
    while i < n {
        let m = party_mon(env, c, i);
        if env.callr(0x021D_5B7C, &mut [Arg::V(m)], c) != 0 {
            if let Some(o) = out.as_deref_mut() {
                o[k as usize] = i as u8;
            }
            k = (k + 1) & 0xFF;
        }
        i = (i + 1) & 0xFF;
    }
    k
}

/// C: `FUN_overlay_d_93__021d09e0`: sorts the slot list `list[0..n)` (n <= 6) by each mon's best move score against `target`,
/// descending (a stable selection sort that swaps list entries with their scores). Score of a live mon = the maximum over its
/// damaging moves with PP of the power `0201c1d8(mv)` (below 10 -> 60) scaled by the effectiveness `021d799c(0201c1c8(mv),
/// 021d57fc(target))`: 0 -> 0, 1 -> 1/4, 2 -> 1/2, 3 -> 1x, 4 -> 2x, 5 -> 4x (as u16), other -> 1x; stored as u16.
pub fn sort_by_power<E: FlowEnv>(env: &mut E, c: &[u8], list: &mut [u8], n: u32, target: u32) {
    let t = env.callr(0x021D_57FC, &mut [Arg::V(target)], c);
    let mut score = [0u16; 6];
    let mut i = 0u32;
    while i < n {
        score[i as usize] = 0;
        let mon = party_mon(env, c, list[i as usize] as u32);
        if !fainted(&mut Lend::r(env, c), mon) {
            let k = env.callr(0x021D_5448, &mut [Arg::V(mon)], c);
            let mut j = 0u32;
            while j < k {
                if env.callr(0x021D_5584, &mut [Arg::V(mon), Arg::V(j)], c) != 0 {
                    let mv = env.callr(0x021D_54C0, &mut [Arg::V(mon), Arg::V(j)], c);
                    if env.callr(0x0201_C314, &mut [Arg::V(mv)], c) != 0 {
                        let ty = env.callr(0x0201_C1C8, &mut [Arg::V(mv)], c);
                        let mut pw = env.callr(0x0201_C1D8, &mut [Arg::V(mv)], c);
                        if pw < 10 {
                            pw = 60;
                        }
                        let x = match env.callr(0x021D_799C, &mut [Arg::V(ty), Arg::V(t)], c) {
                            0 => 0,
                            1 => (pw << 14) >> 16,
                            2 => (pw << 15) >> 16,
                            4 => (pw << 17) >> 16,
                            5 => (pw << 18) >> 16,
                            _ => pw,
                        };
                        if (score[i as usize] as u32) < x {
                            score[i as usize] = x as u16;
                        }
                    }
                }
                j = (j + 1) & 0xFF;
            }
        }
        i = (i + 1) & 0xFF;
    }
    let n = n as usize;
    for a in 0..n {
        for b in a + 1..n {
            if score[a] < score[b] {
                score.swap(a, b);
                list.swap(a, b);
            }
        }
    }
}

/// C: `FUN_overlay_d_93__021d0b10`: sets up the party-list view block `block` through three Thumb-to-ARM veneers (0x0689xxxx code,
/// not ported): `021d0b3c(block, c+0x118 party, b, a)`, `021d0b4c(block, c[0x11C] count)`, `021d0b5c(extra, block)` (`extra` is the
/// first stack argument).
pub fn setup_party_block<E: FlowEnv>(
    env: &mut E,
    c: &mut [u8],
    a: u32,
    b: u32,
    block: u32,
    extra: u32,
) {
    // `block` / `extra` may point into the client object (they do in `021ce308`): it is lent
    let party = rd32(c, OFS_PARTY);
    env.callh(
        0x021D_0B3C,
        &mut [Arg::V(block), Arg::V(party), Arg::V(b), Arg::V(a)],
        c,
    );
    env.callh(
        0x021D_0B4C,
        &mut [Arg::V(block), Arg::V(c[OFS_PARTY_COUNT] as u32)],
        c,
    );
    env.callh(0x021D_0B5C, &mut [Arg::V(extra), Arg::V(block)], c);
}

/// C: `FUN_021d0b6c`: the switch answers of the party view block `blk` (`blk[6]` entries): `021d8394(&c[0x124 + 4 i], blk[3 +
/// i], blk[i])`; result = `&c[0x124]`, size `4 * blk[6]`.
pub fn write_switch_answers<E: FlowEnv>(env: &mut E, c: &mut [u8], c_addr: u32, blk: &[u8]) {
    let mut i = 0usize;
    while i < blk[6] as usize {
        env.callh(
            0x021D_8394,
            &mut [Arg::P, Arg::V(blk[3 + i] as u32), Arg::V(blk[i] as u32)],
            c,
        );
        i = (i + 1) & 0xFF;
    }
    wr32(c, OFS_RESULT_PTR, c_addr.wrapping_add(OFS_ANSWERS as u32));
    wr32(c, OFS_RESULT_SIZE, 4 * blk[6] as u32);
}

/// C: `FUN_021d0bb0`: forced switch answers: from `start = 021b99c0(server, client id)`, for each request `k < c[0x1B3]`
/// (`021b8e28(server, c[0x1B5 + k], &a, &b)`), the next live party slot `i >= start` -> `021d8394(&c[0x124 + 4 * count++], b,
/// i)`, `start = i + 1`. Result = `&c[0x124]`, size `4 * count`.
pub fn forced_switch_answers<E: FlowEnv>(env: &mut E, c: &mut [u8], c_addr: u32) {
    let mut count = 0u32;
    let n = party_size(env, c);
    let mut start = env.callh(
        0x021B_99C0,
        &mut [Arg::V(rd32(c, OFS_SERVER)), Arg::V(c[OFS_ID] as u32)],
        c,
    );
    let mut k = 0usize;
    while k < c[OFS_REQ_COUNT] as usize {
        let (mut a, mut b) = ([0u8; 1], [0u8; 1]);
        env.callh(
            0x021B_8E28,
            &mut [
                Arg::V(rd32(c, OFS_SERVER)),
                Arg::V(c[OFS_REQ_LIST + k] as u32),
                Arg::Out(&mut a),
                Arg::Out(&mut b),
            ],
            c,
        );
        let mut i = start;
        while i < n {
            let m = party_mon(env, c, i);
            if !fainted(&mut Lend::h(env, c), m) {
                count = (count + 1) & 0xFF;
                env.callh(
                    0x021D_8394,
                    &mut [Arg::P, Arg::V(b[0] as u32), Arg::V(i & 0xFF)],
                    c,
                );
                start = (i + 1) & 0xFF;
                break;
            }
            i += 1;
        }
        k += 1;
    }
    wr32(c, OFS_RESULT_PTR, c_addr.wrapping_add(OFS_ANSWERS as u32));
    wr32(c, OFS_RESULT_SIZE, count * 4);
}

/// C: `FUN_overlay_d_93__021d0c78`: the switch requests for this client: `n = (u8) 021eec20(adapter, &p)` (the adapter data:
/// a byte list, resolved in `list` at `list_addr`); `out[k++] = p[i]` for each `i < n` with `021b8de4(server, p[i])` equal to
/// the client id. Returns `k` (a byte).
pub fn switch_requests<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    out: &mut [u8],
    list: &[u8],
    list_addr: u32,
) -> u32 {
    let mut p = [0u8; 4];
    let n = env.callr(
        0x021E_EC20,
        &mut [Arg::V(rd32(c, OFS_ADAPTER)), Arg::Out(&mut p)],
        c,
    ) & 0xFF;
    let base = u32::from_le_bytes(p).wrapping_sub(list_addr) as usize;
    let mut k = 0usize;
    for i in 0..n as usize {
        let id = list[base + i] as u32;
        if c[OFS_ID] as u32
            == env.callr(
                0x021B_8DE4,
                &mut [Arg::V(rd32(c, OFS_SERVER)), Arg::V(id)],
                c,
            )
        {
            out[k] = list[base + i];
            k += 1;
        }
    }
    (k & 0xFF) as u32
}

/// [`switch_requests`] into `c[0x1B5..]`, the count into `c[0x1B3]`.
fn load_requests<E: FlowEnv>(env: &mut E, c: &mut [u8], list: &[u8], list_addr: u32) -> u8 {
    let mut out = [0u8; 0xB];
    let n = switch_requests(env, c, &mut out, list, list_addr) as usize;
    c[OFS_REQ_LIST..OFS_REQ_LIST + n].copy_from_slice(&out[..n]);
    c[OFS_REQ_COUNT] = n as u8;
    n as u8
}

/// C: `sub_021d0cc4`: request 7, mode 0 (yes / no). State 0: `021eaf18(c+0x58, 3, 5)`, `021eaf18(c+0x7C, 3, 1)`,
/// `021eabc4(view, c+0x58, c+0x7C, 0)`, state 1. State 1: once `021eac54(view, &ans)`: `c[0x1AC] = ans != 0`, result =
/// `&c[0x1AC]`, size 1, returns 1. Else 0.
pub fn yes_no_step<E: ClientUi>(env: &mut E, c: &mut [u8], c_addr: u32, state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            env.view(0x021E_AF18, &mut [Arg::P, Arg::V(3), Arg::V(5)]);
            env.view(0x021E_AF18, &mut [Arg::P, Arg::V(3), Arg::V(1)]);
            env.view(
                0x021E_ABC4,
                &mut [Arg::V(view(c)), Arg::P, Arg::P, Arg::V(0)],
            );
            set_st(state, st(state).wrapping_add(1));
            0
        }
        1 => {
            let mut ans = [0u8; 4];
            if env.view(0x021E_AC54, &mut [Arg::V(view(c)), Arg::Out(&mut ans)]) == 0 {
                return 0;
            }
            c[OFS_YESNO] = (u32::from_le_bytes(ans) != 0) as u8;
            wr32(c, OFS_RESULT_PTR, c_addr.wrapping_add(OFS_YESNO as u32));
            wr32(c, OFS_RESULT_SIZE, 1);
            1
        }
        _ => 0,
    }
}

/// C: `sub_021d0d38`: request 5, mode 0: [`switch_in_step`] with kind 2 (a table-entry thunk).
pub fn req5_step<E: ClientUi>(
    env: &mut E,
    c: &mut [u8],
    c_addr: u32,
    state: &mut [u8],
    list: &[u8],
    list_addr: u32,
) -> u32 {
    switch_in_step(env, c, c_addr, state, 2, list, list_addr)
}

/// C: `sub_021d0d44`: request 6, mode 0: [`switch_in_step`] with kind 1 (a table-entry thunk).
pub fn req6_step<E: ClientUi>(
    env: &mut E,
    c: &mut [u8],
    c_addr: u32,
    state: &mut [u8],
    list: &[u8],
    list_addr: u32,
) -> u32 {
    switch_in_step(env, c, c_addr, state, 1, list, list_addr)
}

/// C: `sub_021d0d50`: requests 5 / 6, mode 1 (AI switch-in). The requests ([`switch_requests`] into `c[0x1B5..]`, count
/// `c[0x1B3]`); none -> a "skip" answer (size 4). The live slots ([`live_slots`]): none -> with [`partner_fainted`] and a
/// partner answer ([`check_partners`], its size), else `021d83c4(&c[0x124])` (size 4). Else sorted by [`sort_by_power`]
/// against [`random_opponent`]`(c[0x1B5])` (if any); more requests than slots: battle type 3 and a partner answer -> its
/// size, else as many answers as slots; answer `i`: `021b8e28(server, c[0x1B5 + i], &a, &b)`, `021d8394(&c[0x124 + 4 i], b,
/// slot i)`, size `4 n`. Result `&c[0x124]`. Returns 1.
pub fn ai_switch_in<E: ClientUi>(
    env: &mut E,
    c: &mut [u8],
    c_addr: u32,
    list: &[u8],
    list_addr: u32,
) -> u32 {
    let answers = c_addr.wrapping_add(OFS_ANSWERS as u32);
    if load_requests(env, c, list, list_addr) == 0 {
        env.callh(0x021D_8434, &mut [Arg::P], c);
        wr32(c, OFS_RESULT_PTR, answers);
        wr32(c, OFS_RESULT_SIZE, 4);
        return 1;
    }
    let mut slots = [0u8; 6];
    let k = live_slots(env, c, Some(&mut slots));
    if k == 0 {
        if partner_fainted(env, c) {
            let r = check_partners(env, c);
            if r != 0 {
                wr32(c, OFS_RESULT_SIZE, r);
                wr32(c, OFS_RESULT_PTR, answers);
                return 1;
            }
        }
        env.callh(0x021D_83C4, &mut [Arg::P], c);
        wr32(c, OFS_RESULT_PTR, answers);
        wr32(c, OFS_RESULT_SIZE, 4);
        return 1;
    }
    let mut n = c[OFS_REQ_COUNT] as u32;
    let target = random_opponent(env, c, c[OFS_REQ_LIST] as u32);
    if target != 0 {
        sort_by_power(env, c, &mut slots, k, target);
    }
    if n > k {
        if env.callh(0x021B_8588, &mut [Arg::V(rd32(c, OFS_SERVER))], c) == 3 {
            let r = check_partners(env, c);
            if r != 0 {
                wr32(c, OFS_RESULT_SIZE, r);
                wr32(c, OFS_RESULT_PTR, answers);
                return 1;
            }
        }
        n = k;
    }
    let mut i = 0u32;
    while i < n {
        let (mut a, mut b) = ([0u8; 1], [0u8; 1]);
        env.callh(
            0x021B_8E28,
            &mut [
                Arg::V(rd32(c, OFS_SERVER)),
                Arg::V(c[OFS_REQ_LIST + i as usize] as u32),
                Arg::Out(&mut a),
                Arg::Out(&mut b),
            ],
            c,
        );
        env.callh(
            0x021D_8394,
            &mut [
                Arg::P,
                Arg::V(b[0] as u32),
                Arg::V(slots[i as usize] as u32),
            ],
            c,
        );
        i = (i + 1) & 0xFF;
    }
    wr32(c, OFS_RESULT_PTR, answers);
    wr32(c, OFS_RESULT_SIZE, n << 2);
    1
}

/// C: `sub_021d0e74`: requests 5 / 6 / 4, mode 2: the replay step of request 3 (`021ce1b0`, client2; a table-entry thunk).
pub fn switch_replay<E: FlowEnv>(env: &mut E) -> u32 {
    env.call(0x021C_E1B0, &mut [Arg::P, Arg::P])
}

/// C: `LAB_overlay_d_93__021d0e7c`: requests 5 / 6 / 4, mode 0 (`kind` 2 / 1 / 0; a 7-way jump table on the state). Returns 1
/// in state 6, else 0.
///
/// * 0: the requests ([`switch_requests`] into `c[0x1B5..]`, count `c[0x1B3]`); none -> "skip" answer, size 4, state 4. The
///   live slots ([`live_slots`]): none -> partner answer ([`partner_fainted`] + [`check_partners`]) or `021d83c4`, state 4;
///   a double battle ([`partner_fainted`], with `021b9c00(party, 0)` fetched) with one slot and a fainted slot-0 mon ->
///   `021d8394(&c[0x124], 0, slot)`, size 4, state 4; else `021b8e28(server, c[0x1B5], &a, &b)`, the party view block
///   ([`setup_party_block`]`(c, kind, min(count, slots), c+0x184, c+0x190)`), `021e9a78(view, c+0x184, b, 0, c+0x190)`,
///   `021ce068(c)`, state 1.
/// * 1: `021ce08c(c)` -> `021e9b14(view)`, state 2; else `021e9b20(view)` -> [`write_switch_answers`] from `c+0x190`, state 3.
/// * 2: `021e9b20(view)` -> [`forced_switch_answers`], state 3.   3: `021ce0e0(c)`, state 4.
/// * 4: `021b86c0(server)` -> `c[0x1A9] = 1`, `021ea7fc(view)`, state 5; else state 6.   5: `021ea814(view)` -> state 6.
pub fn switch_in_step<E: ClientUi>(
    env: &mut E,
    c: &mut [u8],
    c_addr: u32,
    state: &mut [u8],
    kind: u32,
    list: &[u8],
    list_addr: u32,
) -> u32 {
    let answers = c_addr.wrapping_add(OFS_ANSWERS as u32);
    let set_result = |c: &mut [u8], size: u32| {
        wr32(c, OFS_RESULT_PTR, answers);
        wr32(c, OFS_RESULT_SIZE, size);
    };
    match st(state) {
        0 => {
            if load_requests(env, c, list, list_addr) == 0 {
                env.callh(0x021D_8434, &mut [Arg::P], c);
                set_result(c, 4);
                set_st(state, 4);
                return 0;
            }
            let mut slots = [0u8; 6];
            let k = live_slots(env, c, Some(&mut slots));
            if k == 0 {
                if partner_fainted(env, c) {
                    let r = check_partners(env, c);
                    if r != 0 {
                        wr32(c, OFS_RESULT_SIZE, r);
                        wr32(c, OFS_RESULT_PTR, answers);
                        set_st(state, 4);
                        return 0;
                    }
                }
                env.callh(0x021D_83C4, &mut [Arg::P], c);
                set_result(c, 4);
                set_st(state, 4);
                return 0;
            }
            let m = (c[OFS_REQ_COUNT] as u32).min(k);
            if partner_fainted(env, c) {
                let mon0 = party_mon(env, c, 0);
                if k == 1 && fainted(&mut Lend::h(env, c), mon0) {
                    env.callh(
                        0x021D_8394,
                        &mut [Arg::P, Arg::V(0), Arg::V(slots[0] as u32)],
                        c,
                    );
                    set_result(c, 4);
                    set_st(state, 4);
                    return 0;
                }
            }
            let (mut a, mut b) = ([0u8; 1], [0u8; 1]);
            env.callh(
                0x021B_8E28,
                &mut [
                    Arg::V(rd32(c, OFS_SERVER)),
                    Arg::V(c[OFS_REQ_LIST] as u32),
                    Arg::Out(&mut a),
                    Arg::Out(&mut b),
                ],
                c,
            );
            let block = c_addr.wrapping_add(OFS_PARTY_BLOCK as u32);
            let sel = c_addr.wrapping_add(OFS_PARTY_SEL as u32);
            setup_party_block(env, c, kind, m, block, sel);
            env.view(
                0x021E_9A78,
                &mut [
                    Arg::V(view(c)),
                    Arg::P,
                    Arg::V(b[0] as u32),
                    Arg::V(0),
                    Arg::P,
                ],
            );
            env.callh(0x021C_E068, &mut [Arg::P], c);
            set_st(state, 1);
            0
        }
        1 => {
            if env.callh(0x021C_E08C, &mut [Arg::P], c) != 0 {
                env.view(0x021E_9B14, &mut [Arg::V(view(c))]);
                set_st(state, 2);
            } else if env.view(0x021E_9B20, &mut [Arg::V(view(c))]) != 0 {
                let blk = c[OFS_PARTY_SEL..OFS_PARTY_SEL + 0x10].to_vec();
                write_switch_answers(env, c, c_addr, &blk);
                set_st(state, 3);
            }
            0
        }
        2 => {
            if env.view(0x021E_9B20, &mut [Arg::V(view(c))]) != 0 {
                forced_switch_answers(env, c, c_addr);
                set_st(state, 3);
            }
            0
        }
        3 => {
            env.callh(0x021C_E0E0, &mut [Arg::P], c);
            set_st(state, 4);
            0
        }
        4 => {
            if env.callh(0x021B_86C0, &mut [Arg::V(rd32(c, OFS_SERVER))], c) != 0 {
                c[OFS_1A9] = 1;
                env.view(0x021E_A7FC, &mut [Arg::V(view(c))]);
                set_st(state, 5);
            } else {
                set_st(state, 6);
            }
            0
        }
        5 => {
            if env.view(0x021E_A814, &mut [Arg::V(view(c))]) != 0 {
                set_st(state, 6);
            }
            0
        }
        6 => 1,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d102c`: battle type 3 and the mon in party slot 0 is fainted (`021d5b68`).
pub fn partner_fainted<E: FlowEnv>(env: &mut E, c: &[u8]) -> bool {
    if env.callr(0x021B_8588, &mut [Arg::V(rd32(c, OFS_SERVER))], c) != 3 {
        return false;
    }
    let m = party_mon(env, c, 0);
    fainted(&mut Lend::r(env, c), m)
}

/// C: `FUN_overlay_d_93__021d1058`: the partner sides 2 and 3 (`021b9ab0(side)` -> party slot -> `021d5b68`): the first one
/// whose mon is alive is answered into the record (`021d83fc(rec, side)`) and returns 4; both fainted -> 0.
pub fn check_partners<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    for side in [2u32, 3] {
        let s = env.callr(0x021B_9AB0, &mut [Arg::V(side)], c);
        let m = party_mon(env, c, s);
        if !fainted(&mut Lend::r(env, c), m) {
            env.callr(0x021D_83FC, &mut [Arg::P, Arg::V(side)], c);
            return 4;
        }
    }
    0
}

#[cfg(test)]
mod tests;
