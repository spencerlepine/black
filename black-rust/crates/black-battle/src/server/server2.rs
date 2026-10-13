//! `battle_server.s` part 2 (0x021B75A8-0x021B902C): sub-proc steps, per-frame loops, main-module getters, position maps (B11).
//!
//! Specs: `tools/rom-diff/svflow_specs/server2.spec` (`nowork`; M is `arg buf 0x1000` with the BATTLE_SETUP at arena 0x480).
//! Every function takes the address `m` of the main module M and reads it (and what it points at) through a [`Mem`] view;
//! callees in other areas (server1 / server3 / server_side / client / adapter, `force` stubs in the spec) and outside the
//! battle server (battle_net 021d4xxx, view, BattleMon, ARM9) go through [`FlowEnv::call`]. Calls inside this area are direct.
//!
//! The sub-proc steps 75a8 .. 81dc are the comm-battle setup state machines; they talk to `battle_net.s` (021d4xxx, link
//! layer: out of scope per D9 at run time, but ported here as logic over the trait). Glue (GameProc / heap / overlay):
//! 85ec (loads overlay 231 around a call into it).

use super::Mem;
use crate::server_flow::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021B_75A8,
    0x021B_7758,
    0x021B_7968,
    0x021B_7BB0,
    0x021B_7CB4,
    0x021B_7E18,
    0x021B_8070,
    0x021B_81DC,
    0x021B_835C,
    0x021B_83F4,
    0x021B_8434,
    0x021B_84D8,
    0x021B_8548,
    0x021B_8564,
    0x021B_8588,
    0x021B_8590,
    0x021B_85A0,
    0x021B_85A8,
    0x021B_85DC,
    0x021B_85EC,
    0x021B_8630,
    0x021B_8640,
    0x021B_8670,
    0x021B_86A0,
    0x021B_86B8,
    0x021B_86C0,
    0x021B_86C8,
    0x021B_86DC,
    0x021B_86E4,
    0x021B_8714,
    0x021B_871C,
    0x021B_8724,
    0x021B_873C,
    0x021B_8748,
    0x021B_8750,
    0x021B_8778,
    0x021B_87E8,
    0x021B_8834,
    0x021B_88EC,
    0x021B_8B78,
    0x021B_8B9C,
    0x021B_8BC4,
    0x021B_8BD0,
    0x021B_8BEC,
    0x021B_8C20,
    0x021B_8C38,
    0x021B_8C50,
    0x021B_8C5C,
    0x021B_8CB0,
    0x021B_8CE8,
    0x021B_8CFC,
    0x021B_8D08,
    0x021B_8D38,
    0x021B_8D8C,
    0x021B_8DC0,
    0x021B_8DEC,
    0x021B_8E28,
    0x021B_8E70,
    0x021B_8ED8,
    0x021B_8F38,
    0x021B_8F5C,
    0x021B_8F7C,
    0x021B_8FC0,
    0x021B_8FE4,
    0x021B_8FF4,
    0x021B_9000,
    0x021B_9010,
];

/// M: the 4 client object pointers.
const M_CLIENTS: u32 = 0x10;
/// M: word read by 871c (handed to 021d4808 by 7968).
const M_C0: u32 = 0xC0;
/// M: u8 per-client "done" flags.
const M_CLIENT_DONE: u32 = 0xC4;
/// M: word handed to 02013080 (85dc).
const M_43C: u32 = 0x43C;
/// M: u8[6] battle position -> client id.
const M_POS_CLIENT: u32 = 0x428;
/// M: u8 own client id.
const M_OWN_ID: u32 = 0x46C;
/// M: u8 side flag of this console.
const M_SIDE: u32 = 0x46D;
/// M: flag bits.
const M_FLAGS: u32 = 0x473;

/// BATTLE_SETUP: battle type word.
const SETUP_TYPE: u32 = 0;
/// BATTLE_SETUP: rule word (0..3).
const SETUP_RULE: u32 = 4;

/// ROM rodata of overlay 93 at 0x021EFCF0..0x021EFD80: `[0..4]` first BattleMon id per client {0, 12, 6, 18} (8d08), `[4..10]`
/// / `[0xA..0x10]` position tables of 8e70 (rule 1 / other rules, 2 sides x 3), then the sub-proc step tables (0x021EFD00..).
/// 8e70 indexes the position tables with an unbounded position, so the following bytes are kept as they are in the ROM.
const ROM_021EFCF0: [u8; 0x90] = [
    0x00, 0x0c, 0x06, 0x12, 0x03, 0x05, 0x00, 0x02, 0x04, 0x00, 0x03, 0x05, 0x07, 0x02, 0x04,
    0x06, //
    0x00, 0x00, 0x00, 0x00, 0xa9, 0x75, 0x1b, 0x02, 0x59, 0x77, 0x1b, 0x02, 0x69, 0x79, 0x1b,
    0x02, //
    0xb1, 0x7b, 0x1b, 0x02, 0x19, 0x7e, 0x1b, 0x02, 0x5d, 0x83, 0x1b, 0x02, 0x00, 0x00, 0x00,
    0x00, //
    0xa9, 0x75, 0x1b, 0x02, 0x59, 0x77, 0x1b, 0x02, 0x69, 0x79, 0x1b, 0x02, 0xb1, 0x7b, 0x1b,
    0x02, //
    0xb5, 0x7c, 0x1b, 0x02, 0x5d, 0x83, 0x1b, 0x02, 0x00, 0x00, 0x00, 0x00, 0xa9, 0x75, 0x1b,
    0x02, //
    0x59, 0x77, 0x1b, 0x02, 0x69, 0x79, 0x1b, 0x02, 0xb1, 0x7b, 0x1b, 0x02, 0xdd, 0x81, 0x1b,
    0x02, //
    0x5d, 0x83, 0x1b, 0x02, 0x00, 0x00, 0x00, 0x00, 0xa9, 0x75, 0x1b, 0x02, 0x59, 0x77, 0x1b,
    0x02, //
    0x69, 0x79, 0x1b, 0x02, 0xb1, 0x7b, 0x1b, 0x02, 0x71, 0x80, 0x1b, 0x02, 0x5d, 0x83, 0x1b,
    0x02, //
    0x02, 0x00, 0x02, 0x00, 0x05, 0x03, 0x06, 0x00, 0x02, 0x06, 0x02, 0x00, 0x02, 0x00, 0x04,
    0x02, //
];

/// A ROM byte of [`ROM_021EFCF0`] by address.
fn rom_byte(addr: u32) -> u8 {
    ROM_021EFCF0[addr.wrapping_sub(0x021E_FCF0) as usize]
}

/// A ROM word of [`ROM_021EFCF0`] by address (the sub-proc step tables of server1's comm sub-procs).
pub(crate) fn rom_word(addr: u32) -> u32 {
    let o = addr.wrapping_sub(0x021E_FCF0) as usize;
    u32::from_le_bytes(ROM_021EFCF0[o..o + 4].try_into().unwrap())
}

/// The BATTLE_SETUP pointer `*M`.
fn setup(mem: &Mem, m: u32) -> u32 {
    mem.r32(m)
}

// ==== main-module getters ======================================================================================

/// C: `FUN_overlay_d_93__021b8548`: calls `021cd914(client)` (client area) for every non-null client object `M[0x10 + 4 i]`.
pub fn quit_clients<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) {
    for i in 0..4 {
        let c = mem.r32(m + M_CLIENTS + 4 * i);
        if c != 0 {
            env.call(0x021C_D914, &mut [Arg::V(c)]);
        }
    }
}

/// C: `FUN_overlay_d_93__021b8564`: 0 if some client slot `i` (0..3) has a client object (`M[0x10 + 4 i] != 0`) whose done
/// flag `M[0xC4 + i]` is still 0, else 1.
pub fn all_clients_done(mem: &Mem, m: u32) -> u32 {
    let pending =
        (0..4).any(|i| mem.r32(m + M_CLIENTS + 4 * i) != 0 && mem.r8(m + M_CLIENT_DONE + i) == 0);
    u32::from(!pending)
}

/// C: `FUN_overlay_d_93__021b8588`: the rule word of the BATTLE_SETUP (`setup + 4`; 0 single, 1 double, 2 triple, 3 rotation?
/// UNVERIFIED meaning).
pub fn rule(mem: &Mem, m: u32) -> u32 {
    mem.r32(setup(mem, m) + SETUP_RULE)
}

/// C: `FUN_overlay_d_93__021b8590`: bit 2 of the flag byte `M[0x473]`.
pub fn flag_bit2(mem: &Mem, m: u32) -> u32 {
    u32::from((mem.r8(m + M_FLAGS) >> 2) & 1)
}

/// C: `FUN_overlay_d_93__021b85a0`: the setup byte `+0x80`.
pub fn setup_byte_80(mem: &Mem, m: u32) -> u32 {
    u32::from(mem.r8(setup(mem, m) + 0x80))
}

/// C: `FUN_overlay_d_93__021b85a8`: 1 if the battle type (86b8) is 1, the rule (8588) is 0, the setup byte `+0x20` (86c0) is 0
/// and `020083C8(setup + 0x5C word)` is 0; else 0.
pub fn is_plain_single<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) -> u32 {
    if battle_type(mem, m) != 1 || rule(mem, m) != 0 || setup_byte_20(mem, m) != 0 {
        return 0;
    }
    let w = mem.r32(setup(mem, m) + 0x5C);
    u32::from(env.call(0x0200_83C8, &mut [Arg::V(w)]) == 0)
}

/// C: `FUN_overlay_d_93__021b85dc`: tail call `02013080(M[0x43C])`.
pub fn call_02013080<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) -> u32 {
    let w = mem.r32(m + M_43C);
    env.call(0x0201_3080, &mut [Arg::V(w)])
}

/// C: `FUN_overlay_d_93__021b85ec`: for battle types 0 / 1: loads overlay 231 (`02034AC4(231)`), `r = 021809D8()` (code in
/// that overlay), unloads it (`02034A5C(231)`); `r == 0` -> 0, else 1 when `021ba1a8(M, 0x200)` (server3) is 0. Other types
/// and a nonzero 021ba1a8 -> 0. Glue: the overlay load / unload stays behind the trait (D3).
pub fn check_overlay_231<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) -> u32 {
    if mem.r32(setup(mem, m) + SETUP_TYPE) > 1 {
        return 0;
    }
    env.call(0x0203_4AC4, &mut [Arg::V(231)]);
    let r = env.call(0x0218_09D8, &mut []);
    env.call(0x0203_4A5C, &mut [Arg::V(231)]);
    if r == 0 {
        return 0;
    }
    u32::from(env.call(0x021B_A1A8, &mut [Arg::P, Arg::V(0x200)]) == 0)
}

/// C: `FUN_overlay_d_93__021b8630`: the setup word `+0x70` for battle types 0 / 1, else 0.
pub fn setup_word_70(mem: &Mem, m: u32) -> u32 {
    let s = setup(mem, m);
    if mem.r32(s + SETUP_TYPE) > 1 {
        0
    } else {
        mem.r32(s + 0x70)
    }
}

/// C: `sub_021b8640`: per-rule count: rule 0 -> 1, 1 -> 3, 2 / 3 / other -> 5 (a jump table in the original).
pub fn rule_count_8640(mem: &Mem, m: u32) -> u32 {
    match rule(mem, m) {
        0 => 1,
        1 => 3,
        _ => 5,
    }
}

/// C: `sub_021b8670`: a per-rule count from the rule word of the BATTLE_SETUP (`*M`): rule 0 -> 1, 1 -> 2, 2 -> 3, 3 -> 1,
/// anything else 1 (a jump table in the original). Probably the number of battle positions per side (single / double /
/// triple / rotation), UNVERIFIED meaning.
pub fn rule_positions_per_side(mem: &Mem, m: u32) -> u32 {
    match rule(mem, m) {
        1 => 2,
        2 => 3,
        _ => 1,
    }
}

/// C: `sub_021b86a0`: 1 if `pos < 2 * 8670(M)` (the position exists under the rule), else 0.
pub fn pos_in_range(mem: &Mem, m: u32, pos: u32) -> u32 {
    u32::from(pos < rule_positions_per_side(mem, m) << 1)
}

/// C: `FUN_overlay_d_93__021b86b8`: the battle type word of the BATTLE_SETUP (`setup + 0`).
pub fn battle_type(mem: &Mem, m: u32) -> u32 {
    mem.r32(setup(mem, m) + SETUP_TYPE)
}

/// C: `FUN_overlay_d_93__021b86c0`: the setup byte `+0x20` (0 = local battle, else comm).
pub fn setup_byte_20(mem: &Mem, m: u32) -> u32 {
    u32::from(mem.r8(setup(mem, m) + 0x20))
}

/// C: `FUN_021B86C8`: 1 if the setup byte `+0x22` (multi / variant flag) is nonzero, else 0.
pub fn setup_byte_22_set(mem: &Mem, m: u32) -> u32 {
    u32::from(mem.r8(setup(mem, m) + 0x22) != 0)
}

/// C: `FUN_overlay_d_93__021b86dc`: the setup halfword `+0x1A`.
pub fn setup_u16_1a(mem: &Mem, m: u32) -> u32 {
    u32::from(mem.r16(setup(mem, m) + 0x1A))
}

/// C: `FUN_overlay_d_93__021b86e4`: per battle type: 0 -> 0, 1 -> 1, 2 / 3 -> 2, other -> 1 (jump table).
pub fn type_class(mem: &Mem, m: u32) -> u32 {
    match battle_type(mem, m) {
        0 => 0,
        1 => 1,
        2 | 3 => 2,
        _ => 1,
    }
}

/// C: `FUN_overlay_d_93__021b8714`: the address `setup + 8`.
pub fn setup_addr_8(mem: &Mem, m: u32) -> u32 {
    setup(mem, m).wrapping_add(8)
}

/// C: `FUN_overlay_d_93__021b871c`: the word `M[0xC0]`.
pub fn word_c0(mem: &Mem, m: u32) -> u32 {
    mem.r32(m + M_C0)
}

/// C: `FUN_overlay_d_93__021b8724`: `0200CED4(setup + 0x68 word, 021d5444(mon))`.
pub fn call_0200ced4<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, mon: u32) -> u32 {
    let v = env.call(0x021D_5444, &mut [Arg::V(mon)]);
    let h = mem.r32(setup(mem, m) + 0x68);
    env.call(0x0200_CED4, &mut [Arg::V(h), Arg::V(v)])
}

/// C: `FUN_overlay_d_93__021b873c`: tail call `0200CF28(setup + 0x68 word)`.
pub fn call_0200cf28<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32) -> u32 {
    let h = mem.r32(setup(mem, m) + 0x68);
    env.call(0x0200_CF28, &mut [Arg::V(h)])
}

/// C: `FUN_overlay_d_93__021b8748`: the setup word `+0x58`.
pub fn setup_word_58(mem: &Mem, m: u32) -> u32 {
    mem.r32(setup(mem, m) + 0x58)
}

/// C: `FUN_overlay_d_93__021b8750`: for battle types 0 / 1 and `flag != 0`: `0200CFA0(setup + 0x68 word,
/// BattleMon_GetPokemon(mon))`.
pub fn call_0200cfa0<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, flag: u32, mon: u32) {
    if battle_type(mem, m) > 1 || flag == 0 {
        return;
    }
    let pp = env.call(0x021D_5844, &mut [Arg::V(mon)]);
    let h = mem.r32(setup(mem, m) + 0x68);
    env.call(0x0200_CFA0, &mut [Arg::V(h), Arg::V(pp)]);
}

// ==== target / position helpers ================================================================================

/// C: `FUN_overlay_d_93__021b8778`: expands a target spec (`bits 8-15` kind, `bits 0-7` position) into a list of positions at
/// `out`; returns the count. Kind 0: `out[0] = pos`, count `pos != 6`. Position 6 (none): `out[0] = 6`, 0. Otherwise by rule:
/// 1 -> 8834, 2 -> 88ec, 0 / 3 / other -> 87e8.
pub fn expand_target<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, spec: u32, out: &mut [u8]) -> u32 {
    let kind = (spec >> 8) & 0xFF;
    let pos = spec & 0xFF;
    if kind == 0 {
        out[0] = pos as u8;
        return u32::from(out[0] != 6);
    }
    if pos == 6 {
        out[0] = 6;
        return 0;
    }
    match rule(mem, m) {
        1 => expand_double(mem, m, kind, pos, out),
        2 => expand_triple(env, mem, m, kind, pos, out),
        _ => expand_single(mem, m, kind, pos, out),
    }
}

/// C: `FUN_overlay_d_93__021b87e8`: target expansion for single battles (kind, position): kinds 1 / 2 / 6 -> the opponent
/// (`8c50(M, pos, 0)`), 3 -> `pos`, 5 / 8 -> {0, 1}, 0 / 4 / 7 / > 8 -> none.
pub fn expand_single(mem: &Mem, m: u32, kind: u32, pos: u32, out: &mut [u8]) -> u32 {
    match kind {
        1 | 2 | 6 => {
            out[0] = opponent_slot(mem, m, pos, 0) as u8;
            1
        }
        3 => {
            out[0] = pos as u8;
            1
        }
        5 | 8 => {
            out[0] = 0;
            out[1] = 1;
            2
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021b8834`: target expansion for double battles (kind, position).
pub fn expand_double(mem: &Mem, m: u32, kind: u32, pos: u32, out: &mut [u8]) -> u32 {
    match kind {
        1 | 6 => {
            out[0] = opponent_slot(mem, m, pos, 0) as u8;
            out[1] = opponent_slot(mem, m, pos, 1) as u8;
            2
        }
        2 => {
            out[0] = opponent_slot(mem, m, pos, 0) as u8;
            out[1] = opponent_slot(mem, m, pos, 1) as u8;
            out[2] = partner_pos(mem, m, pos) as u8;
            3
        }
        3 => {
            out[0] = pos as u8;
            out[1] = partner_pos(mem, m, pos) as u8;
            if out[0] > out[1] {
                out.swap(0, 1);
            }
            2
        }
        4 | 9 => {
            out[0] = partner_pos(mem, m, pos) as u8;
            1
        }
        7 => {
            out[0] = pos as u8;
            out[1] = partner_pos(mem, m, pos) as u8;
            2
        }
        5 | 8 => {
            out[..4].copy_from_slice(&[0, 1, 2, 3]);
            4
        }
        _ => 0,
    }
}

/// The middle-row index trick of 88ec / 8c5c: `(pos >> 1) & 0xFF` with 0 and 2 swapped.
fn mirror_slot(pos: u32) -> u32 {
    match (pos >> 1) & 0xFF {
        0 => 2,
        2 => 0,
        s => s,
    }
}

/// C: `FUN_overlay_d_93__021b88ec`: target expansion for triple battles (kind, position); `c = 021b99f0(pos)` (server3; the
/// position is the centre one?) selects between the full and the adjacent-only lists.
pub fn expand_triple<E: FlowEnv>(
    env: &mut E,
    mem: &Mem,
    m: u32,
    kind: u32,
    pos: u32,
    out: &mut [u8],
) -> u32 {
    let c = env.call(0x021B_99F0, &mut [Arg::V(pos)]) & 0xFF;
    let opp = |s: u32| opponent_slot(mem, m, pos, s) as u8;
    let own = |s: u32| side_slot(pos, s) as u8;
    let mirror = || pos_of(2, pos, mirror_slot(pos)) as u8;
    match kind {
        2 => {
            if c != 0 {
                out[..5].copy_from_slice(&[opp(0), opp(1), opp(2), own(0), own(2)]);
                5
            } else {
                out[0] = opp(1);
                out[1] = mirror();
                out[2] = own(1);
                3
            }
        }
        3 => {
            out[0] = pos as u8;
            if c != 0 {
                out[1] = own(0);
                out[2] = own(2);
                3
            } else {
                out[1] = own(1);
                2
            }
        }
        4 | 9 => {
            if c != 0 {
                out[0] = own(0);
                out[1] = own(2);
                2
            } else {
                out[0] = own(1);
                1
            }
        }
        5 => {
            if c != 0 {
                out[..6].copy_from_slice(&[opp(0), opp(1), opp(2), own(0), own(2), pos as u8]);
                6
            } else {
                out[0] = opp(1);
                out[1] = mirror();
                out[2] = own(1);
                out[3] = pos as u8;
                4
            }
        }
        6 => {
            out[..3].copy_from_slice(&[opp(0), opp(1), opp(2)]);
            3
        }
        7 => {
            out[..3].copy_from_slice(&[own(0), own(1), own(2)]);
            3
        }
        8 => {
            out[..6].copy_from_slice(&[opp(0), opp(1), opp(2), own(0), own(1), own(2)]);
            6
        }
        // 0, 1 and > 9
        _ => {
            if c != 0 {
                out[..3].copy_from_slice(&[opp(0), opp(1), opp(2)]);
                3
            } else {
                out[0] = opp(1);
                out[1] = mirror();
                2
            }
        }
    }
}

/// C: `FUN_overlay_d_93__021b8b78`: the number of positions `i` (0..5) with `M[0x428 + i] == id`.
pub fn client_pos_count(mem: &Mem, m: u32, id: u32) -> u32 {
    (0..6)
        .filter(|&i| u32::from(mem.r8(m + M_POS_CLIENT + i)) == id)
        .count() as u32
}

/// C: `FUN_overlay_d_93__021b8b9c`: 1 if `id < 4` and some position maps to client `id`, else 0.
pub fn client_present(mem: &Mem, m: u32, id: u32) -> u32 {
    if id >= 4 {
        return 0;
    }
    u32::from((0..6).any(|i| u32::from(mem.r8(m + M_POS_CLIENT + i)) == id))
}

/// C: `FUN_overlay_d_93__021b8bc4`: the side of client `id`: `id & 1` (the first argument is unused).
pub fn client_side(id: u32) -> u32 {
    id & 1
}

/// C: `FUN_overlay_d_93__021b8bd0`: 1 if the own client's side (8bc4 of `M[0x46C]`) is `side`, else 0.
pub fn is_own_side(mem: &Mem, m: u32, side: u32) -> u32 {
    u32::from(side == client_side(u32::from(mem.r8(m + M_OWN_ID))))
}

/// C: `FUN_overlay_d_93__021b8bec`: the position of the `n`-th (0-based) slot of client `id` (`M[0x428 + i] == id`), 6 if none.
/// The counter `n` is kept as a byte once decremented.
pub fn client_nth_pos(mem: &Mem, m: u32, id: u32, n: u32) -> u32 {
    let mut n = n;
    for i in 0..6 {
        if u32::from(mem.r8(m + M_POS_CLIENT + i)) == id {
            if n == 0 {
                return i;
            }
            n = n.wrapping_sub(1) & 0xFF;
        }
    }
    6
}

/// C: `FUN_overlay_d_93__021b8c20`: position of slot `slot` on the side of `side` (bit 0): side 1 -> `slot * 2`, side 0 ->
/// `slot * 2 + 1` (byte); the first argument (rule) is unused.
pub fn pos_of(_rule: u32, side: u32, slot: u32) -> u32 {
    if side & 1 != 0 {
        (slot << 1) & 0xFF
    } else {
        (slot.wrapping_mul(2).wrapping_add(1)) & 0xFF
    }
}

/// C: `FUN_overlay_d_93__021b8c38`: position of slot `slot` on the side of `pos`: odd -> `slot * 2 + 1`, even -> `slot * 2`
/// (byte).
pub fn side_slot(pos: u32, slot: u32) -> u32 {
    if pos & 1 == 0 {
        (slot << 1) & 0xFF
    } else {
        (slot.wrapping_mul(2).wrapping_add(1)) & 0xFF
    }
}

/// C: `FUN_overlay_d_93__021b8c50`: tail call `8c20(rule, pos, slot)`: slot `slot` of the side opposing `pos`.
pub fn opponent_slot(mem: &Mem, m: u32, pos: u32, slot: u32) -> u32 {
    pos_of(rule(mem, m), pos, slot)
}

/// C: `sub_021b8c5c`: the opposing position facing `pos` under rule `rule`: rule 0 / > 3 -> slot 0, 1 -> slot
/// `((pos >> 1) & 0xFF) ^ 1`, 2 -> the mirrored slot, 3 -> slot 0 (all through 8c20).
pub fn facing_pos(rule: u32, pos: u32) -> u32 {
    let slot = match rule {
        1 => ((pos >> 1) & 0xFF) ^ 1,
        2 => mirror_slot(pos),
        _ => 0,
    };
    pos_of(rule, pos, slot)
}

/// C: `FUN_overlay_d_93__021b8cb0`: the partner position of `pos`: rule 1 -> `(pos + 2) & 3` (byte), else `pos`.
pub fn partner_pos(mem: &Mem, m: u32, pos: u32) -> u32 {
    if rule(mem, m) == 1 {
        pos.wrapping_add(2) & 3
    } else {
        pos
    }
}

/// C: `FUN_021B8CE8`: 1 if positions `a` and `b` are on different sides (bit 0 differs), else 0; the first argument is unused.
pub fn opposing(a: u32, b: u32) -> u32 {
    u32::from(a & 1 != b & 1)
}

/// C: `FUN_overlay_d_93__021b8cfc`: the client id of position `pos` (`M[0x428 + pos]`, unbounded).
pub fn pos_client(mem: &Mem, m: u32, pos: u32) -> u32 {
    u32::from(mem.r8(m.wrapping_add(pos) + M_POS_CLIENT))
}

/// C: `FUN_overlay_d_93__021b8d08`: the client (0..3) whose BattleMon id range `[first, first + 5]` (first ids {0, 12, 6, 18}
/// in the ROM table 0x021EFCF0; the upper end is a byte) contains `id`, else 0.
pub fn mon_id_client(id: u32) -> u32 {
    for i in 0..4u32 {
        let lo = rom_byte(0x021E_FCF0 + i);
        let hi = u32::from(lo.wrapping_add(5));
        if id >= u32::from(lo) && id <= hi {
            return i;
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021b8d38`: the BattleMon ids of a target spec: 8778(M, spec) into a stack list, then for each position
/// `mon = 021b98ac(pokecon, pos)` (server3, through the thunk 021b98a4); non-null mons with `021d5b68(mon) == 0` (not fainted?)
/// add `021d5440(mon)` (the id) to `out`. Returns the count.
pub fn target_mon_ids<E: FlowEnv>(
    env: &mut E,
    mem: &Mem,
    m: u32,
    pokecon: u32,
    spec: u32,
    out: &mut [u8],
) -> u32 {
    let mut list = [0u8; 8];
    let n = expand_target(env, mem, m, spec, &mut list);
    let mut k = 0u32;
    let mut i = 0u32;
    while i < n {
        let mon = env.call(
            0x021B_98AC,
            &mut [Arg::V(pokecon), Arg::V(u32::from(list[i as usize]))],
        );
        if mon != 0 && env.call(0x021D_5B68, &mut [Arg::V(mon)]) == 0 {
            let id = env.call(0x021D_5440, &mut [Arg::V(mon)]);
            out[k as usize] = id as u8;
            k = (k + 1) & 0xFF;
        }
        i = (i + 1) & 0xFF;
    }
    k
}

/// C: `sub_021b8d8c`: the battle position of BattleMon `id`: client `k = 8d08(id)`, slot `s = 021b9864(pokecon, k, id)`
/// (server3); `s < 0` -> 6, else `8bec(M, k, s)` (6 = none).
pub fn mon_pos<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, pokecon: u32, id: u32) -> u32 {
    let k = mon_id_client(id);
    let s = env.call(0x021B_9864, &mut [Arg::V(pokecon), Arg::V(k), Arg::V(id)]);
    if (s as i32) < 0 {
        return 6;
    }
    client_nth_pos(mem, m, k, s & 0xFF)
}

/// C: `FUN_overlay_d_93__021b8dc0`: 0xFF if BattleMon `id` has no position (8d8c == 6), else `8e70(M, pos)`.
pub fn mon_pos_class<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, pokecon: u32, id: u32) -> u32 {
    let pos = mon_pos(env, mem, m, pokecon, id);
    if pos == 6 {
        0xFF
    } else {
        pos_class(mem, m, pos)
    }
}

/// C: `FUN_overlay_d_93__021b8dec`: the slot index of `pos` within its client: the number of positions `i < pos` (scanned
/// from `pos - 1` down as a byte) that belong to the same client.
pub fn pos_client_slot(mem: &Mem, m: u32, pos: u32) -> u32 {
    let id = pos_client(mem, m, pos);
    let mut n = 0u32;
    let mut i = pos.wrapping_sub(1) & 0xFF;
    let mut more = pos;
    while more != 0 {
        if u32::from(mem.r8(m + i + M_POS_CLIENT)) == id {
            n = (n + 1) & 0xFF;
        }
        more = i;
        i = i.wrapping_sub(1) & 0xFF;
    }
    n
}

/// C: `FUN_overlay_d_93__021b8e28`: `*client_out = 8cfc(M, pos)` and `*slot_out` = the slot index of `pos` within that client
/// (as 8dec, compared against the stored byte).
pub fn pos_client_and_slot(
    mem: &Mem,
    m: u32,
    pos: u32,
    client_out: &mut [u8],
    slot_out: &mut [u8],
) {
    client_out[0] = pos_client(mem, m, pos) as u8;
    let mut n = 0u8;
    let mut i = pos.wrapping_sub(1) & 0xFF;
    let mut more = pos;
    while more != 0 {
        if mem.r8(m + i + M_POS_CLIENT) == client_out[0] {
            n = n.wrapping_add(1);
        }
        more = i;
        i = i.wrapping_sub(1) & 0xFF;
    }
    slot_out[0] = n;
}

/// C: `FUN_overlay_d_93__021b8e70`: a per-position class: `same = (M[0x46D] & 1) == (pos & 1)` (own side); rule 0 -> `!same`;
/// rule 1 -> ROM byte `0x021EFCF4[same * 3 + ((pos >> 1) & 0xFF)]`, other rules `0x021EFCFA[...]` (unbounded index).
pub fn pos_class(mem: &Mem, m: u32, pos: u32) -> u32 {
    let same = u32::from(u32::from(mem.r8(m + M_SIDE) & 1) == pos & 1);
    match rule(mem, m) {
        0 => u32::from(same == 0),
        r => {
            let tab = if r == 1 { 0x021E_FCF4 } else { 0x021E_FCFA };
            u32::from(rom_byte(tab + same * 3 + ((pos >> 1) & 0xFF)))
        }
    }
}

/// C: `FUN_overlay_d_93__021b8ed8`: inverse of 8e70 (class `v` -> position?): rule 0: odd `v` -> `M[0x46D] == 0`, even `v` ->
/// `M[0x46D]`; other rules: odd `v` -> `((M[0x46D] & 1) ^ 1) + v - 3`, even -> `(M[0x46D] & 1) + v - 2` (byte).
pub fn class_pos(mem: &Mem, m: u32, v: u32) -> u32 {
    let odd = v & 1 != 0;
    let side = mem.r8(m + M_SIDE);
    if rule(mem, m) == 0 {
        if odd {
            u32::from(side == 0)
        } else {
            u32::from(side)
        }
    } else {
        let s = u32::from(side & 1);
        if odd {
            ((s ^ 1).wrapping_add(v.wrapping_sub(3))) & 0xFF
        } else {
            (s.wrapping_add(v.wrapping_sub(2))) & 0xFF
        }
    }
}

/// C: `FUN_overlay_d_93__021b8f38`: 1 if the setup byte `+0x22` is set (86c8) and the battle type is neither 0 nor 1.
pub fn multi_not_type01(mem: &Mem, m: u32) -> u32 {
    if setup_byte_22_set(mem, m) == 0 {
        return 0;
    }
    u32::from(!matches!(battle_type(mem, m), 0 | 1))
}

/// C: `FUN_overlay_d_93__021b8f5c`: `021b969c(M + 0xC8, M, id, 0)`, then returns `021b985c(M + 0xC8, id)` (server3).
pub fn pokecon_refresh<E: FlowEnv>(env: &mut E, m: u32, id: u32) -> u32 {
    let _ = m;
    env.call(0x021B_969C, &mut [Arg::P, Arg::P, Arg::V(id), Arg::V(0)]);
    env.call(0x021B_985C, &mut [Arg::P, Arg::V(id)])
}

/// C: `FUN_overlay_d_93__021b8f7c`: in a multi battle (86c8): the partner `k = (id + 2) & 3`; if present (8b9c),
/// `021b969c(M + 0xC8, M, k, 0)` and returns `021b985c(M + 0xC8, k)`. Otherwise 0.
pub fn partner_pokecon_refresh<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, id: u32) -> u32 {
    if setup_byte_22_set(mem, m) == 0 {
        return 0;
    }
    let k = id.wrapping_add(2) & 3;
    if client_present(mem, m, k) == 0 {
        return 0;
    }
    env.call(0x021B_969C, &mut [Arg::P, Arg::P, Arg::V(k), Arg::V(0)]);
    env.call(0x021B_985C, &mut [Arg::P, Arg::V(k)])
}

/// C: `FUN_overlay_d_93__021b8fc0`: 1 if a multi battle (86c8) and bit 1 of client id `id` is set (the second pair), else 0.
pub fn is_multi_second_pair(mem: &Mem, m: u32, id: u32) -> u32 {
    if setup_byte_22_set(mem, m) == 0 {
        return 0;
    }
    u32::from(id & 2 != 0)
}

/// C: `FUN_overlay_d_93__021b8fe4`: tail call `8fc0(M, M[0x46C])`.
pub fn own_is_multi_second_pair(mem: &Mem, m: u32) -> u32 {
    is_multi_second_pair(mem, m, u32::from(mem.r8(m + M_OWN_ID)))
}

/// C: `FUN_overlay_d_93__021b8ff4`: the own client id `M[0x46C]`.
pub fn own_client_id(mem: &Mem, m: u32) -> u32 {
    u32::from(mem.r8(m + M_OWN_ID))
}

/// C: `FUN_overlay_d_93__021b9000`: tail call `9010(M, M[0x46C])`.
pub fn own_partner_id(mem: &Mem, m: u32) -> u32 {
    partner_id(mem, m, u32::from(mem.r8(m + M_OWN_ID)))
}

/// C: `FUN_overlay_d_93__021b9010`: the partner client of `id`: `k = (id + 2) & 3` if present (8b9c), else 4.
pub fn partner_id(mem: &Mem, m: u32, id: u32) -> u32 {
    let k = id.wrapping_add(2) & 3;
    if client_present(mem, m, k) != 0 {
        k
    } else {
        4
    }
}

// ==== sub-proc steps `u32 step(M, u32 *seq)` (comm setup; tables at 0x021EFD04) ==================================

/// M: received / view-side blocks (4 words).
const M_RECV: u32 = 0x3E0;
/// M: 0x18-byte block handed to the server / client creates.
const M_BLOCK18: u32 = 0x3F0;
/// M: 0x20-byte comm record (0x18 bytes of the block, 3 u16, 2 bytes).
const M_RECORD: u32 = 0x408;
/// M: the result pair of the client (`021cdd94`) / server (`021babdc`) frame.
const M_RESULT: u32 = 0x448;
/// M: main-loop function `fn(M)`.
const M_MAIN_FN: u32 = 0x464;
/// M: u16 heap id.
const M_HEAP: u32 = 0x468;
/// M: u8 client count.
const M_CLIENT_COUNT: u32 = 0x46B;
/// M: u8 "this console runs the server".
const M_IS_SERVER: u32 = 0x46E;
/// M: comm data handle (021d4d24).
const M_COMM_HANDLE: u32 = 0x2C4;

/// Thumb address of the server-console main loop 021b8434.
const MAIN_LOOP_SERVER: u32 = 0x021B_8435;
/// Thumb address of the client-console main loop 021b84d8.
const MAIN_LOOP_CLIENT: u32 = 0x021B_84D9;

/// The comm-error check every step starts with: `021d4270() != 0` sets bit 0 of `M[0x473]` and the step returns 1.
fn comm_error<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> bool {
    if env.call(0x021D_4270, &mut []) == 0 {
        return false;
    }
    let f = mem.r8(m + M_FLAGS);
    mem.w8(m + M_FLAGS, (f & !1) | 1);
    true
}

/// `*seq += 1`.
fn seq_next(mem: &mut Mem, seq: u32) {
    let s = mem.r32(seq);
    mem.w32(seq, s.wrapping_add(1));
}

/// Copies `n` bytes from `src` to `dst` (an `ldm`/`stm` block copy or `MI_CpuCopy8`).
fn copy(mem: &mut Mem, src: u32, dst: u32, n: usize) {
    let tmp = mem.read_vec(src, n);
    mem.write_bytes(dst, &tmp);
}

/// C: `sub_021b75a8`: comm handshake step. 0: send the own client id (`021d4294(M[0x46C])`), next when sent; 1: when
/// `021d4348()`: `M[0x46A] = 0`, `M[0x46E] = 021d4360()`, `setup[0x91] = 021d4378()`, seq = `021d4360() ? 2 : 3`; 2: build the
/// comm record at `M + 0x408` (the 0x18-byte block `M + 0x3F0`, `setup[0x86]`, `[0x74]`, `[0x76]` as u16, then the bytes `+0x1E`
/// / low nibble of `+0x1F`: `M[0x43C]` and bit 2 of `M[0x473]` when `021ba1a8(M, 0x1000)`, else 2 and 1) and send it
/// (`021d43b8`), next when sent; 3: receive it (`021d440c(M + 0x408)`) and spread it over `setup[0x86]`, `setup + 0x9C`,
/// `M + 0x3F0`, bit 2 of `M[0x473]`, `M[0x43C]`, `M[0x440]` / `M[0x442]`; returns 1. Other states: 0.
pub fn step_handshake<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, seq: u32) -> u32 {
    if comm_error(env, mem, m) {
        return 1;
    }
    let s = setup(mem, m);
    match mem.r32(seq) {
        0 => {
            let id = u32::from(mem.r8(m + M_OWN_ID));
            if env.call(0x021D_4294, &mut [Arg::V(id)]) != 0 {
                seq_next(mem, seq);
            }
            0
        }
        1 => {
            if env.call(0x021D_4348, &mut []) == 0 {
                return 0;
            }
            mem.w8(m + 0x46A, 0);
            let v = env.call(0x021D_4360, &mut []);
            mem.w8(m + M_IS_SERVER, v as u8);
            let v = env.call(0x021D_4378, &mut []);
            let s = setup(mem, m);
            mem.w8(s + 0x91, v as u8);
            let next = if env.call(0x021D_4360, &mut []) != 0 {
                2
            } else {
                3
            };
            mem.w32(seq, next);
            0
        }
        2 => {
            let r = m + M_RECORD;
            copy(mem, m + M_BLOCK18, r, 0x18);
            let v = mem.r16(s + 0x86);
            mem.w16(r + 0x18, v);
            let v = mem.r16(s + 0x74);
            mem.w16(r + 0x1A, v);
            let v = mem.r16(s + 0x76);
            mem.w16(r + 0x1C, v);
            let (lo, hi) = if env.call(0x021B_A1A8, &mut [Arg::P, Arg::V(0x1000)]) != 0 {
                (mem.r32(m + M_43C) as u8, (mem.r8(m + M_FLAGS) >> 2) & 1)
            } else {
                (2, 1)
            };
            mem.w8(r + 0x1E, lo);
            let b = mem.r8(r + 0x1F);
            mem.w8(r + 0x1F, (b & !0xF) | hi);
            let rec = mem.read_vec(r, 0x20);
            if env.call(0x021D_43B8, &mut [Arg::In(&rec)]) != 0 {
                seq_next(mem, seq);
            }
            0
        }
        3 => {
            let r = m + M_RECORD;
            let mut rec = mem.read_vec(r, 0x20);
            let got = env.call(0x021D_440C, &mut [Arg::Out(&mut rec)]);
            mem.write_bytes(r, &rec);
            if got == 0 {
                return 0;
            }
            let s = setup(mem, m);
            let v = mem.r16(r + 0x18);
            mem.w16(s + 0x86, v);
            copy(mem, r, s + 0x9C, 0x18);
            copy(mem, r, m + M_BLOCK18, 0x18);
            let bit = (mem.r8(r + 0x1F) & 0xF & 1) << 2;
            let f = mem.r8(m + M_FLAGS);
            mem.w8(m + M_FLAGS, (f & !4) | bit);
            let v = u32::from(mem.r8(r + 0x1E));
            mem.w32(m + M_43C, v);
            let v = mem.r16(r + 0x1A);
            mem.w16(m + 0x440, v);
            let v = mem.r16(r + 0x1C);
            mem.w16(m + 0x442, v);
            1
        }
        _ => 0,
    }
}

/// C: `sub_021b7758`: comm sync / party exchange step (13 states). 0 / 1: sync 2 (`021d4af4` send, `021d4b18` wait); 2 / 3: wait
/// `021d4438(setup[0x24])` / `021d44cc()`, then no party transfers (`M[0x471] == 0`) -> seq 10, else (server console) the comm
/// data handle `M[0x2C4] = 021d4d24(0201a918(), M[0x468] & 0x7FFF | 0x8000)` and `M[0x470] = 0`; 4: `M[0x472] = M[0x470] ? 3
/// : 1`, the server sends `setup[0x24 + 4 * M[0x472]]` (`021d4d64`), sync `M[0x470] + 6`; 5 wait for it; 6 the server waits
/// `021d4680(handle)`; 7 `021d4784(M[0x472])`, `++M[0x470] < M[0x471]` -> seq 4, else the server frees the handle
/// (`021d4d5c`) and seq 10; 8 / 9 / > 12: 1; 10: party container 0 (`021b93e0(M + 0xC8, M, 0)`) and for each present client
/// `021ba128` / `021ba648(M, i, 021d47a4(i))`, `021b9460(M + 0xC8, M, i)`; 11: the same for container 1 (`M + 0x1B0`);
/// 12: `021d4800()`. Every completed state advances the seq and returns 0.
pub fn step_party_sync<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, seq: u32) -> u32 {
    let s = setup(mem, m);
    if comm_error(env, mem, m) {
        return 1;
    }
    let sync = |mem: &Mem| u32::from(mem.r8(m + 0x470).wrapping_add(6));
    match mem.r32(seq) {
        0 => {
            env.call(0x021D_4AF4, &mut [Arg::V(2)]);
        }
        1 => {
            if env.call(0x021D_4B18, &mut [Arg::V(2)]) == 0 {
                return 0;
            }
        }
        2 => {
            let p = mem.r32(s + 0x24);
            if env.call(0x021D_4438, &mut [Arg::V(p)]) == 0 {
                return 0;
            }
        }
        3 => {
            if env.call(0x021D_44CC, &mut []) == 0 {
                return 0;
            }
            if mem.r8(m + 0x471) == 0 {
                mem.w32(seq, 10);
                return 0;
            }
            if mem.r8(m + M_IS_SERVER) != 0 {
                let a = env.call(0x0201_A918, &mut []);
                let heap = (u32::from(mem.r16(m + M_HEAP)) & 0x7FFF) | 0x8000;
                let h = env.call(0x021D_4D24, &mut [Arg::V(a), Arg::V(heap)]);
                mem.w32(m + M_COMM_HANDLE, h);
            }
            mem.w8(m + 0x470, 0);
        }
        4 => {
            let k = if mem.r8(m + 0x470) == 0 { 1 } else { 3 };
            mem.w8(m + 0x472, k);
            if mem.r8(m + M_IS_SERVER) != 0 {
                let k = u32::from(mem.r8(m + 0x472));
                let h = mem.r32(m + M_COMM_HANDLE);
                let p = mem.r32(s + 4 * k + 0x24);
                env.call(0x021D_4D64, &mut [Arg::V(h), Arg::V(p)]);
            }
            let n = sync(mem);
            env.call(0x021D_4AF4, &mut [Arg::V(n)]);
        }
        5 => {
            let n = sync(mem);
            if env.call(0x021D_4B18, &mut [Arg::V(n)]) == 0 {
                return 0;
            }
        }
        6 => {
            if mem.r8(m + M_IS_SERVER) != 0 {
                let h = mem.r32(m + M_COMM_HANDLE);
                if env.call(0x021D_4680, &mut [Arg::V(h)]) == 0 {
                    return 0;
                }
            }
        }
        7 => {
            let k = u32::from(mem.r8(m + 0x472));
            if env.call(0x021D_4784, &mut [Arg::V(k)]) == 0 {
                return 0;
            }
            let n = mem.r8(m + 0x470).wrapping_add(1);
            mem.w8(m + 0x470, n);
            if mem.r8(m + 0x470) < mem.r8(m + 0x471) {
                mem.w32(seq, 4);
                return 0;
            }
            if mem.r8(m + M_IS_SERVER) != 0 {
                let h = mem.r32(m + M_COMM_HANDLE);
                env.call(0x021D_4D5C, &mut [Arg::V(h)]);
                mem.w32(m + M_COMM_HANDLE, 0);
            }
            mem.w32(seq, 10);
            return 0;
        }
        10 => {
            env.call(0x021B_93E0, &mut [Arg::P, Arg::P, Arg::V(0)]);
            for i in 0..4 {
                if client_present(mem, m, i) != 0 {
                    let r = env.call(0x021D_47A4, &mut [Arg::V(i)]);
                    env.call(0x021B_A128, &mut [Arg::P, Arg::V(i), Arg::V(r)]);
                    let r = env.call(0x021D_47A4, &mut [Arg::V(i)]);
                    env.call(0x021B_A648, &mut [Arg::P, Arg::V(i), Arg::V(r)]);
                    env.call(0x021B_9460, &mut [Arg::P, Arg::P, Arg::V(i)]);
                }
            }
        }
        11 => {
            env.call(0x021B_93E0, &mut [Arg::P, Arg::P, Arg::V(1)]);
            for i in 0..4 {
                if client_present(mem, m, i) != 0 {
                    env.call(0x021B_9460, &mut [Arg::P, Arg::P, Arg::V(i)]);
                }
            }
        }
        12 => {
            env.call(0x021D_4800, &mut []);
        }
        // 8, 9 and > 12
        _ => return 1,
    }
    seq_next(mem, seq);
    0
}

/// C: `sub_021b7968`: comm player-record exchange step (11 states). 0 / 1: sync 4; 2: wait `021d4808(M[0xC0])`; 3: when
/// `021d4898()`: for each client `i < M[0x46B]` with a record `r = 021d48cc(i)`: `021b9e38(M + 0x20 + 0x28 i, M[0x468], r)`;
/// for another console's client with `setup[0xC1]` bit 2, the name string is rebuilt (a new 15-char StrBuf `02045b38` /
/// `02045ec0` / `02045c04`, at most 15 chars of `02045a90()` plus the terminator `02045a98()` copied into a stack u16[16]
/// handed to `020084d4(M[0x20 + 0x28 i], buf)`, the old `M[0x24 + 0x28 i]` freed (`02045808`) and replaced by a fresh empty
/// one); `021ba670(M, i, M[0x20 + 0x28 i])`; 4: no transfers (`M[0x471] == 0`) -> seq 10, else `M[0x470] = 0`; 5 / 6: sync
/// `M[0x470] + 8`; 7: `M[0x472] = M[0x470] ? 3 : 1`, the server waits `021d4908(setup[0x48 + 4 M[0x472]])`, then (state 8)
/// when `021d49b0()`: `021b9ef0(M + 0x20 + 0x28 M[0x472], 021d49cc())`, `021d49e0()`, `++M[0x470] < M[0x471]` -> seq 5 else
/// seq 10; 9 / > 10: 1; 10: `021d4a6c()` (through the thunk 021d4a04). The unwritten tail of the name buffer is zero here
/// (the original passes whatever is on its stack; UNVERIFIED deviation, harmless for the terminated string).
pub fn step_record_exchange<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, seq: u32) -> u32 {
    let s = setup(mem, m);
    if comm_error(env, mem, m) {
        return 1;
    }
    let sync = |mem: &Mem| u32::from(mem.r8(m + 0x470).wrapping_add(8));
    match mem.r32(seq) {
        0 => {
            env.call(0x021D_4AF4, &mut [Arg::V(4)]);
        }
        1 => {
            if env.call(0x021D_4B18, &mut [Arg::V(4)]) == 0 {
                return 0;
            }
        }
        2 => {
            let w = mem.r32(m + M_C0);
            if env.call(0x021D_4808, &mut [Arg::V(w)]) == 0 {
                return 0;
            }
        }
        3 => {
            if env.call(0x021D_4898, &mut []) == 0 {
                return 0;
            }
            let mut i = 0u32;
            while i < u32::from(mem.r8(m + M_CLIENT_COUNT)) {
                let r = env.call(0x021D_48CC, &mut [Arg::V(i & 0xFF)]);
                if r != 0 {
                    let rec = m + 0x20 + i.wrapping_mul(0x28);
                    let heap = u32::from(mem.r16(m + M_HEAP));
                    env.call(0x021B_9E38, &mut [Arg::P, Arg::V(heap), Arg::V(r)]);
                    if i != u32::from(mem.r8(m + M_OWN_ID)) && (mem.r8(s + 0xC1) >> 2) & 1 != 0 {
                        rebuild_name(env, mem, m, rec);
                    }
                    let p = mem.r32(rec);
                    env.call(0x021B_A670, &mut [Arg::P, Arg::V(i & 0xFF), Arg::V(p)]);
                }
                i = i.wrapping_add(1);
            }
        }
        4 => {
            if mem.r8(m + 0x471) == 0 {
                mem.w32(seq, 10);
                return 0;
            }
            mem.w8(m + 0x470, 0);
        }
        5 => {
            let n = sync(mem);
            env.call(0x021D_4AF4, &mut [Arg::V(n)]);
        }
        6 => {
            let n = sync(mem);
            if env.call(0x021D_4B18, &mut [Arg::V(n)]) == 0 {
                return 0;
            }
        }
        7 => {
            let k = if mem.r8(m + 0x470) == 0 { 1 } else { 3 };
            mem.w8(m + 0x472, k);
            if mem.r8(m + M_IS_SERVER) != 0 {
                let k = u32::from(mem.r8(m + 0x472));
                let p = mem.r32(s + 4 * k + 0x48);
                if env.call(0x021D_4908, &mut [Arg::V(p)]) == 0 {
                    return 0;
                }
            }
            seq_next(mem, seq);
            return record_receive(env, mem, m, seq);
        }
        8 => return record_receive(env, mem, m, seq),
        10 => {
            env.call(0x021D_4A6C, &mut []);
        }
        // 9 and > 10
        _ => return 1,
    }
    seq_next(mem, seq);
    0
}

/// State 8 of 7968 (also the tail of state 7): receive one player record.
fn record_receive<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, seq: u32) -> u32 {
    if env.call(0x021D_49B0, &mut []) == 0 {
        return 0;
    }
    let r = env.call(0x021D_49CC, &mut []);
    env.call(0x021B_9EF0, &mut [Arg::P, Arg::V(r)]);
    env.call(0x021D_49E0, &mut []);
    let n = mem.r8(m + 0x470).wrapping_add(1);
    mem.w8(m + 0x470, n);
    let next = if mem.r8(m + 0x470) < mem.r8(m + 0x471) {
        5
    } else {
        10
    };
    mem.w32(seq, next);
    0
}

/// The name rebuild of 7968 state 3 for the player record at `rec` (`M + 0x20 + 0x28 i`).
fn rebuild_name<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, rec: u32) {
    let heap = u32::from(mem.r16(m + M_HEAP));
    let msg = env.call(
        0x0204_5B38,
        &mut [Arg::V(0), Arg::V(2), Arg::V(0x51), Arg::V(heap)],
    );
    let str_buf = env.call(0x0204_5EC0, &mut [Arg::V(msg), Arg::V(0xF)]);
    env.call(0x0204_5C04, &mut [Arg::V(msg)]);
    let chars = env.call(0x0204_5A90, &mut [Arg::V(str_buf)]);
    let mut buf = [0u8; 0x20];
    let mut n = 0i32;
    while n < 15 && (n as u32) < env.call(0x0204_58F8, &mut [Arg::V(str_buf)]) {
        let c = mem.r16(chars.wrapping_add(n as u32 * 2));
        buf[n as usize * 2..n as usize * 2 + 2].copy_from_slice(&c.to_le_bytes());
        n += 1;
    }
    let eos = env.call(0x0204_5A98, &mut []) as u16;
    buf[n as usize * 2..n as usize * 2 + 2].copy_from_slice(&eos.to_le_bytes());
    env.call(0x0204_5808, &mut [Arg::V(str_buf)]);
    let p = mem.r32(rec);
    env.call(0x0200_84D4, &mut [Arg::V(p), Arg::In(&buf)]);
    let old = mem.r32(rec + 4);
    env.call(0x0204_5808, &mut [Arg::V(old)]);
    let heap = u32::from(mem.r16(m + M_HEAP));
    let msg = env.call(
        0x0204_5B38,
        &mut [Arg::V(0), Arg::V(2), Arg::V(0x51), Arg::V(heap)],
    );
    let fresh = env.call(0x0204_5EC0, &mut [Arg::V(msg), Arg::V(0xF)]);
    env.call(0x0204_5C04, &mut [Arg::V(msg)]);
    mem.w32(rec + 4, fresh);
}

/// C: `sub_021b7bb0`: comm received-data exchange step (4 states). 0: `021d4500()`, sync 3; 1: wait for it; 2: send
/// `021d450c(02007b44(02012ebc(setup[0x58])))`; 3: when `021d45fc()`: for each present client `i` (8b9c) whose data is not
/// local (`021b9f70(M, i) == 0`) with a block `src = 021d4648(i)`: `M[0x3E0 + 4 i] = 02007b20(M[0x468])` and the
/// `02007b00()` bytes copied there (`MI_CpuCopy8`); `021ba698(M, i)` for every `i`; `021d4674()`. > 3: 1.
pub fn step_data_exchange<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, seq: u32) -> u32 {
    let s = setup(mem, m);
    if comm_error(env, mem, m) {
        return 1;
    }
    match mem.r32(seq) {
        0 => {
            env.call(0x021D_4500, &mut []);
            env.call(0x021D_4AF4, &mut [Arg::V(3)]);
        }
        1 => {
            if env.call(0x021D_4B18, &mut [Arg::V(3)]) == 0 {
                return 0;
            }
        }
        2 => {
            let w = mem.r32(s + 0x58);
            let a = env.call(0x0201_2EBC, &mut [Arg::V(w)]);
            let a = env.call(0x0200_7B44, &mut [Arg::V(a)]);
            if env.call(0x021D_450C, &mut [Arg::V(a)]) == 0 {
                return 0;
            }
        }
        3 => {
            if env.call(0x021D_45FC, &mut []) == 0 {
                return 0;
            }
            for i in 0..4u32 {
                if client_present(mem, m, i) != 0
                    && env.call(0x021B_9F70, &mut [Arg::P, Arg::V(i)]) == 0
                {
                    let src = env.call(0x021D_4648, &mut [Arg::V(i)]);
                    if src != 0 {
                        let heap = u32::from(mem.r16(m + M_HEAP));
                        let dst = env.call(0x0200_7B20, &mut [Arg::V(heap)]);
                        mem.w32(m + M_RECV + 4 * i, dst);
                        let n = env.call(0x0200_7B00, &mut []);
                        let dst = mem.r32(m + M_RECV + 4 * i);
                        copy(mem, src, dst, n as usize);
                    }
                }
                env.call(0x021B_A698, &mut [Arg::P, Arg::V(i)]);
            }
            env.call(0x021D_4674, &mut []);
        }
        _ => return 1,
    }
    seq_next(mem, seq);
    0
}

/// `021ba9c8(M, M + 0x3F0, M + 0x1B0, mode, M[0x468])` (server3: the `btl_server.c` create).
fn server_create<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, mode: u32) -> u32 {
    let heap = u32::from(mem.r16(m + M_HEAP));
    env.call(
        0x021B_A9C8,
        &mut [Arg::P, Arg::P, Arg::P, Arg::V(mode), Arg::V(heap)],
    )
}

/// `M[0x10 + 4 id] = 021cd6f0(M, M + 0xC8, setup[0x20], setup[0x1C], id, cmode, a6, mode, 0, M + 0x3F0, M[0x468])`
/// (client1: the client create); returns the client.
fn client_create<E: FlowEnv>(
    env: &mut E,
    mem: &mut Mem,
    m: u32,
    id: u32,
    cmode: u32,
    a6: u32,
    mode: u32,
) -> u32 {
    let s = setup(mem, m);
    let heap = u32::from(mem.r16(m + M_HEAP));
    let b20 = u32::from(mem.r8(s + 0x20));
    let w1c = mem.r32(s + 0x1C);
    let c = env.call(
        0x021C_D6F0,
        &mut [
            Arg::P,
            Arg::P,
            Arg::V(b20),
            Arg::V(w1c),
            Arg::V(id),
            Arg::V(cmode),
            Arg::V(a6),
            Arg::V(mode),
            Arg::V(0),
            Arg::P,
            Arg::V(heap),
        ],
    );
    mem.w32(m + M_CLIENTS + 4 * id, c);
    c
}

/// `021baa94(M[8], 021cd978(client), id, k)` (server_side: attach a client to a server slot).
fn server_attach<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, client: u32, id: u32, k: u32) {
    let ch = env.call(0x021C_D978, &mut [Arg::V(client)]);
    let sv = mem.r32(m + 8);
    env.call(
        0x021B_AA94,
        &mut [Arg::V(sv), Arg::V(ch), Arg::V(id), Arg::V(k)],
    );
}

/// `021baacc(M[8], setup[0x20], setup[0x1C], id, k)` (server_side: register a remote client).
fn server_remote<E: FlowEnv>(env: &mut E, mem: &Mem, m: u32, id: u32, k: u32) {
    let s = setup(mem, m);
    let sv = mem.r32(m + 8);
    let b20 = u32::from(mem.r8(s + 0x20));
    let w1c = mem.r32(s + 0x1C);
    env.call(
        0x021B_AACC,
        &mut [Arg::V(sv), Arg::V(b20), Arg::V(w1c), Arg::V(id), Arg::V(k)],
    );
}

/// The client-console branch shared by 7cb4 / 7e18 / 8070 / 81dc: own client create, `M[0xC] = 021ba9c8(..)`, for each present
/// client `i`: `021bab10(M[0xC], i, kind(i))`, then `021cd960(own client, M[0xC])`.
fn client_console<E: FlowEnv>(
    env: &mut E,
    mem: &mut Mem,
    m: u32,
    id: u32,
    cmode: u32,
    mode: u32,
    by_party: bool,
) {
    client_create(env, mem, m, id, cmode, 0, mode);
    let sv = server_create(env, mem, m, mode);
    mem.w32(m + 0xC, sv);
    for i in 0..4 {
        if client_present(mem, m, i) != 0 {
            let k = if by_party {
                env.call(0x021B_6994, &mut [Arg::P, Arg::V(i)])
            } else {
                1
            };
            let sv = mem.r32(m + 0xC);
            env.call(0x021B_AB10, &mut [Arg::V(sv), Arg::V(i), Arg::V(k)]);
        }
    }
    let c = mem.r32(m + M_CLIENTS + 4 * id);
    let sv = mem.r32(m + 0xC);
    env.call(0x021C_D960, &mut [Arg::V(c), Arg::V(sv)]);
}

/// The common prologue of the create steps: `(own id, 6818(setup))` then the comm-error check (true = abort with 1).
fn create_prologue<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> (u32, u32, bool) {
    let id = u32::from(mem.r8(m + M_OWN_ID));
    let mode = env.call(0x021B_6818, &mut [Arg::P]);
    (id, mode, comm_error(env, mem, m))
}

/// C: `sub_021b7cb4`: one-shot create step of a 2-console battle (seq unused; returns 1). `mode = 6818(setup)`. Server
/// console (`M[0x46E]`): `M[8]` = server create, the own client (mode 1) attached with `021baa94(M[8], .., id, 1)` and the
/// other console registered with `021baacc(M[8], setup[0x20], setup[0x1C], id == 0, 1)`. Client console: own client, `M[0xC]`
/// = server create, `021bab10(M[0xC], i, 1)` for each present client, `021cd960(client, M[0xC])`.
pub fn step_create_pair<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let (id, mode, err) = create_prologue(env, mem, m);
    if err {
        return 1;
    }
    if mem.r8(m + M_IS_SERVER) != 0 {
        let sv = server_create(env, mem, m, mode);
        mem.w32(m + 8, sv);
        let c = client_create(env, mem, m, id, 1, 0, mode);
        server_attach(env, mem, m, c, id, 1);
        server_remote(env, mem, m, u32::from(id == 0), 1);
    } else {
        client_console(env, mem, m, id, 1, mode, false);
    }
    1
}

/// C: `sub_021b7e18`: one-shot create step of a multi battle. `k = setup[0x22] ? 1 : 2`. Server console: server create,
/// own client (`k`, 0) attached; `setup[0x22] == 2`: partner `p = 9010(M, id)`, clients 1 and 3 created (`k`, 1) into
/// `M[0x14]` / `M[0x1C]` and attached, then `021baacc(.., p, k)`; else every other client `i < M[0x46B]` registered with
/// `021baacc(.., i, k)`. Client console: own client (`k`), server create, `021bab10(M[0xC], i, 6994(M, i))`, `021cd960`.
pub fn step_create_multi<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let id = u32::from(mem.r8(m + M_OWN_ID));
    let mode = env.call(0x021B_6818, &mut [Arg::P]);
    let s = setup(mem, m);
    let k = if mem.r8(s + 0x22) != 0 { 1 } else { 2 };
    if comm_error(env, mem, m) {
        return 1;
    }
    if mem.r8(m + M_IS_SERVER) == 0 {
        client_console(env, mem, m, id, k, mode, true);
        return 1;
    }
    let sv = server_create(env, mem, m, mode);
    mem.w32(m + 8, sv);
    let c = client_create(env, mem, m, id, k, 0, mode);
    server_attach(env, mem, m, c, id, k);
    let s = setup(mem, m);
    if mem.r8(s + 0x22) == 2 {
        let p = partner_id(mem, m, id);
        let c = client_create(env, mem, m, 1, k, 1, mode);
        server_attach(env, mem, m, c, 1, k);
        let c = client_create(env, mem, m, 3, k, 1, mode);
        server_attach(env, mem, m, c, 3, k);
        server_remote(env, mem, m, p, k);
    } else {
        let mut i = 0u32;
        while i < u32::from(mem.r8(m + M_CLIENT_COUNT)) {
            if i != id {
                server_remote(env, mem, m, i & 0xFF, k);
            }
            i += 1;
        }
    }
    1
}

/// C: `sub_021b8070`: one-shot create step with client mode 3 (rotation?): as 7cb4 with mode 3 (`021baa94(.., id, 3)`,
/// `021baacc(.., id == 0, 3)`); the client console asks 6994 for each present client's mode.
pub fn step_create_mode3<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let (id, mode, err) = create_prologue(env, mem, m);
    if err {
        return 1;
    }
    if mem.r8(m + M_IS_SERVER) != 0 {
        let sv = server_create(env, mem, m, mode);
        mem.w32(m + 8, sv);
        let c = client_create(env, mem, m, id, 3, 0, mode);
        server_attach(env, mem, m, c, id, 3);
        server_remote(env, mem, m, u32::from(id == 0), 3);
    } else {
        client_console(env, mem, m, id, 3, mode, true);
    }
    1
}

/// C: `sub_021b81dc`: one-shot create step for `M[0x46B]` consoles: server console: server create, own client (mode 1)
/// attached, every other client `i < M[0x46B]` registered (`021baacc(.., i, 1)`); client console as 8070 with mode 1.
pub fn step_create_many<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let (id, mode, err) = create_prologue(env, mem, m);
    if err {
        return 1;
    }
    if mem.r8(m + M_IS_SERVER) != 0 {
        let sv = server_create(env, mem, m, mode);
        mem.w32(m + 8, sv);
        let c = client_create(env, mem, m, id, 1, 0, mode);
        server_attach(env, mem, m, c, id, 1);
        let mut i = 0u32;
        while i < u32::from(mem.r8(m + M_CLIENT_COUNT)) {
            if i != id {
                server_remote(env, mem, m, i & 0xFF, 1);
            }
            i += 1;
        }
    } else {
        client_console(env, mem, m, id, 1, mode, true);
    }
    1
}

/// C: `sub_021b835c`: view step. seq 0: `M[4] = 021e8f20(M, own client, M + 0xC8, 6818(setup), 0x13)` (battle view
/// create), `021cd95c(own client, M[4])`; server console: main loop `M[0x464]` = 8434 and `021bab40(M[8])` (server start),
/// else 84d8; seq++ and 0. seq != 0: 1.
pub fn step_view<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32, seq: u32) -> u32 {
    if comm_error(env, mem, m) {
        return 1;
    }
    if mem.r32(seq) != 0 {
        return 1;
    }
    let mode = env.call(0x021B_6818, &mut [Arg::P]);
    let own = |mem: &Mem| mem.r32(m + M_CLIENTS + 4 * u32::from(mem.r8(m + M_OWN_ID)));
    let c = own(mem);
    let view = env.call(
        0x021E_8F20,
        &mut [Arg::P, Arg::V(c), Arg::P, Arg::V(mode), Arg::V(0x13)],
    );
    mem.w32(m + 4, view);
    let c = own(mem);
    env.call(0x021C_D95C, &mut [Arg::V(c), Arg::V(view)]);
    if mem.r8(m + M_IS_SERVER) != 0 {
        mem.w32(m + M_MAIN_FN, MAIN_LOOP_SERVER);
        let sv = mem.r32(m + 8);
        env.call(0x021B_AB40, &mut [Arg::V(sv)]);
    } else {
        mem.w32(m + M_MAIN_FN, MAIN_LOOP_CLIENT);
    }
    seq_next(mem, seq);
    0
}

/// The client pass of 83f4 / 84d8: every client `c` with `021cd97c(c)` gets `021cdd94(c, M + 0x448)`; then the view frame
/// `021e908c(M[4])`. Returns 1 if some client ran.
fn clients_frame<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let mut r = 0;
    for i in 0..4 {
        let c = mem.r32(m + M_CLIENTS + 4 * i);
        if c != 0 && env.call(0x021C_D97C, &mut [Arg::V(c)]) != 0 {
            let c = mem.r32(m + M_CLIENTS + 4 * i);
            let mut res = mem.read_vec(m + M_RESULT, 8);
            env.call(0x021C_DD94, &mut [Arg::V(c), Arg::Out(&mut res)]);
            mem.write_bytes(m + M_RESULT, &res);
            r = 1;
        }
    }
    let view = mem.r32(m + 4);
    env.call(0x021E_908C, &mut [Arg::V(view)]);
    r
}

/// C: `sub_021b83f4`: one frame with a remote server: `021bab9c(M[8])` (result ignored), then the client pass (see 84d8).
pub fn frame_no_server<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    let sv = mem.r32(m + 8);
    env.call(0x021B_AB9C, &mut [Arg::V(sv)]);
    clients_frame(env, mem, m)
}

/// The quit prologue of 8434 / 84d8: `Some(1)` when `M[0x473]` bits 0 and 1 are set; bit 0 clear and `021d4270()` -> set
/// bit 0 and quit the clients (8548).
fn main_loop_prologue<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> Option<u32> {
    let f = mem.r8(m + M_FLAGS);
    if f & 1 == 0 {
        if env.call(0x021D_4270, &mut []) != 0 {
            let f = mem.r8(m + M_FLAGS);
            mem.w8(m + M_FLAGS, (f & !1) | 1);
            quit_clients(env, mem, m);
        }
    } else if (f >> 1) & 1 != 0 {
        return Some(1);
    }
    None
}

/// C: `sub_021b8434`: main loop of the server console (`M[0x464]`): the quit prologue, then `r = 021bab9c(M[8])` (server
/// step; nonzero -> `021babdc(M[8], M + 0x448)`), every client `i` not done yet gets `M[0xC4 + i] = 021cd97c(c)`, when all
/// are done (8564) `021cdd94(own client, M + 0x448)` and `r = 1`; the view frame `021e908c(M[4])`; returns `r`.
pub fn main_loop_server<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    if let Some(r) = main_loop_prologue(env, mem, m) {
        return r;
    }
    let sv = mem.r32(m + 8);
    let mut r = env.call(0x021B_AB9C, &mut [Arg::V(sv)]);
    if r != 0 {
        let sv = mem.r32(m + 8);
        let mut res = mem.read_vec(m + M_RESULT, 8);
        env.call(0x021B_ABDC, &mut [Arg::V(sv), Arg::Out(&mut res)]);
        mem.write_bytes(m + M_RESULT, &res);
    }
    for i in 0..4 {
        let c = mem.r32(m + M_CLIENTS + 4 * i);
        if c != 0 && mem.r8(m + M_CLIENT_DONE + i) == 0 {
            let d = env.call(0x021C_D97C, &mut [Arg::V(c)]);
            mem.w8(m + M_CLIENT_DONE + i, d as u8);
        }
    }
    if all_clients_done(mem, m) != 0 {
        let c = mem.r32(m + M_CLIENTS + 4 * u32::from(mem.r8(m + M_OWN_ID)));
        let mut res = mem.read_vec(m + M_RESULT, 8);
        env.call(0x021C_DD94, &mut [Arg::V(c), Arg::Out(&mut res)]);
        mem.write_bytes(m + M_RESULT, &res);
        r = 1;
    }
    let view = mem.r32(m + 4);
    env.call(0x021E_908C, &mut [Arg::V(view)]);
    r
}

/// C: `sub_021b84d8`: main loop of a client console: the quit prologue of 8434, then the client pass of 83f4 without the
/// server step.
pub fn main_loop_client<E: FlowEnv>(env: &mut E, mem: &mut Mem, m: u32) -> u32 {
    if let Some(r) = main_loop_prologue(env, mem, m) {
        return r;
    }
    clients_frame(env, mem, m)
}

#[cfg(test)]
mod tests;
