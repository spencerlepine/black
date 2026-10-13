//! The server-side state functions (B10e): `fn(S, &substate) -> done`, run once per frame by
//! [`super::run_state`] (`021bab9c`) through the function pointer at `S[0]`.
//!
//! The substate word is `S + 4` when `021bab9c` calls them; the differential test gives it a
//! buffer of its own, so state functions reach it through [`StateWord`] (the original writes
//! `S + 4` too, through `021babec`, and the two must not be mixed up).

use super::*;

/// Where a state function's `&substate` argument points.
pub enum StateWord<'a> {
    /// `S + 4` (the production case, `021bab9c`).
    InS,
    /// A separate 4-byte word (the differential test).
    Separate(&'a mut [u8]),
}

impl StateWord<'_> {
    fn get(&self, s: &[u8]) -> u32 {
        match self {
            StateWord::InS => rd32(&s[4..]),
            StateWord::Separate(b) => rd32(b),
        }
    }

    fn set(&mut self, s: &mut [u8], v: u32) {
        match self {
            StateWord::InS => s[4..8].copy_from_slice(&v.to_le_bytes()),
            StateWord::Separate(b) => b[..4].copy_from_slice(&v.to_le_bytes()),
        }
    }

    fn bump(&mut self, s: &mut [u8]) {
        let v = self.get(s).wrapping_add(1);
        self.set(s, v);
    }
}

const STATE_ACAD: u32 = 0x021B_ACAD;
const STATE_AE61: u32 = 0x021B_AE61;
const STATE_AEF9: u32 = 0x021B_AEF9;
const STATE_BB0E1: u32 = 0x021B_B0E1;
const STATE_BB209: u32 = 0x021B_B209;
const STATE_BB281: u32 = 0x021B_B281;
const STATE_BB321: u32 = 0x021B_B321;

/// `021bb724(S, 8, hdr + 8, *hdr)` with `hdr = *(S + 0xCB0)`.
fn send_header<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8]) {
    let hdr = rd32(&s[0xCB0..]);
    let first = rd32(&s[inner(s, s_addr, 0xCB0)..]);
    broadcast(env, s, 8, hdr + 8, first);
}

/// `021cc668(S[0x44])` (a thunk target; ported in the events area, which takes a `SvFlowWork`
/// this object only addresses): the stubbed call.
fn who_leads<E: FlowEnv>(env: &mut E, s: &[u8]) -> u32 {
    env.call(0x021C_C668, &mut [Arg::V(ptr_0x44(s))])
}

/// C: `sub_021bac04`.
pub fn state_ac04<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    let mut cur = st.get(s);
    if cur == 0 {
        broadcast_code(env, s, 1);
        st.bump(s);
        cur = 1;
    }
    match cur {
        1 => {
            if all_ready(env, s) {
                each_eeb70(env, s);
                if env.call(0x021B_86B8, &mut [Arg::V(rd32(&s[0xC..]))]) == 4 {
                    set_state(s, STATE_BB281);
                } else {
                    let y = env.callh(0x021B_BB60, &mut [Arg::V(ptr_0x44(s))], s);
                    builder_load(env, s_addr, s, y);
                    st.bump(s);
                }
            }
        }
        2 => {
            if !all_ready(env, s) {
                return 0;
            }
            each_eeb70(env, s);
            st.bump(s);
        }
        3 => {
            send_header(env, s_addr, s);
            st.bump(s);
        }
        4 if all_ready(env, s) => {
            each_eeb70(env, s);
            set_state(s, STATE_ACAD);
        }
        _ => {}
    }
    0
}

/// C: `sub_021bacac`.
pub fn state_acac<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    let main = rd32(&s[0xC..]);
    let mut cur = st.get(s);
    // case 5 runs on into case 6
    if cur == 5 {
        env.callh(0x021B_BC34, &mut [Arg::V(ptr_0x44(s))], s);
        each_eeb70(env, s);
        st.bump(s);
        cur = 6;
    }
    match cur {
        0 => {
            if env.callh(0x021B_C390, &mut [Arg::V(ptr_0x44(s))], s) != 0 {
                send_header(env, s_addr, s);
                st.bump(s);
            } else {
                let v = st.get(s).wrapping_add(2);
                st.set(s, v);
            }
        }
        1 | 4 => {
            if all_ready(env, s) {
                each_eeb70(env, s);
                st.bump(s);
            }
        }
        2 => {
            broadcast_code(env, s, 3);
            st.bump(s);
        }
        3 => {
            if !all_ready(env, s) {
                return 0;
            }
            each_eeb70(env, s);
            if env.callh(0x021B_A5D8, &mut [Arg::V(main)], s) != 0 {
                set_state(s, STATE_BB209);
                return 0;
            }
            if env.callh(0x021B_A618, &mut [Arg::V(main)], s) != 0 {
                set_state(s, STATE_BB281);
                return 0;
            }
            collect_slot_records(env, s_addr, s);
            let one = always_true();
            if builder_flush(env, s_addr, s, 1, one) {
                st.bump(s);
            } else {
                let v = st.get(s).wrapping_add(2);
                st.set(s, v);
            }
        }
        6 => {
            let r = env.callh(
                0x021B_BC44,
                &mut [Arg::V(ptr_0x44(s)), Arg::V(s_addr + OFS_RECORDS as u32)],
                s,
            );
            s[0x48..0x4C].copy_from_slice(&r.to_le_bytes());
            if env.callh(0x021B_86C0, &mut [Arg::V(main)], s) != 0 {
                nop();
            }
            send_header(env, s_addr, s);
            st.bump(s);
        }
        7 => {
            if !all_ready(env, s) {
                return 0;
            }
            each_eeb70(env, s);
            match rd32(&s[0x48..]) {
                0 => set_state_acad(s),
                1 => set_state(s, STATE_BB0E1),
                2 => {
                    let b = env.callh(0x021B_86B8, &mut [Arg::V(main)], s);
                    let c = env.callh(0x021B_8588, &mut [Arg::V(main)], s);
                    set_state(
                        s,
                        if b == 0 && c == 0 {
                            STATE_AE61
                        } else {
                            STATE_AEF9
                        },
                    );
                }
                3 => st.set(s, 6),
                4 => {
                    let x = who_leads(env, s);
                    env.callh(0x021B_916C, &mut [Arg::V(main), Arg::V(x)], s);
                    set_state(s, STATE_BB281);
                }
                6 => {
                    let x = env.callh(0x021B_C55C, &mut [Arg::V(ptr_0x44(s))], s);
                    env.callh(0x021B_9120, &mut [Arg::V(main), Arg::V(x)], s);
                    set_state(s, STATE_BB281);
                }
                _ => return 1,
            }
        }
        _ => {}
    }
    0
}

/// C: `sub_021bae60`.
pub fn state_ae60<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    match st.get(s) {
        0 => {
            broadcast_code(env, s, 7);
            st.bump(s);
        }
        1 => {
            if !all_ready(env, s) {
                return 0;
            }
            let main = rd32(&s[0xC..]);
            let x = env.callh(0x021B_8FF4, &mut [Arg::V(main)], s);
            each_eeb70(env, s);
            let h = rd32(&s[OFS_SLOTS.wrapping_add((x as usize).wrapping_mul(12))..]);
            let rec = env.callh(0x021E_EB64, &mut [Arg::V(h), Arg::V(0)], s);
            if s[rec.wrapping_sub(s_addr) as usize] != 0 {
                let v = env.callh(0x021B_C360, &mut [Arg::V(ptr_0x44(s))], s);
                s[0x48..0x4C].copy_from_slice(&v.to_le_bytes());
                send_header(env, s_addr, s);
                st.bump(s);
            } else {
                set_state(s, STATE_AEF9);
            }
        }
        2 => {
            if !all_ready(env, s) {
                return 0;
            }
            each_eeb70(env, s);
            if rd32(&s[0x48..]) != 0 {
                return 1;
            }
            set_state(s, STATE_AEF9);
        }
        _ => {}
    }
    0
}

/// C: `sub_021baef8`.
pub fn state_aef8<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    let main = rd32(&s[0xC..]);
    let mut cur = st.get(s);
    // case 4 runs on into case 5
    if cur == 4 {
        each_eeb70(env, s);
        env.callh(0x021B_BD80, &mut [Arg::V(ptr_0x44(s))], s);
        st.bump(s);
        cur = 5;
    }
    match cur {
        0 => {
            broadcast(
                env,
                s,
                5,
                s_addr + OFS_LIST as u32,
                s[OFS_LIST_COUNT] as u32,
            );
            st.bump(s);
        }
        1 => {
            if !all_ready(env, s) {
                return 0;
            }
            if party_ready(env, s) {
                let v = slot1_value(env, s_addr, s);
                s[0xEC] = v as u8;
                send_one(env, s, 4, 0, s_addr + 0xEC, 1);
            }
            st.bump(s);
        }
        2 => {
            if !all_ready(env, s) {
                return 0;
            }
            each_eeb70(env, s);
            if env.callh(0x021B_A618, &mut [Arg::V(main)], s) != 0 {
                set_state(s, STATE_BB281);
                return 0;
            }
            collect_slot_records(env, s_addr, s);
            if builder_flush(env, s_addr, s, 2, 0) {
                st.bump(s);
            } else {
                let v = st.get(s).wrapping_add(2);
                st.set(s, v);
            }
        }
        3 => {
            if !all_ready(env, s) {
                return 0;
            }
            each_eeb70(env, s);
            st.bump(s);
        }
        5 => {
            let r = env.callh(
                0x021B_BD8C,
                &mut [Arg::V(ptr_0x44(s)), Arg::V(s_addr + OFS_RECORDS as u32)],
                s,
            );
            s[0x48..0x4C].copy_from_slice(&r.to_le_bytes());
            send_header(env, s_addr, s);
            st.bump(s);
        }
        6 => {
            if !all_ready(env, s) {
                return 0;
            }
            each_eeb70(env, s);
            match rd32(&s[0x48..]) {
                2 => st.set(s, 0),
                3 => st.set(s, 5),
                4 => {
                    let x = who_leads(env, s);
                    env.callh(0x021B_916C, &mut [Arg::V(main), Arg::V(x)], s);
                    set_state(s, STATE_BB281);
                }
                5 => return 1,
                _ => set_state_acad(s),
            }
        }
        _ => {}
    }
    0
}

/// C: `sub_021bb208`.
pub fn state_b208<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    match st.get(s) {
        0 => {
            let mut local = [0u8; 4];
            let r = env.callh(
                0x021E_E9A4,
                &mut [Arg::V(s_addr + OFS_BUILDER as u32), Arg::Out(&mut local)],
                s,
            );
            broadcast(env, s, 10, r, u32::from_le_bytes(local));
            st.bump(s);
        }
        1 => {
            if all_ready(env, s) {
                each_eeb70(env, s);
                broadcast_code(env, s, 0x10);
                st.bump(s);
            }
        }
        2 if all_ready(env, s) => {
            each_eeb70(env, s);
            let x = who_leads(env, s);
            env.call(0x021B_916C, &mut [Arg::V(rd32(&s[0xC..])), Arg::V(x)]);
            set_state(s, STATE_BB281);
        }
        _ => {}
    }
    0
}

/// C: `sub_021bb0e0`: the record-exchange state. Phases 0..=5:
/// 0 sends the id list, 1 collects the clients' records and flushes them, 2 waits, 3/4 reset the header
/// and ask `021bbd2c` for the next step (phase 3 runs on into phase 4's body, so it advances by 2), 5 acts
/// on the step stored in `S[0x48]`.
pub fn state_b0e0<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    let main = rd32(&s[0xC..]);
    let mut phase = st.get(s);
    match phase {
        0 => {
            let n = s[OFS_LIST_COUNT] as u32;
            broadcast(env, s, 6, s_addr + OFS_LIST as u32, n);
            st.bump(s);
        }
        1 => {
            if !all_ready(env, s) {
                return 0;
            }
            each_eeb70(env, s);
            if env.callh(0x021B_A618, &mut [Arg::V(main)], s) != 0 {
                set_state(s, STATE_BB281);
                return 0;
            }
            collect_slot_records(env, s_addr, s);
            if builder_flush(env, s_addr, s, 3, 0) {
                st.bump(s);
            } else {
                let v = st.get(s).wrapping_add(2);
                st.set(s, v);
            }
        }
        2 => {
            if all_ready(env, s) {
                each_eeb70(env, s);
                st.bump(s);
            }
        }
        3 | 4 => {
            if phase == 3 {
                each_eeb70(env, s);
                env.callh(0x021B_BD20, &mut [Arg::V(ptr_0x44(s))], s);
                st.bump(s);
                phase = 4;
            }
            debug_assert_eq!(phase, 4);
            let hdr = inner(s, s_addr, 0xCB0);
            s[hdr..hdr + 8].fill(0);
            let r = env.callh(
                0x021B_BD2C,
                &mut [Arg::V(ptr_0x44(s)), Arg::V(s_addr + OFS_RECORDS as u32)],
                s,
            );
            s[0x48..0x4C].copy_from_slice(&r.to_le_bytes());
            send_header(env, s_addr, s);
            st.bump(s);
        }
        5 => {
            if !all_ready(env, s) {
                return 0;
            }
            each_eeb70(env, s);
            match rd32(&s[0x48..]) {
                1 => st.set(s, 0),
                2 => set_state(s, STATE_AEF9),
                3 => st.set(s, 4),
                4 => {
                    let x = who_leads(env, s);
                    env.callh(0x021B_916C, &mut [Arg::V(main), Arg::V(x)], s);
                    set_state(s, STATE_BB281);
                }
                5 => return 1,
                _ => set_state_acad(s),
            }
        }
        _ => {}
    }
    0
}

/// C: `sub_021bb320`: polls until the main object (or the link layer) says go.
pub fn state_b320<E: FlowEnv>(env: &mut E, s: &mut [u8], st: &mut StateWord) -> u32 {
    match st.get(s) {
        0 => {
            let ready = env.call(0x021B_9114, &mut [Arg::V(rd32(&s[0xC..]))]) != 0
                || env.call(0x0203_62DC, &mut []) & 3 != 0
                || env.call(0x0203_5C60, &mut []) != 0;
            if ready {
                st.bump(s);
                return 1;
            }
            0
        }
        1 => 1,
        _ => 0,
    }
}

/// The common shape of `bb3b0`/`bb3f8`/`bb43c`/`bb484`: broadcast (`code`, `S + 0xA8`, 4), wait,
/// then a final step.
fn state_simple<E: FlowEnv>(
    env: &mut E,
    s_addr: u32,
    s: &mut [u8],
    st: &mut StateWord,
    code: u32,
    last: impl FnOnce(&mut E, &mut [u8]) -> u32,
) -> u32 {
    match st.get(s) {
        0 => {
            broadcast(env, s, code, s_addr + 0xA8, 4);
            st.bump(s);
        }
        1 => {
            if all_ready(env, s) {
                each_eeb70(env, s);
                st.bump(s);
            }
        }
        2 => return last(env, s),
        _ => {}
    }
    0
}

/// C: `sub_021bb368`.
pub fn state_b368<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    state_simple(env, s_addr, s, st, 0xE, |_, s| {
        set_state(s, STATE_BB321);
        0
    })
}

/// C: `sub_021bb3b0`.
pub fn state_b3b0<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    state_simple(env, s_addr, s, st, 0xF, |_, s| {
        set_state(s, STATE_BB321);
        0
    })
}

/// C: `sub_021bb3f8`.
pub fn state_b3f8<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    state_simple(env, s_addr, s, st, 0xD, |_, _| 1)
}

/// C: `sub_021bb43c`.
pub fn state_b43c<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    state_simple(env, s_addr, s, st, 0xB, |_, s| {
        set_state(s, STATE_BB321);
        0
    })
}

/// C: `sub_021bb484`.
pub fn state_b484<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], st: &mut StateWord) -> u32 {
    state_simple(env, s_addr, s, st, 0xC, |env, s| {
        if env.call(0x021B_86C0, &mut [Arg::V(rd32(&s[0xC..]))]) == 0 {
            set_state(s, STATE_BB321);
            0
        } else {
            1
        }
    })
}

/// C: `FUN_overlay_d_93__021bab9c`: runs the current state function (`S[0]`, called as
/// `(S, S + 4)`); when it answers done, broadcasts (9, `S[0xB0]`, 8) and moves to phase 2,
/// where it waits for every client and then answers 1.
pub fn run_state<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8]) -> u32 {
    if s[0xED] == 2 {
        return all_ready(env, s) as u32;
    }
    let f = rd32(s) & !1;
    let mut st = StateWord::InS;
    let done = match f {
        0x021B_AC04 => state_ac04(env, s_addr, s, &mut st),
        0x021B_ACAC => state_acac(env, s_addr, s, &mut st),
        0x021B_AE60 => state_ae60(env, s_addr, s, &mut st),
        0x021B_AEF8 => state_aef8(env, s_addr, s, &mut st),
        0x021B_B0E0 => state_b0e0(env, s_addr, s, &mut st),
        0x021B_B208 => state_b208(env, s_addr, s, &mut st),
        0x021B_B368 => state_b368(env, s_addr, s, &mut st),
        0x021B_B280 => pick_next_state(env, s),
        0x021B_B320 => state_b320(env, s, &mut st),
        0x021B_B3B0 => state_b3b0(env, s_addr, s, &mut st),
        0x021B_B3F8 => state_b3f8(env, s_addr, s, &mut st),
        0x021B_B43C => state_b43c(env, s_addr, s, &mut st),
        0x021B_B484 => state_b484(env, s_addr, s, &mut st),
        _ => env.callh(f, &mut [Arg::V(s_addr), Arg::V(s_addr + 4)], s),
    };
    if done != 0 {
        let b0 = rd32(&s[0xB0..]);
        broadcast(env, s, 9, b0, 8);
        s[0xED] = 2;
    }
    0
}
