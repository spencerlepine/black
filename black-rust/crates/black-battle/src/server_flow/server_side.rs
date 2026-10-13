//! Server-side helpers called from asm/battle_server.s (0x021BAA94-0x021BB968; B10e).
//!
//! The object these functions work on ("S") is the server's client-adapter state, not
//! [`super::SvFlowWork`]: `+0x14` holds four 12-byte client slots, `+0x68` a `BattleRecordBuilder`,
//! `+0xB4` four 3-word record areas and `+0xE4` their four counts. Every function takes `s_addr`,
//! the address `S` has in the original (it only ever reaches an unported callee, which logs it),
//! and `s`, the bytes of `S`.

use super::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
///
/// B10-spec2 moved the last nine in (the slot index of `aa94`/`aacc`/`ab10` is bounded to a client id
/// `0..=3`, the `021eeb64` value/out-pointer shapes are one `outn:` stub, `021cc668` is a `force` stub
/// because the Rust port of this area only addresses the flow work, see the spec header).
pub const PORTED: &[u32] = &[
    0x021B_B4E0,
    0x021B_B6A8,
    0x021B_B828,
    0x021B_B830,
    0x021B_B840,
    0x021B_B848,
    0x021B_B85C,
    0x021B_B878,
    0x021B_B890,
    0x021B_B8A8,
    0x021B_B8B0,
    0x021B_B964,
    0x021B_B920,
    0x021B_B930,
    0x021B_ABDC,
    0x021B_ABEC,
    0x021B_AE5C,
    0x021B_B56C,
    0x021B_B724,
    0x021B_B718,
    0x021B_B768,
    0x021B_B798,
    0x021B_B7D0,
    0x021B_B7FC,
    0x021B_AB40,
    0x021B_AB5C,
    0x021B_ABF4,
    0x021B_B914,
    0x021B_B8BC,
    0x021B_B8DC,
    0x021B_B5FC,
    0x021B_B650,
    0x021B_B628,
    0x021B_B570,
    0x021B_B040,
    0x021B_B0A0,
    0x021B_B280,
    0x021B_AC04,
    0x021B_AE60,
    0x021B_B320,
    0x021B_B3B0,
    0x021B_B3F8,
    0x021B_B43C,
    0x021B_B484,
    0x021B_AA94,
    0x021B_AACC,
    0x021B_AB10,
    0x021B_ACAC,
    0x021B_AEF8,
    0x021B_B208,
    0x021B_AB9C,
    0x021B_B0E0,
    0x021B_B368,
];

const OFS_SLOTS: usize = 0x14;
const SLOT_SIZE: usize = 12;
const OFS_BUILDER: usize = 0x68;
const OFS_RECORDS: usize = 0xB4;
const OFS_COUNTS: usize = 0xE4;
const CLIENTS: usize = 4;

/// C: `FUN_overlay_d_93__021bb828`: marks the slot unused (id byte `+0xB` = 0xFF).
pub fn slot_clear(slot: &mut [u8]) {
    slot[0xB] = 0xFF;
}

/// C: `FUN_overlay_d_93__021bb830`: the slot is used (id byte `+0xB` is not 0xFF).
pub fn slot_used(slot: &[u8]) -> bool {
    slot[0xB] != 0xFF
}

/// C: `FUN_overlay_d_93__021bb840`: slot = (id, handle, flag at `+0xA`).
pub fn slot_set(slot: &mut [u8], id: u8, handle: u32, flag: u8) {
    slot[0xB] = id;
    slot[..4].copy_from_slice(&handle.to_le_bytes());
    slot[0xA] = flag;
}

/// C: `FUN_overlay_d_93__021bb848`: `slot.u32[4] = val; slot[8] = f(val); slot[9] = v`
/// (`f` = `021b9b94`, not ported).
pub fn slot_set_data<E: FlowEnv>(env: &mut E, slot: &mut [u8], val: u32, v: u8) {
    slot[4..8].copy_from_slice(&val.to_le_bytes());
    slot[8] = env.call(0x021B_9B94, &mut [Arg::V(val)]) as u8;
    slot[9] = v;
}

/// Offset in `S` of client slot `i`.
fn slot_ofs(i: u32) -> usize {
    OFS_SLOTS + SLOT_SIZE * i as usize
}

/// C: `FUN_overlay_d_93__021bb85c`: offset of slot `idx` if `idx < 4` and it is used.
pub fn slot_checked(s: &[u8], idx: u32) -> Option<usize> {
    (idx < 4 && s[slot_ofs(idx) + 0xB] != 0xFF).then(|| slot_ofs(idx))
}

/// C: `FUN_overlay_d_93__021bb878`: offset of slot `idx` if it is used (no bound check).
pub fn slot_unchecked(s: &[u8], idx: i32) -> Option<usize> {
    let o = (OFS_SLOTS as i32).wrapping_add(idx.wrapping_mul(12)) as usize;
    (s[o + 0xB] != 0xFF).then_some(o)
}

/// C: `FUN_overlay_d_93__021bb890`: slot `idx` is used (false for `idx >= 4`).
pub fn slot_used_idx(s: &[u8], idx: u32) -> bool {
    idx < 4 && slot_used(&s[slot_ofs(idx)..])
}

/// C: `FUN_overlay_d_93__021bb8a8`: `S[0x30 + i]`.
pub fn byte_0x30(s: &[u8], i: i32) -> u8 {
    s[0x30usize.wrapping_add(i as usize)]
}

/// C: `FUN_overlay_d_93__021bb8b0`: word at `S + b * 12 + c * 4`.
pub fn word_at(s: &[u8], b: i32, c: i32) -> u32 {
    rd32(&s[(b.wrapping_mul(12).wrapping_add(c.wrapping_mul(4))) as usize..])
}

/// C: `FUN_overlay_d_93__021bb964`: `S[0x44]`.
pub fn ptr_0x44(s: &[u8]) -> u32 {
    rd32(&s[0x44..])
}

const OFS_LIST_COUNT: usize = 0xCBA;
const OFS_LIST: usize = 0xCB4;

/// C: `FUN_overlay_d_93__021bb920`: `S[0xCBA] = 0; S[0xCBF] = 0`.
pub fn list_clear(s: &mut [u8]) {
    s[OFS_LIST_COUNT] = 0;
    s[OFS_LIST_COUNT + 5] = 0;
}

/// C: `FUN_overlay_d_93__021bb930`: appends `val` to the u8 list at `S + 0xCB4` (count at
/// `S + 0xCBA`) unless it is already in it.
pub fn list_add(s: &mut [u8], val: u32) {
    let n = s[OFS_LIST_COUNT] as usize;
    for k in 0..n {
        if val == s[OFS_LIST + k] as u32 {
            return;
        }
    }
    s[OFS_LIST_COUNT] = s[OFS_LIST_COUNT].wrapping_add(1);
    s[OFS_LIST + n] = val as u8;
}

/// C: `FUN_overlay_d_93__021babdc`: copies the 2 words at `*(S + 0xB0)` (an address inside `S` in
/// the test environment) to `out`.
pub fn copy_pair(s: &[u8], s_addr: u32, out: &mut [u8]) {
    let src = rd32(&s[0xB0..]).wrapping_sub(s_addr) as usize;
    out[..8].copy_from_slice(&s[src..src + 8]);
}

/// C: `FUN_overlay_d_93__021babec`: sets the state function and clears the substate.
pub fn set_state(p: &mut [u8], func: u32) {
    p[..4].copy_from_slice(&func.to_le_bytes());
    p[4..8].copy_from_slice(&0u32.to_le_bytes());
}

/// C: `FUN_021bae5c`: returns 1.
pub fn always_true() -> u32 {
    1
}

/// C: `FUN_021bb56c`: empty.
pub fn nop() {}

/// Handle (first word) of client slot `i`.
fn slot_handle(s: &[u8], i: usize) -> u32 {
    rd32(&s[OFS_SLOTS + SLOT_SIZE * i..])
}

/// C: `FUN_overlay_d_93__021bb724`: bracketed by `021eeae8`/`021eeafc`, sends `(a, b, c)` to every
/// used client slot's handle (`021eeb00`).
pub fn broadcast<E: FlowEnv>(env: &mut E, s: &mut [u8], a: u32, b: u32, c: u32) {
    env.call(0x021E_EAE8, &mut []);
    for i in 0..CLIENTS {
        if slot_used(&s[OFS_SLOTS + SLOT_SIZE * i..]) {
            let h = slot_handle(s, i);
            // `b` may point into S (the record list, the builder)
            env.callh(
                0x021E_EB00,
                &mut [Arg::V(h), Arg::V(a), Arg::V(b), Arg::V(c)],
                s,
            );
        }
    }
    env.call(0x021E_EAFC, &mut []);
}

/// C: `FUN_overlay_d_93__021bb718`: `broadcast(S, code, 0, 0)`.
pub fn broadcast_code<E: FlowEnv>(env: &mut E, s: &mut [u8], code: u32) {
    broadcast(env, s, code, 0, 0);
}

/// C: `FUN_overlay_d_93__021bb768`: like [`broadcast`] but to one slot, preceded by `021eeb70`;
/// the fourth value is the stack argument `count`.
pub fn send_one<E: FlowEnv>(env: &mut E, s: &mut [u8], code: u32, idx: u32, data: u32, count: u32) {
    let o = slot_ofs(idx);
    if slot_used(&s[o..]) {
        let h = rd32(&s[o..]);
        env.call(0x021E_EB70, &mut [Arg::V(h)]);
        env.callh(
            0x021E_EB00,
            &mut [Arg::V(h), Arg::V(code), Arg::V(data), Arg::V(count)],
            s,
        );
    }
}

/// C: `FUN_overlay_d_93__021bb798`: every used slot's `021eeb2c` answers nonzero.
pub fn all_ready<E: FlowEnv>(env: &mut E, s: &[u8]) -> bool {
    let mut r = true;
    for i in 0..CLIENTS {
        if slot_used(&s[OFS_SLOTS + SLOT_SIZE * i..])
            && env.call(0x021E_EB2C, &mut [Arg::V(slot_handle(s, i))]) == 0
        {
            r = false;
        }
    }
    r
}

/// C: `FUN_overlay_d_93__021bb7d0`: `021eeb70(handle)` for every used slot.
pub fn each_eeb70<E: FlowEnv>(env: &mut E, s: &[u8]) {
    for i in 0..CLIENTS {
        if slot_used(&s[OFS_SLOTS + SLOT_SIZE * i..]) {
            env.call(0x021E_EB70, &mut [Arg::V(slot_handle(s, i))]);
        }
    }
}

/// C: `FUN_overlay_d_93__021bb7fc`: `021eead0(handle)` for every used slot.
pub fn each_eead0<E: FlowEnv>(env: &mut E, s: &[u8]) {
    for i in 0..CLIENTS {
        if slot_used(&s[OFS_SLOTS + SLOT_SIZE * i..]) {
            env.call(0x021E_EAD0, &mut [Arg::V(slot_handle(s, i))]);
        }
    }
}

/// `021b9aec(S[0x10], idx)`, not ported.
fn data_for<E: FlowEnv>(env: &mut E, s: &[u8], idx: u32) -> u32 {
    env.call(0x021B_9AEC, &mut [Arg::V(rd32(&s[0x10..])), Arg::V(idx)])
}

/// C: `FUN_overlay_d_93__021baa94`: slot(idx) = (idx, `handle`, 1), slot data = (`9aec(S[0x10], idx)`, `v`).
pub fn setup_slot_handle<E: FlowEnv>(env: &mut E, s: &mut [u8], handle: u32, idx: u32, v: u32) {
    let d = data_for(env, s, idx);
    let o = slot_ofs(idx);
    slot_set(&mut s[o..], idx as u8, handle, 1);
    slot_set_data(env, &mut s[o..], d, v as u8);
}

/// C: `FUN_overlay_d_93__021baacc`: like [`setup_slot_handle`] with the handle from
/// `021eea58(r2, idx, 1, S.u16[0xCC0])` and flag 0; `v` is the stack argument.
pub fn setup_slot_created<E: FlowEnv>(env: &mut E, s: &mut [u8], r2: u32, idx: u32, v: u32) {
    let d = data_for(env, s, idx);
    let cc0 = u16::from_le_bytes([s[0xCC0], s[0xCC1]]) as u32;
    let h = env.callh(
        0x021E_EA58,
        &mut [Arg::V(r2), Arg::V(idx), Arg::V(1), Arg::V(cc0)],
        s,
    );
    let o = slot_ofs(idx);
    slot_set(&mut s[o..], idx as u8, h, 0);
    slot_set_data(env, &mut s[o..], d, v as u8);
}

/// C: `FUN_overlay_d_93__021bab10`: slot(idx) = (idx, 0, 0), slot data = (`9aec(S[0x10], idx)`, `v`).
pub fn setup_slot_empty<E: FlowEnv>(env: &mut E, s: &mut [u8], idx: u32, v: u32) {
    let d = data_for(env, s, idx);
    let o = slot_ofs(idx);
    slot_set(&mut s[o..], idx as u8, 0, 0);
    slot_set_data(env, &mut s[o..], d, v as u8);
}

/// State function pointer written by [`teardown`] (Thumb bit set).
const STATE_AC05: u32 = 0x021B_AC05;

/// C: `FUN_overlay_d_93__021bab40`: state = `021BAC05`, then `each_eead0` and `021bb9d8(S[0x44])`
/// (the thunk target of `021bb9d0`) free everything.
pub fn teardown<E: FlowEnv>(env: &mut E, s: &mut [u8]) {
    set_state(s, STATE_AC05);
    each_eead0(env, s);
    env.callh(0x021B_B9D8, &mut [Arg::V(ptr_0x44(s))], s);
}

/// C: `FUN_overlay_d_93__021bab5c`: `02045808(S[0xAC])`, `021bbb48(S[0x44])`, frees (`021eeae0`, a
/// thunk to `020307b0`) every used slot handle that is nonzero with `slot[0xA] == 0`, then
/// `020307b0(S)`.
pub fn destroy<E: FlowEnv>(env: &mut E, s_addr: u32, s: &[u8]) {
    env.call(0x0204_5808, &mut [Arg::V(rd32(&s[0xAC..]))]);
    env.call(0x021B_BB48, &mut [Arg::V(ptr_0x44(s))]);
    for i in 0..CLIENTS {
        let o = OFS_SLOTS + SLOT_SIZE * i;
        if slot_used(&s[o..]) && rd32(&s[o..]) != 0 && s[o + 0xA] == 0 {
            env.call(0x0203_07B0, &mut [Arg::V(rd32(&s[o..]))]);
        }
    }
    // S frees itself: lent read-only so the environment can release it (B16)
    env.callr(0x0203_07B0, &mut [Arg::V(s_addr)], s);
}

/// `S` field helper: the address stored at `S + ofs` as an offset into `S` (the data a pointer
/// field addresses lives in `S` itself in the test environment).
fn inner(s: &[u8], s_addr: u32, ofs: usize) -> usize {
    rd32(&s[ofs..]).wrapping_sub(s_addr) as usize
}

/// Next-state function pointers (Thumb bit set) that `bb280` picks.
const STATE_BB369: u32 = 0x021B_B369;
const STATE_BB3B1: u32 = 0x021B_B3B1;
const STATE_BB43D: u32 = 0x021B_B43D;
const STATE_BB485: u32 = 0x021B_B485;
const STATE_BB3F9: u32 = 0x021B_B3F9;
const STATE_BB321: u32 = 0x021B_B321;
const STATE_ACAD: u32 = 0x021B_ACAD;

/// C: `FUN_overlay_d_93__021babf4`: `set_state(S, 0x021BACAD)` (the original is a tail call
/// through the thunk at `021babec`).
pub fn set_state_acad(p: &mut [u8]) {
    set_state(p, STATE_ACAD);
}

/// C: `FUN_overlay_d_93__021bb914`: tail call `021b9234(S[0xC])`.
pub fn call_9234<E: FlowEnv>(env: &mut E, s: &[u8]) {
    env.call(0x021B_9234, &mut [Arg::V(rd32(&s[0xC..]))]);
}

/// C: `FUN_overlay_d_93__021bb8bc`: `02017e1c(pokemon(bpp), 0x73, S[0xAC])`, `02027268(S[0xAC])`.
pub fn pokemon_set_73<E: FlowEnv>(env: &mut E, s: &[u8], bpp: u32) {
    let pk = env.call(0x021D_5844, &mut [Arg::V(bpp)]);
    let ac = rd32(&s[0xAC..]);
    env.call(0x0201_7E1C, &mut [Arg::V(pk), Arg::V(0x73), Arg::V(ac)]);
    env.call(0x0202_7268, &mut [Arg::V(ac)]);
}

/// C: `FUN_overlay_d_93__021bb8dc`: `020271dc(u16(02017e1c(pokemon(bpp), 5, 0)))`, then
/// `021ba1b4(S[0xC], 7)` and `021ba1b4(S[0xC], 0x54)`.
pub fn pokemon_set_5<E: FlowEnv>(env: &mut E, s: &[u8], bpp: u32) {
    let pk = env.call(0x021D_5844, &mut [Arg::V(bpp)]);
    let v = env.call(0x0201_7E1C, &mut [Arg::V(pk), Arg::V(5), Arg::V(0)]);
    env.call(0x0202_71DC, &mut [Arg::V(v & 0xFFFF)]);
    let m = rd32(&s[0xC..]);
    env.call(0x021B_A1B4, &mut [Arg::V(m), Arg::V(7)]);
    env.call(0x021B_A1B4, &mut [Arg::V(m), Arg::V(0x54)]);
}

/// C: `FUN_overlay_d_93__021bb5fc`: initialises the builder at `S + 0x68` (tag 1), loads `tag`
/// (`021ee8d8`), broadcasts (10, result, loaded word) and returns 1.
pub fn builder_load<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8], tag: u32) -> u32 {
    let b = s_addr + OFS_BUILDER as u32;
    env.callh(0x021E_E8B4, &mut [Arg::V(b), Arg::V(1)], s);
    let mut local = [0u8; 4];
    let r = env.callh(
        0x021E_E8D8,
        &mut [Arg::V(b), Arg::Out(&mut local), Arg::V(tag)],
        s,
    );
    broadcast(env, s, 10, r, u32::from_le_bytes(local));
    1
}

/// Where `Finish` writes its result word: a caller-supplied address, or a local of the caller.
pub enum FinishOut<'a> {
    /// A pointer the caller received (only passed on).
    Ptr(u32),
    /// The caller's own local.
    Local(&'a mut [u8]),
}

/// C: `FUN_overlay_d_93__021bb650`: builder init (`mode`), `AddClientEntries` for every used slot
/// (the slot's record words, `(size & 0x3ff) >> 2` of them), then `Finish(code, out)`.
pub fn builder_finish<E: FlowEnv>(
    env: &mut E,
    s_addr: u32,
    s: &mut [u8],
    code: u32,
    mode: u32,
    out: FinishOut<'_>,
) -> u32 {
    let b = s_addr + OFS_BUILDER as u32;
    env.callh(0x021E_E8B4, &mut [Arg::V(b), Arg::V(mode)], s);
    for i in 0..CLIENTS {
        if s[OFS_SLOTS + SLOT_SIZE * i + 0xB] != 0xFF {
            let mut size = [0u8; 4];
            let entries = env.call(
                0x021E_EB64,
                &mut [Arg::V(slot_handle(s, i)), Arg::Out(&mut size)],
            );
            let n = (u32::from_le_bytes(size) & 0x3FF) >> 2;
            env.callh(
                0x021E_E8F4,
                &mut [Arg::V(b), Arg::V(i as u32), Arg::V(entries), Arg::V(n)],
                s,
            );
        }
    }
    match out {
        FinishOut::Ptr(p) => env.callh(0x021E_E96C, &mut [Arg::V(b), Arg::V(code), Arg::V(p)], s),
        FinishOut::Local(l) => {
            env.callh(0x021E_E96C, &mut [Arg::V(b), Arg::V(code), Arg::Out(l)], s)
        }
    }
}

/// C: `FUN_overlay_d_93__021bb628`: [`builder_finish`] into a local; when it answers nonzero,
/// broadcasts (10, answer, local) and returns true.
pub fn builder_flush<E: FlowEnv>(
    env: &mut E,
    s_addr: u32,
    s: &mut [u8],
    code: u32,
    mode: u32,
) -> bool {
    let mut local = [0u8; 4];
    let r = builder_finish(env, s_addr, s, code, mode, FinishOut::Local(&mut local));
    if r == 0 {
        return false;
    }
    broadcast(env, s, 10, r, u32::from_le_bytes(local));
    true
}

/// C: `FUN_overlay_d_93__021bb570`: clears the 2-word header at `*(S + 0xCB0)`, lets the
/// by-`mode` fetcher fill it, then compares `size` with the header and the data with `p3`
/// (`0203efc8`); false only when both agree.
pub fn compare_header<E: FlowEnv>(
    env: &mut E,
    s_addr: u32,
    s: &mut [u8],
    mode: u32,
    p3: u32,
    size: u32,
) -> bool {
    let h = inner(s, s_addr, 0xCB0);
    s[h..h + 8].fill(0);
    let o44 = ptr_0x44(s);
    let a = s_addr + 0xB4;
    match mode {
        1 => {
            env.callh(0x021B_BC34, &mut [Arg::V(o44)], s);
            env.callh(0x021B_BC44, &mut [Arg::V(o44), Arg::V(a)], s);
        }
        2 => {
            env.callh(0x021B_BD80, &mut [Arg::V(o44)], s);
            env.callh(0x021B_BD8C, &mut [Arg::V(o44), Arg::V(a)], s);
        }
        3 => {
            env.callh(0x021B_BD20, &mut [Arg::V(o44)], s);
            env.callh(0x021B_BD2C, &mut [Arg::V(o44), Arg::V(a)], s);
        }
        4 => {
            env.callh(0x021B_BB60, &mut [Arg::V(o44)], s);
        }
        _ => {}
    }
    let h = inner(s, s_addr, 0xCB0);
    if rd32(&s[h..]) == size {
        let hp = rd32(&s[0xCB0..]) + 8;
        if env.callh(0x0203_EFC8, &mut [Arg::V(hp), Arg::V(p3), Arg::V(size)], s) == 0 {
            return false;
        }
    }
    true
}

/// C: `FUN_overlay_d_93__021bb040`: the main object (`021b85a8`) is set up, the party list at
/// `S + 0xCB4` is nonempty and every member passes `021b8cfc`, and `021b9b98(...) >= 2`.
pub fn party_ready<E: FlowEnv>(env: &mut E, s: &[u8]) -> bool {
    let main = rd32(&s[0xC..]);
    if env.call(0x021B_85A8, &mut [Arg::V(main)]) == 0 {
        return false;
    }
    if s[OFS_LIST_COUNT] == 0 {
        return false;
    }
    let mut k = 0;
    while k < s[OFS_LIST_COUNT] as usize {
        let id = s[OFS_LIST + k] as u32;
        if env.call(0x021B_8CFC, &mut [Arg::V(main), Arg::V(id)]) == 0 {
            return false;
        }
        k += 1;
    }
    let x = env.call(0x021B_8FF4, &mut [Arg::V(main)]);
    let r = env.call(0x021B_9AEC, &mut [Arg::V(rd32(&s[0x10..])), Arg::V(x)]);
    env.call(0x021B_9B98, &mut [Arg::V(r)]) >= 2
}

/// C: `FUN_overlay_d_93__021bb0a0`: if slot 1 (`S + 0x20`) is used and `021d8454(record)` is 3,
/// `021b9920(S[0x10], 1, (record word >> 7) & 7)` and `021d5440` of it; 0x1f otherwise.
pub fn slot1_value<E: FlowEnv>(env: &mut E, s_addr: u32, s: &[u8]) -> u32 {
    if slot_used(&s[0x20..]) {
        let rec = env.call(0x021E_EB64, &mut [Arg::V(rd32(&s[0x20..])), Arg::V(0)]);
        if env.call(0x021D_8454, &mut [Arg::V(rec)]) == 3 {
            let w = rd32(&s[rec.wrapping_sub(s_addr) as usize..]);
            let r = env.call(
                0x021B_9920,
                &mut [Arg::V(rd32(&s[0x10..])), Arg::V(1), Arg::V((w >> 7) & 7)],
            );
            return env.call(0x021D_5440, &mut [Arg::V(r)]);
        }
    }
    0x1F
}

/// C: `sub_021bb280`: `S.u16[0xAA] = 021ba54c(S[0xC])` (a thunk to `021b9...`), `S.u16[0xA8] =
/// 021b8ff4(S[0xC])`, then the next state function from `021b86b8(S[0xC])` and the first value.
pub fn pick_next_state<E: FlowEnv>(env: &mut E, s: &mut [u8]) -> u32 {
    let main = rd32(&s[0xC..]);
    let a = env.callh(0x021B_A54C, &mut [Arg::V(main)], s);
    s[0xAA..0xAC].copy_from_slice(&(a as u16).to_le_bytes());
    let b = env.callh(0x021B_8FF4, &mut [Arg::V(main)], s);
    s[0xA8..0xAA].copy_from_slice(&(b as u16).to_le_bytes());
    let mode = env.callh(0x021B_86B8, &mut [Arg::V(main)], s);
    let next = match mode {
        0 => {
            if a == 1 {
                STATE_BB369
            } else if a == 0 || a == 2 {
                STATE_BB3B1
            } else {
                STATE_BB321
            }
        }
        1 => STATE_BB43D,
        2 => STATE_BB485,
        3 => STATE_BB3F9,
        _ => {
            if a == 1 {
                let x = env.callh(0x021B_86DC, &mut [Arg::V(main)], s);
                env.callh(0x0200_5D90, &mut [Arg::V(x), Arg::V(0xFFFF)], s);
            }
            STATE_BB321
        }
    };
    set_state(s, next);
    0
}

fn rd32(b: &[u8]) -> u32 {
    u32::from_le_bytes(b[..4].try_into().unwrap())
}

/// C: `FUN_overlay_d_93__021bb6a8`: for every used client slot, clears its record count and
/// copies the record words of the slot's handle (`size >> 2` of them, the handle lookup
/// `021eeb64` returns the data address and writes the size) to `S + 0xB4 + i * 12`, counting
/// them in `S[0xE4 + i]`.
pub fn collect_slot_records<E: FlowEnv>(env: &mut E, s_addr: u32, s: &mut [u8]) {
    for i in 0..CLIENTS {
        s[OFS_COUNTS + i] = 0;
        let slot = OFS_SLOTS + SLOT_SIZE * i;
        if !slot_used(&s[slot..]) {
            continue;
        }
        let handle = rd32(&s[slot..]);
        let mut size = [0u8; 4];
        let data = env.callh(0x021E_EB64, &mut [Arg::V(handle), Arg::Out(&mut size)], s);
        let words = u32::from_le_bytes(size) >> 2;
        // the data address lies inside `S` in the test environment; in the game it is the
        // channel object's reply block (B16: read through `peek`)
        let mut ext = vec![0u8; 4 * words as usize];
        let outside = env.peek(data, &mut ext);
        let src = data.wrapping_sub(s_addr) as usize;
        for k in 0..words as usize {
            let w = if outside {
                rd32(&ext[4 * k..])
            } else {
                rd32(&s[src + 4 * k..])
            };
            s[OFS_RECORDS + SLOT_SIZE * i + 4 * k..][..4].copy_from_slice(&w.to_le_bytes());
            s[OFS_COUNTS + i] = s[OFS_COUNTS + i].wrapping_add(1);
        }
    }
}

/// C: `FUN_021bb4e0`: loads a recorded stream (`data`, `size`) into the builder at `S + 0x68`
/// and unpacks it: every entry `ReadNext` yields (client id, up to 3 words) goes to
/// `S + 0xB4 + id * 12` and its word count to `S[0xE4 + id]`.
pub fn load_record_stream<E: FlowEnv>(
    env: &mut E,
    s_addr: u32,
    s: &mut [u8],
    data: u32,
    size: u32,
) {
    for i in 0..CLIENTS {
        s[OFS_COUNTS + i] = 0;
    }
    let builder = s_addr + OFS_BUILDER as u32;
    env.callh(
        0x021E_E9B4,
        &mut [Arg::V(builder), Arg::V(data), Arg::V(size)],
        s,
    );
    loop {
        let mut read_pos = [0u8; 4];
        let mut client = [0u8; 1];
        let mut count = [0u8; 1];
        let mut entries = [0u8; 12];
        // `BattleRecordBuilder_ReadNext(builder, &rp, &client, &count, entries)`
        let more = env.callh(
            0x021E_E9E4,
            &mut [
                Arg::V(builder),
                Arg::Out(&mut read_pos),
                Arg::Out(&mut client),
                Arg::Out(&mut count),
                Arg::Out(&mut entries),
            ],
            s,
        );
        if more == 0 {
            break;
        }
        let id = client[0] as usize;
        for k in 0..count[0] as usize {
            s[OFS_RECORDS + SLOT_SIZE * id + 4 * k..][..4]
                .copy_from_slice(&entries[4 * k..4 * k + 4]);
        }
        s[OFS_COUNTS + id] = count[0];
    }
}

mod states;
pub use states::*;

#[cfg(test)]
mod tests;
