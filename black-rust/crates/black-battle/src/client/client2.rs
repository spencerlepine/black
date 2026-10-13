//! `btl_client.c` part 2 (0x021CDDDC-0x021CF5B0): the first request steps, the action-selection callbacks and their helpers
//! (B13-spec1 / B13-spec2; ported B13a).
//!
//! Specs: `tools/rom-diff/svflow_specs/client2.spec` (`nowork`, the client object is `arg buf 0x1C0`, a step's or callback's
//! state word `arg buf 0x10`). Shapes: a request step `u32 step(c, u32 *state)` returns 1 once the request is answered; a
//! pending callback (installed at `c+0xD0` by [`set_callback`], run by [`run_callback`] with `&c+0xD4`) `u32 cb(c, u32 *state)`
//! returns 0 and chains to the next callback. The ports take the object as `c` (with its address `c_addr` where the original
//! stores or passes an address into it) and the state word as `state` (4 bytes).
//!
//! Callees: battle-view and sound calls through [`ClientUi`] (D3 / D7); BattleMon accessors, the battle server's getters,
//! the action-record helpers (`021d8xxx`), the Thumb-to-ARM veneers and the callees in other client areas through
//! [`FlowEnv::call`]; a callback held in `c+0xD0` is dispatched through `env.call(ptr & !1, ..)`.
//!
//! Message buffers (Z20): the message calls `021eaf18` / `021eaf50` / `021eaf78` and `021cf5b0`'s last argument take the
//! buffer as its address (`v`: `c+0x58`, `c+0x7C` for the second yes / no line, or 0 = no message), so the ports pass
//! `c_addr + 0x58` where the original does.

use super::{rd16, rd32, wr16, wr32, ClientUi};
use crate::calc::u32_div;
use crate::server_flow::env::{Arg, FlowEnv, Lend};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021C_DDDC,
    0x021C_DE8C,
    0x021C_DECC,
    0x021C_DEE8,
    0x021C_DF1C,
    0x021C_DF58,
    0x021C_DFE4,
    0x021C_E040,
    0x021C_E044,
    0x021C_E068,
    0x021C_E08C,
    0x021C_E0BC,
    0x021C_E0E0,
    0x021C_E0F8,
    0x021C_E150,
    0x021C_E160,
    0x021C_E170,
    0x021C_E1B0,
    0x021C_E2E8,
    0x021C_E308,
    0x021C_E3B4,
    0x021C_E3C8,
    0x021C_E4D0,
    0x021C_E7D4,
    0x021C_E8F0,
    0x021C_EA94,
    0x021C_ECB4,
    0x021C_ED4C,
    0x021C_EDC0,
    0x021C_EEB8,
    0x021C_EFFC,
    0x021C_F078,
    0x021C_F220,
    0x021C_F2F0,
    0x021C_F360,
    0x021C_F37C,
    0x021C_F3B8,
    0x021C_F3F8,
    0x021C_F41C,
    0x021C_F430,
    0x021C_F43C,
    0x021C_F448,
    0x021C_F470,
    0x021C_F524,
    0x021C_F58C,
];

const OFS_SERVER: usize = 0x00;
const OFS_POKECON: usize = 0x04;
/// The mon being asked for (`021b990c` result).
const OFS_MON: usize = 0x08;
/// Pointer to the current answer record (`c+0x124 + 4 * index`).
const OFS_REC: usize = 0x0C;
const OFS_READER: usize = 0x14;
const OFS_VIEW: usize = 0x54;
/// The message buffer (`021eaf18` ..) and the second one of the yes / no window.
const OFS_MSG: usize = 0x58;
const OFS_MSG2: usize = 0x7C;
/// The fight-menu table of a double battle (3 x 8 bytes, [`fight_table`]).
const OFS_FIGHT: usize = 0xA4;
const FIGHT_TABLE_SIZE: usize = 0x18;
const OFS_BC: usize = 0xBC;
const OFS_C4: usize = 0xC4;
const OFS_CB: usize = 0xD0;
const OFS_CB_STATE: usize = 0xD4;
const OFS_RESULT_PTR: usize = 0xD8;
const OFS_RESULT_SIZE: usize = 0xDC;
const OFS_BGM: usize = 0xE4;
const OFS_SE: usize = 0xE6;
const OFS_ITEM_FLAGS: usize = 0x112;
const OFS_PARTY: usize = 0x118;
const OFS_COUNT: usize = 0x11C;
const OFS_CUR: usize = 0x11D;
const OFS_SHOWN: usize = 0x11E;
const OFS_FIRST: usize = 0x11F;
const OFS_120: usize = 0x120;
const OFS_ANSWERS: usize = 0x124;
const OFS_130: usize = 0x130;
const OFS_HEAP_ID: usize = 0x19A;
const OFS_HP_REF: usize = 0x19C;
const OFS_ITEM: usize = 0x19E;
const OFS_1A0: usize = 0x1A0;
const OFS_ID: usize = 0x1A6;
const OFS_1A9: usize = 0x1A9;
const OFS_1AA: usize = 0x1AA;
const OFS_COUNTER: usize = 0x1AB;
const OFS_FADE_DONE: usize = 0x1AD;
const OFS_1AF: usize = 0x1AF;
const OFS_1B0: usize = 0x1B0;
const OFS_1B1: usize = 0x1B1;
const OFS_FLAGS: usize = 0x1B2;

/// Callback addresses (Thumb pointers) of this area.
pub const CB_SELECT_START: u32 = 0x021C_E309;
pub const CB_FINISH: u32 = 0x021C_E3C9;
pub const CB_SELECT: u32 = 0x021C_E4D1;
pub const CB_ITEM_ANIM: u32 = 0x021C_E7D5;
pub const CB_FIGHT: u32 = 0x021C_EA95;
pub const CB_TARGET: u32 = 0x021C_ED4D;
pub const CB_PARTY: u32 = 0x021C_EDC1;
pub const CB_BAG: u32 = 0x021C_EEB9;
pub const CB_RUN: u32 = 0x021C_F079;
pub const CB_NEXT: u32 = 0x021C_F221;
pub const CB_END: u32 = 0x021C_F2F1;

/// The four battle items of the shortcut check (ROM table `0x021EFF78`, u16 each).
const SHORTCUT_ITEMS: [u16; 4] = [0x12, 0x11, 0x13, 0x14];

/// The struggle move id.
const MOVE_STRUGGLE: u32 = 0xA5;

fn view(c: &[u8]) -> u32 {
    rd32(c, OFS_VIEW)
}

fn st(s: &[u8]) -> u32 {
    rd32(s, 0)
}

fn set_st(s: &mut [u8], v: u32) {
    wr32(s, 0, v);
}

/// Address of the answer record of mon `i`.
fn answer_addr(c_addr: u32, i: u8) -> u32 {
    c_addr.wrapping_add(OFS_ANSWERS as u32 + 4 * i as u32)
}

/// `021b990c(pokecon, client id, i)`: the `i`-th mon of this client.
fn client_mon<E: FlowEnv>(env: &mut E, c: &[u8], i: u32) -> u32 {
    env.callr(
        0x021B_990C,
        &mut [
            Arg::V(rd32(c, OFS_POKECON)),
            Arg::V(c[OFS_ID] as u32),
            Arg::V(i),
        ],
        c,
    )
}

/// `021d8434(rec)`: marks the answer record at `c + ofs` "skip".
fn rec_skip<E: FlowEnv>(env: &mut E, c: &mut [u8], ofs: usize) {
    let rec = c[ofs..].as_ptr() as usize;
    env.callh(0x021D_8434, &mut [Arg::PHost(rec)], c);
}

/// `021eaf18(buf, kind, id)`: starts a message in the message buffer at address `buf`.
fn msg_start<E: ClientUi>(env: &mut E, buf: u32, kind: u32, id: u32) {
    env.view(0x021E_AF18, &mut [Arg::V(buf), Arg::V(kind), Arg::V(id)]);
}

/// `021eaf50(buf, value)`: adds an argument to the message at address `buf`.
fn msg_arg<E: ClientUi>(env: &mut E, buf: u32, v: u32) {
    env.view(0x021E_AF50, &mut [Arg::V(buf), Arg::V(v)]);
}

/// `021eaf78(buf, v)`: the message at address `buf` (`v` = 0 or 0xFF).
fn msg_end<E: ClientUi>(env: &mut E, buf: u32, v: u32) {
    env.view(0x021E_AF78, &mut [Arg::V(buf), Arg::V(v)]);
}

/// Address of the client's message buffer (`c+0x58`).
fn msg_buf(c_addr: u32) -> u32 {
    c_addr.wrapping_add(OFS_MSG as u32)
}

/// `021ea710(view)`: has the message window finished?
fn msg_done<E: ClientUi>(env: &mut E, c: &[u8]) -> bool {
    env.view(0x021E_A710, &mut [Arg::V(view(c))]) != 0
}

/// C: `sub_021cdddc`: request 1, mode 0. State 0: `021e90a0(view, battle mode 021b86b8(server) == 4)`, state 1. State 1, once
/// `021e90d0(view)`: BGM / SE ids `+0xE4` / `+0xE6` from `021ba5c4` / `021ba5d0(server)`; with either, `021f84e8(SE)` and, with
/// an SE, `021f8504(0, 1, 1)`; [`store_active_hp`]; with the idle step (`021cd9e0`), `021eaea8(view, (u16) 021d3fa8(cursor))`;
/// returns 1. Other states 0.
pub fn req1_step<E: ClientUi>(env: &mut E, c: &mut [u8], state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            let four = env.callh(0x021B_86B8, &mut [Arg::V(rd32(c, OFS_SERVER))], c) == 4;
            env.view(0x021E_90A0, &mut [Arg::V(view(c)), Arg::V(four as u32)]);
            set_st(state, st(state).wrapping_add(1));
            0
        }
        1 => {
            if env.view(0x021E_90D0, &mut [Arg::V(view(c))]) == 0 {
                return 0;
            }
            let server = rd32(c, OFS_SERVER);
            let bgm = env.callh(0x021B_A5C4, &mut [Arg::V(server)], c);
            wr16(c, OFS_BGM, bgm as u16);
            let se = env.callh(0x021B_A5D0, &mut [Arg::V(rd32(c, OFS_SERVER))], c);
            wr16(c, OFS_SE, se as u16);
            if rd16(c, OFS_BGM) != 0 || rd16(c, OFS_SE) != 0 {
                env.sound(0x021F_84E8, &mut [Arg::V(rd16(c, OFS_SE) as u32)]);
                if rd16(c, OFS_SE) != 0 {
                    env.sound(0x021F_8504, &mut [Arg::V(0), Arg::V(1), Arg::V(1)]);
                }
            }
            store_active_hp(env, c);
            if env.call_held(c, 0x021C_D9E0, &mut [Arg::P]) != 0 {
                let n = env.call(0x021D_3FA8, &mut [Arg::PBuf(&mut c[0x18..0x28])]) & 0xFFFF;
                env.view(0x021E_AEA8, &mut [Arg::V(view(c)), Arg::V(n)]);
            }
            1
        }
        _ => 0,
    }
}

/// C: `sub_021cde8c`: request 1, mode 2 (replay). With a view: `r = `[`req1_step`]; when `r` and `021ee688(reader, client
/// id)`, `021d3f54(cursor)`; returns `r`. Without a view: `021ee688(reader, client id)`, returns 1.
pub fn req1_replay<E: ClientUi>(env: &mut E, c: &mut [u8], state: &mut [u8]) -> u32 {
    if view(c) == 0 {
        env.callh(
            0x021E_E688,
            &mut [Arg::V(rd32(c, OFS_READER)), Arg::V(c[OFS_ID] as u32)],
            c,
        );
        return 1;
    }
    let r = req1_step(env, c, state);
    if r != 0
        && env.callh(
            0x021E_E688,
            &mut [Arg::V(rd32(c, OFS_READER)), Arg::V(c[OFS_ID] as u32)],
            c,
        ) != 0
    {
        env.call(0x021D_3F54, &mut [Arg::PBuf(&mut c[0x18..0x28])]);
    }
    r
}

/// C: `FUN_overlay_d_93__021cdecc`: with an active mon ([`active_mon`]), `+0x19C` = its HP (`BattleMon_Get(mon, 0xD)`).
pub fn store_active_hp<E: FlowEnv>(env: &mut E, c: &mut [u8]) {
    let mon = active_mon(env, c);
    if mon != 0 {
        let hp = env.callh(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xD)], c);
        wr16(c, OFS_HP_REF, hp as u16);
    }
}

/// C: `FUN_overlay_d_93__021cdee8`: how far the active mon's HP is below the reference HP (`c.u16[0x19C]`), in percent of the
/// current HP: 0 without an active mon ([`active_mon`]) or when HP >= reference, else `100 * (ref - HP) / HP`
/// (`BattleMon_Get(mon, 0xD)`; division by zero answers the dividend like the original runtime routine).
pub fn hp_drop_percent<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    let mon = active_mon(env, c);
    if mon == 0 {
        return 0;
    }
    let hp = env.callr(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xD)], c);
    let reference = rd16(c, OFS_HP_REF) as u32;
    if hp >= reference {
        return 0;
    }
    u32_div(100u32.wrapping_mul(reference - hp), hp)
}

/// C: `FUN_overlay_d_93__021cdf1c`: the active mon of this client: 0 when `021b8588(server)` (battle type) or `021b86c0(server)`
/// is nonzero, else `021b9920(pokecon, 021b9064(server, client id, 0) & 0xFF, 0)`.
pub fn active_mon<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    let server = rd32(c, OFS_SERVER);
    if env.callr(0x021B_8588, &mut [Arg::V(server)], c) != 0 {
        return 0;
    }
    if env.callr(0x021B_86C0, &mut [Arg::V(server)], c) != 0 {
        return 0;
    }
    let pos = env.callr(
        0x021B_9064,
        &mut [Arg::V(server), Arg::V(c[OFS_ID] as u32), Arg::V(0)],
        c,
    );
    env.callr(
        0x021B_9920,
        &mut [Arg::V(rd32(c, OFS_POKECON)), Arg::V(pos & 0xFF), Arg::V(0)],
        c,
    )
}

/// C: `sub_021cdf58`: request 2, mode 0 (a 5-way jump table on the state). 0: `021eaa00(view, c[0xC4])`, [`fade_start`],
/// state 1. 1: [`fade_poll`] set -> `021eaa5c(view)`, state 2; else when `021eaa34(view, &b)`: `c[0xC4] = b`, state 3.
/// 2: `c[0xC4] = `[`random_choice`]`(c[0xC4])`, state 3. 3: [`fade_end`], state 4. 4: result = `&c[0xC4]`, size 1, returns 1.
pub fn req2_step<E: ClientUi>(env: &mut E, c: &mut [u8], c_addr: u32, state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            env.view(
                0x021E_AA00,
                &mut [Arg::V(view(c)), Arg::V(c[OFS_C4] as u32)],
            );
            fade_start(env, c);
            set_st(state, st(state).wrapping_add(1));
        }
        1 => {
            if fade_poll(env, c) != 0 {
                env.view(0x021E_AA5C, &mut [Arg::V(view(c))]);
                set_st(state, 2);
            } else {
                let mut b = [0u8; 4];
                if env.view(0x021E_AA34, &mut [Arg::V(view(c)), Arg::Out(&mut b)]) != 0 {
                    c[OFS_C4] = b[0];
                    set_st(state, 3);
                }
            }
        }
        2 => {
            c[OFS_C4] = random_choice(env, c[OFS_C4] as u32);
            set_st(state, st(state).wrapping_add(1));
        }
        3 => {
            fade_end(env, c);
            set_st(state, st(state).wrapping_add(1));
        }
        4 => {
            wr32(c, OFS_RESULT_PTR, c_addr.wrapping_add(OFS_C4 as u32));
            wr32(c, OFS_RESULT_SIZE, 1);
            return 1;
        }
        _ => {}
    }
    0
}

/// C: `FUN_overlay_d_93__021cdfe4`: a random choice 1..3 weighted by `mode` (`r = rand() * 100 >> 32`, `0203f0a8`): mode 0:
/// `r < 30` -> 3, `r < 60` -> 2, else 1; mode 2: `r >= 40` -> 1, else 3; mode 3: `r >= 40` -> 1, else 2; any other mode:
/// `r >= 50` -> 3, else 2.
pub fn random_choice<E: FlowEnv>(env: &mut E, mode: u32) -> u8 {
    let rnd = env.call(0x0203_F0A8, &mut []);
    let r = ((rnd as u64 * 100) >> 32) as u32;
    match mode {
        0 => {
            if r < 30 {
                3
            } else if r < 60 {
                2
            } else {
                1
            }
        }
        2 => {
            if r < 40 {
                3
            } else {
                1
            }
        }
        3 => {
            if r < 40 {
                2
            } else {
                1
            }
        }
        _ => {
            if r < 50 {
                2
            } else {
                3
            }
        }
    }
}

/// C: `sub_021ce040`: request 2, mode 2: answered at once (1).
pub fn req2_replay() -> u32 {
    1
}

/// C: `sub_021ce044`: request 2, mode 1: `c[0xC4] = `[`random_choice`]`(c[0xC4])`, result = `&c[0xC4]`, size 1; returns 1.
pub fn req2_auto<E: FlowEnv>(env: &mut E, c: &mut [u8], c_addr: u32) -> u32 {
    c[OFS_C4] = random_choice(env, c[OFS_C4] as u32);
    wr32(c, OFS_RESULT_PTR, c_addr.wrapping_add(OFS_C4 as u32));
    wr32(c, OFS_RESULT_SIZE, 1);
    1
}

/// C: `FUN_overlay_d_93__021ce068`: with the BGM flag `+0xE4`: `021f8504(1, 1, 1)`, `c[0x1AD] = 0`.
pub fn fade_start<E: ClientUi>(env: &mut E, c: &mut [u8]) {
    if rd16(c, OFS_BGM) != 0 {
        env.sound(0x021F_8504, &mut [Arg::V(1), Arg::V(1), Arg::V(1)]);
        c[OFS_FADE_DONE] = 0;
    }
}

/// C: `FUN_overlay_d_93__021ce08c`: with the BGM flag: `c[0x1AD]` becomes 1 once `021f8524(1)` answers nonzero (only polled
/// while it is 0); returns `c[0x1AD]`. Without the flag 0.
pub fn fade_poll<E: ClientUi>(env: &mut E, c: &mut [u8]) -> u32 {
    if rd16(c, OFS_BGM) == 0 {
        return 0;
    }
    if c[OFS_FADE_DONE] == 0 && env.sound(0x021F_8524, &mut [Arg::V(1)]) != 0 {
        c[OFS_FADE_DONE] = 1;
    }
    c[OFS_FADE_DONE] as u32
}

/// C: `FUN_overlay_d_93__021ce0bc`: [`fade_poll`] set: (`next` != 0: [`set_callback`]`(c, next)`), returns 1; else 0.
pub fn fade_poll_then<E: ClientUi>(env: &mut E, c: &mut [u8], next: u32) -> u32 {
    if fade_poll(env, c) == 0 {
        return 0;
    }
    if next != 0 {
        set_callback(c, next);
    }
    1
}

/// C: `FUN_overlay_d_93__021ce0e0`: with the BGM flag: `021f8504(1, 0, 0)`.
pub fn fade_end<E: ClientUi>(env: &mut E, c: &[u8]) {
    if rd16(c, OFS_BGM) != 0 {
        env.sound(0x021F_8504, &mut [Arg::V(1), Arg::V(0), Arg::V(0)]);
    }
}

/// C: `FUN_overlay_d_93__021ce0f8`: the "what will `<mon>` do" message into the buffer `buf`: battle type 3 (`021b8588`) ->
/// message 0x46 with the client id, else message 0x45 with `021d5440(mon)`; then `021eaf78(buf, 0)`.
pub fn sent_out_message<E: ClientUi>(env: &mut E, c: &[u8], mon: u32, buf: u32) {
    if env.callr(0x021B_8588, &mut [Arg::V(rd32(c, OFS_SERVER))], c) == 3 {
        env.view(0x021E_AF18, &mut [Arg::V(buf), Arg::V(1), Arg::V(0x46)]);
        env.view(0x021E_AF50, &mut [Arg::V(buf), Arg::V(c[OFS_ID] as u32)]);
    } else {
        env.view(0x021E_AF18, &mut [Arg::V(buf), Arg::V(1), Arg::V(0x45)]);
        let name = env.callr(0x021D_5440, &mut [Arg::V(mon)], c);
        env.view(0x021E_AF50, &mut [Arg::V(buf), Arg::V(name)]);
    }
    env.view(0x021E_AF78, &mut [Arg::V(buf), Arg::V(0)]);
}

/// C: `FUN_overlay_d_93__021ce150`: installs the pending callback `cb` (Thumb pointer) at `+0xD0`, its state `+0xD4` = 0.
pub fn set_callback(c: &mut [u8], cb: u32) {
    wr32(c, OFS_CB, cb);
    wr32(c, OFS_CB_STATE, 0);
}

/// C: `FUN_overlay_d_93__021ce160`: runs the pending callback `(c->cb)(c, &c->cb_state)`; returns its answer.
pub fn run_callback<E: FlowEnv>(env: &mut E, c: &mut [u8]) -> u32 {
    let cb = rd32(c, OFS_CB);
    let state = c[OFS_CB_STATE..].as_ptr() as usize;
    env.call_held(c, cb & !1, &mut [Arg::P, Arg::PHost(state)])
}

/// C: `sub_021ce170`: request 3, mode 0 (the action selection). State 0: callback [`select_start`], [`fade_start`], state 1.
/// State 1: [`fade_poll`] (answer unused), then [`run_callback`] != 0 -> 1. Other states 0.
pub fn req3_step<E: ClientUi>(env: &mut E, c: &mut [u8], state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            set_callback(c, CB_SELECT_START);
            fade_start(env, c);
            set_st(state, st(state).wrapping_add(1));
            0
        }
        1 => {
            fade_poll(env, c);
            (run_callback(env, c) != 0) as u32
        }
        _ => 0,
    }
}

/// C: `sub_021ce1b0`: request 3, mode 2 (replay). State 0: flag bit 0x20 -> [`answer_skip`], 1. Else the next record
/// `rec = 021ee6b8(reader, client id, &count, &more)`; `more` -> `021d3f54(cursor)`; result = `rec`, size `4 * count`; by the
/// record kind `rec[0] & 0xF`: 8 -> `021d3fd0(cursor)`, with a view message 0xB7 (`021eaf18`, `021ea614(view, buf)`) and state 1,
/// without one [`answer_skip`], flag 0x10, 1; 9 -> flag 0x20, `021d3fd0(cursor)`, with a view message 0xB8 and state 2, without
/// one [`answer_skip`], 1; else 1. State 1: message done -> [`answer_skip`], flag 0x10, 1. State 2: message done ->
/// [`answer_skip`], 1. `recs` is the memory the record pointer points into (at address `recs_addr`).
pub fn req3_replay<E: ClientUi>(
    env: &mut E,
    c: &mut [u8],
    c_addr: u32,
    state: &mut [u8],
    recs: &[u8],
    recs_addr: u32,
) -> u32 {
    match st(state) {
        0 => {
            if c[OFS_FLAGS] & 0x20 != 0 {
                answer_skip(env, c, c_addr);
                return 1;
            }
            let (mut count, mut more) = ([0u8; 1], [0u8; 1]);
            let rec = env.callh(
                0x021E_E6B8,
                &mut [
                    Arg::V(rd32(c, OFS_READER)),
                    Arg::V(c[OFS_ID] as u32),
                    Arg::Out(&mut count),
                    Arg::Out(&mut more),
                ],
                c,
            );
            if more[0] != 0 {
                env.call(0x021D_3F54, &mut [Arg::PBuf(&mut c[0x18..0x28])]);
            }
            wr32(c, OFS_RESULT_PTR, rec);
            wr32(c, OFS_RESULT_SIZE, 4 * count[0] as u32);
            let kind = rd32(recs, rec.wrapping_sub(recs_addr) as usize) & 0xF;
            match kind {
                8 => {
                    env.call(0x021D_3FD0, &mut [Arg::PBuf(&mut c[0x18..0x28])]);
                    if view(c) != 0 {
                        msg_start(&mut Lend::h(env, c), msg_buf(c_addr), 1, 0xB7);
                        env.view(0x021E_A614, &mut [Arg::V(view(c)), Arg::P]);
                        set_st(state, st(state).wrapping_add(1));
                        return 0;
                    }
                    answer_skip(env, c, c_addr);
                    c[OFS_FLAGS] |= 0x10;
                    1
                }
                9 => {
                    c[OFS_FLAGS] |= 0x20;
                    env.call(0x021D_3FD0, &mut [Arg::PBuf(&mut c[0x18..0x28])]);
                    if view(c) != 0 {
                        msg_start(&mut Lend::h(env, c), msg_buf(c_addr), 1, 0xB8);
                        env.view(0x021E_A614, &mut [Arg::V(view(c)), Arg::P]);
                        set_st(state, st(state).wrapping_add(2));
                        return 0;
                    }
                    answer_skip(env, c, c_addr);
                    1
                }
                _ => 1,
            }
        }
        1 => {
            if !msg_done(env, c) {
                return 0;
            }
            answer_skip(env, c, c_addr);
            c[OFS_FLAGS] |= 0x10;
            1
        }
        2 => {
            if !msg_done(env, c) {
                return 0;
            }
            answer_skip(env, c, c_addr);
            1
        }
        _ => 0,
    }
}

/// C: `FUN_overlay_d_93__021ce2e8`: answer = one "skip" record: `+0xC` = `&c[0x124]`, `021d8434(rec)`, result = `+0xC`,
/// size 4.
pub fn answer_skip<E: FlowEnv>(env: &mut E, c: &mut [u8], c_addr: u32) {
    wr32(c, OFS_REC, c_addr.wrapping_add(OFS_ANSWERS as u32));
    rec_skip(env, c, OFS_ANSWERS);
    wr32(c, OFS_RESULT_PTR, rd32(c, OFS_REC));
    wr32(c, OFS_RESULT_SIZE, 4);
}

/// C: `sub_021ce308`: callback 0x021CE309, the start of the action selection: index `+0x11D` = 0, shown index `+0x11E` = -1,
/// first index `+0x11F` = 0, flag bit 4 cleared; party view block `021d0b10(c, 0, count, c+0x184, c+0x190)` (client3);
/// [`clear_130`], [`clear_1a0`], the first three answer records "skip"; the first mon `i < count` that has no answer yet
/// ([`mon_answered`]`(mon, 0) == 0`) becomes the first index; callback [`select_loop`]. Returns 0.
pub fn select_start<E: FlowEnv>(env: &mut E, c: &mut [u8]) -> u32 {
    c[OFS_CUR] = 0;
    c[OFS_SHOWN] = 0xFF;
    c[OFS_FIRST] = 0;
    c[OFS_FLAGS] &= !4;
    // (c, 0, count, &c[0x184] party view block, &c[0x190])
    let (blk, extra) = (c[0x184..].as_ptr() as usize, c[0x190..].as_ptr() as usize);
    env.callh(
        0x021D_0B10,
        &mut [
            Arg::P,
            Arg::V(0),
            Arg::V(c[OFS_COUNT] as u32),
            Arg::PHost(blk),
            Arg::PHost(extra),
        ],
        c,
    );
    clear_130(c);
    clear_1a0(c);
    for i in 0..3 {
        rec_skip(env, c, OFS_ANSWERS + 4 * i);
    }
    let mut i = 0u32;
    while i < c[OFS_COUNT] as u32 {
        let mon = client_mon(env, c, i & 0xFF);
        if mon_answered(&mut Lend::h(env, c), mon, 0) == 0 {
            c[OFS_FIRST] = i as u8;
            break;
        }
        i += 1;
    }
    set_callback(c, CB_SELECT);
    0
}

/// C: `FUN_overlay_d_93__021ce3b4`: `021ea614(view)`, `c[0x120]` = 1.
pub fn print_message<E: ClientUi>(env: &mut E, c: &mut [u8]) {
    env.view(0x021E_A614, &mut [Arg::V(view(c))]);
    c[OFS_120] = 1;
}

/// C: `sub_021ce3c8`: callback 0x021CE3C9, finish the selection. State 0: `021eae44(view)`, state 1. State 1, once
/// `021eae54(view)`: for each remaining index `c[0x11D] < c[0x11C]`: the mon (`+8`) and its record (`+0xC`, marked "skip");
/// without an answer ([`mon_answered`]), `r = 021cf81c(c, mon, &tmp)` (client3): `r < 4` -> `021d82d0(rec, n = 021d54c0(mon, r),
/// 021d81f8(server, pokecon, mon, n, c+0x38))`, else [`struggle`]; then state 2. State 2: result = `&c[0x124]`, size
/// `4 * c[0x11C]`, callback [`select_end`]. Returns 0.
pub fn select_finish<E: ClientUi>(env: &mut E, c: &mut [u8], c_addr: u32, state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            env.view(0x021E_AE44, &mut [Arg::V(view(c))]);
            set_st(state, st(state).wrapping_add(1));
        }
        1 => {
            if env.view(0x021E_AE54, &mut [Arg::V(view(c))]) == 0 {
                return 0;
            }
            while c[OFS_CUR] < c[OFS_COUNT] {
                let mon = client_mon(env, c, c[OFS_CUR] as u32);
                wr32(c, OFS_MON, mon);
                wr32(c, OFS_REC, answer_addr(c_addr, c[OFS_CUR]));
                let rec = rd32(c, OFS_REC);
                rec_skip(env, c, rec.wrapping_sub(c_addr) as usize);
                let mon = rd32(c, OFS_MON);
                if mon_answered(&mut Lend::h(env, c), mon, rec) == 0 {
                    let r = env.callh(
                        0x021C_F81C,
                        &mut [Arg::P, Arg::V(rd32(c, OFS_MON)), Arg::P],
                        c,
                    );
                    if r < 4 {
                        let n =
                            env.callh(0x021D_54C0, &mut [Arg::V(rd32(c, OFS_MON)), Arg::V(r)], c);
                        // the saved RNG context buffer at c + 0x38
                        let sv = c[0x38..].as_ptr() as usize;
                        let t = env.callh(
                            0x021D_81F8,
                            &mut [
                                Arg::V(rd32(c, OFS_SERVER)),
                                Arg::V(rd32(c, OFS_POKECON)),
                                Arg::V(rd32(c, OFS_MON)),
                                Arg::V(n),
                                Arg::PHost(sv),
                            ],
                            c,
                        );
                        env.callh(0x021D_82D0, &mut [Arg::PAt(rec), Arg::V(n), Arg::V(t)], c);
                    } else {
                        struggle(env, rec, c, rd32(c, OFS_MON));
                    }
                }
                c[OFS_CUR] = c[OFS_CUR].wrapping_add(1);
            }
            set_st(state, st(state).wrapping_add(1));
        }
        2 => {
            wr32(c, OFS_RESULT_PTR, c_addr.wrapping_add(OFS_ANSWERS as u32));
            wr32(c, OFS_RESULT_SIZE, 4 * c[OFS_COUNT] as u32);
            set_callback(c, CB_END);
        }
        _ => {}
    }
    0
}

/// C: `sub_021ce4d0`: callback 0x021CE4D1, the selection loop (a 9-way jump table on the state, an inner 5-way one on the
/// menu answer). Returns 0.
///
/// * 0: the mon of index `c[0x11D]` (`+8`), its record (`+0xC`, "skip"), `c[0x1AF] = 0`; answered ([`mon_answered`]) ->
///   callback [`select_next`]; a battle-item shortcut ([`item_shortcut`], item into `+0x19E`) -> callback [`item_anim`];
///   else state 1.
/// * 1: a new index (`(s8) c[0x11E] != c[0x11D]`) or `c[0x120]`: the "what will it do" message (`021ce0f8`, `021ea65c`),
///   `c[0x120] = 0`, `c[0x11E] = c[0x11D]`, state 2; else state 3.
/// * 2: message done -> state 3 and on as 3.   3: `021e9760(view, mon, c[0x11D] > c[0x11F], rec)`, state 4.
/// * 4: [`fade_poll_then`]`(c, finish)` -> `021e979c(view)`; else the menu answer `021e97a4(view)`: 1 fight -> no item use
///   (`021cf430`), callback [`fight_menu`]; 2 bag -> flag 0x21 (`021d62c4`) -> state 6, mode-1 flag `c[0x1AA] == 1` and
///   `!021ba1ec(server)` -> `021e9964(view)`, state 5, else callback [`bag_page`]; 3 party -> flag 0x21 -> `021e9964(view)`,
///   state 6, else no item use, callback [`party_page`]; 4 back -> at the first index: flag 0x21 and `021b86e4(server) != 2`
///   -> state 6, else no item use, callback [`run_menu`]; otherwise step back over the answered mons and undo the previous
///   answer (`021cf43c`, the counter `+0x1AB`, `021d8454` 3 -> veneer `021ce7c4(c+0x190)`, 2 -> [`clear_1a0_slot`]), callback
///   [`select_loop`].
/// * 5: `021e9974(view)` -> state 3.   6: `021e9954(view)`, state 7.
/// * 7: `021e9974(view)` -> message 0xC5 with the mon's name (`021d5440`), [`print_message`], state 8.
/// * 8: message done -> state 1.
pub fn select_loop<E: ClientUi>(env: &mut E, c: &mut [u8], c_addr: u32, state: &mut [u8]) -> u32 {
    let next = |s: &mut [u8]| set_st(s, st(s).wrapping_add(1));
    match st(state) {
        0 => {
            let mon = client_mon(env, c, c[OFS_CUR] as u32);
            wr32(c, OFS_MON, mon);
            wr32(c, OFS_REC, answer_addr(c_addr, c[OFS_CUR]));
            c[OFS_1AF] = 0;
            let rec = rd32(c, OFS_REC);
            rec_skip(env, c, rec.wrapping_sub(c_addr) as usize);
            let mon = rd32(c, OFS_MON);
            if mon_answered(&mut Lend::h(env, c), mon, rec) != 0 {
                set_callback(c, CB_NEXT);
                return 0;
            }
            if let Some(item) = item_shortcut(env, c) {
                wr16(c, OFS_ITEM, item);
                set_callback(c, CB_ITEM_ANIM);
                return 0;
            }
            next(state);
        }
        1 => {
            if c[OFS_SHOWN] as i8 as i32 != c[OFS_CUR] as i32 || c[OFS_120] != 0 {
                sent_out_message(env, c, rd32(c, OFS_MON), msg_buf(c_addr));
                env.view(0x021E_A65C, &mut [Arg::V(view(c)), Arg::P]);
                c[OFS_120] = 0;
                c[OFS_SHOWN] = c[OFS_CUR];
                next(state);
            } else {
                set_st(state, 3);
            }
        }
        2 | 3 => {
            if st(state) == 2 {
                if !msg_done(env, c) {
                    return 0;
                }
                next(state);
            }
            let later = (c[OFS_CUR] > c[OFS_FIRST]) as u32;
            env.view(
                0x021E_9760,
                &mut [
                    Arg::V(view(c)),
                    Arg::V(rd32(c, OFS_MON)),
                    Arg::V(later),
                    Arg::P,
                ],
            );
            next(state);
        }
        4 => {
            if fade_poll_then(env, c, CB_FINISH) != 0 {
                env.view(0x021E_979C, &mut [Arg::V(view(c))]);
                return 0;
            }
            match env.view(0x021E_97A4, &mut [Arg::V(view(c))]) {
                1 => {
                    set_130(c, c[OFS_CUR] as u32, 0);
                    set_callback(c, CB_FIGHT);
                }
                2 => {
                    if env.callh(
                        0x021D_62C4,
                        &mut [Arg::V(rd32(c, OFS_MON)), Arg::V(0x21)],
                        c,
                    ) != 0
                    {
                        set_st(state, 6);
                    } else if c[OFS_1AA] == 1
                        && env.callh(0x021B_A1EC, &mut [Arg::V(rd32(c, OFS_SERVER))], c) == 0
                    {
                        env.view(0x021E_9964, &mut [Arg::V(view(c))]);
                        set_st(state, 5);
                    } else {
                        set_callback(c, CB_BAG);
                    }
                }
                3 => {
                    if env.callh(
                        0x021D_62C4,
                        &mut [Arg::V(rd32(c, OFS_MON)), Arg::V(0x21)],
                        c,
                    ) != 0
                    {
                        env.view(0x021E_9964, &mut [Arg::V(view(c))]);
                        set_st(state, 6);
                    } else {
                        set_130(c, c[OFS_CUR] as u32, 0);
                        set_callback(c, CB_PARTY);
                    }
                }
                4 => {
                    if c[OFS_CUR] == c[OFS_FIRST] {
                        if env.callh(
                            0x021D_62C4,
                            &mut [Arg::V(rd32(c, OFS_MON)), Arg::V(0x21)],
                            c,
                        ) != 0
                            && env.callh(0x021B_86E4, &mut [Arg::V(rd32(c, OFS_SERVER))], c) != 2
                        {
                            set_st(state, 6);
                        } else {
                            set_130(c, c[OFS_CUR] as u32, 0);
                            set_callback(c, CB_RUN);
                        }
                        return 0;
                    }
                    if c[OFS_CUR] == 0 {
                        return 0;
                    }
                    loop {
                        c[OFS_CUR] = c[OFS_CUR].wrapping_sub(1);
                        let mon = client_mon(env, c, c[OFS_CUR] as u32);
                        if mon_answered(&mut Lend::h(env, c), mon, 0) == 0 {
                            break;
                        }
                        if c[OFS_CUR] == 0 {
                            return 0;
                        }
                    }
                    let used = get_130(c, c[OFS_CUR] as u32);
                    c[OFS_COUNTER] = c[OFS_COUNTER].wrapping_add(used);
                    let rec = answer_addr(c_addr, c[OFS_CUR]);
                    if env.callh(0x021D_8454, &mut [Arg::PAt(rec)], c) == 3 {
                        let p = c[0x190..].as_ptr() as usize;
                        env.callh(0x021C_E7C4, &mut [Arg::PHost(p)], c);
                    }
                    let rec = answer_addr(c_addr, c[OFS_CUR]);
                    if env.callh(0x021D_8454, &mut [Arg::PAt(rec)], c) == 2 {
                        clear_1a0_slot(env, c, c[OFS_CUR] as u32);
                    }
                    set_callback(c, CB_SELECT);
                }
                _ => {}
            }
        }
        5 => {
            if env.view(0x021E_9974, &mut [Arg::V(view(c))]) != 0 {
                set_st(state, 3);
            }
        }
        6 => {
            env.view(0x021E_9954, &mut [Arg::V(view(c))]);
            next(state);
        }
        7 => {
            if env.view(0x021E_9974, &mut [Arg::V(view(c))]) != 0 {
                msg_start(&mut Lend::h(env, c), msg_buf(c_addr), 1, 0xC5);
                let name = env.callh(0x021D_5440, &mut [Arg::V(rd32(c, OFS_MON))], c);
                msg_arg(&mut Lend::h(env, c), msg_buf(c_addr), name);
                print_message(env, c);
                next(state);
            }
        }
        8 if msg_done(env, c) => set_st(state, 1),
        _ => {}
    }
    0
}

/// C: `sub_021ce7d4`: callback 0x021CE7D5, the battle-item shortcut animation (a 4-way jump table on the state). Returns 0.
/// 0: `id = 021b904c(server, 0)`; `021f7f58(021ba020(server, id), 9, 0, 0, 0)`, `021f7998(0x270)`, `021ea6b0(view,
/// 021ba004(server, id), item +0x19E)`; item 0x13 / 0x14 with `021d7f38(021ba020(server, id))`, flag bit 1 clear and
/// (`021f87ec(0x47B)`, then) `021f8774() == 0` -> `02005e30(8)`, state 1; else state 2. 1: `02005e50() == 0` ->
/// `02005d90(0x47B, 0xFFFF)`, flag bit 1, state 2. 2: `021f7eb4() == 0` and message done -> `021f7998(0x271)`, state 3.
/// 3: `021f7eb4() == 0` -> callback [`select_loop`].
pub fn item_anim<E: ClientUi>(env: &mut E, c: &mut [u8], state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            let server = rd32(c, OFS_SERVER);
            let id = env.callh(0x021B_904C, &mut [Arg::V(server), Arg::V(0)], c);
            let bag = env.callh(
                0x021B_A004,
                &mut [Arg::V(rd32(c, OFS_SERVER)), Arg::V(id)],
                c,
            );
            let who = env.callh(
                0x021B_A020,
                &mut [Arg::V(rd32(c, OFS_SERVER)), Arg::V(id)],
                c,
            );
            env.sound(
                0x021F_7F58,
                &mut [Arg::V(who), Arg::V(9), Arg::V(0), Arg::V(0), Arg::V(0)],
            );
            env.sound(0x021F_7998, &mut [Arg::V(0x270)]);
            env.view(
                0x021E_A6B0,
                &mut [
                    Arg::V(view(c)),
                    Arg::V(bag),
                    Arg::V(rd16(c, OFS_ITEM) as u32),
                ],
            );
            let mut next = 2;
            if (rd16(c, OFS_ITEM).wrapping_sub(0x13)) <= 1 {
                let who = env.callh(
                    0x021B_A020,
                    &mut [Arg::V(rd32(c, OFS_SERVER)), Arg::V(id)],
                    c,
                );
                if env.callh(0x021D_7F38, &mut [Arg::V(who)], c) != 0 && c[OFS_FLAGS] & 1 == 0 {
                    env.sound(0x021F_87EC, &mut [Arg::V(0x47B)]);
                    if env.sound(0x021F_8774, &mut []) == 0 {
                        env.sound(0x0200_5E30, &mut [Arg::V(8)]);
                        next = 1;
                    }
                }
            }
            set_st(state, next);
        }
        1 => {
            if env.sound(0x0200_5E50, &mut []) == 0 {
                env.sound(0x0200_5D90, &mut [Arg::V(0x47B), Arg::V(0xFFFF)]);
                c[OFS_FLAGS] |= 1;
                set_st(state, st(state).wrapping_add(1));
            }
        }
        2 => {
            if env.sound(0x021F_7EB4, &mut []) == 0 && msg_done(env, c) {
                env.sound(0x021F_7998, &mut [Arg::V(0x271)]);
                set_st(state, st(state).wrapping_add(1));
            }
        }
        3 if env.sound(0x021F_7EB4, &mut []) == 0 => set_callback(c, CB_SELECT),
        _ => {}
    }
    0
}

/// C: `FUN_overlay_d_93__021ce8f0`: battle-item shortcut check (only when `021b86b8(server) == 1`): `id = 021b904c(server, 0)`,
/// the bag `021ba004(server, id)` (none -> `None`), the party `021b9aec(pokecon, id)`, the first live mon (`021b990c` /
/// `021d5b68`, up to `021b99b8(server, id)` slots). For each of the four items of the ROM table 0x021EFF78 whose flag
/// `c[0x112 + i]` is clear: an item the bag lacks (`0202a298(bag, item, heap id)`) gets its flag set; a present one passes its
/// test (item - 0x11: 0 `!021d5b48(mon)`, 1 HP <= `021d7b60(mon, 2)`, 2 party `021b9b94 > 1` and `021b9b98 == 1`, 3 both of
/// 2 and the HP test) -> flag set, remembered. Returns the last remembered item (the original writes it through its out
/// pointer and answers 1), else `None` (0).
pub fn item_shortcut<E: FlowEnv>(env: &mut E, c: &mut [u8]) -> Option<u16> {
    let server = rd32(c, OFS_SERVER);
    if env.callh(0x021B_86B8, &mut [Arg::V(server)], c) != 1 {
        return None;
    }
    let id = env.callh(0x021B_904C, &mut [Arg::V(server), Arg::V(0)], c);
    let bag = env.callh(0x021B_A004, &mut [Arg::V(server), Arg::V(id)], c);
    if bag == 0 {
        return None;
    }
    let pokecon = rd32(c, OFS_POKECON);
    let party = env.callh(0x021B_9AEC, &mut [Arg::V(pokecon), Arg::V(id)], c);
    let mut mon = env.callh(
        0x021B_990C,
        &mut [Arg::V(pokecon), Arg::V(id), Arg::V(0)],
        c,
    );
    if env.callh(0x021D_5B68, &mut [Arg::V(mon)], c) != 0 {
        let n = env.callh(0x021B_99B8, &mut [Arg::V(server), Arg::V(id)], c);
        let mut i = 1u32;
        while i < n {
            mon = env.callh(
                0x021B_990C,
                &mut [Arg::V(pokecon), Arg::V(id), Arg::V(i & 0xFF)],
                c,
            );
            if env.callh(0x021D_5B68, &mut [Arg::V(mon)], c) == 0 {
                break;
            }
            i += 1;
        }
    }
    let hp_low = |env: &mut E| {
        let hp = env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(0xD)]);
        hp <= env.call(0x021D_7B60, &mut [Arg::V(mon), Arg::V(2)])
    };
    let party_ok = |env: &mut E| {
        env.call(0x021B_9B94, &mut [Arg::V(party)]) > 1
            && env.call(0x021B_9B98, &mut [Arg::V(party)]) == 1
    };
    let mut found: i32 = -1;
    for (i, &item) in SHORTCUT_ITEMS.iter().enumerate() {
        if c[OFS_ITEM_FLAGS + i] != 0 {
            continue;
        }
        let heap = rd16(c, OFS_HEAP_ID) as u32;
        if env.callh(
            0x0202_A298,
            &mut [Arg::V(bag), Arg::V(item as u32), Arg::V(heap)],
            c,
        ) == 0
        {
            c[OFS_ITEM_FLAGS + i] = 1;
            continue;
        }
        let hit = match (item as u32).wrapping_sub(0x11) {
            0 => env.callh(0x021D_5B48, &mut [Arg::V(mon)], c) == 0,
            1 => hp_low(env),
            2 => party_ok(env),
            3 => party_ok(env) && hp_low(env),
            _ => false,
        };
        if hit {
            found = i as i32;
            c[OFS_ITEM_FLAGS + i] = 1;
        }
    }
    if found >= 0 {
        Some(SHORTCUT_ITEMS[found as usize])
    } else {
        None
    }
}

/// C: `sub_021cea94`: callback 0x021CEA95, the fight menu (an 8-way jump table on the state). Returns 0.
///
/// * 0: battle type 3 (`021b8588`) -> the double-battle move table ([`fight_table`]) at `+0xA4`, state 2; else
///   [`struggle_if_stuck`] -> callback [`select_next`], else state 1.
/// * 1: `021e97bc(view, mon, rec)`, state 3.   2: `021e9808(view, c+0xA4, rec)`, state 3.
/// * 3: [`fade_poll_then`]`(c, finish)` -> `021e98ac(view)`; else `021e98b4(view)` -> the record kind `021d8454(rec)`: 0 ->
///   callback [`select_loop`], 5 -> [`select_next`]; else the move of the record (bits 7..22 of its first word) and the
///   mon (battle type 3: side `(c+0xBC >> 4) & 7`, slot `021b9ab0(side)`, mon `c+0xA4 + 8 * slot`): a move with a target
///   choice (`021d832c(rec)`) -> `c[0x1B0]` = slot, `c[0x1B1] = 021d5778(mon, move)`, not 4 -> callback [`target_page`];
///   a blocked move (`021cf5b0(c, mon, move, buf)`, client3) -> `021eaf78(buf, 0xFF)`, [`print_message`], state 7; else (type
///   3, side >= 2, live mon) a second record (`021d8340(c+0xC0)`, `+0xC` += 4, `021d83fc`, `021d82d0(rec, x, 6)`,
///   `c[0x1AF]++`); state 4.
/// * 4: `021d7ef0(battle type)` -> state 5, else callback [`select_next`].   5: `021e98bc(view, mon, rec)`, state 6.
/// * 6: [`fade_poll_then`] -> `021e9920(view)`; else `021e98e8(view)`: 1 -> [`select_next`], 2 -> state 1.
/// * 7: message done -> `021e9858(view)`, state 3.
///
/// The record pointer `+0xC` points into the object (the spec fixes it to `&c[0x124]`); `c_addr` resolves it.
pub fn fight_menu<E: ClientUi>(env: &mut E, c: &mut [u8], c_addr: u32, state: &mut [u8]) -> u32 {
    let server = rd32(c, OFS_SERVER);
    match st(state) {
        0 => {
            if env.callh(0x021B_8588, &mut [Arg::V(server)], c) == 3 {
                let mut table = [0u8; FIGHT_TABLE_SIZE];
                fight_table(env, c, &mut table);
                c[OFS_FIGHT..OFS_FIGHT + FIGHT_TABLE_SIZE].copy_from_slice(&table);
                set_st(state, 2);
            } else if struggle_if_stuck(env, c, rd32(c, OFS_MON), rd32(c, OFS_REC)) != 0 {
                set_callback(c, CB_NEXT);
            } else {
                set_st(state, 1);
            }
        }
        1 => {
            env.view(
                0x021E_97BC,
                &mut [
                    Arg::V(view(c)),
                    Arg::V(rd32(c, OFS_MON)),
                    Arg::PAt(rd32(c, OFS_REC)),
                ],
            );
            set_st(state, 3);
        }
        2 => {
            // (view, &c[0xA4], rec)
            env.view(
                0x021E_9808,
                &mut [
                    Arg::V(view(c)),
                    Arg::PAt(c_addr.wrapping_add(0xA4)),
                    Arg::PAt(rd32(c, OFS_REC)),
                ],
            );
            set_st(state, 3);
        }
        3 => {
            if fade_poll_then(env, c, CB_FINISH) != 0 {
                env.view(0x021E_98AC, &mut [Arg::V(view(c))]);
                return 0;
            }
            if env.view_h(0x021E_98B4, &mut [Arg::V(view(c))], c) == 0 {
                return 0;
            }
            match env.callh(0x021D_8454, &mut [Arg::PAt(rd32(c, OFS_REC))], c) {
                0 => {
                    set_callback(c, CB_SELECT);
                    return 0;
                }
                5 => {
                    set_callback(c, CB_NEXT);
                    return 0;
                }
                _ => {}
            }
            let mut mon = rd32(c, OFS_MON);
            let rec = rd32(c, OFS_REC);
            let mut slot = c[OFS_CUR] as u32;
            let mut side = 0u32;
            if env.callh(0x021B_8588, &mut [Arg::V(server)], c) == 3 {
                side = (rd32(c, OFS_BC) >> 4) & 7;
                slot = env.callh(0x021B_9AB0, &mut [Arg::V(side)], c);
                mon = rd32(c, OFS_FIGHT + 8 * slot as usize);
            }
            let rec_word = |c: &[u8]| rd32(c, rec.wrapping_sub(c_addr) as usize);
            if env.callh(0x021D_832C, &mut [Arg::PAt(rec)], c) != 0 {
                c[OFS_1B0] = slot as u8;
                let mv = (rec_word(c) >> 7) & 0xFFFF;
                c[OFS_1B1] = env.callh(0x021D_5778, &mut [Arg::V(mon), Arg::V(mv)], c) as u8;
                if c[OFS_1B1] != 4 {
                    set_callback(c, CB_TARGET);
                    return 0;
                }
            }
            let mv = (rec_word(c) >> 7) & 0xFFFF;
            if env.callh(
                0x021C_F5B0,
                &mut [Arg::P, Arg::V(mon), Arg::V(mv), Arg::V(msg_buf(c_addr))],
                c,
            ) != 0
            {
                msg_end(&mut Lend::h(env, c), msg_buf(c_addr), 0xFF);
                print_message(env, c);
                set_st(state, 7);
                return 0;
            }
            if env.callh(0x021B_8588, &mut [Arg::V(server)], c) == 3
                && side != 0
                && side != 1
                && env.callh(0x021D_5B68, &mut [Arg::V(mon)], c) == 0
            {
                // `021d8340(c + 0xC0)` (as the original)
                let x = env.callh(0x021D_8340, &mut [Arg::PAt(c_addr.wrapping_add(0xC0))], c);
                let old = rd32(c, OFS_REC);
                wr32(c, OFS_REC, old.wrapping_add(4));
                env.callh(0x021D_83FC, &mut [Arg::PAt(old), Arg::V(side)], c);
                env.callh(
                    0x021D_82D0,
                    &mut [Arg::PAt(old.wrapping_add(4)), Arg::V(x), Arg::V(6)],
                    c,
                );
                c[OFS_1AF] = c[OFS_1AF].wrapping_add(1);
            }
            set_st(state, 4);
        }
        4 => {
            let t = env.callh(0x021B_8588, &mut [Arg::V(server)], c);
            if env.callh(0x021D_7EF0, &mut [Arg::V(t)], c) != 0 {
                set_st(state, 5);
            } else {
                set_callback(c, CB_NEXT);
            }
        }
        5 => {
            env.view(
                0x021E_98BC,
                &mut [Arg::V(view(c)), Arg::V(rd32(c, OFS_MON)), Arg::P],
            );
            set_st(state, 6);
        }
        6 => {
            if fade_poll_then(env, c, CB_FINISH) != 0 {
                env.view(0x021E_9920, &mut [Arg::V(view(c))]);
                return 0;
            }
            match env.view(0x021E_98E8, &mut [Arg::V(view(c))]) {
                1 => set_callback(c, CB_NEXT),
                2 => set_st(state, 1),
                _ => {}
            }
        }
        7 if msg_done(env, c) => {
            env.view(0x021E_9858, &mut [Arg::V(view(c))]);
            set_st(state, 3);
        }
        _ => {}
    }
    0
}

/// C: `FUN_overlay_d_93__021cecb4`: the move table of a double battle into `out` (3 x 8 bytes): for side `i < 3`, mon =
/// `021b9c00(party +0x118, i)`, `out[8i]` = mon, `out[8i + 4 + j] = 0` (j < 4); a live mon (`021d5b68 == 0`): for
/// `j < 021d5448(mon)`, `out[8i + 4 + j]` = the move is usable (`021d5584(mon, j)` and not blocked, `021cf5b0(c, mon,
/// 021d54c0(mon, j), 0)` == 0).
pub fn fight_table<E: FlowEnv>(env: &mut E, c: &[u8], out: &mut [u8]) {
    for i in 0..3usize {
        let mon = env.callr(
            0x021B_9C00,
            &mut [Arg::V(rd32(c, OFS_PARTY)), Arg::V(i as u32)],
            c,
        );
        wr32(out, 8 * i, mon);
        out[8 * i + 4..8 * i + 8].fill(0);
        if env.callr(0x021D_5B68, &mut [Arg::V(mon)], c) != 0 {
            continue;
        }
        let n = env.callr(0x021D_5448, &mut [Arg::V(mon)], c);
        let mut j = 0u32;
        while j < n {
            let ok = env.callr(0x021D_5584, &mut [Arg::V(mon), Arg::V(j & 0xFF)], c) != 0 && {
                let mv = env.callr(0x021D_54C0, &mut [Arg::V(mon), Arg::V(j & 0xFF)], c);
                env.callr(
                    0x021C_F5B0,
                    &mut [Arg::P, Arg::V(mon), Arg::V(mv), Arg::V(0)],
                    c,
                ) == 0
            };
            out[8 * i + 4 + j as usize] = ok as u8;
            j += 1;
        }
    }
}

/// C: `sub_021ced4c`: callback 0x021CED4D, the target page. 0: `021e9ce0(view, c[0x1B0], c[0x1B1])`, state 1. 1:
/// [`fade_poll_then`]`(c, 0)` -> `021e9b14(view)`, state 2; else `021e9b20(view)` -> callback [`fight_menu`]. 2:
/// `021e9b20(view)` -> callback [`select_finish`]. Returns 0.
pub fn target_page<E: ClientUi>(env: &mut E, c: &mut [u8], state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            env.view(
                0x021E_9CE0,
                &mut [
                    Arg::V(view(c)),
                    Arg::V(c[OFS_1B0] as u32),
                    Arg::V(c[OFS_1B1] as u32),
                ],
            );
            set_st(state, st(state).wrapping_add(1));
        }
        1 => {
            if fade_poll_then(env, c, 0) != 0 {
                env.view(0x021E_9B14, &mut [Arg::V(view(c))]);
                set_st(state, 2);
            } else if env.view(0x021E_9B20, &mut [Arg::V(view(c))]) != 0 {
                set_callback(c, CB_FIGHT);
            }
        }
        2 if env.view(0x021E_9B20, &mut [Arg::V(view(c))]) != 0 => set_callback(c, CB_FINISH),
        _ => {}
    }
    0
}

/// C: `sub_021cedc0`: callback 0x021CEDC1, the party page. 0: `r = 021cf8d4(c, mon, &a, &b)` (client3); `f` = 1 unless
/// `r == 4` and the low byte of `a` differs from `021d5440(mon)`; `021e9a78(view, c+0x184, c[0x11D], f, c+0x190)`, state 1.
/// 1: [`fade_poll_then`]`(c, 0)` -> `021e9b14(view)`, state 2; else `021e9b20(view)` -> veneer `021cee98(c+0x190)` set or
/// `n = ` veneer `021ceea8(c+0x190)` >= 6 -> callback [`select_loop`]; else `021d8394(rec, c[0x11D], n)`, callback
/// [`select_next`]. 2: `021e9b20(view)` -> callback [`select_finish`]. Returns 0.
pub fn party_page<E: ClientUi>(env: &mut E, c: &mut [u8], state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            let (mut a, mut b) = ([0u8; 2], [0u8; 2]);
            let r = env.callh(
                0x021C_F8D4,
                &mut [
                    Arg::P,
                    Arg::V(rd32(c, OFS_MON)),
                    Arg::Out(&mut a),
                    Arg::Out(&mut b),
                ],
                c,
            );
            let f = if r & 0xFF == 4 {
                let name = env.callh(0x021D_5440, &mut [Arg::V(rd32(c, OFS_MON))], c);
                (a[0] as u32 == name) as u32
            } else {
                1
            };
            env.view(
                0x021E_9A78,
                &mut [
                    Arg::V(view(c)),
                    Arg::P,
                    Arg::V(c[OFS_CUR] as u32),
                    Arg::V(f),
                    Arg::P,
                ],
            );
            set_st(state, st(state).wrapping_add(1));
        }
        1 => {
            if fade_poll_then(env, c, 0) != 0 {
                env.view(0x021E_9B14, &mut [Arg::V(view(c))]);
                set_st(state, 2);
            } else if env.view(0x021E_9B20, &mut [Arg::V(view(c))]) != 0 {
                if env.callh(0x021C_EE98, &mut [Arg::P], c) != 0 {
                    set_callback(c, CB_SELECT);
                } else {
                    let n = env.callh(0x021C_EEA8, &mut [Arg::P], c);
                    if n >= 6 {
                        set_callback(c, CB_SELECT);
                    } else {
                        env.callh(
                            0x021D_8394,
                            &mut [Arg::P, Arg::V(c[OFS_CUR] as u32), Arg::V(n)],
                            c,
                        );
                        set_callback(c, CB_NEXT);
                    }
                }
            }
        }
        2 if env.view(0x021E_9B20, &mut [Arg::V(view(c))]) != 0 => set_callback(c, CB_FINISH),
        _ => {}
    }
    0
}

/// C: `sub_021ceeb8`: callback 0x021CEEB9, the bag page. 0: `021e9d04(view, c[0x1AA], c[0x1AB], `[`sum_130`]`, c[0x11D] ==
/// c[0x11F], `[`single_live_mon`]`)`, state 1. 1: [`fade_poll_then`]`(c, 0)` -> `021e9e70(view)`, state 2; else
/// `021e9e80(view)` -> `item = 021ea01c(view)`, `page = 021ea044(view)`; no item or page 6 -> callback [`select_loop`];
/// else `n = 021ea030(view)`, `k = 021ea04c(view)`, the counter `+0x1AB` -= `n` (below `n`: 0 and `n` = 0),
/// [`set_130`]`(c, c[0x11D], n)`, `021d8358(rec, item, page, k)`, `021ea058(view)`, `021d805c(item, 0xF) == 4` -> flag bit
/// 4, [`set_1a0_slot`]`(c, c[0x11D], item)`, callback [`select_next`]. 2: `021e9e80(view)` -> callback [`select_finish`].
/// Returns 0.
pub fn bag_page<E: ClientUi>(env: &mut E, c: &mut [u8], state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            let s = sum_130(c);
            let first = (c[OFS_CUR] == c[OFS_FIRST]) as u32;
            let single = single_live_mon(env, c) & 0xFF;
            env.view(
                0x021E_9D04,
                &mut [
                    Arg::V(view(c)),
                    Arg::V(c[OFS_1AA] as u32),
                    Arg::V(c[OFS_COUNTER] as u32),
                    Arg::V(s),
                    Arg::V(first),
                    Arg::V(single),
                ],
            );
            set_st(state, st(state).wrapping_add(1));
        }
        1 => {
            if fade_poll_then(env, c, 0) != 0 {
                env.view(0x021E_9E70, &mut [Arg::V(view(c))]);
                set_st(state, 2);
                return 0;
            }
            if env.view(0x021E_9E80, &mut [Arg::V(view(c))]) == 0 {
                return 0;
            }
            let item = env.view(0x021E_A01C, &mut [Arg::V(view(c))]);
            let page = env.view(0x021E_A044, &mut [Arg::V(view(c))]);
            if item == 0 || page == 6 {
                set_callback(c, CB_SELECT);
                return 0;
            }
            let mut n = env.view(0x021E_A030, &mut [Arg::V(view(c))]);
            let k = env.view(0x021E_A04C, &mut [Arg::V(view(c))]);
            if c[OFS_COUNTER] as u32 >= n {
                c[OFS_COUNTER] = (c[OFS_COUNTER] as u32).wrapping_sub(n) as u8;
            } else {
                c[OFS_COUNTER] = 0;
                n = 0;
            }
            set_130(c, c[OFS_CUR] as u32, n);
            env.callh(
                0x021D_8358,
                &mut [Arg::P, Arg::V(item), Arg::V(page & 0xFF), Arg::V(k & 0xFF)],
                c,
            );
            env.view(0x021E_A058, &mut [Arg::V(view(c))]);
            if env.callh(0x021D_805C, &mut [Arg::V(item), Arg::V(0xF)], c) == 4 {
                c[OFS_FLAGS] |= 4;
            }
            set_1a0_slot(env, c, c[OFS_CUR] as u32, item);
            set_callback(c, CB_NEXT);
        }
        2 if env.view(0x021E_9E80, &mut [Arg::V(view(c))]) != 0 => set_callback(c, CB_FINISH),
        _ => {}
    }
    0
}

/// C: `FUN_021ceffc`: only when `021b86b8(server) == 0`: of the first `min(021b9b94(party), 021b8670(server))` mons of party 1
/// (`021b9af8(pokecon, 1)`, `021b9c00`), exactly one is alive (`021d5b68 == 0`) and some live one passes `021d5c30`.
pub fn single_live_mon<E: FlowEnv>(env: &mut E, c: &[u8]) -> u32 {
    if env.callr(0x021B_86B8, &mut [Arg::V(rd32(c, OFS_SERVER))], c) != 0 {
        return 0;
    }
    let party = env.callr(
        0x021B_9AF8,
        &mut [Arg::V(rd32(c, OFS_POKECON)), Arg::V(1)],
        c,
    );
    let n = env.callr(0x021B_9B94, &mut [Arg::V(party)], c);
    let mut m = env.callr(0x021B_8670, &mut [Arg::V(rd32(c, OFS_SERVER))], c);
    if m > n {
        m = n;
    }
    let (mut live, mut any) = (0u32, false);
    for i in 0..m {
        let mon = env.callr(0x021B_9C00, &mut [Arg::V(party), Arg::V(i & 0xFF)], c);
        if env.callr(0x021D_5B68, &mut [Arg::V(mon)], c) == 0 {
            live += 1;
            if env.callr(0x021D_5C30, &mut [Arg::V(mon)], c) != 0 {
                any = true;
            }
        }
    }
    (live == 1 && any) as u32
}

/// C: `sub_021cf078`: callback 0x021CF079, the run menu (a 6-way jump table on the state). Returns 0.
/// 0: by `021b86e4(server)`: 1 -> message 0x4C, state 1; 2 -> message 0x4D with `021eaf78(buf, 0xFF)`, state 3; else
/// `021cf918(c, &a, &b)` (client3) == 4 -> state 5, else message `b != 0 ? 0x365 (kind 2; a, b) : 0x4A`, state 1 (each message
/// shown with [`print_message`]). 1: message done -> the "what will it do" message (`021ce0f8`, `021ea65c`), state 2.
/// 2: message done -> callback [`select_loop`]. 3: `021ea710(view)` (answer unused), `021ea720(view)` -> the yes/no window
/// (`021eaf18(c+0x58, 3, 8)`, `021eaf18(c+0x7C, 3, 9)`, `021eabc4(view, c+0x58, c+0x7C, 1)`), state 4. 4: message done ->
/// `021eac54(view, &ans)` ? (`ans` ? [`select_loop`] : state 5) : [`fade_poll`] -> callback [`select_finish`].
/// 5: `021d8414(rec)`, flag bit 4, callback [`select_next`].
pub fn run_menu<E: ClientUi>(env: &mut E, c: &mut [u8], c_addr: u32, state: &mut [u8]) -> u32 {
    let buf = msg_buf(c_addr);
    match st(state) {
        0 => match env.callh(0x021B_86E4, &mut [Arg::V(rd32(c, OFS_SERVER))], c) {
            1 => {
                msg_start(&mut Lend::h(env, c), buf, 1, 0x4C);
                print_message(env, c);
                set_st(state, 1);
            }
            2 => {
                msg_start(&mut Lend::h(env, c), buf, 1, 0x4D);
                msg_end(&mut Lend::h(env, c), buf, 0xFF);
                print_message(env, c);
                set_st(state, 3);
            }
            _ => {
                let (mut a, mut b) = ([0u8; 2], [0u8; 2]);
                let r = env.callh(
                    0x021C_F918,
                    &mut [Arg::P, Arg::Out(&mut a), Arg::Out(&mut b)],
                    c,
                );
                if r == 4 {
                    set_st(state, 5);
                    return 0;
                }
                let b = u16::from_le_bytes(b) as u32;
                if b != 0 {
                    msg_start(&mut Lend::h(env, c), buf, 2, 0x365);
                    msg_arg(&mut Lend::h(env, c), buf, a[0] as u32);
                    msg_arg(&mut Lend::h(env, c), buf, b);
                } else {
                    msg_start(&mut Lend::h(env, c), buf, 1, 0x4A);
                }
                print_message(env, c);
                set_st(state, 1);
            }
        },
        1 => {
            if msg_done(env, c) {
                sent_out_message(env, c, rd32(c, OFS_MON), buf);
                env.view(0x021E_A65C, &mut [Arg::V(view(c)), Arg::P]);
                set_st(state, 2);
            }
        }
        2 => {
            if msg_done(env, c) {
                set_callback(c, CB_SELECT);
            }
        }
        3 => {
            msg_done(env, c);
            if env.view(0x021E_A720, &mut [Arg::V(view(c))]) != 0 {
                msg_start(&mut Lend::h(env, c), buf, 3, 8);
                msg_start(
                    &mut Lend::h(env, c),
                    c_addr.wrapping_add(OFS_MSG2 as u32),
                    3,
                    9,
                );
                env.view(
                    0x021E_ABC4,
                    &mut [Arg::V(view(c)), Arg::P, Arg::P, Arg::V(1)],
                );
                set_st(state, 4);
            }
        }
        4 => {
            if !msg_done(env, c) {
                return 0;
            }
            let mut ans = [0u8; 4];
            if env.view(0x021E_AC54, &mut [Arg::V(view(c)), Arg::Out(&mut ans)]) != 0 {
                if u32::from_le_bytes(ans) != 0 {
                    set_callback(c, CB_SELECT);
                } else {
                    set_st(state, 5);
                }
            } else if fade_poll(env, c) != 0 {
                set_callback(c, CB_FINISH);
            }
        }
        5 => {
            env.callh(0x021D_8414, &mut [Arg::P], c);
            c[OFS_FLAGS] |= 4;
            set_callback(c, CB_NEXT);
        }
        _ => {}
    }
    0
}

/// C: `sub_021cf220`: callback 0x021CF221, the next mon. 0: `021e9964(view)`, state 1. 1: once `021e9974(view)`: index
/// `+0x11D` + 1; more mons: flag bit 4 clear -> callback [`select_loop`]; set -> every remaining mon without an answer
/// ([`mon_answered`] on its record) gets a "skip" record. All done: answers = `c[0x11C] + c[0x1AF]`, `c[0x1AF] = 0`, result =
/// `&c[0x124]`, size `4 * (answers & 0xFF)`, callback [`select_end`]. Returns 0.
pub fn select_next<E: ClientUi>(env: &mut E, c: &mut [u8], c_addr: u32, state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            env.view(0x021E_9964, &mut [Arg::V(view(c))]);
            set_st(state, st(state).wrapping_add(1));
            0
        }
        1 => {
            if env.view(0x021E_9974, &mut [Arg::V(view(c))]) == 0 {
                return 0;
            }
            c[OFS_CUR] = c[OFS_CUR].wrapping_add(1);
            if c[OFS_CUR] < c[OFS_COUNT] {
                if c[OFS_FLAGS] & 4 == 0 {
                    set_callback(c, CB_SELECT);
                    return 0;
                }
                while c[OFS_CUR] < c[OFS_COUNT] {
                    let mon = client_mon(env, c, c[OFS_CUR] as u32);
                    let rec = answer_addr(c_addr, c[OFS_CUR]);
                    if mon_answered(&mut Lend::h(env, c), mon, rec) == 0 {
                        rec_skip(env, c, rec.wrapping_sub(c_addr) as usize);
                    }
                    c[OFS_CUR] = c[OFS_CUR].wrapping_add(1);
                }
            }
            let total = c[OFS_COUNT] as u32 + c[OFS_1AF] as u32;
            c[OFS_1AF] = 0;
            wr32(c, OFS_RESULT_PTR, c_addr.wrapping_add(OFS_ANSWERS as u32));
            wr32(c, OFS_RESULT_SIZE, (total & 0xFF) << 2);
            set_callback(c, CB_END);
            0
        }
        _ => 0,
    }
}

/// C: `sub_021cf2f0`: callback 0x021CF2F1, the end of the selection. 0: [`clear_1a0_all`]; `021b86c0(server)` -> `c[0x1A9] = 1`,
/// `021ea7fc(view)`, state 1; else state 2. 1: `021ea814(view)` -> state 2. 2: `021f7eb4() == 0` -> [`fade_end`], state 3, 1.
/// Other states 1. Else 0.
pub fn select_end<E: ClientUi>(env: &mut E, c: &mut [u8], state: &mut [u8]) -> u32 {
    match st(state) {
        0 => {
            clear_1a0_all(env, c);
            if env.callh(0x021B_86C0, &mut [Arg::V(rd32(c, OFS_SERVER))], c) != 0 {
                c[OFS_1A9] = 1;
                env.view(0x021E_A7FC, &mut [Arg::V(view(c))]);
                set_st(state, st(state).wrapping_add(1));
            } else {
                set_st(state, st(state).wrapping_add(2));
            }
            0
        }
        1 => {
            if env.view(0x021E_A814, &mut [Arg::V(view(c))]) != 0 {
                set_st(state, st(state).wrapping_add(1));
            }
            0
        }
        2 => {
            if env.sound(0x021F_7EB4, &mut []) != 0 {
                return 0;
            }
            fade_end(env, c);
            set_st(state, st(state).wrapping_add(1));
            1
        }
        _ => 1,
    }
}

/// C: `FUN_overlay_d_93__021cf360`: the three halfwords at `+0x1A0` = 0.
pub fn clear_1a0(c: &mut [u8]) {
    for i in 0..3 {
        wr16(c, OFS_1A0 + 2 * i, 0);
    }
}

/// C: `FUN_overlay_d_93__021cf37c`: with `c[0x1AA] == 0` and `021b8ff4(server)` equal to the client id: `c.u16[0x1A0 + 2 i]`
/// = `v`, `021b9090(server, client id, v)`.
pub fn set_1a0_slot<E: FlowEnv>(env: &mut E, c: &mut [u8], i: u32, v: u32) {
    if c[OFS_1AA] != 0 {
        return;
    }
    if c[OFS_ID] as u32 != env.callh(0x021B_8FF4, &mut [Arg::V(rd32(c, OFS_SERVER))], c) {
        return;
    }
    wr16(
        c,
        (OFS_1A0 as u32).wrapping_add(i.wrapping_mul(2)) as usize,
        v as u16,
    );
    env.callh(
        0x021B_9090,
        &mut [
            Arg::V(rd32(c, OFS_SERVER)),
            Arg::V(c[OFS_ID] as u32),
            Arg::V(v),
        ],
        c,
    );
}

/// C: `FUN_overlay_d_93__021cf3b8`: with `c[0x1AA] == 0` and `021b8ff4(server)` equal to the client id: a set slot
/// `c.u16[0x1A0 + 2 i]` becomes 0 and `021b90bc(server, client id)`.
pub fn clear_1a0_slot<E: FlowEnv>(env: &mut E, c: &mut [u8], i: u32) {
    if c[OFS_1AA] != 0 {
        return;
    }
    if c[OFS_ID] as u32 != env.callh(0x021B_8FF4, &mut [Arg::V(rd32(c, OFS_SERVER))], c) {
        return;
    }
    let ofs = (OFS_1A0 as u32).wrapping_add(i.wrapping_mul(2)) as usize;
    if rd16(c, ofs) == 0 {
        return;
    }
    wr16(c, ofs, 0);
    env.callh(
        0x021B_90BC,
        &mut [Arg::V(rd32(c, OFS_SERVER)), Arg::V(c[OFS_ID] as u32)],
        c,
    );
}

/// C: `FUN_overlay_d_93__021cf3f8`: with `c[0x1AA] == 0`: [`clear_1a0_slot`] for slots 0..3.
pub fn clear_1a0_all<E: FlowEnv>(env: &mut E, c: &mut [u8]) {
    if c[OFS_1AA] != 0 {
        return;
    }
    for i in 0..3 {
        clear_1a0_slot(env, c, i);
    }
}

/// C: `FUN_overlay_d_93__021cf41c`: the three bytes at `+0x130` = 0 (`MI_CpuFill8`).
pub fn clear_130(c: &mut [u8]) {
    c[OFS_130..OFS_130 + 3].fill(0);
}

/// C: `FUN_overlay_d_93__021cf430`: `c[0x130 + i]` = `v` (a byte).
pub fn set_130(c: &mut [u8], i: u32, v: u32) {
    c[OFS_130 + i as usize] = v as u8;
}

/// C: `FUN_overlay_d_93__021cf43c`: `c[0x130 + i]`.
pub fn get_130(c: &[u8], i: u32) -> u8 {
    c[OFS_130 + i as usize]
}

/// C: `FUN_overlay_d_93__021cf448`: the byte sum of `c[0x130 .. 0x130 + c[0x11D]]`.
pub fn sum_130(c: &[u8]) -> u32 {
    let n = c[OFS_CUR] as usize;
    let s = c[OFS_130..OFS_130 + n]
        .iter()
        .fold(0u32, |a, &b| a.wrapping_add(b as u32));
    s & 0xFF
}

/// C: `FUN_overlay_d_93__021cf470`: does `mon` need no choice? Fainted (`021d5b68`) -> "skip" record (`021d8434(rec)`); flag
/// `021d5bdc(mon, 0xC)` -> `021d8444(rec)`; flag 0x19 (`021d62c4`) -> `021d82d0(rec, 021e8b80(021d6314(mon, 0x19)),
/// 021d67c4(mon))`; flag 0x1A -> `m = 021d67a8(mon)`, `021d5778(mon, m)`, `021d82d0(rec, m, 021d67c4(mon))`; each returns 1
/// (the record calls only with a record, `rec != 0`). Else 0. (The original's first argument, the client, is unused.)
pub fn mon_answered<E: FlowEnv>(env: &mut E, mon: u32, rec: u32) -> u32 {
    if env.call(0x021D_5B68, &mut [Arg::V(mon)]) != 0 {
        if rec != 0 {
            env.call(0x021D_8434, &mut [Arg::PAt(rec)]);
        }
        return 1;
    }
    if env.call(0x021D_5BDC, &mut [Arg::V(mon), Arg::V(0xC)]) != 0 {
        if rec != 0 {
            env.call(0x021D_8444, &mut [Arg::PAt(rec)]);
        }
        return 1;
    }
    if env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x19)]) != 0 {
        let w = env.call(0x021D_6314, &mut [Arg::V(mon), Arg::V(0x19)]);
        let m = env.call(0x021E_8B80, &mut [Arg::V(w)]);
        let t = env.call(0x021D_67C4, &mut [Arg::V(mon)]);
        if rec != 0 {
            env.call(0x021D_82D0, &mut [Arg::PAt(rec), Arg::V(m), Arg::V(t)]);
        }
        return 1;
    }
    if env.call(0x021D_62C4, &mut [Arg::V(mon), Arg::V(0x1A)]) != 0 {
        let m = env.call(0x021D_67A8, &mut [Arg::V(mon)]);
        let t = env.call(0x021D_67C4, &mut [Arg::V(mon)]);
        env.call(0x021D_5778, &mut [Arg::V(mon), Arg::V(m)]);
        if rec != 0 {
            env.call(0x021D_82D0, &mut [Arg::PAt(rec), Arg::V(m), Arg::V(t)]);
        }
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021cf524`: no usable move (for `j < 021d5448(mon)`: not blocked, `021cf5b0(c, mon, 021d54c0(mon, j),
/// 0)` == 0, and `021d5584(mon, j)`): [`struggle`] into `rec`, returns 1; else 0.
pub fn struggle_if_stuck<E: FlowEnv>(env: &mut E, c: &mut [u8], mon: u32, rec: u32) -> u32 {
    let n = env.callh(0x021D_5448, &mut [Arg::V(mon)], c);
    let mut usable = 0u32;
    let mut j = 0u32;
    while j < n {
        let mv = env.callh(0x021D_54C0, &mut [Arg::V(mon), Arg::V(j & 0xFF)], c);
        if env.callh(
            0x021C_F5B0,
            &mut [Arg::P, Arg::V(mon), Arg::V(mv), Arg::V(0)],
            c,
        ) == 0
            && env.callh(0x021D_5584, &mut [Arg::V(mon), Arg::V(j & 0xFF)], c) != 0
        {
            usable += 1;
        }
        j += 1;
    }
    if usable == 0 {
        struggle(env, rec, c, mon);
        return 1;
    }
    0
}

/// C: `FUN_overlay_d_93__021cf58c`: the struggle answer into the record: `021d82d0(rec, 0xA5, 021d81f8(server, pokecon, mon,
/// 0xA5, c+0x38))` (the record is only passed on).
pub fn struggle<E: FlowEnv>(env: &mut E, rec: u32, c: &mut [u8], mon: u32) {
    let c38 = c[0x38..].as_ptr() as usize;
    let t = env.callh(
        0x021D_81F8,
        &mut [
            Arg::V(rd32(c, OFS_SERVER)),
            Arg::V(rd32(c, OFS_POKECON)),
            Arg::V(mon),
            Arg::V(MOVE_STRUGGLE),
            Arg::PHost(c38),
        ],
        c,
    );
    env.callh(
        0x021D_82D0,
        &mut [Arg::PAt(rec), Arg::V(MOVE_STRUGGLE), Arg::V(t)],
        c,
    );
}

#[cfg(test)]
mod tests;
