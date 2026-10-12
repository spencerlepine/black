//! The battle **message printer** half of `btl_adapter.c` (0x021EEC78-0x021EF71C).
//!
//! UI-side code (callers in 0x021E9034-0x021EE200): a context at `0x021F6398` (`ctx`, 0x60 bytes in the
//! specs), the word-set helpers `eed00..eed90`, the message-id dispatch `eeef4` with its handlers, and the tag
//! expander `eefe0`. Everything it calls outside the file (message bank, word-set and window routines
//! `0201xxxx`/`0204xxxx`, the battle-state queries `021b8xxx`/`021b9xxx`/`021baxxx`) goes through [`FlowEnv`].
//!
//! # Context layout (`ctx`)
//!
//! | ofs | size | meaning |
//! |---|---|---|
//! | `+0x00` | u32 | system / battle object (first argument of the battle-state queries) |
//! | `+0x04` | u32 | pokecon (argument of `021b9940`) |
//! | `+0x08` | u32 | word-set handle (`0201EC64`) |
//! | `+0x0C` | u32 | message buffer handle (`020457B0`) |
//! | `+0x10..+0x30` | u32 x 8 | message-bank handles (`02045B38`, bank ids at `0x021F3990`) |
//! | `+0x30..+0x50` | u32 x 8 | message arguments (written by [`msg_printf`]) |
//! | `+0x50` | u16 | init argument 3 (heap id) |
//! | `+0x54` | u32 | side / mon id (init argument 1) |
//! | `+0x58` | u16 | "sub-message" flag from `02013158`; nonzero disables the side-variant lookups |
//! | `+0x5A` | u16 | set to 1 by [`msg_flagged`] while it expands a message |
//!
//! Strings are read through [`Msg`] (a base address and the bytes behind it): the original walks halfwords
//! at addresses that the message-buffer routines (`02045A90`, `0201D688`, ...) return, and the port turns an
//! address back into an offset with the fixed base.

use super::{rd16, rd32, wr16, wr32};
use crate::server_flow::env::{Arg, FlowEnv};

/// Bytes of the context at `0x021F6398` that the specs model.
pub const CTX_SIZE: usize = 0x60;

const OFS_SYS: usize = 0x00;
const OFS_POKECON: usize = 0x04;
const OFS_WORDSET: usize = 0x08;
const OFS_MSGBUF: usize = 0x0C;
const OFS_BANKS: usize = 0x10;
const OFS_ARGS: usize = 0x30;
const OFS_PARAM3: usize = 0x50;
const OFS_SIDE: usize = 0x54;
const OFS_SUB: usize = 0x58;
const OFS_FLAG: usize = 0x5A;

/// Message bank ids at `0x021F3990` (8 halfwords), one handle per id in `ctx + 0x10`.
const BANK_IDS: [u16; 8] = [0x0F, 0x0E, 0x0D, 0x10, 0xB6, 0xCC, 0xE9, 0xB8];

/// Message-id remap table at `0x021F39A0` (27 halfwords): the ids for which [`msg_remap`] picks id or id + 1.
const REMAP_IDS: [u16; 27] = [
    0x94, 0x96, 0x98, 0x9A, 0x9C, 0x9E, 0xA0, 0xA2, 0xB0, 0x84, 0x86, 0x80, 0x82, 0x7C, 0x7E, 0x88,
    0x8A, 0x8C, 0x8E, 0x90, 0x92, 0xA4, 0xA6, 0xA8, 0xAA, 0xAC, 0xAE,
];

/// Handlers of the dispatch table at `0x021F39D8` (14 entries of `{u16 id, u16 pad, u32 handler}`).
#[derive(Clone, Copy)]
enum Handler {
    Ef460,
    Ef508,
    Ef3a4,
    Ef594,
    Ef204,
}

const DISPATCH: [(u16, Handler); 14] = [
    (0x1B, Handler::Ef460),
    (0x5A, Handler::Ef460),
    (0x99, Handler::Ef508),
    (0xAE, Handler::Ef508),
    (0x3AA, Handler::Ef3a4),
    (0x21, Handler::Ef594),
    (0x414, Handler::Ef204),
    (0x417, Handler::Ef204),
    (0x14D, Handler::Ef204),
    (0x471, Handler::Ef204),
    (9, Handler::Ef204),
    (0xC, Handler::Ef204),
    (0x12, Handler::Ef204),
    (0x15, Handler::Ef204),
];

/// A message string: the halfwords at `base..base + bytes.len()`.
pub struct Msg<'a> {
    pub base: u32,
    pub bytes: &'a [u8],
}

impl Msg<'_> {
    fn h(&self, addr: u32) -> u32 {
        rd16(self.bytes, addr.wrapping_sub(self.base) as usize) as u32
    }
}

fn word(args: &[u8], i: u32) -> u32 {
    rd32(args, 4 * i as usize)
}

fn call<E: FlowEnv>(env: &mut E, addr: u32, args: &mut [Arg<'_>]) -> u32 {
    env.call(addr, args)
}

/// Set the message (`02045CAC(bank, id, buffer)`).
fn set_message<E: FlowEnv>(env: &mut E, bank: u32, id: u32, buf: u32) {
    call(
        env,
        0x0204_5CAC,
        &mut [Arg::V(bank), Arg::V(id), Arg::V(buf)],
    );
}

/// Show the message (`0201F250(wordset, a, buffer)`).
fn show<E: FlowEnv>(env: &mut E, ctx: &[u8], a: u32) {
    call(
        env,
        0x0201_F250,
        &mut [
            Arg::V(rd32(ctx, OFS_WORDSET)),
            Arg::V(a),
            Arg::V(rd32(ctx, OFS_MSGBUF)),
        ],
    );
}

/// The side variant (0, 1 or 2) of the mon `val` for the current side: 0 when the sub-message flag is set
/// or `021b8ce8` says no; else 1 when `021b86b8(sys)` is 0 or 4, else 2. (The original asks `021b86b8`
/// twice; both calls are made.) The block repeated in `ef204`/`ef280`/`ef3a4`/`ef460`/`ef508`/`ef5e8`.
fn side_variant<E: FlowEnv>(env: &mut E, ctx: &[u8], val: u32) -> u32 {
    if rd16(ctx, OFS_SUB) != 0 {
        return 0;
    }
    let sys = rd32(ctx, OFS_SYS);
    let r2 = call(env, 0x021B_8D08, &mut [Arg::V(val)]);
    let ok = call(
        env,
        0x021B_8CE8,
        &mut [Arg::V(sys), Arg::V(rd32(ctx, OFS_SIDE) & 0xFF), Arg::V(r2)],
    );
    if ok == 0 {
        return 0;
    }
    if call(env, 0x021B_86B8, &mut [Arg::V(sys)]) == 0 {
        return 1;
    }
    if call(env, 0x021B_86B8, &mut [Arg::V(sys)]) == 4 {
        1
    } else {
        2
    }
}

/// C: `FUN_overlay_d_93__021eec78`: initialise the context: `ctx[0] = sys`, `ctx[4] = r2`, `ctx+0x50 = r3`,
/// `ctx+0x54 = r1`, the sub-message flag from `02013158(*sys, 0x200)` (through the thunk `021ba1a8`; `sys_word0`
/// is `*sys`), `ctx+0x5A = 0`, the word-set handle (`0201EC64`), the message buffer (`020457B0(0x100, r3)`) and
/// the eight message-bank handles (`02045B38(0, 2, bank id, r3)`).
pub fn init<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    sys: u32,
    sys_word0: u32,
    r1: u32,
    r2: u32,
    r3: u32,
) {
    wr32(ctx, OFS_SYS, sys);
    wr32(ctx, OFS_POKECON, r2);
    wr16(ctx, OFS_PARAM3, r3 as u16);
    wr32(ctx, OFS_SIDE, r1);
    let sub = call(env, 0x0201_3158, &mut [Arg::V(sys_word0), Arg::V(0x200)]);
    wr16(ctx, OFS_SUB, sub as u16);
    wr16(ctx, OFS_FLAG, 0);
    let ws = call(env, 0x0201_EC64, &mut [Arg::V(r3)]);
    wr32(ctx, OFS_WORDSET, ws);
    let buf = call(env, 0x0204_57B0, &mut [Arg::V(0x100), Arg::V(r3)]);
    wr32(ctx, OFS_MSGBUF, buf);
    for (i, id) in BANK_IDS.iter().enumerate() {
        let h = call(
            env,
            0x0204_5B38,
            &mut [Arg::V(0), Arg::V(2), Arg::V(*id as u32), Arg::V(r3)],
        );
        wr32(ctx, OFS_BANKS + 4 * i, h);
    }
}

/// C: `FUN_overlay_d_93__021eecd8`: free the word-set handle, the message buffer and the eight bank handles.
pub fn free<E: FlowEnv>(env: &mut E, ctx: &[u8]) {
    call(env, 0x0201_ED04, &mut [Arg::V(rd32(ctx, OFS_WORDSET))]);
    call(env, 0x0204_5808, &mut [Arg::V(rd32(ctx, OFS_MSGBUF))]);
    for i in 0..8 {
        call(
            env,
            0x0204_5C04,
            &mut [Arg::V(rd32(ctx, OFS_BANKS + 4 * i))],
        );
    }
}

/// C: `FUN_overlay_d_93__021eed00`: word-set slot `slot`: the visible Pokemon of mon `mon`
/// (`0201EEF4(wordset, slot, GetVisibleMon(021b9940(pokecon, mon)))`).
pub fn word_visible_mon<E: FlowEnv>(env: &mut E, ctx: &[u8], mon: u32, slot: u32) {
    let bpp = call(
        env,
        0x021B_9940,
        &mut [Arg::V(rd32(ctx, OFS_POKECON)), Arg::V(mon)],
    );
    let p = call(env, 0x021D_5864, &mut [Arg::V(bpp)]);
    call(
        env,
        0x0201_EEF4,
        &mut [Arg::V(rd32(ctx, OFS_WORDSET)), Arg::V(slot), Arg::V(p)],
    );
}

/// C: `FUN_overlay_d_93__021eed24`: like [`word_visible_mon`] with `BattleMon_GetPokemon` (`021d5844`).
pub fn word_pokemon<E: FlowEnv>(env: &mut E, ctx: &[u8], mon: u32, slot: u32) {
    let bpp = call(
        env,
        0x021B_9940,
        &mut [Arg::V(rd32(ctx, OFS_POKECON)), Arg::V(mon)],
    );
    let p = call(env, 0x021D_5844, &mut [Arg::V(bpp)]);
    call(
        env,
        0x0201_EEF4,
        &mut [Arg::V(rd32(ctx, OFS_WORDSET)), Arg::V(slot), Arg::V(p)],
    );
}

/// C: `FUN_overlay_d_93__021eed48`: like [`word_visible_mon`] with `0201EE84`.
pub fn word_visible_mon2<E: FlowEnv>(env: &mut E, ctx: &[u8], mon: u32, slot: u32) {
    let bpp = call(
        env,
        0x021B_9940,
        &mut [Arg::V(rd32(ctx, OFS_POKECON)), Arg::V(mon)],
    );
    let p = call(env, 0x021D_5864, &mut [Arg::V(bpp)]);
    call(
        env,
        0x0201_EE84,
        &mut [Arg::V(rd32(ctx, OFS_WORDSET)), Arg::V(slot), Arg::V(p)],
    );
}

/// C: `FUN_overlay_d_93__021eed6c`: word-set from a value: `0201F0F8(a, b, 021ba020(sys, c) & 0xFF)`.
pub fn word_from_value<E: FlowEnv>(env: &mut E, ctx: &[u8], a: u32, b: u32, c: u32) {
    let v = call(
        env,
        0x021B_A020,
        &mut [Arg::V(rd32(ctx, OFS_SYS)), Arg::V(c)],
    );
    call(
        env,
        0x0201_F0F8,
        &mut [Arg::V(a), Arg::V(b), Arg::V(v & 0xFF)],
    );
}

/// C: `FUN_overlay_d_93__021eed90`: word-set by kind `kind` for slot (`a`, `b`): kind 4 -> `021ba080`
/// -> `0201F02C`; else when `021b9f70(sys, kind)` is 0 -> `021ba064` -> `0201F02C`; else with
/// `021b86b8(sys) == 1`: `021ba004(sys, kind)` nonzero -> `0201F134`; otherwise `021b9fac` (also
/// writing a value that `0202A3CC` maps) -> `0201EE0C(a, b, x, y, 1, 2)`.
pub fn word_by_kind<E: FlowEnv>(env: &mut E, ctx: &[u8], a: u32, b: u32, kind: u32) {
    let sys = rd32(ctx, OFS_SYS);
    if kind == 4 {
        let v = call(env, 0x021B_A080, &mut [Arg::V(sys)]);
        call(env, 0x0201_F02C, &mut [Arg::V(a), Arg::V(b), Arg::V(v)]);
        return;
    }
    if call(env, 0x021B_9F70, &mut [Arg::V(sys), Arg::V(kind)]) == 0 {
        let v = call(env, 0x021B_A064, &mut [Arg::V(sys), Arg::V(kind)]);
        call(env, 0x0201_F02C, &mut [Arg::V(a), Arg::V(b), Arg::V(v)]);
        return;
    }
    if call(env, 0x021B_86B8, &mut [Arg::V(sys)]) == 1 {
        let v = call(env, 0x021B_A004, &mut [Arg::V(sys), Arg::V(kind)]);
        if v != 0 {
            call(env, 0x0201_F134, &mut [Arg::V(a), Arg::V(b), Arg::V(v)]);
        }
        return;
    }
    let mut out = [0u8; 4];
    let x = call(
        env,
        0x021B_9FAC,
        &mut [Arg::V(sys), Arg::V(kind), Arg::Out(&mut out)],
    );
    let y = call(env, 0x0202_A3CC, &mut [Arg::V(u32::from_le_bytes(out))]);
    call(
        env,
        0x0201_EE0C,
        &mut [
            Arg::V(a),
            Arg::V(b),
            Arg::V(x),
            Arg::V(y),
            Arg::V(1),
            Arg::V(2),
        ],
    );
}

/// C: `FUN_overlay_d_93__021eee24` (varargs, `push {r0-r3}`): copy the `count` words `vals` to `ctx + 0x30`
/// (zero the rest of the 8), then [`msg_remap`]`(a, id, ctx + 0x30)`. `count` > 8 would overflow the
/// context in the original; the callers pass at most 8.
#[allow(clippy::too_many_arguments)]
pub fn msg_printf<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    a: u32,
    id: u32,
    count: u32,
    vals: &[u32],
    msg: &Msg,
) {
    let mut i = 0usize;
    while (i as u32) < count {
        wr32(ctx, OFS_ARGS + 4 * i, vals[i]);
        i += 1;
    }
    while i < 8 {
        wr32(ctx, OFS_ARGS + 4 * i, 0);
        i += 1;
    }
    let args: [u8; 32] = ctx[OFS_ARGS..OFS_ARGS + 32].try_into().unwrap();
    msg_remap(env, ctx, a, id, &args, msg);
}

/// C: `FUN_overlay_d_93__021eee80`: when `id` is in the 27-entry remap table, add 1 (16-bit) unless
/// `021b8bd0(sys, args[0] & 0xFF)` says so; then [`msg_show`].
pub fn msg_remap<E: FlowEnv>(env: &mut E, ctx: &mut [u8], a: u32, id: u32, args: &[u8], msg: &Msg) {
    let mut id = id;
    if REMAP_IDS.iter().any(|&r| r as u32 == id) {
        let ok = call(
            env,
            0x021B_8BD0,
            &mut [Arg::V(rd32(ctx, OFS_SYS)), Arg::V(word(args, 0) & 0xFF)],
        );
        if ok == 0 {
            id = (id + 1) & 0xFFFF;
        }
    }
    msg_show(env, ctx, a, id, args, msg);
}

/// C: `FUN_overlay_d_93__021eeec8`: set message `id` in bank `ctx[0x10]`, expand its tags, show it.
pub fn msg_show<E: FlowEnv>(env: &mut E, ctx: &mut [u8], a: u32, id: u32, args: &[u8], msg: &Msg) {
    set_message(env, rd32(ctx, OFS_BANKS), id, rd32(ctx, OFS_MSGBUF));
    expand_tags(
        env,
        ctx,
        rd32(ctx, OFS_MSGBUF),
        args,
        rd32(ctx, OFS_WORDSET),
        msg,
    );
    show(env, ctx, a);
}

/// C: `FUN_overlay_d_93__021eeef4`: dispatch on the message id through the 14-entry table at `0x021F39D8`
/// (handlers `ef460`, `ef508`, `ef3a4`, `ef594`, `ef204`), default [`msg_by_tags`].
pub fn msg_dispatch<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    a: u32,
    id: u32,
    args: &[u8],
    msg: &Msg,
) {
    for (key, h) in DISPATCH {
        if id == key as u32 {
            match h {
                Handler::Ef460 => msg_count_a(env, ctx, a, id, args, msg),
                Handler::Ef508 => msg_mon(env, ctx, a, id, args, msg),
                Handler::Ef3a4 => msg_count_b(env, ctx, a, id, args, msg),
                Handler::Ef594 => msg_by_side(env, ctx, a, args, msg),
                Handler::Ef204 => msg_variant(env, ctx, a, id, args, msg),
            }
            return;
        }
    }
    msg_by_tags(env, ctx, a, id, args, msg);
}

/// Output pointers of [`tag_next`]: the tag's group (u32), type (u16) and index (u8) are stored here.
pub struct TagOut<'a> {
    pub group: &'a mut [u8],
    pub ty: &'a mut [u8],
    pub idx: &'a mut [u8],
}

/// C: `FUN_overlay_d_93__021eef20`: read the next tag of the string at address `p`: skips characters until a
/// tag start; a valid tag (`0201D5CC`) fills `group`/`ty`/`idx` and returns the address after it; 0 at the
/// terminator (or when `p` is 0).
pub fn tag_next<E: FlowEnv>(env: &mut E, msg: &Msg, p: u32, out: &mut TagOut) -> u32 {
    if p == 0 {
        return 0;
    }
    let term = call(env, 0x0204_5A98, &mut []);
    let tag = call(env, 0x0201_D5C4, &mut []);
    let mut p = p;
    loop {
        let h = msg.h(p);
        if h == term {
            return 0;
        }
        if h == tag {
            if call(env, 0x0201_D5CC, &mut [Arg::P]) != 0 {
                let idx = call(env, 0x0201_D670, &mut [Arg::P, Arg::V(0)]);
                out.idx[0] = idx as u8;
                let g = call(env, 0x0201_D5F4, &mut [Arg::P]);
                wr32(out.group, 0, g);
                let t = call(env, 0x0201_D658, &mut [Arg::P]);
                wr16(out.ty, 0, t as u16);
                return call(env, 0x0201_D688, &mut [Arg::P]);
            }
            p = call(env, 0x0201_D688, &mut [Arg::P]);
        } else {
            p = p.wrapping_add(2);
        }
    }
}

/// C: `FUN_overlay_d_93__021eef98`: the number of tags of the message at `45A90(handle)` with group 1 and
/// type 1, 2 or 12.
pub fn count_tags<E: FlowEnv>(env: &mut E, handle: u32, msg: &Msg) -> u32 {
    let mut p = call(env, 0x0204_5A90, &mut [Arg::V(handle)]);
    let mut n = 0u32;
    if p == 0 {
        return 0;
    }
    let (mut g, mut t, mut i) = ([0u8; 4], [0u8; 2], [0u8; 1]);
    loop {
        p = tag_next(
            env,
            msg,
            p,
            &mut TagOut {
                group: &mut g,
                ty: &mut t,
                idx: &mut i,
            },
        );
        if p != 0 && u32::from_le_bytes(g) == 1 {
            let ty = u16::from_le_bytes(t);
            if ty == 1 || ty == 2 || ty == 12 {
                n = (n + 1) & 0xFF;
            }
        }
        if p == 0 {
            return n;
        }
    }
}

/// C: `FUN_overlay_d_93__021eefe0`: expand the tags of the message at `45A90(handle)`: per tag, dispatch on
/// its type (through the 16-entry halfword table at `0x021EF0F0`) to the word-set helpers, with the tag's
/// argument `args[index]` (index shifted down by the number of type-14 tags before it). `a2` is the first
/// argument of the helpers' `0201Fxxx` calls.
pub fn expand_tags<E: FlowEnv>(
    env: &mut E,
    ctx: &[u8],
    handle: u32,
    args: &[u8],
    a2: u32,
    msg: &Msg,
) {
    let term = call(env, 0x0204_5A98, &mut []);
    let tag = call(env, 0x0201_D5C4, &mut []);
    let mut counts = [0u8; 8];
    // pass 1: for every tag (group 1, type 14), the following indices shift by one
    let mut p = call(env, 0x0204_5A90, &mut [Arg::V(handle)]);
    while msg.h(p) != term {
        if msg.h(p) == tag {
            if call(env, 0x0201_D5CC, &mut [Arg::P]) != 0 {
                let idx = call(env, 0x0201_D670, &mut [Arg::P, Arg::V(0)]);
                let g = call(env, 0x0201_D5F4, &mut [Arg::P]) & 0xFF;
                if g == 1 && call(env, 0x0201_D658, &mut [Arg::P]) == 14 && idx.wrapping_add(1) < 8
                {
                    for c in counts.iter_mut().skip(idx as usize + 1) {
                        *c = c.wrapping_add(1);
                    }
                }
            }
            p = call(env, 0x0201_D688, &mut [Arg::P]);
        } else {
            p = p.wrapping_add(2);
        }
    }
    // pass 2: dispatch
    let mut p = call(env, 0x0204_5A90, &mut [Arg::V(handle)]);
    while msg.h(p) != term {
        if msg.h(p) != tag {
            p = p.wrapping_add(2);
            continue;
        }
        if call(env, 0x0201_D5CC, &mut [Arg::P]) != 0 {
            let idx = call(env, 0x0201_D670, &mut [Arg::P, Arg::V(0)]);
            let g = call(env, 0x0201_D5F4, &mut [Arg::P]) & 0xFF;
            // the original reads an 8-byte stack array here; the specs keep idx below 8
            let cnt = counts.get(idx as usize).copied().unwrap_or(0) as u32;
            let r5 = if idx >= cnt { idx - cnt } else { idx };
            if g == 2 {
                let t = call(env, 0x0201_D658, &mut [Arg::P]).wrapping_add(1) & 0xFF;
                call(
                    env,
                    0x0201_EFA4,
                    &mut [
                        Arg::V(a2),
                        Arg::V(idx),
                        Arg::V(word(args, r5)),
                        Arg::V(t),
                        Arg::V(0),
                        Arg::V(1),
                    ],
                );
            } else {
                let ty = call(env, 0x0201_D658, &mut [Arg::P]);
                let w = word(args, r5);
                match ty {
                    0 => word_by_kind(env, ctx, a2, idx & 0xFF, w & 0xFF),
                    1 => word_visible_mon2(env, ctx, w & 0xFF, idx & 0xFF),
                    2 => {
                        if rd16(ctx, OFS_FLAG) == 0 {
                            word_visible_mon(env, ctx, w & 0xFF, idx);
                        } else {
                            word_pokemon(env, ctx, w & 0xFF, idx);
                        }
                    }
                    12 => word_pokemon(env, ctx, w & 0xFF, idx),
                    14 => word_from_value(env, ctx, a2, idx & 0xFF, w & 0xFF),
                    6 => {
                        call(env, 0x0201_EF8C, &mut [Arg::V(a2), Arg::V(idx), Arg::V(w)]);
                    }
                    7 => {
                        call(env, 0x0201_EF34, &mut [Arg::V(a2), Arg::V(idx), Arg::V(w)]);
                    }
                    9 => {
                        call(env, 0x0201_EF40, &mut [Arg::V(a2), Arg::V(idx), Arg::V(w)]);
                    }
                    3 | 15 => {
                        call(env, 0x0201_F020, &mut [Arg::V(a2), Arg::V(idx), Arg::V(w)]);
                    }
                    // 4, 5, 8, 10, 11, 13 and everything above 15 expand to nothing
                    _ => {}
                }
            }
        }
        p = call(env, 0x0201_D688, &mut [Arg::P]);
    }
}

/// C: `FUN_overlay_d_93__021ef1cc`: set message `b` in bank `ctx[0x14]`; with exactly two counted tags
/// (`eef98`) continue with [`msg_two_mons`], else [`msg_variant`].
pub fn msg_by_tags<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    a: u32,
    b: u32,
    args: &[u8],
    msg: &Msg,
) {
    set_message(env, rd32(ctx, OFS_BANKS + 4), b, rd32(ctx, OFS_MSGBUF));
    if count_tags(env, rd32(ctx, OFS_MSGBUF), msg) == 2 {
        msg_two_mons(env, ctx, a, b, args, msg);
    } else {
        msg_variant(env, ctx, a, b, args, msg);
    }
}

/// C: `FUN_overlay_d_93__021ef204`: message `b` + side variant of mon `args[0]` (0 / 1 / 2) in bank
/// `ctx[0x14]`, expand, show.
pub fn msg_variant<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    a: u32,
    b: u32,
    args: &[u8],
    msg: &Msg,
) {
    let v = side_variant(env, ctx, word(args, 0) & 0xFF);
    let id = b.wrapping_add(v) & 0xFFFF;
    set_message(env, rd32(ctx, OFS_BANKS + 4), id, rd32(ctx, OFS_MSGBUF));
    expand_tags(
        env,
        ctx,
        rd32(ctx, OFS_MSGBUF),
        args,
        rd32(ctx, OFS_WORDSET),
        msg,
    );
    show(env, ctx, a);
}

/// C: `FUN_overlay_d_93__021ef280`: like [`msg_variant`] for two mons (`args[0]`, `args[1]`): base offset
/// 0 / 3 / 5 by the first mon's variant, plus 1 when both variants are equal.
pub fn msg_two_mons<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    a: u32,
    b: u32,
    args: &[u8],
    msg: &Msg,
) {
    let m1 = word(args, 1) & 0xFF;
    let v1 = side_variant(env, ctx, word(args, 0) & 0xFF);
    let id = if v1 == 0 {
        let v2 = side_variant(env, ctx, m1);
        b.wrapping_add(v2) & 0xFFFF
    } else {
        let mut id = match v1 {
            1 => b.wrapping_add(3),
            2 => b.wrapping_add(5),
            _ => b,
        } & 0xFFFF;
        let v3 = side_variant(env, ctx, m1);
        if v1 == v3 {
            id = id.wrapping_add(1) & 0xFFFF;
        }
        id
    };
    set_message(env, rd32(ctx, OFS_BANKS + 4), id, rd32(ctx, OFS_MSGBUF));
    expand_tags(
        env,
        ctx,
        rd32(ctx, OFS_MSGBUF),
        args,
        rd32(ctx, OFS_WORDSET),
        msg,
    );
    show(env, ctx, a);
}

/// 0x15 * (0 / 1 / 2) for a count: the second argument `n` (signed) is >= 3 -> 2, == 2 -> 1, else 0.
fn count_step(n: u32) -> u32 {
    let mut k = 0;
    if (n as i32) >= 3 {
        k = 2;
    }
    if n == 2 {
        k = 1;
    }
    (0x15 * k) & 0xFFFF
}

/// C: `sub_021ef3a4` (dispatch handler for message 0x3AA): message `b` + `0x15 * count(args[3])` +
/// `3 * (args[2] - 1)` + side variant of mon `args[0]`; mon word 0 -> slot 0, `0201EF40(wordset, 1, args[1])`.
pub fn msg_count_b<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    a: u32,
    b: u32,
    args: &[u8],
    msg: &Msg,
) {
    let _ = msg;
    let r7 = word(args, 2).wrapping_sub(1) & 0xFF;
    let b = b.wrapping_add(count_step(word(args, 3))) & 0xFFFF;
    let v = side_variant(env, ctx, word(args, 0) & 0xFF);
    word_visible_mon(env, ctx, word(args, 0) & 0xFF, 0);
    call(
        env,
        0x0201_EF40,
        &mut [
            Arg::V(rd32(ctx, OFS_WORDSET)),
            Arg::V(1),
            Arg::V(word(args, 1)),
        ],
    );
    let id = b.wrapping_add(r7.wrapping_mul(3)).wrapping_add(v) & 0xFFFF;
    set_message(env, rd32(ctx, OFS_BANKS + 4), id, rd32(ctx, OFS_MSGBUF));
    show(env, ctx, a);
}

/// C: `sub_021ef460` (dispatch handler for 0x1B / 0x5A): like [`msg_count_b`] with the count in
/// `args[2]` and the multiplier `args[1] - 1`, no second word, and the mon word set before the variant
/// lookup. It only sets the message and shows it (no tag expansion).
pub fn msg_count_a<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    a: u32,
    b: u32,
    args: &[u8],
    msg: &Msg,
) {
    let _ = msg;
    let r6 = word(args, 1).wrapping_sub(1) & 0xFF;
    let b = b.wrapping_add(count_step(word(args, 2))) & 0xFFFF;
    word_visible_mon(env, ctx, word(args, 0) & 0xFF, 0);
    let v = side_variant(env, ctx, word(args, 0) & 0xFF);
    let id = b.wrapping_add(r6.wrapping_mul(3)).wrapping_add(v) & 0xFFFF;
    set_message(env, rd32(ctx, OFS_BANKS + 4), id, rd32(ctx, OFS_MSGBUF));
    show(env, ctx, a);
}

/// C: `sub_021ef508` (dispatch handler for 0x99 / 0xAE): message `b` + `3 * (args[1] - 1)` + side variant
/// of mon `args[0]`; mon word 0 first.
pub fn msg_mon<E: FlowEnv>(env: &mut E, ctx: &mut [u8], a: u32, b: u32, args: &[u8], msg: &Msg) {
    let _ = msg;
    let r5 = word(args, 1).wrapping_sub(1) & 0xFF;
    word_visible_mon(env, ctx, word(args, 0) & 0xFF, 0);
    let v = side_variant(env, ctx, word(args, 0) & 0xFF);
    let id = b.wrapping_add(r5.wrapping_mul(3)).wrapping_add(v) & 0xFFFF;
    set_message(env, rd32(ctx, OFS_BANKS + 4), id, rd32(ctx, OFS_MSGBUF));
    show(env, ctx, a);
}

/// C: `sub_021ef594` (dispatch handler for 0x21): message 0x21 when mon `args[0]` is the battle's own mon
/// (`021b8ff4`), 0x23 when `021b9f70` says so for it, else 0x22; set in bank `ctx[0x10]`, expand, show.
pub fn msg_by_side<E: FlowEnv>(env: &mut E, ctx: &mut [u8], a: u32, args: &[u8], msg: &Msg) {
    let m = word(args, 0) & 0xFF;
    let sys = rd32(ctx, OFS_SYS);
    let own = call(env, 0x021B_8FF4, &mut [Arg::V(sys)]);
    let id = if m == own {
        0x21
    } else if call(env, 0x021B_9F70, &mut [Arg::V(sys), Arg::V(m)]) != 0 {
        0x23
    } else {
        0x22
    };
    set_message(env, rd32(ctx, OFS_BANKS), id, rd32(ctx, OFS_MSGBUF));
    expand_tags(
        env,
        ctx,
        rd32(ctx, OFS_MSGBUF),
        args,
        rd32(ctx, OFS_WORDSET),
        msg,
    );
    show(env, ctx, a);
}

/// C: `FUN_overlay_d_93__021ef5e8`: mon `b` word 0, then message `3 * c` + side variant of `b` in bank
/// `ctx[0x18]`, show (no tag expansion).
pub fn msg_mon_side<E: FlowEnv>(env: &mut E, ctx: &[u8], a: u32, b: u32, c: u32) {
    let v = side_variant(env, ctx, b);
    word_visible_mon(env, ctx, b, 0);
    let id = ((c.wrapping_mul(3) & 0xFFFF).wrapping_add(v)) & 0xFFFF;
    set_message(env, rd32(ctx, OFS_BANKS + 8), id, rd32(ctx, OFS_MSGBUF));
    show(env, ctx, a);
}

/// C: `FUN_overlay_d_93__021ef664`: when `b` is nonzero, set message `3 * (b - 1) + 1 + (c != 0)` from
/// the bank handle at `ctx + 0x2C` into buffer `a`.
pub fn msg_choice<E: FlowEnv>(env: &mut E, ctx: &[u8], a: u32, b: u32, c: u32) {
    if b == 0 {
        return;
    }
    let base = (b.wrapping_sub(1).wrapping_mul(3)) & 0xFFFF;
    let id = (if c == 0 { base + 1 } else { base + 2 }) & 0xFFFF;
    set_message(env, rd32(ctx, 0x2C), id, a);
}

/// C: `FUN_overlay_d_93__021ef694`: tail call `02045CAC(ctx[0x1C], b, a)`.
pub fn msg_set_1c<E: FlowEnv>(env: &mut E, ctx: &[u8], a: u32, b: u32) {
    set_message(env, rd32(ctx, 0x1C), b, a);
}

/// C: `FUN_overlay_d_93__021ef6a8`: thunk to [`msg_flagged`].
pub fn msg_flagged_thunk<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    a: u32,
    b: u32,
    args: &[u8],
    msg: &Msg,
) {
    msg_flagged(env, ctx, a, b, args, msg);
}

/// C: `FUN_overlay_d_93__021ef6b0`: set message `b` in bank `ctx[0x24]`, expand its tags with the flag
/// `ctx + 0x5A` raised, show.
pub fn msg_flagged<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    a: u32,
    b: u32,
    args: &[u8],
    msg: &Msg,
) {
    set_message(env, rd32(ctx, 0x24), b, rd32(ctx, OFS_MSGBUF));
    wr16(ctx, OFS_FLAG, 1);
    expand_tags(
        env,
        ctx,
        rd32(ctx, OFS_MSGBUF),
        args,
        rd32(ctx, OFS_WORDSET),
        msg,
    );
    wr16(ctx, OFS_FLAG, 0);
    show(env, ctx, a);
}

/// C: `FUN_overlay_d_93__021ef6e8`: thunk to [`msg_bank28`].
pub fn msg_bank28_thunk<E: FlowEnv>(
    env: &mut E,
    ctx: &mut [u8],
    a: u32,
    b: u32,
    args: &[u8],
    msg: &Msg,
) {
    msg_bank28(env, ctx, a, b, args, msg);
}

/// C: `FUN_overlay_d_93__021ef6f0`: set message `b` in bank `ctx[0x28]`, expand its tags, show.
pub fn msg_bank28<E: FlowEnv>(env: &mut E, ctx: &mut [u8], a: u32, b: u32, args: &[u8], msg: &Msg) {
    set_message(env, rd32(ctx, 0x28), b, rd32(ctx, OFS_MSGBUF));
    expand_tags(
        env,
        ctx,
        rd32(ctx, OFS_MSGBUF),
        args,
        rd32(ctx, OFS_WORDSET),
        msg,
    );
    show(env, ctx, a);
}
