//! `btl_client.c` part 4 (0x021D10B8-0x021D2FD8): the request steps 021d10b8 .. 021d1a74 and the first playback-table handlers
//! (B13-spec1 / B13-spec2, ported in B13b).
//!
//! Specs: `tools/rom-diff/svflow_specs/client4.spec` (`nowork`, the client object is `arg buf 0x1C0`, a step's state word
//! `arg buf 0x10`, a handler's argument words `arg buf 0x40`).
//!
//! * Request steps (`u32 step(client *c, u32 *state)`, request table 0x021EFFEC): 1 = the request is answered. The mode-2 entries
//!   (`021d127c`, `021d13fc`, `021d1708`, `021d1a60`) run their mode-0 step only when the client has a view (`c+0x54`).
//! * Playback handlers (`u32 handler(client *c, u32 *state, u32 *args)`, table [`PLAYBACK_TABLE`]): 1 = the command is played.
//! * The command player [`play_commands`] (`021d1a74`, request 8) copies the server's command frames from the adapter into the
//!   client's command queue and runs them through the playback table.
//!
//! Callees outside the area (battle view `c+0x54` 021eaxxx, the view-side object `c+0x34` 021efxxx, the sound layer 0200xxxx and
//! overlay 94, server queries 021b8xxx / 021b9xxx, BattleMon methods 021d5xxx-021d6xxx, the adapter 021eec20 and the other client
//! areas) are reached through `FlowEnv::call` (D3 / D7). Exceptions, as the specs require: the client2 port `021cdee8`
//! ([`super::client2::hp_drop_percent`]) and the cmd_queue `pop_u8` behind the thunk `021cd644` run as real code.
//!
//! Pointers the adapter hands out (`021eec20(adapter, &p)`) point into memory the client does not own; a port that dereferences
//! one takes that memory as `data` plus its address `data_base` (`p - data_base` is the offset). In the differential test it is
//! the spec's extra `arg buf`; in the game it is the adapter's receive buffer.

use super::{rd16, rd32, wr32};
use crate::server_flow::cmd_queue;
use crate::server_flow::env::{Arg, FlowEnv, Lend, Lent};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021D_10B8,
    0x021D_123C,
    0x021D_127C,
    0x021D_1290,
    0x021D_13A4,
    0x021D_13FC,
    0x021D_1410,
    0x021D_1708,
    0x021D_171C,
    0x021D_185C,
    0x021D_18EC,
    0x021D_1960,
    0x021D_1A18,
    0x021D_1A60,
    0x021D_1A74,
    0x021D_1BD0,
    0x021D_1C40,
    0x021D_1CAC,
    0x021D_1CF4,
    0x021D_1DF4,
    0x021D_1E6C,
    0x021D_1EB4,
    0x021D_1F10,
    0x021D_1F58,
    0x021D_1FB4,
    0x021D_2000,
    0x021D_2030,
    0x021D_20BC,
    0x021D_210C,
    0x021D_215C,
    0x021D_21D4,
    0x021D_2234,
    0x021D_2278,
    0x021D_2320,
    0x021D_2374,
    0x021D_23B8,
    0x021D_2424,
    0x021D_2490,
    0x021D_2510,
    0x021D_2580,
    0x021D_25C4,
    0x021D_2628,
    0x021D_266C,
    0x021D_2700,
    0x021D_2814,
    0x021D_284C,
    0x021D_28AC,
    0x021D_2930,
    0x021D_2990,
    0x021D_299C,
    0x021D_29CC,
    0x021D_2A20,
    0x021D_2C7C,
    0x021D_2CA8,
];

/// Client object offsets used by the client4 / client5 ports (layout table in [`super`]).
pub(super) mod ofs {
    pub const SERVER: usize = 0x000;
    pub const POKECON: usize = 0x004;
    pub const RECORD: usize = 0x010;
    pub const VIEW_SIDE: usize = 0x034;
    pub const ADAPTER: usize = 0x050;
    pub const VIEW: usize = 0x054;
    pub const RECORDER: usize = 0x0A0;
    pub const RESULT_PTR: usize = 0x0D8;
    pub const RESULT_SIZE: usize = 0x0DC;
    pub const ITEMS: usize = 0x0E8;
    pub const PARTY: usize = 0x118;
    pub const ACTION: usize = 0x124;
    pub const QUEUE: usize = 0x134;
    pub const CMD_ARGS: usize = 0x138;
    pub const CMD_ID: usize = 0x178;
    pub const CMD_HANDLER: usize = 0x17C;
    pub const CMD_STATE: usize = 0x180;
    pub const HP_REF_OWN: usize = 0x19A;
    pub const CLIENT_ID: usize = 0x1A6;
    pub const COUNTER: usize = 0x1AB;
    pub const VIEW_RESET: usize = 0x1A9;
    pub const REC_CHAPTER: usize = 0x1AE;
    pub const FLAGS: usize = 0x1B2;
}

/// Word `i` of a handler's argument record.
pub(super) fn arg(args: &[u8], i: usize) -> u32 {
    rd32(args, 4 * i)
}

/// `*state += 1`.
pub(super) fn state_inc(state: &mut [u8]) {
    let s = rd32(state, 0);
    wr32(state, 0, s.wrapping_add(1));
}

fn set_state(state: &mut [u8], v: u32) {
    wr32(state, 0, v);
}

/// `021eaf18(c+0x58, kind, id)`: start a message in the client's message buffer.
pub(super) fn msg_set<E: FlowEnv>(env: &mut E, kind: u32, id: u32) {
    env.call(0x021E_AF18, &mut [Arg::P, Arg::V(kind), Arg::V(id)]);
}

/// `021eaf50(c+0x58, v)`: append a message argument.
pub(super) fn msg_arg<E: FlowEnv>(env: &mut E, v: u32) {
    env.call(0x021E_AF50, &mut [Arg::P, Arg::V(v)]);
}

/// `021ea614(view, c+0x58)`: print the message.
pub(super) fn msg_show<E: FlowEnv>(env: &mut E, c: &[u8]) {
    env.callr(0x021E_A614, &mut [Arg::V(rd32(c, ofs::VIEW)), Arg::P], c);
}

/// `021ea710(view) != 0`: the printed message has finished.
pub(super) fn msg_done<E: FlowEnv>(env: &mut E, c: &[u8]) -> bool {
    env.callr(0x021E_A710, &mut [Arg::V(rd32(c, ofs::VIEW))], c) != 0
}

fn view_call<E: FlowEnv>(env: &mut E, c: &[u8], addr: u32, extra: &[u32]) -> u32 {
    let mut a: Vec<Arg<'_>> = vec![Arg::V(rd32(c, ofs::VIEW))];
    a.extend(extra.iter().map(|&v| Arg::V(v)));
    env.callr(addr, &mut a, c)
}

fn server_call<E: FlowEnv>(env: &mut E, c: &[u8], addr: u32, extra: &[u32]) -> u32 {
    let mut a: Vec<Arg<'_>> = vec![Arg::V(rd32(c, ofs::SERVER))];
    a.extend(extra.iter().map(|&v| Arg::V(v)));
    env.callr(addr, &mut a, c)
}

/// `021b904c(server, i)`: id of trainer i of the opponent side.
fn trainer_id<E: FlowEnv>(env: &mut E, c: &[u8], i: u32) -> u32 {
    server_call(env, c, 0x021B_904C, &[i])
}

/// `021b8d8c(server, pokecon, id)`: view slot of a mon id.
fn view_slot<E: FlowEnv>(env: &mut E, c: &[u8], id: u32) -> u32 {
    env.callr(
        0x021B_8D8C,
        &mut [
            Arg::V(rd32(c, ofs::SERVER)),
            Arg::V(rd32(c, ofs::POKECON)),
            Arg::V(id),
        ],
        c,
    )
}

/// `021b8dc0(server, pokecon, id)`: view slot of a mon id (0xFF = not on the field).
fn field_slot<E: FlowEnv>(env: &mut E, c: &[u8], id: u32) -> u32 {
    env.callr(
        0x021B_8DC0,
        &mut [
            Arg::V(rd32(c, ofs::SERVER)),
            Arg::V(rd32(c, ofs::POKECON)),
            Arg::V(id),
        ],
        c,
    )
}

/// `021b8e70(server, id)`: view position.
fn view_pos<E: FlowEnv>(env: &mut E, c: &[u8], id: u32) -> u32 {
    server_call(env, c, 0x021B_8E70, &[id])
}

/// `021b9934(pokecon, id)`: BattleMon.
fn mon_of<E: FlowEnv>(env: &mut E, c: &[u8], id: u32) -> u32 {
    env.callr(
        0x021B_9934,
        &mut [Arg::V(rd32(c, ofs::POKECON)), Arg::V(id)],
        c,
    )
}

/// `021cd9e0(c)` (client1, `force` stub): the client plays no animations.
fn is_quiet<E: FlowEnv>(env: &mut E, c: &[u8]) -> bool {
    env.call(0x021C_D9E0, &mut [Arg::PHost(c.as_ptr() as usize)]) != 0
}

/// `021b8590(server)`: battle animations are on.
fn anims_on<E: FlowEnv>(env: &mut E, c: &[u8]) -> bool {
    server_call(env, c, 0x021B_8590, &[]) != 0
}

/// `021f7eb4()` (overlay 94): an effect / sound sequence is still running.
fn fx_busy<E: FlowEnv>(env: &mut E) -> bool {
    env.call(0x021F_7EB4, &mut []) != 0
}

/// `021f7998(id)` (overlay 94): start a sound effect sequence.
fn fx_start<E: FlowEnv>(env: &mut E, id: u32) {
    env.call(0x021F_7998, &mut [Arg::V(id)]);
}

/// `021eec20(adapter, &p)` (adapter, `force` stub): size of the received data and its address `p`.
fn adapter_data<E: FlowEnv>(env: &mut E, c: &[u8]) -> (u32, u32) {
    let mut p = [0u8; 4];
    let size = env.callr(
        0x021E_EC20,
        &mut [Arg::V(rd32(c, ofs::ADAPTER)), Arg::Out(&mut p)],
        c,
    );
    (size, u32::from_le_bytes(p))
}

/// The answer block: result pointer `c+0xD8` = `&c[0x124]` (the action record), size 4.
fn answer_action(c: &mut [u8], c_addr: u32) {
    wr32(c, ofs::RESULT_PTR, c_addr.wrapping_add(ofs::ACTION as u32));
    wr32(c, ofs::RESULT_SIZE, 4);
}

/// C: `FUN_overlay_d_93__021d10b8`: request 4, mode 0 (6-state jump table; switch-in prompt). `data` / `data_base`: the
/// adapter's receive memory (`p` of `021eec20`), `c_addr` the client's address. 0: `p[0] == 0x1F` -> state 4; else message 0x14
/// with `p[0]` (`021eaf18(c+0x58, 1, 0x14)`, args 1 and `p[0]`, `021eaf78(c+0x58, 0xFF)`), state 1. 1: `021ea710(view)` (answer
/// unused); when `021ea720(view)`: yes / no window (`021eaf18` on `c+0x58` (3, 6) and `c+0x7C` (3, 7), `021eabc4(view, c+0x58,
/// c+0x7C, 1)`), state 2. 2: message done and `021eac54(view, &ans)`: yes -> state 4; no -> party list (`021d0b10(c, 0, 1,
/// c+0x184, c+0x190)`, `021e9a78(view, c+0x184, 0, 0, c+0x190)`), state 3. 3: `021e9b20(view)` -> cancelled (veneer `021d121c`) or
/// slot n = veneer `021d122c` outside 1..5 -> state 4; else `ActionRecord::set_switch(c+0x124, 0, n)`, state 5. 4:
/// `ActionRecord::set_none(c+0x124)`, state 5, answer the record (1). 5: answer the record (1). Other states: 0.
pub fn req_switch_prompt<E: FlowEnv>(
    env: &mut E,
    c: &mut [u8],
    c_addr: u32,
    state: &mut [u8],
    data: &[u8],
    data_base: u32,
) -> u32 {
    let view = rd32(c, ofs::VIEW);
    match rd32(state, 0) {
        0 => {
            let (_, p) = adapter_data(env, c);
            let b = data[p.wrapping_sub(data_base) as usize] as u32;
            if b == 0x1F {
                set_state(state, 4);
                return 0;
            }
            msg_set(&mut Lend::h(env, c), 1, 0x14);
            msg_arg(&mut Lend::h(env, c), 1);
            msg_arg(&mut Lend::h(env, c), b);
            env.callh(0x021E_AF78, &mut [Arg::P, Arg::V(0xFF)], c);
            msg_show(env, c);
            set_state(state, 1);
            0
        }
        1 => {
            msg_done(env, c);
            if view_call(env, c, 0x021E_A720, &[]) != 0 {
                env.callh(0x021E_AF18, &mut [Arg::P, Arg::V(3), Arg::V(6)], c);
                env.callh(0x021E_AF18, &mut [Arg::P, Arg::V(3), Arg::V(7)], c);
                env.callh(
                    0x021E_ABC4,
                    &mut [Arg::V(view), Arg::P, Arg::P, Arg::V(1)],
                    c,
                );
                set_state(state, 2);
            }
            0
        }
        2 => {
            if !msg_done(env, c) {
                return 0;
            }
            let mut ans = [0u8; 4];
            if env.callh(0x021E_AC54, &mut [Arg::V(view), Arg::Out(&mut ans)], c) == 0 {
                return 0;
            }
            if u32::from_le_bytes(ans) != 0 {
                set_state(state, 4);
                return 0;
            }
            // (c, 0, 1, &c[0x184] party view block, &c[0x190])
            let (ca, blk, extra) = (
                c.as_ptr() as usize,
                c[0x184..].as_ptr() as usize,
                c[0x190..].as_ptr() as usize,
            );
            env.callh(
                0x021D_0B10,
                &mut [
                    Arg::PHost(ca),
                    Arg::V(0),
                    Arg::V(1),
                    Arg::PHost(blk),
                    Arg::PHost(extra),
                ],
                c,
            );
            // B19: the party list writes the player's choice into `c + 0x190` (pointers passed)
            env.callh(
                0x021E_9A78,
                &mut [
                    Arg::V(view),
                    Arg::PHost(blk),
                    Arg::V(0),
                    Arg::V(0),
                    Arg::PHost(extra),
                ],
                c,
            );
            set_state(state, 3);
            0
        }
        3 => {
            if view_call(env, c, 0x021E_9B20, &[]) == 0 {
                return 0;
            }
            if env.callh(0x021D_121C, &mut [Arg::P], c) != 0 {
                set_state(state, 4);
                return 0;
            }
            let n = env.callh(0x021D_122C, &mut [Arg::P], c);
            if n == 0 || n >= 6 {
                set_state(state, 4);
                return 0;
            }
            env.callh(0x021D_8394, &mut [Arg::P, Arg::V(0), Arg::V(n)], c);
            set_state(state, 5);
            0
        }
        4 => {
            env.callh(0x021D_8434, &mut [Arg::P], c);
            set_state(state, 5);
            answer_action(c, c_addr);
            1
        }
        5 => {
            answer_action(c, c_addr);
            1
        }
        _ => 0,
    }
}

/// C: `sub_021d123c`: request 0xA, mode 0 (record the server's data): `size = 021eec20(adapter, &p)`; recording (`c+0x10`) ->
/// `BattleRecord_Append(rec, p, size)` (`021ee608`); a recorder at `c+0xA0` -> `021bb4e0(recorder, p, size)` (server_flow port,
/// `force` stub) and `c[0x1AE] = 021ee638(p)`. Returns 1.
pub fn req_record_data<E: FlowEnv>(env: &mut E, c: &mut [u8]) -> u32 {
    let (size, p) = adapter_data(env, c);
    let rec = rd32(c, ofs::RECORD);
    if rec != 0 {
        env.callh(0x021E_E608, &mut [Arg::V(rec), Arg::V(p), Arg::V(size)], c);
    }
    let recorder = rd32(c, ofs::RECORDER);
    if recorder != 0 {
        env.callh(
            0x021B_B4E0,
            &mut [Arg::V(recorder), Arg::V(p), Arg::V(size)],
            c,
        );
        c[ofs::REC_CHAPTER] = env.callh(0x021E_E638, &mut [Arg::V(p)], c) as u8;
    }
    1
}

/// C: `sub_021d127c`: request 0xD, mode 2: with a view -> [`req_battle_result_bgm`], else 1.
pub fn req_battle_result_bgm_mode2<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    data: &[u8],
    data_base: u32,
) -> u32 {
    if rd32(c, ofs::VIEW) != 0 {
        req_battle_result_bgm(env, c, state, data, data_base)
    } else {
        1
    }
}

/// C: `FUN_overlay_d_93__021d1290`: request 0xD, mode 0 (4-state jump table; battle-end message and music). 0: flag bit 5 of
/// `c[0x1B2]` -> 1; id = `021b904c(server, 0)`, r = [`battle_result`], m = `(u8) 021b86c8(server)` (two trainers),
/// `021b916c(server, r)`; r 0 / 1 / 2 -> message 0x32|0x33 / 0x30|0x31 / 0x34|0x35 (by m) with id (and `021b904c(server, 1)` when
/// m); r 1 -> `02005d90(021b86dc(server), 0xFFFF)` (win BGM), state 1; else `02005e30(0x1E)` (fade out), state 2; other r -> 1.
/// 1: message done -> 1. 2: message done -> state 3. 3: `!02005e50()` -> `02005d28()`, 1. Other states: 0.
pub fn req_battle_result_bgm<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    data: &[u8],
    data_base: u32,
) -> u32 {
    match rd32(state, 0) {
        0 => {
            if (c[ofs::FLAGS] >> 5) & 1 != 0 {
                return 1;
            }
            let id = trainer_id(env, c, 0);
            let r = battle_result(env, c, data, data_base) as u32;
            let m = server_call(env, c, 0x021B_86C8, &[]) & 0xFF;
            server_call(env, c, 0x021B_916C, &[r]);
            let msg = match r {
                0 => 0x32,
                1 => 0x30,
                2 => 0x34,
                _ => return 1,
            } + (m != 0) as u32;
            msg_set(&mut Lend::r(env, c), 1, msg);
            msg_arg(&mut Lend::r(env, c), id);
            if m != 0 {
                let id1 = trainer_id(env, c, 1);
                msg_arg(&mut Lend::r(env, c), id1);
            }
            msg_show(env, c);
            if r == 1 {
                let bgm = server_call(env, c, 0x021B_86DC, &[]);
                env.callr(0x0200_5D90, &mut [Arg::V(bgm), Arg::V(0xFFFF)], c);
                state_inc(state);
            } else {
                env.callr(0x0200_5E30, &mut [Arg::V(0x1E)], c);
                let s = rd32(state, 0);
                set_state(state, s.wrapping_add(2));
            }
            0
        }
        1 => msg_done(env, c) as u32,
        2 => {
            if msg_done(env, c) {
                state_inc(state);
            }
            0
        }
        3 => {
            if env.callr(0x0200_5E50, &mut [], c) != 0 {
                return 0;
            }
            env.callr(0x0200_5D28, &mut [], c);
            1
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d13a4`: the battle result from the adapter data `p` (`021eec20`; `data` / `data_base` as in
/// [`req_switch_prompt`]): u16 `p[1]` 0 -> `021b902c((u8) p[0], c[0x1A6])` ? 0 : 1; 1 -> `021b902c(..)` ? 1 : 0; else `p[1]`.
pub fn battle_result<E: FlowEnv>(env: &mut E, c: &[u8], data: &[u8], data_base: u32) -> u16 {
    let (_, p) = adapter_data(env, c);
    let o = p.wrapping_sub(data_base) as usize;
    let h = rd16(data, o + 2);
    let side = |env: &mut E| {
        env.call(
            0x021B_902C,
            &mut [
                Arg::V(rd16(data, o) as u32 & 0xFF),
                Arg::V(c[ofs::CLIENT_ID] as u32),
            ],
        ) != 0
    };
    match h {
        0 => {
            if side(env) {
                rd16(data, o + 2)
            } else {
                1
            }
        }
        1 => {
            if side(env) {
                rd16(data, o + 2)
            } else {
                0
            }
        }
        _ => rd16(data, o + 2),
    }
}

/// C: `sub_021d13fc`: request 0xB, mode 2: with a view -> [`req_battle_end`], else 1.
pub fn req_battle_end_mode2<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    data: &[u8],
    data_base: u32,
) -> u32 {
    if rd32(c, ofs::VIEW) != 0 {
        req_battle_end(env, c, state, data, data_base)
    } else {
        1
    }
}

/// C: `FUN_overlay_d_93__021d1410`: request 0xB, mode 0 (battle end against trainers; 12-state jump table). 0: [`battle_result`]
/// 1 (won): win BGM `021f878c(021b86dc(server))`, message 0x2C (one trainer: `021b904c(server, 0) == 021b904c(server, 1)`) or
/// 0x2D with both ids, state 1; else `021ba1a8(server, 0x100)` -> 1, or message 0x36 with `021b8ff4(server)` twice, state 9.
/// 1: message done -> trainer sprite in (`021f7f58(021ba020(server, id0), 9, 0, 0, 0)`, `021f7998(0x270)`), state 2. 2: effect
/// done -> `021ea6b0(view, 021ba004(server, id0), 1)`, state 3. 3: message done -> two trainers ? (`021f7998(0x271)`, state 4) :
/// state 7. 4: effect done -> the same for trainer 1, state 5. 5: effect done -> `021ea6b0` for trainer 1, state 6. 6: message
/// done -> state 7. 7: prize money `021b919c(server)` -> message 0x3A (`021b8ff4`, money); `021b9210(server)` ? state 8 : state
/// 0xB. 8: message done -> `021b9210` -> message 0x3B, state 0xB. 9: message done -> `021b9244(server)` -> message 0x38 (client
/// id, value), state 0xA. 0xA: message done -> message 0x39 (client id), state 0xB. 0xB: message done -> 1. Other states: 1.
pub fn req_battle_end<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    data: &[u8],
    data_base: u32,
) -> u32 {
    let st = rd32(state, 0);
    let next = |state: &mut [u8], v: u32| {
        set_state(state, v);
        0
    };
    match st {
        0 => {
            let r = battle_result(env, c, data, data_base);
            if r == 1 {
                let bgm = server_call(env, c, 0x021B_86DC, &[]);
                env.callr(0x021F_878C, &mut [Arg::V(bgm)], c);
                let id0 = trainer_id(env, c, 0);
                let id1 = trainer_id(env, c, 1);
                if id0 == id1 {
                    msg_set(&mut Lend::r(env, c), 1, 0x2C);
                    msg_arg(&mut Lend::r(env, c), id0);
                } else {
                    msg_set(&mut Lend::r(env, c), 1, 0x2D);
                    msg_arg(&mut Lend::r(env, c), id0);
                    msg_arg(&mut Lend::r(env, c), id1);
                }
                msg_show(env, c);
                return next(state, 1);
            }
            if server_call(env, c, 0x021B_A1A8, &[0x100]) != 0 {
                return 1;
            }
            let m = server_call(env, c, 0x021B_8FF4, &[]);
            msg_set(&mut Lend::r(env, c), 1, 0x36);
            msg_arg(&mut Lend::r(env, c), m);
            msg_arg(&mut Lend::r(env, c), m);
            msg_show(env, c);
            next(state, 9)
        }
        1 => {
            if !msg_done(env, c) {
                return 0;
            }
            let id0 = trainer_id(env, c, 0);
            let t = server_call(env, c, 0x021B_A020, &[id0]);
            env.callr(
                0x021F_7F58,
                &mut [Arg::V(t), Arg::V(9), Arg::V(0), Arg::V(0), Arg::V(0)],
                c,
            );
            fx_start(&mut Lend::r(env, c), 0x270);
            next(state, 2)
        }
        2 => {
            if fx_busy(&mut Lend::r(env, c)) {
                return 0;
            }
            let id0 = trainer_id(env, c, 0);
            let x = server_call(env, c, 0x021B_A004, &[id0]);
            view_call(env, c, 0x021E_A6B0, &[x, 1]);
            next(state, 3)
        }
        3 => {
            if !msg_done(env, c) {
                return 0;
            }
            let id0 = trainer_id(env, c, 0);
            let id1 = trainer_id(env, c, 1);
            if id0 != id1 {
                fx_start(&mut Lend::r(env, c), 0x271);
                return next(state, 4);
            }
            next(state, 7)
        }
        4 => {
            if fx_busy(&mut Lend::r(env, c)) {
                return 0;
            }
            let id1 = trainer_id(env, c, 1);
            let t = server_call(env, c, 0x021B_A020, &[id1]);
            env.callr(
                0x021F_7F58,
                &mut [Arg::V(t), Arg::V(9), Arg::V(0), Arg::V(0), Arg::V(0)],
                c,
            );
            fx_start(&mut Lend::r(env, c), 0x270);
            next(state, 5)
        }
        5 => {
            if fx_busy(&mut Lend::r(env, c)) {
                return 0;
            }
            let id1 = trainer_id(env, c, 1);
            let x = server_call(env, c, 0x021B_A004, &[id1]);
            view_call(env, c, 0x021E_A6B0, &[x, 1]);
            next(state, 6)
        }
        6 => {
            if msg_done(env, c) {
                return next(state, 7);
            }
            0
        }
        7 => {
            let money = server_call(env, c, 0x021B_919C, &[]);
            if money != 0 {
                let m = server_call(env, c, 0x021B_8FF4, &[]);
                msg_set(&mut Lend::r(env, c), 1, 0x3A);
                msg_arg(&mut Lend::r(env, c), m);
                msg_arg(&mut Lend::r(env, c), money);
                msg_show(env, c);
            }
            if server_call(env, c, 0x021B_9210, &[]) == 0 {
                return next(state, 0xB);
            }
            next(state, 8)
        }
        8 => {
            if !msg_done(env, c) {
                return 0;
            }
            let v = server_call(env, c, 0x021B_9210, &[]);
            if v != 0 {
                let m = server_call(env, c, 0x021B_8FF4, &[]);
                msg_set(&mut Lend::r(env, c), 1, 0x3B);
                msg_arg(&mut Lend::r(env, c), m);
                msg_arg(&mut Lend::r(env, c), v);
                msg_show(env, c);
            }
            next(state, 0xB)
        }
        9 => {
            if !msg_done(env, c) {
                return 0;
            }
            let v = server_call(env, c, 0x021B_9244, &[]);
            if v != 0 {
                msg_set(&mut Lend::r(env, c), 1, 0x38);
                msg_arg(env, c[ofs::CLIENT_ID] as u32);
                msg_arg(&mut Lend::r(env, c), v);
                msg_show(env, c);
            }
            next(state, 0xA)
        }
        0xA => {
            if !msg_done(env, c) {
                return 0;
            }
            msg_set(&mut Lend::r(env, c), 1, 0x39);
            msg_arg(env, c[ofs::CLIENT_ID] as u32);
            msg_show(env, c);
            next(state, 0xB)
        }
        0xB => msg_done(env, c) as u32,
        _ => 1,
    }
}

/// C: `sub_021d1708`: request 0xC, mode 2: with a view -> [`req_trainer_message`], else 1.
pub fn req_trainer_message_mode2<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    data: &[u8],
    data_base: u32,
) -> u32 {
    if rd32(c, ofs::VIEW) != 0 {
        req_trainer_message(env, c, state, data, data_base)
    } else {
        1
    }
}

/// C: `FUN_overlay_d_93__021d171c`: request 0xC, mode 0 (trainer comes back and speaks; 6-state jump table). 0: flag bit 5 of
/// `c[0x1B2]` -> 1; r = [`battle_result`], `021b916c(server, r)`; r 1 -> `021f878c(021b86dc(server))`; r > 1 -> 1; trainer 0
/// slides in (`021f7f58(021ba020(server, id0), 9, 0, 0, 0)`, `021f7998(0x270)`) and speaks ([`trainer_speech`]`(c, r, 0)`), state
/// 1\. 1: effect done -> `021ea600(view)`, state 2. 2: message done -> one trainer ? 1 : (`021f7998(0x271)`, state 3). 3: effect
/// done -> r again, trainer 1 in and speaks, state 4. 4: effect done -> `021ea600(view)`, state 5. 5: message done -> 1. Other
/// states: 0.
pub fn req_trainer_message<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    data: &[u8],
    data_base: u32,
) -> u32 {
    match rd32(state, 0) {
        0 => {
            if (c[ofs::FLAGS] >> 5) & 1 != 0 {
                return 1;
            }
            let r = battle_result(env, c, data, data_base) as u32;
            server_call(env, c, 0x021B_916C, &[r]);
            if r == 1 {
                let bgm = server_call(env, c, 0x021B_86DC, &[]);
                env.callr(0x021F_878C, &mut [Arg::V(bgm)], c);
            }
            if r > 1 {
                return 1;
            }
            let id0 = trainer_id(env, c, 0);
            let t = server_call(env, c, 0x021B_A020, &[id0]);
            env.callr(
                0x021F_7F58,
                &mut [Arg::V(t), Arg::V(9), Arg::V(0), Arg::V(0), Arg::V(0)],
                c,
            );
            fx_start(&mut Lend::r(env, c), 0x270);
            trainer_speech(env, c, r, 0);
            set_state(state, 1);
            0
        }
        1 => {
            if !fx_busy(&mut Lend::r(env, c)) {
                view_call(env, c, 0x021E_A600, &[]);
                set_state(state, 2);
            }
            0
        }
        2 => {
            if !msg_done(env, c) {
                return 0;
            }
            let id0 = trainer_id(env, c, 0);
            let id1 = trainer_id(env, c, 1);
            if id0 == id1 {
                return 1;
            }
            fx_start(&mut Lend::r(env, c), 0x271);
            set_state(state, 3);
            0
        }
        3 => {
            if fx_busy(&mut Lend::r(env, c)) {
                return 0;
            }
            let r = battle_result(env, c, data, data_base) as u32;
            let id1 = trainer_id(env, c, 1);
            let t = server_call(env, c, 0x021B_A020, &[id1]);
            env.callr(
                0x021F_7F58,
                &mut [Arg::V(t), Arg::V(9), Arg::V(0), Arg::V(0), Arg::V(0)],
                c,
            );
            fx_start(&mut Lend::r(env, c), 0x270);
            trainer_speech(env, c, r, 1);
            set_state(state, 4);
            0
        }
        4 => {
            if !fx_busy(&mut Lend::r(env, c)) {
                view_call(env, c, 0x021E_A600, &[]);
                set_state(state, 5);
            }
            0
        }
        5 => msg_done(env, c) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d185c`: the speech of trainer `which` for result `r`: id = `021b904c(server, which)`, t =
/// `021ba02c(server, id, r)`; t and `02024450(t)` -> s = `0202435c(t, (c.u16[0x19A] & 0x7FFF) | 0x8000)`, `021ea5e4(view, s)`,
/// `02045808(s)`; else n = `021ba004(server, id)` -> message 7 / 0 with n and r (`021ea5f0(view, c+0x58)`).
pub fn trainer_speech<E: FlowEnv>(env: &mut E, c: &[u8], r: u32, which: u32) {
    let id = trainer_id(env, c, which);
    let t = server_call(env, c, 0x021B_A02C, &[id, r]);
    if t != 0 && env.callr(0x0202_4450, &mut [Arg::V(t)], c) != 0 {
        let v = ((rd16(c, ofs::HP_REF_OWN) as u32 & 0x7FFF) | 0x8000) & 0xFFFF;
        let s = env.callr(0x0202_435C, &mut [Arg::V(t), Arg::V(v)], c);
        view_call(env, c, 0x021E_A5E4, &[s]);
        env.callr(0x0204_5808, &mut [Arg::V(s)], c);
        return;
    }
    let n = server_call(env, c, 0x021B_A004, &[id]);
    if n != 0 {
        msg_set(&mut Lend::r(env, c), 7, 0);
        msg_arg(&mut Lend::r(env, c), n);
        msg_arg(&mut Lend::r(env, c), r);
        env.callr(0x021E_A5F0, &mut [Arg::V(rd32(c, ofs::VIEW)), Arg::P], c);
    }
}

/// C: `sub_021d18ec`: request 0xE, mode 0: 0: `!021ba1a8(server, 0x200)` and money `021b9210(server)` -> message 0x3B
/// (`021b8ff4`, money); state 1, then as state 1. 1: message done -> 1. Other states: 0.
pub fn req_money_message<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            if server_call(env, c, 0x021B_A1A8, &[0x200]) == 0 {
                let v = server_call(env, c, 0x021B_9210, &[]);
                if v != 0 {
                    let m = server_call(env, c, 0x021B_8FF4, &[]);
                    msg_set(&mut Lend::r(env, c), 1, 0x3B);
                    msg_arg(&mut Lend::r(env, c), m);
                    msg_arg(&mut Lend::r(env, c), v);
                    msg_show(env, c);
                }
            }
            state_inc(state);
            msg_done(env, c) as u32
        }
        1 => msg_done(env, c) as u32,
        _ => 0,
    }
}

/// C: `sub_021d1960`: request 0xF, mode 0 (4-state jump table; lost against a wild / trainer): 0: message 0x36 (client id),
/// state 1. 1: message done -> `021b9244(server)` -> message 0x37 (client id, value); state 2. 2: message done -> message 0x39
/// (client id), state 3. 3: message done -> state 4. Other states: 1.
pub fn req_lost_message<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8]) -> u32 {
    let id = c[ofs::CLIENT_ID] as u32;
    match rd32(state, 0) {
        0 => {
            msg_set(&mut Lend::r(env, c), 1, 0x36);
            msg_arg(&mut Lend::r(env, c), id);
            msg_show(env, c);
            state_inc(state);
            0
        }
        1 => {
            if !msg_done(env, c) {
                return 0;
            }
            let v = server_call(env, c, 0x021B_9244, &[]);
            if v != 0 {
                msg_set(&mut Lend::r(env, c), 1, 0x37);
                msg_arg(&mut Lend::r(env, c), id);
                msg_arg(&mut Lend::r(env, c), v);
                msg_show(env, c);
            }
            state_inc(state);
            0
        }
        2 => {
            if !msg_done(env, c) {
                return 0;
            }
            msg_set(&mut Lend::r(env, c), 1, 0x39);
            msg_arg(&mut Lend::r(env, c), id);
            msg_show(env, c);
            state_inc(state);
            0
        }
        3 => {
            if msg_done(env, c) {
                state_inc(state);
            }
            0
        }
        _ => 1,
    }
}

/// C: `sub_021d1a18`: request 0x10, mode 0: 0: message 0xB7, state 1. 1: message done -> state 2. 2: 1. Other states: 0.
pub fn req_message_b7<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            msg_set(&mut Lend::r(env, c), 1, 0xB7);
            msg_show(env, c);
            state_inc(state);
            0
        }
        1 => {
            if msg_done(env, c) {
                state_inc(state);
            }
            0
        }
        2 => 1,
        _ => 0,
    }
}

/// C: `sub_021d1a60`: request 8, mode 2: with a view -> [`play_commands`], else 1.
pub fn play_commands_mode2<E: FlowEnv>(
    env: &mut E,
    c: &mut [u8],
    state: &mut [u8],
    queue: &mut [u8],
    queue_base: u32,
    data: &[u8],
    data_base: u32,
) -> u32 {
    if rd32(c, ofs::VIEW) != 0 {
        play_commands(env, c, state, queue, queue_base, data, data_base)
    } else {
        1
    }
}

/// The playback table at 0x021F00DC: `{BtlCmd id, Thumb handler}` (0x5D entries), searched by [`play_commands`].
pub const PLAYBACK_TABLE: [(u32, u32); 0x5D] = [
    (0x5A, 0x021D_1E6D),
    (0x5C, 0x021D_1EB5),
    (0x5B, 0x021D_1F11),
    (0x5D, 0x021D_1F59),
    (0x59, 0x021D_1FB5),
    (0x30, 0x021D_2031),
    (0x31, 0x021D_20BD),
    (0x32, 0x021D_210D),
    (0x33, 0x021D_215D),
    (0x34, 0x021D_21D5),
    (0x35, 0x021D_2235),
    (0x36, 0x021D_2279),
    (0x39, 0x021D_2321),
    (0x3A, 0x021D_2375),
    (0x3B, 0x021D_1BD1),
    (0x3C, 0x021D_1CAD),
    (0x3D, 0x021D_1CF5),
    (0x37, 0x021D_2425),
    (0x38, 0x021D_23B9),
    (0x3E, 0x021D_29CD),
    (0x3F, 0x021D_2491),
    (0x40, 0x021D_2511),
    (0x41, 0x021D_2581),
    (0x42, 0x021D_25C5),
    (0x57, 0x021D_3631),
    (0x58, 0x021D_3689),
    (0x01, 0x021D_36CD),
    (0x02, 0x021D_36ED),
    (0x03, 0x021D_377D),
    (0x04, 0x021D_370D),
    (0x05, 0x021D_3735),
    (0x06, 0x021D_375D),
    (0x07, 0x021D_3795),
    (0x08, 0x021D_37B9),
    (0x09, 0x021D_37DD),
    (0x0A, 0x021D_37FD),
    (0x0B, 0x021D_381D),
    (0x0C, 0x021D_3895),
    (0x0D, 0x021D_38AD),
    (0x0E, 0x021D_38C5),
    (0x0F, 0x021D_38E1),
    (0x10, 0x021D_3901),
    (0x11, 0x021D_3919),
    (0x12, 0x021D_3935),
    (0x13, 0x021D_3D2D),
    (0x14, 0x021D_3D51),
    (0x16, 0x021D_3989),
    (0x15, 0x021D_39A9),
    (0x17, 0x021D_3A21),
    (0x18, 0x021D_3A41),
    (0x19, 0x021D_3A7D),
    (0x1A, 0x021D_3A99),
    (0x1B, 0x021D_3AB5),
    (0x1C, 0x021D_3AD1),
    (0x1D, 0x021D_3AED),
    (0x1E, 0x021D_3B09),
    (0x1F, 0x021D_3B29),
    (0x20, 0x021D_3B5D),
    (0x21, 0x021D_3B75),
    (0x22, 0x021D_3B89),
    (0x23, 0x021D_3B9D),
    (0x24, 0x021D_3BB1),
    (0x25, 0x021D_3BC1),
    (0x26, 0x021D_3BE1),
    (0x27, 0x021D_3C0D),
    (0x28, 0x021D_3C2D),
    (0x29, 0x021D_3C45),
    (0x2A, 0x021D_3C71),
    (0x2B, 0x021D_3C91),
    (0x2C, 0x021D_3CA9),
    (0x2D, 0x021D_3CC9),
    (0x2E, 0x021D_3CFD),
    (0x2F, 0x021D_3D1D),
    (0x43, 0x021D_2629),
    (0x44, 0x021D_266D),
    (0x45, 0x021D_2A21),
    (0x46, 0x021D_2FD9),
    (0x47, 0x021D_3139),
    (0x48, 0x021D_31A1),
    (0x49, 0x021D_324D),
    (0x4A, 0x021D_32E1),
    (0x4B, 0x021D_344D),
    (0x4C, 0x021D_34B1),
    (0x4D, 0x021D_34E9),
    (0x4E, 0x021D_3569),
    (0x4F, 0x021D_35C9),
    (0x50, 0x021D_2701),
    (0x51, 0x021D_2815),
    (0x52, 0x021D_284D),
    (0x53, 0x021D_28AD),
    (0x54, 0x021D_2931),
    (0x55, 0x021D_2991),
    (0x56, 0x021D_299D),
];

/// The "end of the command stream" id `021cd5e0` answers (`0x5E`).
const CMD_END: u32 = 0x5E;

/// C: `FUN_overlay_d_93__021d1a74`: request 8, mode 0: the command player (5-state jump table). `queue` / `queue_base`: the
/// command queue `c+0x134` points at; `data` / `data_base`: the adapter's receive memory. 0: `size = 021eec20(adapter, &p)`; a
/// recorder (`c+0xA0`), a chapter byte `c[0x1AE]` and flag bit 3 of `c[0x1B2]` -> `021bb570(recorder, c[0x1AE], p, size)`
/// (server_flow port, `force` stub) succeeding -> `021b9184(server)` and flag bit 3 off; then `c[0x1AE] = 0`. The frames are
/// copied to the queue (`MI_CpuCopy32(p, q+8, (u16) size)`, `q[0] = (u16) size`, `q[4] = 0`); a pending view reset `c[0x1A9]` ->
/// cleared and `021ea824(view)`; state 1, then as state 1. 1: the queue is read to its end (`q[4] == q[0]`) -> state 4, 1;
/// the cursor `021d3f4c(c+0x18)` asks to wait -> 1; state 2, then as 2. 2: `cmd = 021cd5e0(q, c+0x138)` (cmd_queue `read_cmd`,
/// `force` stub) -> `c+0x178`; 0x5E -> 1; an id of [`PLAYBACK_TABLE`] -> handler to `c+0x17C`, its state `c+0x180 = 0`, state 3,
/// then as 3; an unknown id -> state 1 and again. 3: `(c+0x17C)(c, c+0x180, c+0x138)` (through `env.call`) played -> state 1 and
/// again; else 0. 4: 1. Other states: 0.
pub fn play_commands<E: FlowEnv>(
    env: &mut E,
    c: &mut [u8],
    state: &mut [u8],
    queue: &mut [u8],
    queue_base: u32,
    data: &[u8],
    data_base: u32,
) -> u32 {
    let q_ofs = |c: &[u8]| rd32(c, ofs::QUEUE).wrapping_sub(queue_base) as usize;
    loop {
        let st = rd32(state, 0);
        // states fall through into the next one, as in the original
        let mut st_run = st;
        if st_run == 0 {
            let (size, p) = adapter_data(env, c);
            let recorder = rd32(c, ofs::RECORDER);
            if recorder != 0 && c[ofs::REC_CHAPTER] != 0 && (c[ofs::FLAGS] >> 3) & 1 != 0 {
                let ok = env.callh(
                    0x021B_B570,
                    &mut [
                        Arg::V(recorder),
                        Arg::V(c[ofs::REC_CHAPTER] as u32),
                        Arg::V(p),
                        Arg::V(size),
                    ],
                    c,
                );
                if ok != 0 {
                    server_call(env, c, 0x021B_9184, &[]);
                    c[ofs::FLAGS] &= !8;
                }
                c[ofs::REC_CHAPTER] = 0;
            }
            let n = (size & 0xFFFF) as usize;
            let q = q_ofs(c);
            let src = p.wrapping_sub(data_base) as usize;
            // MI_CpuCopy32 (word copy; the sizes are multiples of 4)
            queue[q + 8..q + 8 + n].copy_from_slice(&data[src..src + n]);
            wr32(queue, q, size & 0xFFFF);
            wr32(queue, q + 4, 0);
            if c[ofs::VIEW_RESET] != 0 {
                c[ofs::VIEW_RESET] = 0;
                view_call(env, c, 0x021E_A824, &[]);
            }
            state_inc(state);
            st_run = 1;
        }
        if st_run == 1 {
            let q = q_ofs(c);
            if rd32(queue, q + 4) == rd32(queue, q) {
                set_state(state, 4);
                return 1;
            }
            if env.call(0x021D_3F4C, &mut [Arg::PBuf(&mut c[0x18..0x28])]) != 0 {
                return 1;
            }
            state_inc(state);
            st_run = 2;
        }
        if st_run == 2 {
            let q = rd32(c, ofs::QUEUE);
            // the queue is lent with the client object (the reader works on both)
            let out = c[ofs::CMD_ARGS..].as_ptr() as usize;
            let cmd = env.call_l(
                0x021C_D5E0,
                &mut [Arg::V(q), Arg::PHost(out)],
                Lent {
                    bytes: vec![&mut *c, &mut *queue],
                    ..Lent::default()
                },
            );
            wr32(c, ofs::CMD_ID, cmd);
            if cmd == CMD_END {
                return 1;
            }
            match PLAYBACK_TABLE.iter().find(|e| e.0 == cmd) {
                None => {
                    set_state(state, 1);
                    continue;
                }
                Some(&(_, f)) => {
                    wr32(c, ofs::CMD_HANDLER, f);
                    wr32(c, ofs::CMD_STATE, 0);
                    state_inc(state);
                    st_run = 3;
                }
            }
        }
        match st_run {
            3 => {
                let f = rd32(c, ofs::CMD_HANDLER);
                let st = c[ofs::CMD_STATE..].as_ptr() as usize;
                let args = c[ofs::CMD_ARGS..].as_ptr() as usize;
                let ca = c.as_ptr() as usize;
                let lent = Lent {
                    bytes: vec![&mut *c, &mut *queue],
                    ..Lent::default()
                };
                if env.call_lent(
                    lent,
                    f & !1,
                    &mut [Arg::PHost(ca), Arg::PHost(st), Arg::PHost(args)],
                ) == 0
                {
                    return 0;
                }
                set_state(state, 1);
            }
            4 => return 1,
            _ => return 0,
        }
    }
}

/// C: `sub_021d1bd0`: command 0x3B (HP message): 0: id = [`hp_message_id`]`(c, u8 a0, &f)`; message id, f -> args a0 and a1
/// (else only a1), state 1. 1: message done -> 1. Other states: 0.
pub fn cmd_hp_message<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let mut f = [0u8; 4];
            let id = hp_message_id(env, c, arg(args, 0) & 0xFF, &mut f);
            msg_set(&mut Lend::r(env, c), 1, id);
            if rd32(&f, 0) != 0 {
                msg_arg(&mut Lend::r(env, c), arg(args, 0));
            }
            msg_arg(&mut Lend::r(env, c), arg(args, 1));
            msg_show(env, c);
            state_inc(state);
            0
        }
        1 => msg_done(env, c) as u32,
        _ => 0,
    }
}

/// C: `FUN_021d1c40`: message id for client `id`: this client (`c[0x1A6]`): `*out = 0`; battle type `021b8588(server)` or the
/// multi flag `021b86c0(server)` -> 0x1A; else by [`super::client2::hp_drop_percent`]`(c)`: >= 75 -> 0x1D, > 50 -> 0x1C, > 25 ->
/// 0x1B, != 0 -> 0x1A, 0 -> 0x19. Another client: `*out = 1`, `021b9f70(server, id)` ? 0x1E : 0x1F.
pub fn hp_message_id<E: FlowEnv>(env: &mut E, c: &[u8], id: u32, out: &mut [u8]) -> u32 {
    if id == c[ofs::CLIENT_ID] as u32 {
        wr32(out, 0, 0);
        if server_call(env, c, 0x021B_8588, &[]) != 0 || server_call(env, c, 0x021B_86C0, &[]) != 0
        {
            return 0x1A;
        }
        let p = super::client2::hp_drop_percent(env, c);
        if p >= 75 {
            0x1D
        } else if p > 50 {
            0x1C
        } else if p > 25 {
            0x1B
        } else if p != 0 {
            0x1A
        } else {
            0x19
        }
    } else {
        wr32(out, 0, 1);
        if server_call(env, c, 0x021B_9F70, &[id]) != 0 {
            0x1E
        } else {
            0x1F
        }
    }
}

/// C: `sub_021d1cac`: command 0x3C: 0: `021ea458(view, 021b8e70(server, u8 a0), u16 a1)`, state 1. 1: `021ea478(view)` -> 1.
/// Other states: 0.
pub fn cmd_hp_gauge<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let p = view_pos(env, c, arg(args, 0) & 0xFF);
            view_call(env, c, 0x021E_A458, &[p, arg(args, 1) & 0xFFFF]);
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A478, &[]) != 0) as u32,
        _ => 0,
    }
}

/// C: `sub_021d1cf4`: command 0x3D (send out a mon; a = u8 a0, b = u8 a1, message flag u8 a3): 0: flag -> n =
/// `021d5440(021b9920(pokecon, a, b))`; `021b8ce8(server, c[0x1A6], a)` ? (`021b9f70(server, a)` ? message 0xE with a : message
/// 0x11 with a) : (a is this client ? message [`own_send_out_message`] : message 0x11 with a), then n, show; state 1. 1: message
/// done -> `021ea488(view, 021b8bec(server, a, b), a, b)`, state 2. 2: `021ea4c4(view)` -> `021cdecc(c)` (client2, `force` stub),
/// 1\. Other states: 0.
pub fn cmd_send_out<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    let a = arg(args, 0) & 0xFF;
    let b = arg(args, 1) & 0xFF;
    let show = arg(args, 3) & 0xFF;
    match rd32(state, 0) {
        0 => {
            if show != 0 {
                let mon = env.callr(
                    0x021B_9920,
                    &mut [Arg::V(rd32(c, ofs::POKECON)), Arg::V(a), Arg::V(b)],
                    c,
                );
                let n = env.callr(0x021D_5440, &mut [Arg::V(mon)], c);
                let own = c[ofs::CLIENT_ID] as u32;
                if server_call(env, c, 0x021B_8CE8, &[own, a]) == 0 {
                    if own == a {
                        let id = own_send_out_message(env, c);
                        msg_set(&mut Lend::r(env, c), 1, id);
                    } else {
                        msg_set(&mut Lend::r(env, c), 1, 0x11);
                        msg_arg(&mut Lend::r(env, c), a);
                    }
                } else if server_call(env, c, 0x021B_9F70, &[a]) != 0 {
                    msg_set(&mut Lend::r(env, c), 1, 0xE);
                    msg_arg(&mut Lend::r(env, c), a);
                } else {
                    msg_set(&mut Lend::r(env, c), 1, 0x11);
                    msg_arg(&mut Lend::r(env, c), a);
                }
                msg_arg(&mut Lend::r(env, c), n);
                msg_show(env, c);
            }
            state_inc(state);
            0
        }
        1 => {
            if msg_done(env, c) {
                let x = server_call(env, c, 0x021B_8BEC, &[a, b]);
                view_call(env, c, 0x021E_A488, &[x, a, b]);
                state_inc(state);
            }
            0
        }
        2 => {
            if view_call(env, c, 0x021E_A4C4, &[]) != 0 {
                env.callr(0x021C_DECC, &mut [Arg::PHost(c.as_ptr() as usize)], c);
                return 1;
            }
            0
        }
        _ => 0,
    }
}

/// C: `FUN_021d1df4`: send-out message of this client by the active mon's HP: battle type `021b8588(server)` -> 0xB; mon =
/// `021b9920(pokecon, (u8) 021b9064(server, c[0x1A6], 0), 0)`; fainted (`BattleMon::is_fainted`) -> 0xB; by
/// `BattleMon::hp_percent_fx(mon)` (signed): >= 0x4D800 -> 0xB, >= 0x37000 -> 0x15, >= 0x20800 -> 0x16, >= 0xA000 -> 0x17, else
/// 0x18.
pub fn own_send_out_message<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    if server_call(env, c, 0x021B_8588, &[]) != 0 {
        return 0xB;
    }
    let pos = server_call(env, c, 0x021B_9064, &[c[ofs::CLIENT_ID] as u32, 0]) & 0xFF;
    let mon = env.callr(
        0x021B_9920,
        &mut [Arg::V(rd32(c, ofs::POKECON)), Arg::V(pos), Arg::V(0)],
        c,
    );
    if env.callr(0x021D_5B68, &mut [Arg::V(mon)], c) != 0 {
        return 0xB;
    }
    let r = env.callr(0x021D_5C44, &mut [Arg::V(mon)], c) as i32;
    if r >= 0x4D800 {
        0xB
    } else if r >= 0x37000 {
        0x15
    } else if r >= 0x20800 {
        0x16
    } else if r >= 0xA000 {
        0x17
    } else {
        0x18
    }
}

/// The message relays 0x5A / 0x5C / 0x5B / 0x5D: state 0: quiet -> 1; `print(view, u16 a0, &args[skip])`, state 1. 1: with
/// `se`: `021ea720(view)` -> `020061e4(a1)`; message done -> 1. Other states: 0.
fn message_relay<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
    print: u32,
    se: bool,
) -> u32 {
    match rd32(state, 0) {
        0 => {
            if is_quiet(&mut Lend::r(env, c), c) {
                return 1;
            }
            env.callr(
                print,
                &mut [
                    Arg::V(rd32(c, ofs::VIEW)),
                    Arg::V(arg(args, 0) & 0xFFFF),
                    Arg::P,
                ],
                c,
            );
            state_inc(state);
            0
        }
        1 => {
            if se && view_call(env, c, 0x021E_A720, &[]) != 0 {
                env.callr(0x0200_61E4, &mut [Arg::V(arg(args, 1))], c);
            }
            msg_done(env, c) as u32
        }
        _ => 0,
    }
}

/// C: `sub_021d1e6c`: command 0x5A: message `021ea678(view, u16 a0, &args[1])` (not when quiet).
pub fn cmd_message<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    message_relay(env, c, state, args, 0x021E_A678, false)
}

/// C: `sub_021d1eb4` (with the pieces `LAB_overlay_d_93__021d1ec8` / `LAB_overlay_d_93__021d1eec`): command 0x5C: message
/// `021ea678(view, u16 a0, &args[2])` with the sound effect a1 when the window asks for it (`021ea720`).
pub fn cmd_message_se<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    message_relay(env, c, state, args, 0x021E_A678, true)
}

/// C: `FUN_overlay_d_93__021d1f10`: command 0x5B: as [`cmd_message`] with the other window `021ea694`.
pub fn cmd_message_alt<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    message_relay(env, c, state, args, 0x021E_A694, false)
}

/// C: `sub_021d1f58`: command 0x5D: as [`cmd_message_se`] with the other window `021ea694`.
pub fn cmd_message_alt_se<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    message_relay(env, c, state, args, 0x021E_A694, true)
}

/// C: `FUN_overlay_d_93__021d1fb4`: command 0x59: 0: quiet -> 1; `021ea6f4(view, u8 a0, u16 a1)`, state 1. 1: message done ->
/// 1\. Other states: 0.
pub fn cmd_message_6f4<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            if is_quiet(&mut Lend::r(env, c), c) {
                return 1;
            }
            view_call(
                env,
                c,
                0x021E_A6F4,
                &[arg(args, 0) & 0xFF, arg(args, 1) & 0xFFFF],
            );
            state_inc(state);
            0
        }
        1 => msg_done(env, c) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2000`: animation parameter of move `mv` used by mon `id`: move 0xAE with a mon
/// (`021b98ac(pokecon, id)`) -> `021d80a8(mon)`; else `0201bd28(mv, 0x1B)` (waza parameter).
pub fn move_anim_param<E: FlowEnv>(env: &mut E, c: &[u8], mv: u32, id: u32) -> u32 {
    if mv == 0xAE {
        let mon = env.callr(
            0x021B_98AC,
            &mut [Arg::V(rd32(c, ofs::POKECON)), Arg::V(id)],
            c,
        );
        if mon != 0 {
            return env.callr(0x021D_80A8, &mut [Arg::V(mon)], c);
        }
    }
    env.callr(0x0201_BD28, &mut [Arg::V(mv), Arg::V(0x1B)], c)
}

/// C: `FUN_overlay_d_93__021d2030`: command 0x30 (move animation): 0: quiet or animations off -> 1; `021ea0a0(view, u8 a0, u8
/// a1, u16 a2, `[`move_anim_param`]`(c, u16 a2, u8 a0), u8 a3, 0)`, state 1. 1: `021ea0e4(view)` -> 1. Other states: 0.
pub fn cmd_move_anim<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            if is_quiet(&mut Lend::r(env, c), c) || !anims_on(env, c) {
                return 1;
            }
            let a0 = arg(args, 0) & 0xFF;
            let a1 = arg(args, 1) & 0xFF;
            let a2 = arg(args, 2) & 0xFFFF;
            let a3 = arg(args, 3) & 0xFF;
            let e = move_anim_param(env, c, a2, a0);
            view_call(env, c, 0x021E_A0A0, &[a0, a1, a2, e, a3, 0]);
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A0E4, &[]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d20bc`: command 0x31: 0: `021ea354(view, 021b8dc0(server, pokecon, u8 a0), a1 != 0)`, state 1. 1:
/// effect done (`!021f7eb4()`) -> 1. Other states: 0.
pub fn cmd_status_anim<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let s = field_slot(env, c, arg(args, 0) & 0xFF);
            view_call(env, c, 0x021E_A354, &[s, (arg(args, 1) != 0) as u32]);
            state_inc(state);
            0
        }
        1 => (!fx_busy(&mut Lend::r(env, c))) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d210c`: command 0x32: 0: `021ea0f4(view, u16 a2, 021b8d8c(server, pokecon, u8 a0), u8 a1)`, state
/// 1\. 1: `021ea120(view)` -> 1. Other states: 0.
pub fn cmd_effect_anim<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let s = view_slot(env, c, arg(args, 0) & 0xFF);
            view_call(
                env,
                c,
                0x021E_A0F4,
                &[arg(args, 2) & 0xFFFF, s, arg(args, 1) & 0xFF],
            );
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A120, &[]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d215c`: command 0x33: 0: n = u16 a0 values popped from the command queue (`c+0x134`, cmd_queue
/// [`cmd_queue::pop_u8`] through the thunk `021cd644`; an 8-byte list, n <= 8) -> `021ea280(view, n, a1, list, u16 a2)`, state
/// 1\. 1: `021ea2bc(view)` -> 1. Other states: 0. `queue` / `queue_base` as in [`play_commands`].
pub fn cmd_targets_anim<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
    queue: &mut [u8],
    queue_base: u32,
) -> u32 {
    match rd32(state, 0) {
        0 => {
            let n = arg(args, 0) & 0xFFFF;
            let mut list = [0u8; 8];
            let q = rd32(c, ofs::QUEUE).wrapping_sub(queue_base) as usize;
            let mut i: u8 = 0;
            while (i as i32) < n as i32 {
                list[i as usize] = cmd_queue::pop_u8(&mut queue[q..]);
                i = i.wrapping_add(1);
            }
            env.callr(
                0x021E_A280,
                &mut [
                    Arg::V(rd32(c, ofs::VIEW)),
                    Arg::V(n),
                    Arg::V(arg(args, 1)),
                    Arg::In(&list),
                    Arg::V(arg(args, 2) & 0xFFFF),
                ],
                c,
            );
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A2BC, &[]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d21d4`: command 0x34: 0: `021ea384(view, 021b8d8c(server, pokecon, u8 a0))`, state 1. 1:
/// `021ea3a0(view)` -> message 0x61 (`021ea678(view, 0x61, args)`), state 2. 2: message done -> 1. Other states: 0.
pub fn cmd_slot_anim_message<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
) -> u32 {
    match rd32(state, 0) {
        0 => {
            let s = view_slot(env, c, arg(args, 0) & 0xFF);
            view_call(env, c, 0x021E_A384, &[s]);
            state_inc(state);
            0
        }
        1 => {
            if view_call(env, c, 0x021E_A3A0, &[]) != 0 {
                env.callr(
                    0x021E_A678,
                    &mut [Arg::V(rd32(c, ofs::VIEW)), Arg::V(0x61), Arg::P],
                    c,
                );
                state_inc(state);
            }
            0
        }
        2 => msg_done(env, c) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2234`: command 0x35: 0: `021f81d0(a1, 021b8e70(server, 021b8d8c(server, pokecon, u8 a0)))`, state
/// 1\. 1: 1. Other states: 0.
pub fn cmd_fx_81d0<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let s = view_slot(env, c, arg(args, 0) & 0xFF);
            let p = view_pos(env, c, s);
            env.callr(0x021F_81D0, &mut [Arg::V(arg(args, 1)), Arg::V(p)], c);
            state_inc(state);
            0
        }
        1 => 1,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2278`: command 0x36 (hit animation): 0: not quiet -> `021ea0a0(view, s, s, 1, 0201bd28(1, 0x1B), 0,
/// 0)` with s = `021b8d8c(server, pokecon, u8 a0)`; state 1. 1: `021ea0e4(view)` -> s again, not quiet -> `020061e4(0x564)`;
/// `021ea0f4(view, 1, s, 1)`, state 2. 2: `021ea120(view)` -> 1. Other states: 0.
pub fn cmd_hit_anim<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            if !is_quiet(&mut Lend::r(env, c), c) {
                let s = view_slot(env, c, arg(args, 0) & 0xFF);
                let e = env.callr(0x0201_BD28, &mut [Arg::V(1), Arg::V(0x1B)], c);
                view_call(env, c, 0x021E_A0A0, &[s, s, 1, e, 0, 0]);
            }
            state_inc(state);
            0
        }
        1 => {
            if view_call(env, c, 0x021E_A0E4, &[]) != 0 {
                let s = view_slot(env, c, arg(args, 0) & 0xFF);
                if !is_quiet(&mut Lend::r(env, c), c) {
                    env.callr(0x0200_61E4, &mut [Arg::V(0x564)], c);
                }
                view_call(env, c, 0x021E_A0F4, &[1, s, 1]);
                state_inc(state);
            }
            0
        }
        2 => (view_call(env, c, 0x021E_A120, &[]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2320`: command 0x39 (faint): 0: `021ea40c(view, 021b8d8c(server, pokecon, u8 a0))`, state 1. 1:
/// `021ea428(view)` -> `BattleMon::reset_on_switch_out(021b9934(pokecon, u8 a0))`, 1. Other states: 0.
pub fn cmd_faint_anim<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let s = view_slot(env, c, arg(args, 0) & 0xFF);
            view_call(env, c, 0x021E_A40C, &[s]);
            state_inc(state);
            0
        }
        1 => {
            if view_call(env, c, 0x021E_A428, &[]) != 0 {
                let mon = mon_of(env, c, arg(args, 0) & 0xFF);
                env.callr(0x021D_646C, &mut [Arg::V(mon)], c);
                return 1;
            }
            0
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2374`: command 0x3A: 0: `021ea438(view, 021b8d8c(server, pokecon, u8 a0))`, state 1. 1:
/// `021ea448(view)` -> 1. Other states: 0.
pub fn cmd_slot_anim_438<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let s = view_slot(env, c, arg(args, 0) & 0xFF);
            view_call(env, c, 0x021E_A438, &[s]);
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A448, &[]) != 0) as u32,
        _ => 0,
    }
}

/// The 0x38 / 0x37 shape: p = `021b8e70(server, 021b8d8c(server, pokecon, u8 a0))` first; 0: quiet or animations off -> 1;
/// `start(view, p)`, state 1. 1: `021ea7e8(view, p)` -> 1. Other states: 0.
fn position_anim<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
    start: u32,
) -> u32 {
    let s = view_slot(env, c, arg(args, 0) & 0xFF);
    let p = view_pos(env, c, s);
    match rd32(state, 0) {
        0 => {
            if is_quiet(&mut Lend::r(env, c), c) || !anims_on(env, c) {
                return 1;
            }
            view_call(env, c, start, &[p]);
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A7E8, &[p]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d23b8`: command 0x38: position animation `021ea7c8(view, p)`, done by `021ea7e8(view, p)`.
pub fn cmd_position_anim_7c8<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
) -> u32 {
    position_anim(env, c, state, args, 0x021E_A7C8)
}

/// C: `FUN_overlay_d_93__021d2424`: command 0x37: position animation `021ea7d8(view, p)`, done by `021ea7e8(view, p)`.
pub fn cmd_position_anim_7d8<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
) -> u32 {
    position_anim(env, c, state, args, 0x021E_A7D8)
}

/// `{message id, sound id}` of the weather / field kinds 0..4 (ROM table 0x021EFFAE, 4-byte records, `021d2490`).
const FIELD_FX: [(u16, u16); 5] = [
    (0x54, 0x26B),
    (0x54, 0x26B),
    (0x55, 0x268),
    (0x57, 0x269),
    (0x56, 0x26A),
];

/// C: `FUN_overlay_d_93__021d2490`: command 0x3F (k = u8 a0): 0: `021ef9c0(c.view_side, k, u16 a1)`; quiet -> 1; k < 5 ->
/// `021f7998(FIELD_FX[k].1)`; state 1. 1: effect done -> `021ea678(view, FIELD_FX[k].0, 0)`, state 2. 2: message done -> 1.
/// Other states: 0. (Deviation: for k >= 5 the original's state 1 reads past the table; the port uses message 0 there.)
pub fn cmd_field_effect<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    let k = arg(args, 0) & 0xFF;
    match rd32(state, 0) {
        0 => {
            env.callr(
                0x021E_F9C0,
                &mut [
                    Arg::V(rd32(c, ofs::VIEW_SIDE)),
                    Arg::V(k),
                    Arg::V(arg(args, 1) & 0xFFFF),
                ],
                c,
            );
            if is_quiet(&mut Lend::r(env, c), c) {
                return 1;
            }
            if k < 5 {
                fx_start(&mut Lend::r(env, c), FIELD_FX[k as usize].1 as u32);
            }
            state_inc(state);
            0
        }
        1 => {
            if !fx_busy(&mut Lend::r(env, c)) {
                let id = FIELD_FX.get(k as usize).map_or(0, |e| e.0 as u32);
                view_call(env, c, 0x021E_A678, &[id, 0]);
                state_inc(state);
            }
            0
        }
        2 => msg_done(env, c) as u32,
        _ => 0,
    }
}

/// C: `sub_021d2510`: command 0x40 (5-way jump table on u8 a0): 0: kind 0 or > 4 -> 1; kinds 1 / 2 / 3 / 4 -> message 0x59 /
/// 0x5A / 0x5C / 0x5B (`021ea678(view, id, 0)`), state 1. 1: message done -> `021ef9c8(c.view_side)`, 1. Other states: 0.
pub fn cmd_field_end<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let id = match arg(args, 0) & 0xFF {
                1 => 0x59,
                2 => 0x5A,
                3 => 0x5C,
                4 => 0x5B,
                _ => return 1,
            };
            view_call(env, c, 0x021E_A678, &[id, 0]);
            state_inc(state);
            0
        }
        1 => {
            if msg_done(env, c) {
                env.callr(0x021E_F9C8, &mut [Arg::V(rd32(c, ofs::VIEW_SIDE))], c);
                return 1;
            }
            0
        }
        _ => 0,
    }
}

/// The 0x41 / 0x43 shape: 0: `021ea384(view, 021b8d8c(server, pokecon, u8 a0))`, state 1. 1: `021ea3a0(view)` -> 1.
fn slot_anim_384<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let s = view_slot(env, c, arg(args, 0) & 0xFF);
            view_call(env, c, 0x021E_A384, &[s]);
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A3A0, &[]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2580`: command 0x41: slot animation `021ea384` / `021ea3a0`.
pub fn cmd_slot_anim_41<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    slot_anim_384(env, c, state, args)
}

/// C: `FUN_overlay_d_93__021d25c4`: command 0x42: s = `021b8d8c(server, pokecon, u8 a0)` first; 0: quiet or animations off -> 1;
/// `021ea838(view, s)`, state 1. 1: `021ea850(view, s)` -> 1. Other states: 0.
pub fn cmd_slot_anim_838<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    let s = view_slot(env, c, arg(args, 0) & 0xFF);
    match rd32(state, 0) {
        0 => {
            if is_quiet(&mut Lend::r(env, c), c) || !anims_on(env, c) {
                return 1;
            }
            view_call(env, c, 0x021E_A838, &[s]);
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A850, &[s]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2628`: command 0x43: as [`cmd_slot_anim_41`].
pub fn cmd_slot_anim_43<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    slot_anim_384(env, c, state, args)
}

/// C: `FUN_overlay_d_93__021d266c`: command 0x44 (swap two party slots; k = u8 a0, slots u8 a1 / u8 a2): 0: a / b =
/// `021b8dec(server, slot)`, p / q = `021b8e70(server, slot)`, `BattleParty_SwapSlots(021b9aec(pokecon, k), a, b)`
/// (`021b9c10`), `021ea904(view, k, p, q, a, b)`, state 1. 1: `021ea950(view)` -> 1. Other states: 0.
pub fn cmd_swap_slots<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let k = arg(args, 0) & 0xFF;
            let s1 = arg(args, 1) & 0xFF;
            let s2 = arg(args, 2) & 0xFF;
            let a = server_call(env, c, 0x021B_8DEC, &[s1]);
            let b = server_call(env, c, 0x021B_8DEC, &[s2]);
            let p = view_pos(env, c, s1);
            let q = view_pos(env, c, s2);
            let party = env.callr(
                0x021B_9AEC,
                &mut [Arg::V(rd32(c, ofs::POKECON)), Arg::V(k)],
                c,
            );
            env.callr(0x021B_9C10, &mut [Arg::V(party), Arg::V(a), Arg::V(b)], c);
            view_call(env, c, 0x021E_A904, &[k, p, q, a, b]);
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A950, &[]) != 0) as u32,
        _ => 0,
    }
}

/// One 0x50 swap: k = u8 `args[i]`, slot = u8 `args[i + 1]`: x = `021b8bec(server, k, slot)`, y = `021b8bec(server, k, 1)`,
/// positions p / q of x / y, `BattleParty_SwapSlots(021b9aec(pokecon, k), slot, 1)`, `021ea904(view, k, p, q, slot, 1)`.
fn swap_with_first<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8], i: usize) {
    let k = arg(args, i) & 0xFF;
    let slot = arg(args, i + 1) & 0xFF;
    let x = server_call(env, c, 0x021B_8BEC, &[k, slot]);
    let y = server_call(env, c, 0x021B_8BEC, &[k, 1]);
    let p = view_pos(env, c, x);
    let q = view_pos(env, c, y);
    let party = env.callr(
        0x021B_9AEC,
        &mut [Arg::V(rd32(c, ofs::POKECON)), Arg::V(k)],
        c,
    );
    env.callr(
        0x021B_9C10,
        &mut [Arg::V(party), Arg::V(slot), Arg::V(1)],
        c,
    );
    view_call(env, c, 0x021E_A904, &[k, p, q, slot, 1]);
}

/// C: `FUN_overlay_d_93__021d2700`: command 0x50 (rotation; 4-state jump table): 0: `021f7998(0x281)`, state 1. 1: effect done ->
/// swap (a0, a1) with slot 1, state 2. 2: `021ea950(view)` -> swap (a2, a3), state 3. 3: `021ea950(view)` -> 1. Other states: 0.
pub fn cmd_rotate<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            fx_start(&mut Lend::r(env, c), 0x281);
            state_inc(state);
            0
        }
        1 => {
            if !fx_busy(&mut Lend::r(env, c)) {
                swap_with_first(env, c, args, 0);
                state_inc(state);
            }
            0
        }
        2 => {
            if view_call(env, c, 0x021E_A950, &[]) != 0 {
                swap_with_first(env, c, args, 2);
                state_inc(state);
            }
            0
        }
        3 => (view_call(env, c, 0x021E_A950, &[]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2814`: command 0x51: 0: `021f7e94(021b8e70(server, u8 a0))`, state 1. 1: effect done -> 1. Other
/// states: 0.
pub fn cmd_fx_7e94<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let p = view_pos(env, c, arg(args, 0) & 0xFF);
            env.callr(0x021F_7E94, &mut [Arg::V(p)], c);
            state_inc(state);
            0
        }
        1 => (!fx_busy(&mut Lend::r(env, c))) as u32,
        _ => 0,
    }
}

/// C: `sub_021d284c`: command 0x52: 0: p = `021b8e70(server, u8 a0)`; quiet -> `021f88bc(1)`; `021f7ea4(p)`, state 1. 1: effect
/// done -> quiet -> `021f88bc(0)`; 1. Other states: 0.
pub fn cmd_fx_7ea4<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let p = view_pos(env, c, arg(args, 0) & 0xFF);
            if is_quiet(&mut Lend::r(env, c), c) {
                env.callr(0x021F_88BC, &mut [Arg::V(1)], c);
            }
            env.callr(0x021F_7EA4, &mut [Arg::V(p)], c);
            state_inc(state);
            0
        }
        1 => {
            if fx_busy(&mut Lend::r(env, c)) {
                return 0;
            }
            if is_quiet(&mut Lend::r(env, c), c) {
                env.callr(0x021F_88BC, &mut [Arg::V(0)], c);
            }
            1
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d28ac`: command 0x53 (transform): 0: m1 / m2 = `021b9934(pokecon, u8 a0 / u8 a1)`, s1 / s2 =
/// `021b8dc0(server, pokecon, u8 a0 / u8 a1)`; `BattleMon::transform_into(m1, m2)`; `021ea8b4(view, s1, s2)`, state 1. 1:
/// `021ea8d4(view)` -> 1. Other states: 0.
pub fn cmd_transform<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8], args: &[u8]) -> u32 {
    match rd32(state, 0) {
        0 => {
            let a = arg(args, 0) & 0xFF;
            let b = arg(args, 1) & 0xFF;
            let m1 = mon_of(env, c, a);
            let m2 = mon_of(env, c, b);
            let s1 = field_slot(env, c, a);
            let s2 = field_slot(env, c, b);
            env.callr(0x021D_6C50, &mut [Arg::V(m1), Arg::V(m2)], c);
            view_call(env, c, 0x021E_A8B4, &[s1, s2]);
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A8D4, &[]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2930`: command 0x54: 0: quiet -> 1; `021ea1b8(view, u16 a2, 021b8d8c(server, pokecon, u8 a0),
/// GetTypeEffectivenessClass(a1))` (`021d7ad4`), state 1. 1: `021ea1f0(view)` -> 1. Other states: 0.
pub fn cmd_effectiveness_anim<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
) -> u32 {
    match rd32(state, 0) {
        0 => {
            if is_quiet(&mut Lend::r(env, c), c) {
                return 1;
            }
            let s = view_slot(env, c, arg(args, 0) & 0xFF);
            let t = env.callr(0x021D_7AD4, &mut [Arg::V(arg(args, 1))], c);
            view_call(env, c, 0x021E_A1B8, &[arg(args, 2) & 0xFFFF, s, t]);
            state_inc(state);
            0
        }
        1 => (view_call(env, c, 0x021E_A1F0, &[]) != 0) as u32,
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2990`: command 0x55 (one shot): `021f878c(args[0])` (overlay 94 sound call, not ported); returns 1.
pub fn play_sound<E: FlowEnv>(env: &mut E, args: &[u8]) -> u32 {
    env.call(0x021F_878C, &mut [Arg::V(rd32(args, 0))]);
    1
}

/// C: `FUN_overlay_d_93__021d299c`: command 0x56: two-state handler: state 0 starts the view effect `021ea8e4(view)` and moves to
/// state 1 (0 = not done); state 1 answers `021ea8f4(view) != 0`; any other state is 0.
pub fn view_effect_step<E: FlowEnv>(env: &mut E, c: &[u8], state: &mut [u8]) -> u32 {
    let view = rd32(c, ofs::VIEW);
    match rd32(state, 0) {
        0 => {
            env.callr(0x021E_A8E4, &mut [Arg::V(view)], c);
            wr32(state, 0, 1);
            0
        }
        1 => (env.callr(0x021E_A8F4, &mut [Arg::V(view)], c) != 0) as u32,
        _ => 0,
    }
}

/// EV kinds of command 0x3E (ROM table 0x021EFF88, u16; entry i is used while entry i + 1 is nonzero).
const EV_IDS: [u16; 7] = [0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x011E];

/// C: `FUN_overlay_d_93__021d29cc`: command 0x3E (effort values): pp = `BattleMon_GetPokemon(021b9934(pokecon, u8 a0))`; for i <
/// 6: v = `02017e1c(pp, EV_IDS[i], 0) + args[1 + i]`, at most 0xFF, `02017e40(pp, EV_IDS[i], v)`. Returns 1.
pub fn cmd_add_evs<E: FlowEnv>(env: &mut E, c: &[u8], args: &[u8]) -> u32 {
    let mon = mon_of(env, c, arg(args, 0) & 0xFF);
    let pp = env.callr(0x021D_5844, &mut [Arg::V(mon)], c);
    for i in 0..6 {
        if EV_IDS[i + 1] == 0 {
            continue;
        }
        let id = EV_IDS[i] as u32;
        let cur = env.callr(0x0201_7E1C, &mut [Arg::V(pp), Arg::V(id), Arg::V(0)], c);
        let v = arg(args, 1 + i).wrapping_add(cur).min(0xFF);
        env.callr(0x0201_7E40, &mut [Arg::V(pp), Arg::V(id), Arg::V(v)], c);
    }
    1
}

/// Overlay globals of the level-up sequence (`g` = the 0x20 bytes at 0x021F3B64): u16 move to learn at +0, the
/// [`learn_move_step`] state at +8 (0x021F3B6C), the frame counter at +8 while 021d2a20 waits for a key, the EXP still to add at
/// +0xC, a word at +0x10 (0x021F3B74, passed to `02018328`), the level-up info block at +0x14 (0x021F3B78, level byte first).
mod g {
    pub const MOVE: usize = 0x00;
    pub const COUNTER: usize = 0x08;
    pub const EXP: usize = 0x0C;
    pub const LEARN_CURSOR: usize = 0x10;
    pub const INFO: usize = 0x14;
    pub const ADDR_BASE: u32 = 0x021F_3B64;
}

/// `020362dc() & 3` (A / B pressed) or `02035c60()` (touch).
fn key_pressed<E: FlowEnv>(env: &mut E) -> bool {
    env.call(0x0203_62DC, &mut []) & 3 != 0 || env.call(0x0203_5C60, &mut []) != 0
}

/// The level-up fanfare check of 021d2a20 / 021d2ca8: `021ea720(view)` -> `02005de4(1)`, `02005e64()`, `02005d90(0x515, 0xFFFF)`.
fn fanfare_on_cue<E: FlowEnv>(env: &mut E, c: &[u8]) {
    if view_call(env, c, 0x021E_A720, &[]) != 0 {
        env.callr(0x0200_5DE4, &mut [Arg::V(1)], c);
        env.callr(0x0200_5E64, &mut [], c);
        env.callr(0x0200_5D90, &mut [Arg::V(0x515), Arg::V(0xFFFF)], c);
    }
}

/// C: `sub_021d2a20`: command 0x45 (EXP gain with level ups and new moves; 14-state jump table; `g` = the globals at
/// 0x021F3B64, see [`g`]). k = u8 a0, m = `021b9934(pokecon, k)`, s = `021b8dc0(server, pokecon, k)`, f = s != 0xFF and
/// `021f8204(s, s)` (the gauge is shown). 0: `g.exp = a1`, `g.counter = 0`, state 1, then as 1. 1: no EXP left -> state 0xD;
/// `BattleMon::gain_exp(m, &exp, g.info)` (`021d69b8`) levels up -> state 3 (exp kept); else f ? (`021f8134(s, g.exp)`, state 2) :
/// state 0xD, exp 0. 2: `!021f8180()` -> state 0xD. 3: `BattleMon::recalc_party_stats(m)`, `BattleMon::refresh_from_party(m)`;
/// f -> `021f8150(s, m)`, state 4; else state 6. 4: `!021f8180()` and `!02006238()` -> `021ea3b8(view, s, 0x25D)`, state 5. 5:
/// `021ea3d4(view)` -> state 6. 6: message 0x3C (k, level `g.info[0]`), state 7. 7: fanfare on cue; message done and
/// `!02005f38()` -> `02005e9c()`, `02005de4(0)`, counter 0, state 8. 8: counter + 1; a key or counter > 0x50 -> counter 0,
/// `021ba238(server, m)`, `021eada4(view, m, &g.info)`, state 9. 9..0xB: the stat windows (`021eadb4` + key -> `021eadc4`;
/// `021eadd4` + key -> `021eade4`; `021eadf4` -> counter 0), states 0xA / 0xB / 0xC. 0xC: [`learn_move_step`]`(c, &g.counter,
/// m)` -> state 1. 0xD: `021b9358(server, k)`, 1. Other states: 0.
pub fn cmd_level_up<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    args: &[u8],
    gl: &mut [u8],
) -> u32 {
    let k = arg(args, 0) & 0xFF;
    let m = mon_of(env, c, k);
    let s = field_slot(env, c, k);
    let f = s != 0xFF && env.callr(0x021F_8204, &mut [Arg::V(s), Arg::V(s)], c) != 0;
    let mut st = rd32(state, 0);
    if st == 0 {
        wr32(gl, g::EXP, arg(args, 1));
        wr32(gl, g::COUNTER, 0);
        set_state(state, 1);
        st = 1;
    }
    let next = |state: &mut [u8], v: u32| {
        set_state(state, v);
        0
    };
    match st {
        1 => {
            let exp = rd32(gl, g::EXP);
            if exp == 0 {
                return next(state, 0xD);
            }
            // UNVERIFIED (spec models the out pointers as `p`): the original passes &exp and the info block; gain_exp writes both
            let up = env.callr(0x021D_69B8, &mut [Arg::V(m), Arg::P, Arg::P], c);
            if up != 0 {
                set_state(state, 3);
                wr32(gl, g::EXP, exp);
                return 0;
            }
            if f {
                env.callr(0x021F_8134, &mut [Arg::V(s), Arg::V(rd32(gl, g::EXP))], c);
                set_state(state, 2);
            } else {
                set_state(state, 0xD);
            }
            wr32(gl, g::EXP, 0);
            0
        }
        2 => {
            if env.callr(0x021F_8180, &mut [], c) != 0 {
                return 0;
            }
            next(state, 0xD)
        }
        3 => {
            env.callr(0x021D_6EA0, &mut [Arg::V(m)], c);
            env.callr(0x021D_6BFC, &mut [Arg::V(m)], c);
            if f {
                env.callr(0x021F_8150, &mut [Arg::V(s), Arg::V(m)], c);
                return next(state, 4);
            }
            next(state, 6)
        }
        4 => {
            if env.callr(0x021F_8180, &mut [], c) != 0 || env.callr(0x0200_6238, &mut [], c) != 0 {
                return 0;
            }
            view_call(env, c, 0x021E_A3B8, &[s, 0x25D]);
            next(state, 5)
        }
        5 => {
            if view_call(env, c, 0x021E_A3D4, &[]) != 0 {
                return next(state, 6);
            }
            0
        }
        6 => {
            msg_set(&mut Lend::r(env, c), 1, 0x3C);
            msg_arg(&mut Lend::r(env, c), k);
            msg_arg(&mut Lend::r(env, c), gl[g::INFO] as u32);
            msg_show(env, c);
            next(state, 7)
        }
        7 => {
            fanfare_on_cue(env, c);
            if !msg_done(env, c) || env.callr(0x0200_5F38, &mut [], c) != 0 {
                return 0;
            }
            env.callr(0x0200_5E9C, &mut [], c);
            env.callr(0x0200_5DE4, &mut [Arg::V(0)], c);
            wr32(gl, g::COUNTER, 0);
            next(state, 8)
        }
        8 => {
            let n = rd32(gl, g::COUNTER).wrapping_add(1);
            wr32(gl, g::COUNTER, n);
            if !key_pressed(&mut Lend::r(env, c)) && (rd32(gl, g::COUNTER) as i32) <= 0x50 {
                return 0;
            }
            wr32(gl, g::COUNTER, 0);
            server_call(env, c, 0x021B_A238, &[m]);
            view_call(env, c, 0x021E_ADA4, &[m, g::ADDR_BASE + g::INFO as u32]);
            next(state, 9)
        }
        9 => {
            if view_call(env, c, 0x021E_ADB4, &[]) == 0 || !key_pressed(&mut Lend::r(env, c)) {
                return 0;
            }
            view_call(env, c, 0x021E_ADC4, &[]);
            next(state, 0xA)
        }
        0xA => {
            if view_call(env, c, 0x021E_ADD4, &[]) == 0 || !key_pressed(&mut Lend::r(env, c)) {
                return 0;
            }
            view_call(env, c, 0x021E_ADE4, &[]);
            next(state, 0xB)
        }
        0xB => {
            if view_call(env, c, 0x021E_ADF4, &[]) == 0 {
                return 0;
            }
            wr32(gl, g::COUNTER, 0);
            next(state, 0xC)
        }
        0xC => {
            // the learn step's state word is g.counter (0x021F3B6C); it touches no other byte of g there
            let mut w = [0u8; 4];
            w.copy_from_slice(&gl[g::COUNTER..g::COUNTER + 4]);
            let done = learn_move_step(env, c, &mut w, m, gl);
            gl[g::COUNTER..g::COUNTER + 4].copy_from_slice(&w);
            if done != 0 {
                return next(state, 1);
            }
            0
        }
        0xD => {
            server_call(env, c, 0x021B_9358, &[k]);
            1
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021d2c7c`: message-window callback (`021ea638` in 021d2ca8): event 3 plays the window sound
/// (`020061e4(0x56B)`, answer 0), event 5 answers whether it has finished (`02006254(0x56B) != 0`), any other event 0.
pub fn window_sound_event<E: FlowEnv>(env: &mut E, event: u32) -> u32 {
    match event {
        3 => {
            env.call(0x0200_61E4, &mut [Arg::V(SE_WINDOW)]);
            0
        }
        5 => (env.call(0x0200_6254, &mut [Arg::V(SE_WINDOW)]) != 0) as u32,
        _ => 0,
    }
}

/// The sound id the yes/no message window polls (`021d2c7c`).
const SE_WINDOW: u32 = 0x56B;

/// Message kind 5 (level-up / move messages) with the mon id and a value: `021eaf18(c+0x58, 5, id)`, args n and v.
fn move_msg<E: FlowEnv>(env: &mut E, id: u32, n: u32, v: u32) {
    msg_set(env, 5, id);
    msg_arg(env, n);
    msg_arg(env, v);
}

/// C: `FUN_021d2ca8`: learn the moves of a level up (14-state jump table; `state` is the caller's state word, `g` the globals at
/// 0x021F3B64, see [`g`]; `mon` the BattleMon). n = `021d5440(mon)` (its id). 0: `g.cursor = 0`, `g.move = 0`, state 1, then as
/// 1\. 1: `g.move = 02018328(pp, &g.cursor, c.u16[0x19A])` (next learnable move of `BattleMon_GetPokemon(mon)`): 0 -> 1 (done);
/// 0xFFFE -> 0; bit 15 (no free slot) -> `g.move &= 0x7FFF`, state 5; else state 2. 2: `BattleMon::refresh_from_party(021b9934(
/// pokecon, n))`, message 5/3 (n, move), state 3, then as 3. 3: fanfare on cue; message done -> state 4. 4: `!02005f38()` ->
/// `02005e9c()`, `02005de4(0)`, state 1. 5: message 5/4 (n, move) with `021eaf78(c+0x58, 0xFF)`, state 6, then as 6. 6:
/// `021ea720(view)` -> yes / no window (6/2, 6/3); message done -> state 7. 7: `021eac54(view, &ans)`: yes -> state 0xA; no ->
/// `021eac64(view, (u8) 021b9cbc(021b9af8(pokecon, c[0x1A6]), n), move)` (forget-a-move list), state 8. 8: `021eac80(view, &slot)`:
/// slot 4 -> state 5; else old = (u16) `02017e1c(pp, slot + 0x36, 0)`, `020181f4(pp, move, slot)`, message 5/5 (n, old) through
/// `021ea638(view, c+0x58, 021d2c7d)`, state 9. 9: message done -> state 2. 0xA: message 5/7 (n, move) + 0xFF, state 0xB. 0xB:
/// `021ea720` -> yes / no window (6/4, 6/5 with the move); message done -> state 0xC. 0xC: `021eac54`: yes -> state 5; no ->
/// message 5/8 (n, move), state 0xD. 0xD: message done -> state 1. Other states: 0.
pub fn learn_move_step<E: FlowEnv>(
    env: &mut E,
    c: &[u8],
    state: &mut [u8],
    mon: u32,
    gl: &mut [u8],
) -> u32 {
    let n = env.callr(0x021D_5440, &mut [Arg::V(mon)], c);
    let mv = |gl: &[u8]| rd16(gl, g::MOVE) as u32;
    let set_move =
        |gl: &mut [u8], v: u32| gl[g::MOVE..g::MOVE + 2].copy_from_slice(&(v as u16).to_le_bytes());
    let view = rd32(c, ofs::VIEW);
    let mut st = rd32(state, 0);
    if st == 0 {
        wr32(gl, g::LEARN_CURSOR, 0);
        set_move(gl, 0);
        set_state(state, 1);
        st = 1;
    }
    match st {
        1 => {
            let pp = env.callr(0x021D_5844, &mut [Arg::V(mon)], c);
            let r = env.callr(
                0x0201_8328,
                &mut [
                    Arg::V(pp),
                    // B18b: the learn cursor word of the global block (`region cursor` of 02018328)
                    Arg::PAt(g::ADDR_BASE + g::LEARN_CURSOR as u32),
                    Arg::V(rd16(c, ofs::HP_REF_OWN) as u32),
                ],
                c,
            );
            set_move(gl, r);
            let v = mv(gl);
            if v == 0 {
                return 1;
            }
            if v == 0xFFFE {
                return 0;
            }
            if v & 0x8000 != 0 {
                set_move(gl, v & 0x7FFF);
                set_state(state, 5);
            } else {
                set_state(state, 2);
            }
            0
        }
        2 | 3 => {
            if st == 2 {
                let m2 = mon_of(env, c, n);
                env.callr(0x021D_5844, &mut [Arg::V(m2)], c);
                env.callr(0x021D_6BFC, &mut [Arg::V(m2)], c);
                move_msg(&mut Lend::r(env, c), 3, n, mv(gl));
                msg_show(env, c);
                set_state(state, 3);
            }
            fanfare_on_cue(env, c);
            if msg_done(env, c) {
                set_state(state, 4);
            }
            0
        }
        4 => {
            if env.callr(0x0200_5F38, &mut [], c) != 0 {
                return 0;
            }
            env.callr(0x0200_5E9C, &mut [], c);
            env.callr(0x0200_5DE4, &mut [Arg::V(0)], c);
            set_state(state, 1);
            0
        }
        5 | 6 => {
            if st == 5 {
                move_msg(&mut Lend::r(env, c), 4, n, mv(gl));
                env.callr(0x021E_AF78, &mut [Arg::P, Arg::V(0xFF)], c);
                msg_show(env, c);
                set_state(state, 6);
            }
            if view_call(env, c, 0x021E_A720, &[]) != 0 {
                env.callr(0x021E_AF18, &mut [Arg::P, Arg::V(6), Arg::V(2)], c);
                env.callr(0x021E_AF18, &mut [Arg::P, Arg::V(6), Arg::V(3)], c);
                env.callr(
                    0x021E_ABC4,
                    &mut [Arg::V(view), Arg::P, Arg::P, Arg::V(1)],
                    c,
                );
            }
            if msg_done(env, c) {
                set_state(state, 7);
            }
            0
        }
        7 => {
            let mut ans = [0u8; 4];
            if env.callr(0x021E_AC54, &mut [Arg::V(view), Arg::Out(&mut ans)], c) == 0 {
                return 0;
            }
            if u32::from_le_bytes(ans) != 0 {
                set_state(state, 0xA);
                return 0;
            }
            let party = env.callr(
                0x021B_9AF8,
                &mut [
                    Arg::V(rd32(c, ofs::POKECON)),
                    Arg::V(c[ofs::CLIENT_ID] as u32),
                ],
                c,
            );
            let x = env.callr(0x021B_9CBC, &mut [Arg::V(party), Arg::V(n)], c);
            view_call(env, c, 0x021E_AC64, &[x & 0xFF, mv(gl)]);
            set_state(state, 8);
            0
        }
        8 => {
            let mut slot = [0u8; 1];
            if env.callr(0x021E_AC80, &mut [Arg::V(view), Arg::Out(&mut slot)], c) == 0 {
                return 0;
            }
            let slot = slot[0] as u32;
            if slot == 4 {
                set_state(state, 5);
                return 0;
            }
            let m2 = mon_of(env, c, n);
            let pp = env.callr(0x021D_5844, &mut [Arg::V(m2)], c);
            let old = env.callr(
                0x0201_7E1C,
                &mut [Arg::V(pp), Arg::V(slot + 0x36), Arg::V(0)],
                c,
            ) & 0xFFFF;
            env.callr(
                0x0201_81F4,
                &mut [Arg::V(pp), Arg::V(mv(gl)), Arg::V(slot)],
                c,
            );
            move_msg(&mut Lend::r(env, c), 5, n, old);
            env.callr(
                0x021E_A638,
                &mut [Arg::V(view), Arg::P, Arg::V(0x021D_2C7D)],
                c,
            );
            set_state(state, 9);
            0
        }
        9 => {
            if msg_done(env, c) {
                set_state(state, 2);
            }
            0
        }
        0xA | 0xB => {
            if st == 0xA {
                move_msg(&mut Lend::r(env, c), 7, n, mv(gl));
                env.callr(0x021E_AF78, &mut [Arg::P, Arg::V(0xFF)], c);
                msg_show(env, c);
                set_state(state, 0xB);
                return 0;
            }
            if view_call(env, c, 0x021E_A720, &[]) != 0 {
                env.callr(0x021E_AF18, &mut [Arg::P, Arg::V(6), Arg::V(4)], c);
                env.callr(0x021E_AF50, &mut [Arg::P, Arg::V(mv(gl))], c);
                env.callr(0x021E_AF18, &mut [Arg::P, Arg::V(6), Arg::V(5)], c);
                env.callr(0x021E_AF50, &mut [Arg::P, Arg::V(mv(gl))], c);
                env.callr(
                    0x021E_ABC4,
                    &mut [Arg::V(view), Arg::P, Arg::P, Arg::V(1)],
                    c,
                );
            }
            if msg_done(env, c) {
                set_state(state, 0xC);
            }
            0
        }
        0xC => {
            let mut ans = [0u8; 4];
            if env.callr(0x021E_AC54, &mut [Arg::V(view), Arg::Out(&mut ans)], c) == 0 {
                return 0;
            }
            if u32::from_le_bytes(ans) != 0 {
                set_state(state, 5);
                return 0;
            }
            move_msg(&mut Lend::r(env, c), 8, n, mv(gl));
            msg_show(env, c);
            set_state(state, 0xD);
            0
        }
        0xD => {
            if msg_done(env, c) {
                set_state(state, 1);
            }
            0
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests;
