//! `btl_client.c` part 5 (0x021D2FD8-0x021D3F04): playback-table command handlers and client getters (B13-spec1 / B13-spec2,
//! ported in B13b).
//!
//! Specs: `tools/rom-diff/svflow_specs/client5.spec` (`nowork`, the client object is `arg buf 0x1C0`, a handler's state word
//! `arg buf 0x10`, the argument record `arg buf 0x40`).
//!
//! The handlers are the entries of the playback table 0x021F00DC (`{BtlCmd id, fn}`, see [`super::client4::PLAYBACK_TABLE`]):
//! `u32 handler(client *c, u32 *state, u32 *args)`, 1 = the command is played. The one-shot ones ignore `state`; most of them
//! are "`mon = 021b9934(pokecon, u8 args[0])`, then one BattleMon call". The BattleMon methods, server queries (021b8xxx /
//! 021b9xxx), the battle view (`c+0x54`, 021eaxxx), the view-side object (`c+0x34`, 021efxxx) and sound / effects (overlay 94)
//! are reached through `FlowEnv::call` (D3 / D7: the view half is rewritten, not ported). The doc comment of each handler names
//! the BattleMon method of [`crate::battle_mon::BattleMon`] the callee address is.

use super::client4::{arg, msg_arg, msg_done, msg_set, msg_show, ofs, state_inc};
use super::{rd16, rd32};
use crate::server_flow::env::{Arg, FlowEnv, Lend};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021D_2FD8,
    0x021D_3138,
    0x021D_31A0,
    0x021D_324C,
    0x021D_32E0,
    0x021D_344C,
    0x021D_34B0,
    0x021D_34E8,
    0x021D_3568,
    0x021D_35C8,
    0x021D_3630,
    0x021D_3688,
    0x021D_36CC,
    0x021D_36EC,
    0x021D_370C,
    0x021D_3734,
    0x021D_375C,
    0x021D_377C,
    0x021D_3794,
    0x021D_37B8,
    0x021D_37DC,
    0x021D_37FC,
    0x021D_381C,
    0x021D_3894,
    0x021D_38AC,
    0x021D_38C4,
    0x021D_38E0,
    0x021D_3900,
    0x021D_3918,
    0x021D_3934,
    0x021D_3988,
    0x021D_39A8,
    0x021D_3A20,
    0x021D_3A40,
    0x021D_3A7C,
    0x021D_3A98,
    0x021D_3AB4,
    0x021D_3AD0,
    0x021D_3AEC,
    0x021D_3B08,
    0x021D_3B28,
    0x021D_3B5C,
    0x021D_3B74,
    0x021D_3B88,
    0x021D_3B9C,
    0x021D_3BB0,
    0x021D_3BC0,
    0x021D_3BE0,
    0x021D_3C0C,
    0x021D_3C2C,
    0x021D_3C44,
    0x021D_3C70,
    0x021D_3C90,
    0x021D_3CA8,
    0x021D_3CC8,
    0x021D_3CFC,
    0x021D_3D1C,
    0x021D_3D2C,
    0x021D_3D50,
    0x021D_3D70,
    0x021D_3D7C,
    0x021D_3D84,
    0x021D_3D90,
    0x021D_3D9C,
    0x021D_3DC4,
    0x021D_3E70,
    0x021D_3EC8,
];

/// `BattleMon* 021b9934(pokecon, id)` (server-side mon lookup, not ported).
const MON_OF: u32 = 0x021B_9934;
/// `u8 021b8d8c(server, pokecon, id)`: view slot of a mon id.
const VIEW_SLOT: u32 = 0x021B_8D8C;
/// `021b8e70(server, id)`: view position of a slot / mon id.
const VIEW_POS: u32 = 0x021B_8E70;
/// `021cd9e0(c)` (client1): the client plays no animations (the "idle" / fast mode); `force` stub in the specs.
const IS_QUIET: u32 = 0x021C_D9E0;
/// `021f7eb4()` (overlay 94): an effect / sound sequence is still running.
const FX_BUSY: u32 = 0x021F_7EB4;

/// `021b9934(pokecon, u8 args[0])`: the BattleMon a command addresses.
fn mon_of<E: FlowEnv>(env: &mut E, c: &[u8], id: u32) -> u32 {
    env.callr(
        MON_OF,
        &mut [Arg::V(rd32(c, ofs::POKECON)), Arg::V(id & 0xFF)],
        c,
    )
}

/// `021b8d8c(server, pokecon, id)`.
fn view_slot<E: FlowEnv>(env: &mut E, c: &[u8], id: u32) -> u32 {
    env.callr(
        VIEW_SLOT,
        &mut [
            Arg::V(rd32(c, ofs::SERVER)),
            Arg::V(rd32(c, ofs::POKECON)),
            Arg::V(id),
        ],
        c,
    )
}

/// `021b8e70(server, id)`.
fn view_pos<E: FlowEnv>(env: &mut E, c: &[u8], id: u32) -> u32 {
    env.callr(VIEW_POS, &mut [Arg::V(rd32(c, ofs::SERVER)), Arg::V(id)], c)
}

fn is_quiet<E: FlowEnv>(env: &mut E) -> bool {
    env.call(IS_QUIET, &mut [Arg::P]) != 0
}

fn fx_busy<E: FlowEnv>(env: &mut E) -> bool {
    env.call(FX_BUSY, &mut []) != 0
}

/// The common one-shot shape: `callee(021b9934(pokecon, u8 args[0]), extra...)`, answer 1.
fn mon_relay<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8], callee: u32, extra: &[u32]) -> u32 {
    let mon = mon_of(env, c, arg(args, 0));
    let mut a: Vec<Arg<'_>> = vec![Arg::V(mon)];
    a.extend(extra.iter().map(|&v| Arg::V(v)));
    env.callr(callee, &mut a, c);
    1
}

/// Message ids of the experience message by kind `args[1]` (ROM table 0x021EFF80, `021d2fd8`).
const EXP_MESSAGES: [u16; 4] = [0x3D, 0x3E, 0x3F, 0x40];

/// C: `sub_021d2fd8`: command 0x46 (experience gain; 5-state jump table). 0: gauge animation `021f7cfc(021b8e70(server, u8 a0),
/// u16 a5, u8 a1, a2, a4)`. 1: when `021f7eb4()` is done: a2 (level up) -> message 0x41 with the mon's level `021d5440(021b98ac(
/// pokecon, u8 a0))` and the fanfare `021f883c(0x518)`; else message `EXP_MESSAGES[a1]` (a1 < 4) or 0x3D; show it. 2: message
/// done -> a2 ? `021ba2d4(server, mon)` : finished. 3: when `02005f38()` is done: `021f7c0c()`, `02005d90(0x47C, 0xFFFF)`,
/// `021b92a0(server)`, then a3 -> message 0x42 (level), else finished. 4: message done. Other states: finished.
pub fn cmd_exp_gain<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    let a = |i| arg(args, i);
    let server = rd32(c, ofs::SERVER);
    let pokecon = rd32(c, ofs::POKECON);
    let mon_by_id =
        |env: &mut E| env.callr(0x021B_98AC, &mut [Arg::V(pokecon), Arg::V(a(0) & 0xFF)], c);
    match rd32(state, 0) {
        0 => {
            let p = view_pos(env, c, a(0) & 0xFF);
            env.callr(
                0x021F_7CFC,
                &mut [
                    Arg::V(p),
                    Arg::V(a(5) & 0xFFFF),
                    Arg::V(a(1) & 0xFF),
                    Arg::V(a(2)),
                    Arg::V(a(4)),
                ],
                c,
            );
            state_inc(state);
            0
        }
        1 => {
            if fx_busy(&mut Lend::r(env, c)) {
                return 0;
            }
            if a(2) != 0 {
                let mon = mon_by_id(env);
                msg_set(&mut Lend::r(env, c), 1, 0x41);
                let lv = env.callr(0x021D_5440, &mut [Arg::V(mon)], c);
                msg_arg(&mut Lend::r(env, c), lv);
                env.callr(0x021F_883C, &mut [Arg::V(0x518)], c);
            } else {
                let id = if a(1) < 4 {
                    EXP_MESSAGES[a(1) as usize] as u32
                } else {
                    0x3D
                };
                msg_set(&mut Lend::r(env, c), 1, id);
            }
            msg_show(env, c);
            state_inc(state);
            0
        }
        2 => {
            if !msg_done(env, c) {
                return 0;
            }
            if a(2) == 0 {
                return 1;
            }
            let mon = mon_by_id(env);
            env.callr(0x021B_A2D4, &mut [Arg::V(server), Arg::V(mon)], c);
            state_inc(state);
            0
        }
        3 => {
            if env.callr(0x0200_5F38, &mut [], c) != 0 {
                return 0;
            }
            env.callr(0x021F_7C0C, &mut [], c);
            env.callr(0x0200_5D90, &mut [Arg::V(0x47C), Arg::V(0xFFFF)], c);
            env.callr(0x021B_92A0, &mut [Arg::V(server)], c);
            if a(3) == 0 {
                return 1;
            }
            let mon = mon_by_id(env);
            msg_set(&mut Lend::r(env, c), 1, 0x42);
            let lv = env.callr(0x021D_5440, &mut [Arg::V(mon)], c);
            msg_arg(&mut Lend::r(env, c), lv);
            msg_show(env, c);
            state_inc(state);
            0
        }
        4 => {
            if msg_done(env, c) {
                state_inc(state);
            }
            0
        }
        _ => 1,
    }
}

/// C: `sub_021d3138`: command 0x47 (EV / effort gauge): 0: `021f7d44(021b8e70(server, u8 a0), u16 a1)`. 1: when `021f7eb4()` is
/// done: message 0x43. 2: message done -> 1. Other states: 0 (never finishes, as in the original).
pub fn cmd_gauge_message<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let p = view_pos(env, c, arg(args, 0) & 0xFF);
            env.callr(
                0x021F_7D44,
                &mut [Arg::V(p), Arg::V(arg(args, 1) & 0xFFFF)],
                c,
            );
            state_inc(state);
            0
        }
        1 => {
            if !fx_busy(&mut Lend::r(env, c)) {
                msg_set(&mut Lend::r(env, c), 1, 0x43);
                msg_show(env, c);
                state_inc(state);
            }
            0
        }
        2 => msg_done(env, c) as u32,
        _ => 0,
    }
}

/// C: `sub_021d31a0`: command 0x48 (party slot status, k = u8 a0, slot = u8 a1): 0: `021b9c50(021b9aec(pokecon, k), slot, 0, 0)`,
/// `021eaa6c(view, k, slot)`. 1: when `021eaad8(view)`: slot 1 -> 1; else message (slot 3 ? 0x26 / 0x27 : 0x28 / 0x29 by the
/// side flag `021b9f70(server, u8 a0)`) through `021ea678(view, id, args)`. 2: message done -> 1. Other states: 0.
pub fn cmd_party_slot<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    let view = rd32(c, ofs::VIEW);
    match rd32(state, 0) {
        0 => {
            let k = arg(args, 0) & 0xFF;
            let slot = arg(args, 1) & 0xFF;
            let party = env.callr(
                0x021B_9AEC,
                &mut [Arg::V(rd32(c, ofs::POKECON)), Arg::V(k)],
                c,
            );
            env.callr(
                0x021B_9C50,
                &mut [Arg::V(party), Arg::V(slot), Arg::V(0), Arg::V(0)],
                c,
            );
            env.callr(0x021E_AA6C, &mut [Arg::V(view), Arg::V(k), Arg::V(slot)], c);
            state_inc(state);
            0
        }
        1 => {
            if env.callr(0x021E_AAD8, &mut [Arg::V(view)], c) == 0 {
                return 0;
            }
            let slot = arg(args, 1) & 0xFF;
            if slot == 1 {
                return 1;
            }
            let f = env.callr(
                0x021B_9F70,
                &mut [Arg::V(rd32(c, ofs::SERVER)), Arg::V(arg(args, 0) & 0xFF)],
                c,
            ) & 0xFF;
            let id = match (slot == 3, f != 0) {
                (true, true) => 0x26,
                (true, false) => 0x27,
                (false, true) => 0x28,
                (false, false) => 0x29,
            };
            env.callr(0x021E_A678, &mut [Arg::V(view), Arg::V(id), Arg::P], c);
            state_inc(state);
            0
        }
        2 => msg_done(env, c) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d324c`: command 0x49 (ability change with the ability window; k = u8 a0, s = `021b8d8c(server,
/// pokecon, k)` first): 0: quiet -> `BattleMon::set_ability(mon, a1)` at once, 1; else `021ea768(view, s, 0)`. 1: window open
/// (`021ea778(view, s)`) -> set_ability, `021ea7a8(view, s)`. 2: `021ea7b8(view, s)` -> next. Other states: 1.
pub fn cmd_ability_change<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    let k = arg(args, 0) & 0xFF;
    let view = rd32(c, ofs::VIEW);
    let s = view_slot(env, c, k);
    match rd32(state, 0) {
        0 => {
            if is_quiet(&mut Lend::r(env, c)) {
                let mon = mon_of(env, c, k);
                env.callr(0x021D_66D0, &mut [Arg::V(mon), Arg::V(arg(args, 1))], c);
                return 1;
            }
            env.callr(0x021E_A768, &mut [Arg::V(view), Arg::V(s), Arg::V(0)], c);
            state_inc(state);
            0
        }
        1 => {
            if env.callr(0x021E_A778, &mut [Arg::V(view), Arg::V(s)], c) != 0 {
                let mon = mon_of(env, c, k);
                env.callr(0x021D_66D0, &mut [Arg::V(mon), Arg::V(arg(args, 1))], c);
                env.callr(0x021E_A7A8, &mut [Arg::V(view), Arg::V(s)], c);
                state_inc(state);
            }
            0
        }
        2 => {
            if env.callr(0x021E_A7B8, &mut [Arg::V(view), Arg::V(s)], c) != 0 {
                state_inc(state);
            }
            0
        }
        _ => 1,
    }
}

/// Global wait counter of `021d32e0` (word at 0x021F3B64 + 4, `g` = the 0x20-byte block at 0x021F3B64).
const G_WAIT: usize = 4;

/// C: `sub_021d32e0`: command 0x4A (two mons swap abilities: a = u8 a0, b = u8 a1, abilities u16 a2 / u16 a3; 7-state jump
/// table; s / t = view slots of a / b, ma / mb = their BattleMons, all looked up first). 0: quiet or same side (`021b9a24(a, b)`)
/// -> set both abilities, 1; else open the window of a (`021ea768(view, s, 0)`), wait 8. 1: `021ea778(view, s)` (polled, answer
/// unused); while the wait runs count it down, then open the window of b. 2: both windows open -> next. 3: set both abilities,
/// wait 0. 4: after 8 more frames close the window of a (`021ea7a8`). 5: a closed (`021ea7b8`) -> close b. 6: b closed -> next.
/// Other states: 1. `g` is the overlay global block at 0x021F3B64.
pub fn cmd_ability_swap<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
    g: &mut [u8],
) -> u32 {
    let a = arg(args, 0) & 0xFF;
    let b = arg(args, 1) & 0xFF;
    let ab_a = arg(args, 2) & 0xFFFF;
    let ab_b = arg(args, 3) & 0xFFFF;
    let view = rd32(c, ofs::VIEW);
    let s = view_slot(env, c, a);
    let t = view_slot(env, c, b);
    let ma = mon_of(env, c, a);
    let mb = mon_of(env, c, b);
    let set_both = |env: &mut E| {
        env.call(0x021D_66D0, &mut [Arg::V(ma), Arg::V(ab_a)]);
        env.call(0x021D_66D0, &mut [Arg::V(mb), Arg::V(ab_b)]);
    };
    match rd32(state, 0) {
        0 => {
            if is_quiet(&mut Lend::r(env, c))
                || env.callr(0x021B_9A24, &mut [Arg::V(a), Arg::V(b)], c) != 0
            {
                set_both(env);
                return 1;
            }
            env.callr(0x021E_A768, &mut [Arg::V(view), Arg::V(s), Arg::V(0)], c);
            super::wr32(g, G_WAIT, 8);
            state_inc(state);
            0
        }
        1 => {
            env.callr(0x021E_A778, &mut [Arg::V(view), Arg::V(s)], c);
            let w = rd32(g, G_WAIT);
            if w != 0 {
                super::wr32(g, G_WAIT, w - 1);
                return 0;
            }
            env.callr(0x021E_A768, &mut [Arg::V(view), Arg::V(t), Arg::V(0)], c);
            state_inc(state);
            0
        }
        2 => {
            let ra = env.callr(0x021E_A778, &mut [Arg::V(view), Arg::V(s)], c) & 0xFF;
            let rb = env.callr(0x021E_A778, &mut [Arg::V(view), Arg::V(t)], c) & 0xFF;
            if ra != 0 && rb != 0 {
                state_inc(state);
            }
            0
        }
        3 => {
            set_both(env);
            super::wr32(g, G_WAIT, 0);
            state_inc(state);
            0
        }
        4 => {
            let w = rd32(g, G_WAIT).wrapping_add(1);
            super::wr32(g, G_WAIT, w);
            if w > 8 {
                env.callr(0x021E_A7A8, &mut [Arg::V(view), Arg::V(s)], c);
                state_inc(state);
            }
            0
        }
        5 => {
            if env.callr(0x021E_A7B8, &mut [Arg::V(view), Arg::V(s)], c) != 0 {
                env.callr(0x021E_A7A8, &mut [Arg::V(view), Arg::V(t)], c);
                state_inc(state);
            }
            0
        }
        6 => {
            if env.callr(0x021E_A7B8, &mut [Arg::V(view), Arg::V(t)], c) != 0 {
                state_inc(state);
            }
            0
        }
        _ => 1,
    }
}

/// C: `sub_021d344c`: command 0x4B (illusion broken): 0: s = `021b8d8c(server, pokecon, u8 a0)`, mon, quiet flag;
/// `BattleMon::clear_illusion_mon(mon)`, `021ea868(view, s, quiet)`. 1: `021ea878(view)` -> next. Other states: 1.
pub fn cmd_illusion_break<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    let view = rd32(c, ofs::VIEW);
    match rd32(state, 0) {
        0 => {
            let k = arg(args, 0) & 0xFF;
            let s = view_slot(env, c, k);
            let mon = mon_of(env, c, k);
            let quiet = env.callr(IS_QUIET, &mut [Arg::P], c);
            env.callr(0x021D_6C2C, &mut [Arg::V(mon)], c);
            env.callr(
                0x021E_A868,
                &mut [Arg::V(view), Arg::V(s), Arg::V(quiet)],
                c,
            );
            state_inc(state);
            0
        }
        1 => {
            if env.callr(0x021E_A878, &mut [Arg::V(view)], c) != 0 {
                state_inc(state);
            }
            0
        }
        _ => 1,
    }
}

/// C: `FUN_overlay_d_93__021d34b0`: command 0x4C (sound effect): state 0: quiet -> 1; else `021f7998(a0)`. Other states:
/// `021f7eb4()` done -> 1.
pub fn cmd_play_se<E: FlowEnv>(env: &mut E, state: &mut [u8], args: &[u8]) -> u32 {
    if rd32(state, 0) == 0 {
        if is_quiet(env) {
            return 1;
        }
        env.call(0x021F_7998, &mut [Arg::V(arg(args, 0))]);
        state_inc(state);
        0
    } else {
        (!fx_busy(env)) as u32
    }
}

/// C: `FUN_overlay_d_93__021d34e8`: command 0x4D (move animation on a position; p = `021b8e70(server, u8 a0)` first): state 0:
/// quiet -> 1; animations off (`!021b8590(server)`) and the veneer `021d3558(u16 a1)` (0x0689xxxx code) -> 1; else
/// `021ea3b8(view, p, u16 a1)`. Other states: `021ea3d4(view)` -> 1.
pub fn cmd_position_anim<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    let view = rd32(c, ofs::VIEW);
    let p = view_pos(env, c, arg(args, 0) & 0xFF);
    if rd32(state, 0) == 0 {
        if is_quiet(&mut Lend::r(env, c)) {
            return 1;
        }
        if env.callr(0x021B_8590, &mut [Arg::V(rd32(c, ofs::SERVER))], c) == 0
            && env.callr(0x021D_3558, &mut [Arg::V(arg(args, 1) & 0xFFFF)], c) != 0
        {
            return 1;
        }
        env.callr(
            0x021E_A3B8,
            &mut [Arg::V(view), Arg::V(p), Arg::V(arg(args, 1) & 0xFFFF)],
            c,
        );
        state_inc(state);
        0
    } else {
        (env.callr(0x021E_A3D4, &mut [Arg::V(view)], c) != 0) as u32
    }
}

/// C: `sub_021d3568`: command 0x4E (animation between two positions): state 0: quiet -> 1; else `021ea3e8(view, 021b8e70(server,
/// u8 a0), 021b8e70(server, u8 a1), u16 a2)`. Other states: `021ea3f8(view)` -> 1.
pub fn cmd_pair_anim<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    let view = rd32(c, ofs::VIEW);
    if rd32(state, 0) == 0 {
        if is_quiet(&mut Lend::r(env, c)) {
            return 1;
        }
        let p = view_pos(env, c, arg(args, 0) & 0xFF);
        let q = view_pos(env, c, arg(args, 1) & 0xFF);
        env.callr(
            0x021E_A3E8,
            &mut [
                Arg::V(view),
                Arg::V(p),
                Arg::V(q),
                Arg::V(arg(args, 2) & 0xFFFF),
            ],
            c,
        );
        state_inc(state);
        0
    } else {
        (env.callr(0x021E_A3F8, &mut [Arg::V(view)], c) != 0) as u32
    }
}

/// C: `FUN_overlay_d_93__021d35c8`: command 0x4F (form change): 0: p = `021b8e70(server, 021b8d8c(server, pokecon, u8 a0))`,
/// `BattleMon::change_form(mon, u8 a1)`, `021ea888(view, p)`. 1: `021ea8a4(view)` -> next. Other states: 1.
pub fn cmd_form_change<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    let view = rd32(c, ofs::VIEW);
    match rd32(state, 0) {
        0 => {
            let s = view_slot(env, c, arg(args, 0) & 0xFF);
            let p = view_pos(env, c, s);
            let mon = mon_of(env, c, arg(args, 0));
            env.callr(
                0x021D_66D8,
                &mut [Arg::V(mon), Arg::V(arg(args, 1) & 0xFF)],
                c,
            );
            env.callr(0x021E_A888, &mut [Arg::V(view), Arg::V(p)], c);
            state_inc(state);
            0
        }
        1 => {
            if env.callr(0x021E_A8A4, &mut [Arg::V(view)], c) != 0 {
                state_inc(state);
            }
            0
        }
        _ => 1,
    }
}

/// C: `FUN_overlay_d_93__021d3630`: command 0x57 (open the ability window of a mon; s = `021b8d8c(server, pokecon, u8 a0)`
/// first): 0: quiet -> 1; `021ea768(view, s, 1)`. 1: `021ea778(view, s)` -> next. Other states: 1.
pub fn cmd_ability_window_open<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
) -> u32 {
    let view = rd32(c, ofs::VIEW);
    let s = view_slot(env, c, arg(args, 0) & 0xFF);
    match rd32(state, 0) {
        0 => {
            if is_quiet(&mut Lend::r(env, c)) {
                return 1;
            }
            env.callr(0x021E_A768, &mut [Arg::V(view), Arg::V(s), Arg::V(1)], c);
            state_inc(state);
            0
        }
        1 => {
            if env.callr(0x021E_A778, &mut [Arg::V(view), Arg::V(s)], c) != 0 {
                state_inc(state);
            }
            0
        }
        _ => 1,
    }
}

/// C: `sub_021d3688`: command 0x58 (close the ability window; s = `021b8d8c(server, pokecon, u8 a0)` first): 0: `021ea788(view, s)`.
/// 1: `021ea798(view, s)` -> 1. Other states: 0.
pub fn cmd_ability_window_close<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
) -> u32 {
    let view = rd32(c, ofs::VIEW);
    let s = view_slot(env, c, arg(args, 0) & 0xFF);
    match rd32(state, 0) {
        0 => {
            env.callr(0x021E_A788, &mut [Arg::V(view), Arg::V(s)], c);
            state_inc(state);
            0
        }
        1 => (env.callr(0x021E_A798, &mut [Arg::V(view), Arg::V(s)], c) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d36cc`: command 0x01: `BattleMon::damage(mon, u16 a1)`.
pub fn cmd_mon_damage<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_5F90, &[arg(args, 1) & 0xFFFF])
}

/// C: `FUN_overlay_d_93__021d36ec`: command 0x02: `BattleMon::heal(mon, u16 a1)`.
pub fn cmd_mon_heal<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_5FA4, &[arg(args, 1) & 0xFFFF])
}

/// C: `FUN_overlay_d_93__021d370c`: command 0x04: `BattleMon::move_spend_pp(mon, u8 a1, u8 a2)`.
pub fn cmd_mon_spend_pp<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(
        env,
        c,
        args,
        0x021D_5608,
        &[arg(args, 1) & 0xFF, arg(args, 2) & 0xFF],
    )
}

/// C: `FUN_overlay_d_93__021d3734`: command 0x05: `BattleMon::move_spend_orig_pp(mon, u8 a1, u8 a2)`.
pub fn cmd_mon_spend_orig_pp<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(
        env,
        c,
        args,
        0x021D_5640,
        &[arg(args, 1) & 0xFF, arg(args, 2) & 0xFF],
    )
}

/// C: `FUN_overlay_d_93__021d375c`: command 0x06: `BattleMon::move_mark_used(mon, u8 a1)`.
pub fn cmd_mon_mark_used<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_56CC, &[arg(args, 1) & 0xFF])
}

/// C: `FUN_overlay_d_93__021d377c`: command 0x03: `BattleMon::faint(mon)`.
pub fn cmd_mon_faint<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_5FB8, &[])
}

/// C: `FUN_overlay_d_93__021d3794`: command 0x07: `BattleMon::move_restore_pp(mon, u8 a1, u8 a2)`.
pub fn cmd_mon_restore_pp<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(
        env,
        c,
        args,
        0x021D_5674,
        &[arg(args, 1) & 0xFF, arg(args, 2) & 0xFF],
    )
}

/// C: `FUN_overlay_d_93__021d37b8` (`sub_021d37b8`): command 0x08: `BattleMon::move_restore_orig_pp(mon, u8 a1, u8 a2)`.
pub fn cmd_mon_restore_orig_pp<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(
        env,
        c,
        args,
        0x021D_56A0,
        &[arg(args, 1) & 0xFF, arg(args, 2) & 0xFF],
    )
}

/// C: `FUN_overlay_d_93__021d37dc` (`sub_021d37dc`): command 0x09: `BattleMon::raise_stage(mon, a1, u8 a2)`.
pub fn cmd_mon_raise_stage<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(
        env,
        c,
        args,
        0x021D_5DC0,
        &[arg(args, 1), arg(args, 2) & 0xFF],
    )
}

/// C: `FUN_overlay_d_93__021d37fc` (`sub_021d37fc`): command 0x0A: `BattleMon::lower_stage(mon, a1, u8 a2)`.
pub fn cmd_mon_lower_stage<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(
        env,
        c,
        args,
        0x021D_5E38,
        &[arg(args, 1), arg(args, 2) & 0xFF],
    )
}

/// C: `FUN_overlay_d_93__021d381c` (`sub_021d381c`): command 0x0B: `BattleMon::set_stage(mon, k, u8 args[k])` for k = 1..7.
pub fn cmd_mon_set_stages<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let mon = mon_of(env, c, arg(args, 0));
    for k in 1..=7 {
        env.callr(
            0x021D_5EA8,
            &mut [Arg::V(mon), Arg::V(k), Arg::V(arg(args, k as usize) & 0xFF)],
            c,
        );
    }
    1
}

/// C: `FUN_overlay_d_93__021d3894`: command 0x0C: `BattleMon::clear_negative_stages(mon)`.
pub fn cmd_mon_clear_negative_stages<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_5EFC, &[])
}

/// C: `FUN_overlay_d_93__021d38ac`: command 0x0D: `BattleMon::reset_stages(mon)`.
pub fn cmd_mon_reset_stages<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_5F08, &[])
}

/// C: `FUN_overlay_d_93__021d38c4`: command 0x0E: `BattleMon::add_crit_boost(mon, a1)`.
pub fn cmd_mon_add_crit_boost<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_5F38, &[arg(args, 1)])
}

/// C: `FUN_overlay_d_93__021d38e0`: command 0x0F: `BattleMon::set_sick(mon, a1, a2)`.
pub fn cmd_mon_set_sick<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_6034, &[arg(args, 1), arg(args, 2)])
}

/// C: `FUN_overlay_d_93__021d3900` (`sub_021d3900`): command 0x10: `BattleMon::cure_major_status(mon)`.
pub fn cmd_mon_cure_major_status<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_61F0, &[])
}

/// C: `FUN_overlay_d_93__021d3918`: command 0x11: `BattleMon::cure_sick(mon, a1)`.
pub fn cmd_mon_cure_sick<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_6238, &[arg(args, 1)])
}

/// C: `FUN_overlay_d_93__021d3934`: command 0x12 (switch in), from the client's own copy of the command arguments (`c+0x138` side,
/// `c+0x13C` slot a, `c+0x140` slot b, u8 each) and u16 a3: party = `021b9aec(pokecon, side)`; a != b ->
/// `BattleParty_SwapSlots(party, a, b)` (`021b9c10`); `m = 021b9bf0(party, a)`, `BattleMon::set_148(m, u16 a3)`,
/// `BattleMon::reset_on_replace(m)`.
pub fn cmd_switch_in<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let a = rd32(c, ofs::CMD_ARGS + 4) & 0xFF;
    let b = rd32(c, ofs::CMD_ARGS + 8) & 0xFF;
    let party = env.callr(
        0x021B_9AEC,
        &mut [
            Arg::V(rd32(c, ofs::POKECON)),
            Arg::V(rd32(c, ofs::CMD_ARGS) & 0xFF),
        ],
        c,
    );
    if a != b {
        env.callr(0x021B_9C10, &mut [Arg::V(party), Arg::V(a), Arg::V(b)], c);
    }
    let m = env.callr(0x021B_9BF0, &mut [Arg::V(party), Arg::V(a)], c);
    env.callr(
        0x021D_63EC,
        &mut [Arg::V(m), Arg::V(arg(args, 3) & 0xFFFF)],
        c,
    );
    env.callr(0x021D_6580, &mut [Arg::V(m)], c);
    1
}

/// C: `FUN_overlay_d_93__021d3988`: command 0x16: `BattleMon::set_type_pair(mon, u16 a1)`.
pub fn cmd_mon_set_type_pair<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_66B0, &[arg(args, 1) & 0xFFFF])
}

/// C: `FUN_overlay_d_93__021d39a8`: command 0x15 (sick ticks): the mon ids `list = 021d8090(a2)` (8-byte out list), then for
/// rounds i < a1: `e = 021d3a10(i)` (veneer to 0x0689xxxx, not ported), 0 -> stop; `BattleMon::tick_sick(m, e, 0, 0)` on every
/// listed mon `m = 021b9934(pokecon, list[j])` (j < a0) that has not fainted (`BattleMon::is_fainted`).
pub fn cmd_tick_sicks<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let k = arg(args, 0);
    let n = arg(args, 1);
    let mut list = [0u8; 8];
    env.callr(
        0x021D_8090,
        &mut [Arg::V(arg(args, 2)), Arg::Out(&mut list)],
        c,
    );
    let mut i = 0u32;
    while i < n {
        let e = env.callr(0x021D_3A10, &mut [Arg::V(i)], c);
        if e == 0 {
            break;
        }
        let mut j = 0u32;
        while j < k {
            let m = env.callr(
                MON_OF,
                &mut [
                    Arg::V(rd32(c, ofs::POKECON)),
                    Arg::V(list[j as usize] as u32),
                ],
                c,
            );
            if env.callr(0x021D_5B68, &mut [Arg::V(m)], c) == 0 {
                env.callr(
                    0x021D_6064,
                    &mut [Arg::V(m), Arg::V(e), Arg::V(0), Arg::V(0)],
                    c,
                );
            }
            j += 1;
        }
        i += 1;
    }
    1
}

/// C: `FUN_overlay_d_93__021d3a20`: command 0x17: `BattleMon::set_u14(mon, u16 a1)` (`021d6730`).
pub fn cmd_mon_set_u14<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_6730, &[arg(args, 1) & 0xFFFF])
}

/// C: `FUN_overlay_d_93__021d3a40`: command 0x18: `BattleMon::record_move(mon, u8 a1, a2, u8 a3, u16 a4, u16 a5)`.
pub fn cmd_mon_record_move<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let a = |i| arg(args, i);
    mon_relay(
        env,
        c,
        args,
        0x021D_6744,
        &[a(1) & 0xFF, a(2), a(3) & 0xFF, a(4) & 0xFFFF, a(5) & 0xFFFF],
    )
}

/// C: `FUN_overlay_d_93__021d3a7c`: command 0x19: `BattleMon::set_flag_b(mon, a1)`.
pub fn cmd_mon_set_flag_b<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_5FE4, &[arg(args, 1)])
}

/// C: `FUN_overlay_d_93__021d3a98`: command 0x1A: `BattleMon::clear_flag_b(mon, a1)`.
pub fn cmd_mon_clear_flag_b<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_6008, &[arg(args, 1)])
}

/// C: `FUN_overlay_d_93__021d3ab4`: command 0x1B: `BattleMon::set_flag_a(mon, a1)`.
pub fn cmd_mon_set_flag_a<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_5FC0, &[arg(args, 1)])
}

/// C: `FUN_overlay_d_93__021d3ad0`: command 0x1C: `BattleMon::clear_flag_a(mon, a1)`.
pub fn cmd_mon_clear_flag_a<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_6440, &[arg(args, 1)])
}

/// C: `FUN_overlay_d_93__021d3aec`: command 0x1D: `BattleMon::set_ability(mon, a1)`.
pub fn cmd_mon_set_ability<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_66D0, &[arg(args, 1)])
}

/// C: `FUN_overlay_d_93__021d3b08`: command 0x1E: `BattleMon::held_item` setter `021d5b44(mon, u16 a1)`.
pub fn cmd_mon_set_item<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_5B44, &[arg(args, 1) & 0xFFFF])
}

/// C: `FUN_overlay_d_93__021d3b28`: command 0x1F: `BattleMon::move_replace(mon, u8 a1, u16 a4, u8 a2, a3)`.
pub fn cmd_mon_move_replace<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let a = |i| arg(args, i);
    mon_relay(
        env,
        c,
        args,
        0x021D_5734,
        &[a(1) & 0xFF, a(4) & 0xFFFF, a(2) & 0xFF, a(3)],
    )
}

/// C: `FUN_overlay_d_93__021d3b5c`: command 0x20: `BattleMon::reset_on_battle_end(mon)`.
pub fn cmd_mon_reset_on_battle_end<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_64F0, &[])
}

/// C: `FUN_overlay_d_93__021d3b74`: command 0x21: `021ef9f4(c.view_side, a0, a1, 0)`.
pub fn cmd_view_side_ef9f4<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    env.callr(
        0x021E_F9F4,
        &mut [
            Arg::V(rd32(c, ofs::VIEW_SIDE)),
            Arg::V(arg(args, 0)),
            Arg::V(arg(args, 1)),
            Arg::V(0),
        ],
        c,
    );
    1
}

/// C: `FUN_overlay_d_93__021d3b88` (`sub_021d3b88`): command 0x22: `021efad8(c.view_side, a0, u8 a1)`.
pub fn cmd_view_side_efad8<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    env.callr(
        0x021E_FAD8,
        &mut [
            Arg::V(rd32(c, ofs::VIEW_SIDE)),
            Arg::V(arg(args, 0)),
            Arg::V(arg(args, 1) & 0xFF),
        ],
        c,
    );
    1
}

/// C: `FUN_overlay_d_93__021d3b9c`: command 0x23: `021efb34(c.view_side, u8 a0)`.
pub fn cmd_view_side_efb34<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    env.callr(
        0x021E_FB34,
        &mut [Arg::V(rd32(c, ofs::VIEW_SIDE)), Arg::V(arg(args, 0) & 0xFF)],
        c,
    );
    1
}

/// C: `sub_021d3bb0`: command 0x24: `021efa94(c.view_side, a0)`.
pub fn cmd_view_side_efa94<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    env.callr(
        0x021E_FA94,
        &mut [Arg::V(rd32(c, ofs::VIEW_SIDE)), Arg::V(arg(args, 0))],
        c,
    );
    1
}

/// C: `FUN_overlay_d_93__021d3bc0`: command 0x25: `BattleMon::set_u8_157(mon, a1, u8 a2)` (`021d69a0`).
pub fn cmd_mon_set_u8_157<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(
        env,
        c,
        args,
        0x021D_69A0,
        &[arg(args, 1), arg(args, 2) & 0xFF],
    )
}

/// C: `FUN_overlay_d_93__021d3be0` (`sub_021d3be0`): command 0x26: `BattleMon::copy_passable_state(mon(u8 a1), mon(u8 a0))`
/// (the a0 mon is looked up first).
pub fn cmd_mon_copy_passable<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let src = mon_of(env, c, arg(args, 0));
    let dst = mon_of(env, c, arg(args, 1));
    env.callr(0x021D_65C8, &mut [Arg::V(dst), Arg::V(src)], c);
    1
}

/// C: `FUN_overlay_d_93__021d3c0c`: command 0x27: `BattleMon::make_substitute(mon, u16 a1)`.
pub fn cmd_mon_make_substitute<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_6D5C, &[arg(args, 1) & 0xFFFF])
}

/// C: `FUN_overlay_d_93__021d3c2c`: command 0x28: `BattleMon::set_substitute_hp` entry `021d6d70(mon)`.
pub fn cmd_mon_clear_substitute<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_6D70, &[])
}

/// C: `sub_021d3c44`: command 0x29: a0 is this client (`c[0x1A6]`) -> `client1::counter_add(c, u8 a1)` (021cd9f4, PORTED: real
/// code on both sides); else `021b9b10(server, u8 a0)`.
pub fn cmd_counter_add<E: FlowEnv>(env: &mut E, c: &mut [u8], args: &[u8]) -> u32 {
    let id = arg(args, 0) & 0xFF;
    let n = arg(args, 1) & 0xFF;
    if id == c[ofs::CLIENT_ID] as u32 {
        super::client1::counter_add(c, n);
    } else {
        env.callh(
            0x021B_9B10,
            &mut [Arg::V(rd32(c, ofs::SERVER)), Arg::V(id)],
            c,
        );
    }
    1
}

/// C: `sub_021d3c70`: command 0x2A: `021b9d10(server, 021b9aec(pokecon, u8 a0), u8 a0)`.
pub fn cmd_party_9d10<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let k = arg(args, 0) & 0xFF;
    let x = env.callr(
        0x021B_9AEC,
        &mut [Arg::V(rd32(c, ofs::POKECON)), Arg::V(k)],
        c,
    );
    env.callr(
        0x021B_9D10,
        &mut [Arg::V(rd32(c, ofs::SERVER)), Arg::V(x), Arg::V(k)],
        c,
    );
    1
}

/// C: `sub_021d3c90`: command 0x2B: `021d6738(mon)` (an entry of `BattleMon::set_u14`).
pub fn cmd_mon_6738<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_6738, &[])
}

/// C: `FUN_overlay_d_93__021d3ca8`: command 0x2C: `BattleMon::cure_sicks_linked_to(mon, u8 a1)`.
pub fn cmd_mon_cure_linked<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_6264, &[arg(args, 1) & 0xFF])
}

/// C: `FUN_overlay_d_93__021d3cc8`: playback command 0x2D (BattleMon status block): `mon = 021b9934(pokecon, u8 args[0])`, then
/// `021d6848(mon, &{u16 args[4], u16 args[5], u8 args[3], u8 args[1], u8 args[2], pad})`. Returns 1 ("done").
pub fn relay_mon_block<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let w = |i: usize| super::rd32(args, 4 * i);
    let mon = env.callr(
        0x021B_9934,
        &mut [Arg::V(super::rd32(c, ofs::POKECON)), Arg::V(w(0) & 0xFF)],
        c,
    );
    let mut block = [0u8; 8];
    block[0..2].copy_from_slice(&(w(4) as u16).to_le_bytes());
    block[2..4].copy_from_slice(&(w(5) as u16).to_le_bytes());
    block[4] = w(3) as u8;
    block[5] = w(1) as u8;
    block[6] = w(2) as u8;
    env.callr(0x021D_6848, &mut [Arg::V(mon), Arg::In(&block[..7])], c);
    1
}

/// C: `sub_021d3cfc`: command 0x2E: `BattleMon::on_turn_start(mon)`, `BattleMon::clear_1f6(mon)`.
pub fn cmd_mon_turn_start<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let mon = mon_of(env, c, arg(args, 0));
    env.callr(0x021D_6408, &mut [Arg::V(mon)], c);
    env.callr(0x021D_6E88, &mut [Arg::V(mon)], c);
    1
}

/// C: `sub_021d3d1c`: command 0x2F: `021efc78(c.view_side, 0, 0)`.
pub fn cmd_view_side_efc78<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    env.callr(
        0x021E_FC78,
        &mut [Arg::V(rd32(c, ofs::VIEW_SIDE)), Arg::V(0), Arg::V(0)],
        c,
    );
    1
}

/// C: `sub_021d3d2c`: command 0x13: `BattleMon::set_stat(mon, a1, u16 a2)`.
pub fn cmd_mon_set_stat<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(
        env,
        c,
        args,
        0x021D_5970,
        &[arg(args, 1), arg(args, 2) & 0xFFFF],
    )
}

/// C: `FUN_overlay_d_93__021d3d50`: command 0x14: `BattleMon::set_weight(mon, u16 a1)`.
pub fn cmd_mon_set_weight<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    mon_relay(env, c, args, 0x021D_67D8, &[arg(args, 1) & 0xFFFF])
}

/// C: `FUN_overlay_d_93__021d3d70`: the client id `c[0x1A6]`.
pub fn client_id(c: &[u8]) -> u8 {
    c[ofs::CLIENT_ID]
}

/// C: `FUN_overlay_d_93__021d3d7c`: the party container `c+0x118`.
pub fn party(c: &[u8]) -> u32 {
    rd32(c, ofs::PARTY)
}

/// C: `sub_021d3d84`: tail call `021ef9a8(c.view_side)` (view-side accessor; called from overlay 95).
pub fn view_side_ef9a8<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    env.callr(0x021E_F9A8, &mut [Arg::V(rd32(c, ofs::VIEW_SIDE))], c)
}

/// C: `FUN_overlay_d_93__021d3d90`: the counter byte `c[0x1AB]`.
pub fn counter(c: &[u8]) -> u8 {
    c[ofs::COUNTER]
}

/// C: `FUN_overlay_d_93__021d3d9c`: the four trainer items `c.u16[0xE8 + 2i] = 021b9f84(server, client id, i)`.
pub fn load_trainer_items<E: FlowEnv>(env: &mut E, c: &mut [u8]) {
    for i in 0..4u32 {
        let v = env.callh(
            0x021B_9F84,
            &mut [
                Arg::V(rd32(c, ofs::SERVER)),
                Arg::V(c[ofs::CLIENT_ID] as u32),
                Arg::V(i),
            ],
            c,
        );
        let o = ofs::ITEMS + 2 * i as usize;
        c[o..o + 2].copy_from_slice(&(v as u16).to_le_bytes());
    }
}

/// Party-size thresholds of the trainer items (ROM table 0x021EFF74, `021d3dc4`): item slot i is usable while the party count
/// (`021b9b94`) is at most `ITEM_MIN_PARTY[i]`.
const ITEM_MIN_PARTY: [u8; 4] = [6, 4, 2, 1];

/// C: `FUN_overlay_d_93__021d3dc4`: the trainer item to use on `mon` (party container `party`): none while the mon has sick 0x13
/// (`BattleMon::has_sick`); else the first slot i < 4 with `ITEM_MIN_PARTY[i] >= 021b9b94(party)` (the scan stops at the first
/// smaller entry) whose item `c.u16[0xE8 + 2i]` is set and fits: kind 0x29 (`021d805c`) -> HP (`021d59b4(mon, 0xD)`) <=
/// `max_hp_div(mon, 4)`; else [`item_boosts_stat`] or [`item_cures_status`]. The slot is cleared and the item returned; 0 = none.
pub fn pick_trainer_item<E: FlowEnv>(env: &mut E, c: &mut [u8], mon: u32, party: u32) -> u16 {
    let count = env.callh(0x021B_9B94, &mut [Arg::V(party)], c);
    if env.callh(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x13)], c) != 0 {
        return 0;
    }
    for (i, &min) in ITEM_MIN_PARTY.iter().enumerate() {
        if (min as u32) < count {
            return 0;
        }
        let o = ofs::ITEMS + 2 * i;
        let item = rd16(c, o) as u32;
        if item == 0 {
            continue;
        }
        let fits = if env.callh(0x021D_805C, &mut [Arg::V(item), Arg::V(0x29)], c) != 0 {
            let hp = env.callh(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xD)], c);
            hp <= env.callh(0x021D_7B60, &mut [Arg::V(mon), Arg::V(4)], c)
        } else {
            item_boosts_stat(&mut Lend::h(env, c), item, mon) != 0
                || item_cures_status(&mut Lend::h(env, c), item, mon) != 0
        };
        if fits {
            c[o..o + 2].copy_from_slice(&0u16.to_le_bytes());
            return item as u16;
        }
    }
    0
}

/// `{item kind, stat}` pairs of the ROM table 0x021EFF94 (`021d3e70`): kind for `021d805c`, stat for `021d5d14`.
const ITEM_STAT_BOOSTS: [(u8, u8); 6] = [
    (0x1E, 1),
    (0x1F, 2),
    (0x20, 3),
    (0x21, 4),
    (0x22, 5),
    (0x23, 6),
];

/// C: `FUN_overlay_d_93__021d3e70`: does `item` help `mon`: the first table pair whose kind the item has (`021d805c(item, kind)`)
/// and whose stat can still rise (`BattleMon::can_change_stage(mon, stat, 1)`) answers 1; else kind 0x24 and crit stage 0
/// (`!BattleMon::crit_stage(mon)`) answers 1; else 0.
pub fn item_boosts_stat<E: FlowEnv>(env: &mut E, item: u32, mon: u32) -> u32 {
    for &(kind, stat) in &ITEM_STAT_BOOSTS {
        if env.call(0x021D_805C, &mut [Arg::V(item), Arg::V(kind as u32)]) != 0
            && env.call(
                0x021D_5D14,
                &mut [Arg::V(mon), Arg::V(stat as u32), Arg::V(1)],
            ) != 0
        {
            return 1;
        }
    }
    if env.call(0x021D_805C, &mut [Arg::V(item), Arg::V(0x24)]) != 0
        && env.call(0x021D_5F14, &mut [Arg::V(mon)]) == 0
    {
        return 1;
    }
    0
}

/// `{item kind, status flag}` pairs of the ROM table 0x021EFFA0 (`021d3ec8`): kind for `021d805c`, flag for `021d62c4`.
const ITEM_STATUS_CURES: [(u8, u8); 7] = [
    (18, 2),
    (19, 5),
    (20, 4),
    (21, 3),
    (22, 1),
    (23, 6),
    (24, 7),
];

/// C: `FUN_overlay_d_93__021d3ec8`: does `item` cure a status `mon` has: the first table pair whose kind the item has
/// (`021d805c(item, kind)`) and whose flag the mon has (`021d62c4(mon, flag)`) answers 1; else 0.
pub fn item_cures_status<E: FlowEnv>(env: &mut E, item: u32, mon: u32) -> u32 {
    for &(kind, flag) in &ITEM_STATUS_CURES {
        if env.call(0x021D_805C, &mut [Arg::V(item), Arg::V(kind as u32)]) != 0
            && env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(flag as u32)]) != 0
        {
            return 1;
        }
    }
    0
}

#[cfg(test)]
mod tests;
