//! Flow core, part 6 (0x021C65A8-0x021C7CBC; B10k).
//!
//! The message round trips between the server flow and the client for the damage / ability / item questions of the
//! turn (`65a8` .. `7bf8`), the type-chart check group (`6e3c`, `7038`, `6f0c`, `6f34`, `6fc0`), the command emitter
//! `70e4` and the statistics counter `7c24`. Specs: `tools/rom-diff/svflow_specs/core6.spec` (52 functions; the 9
//! `bx pc` list-accessor veneers and their two thunk halves are listed at the top of the spec).
//!
//! B10k: all 52 specced functions are ported. Callees outside this area go through [`FlowEnv`]; `cmd_queue` ones and
//! the in-area functions are called directly (they run as real code on both sides of the diff).
//!
//! Every round trip has the same shape: `74f8(msg)` starts a message, `75ec` / `75a8` / `7630` add numbered
//! arguments (BattleMon arguments as their id from `5440`), `70f0(work, cmd)` sends it, `775c(slot)` reads a reply
//! and `7540(msg + n)` ends it.
//!
//! All 52 specced functions are in [`PORTED`] (52/52). The 40 trap functions are `force` stubs in the core1-core5 specs, so
//! their callers in core1-core5 keep reaching them through `env.call`.

use super::cmd_queue;
use super::env::{Arg, FlowEnv};
use super::SvFlowWork;

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x021C_65A8,
    0x021C_6668,
    0x021C_66AC,
    0x021C_6720,
    0x021C_676C,
    0x021C_6848,
    0x021C_6878,
    0x021C_68A8,
    0x021C_6948,
    0x021C_69B8,
    0x021C_6A04,
    0x021C_6A9C,
    0x021C_6B5C,
    0x021C_6BB8,
    0x021C_6BF8,
    0x021C_6CAC,
    0x021C_6DD0,
    0x021C_6E00,
    0x021C_6E3C,
    0x021C_6F0C,
    0x021C_6F34,
    0x021C_6FC0,
    0x021C_7008,
    0x021C_7038,
    0x021C_70E4,
    0x021C_7194,
    0x021C_71E0,
    0x021C_728C,
    0x021C_7330,
    0x021C_7374,
    0x021C_7474,
    0x021C_75EC,
    0x021C_7664,
    0x021C_76F4,
    0x021C_7780,
    0x021C_77B0,
    0x021C_7820,
    0x021C_785C,
    0x021C_78A8,
    0x021C_78F4,
    0x021C_7998,
    0x021C_79E0,
    0x021C_7A54,
    0x021C_7AA0,
    0x021C_7AD0,
    0x021C_7B0C,
    0x021C_7B3C,
    0x021C_7B7C,
    0x021C_7BA8,
    0x021C_7BF8,
    0x021C_7C18,
    0x021C_7C24,
];

/// `work + 0x4`: the main module (address only).
const OFS_MAIN: usize = 0x4;
/// `work + 0x8`: the party container (address only).
const OFS_POKECON: usize = 0x8;
/// `work + 0x1D74`: the packed queue-header word that `6fc0` brackets with `cc8d8` / `cc95c`.
const OFS_QUEUE_WORD: usize = 0x1D74;

// ---- the message-builder vocabulary (all outside the area) ----

/// `FUN_overlay_d_93__021d5440`: the id (0..5) of a BattleMon.
fn mon_id<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    env.call(0x021D_5440, &mut [Arg::V(mon)])
}

/// `021d74f8(msg)`: starts message `msg`.
fn begin<E: FlowEnv>(env: &mut E, msg: u32) {
    env.call(0x021D_74F8, &mut [Arg::V(msg)]);
}

/// `021d75ec(slot, v)`: sets argument `slot` (value form).
fn put<E: FlowEnv>(env: &mut E, slot: u32, v: u32) {
    env.call(0x021D_75EC, &mut [Arg::V(slot), Arg::V(v)]);
}

/// `021d75a8(slot, v)`: sets argument `slot` (second form).
fn put_a8<E: FlowEnv>(env: &mut E, slot: u32, v: u32) {
    env.call(0x021D_75A8, &mut [Arg::V(slot), Arg::V(v)]);
}

/// `021d7630(slot, v)`: declares reply slot `slot` with the default `v`.
fn put_reply<E: FlowEnv>(env: &mut E, slot: u32, v: u32) {
    env.call(0x021D_7630, &mut [Arg::V(slot), Arg::V(v)]);
}

/// `021d70f0(work, cmd)`: sends the message with command `cmd`.
fn send<E: FlowEnv>(env: &mut E, cmd: u32) {
    env.call(0x021D_70F0, &mut [Arg::P, Arg::V(cmd)]);
}

/// `021d775c(slot)`: the reply in `slot`.
fn read<E: FlowEnv>(env: &mut E, slot: u32) -> u32 {
    env.call(0x021D_775C, &mut [Arg::V(slot)])
}

/// `021d7540(msg + n)`: ends the message.
fn end<E: FlowEnv>(env: &mut E, msg: u32) {
    env.call(0x021D_7540, &mut [Arg::V(msg)]);
}

/// `021d7674(0x35, 0x1000, 0x19A, 0x20000)`: declares the reply slot 0x35 with a range (the damage-scale reply).
fn put_reply_range<E: FlowEnv>(env: &mut E, a: u32, b: u32) {
    env.call(
        0x021D_7674,
        &mut [Arg::V(0x35), Arg::V(0x1000), Arg::V(a), Arg::V(b)],
    );
}

/// `0201BD28(x, k)`: byte field `k` of the record/word `x`.
fn field<E: FlowEnv>(env: &mut E, x: u32, k: u32) -> u32 {
    env.call(0x0201_BD28, &mut [Arg::V(x), Arg::V(k)])
}

/// `021d59b4(mon, k)`: BattleMon value `k`.
fn mon_value<E: FlowEnv>(env: &mut E, mon: u32, k: u32) -> u32 {
    env.call(0x021D_59B4, &mut [Arg::V(mon), Arg::V(k)])
}

/// `021d7b10(x, y)`: scales `x` by the fixed-point reply `y`.
fn scale<E: FlowEnv>(env: &mut E, x: u32, y: u32) -> u32 {
    env.call(0x021D_7B10, &mut [Arg::V(x), Arg::V(y)])
}

fn u16_at(b: &[u8], ofs: usize) -> u32 {
    u16::from_le_bytes([b[ofs], b[ofs + 1]]) as u32
}

fn u32_at(b: &[u8], ofs: usize) -> u32 {
    u32::from_le_bytes(b[ofs..ofs + 4].try_into().unwrap())
}

/// Sends the member ids of a list: argument `5` = the count, `6 + i` = the id of member `i` (`list_count` /
/// `list_member` are the `bx pc` veneers of the list object).
fn put_list_ids<E: FlowEnv>(env: &mut E, list: u32, n: u32, member: u32) {
    for i in 0..n {
        let m = env.call(member, &mut [Arg::V(list), Arg::V(i)]);
        let id = mon_id(env, m);
        put(env, i + 6, id);
    }
}

// ---- 65a8 .. 6e00 ----

/// Port of `FUN_overlay_d_93__021c65a8`: message 0x2D0D: argument 2 = id of `mon`, 0x12 = `value`, 5 = member count of
/// `list`, 6.. = member ids; returns the reply 0x3E. The u16 reply 0x13 is stored through the pointer at
/// `work + 0x4A8` when non-zero (`slot16` is the 2-byte target of that pointer).
pub fn list_round_trip<E: FlowEnv>(
    env: &mut E,
    slot16: &mut [u8],
    mon: u32,
    value: u32,
    list: u32,
) -> u32 {
    let n = env.call(0x021C_6648, &mut [Arg::V(list)]);
    begin(env, 0x2D0D);
    let id = mon_id(env, mon);
    put(env, 2, id);
    put(env, 0x12, value);
    put(env, 5, n);
    put_list_ids(env, list, n, 0x021C_6658);
    put_reply(env, 0x3E, 0);
    put_reply(env, 0x13, 0);
    send(env, 0x24);
    let r = read(env, 0x3E);
    let u = read(env, 0x13) & 0xFFFF;
    end(env, 0x2D1E);
    if u != 0 {
        slot16[..2].copy_from_slice(&(u as u16).to_le_bytes());
    }
    r
}

/// Port of `FUN_overlay_d_93__021c6668`: message 0x2D36 for `mon`, returns the reply 0x51.
pub fn message_2d36<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    begin(env, 0x2D36);
    let id = mon_id(env, mon);
    put_a8(env, 3, id);
    put_a8(env, 0x51, 0);
    send(env, 0x2E);
    let r = read(env, 0x51);
    end(env, 0x2D36 + 5);
    r
}

/// Port of `FUN_overlay_d_93__021c66ac`: message 0x2D4F for `mon` and the first member of `list` (0x1F when none);
/// returns the reply 0x41.
pub fn message_2d4f<E: FlowEnv>(env: &mut E, mon: u32, list: u32) -> u32 {
    let first = env.call(0x021C_6710, &mut [Arg::V(list), Arg::V(0)]);
    begin(env, 0x2D4F);
    let id = mon_id(env, mon);
    put(env, 3, id);
    let id2 = if first == 0 { 0x1F } else { mon_id(env, first) };
    put(env, 4, id2);
    put_reply(env, 0x41, 0);
    send(env, 0x93);
    let r = read(env, 0x41);
    end(env, 0x2D55);
    r
}

/// Port of `FUN_overlay_d_93__021c6720`: message 0x2D66 for `mon` with `value`; returns the reply 0x51.
pub fn message_2d66<E: FlowEnv>(env: &mut E, mon: u32, value: u32) -> u32 {
    begin(env, 0x2D66);
    let id = mon_id(env, mon);
    put(env, 3, id);
    put(env, 0x12, value);
    put_reply(env, 0x51, 0);
    send(env, 0x94);
    let r = read(env, 0x51);
    end(env, 0x2D66 + 6);
    r
}

/// Port of `FUN_overlay_d_93__021c676c`: message 0x2D80 for `mon` and the members of `list` (0x1F when empty); stores the
/// reply 6 (0x1F when the reply 0x51 is 0) into `out_u8[0]` and the reply 0x4F into `out_int`; true when the reply
/// 0x41 is 0.
pub fn message_2d80<E: FlowEnv>(
    env: &mut E,
    mon: u32,
    list: u32,
    out_u8: &mut [u8],
    out_int: &mut [u8],
) -> bool {
    let n = env.call(0x021C_6828, &mut [Arg::V(list)]);
    begin(env, 0x2D80);
    let id = mon_id(env, mon);
    put(env, 3, id);
    put(env, 5, n);
    if n == 0 {
        put(env, 6, 0x1F);
    } else {
        put_list_ids(env, list, n, 0x021C_6838);
    }
    put_reply(env, 0x41, 0);
    put_reply(env, 0x51, 0);
    put_reply(env, 0x4F, 0);
    send(env, 0x95);
    let v = if read(env, 0x51) == 0 {
        0x1F
    } else {
        read(env, 6)
    };
    out_u8[0] = v as u8;
    let w = read(env, 0x4F);
    out_int[..4].copy_from_slice(&w.to_le_bytes());
    let r = read(env, 0x41);
    end(env, 0x2D9D);
    r == 0
}

/// Port of `FUN_overlay_d_93__021c6848`: message 0x2DAC (command 0x96) for `mon`.
pub fn message_2dac<E: FlowEnv>(env: &mut E, mon: u32) {
    begin(env, 0x2DAC);
    let id = mon_id(env, mon);
    put(env, 3, id);
    send(env, 0x96);
    end(env, 0x2DAC + 3);
}

/// Port of `FUN_overlay_d_93__021c6878`: message 0x2DBD (command 0x97) for `mon`.
pub fn message_2dbd<E: FlowEnv>(env: &mut E, mon: u32) {
    begin(env, 0x2DBD);
    let id = mon_id(env, mon);
    put(env, 3, id);
    send(env, 0x97);
    end(env, 0x2DBD + 3);
}

/// Port of `FUN_overlay_d_93__021c68a8`: message 0x2DD3 for `mon` and the members of `list`; true when the reply 0x41 is 0.
pub fn message_2dd3<E: FlowEnv>(env: &mut E, mon: u32, list: u32) -> bool {
    let n = env.call(0x021C_6928, &mut [Arg::V(list)]);
    begin(env, 0x2DD3);
    let id = mon_id(env, mon);
    put(env, 3, id);
    put_reply(env, 0x41, 0);
    put(env, 5, n);
    put_list_ids(env, list, n, 0x021C_6938);
    send(env, 0x98);
    let r = read(env, 0x41);
    end(env, 0x2DDE);
    r & 0xFF == 0
}

/// Port of `FUN_overlay_d_93__021c6948`: when `5c08(b)` is not 0x10, message 0x2DF5 for both mons and that value; true when
/// the reply 0x42 is non-zero.
pub fn message_2df5<E: FlowEnv>(env: &mut E, a: u32, b: u32) -> bool {
    let v = env.call(0x021D_5C08, &mut [Arg::V(b)]);
    let mut res = false;
    if v != 0x10 {
        begin(env, 0x2DF5);
        let ia = mon_id(env, a);
        put(env, 3, ia);
        let ib = mon_id(env, b);
        put(env, 4, ib);
        put(env, 0x21, v);
        put_reply(env, 0x42, 1);
        send(env, 0x99);
        res = read(env, 0x42) != 0;
        end(env, 0x2DFF);
    }
    res
}

/// Port of `FUN_overlay_d_93__021c69b8`: `c1f4(x)`; when it is 0 message 0x2E15 for `mon` and the reply 0x51 instead.
pub fn message_2e15<E: FlowEnv>(env: &mut E, mon: u32, x: u32) -> u32 {
    let mut r = env.call(0x0201_C1F4, &mut [Arg::V(x)]);
    if r == 0 {
        begin(env, 0x2E15);
        let id = mon_id(env, mon);
        put(env, 3, id);
        put_reply(env, 0x51, 0);
        send(env, 0x32);
        r = read(env, 0x51);
        end(env, 0x2E15 + 5);
    }
    r
}

/// Port of `FUN_overlay_d_93__021c6a04`: message 0x2E2F for the mons `a`, `b` and the move record `rec` (u16 id at +0,
/// u32 at +8); the replies 0x2B / 0x35 are scaled by `7b10` (returned).
pub fn message_2e2f<E: FlowEnv>(env: &mut E, a: u32, b: u32, rec: &[u8]) -> u32 {
    let v = field(env, u16_at(rec, 0), 4);
    begin(env, 0x2E2F);
    let ia = mon_id(env, a);
    put(env, 3, ia);
    let ib = mon_id(env, b);
    put(env, 4, ib);
    put(env, 0x12, u16_at(rec, 0));
    put(env, 0x1A, u32_at(rec, 8));
    put_reply(env, 0x2B, v);
    put_reply_range(env, 0x19A, 0x20000);
    send(env, 0x34);
    let r1 = read(env, 0x2B);
    let r2 = read(env, 0x35);
    end(env, 0x2E2F + 10);
    scale(env, r1, r2)
}

/// Port of `FUN_overlay_d_93__021c6a9c`: 0 when `a`'s value 0xF is below `b`'s; else message 0x2E5A (reply 0x41): non-zero
/// reply -> 0; otherwise `bf324(work, a, b)` (core3) -> 1, else a percent roll `78ac(100)` against
/// `bd28(r3, 4) + (va - vb)`.
pub fn priority_roll<E: FlowEnv>(env: &mut E, a: u32, b: u32, r3: u32) -> u32 {
    let va = mon_value(env, a, 0xF) & 0xFF;
    let vb = mon_value(env, b, 0xF) & 0xFF;
    if va < vb {
        return 0;
    }
    begin(env, 0x2E5A);
    let ia = mon_id(env, a);
    put(env, 3, ia);
    let ib = mon_id(env, b);
    put(env, 4, ib);
    put_reply(env, 0x41, 0);
    send(env, 0x70);
    let r = read(env, 0x41);
    end(env, 0x2E5A + 6);
    if r != 0 {
        return 0;
    }
    if env.call(0x021B_F324, &mut [Arg::P, Arg::V(a), Arg::V(b)]) == 0 {
        let u = field(env, r3, 4) & 0xFF;
        let roll = env.call(0x021D_78AC, &mut [Arg::V(100)]);
        if (u.wrapping_add(va.wrapping_sub(vb) & 0xFF) & 0xFF) <= roll {
            return 0;
        }
    }
    1
}

/// Port of `FUN_overlay_d_93__021c6b5c`: message 0x2E83 for the mons `a`, `b` and the byte at `rec + 6`; returns the reply 0x51.
pub fn message_2e83<E: FlowEnv>(env: &mut E, a: u32, b: u32, rec: &[u8]) -> u32 {
    begin(env, 0x2E83);
    let ia = mon_id(env, a);
    put(env, 3, ia);
    let ib = mon_id(env, b);
    put(env, 4, ib);
    put(env, 0x16, rec[6] as u32);
    put_reply(env, 0x51, 0);
    send(env, 0x30);
    let r = read(env, 0x51);
    end(env, 0x2E83 + 7);
    r
}

/// Port of `FUN_overlay_d_93__021c6bb8`: message 0x2E98 (command 0x31) for the mons `a`, `b`.
pub fn message_2e98<E: FlowEnv>(env: &mut E, a: u32, b: u32) {
    begin(env, 0x2E98);
    let ia = mon_id(env, a);
    put(env, 3, ia);
    let ib = mon_id(env, b);
    put(env, 4, ib);
    send(env, 0x31);
    end(env, 0x2E98 + 4);
}

/// Port of `FUN_overlay_d_93__021c6bf8`: message 0x2EB0 (reply 0x41, value 0x2C = `bd28(r3, 7) + 5f14(a)`); when the reply
/// is 0: `c284(r3)` -> 1, else `7930(min(reply 0x2C, 4))`; then `911c(work[4], 3)` forces 1.
pub fn message_2eb0<E: FlowEnv>(env: &mut E, w: &SvFlowWork, a: u32, b: u32, r3: u32) -> u32 {
    let v = field(env, r3, 7);
    let extra = env.call(0x021D_5F14, &mut [Arg::V(a)]) & 0xFF;
    begin(env, 0x2EB0);
    let ib = mon_id(env, b);
    put(env, 4, ib);
    let ia = mon_id(env, a);
    put(env, 3, ia);
    put_a8(env, 0x2C, ((v & 0xFFFF).wrapping_add(extra)) & 0xFFFF);
    put_reply(env, 0x41, 0);
    send(env, 0x36);
    let mut res = 0;
    if read(env, 0x41) == 0 {
        if env.call(0x0201_C284, &mut [Arg::V(r3)]) == 0 {
            let mut u = read(env, 0x2C);
            if (u as i32) > 4 {
                u = 4;
            }
            res = env.call(0x021D_7930, &mut [Arg::V(u & 0xFF)]);
        } else {
            res = 1;
        }
        let main = u32_at(&w.mem[..], OFS_MAIN);
        if env.call(0x021B_911C, &mut [Arg::V(main), Arg::V(3)]) != 0 {
            res = 1;
        }
    }
    end(env, 0x2EC7);
    res
}

/// Port of `FUN_overlay_d_93__021c6cac`: message 0x2EF3 for `mon` (replies 0x35 / 0x41 / 0x51 / 0x36) scaling either
/// `59b4(mon, 0xE)` (when `x` has field 0x1F but not 0x1E) or `amount` through `7b3c`; `*out` gets the reply 0x51.
pub fn scaled_message<E: FlowEnv>(
    env: &mut E,
    mon: u32,
    x: u32,
    amount: u32,
    out: &mut [u8],
) -> u32 {
    let id = mon_id(env, mon);
    let mut f = field(env, x, 0x1E) & 0xFF;
    let mut kind = 0;
    out[..4].copy_from_slice(&0u32.to_le_bytes());
    if f == 0 {
        f = field(env, x, 0x1F) & 0xFF;
        if f != 0 {
            kind = 2;
        }
    } else {
        kind = 1;
    }
    if env.call(0x0201_C1BC, &mut [Arg::V(x)]) == 8 {
        return 0;
    }
    begin(env, 0x2EF3);
    put(env, 3, id);
    put_a8(env, 0x35, f);
    put_a8(env, 0x36, 0);
    put_reply(env, 0x41, 0);
    put_reply(env, 0x51, 0);
    send(env, 0x50);
    let r35 = read(env, 0x35) & 0xFF;
    let r41 = read(env, 0x41) & 0xFF;
    let r51 = read(env, 0x51) & 0xFF;
    let r36 = read(env, 0x36) & 0xFF;
    end(env, 0x2EF3 + 0xD);
    if r51 == 0 && r41 == 1 {
        return 0;
    }
    let t = r35.wrapping_add(r36) & 0xFF;
    if t == 0 {
        return 0;
    }
    let mut v;
    if kind == 2 {
        let base = mon_value(env, mon, 0xE);
        v = env.call(0x021D_7B3C, &mut [Arg::V(base), Arg::V(t)]);
        if (v as i32) < 1 {
            v = 1;
        }
    } else if amount == 0 {
        v = 0;
    } else {
        v = env.call(0x021D_7B3C, &mut [Arg::V(amount), Arg::V(t)]);
        if (v as i32) < 1 {
            v = 1;
        }
    }
    out[..4].copy_from_slice(&r51.to_le_bytes());
    v
}

/// Port of `FUN_overlay_d_93__021c6dd0`: message 0x2F2D (command 0x72) for `mon`.
pub fn message_2f2d<E: FlowEnv>(env: &mut E, mon: u32) {
    begin(env, 0x2F2D);
    let id = mon_id(env, mon);
    put(env, 2, id);
    send(env, 0x72);
    end(env, 0x2F2D + 3);
}

/// C: `FUN_overlay_d_93__021c6e00`: message 0x2F3D for `mon` and `value`, sent with `70e4`.
pub fn message_2f3d<E: FlowEnv>(env: &mut E, mon: u32, value: u32) {
    begin(env, 0x2F3D);
    let id = mon_id(env, mon);
    put_a8(env, 2, id);
    put_a8(env, 3, value);
    env.call(0x021D_70E4, &mut [Arg::P, Arg::V(0x73)]);
    end(env, 0x2F3D + 4);
}

/// Port of `FUN_overlay_d_93__021c6e3c`: type-chart check of the move type `rec[6]` (0x11 -> 3) against the defender `b`'s
/// type pair: `7038` for each type of the pair (combined by `79e8`), then for type 4 the pair-resist question `6f0c`;
/// with `flag` set and the question answered a message is sent (`5750`, or the bracketed `6fc0` when it was 1).
/// Returns 3 (no effect), the multiplier code, or 0.
pub fn type_check<E: FlowEnv>(
    env: &mut E,
    w: &mut SvFlowWork,
    a: u32,
    b: u32,
    rec: &[u8],
    flag: u32,
) -> u32 {
    let ty = rec[6] as u32;
    let mut local = false;
    if ty == 0x11 {
        return 3;
    }
    let pair = env.call(0x021D_57FC, &mut [Arg::V(b)]) & 0xFFFF;
    let t1 = env.call(0x021E_8CFC, &mut [Arg::V(pair)]) & 0xFF;
    let mut m = type_question(env, a, b, ty, t1);
    if env.call(0x021E_8DA8, &mut [Arg::V(pair)]) == 0 {
        let t2 = env.call(0x021E_8D04, &mut [Arg::V(pair)]) & 0xFF;
        let m2 = type_question(env, a, b, ty, t2);
        m = env.call(0x021D_79E8, &mut [Arg::V(m & 0xFF), Arg::V(m2)]) & 0xFF;
    }
    if m == 0 {
        if ty == 4 && !pair_resists(env, b, 1) {
            return 3;
        }
    } else {
        if ty != 4 {
            return m;
        }
        local = pair_resists(env, b, 0);
        if !local {
            return m;
        }
    }
    if flag != 0 {
        if !local {
            env.call(0x021C_5750, &mut [Arg::P, Arg::V(b)]);
        } else {
            bracketed_messages(env, w, b);
        }
    }
    0
}

/// C: `FUN_overlay_d_93__021c6f34`: `5814(mon, 2)` (when `x` != 0, else 0) as the default of the reply 0x51 of message
/// 0x2FBC; returns reply 0x51 when the reply 0x41 is 0, else 0.
pub fn resist_message<E: FlowEnv>(env: &mut E, mon: u32, x: u32) -> u32 {
    let q = if x == 0 {
        0
    } else {
        env.call(0x021D_5814, &mut [Arg::V(mon), Arg::V(2)]) & 0xFF
    };
    begin(env, 0x2FBC);
    let id = mon_id(env, mon);
    put(env, 2, id);
    put_reply(env, 0x51, q);
    put_reply(env, 0x41, 0);
    send(env, 0x12);
    env.call(0x021C_6FB0, &mut [Arg::P, Arg::V(mon)]);
    let r51 = read(env, 0x51) & 0xFF;
    let r41 = read(env, 0x41) & 0xFF;
    end(env, 0x2FBC + 8);
    if r41 == 0 {
        r51
    } else {
        0
    }
}

/// C: `FUN_overlay_d_93__021c7008`: message 0x2FE9 (command 0x1B) for `mon`.
pub fn message_1b<E: FlowEnv>(env: &mut E, mon: u32) {
    begin(env, 0x2FE9);
    let id = mon_id(env, mon);
    put(env, 2, id);
    send(env, 0x1B);
    end(env, 0x2FE9 + 3);
}

/// C: `FUN_overlay_d_93__021c7038`: type-chart question (message 0x3002, sent with 0x3E) for the mons `a`, `b` and the
/// move types `ty` (slot 0x16) and `ty2` (slot 0x15, the stack argument, low byte): 3 when the client answers slot
/// 0x4C, else the `794c(ty, ty2)` multiplier code, replaced by 3 when that is 0 and slot 0x4B is set.
pub fn type_question<E: FlowEnv>(env: &mut E, a: u32, b: u32, ty: u32, ty2: u32) -> u32 {
    begin(env, 0x3002);
    let id_a = mon_id(env, a);
    put(env, 3, id_a);
    let id_b = mon_id(env, b);
    put(env, 4, id_b);
    put(env, 0x15, ty2 & 0xFF);
    put(env, 0x16, ty);
    put_reply(env, 0x4B, 0);
    put_reply(env, 0x4C, 0);
    env.call(0x021C_70D4, &mut [Arg::P, Arg::V(b)]);
    send(env, 0x3E);
    let r4b = read(env, 0x4B);
    let r4c = read(env, 0x4C);
    end(env, 0x3002 + 0xD);
    if r4c != 0 {
        return 3;
    }
    let mult = env.call(0x021D_794C, &mut [Arg::V(ty), Arg::V(ty2 & 0xFF)]);
    if mult == 0 && r4b != 0 {
        3
    } else {
        mult
    }
}

/// C: `FUN_overlay_d_93__021c6f0c`: true when `ef948(2)` is 0 and the message pair `6f34(work, mon, flag)` answers
/// non-zero.
pub fn pair_resists<E: FlowEnv>(env: &mut E, mon: u32, flag: u32) -> bool {
    env.call(0x021E_F948, &mut [Arg::V(2)]) == 0 && resist_message(env, mon, flag) != 0
}

/// C: `FUN_overlay_d_93__021c6fc0`: brackets the packed queue-header word at `work + 0x1D74` (`cc8d8` ... `cc95c`)
/// around `7008(work, mon)` and, unless the flow API `88e4` says 2, `5750(work, mon)`.
pub fn bracketed_messages<E: FlowEnv>(env: &mut E, w: &mut SvFlowWork, mon: u32) {
    let old = cmd_queue::advance_fields(&mut w.mem[OFS_QUEUE_WORD..OFS_QUEUE_WORD + 4]);
    message_1b(env, mon);
    if env.call(0x021C_88E4, &mut [Arg::P]) != 2 {
        env.call(0x021C_5750, &mut [Arg::P, Arg::V(mon)]);
    }
    cmd_queue::set_word(&mut w.mem[OFS_QUEUE_WORD..OFS_QUEUE_WORD + 4], old);
}

// ---- 70e4 .. 7c24 ----

/// C: `FUN_overlay_d_93__021c70e4`: puts command 0x4C / 0x4D / 0x4E on the queue `q` (the object at `work + 0xC`):
/// the kind is 0 when `c` and `b` are both 6, 1 when only `c` is 6, else 2; with `flag` set the command is written
/// in place at `pos` (`cd568`), else appended (`cd4b0`).
#[allow(clippy::too_many_arguments)]
pub fn emit_move_command<E: FlowEnv>(
    env: &mut E,
    q: &mut [u8],
    scratch: &mut [u8],
    a: u32,
    b: u32,
    c: u32,
    flag: u32,
    pos: u32,
) {
    let kind = if c == 6 {
        if b == 6 {
            0
        } else {
            1
        }
    } else {
        2
    };
    let (cmd, args): (u32, Vec<u32>) = match kind {
        0 => (0x4C, vec![a]),
        1 => (0x4D, vec![b, a]),
        _ => (0x4E, vec![b, c, a]),
    };
    if flag != 0 {
        cmd_queue::put_cmd_at(env, q, pos & 0xFFFF, cmd, &args, scratch);
    } else {
        cmd_queue::put_cmd(env, q, cmd, &args, scratch);
    }
}

/// C: `FUN_overlay_d_93__021c7194`: message 0x306F for `mon` with `value` (slot 0x17); returns the reply 0x51.
pub fn message_306f<E: FlowEnv>(env: &mut E, mon: u32, value: u32) -> u32 {
    begin(env, 0x306F);
    let id = mon_id(env, mon);
    put(env, 2, id);
    put(env, 0x17, value);
    put_reply(env, 0x51, 0);
    send(env, 0x4F);
    let r = read(env, 0x51);
    end(env, 0x306F + 6);
    r
}

/// Port of `FUN_overlay_d_93__021c71e0`: fills the 6-byte answer `rec` for the item power `v = bd28(x, 8)`: with `v > 1`
/// message 0x3090 (replies 0x51 / 0x2A / 0x42) and returns true, else `rec = {1, _, 0, 0}` and false.
pub fn power_message<E: FlowEnv>(env: &mut E, mon: u32, x: u32, rec: &mut [u8]) -> bool {
    let v = field(env, x, 8) & 0xFF;
    rec[1] = 0;
    rec[5] = 3;
    if v > 1 {
        let t = env.call(0x021D_7BA4, &mut [Arg::V(v)]);
        begin(env, 0x3090);
        let id = mon_id(env, mon);
        put(env, 3, id);
        put(env, 0x29, v);
        put_a8(env, 0x2A, t);
        put_reply(env, 0x51, 0);
        put_reply(env, 0x42, 0);
        send(env, 0x35);
        let second = if v < 6 && read(env, 0x51) != 0 {
            rec[0] = v as u8;
            0
        } else {
            rec[0] = read(env, 0x2A) as u8;
            read(env, 0x42) as u8
        };
        rec[2] = second;
        rec[3] = 1;
        end(env, 0x30A7);
        true
    } else {
        rec[0] = 1;
        rec[2] = 0;
        rec[3] = 0;
        false
    }
}

/// Port of `FUN_overlay_d_93__021c728c`: two sends (0x37 / 0x38) of message 0x30C7 for the mons `a`, `b` and the move
/// record `rec` (u16 id at +0, byte at +6, u32 at +8); returns `7b10(reply 0x30, reply 0x31)` as a u16.
pub fn move_record_message<E: FlowEnv>(env: &mut E, a: u32, b: u32, rec: &[u8]) -> u32 {
    let c = env.call(0x0201_C1D8, &mut [Arg::V(u16_at(rec, 0))]);
    begin(env, 0x30C7);
    let ia = mon_id(env, a);
    put(env, 3, ia);
    let ib = mon_id(env, b);
    put(env, 4, ib);
    put(env, 0x12, u16_at(rec, 0));
    put(env, 0x16, rec[6] as u32);
    put(env, 0x1A, u32_at(rec, 8));
    put_reply(env, 0x30, c);
    send(env, 0x37);
    env.call(
        0x021D_7674,
        &mut [
            Arg::V(0x31),
            Arg::V(0x1000),
            Arg::V(0x29),
            Arg::V(0x20_0000),
        ],
    );
    send(env, 0x38);
    let r1 = read(env, 0x30);
    let r2 = read(env, 0x31);
    end(env, 0x30C7 + 0xD);
    scale(env, r1 & 0xFFFF, r2) & 0xFFFF
}

/// Port of `FUN_overlay_d_93__021c7330`: message 0x30E9 for `mon` (slot 4, default 1); returns the reply 0x51.
pub fn message_30e9<E: FlowEnv>(env: &mut E, mon: u32) -> u32 {
    begin(env, 0x30E9);
    let id = mon_id(env, mon);
    put(env, 4, id);
    put_reply(env, 0x51, 1);
    send(env, 0x80);
    let r = read(env, 0x51);
    end(env, 0x30E9 + 5);
    r
}

/// The BattleMon value the damage messages ask for: `58a8` when `flagged`, else `5ad8` (`alt`) or `59b4`.
fn pick_value<E: FlowEnv>(env: &mut E, mon: u32, k: u32, flagged: bool, alt: bool) -> u32 {
    let addr = if flagged {
        0x021D_58A8
    } else if alt {
        0x021D_5AD8
    } else {
        0x021D_59B4
    };
    env.call(addr, &mut [Arg::V(mon), Arg::V(k)])
}

/// Port of `FUN_overlay_d_93__021c7374`: message 0x30FE (replies 0x3B and 0x51) for the mons `a`, `b`, then message 0x3123
/// with the record `rec` and the chosen BattleMon value; returns `7b10(reply 0x33, reply 0x35)` as a u16.
pub fn damage_message<E: FlowEnv>(
    env: &mut E,
    w: &SvFlowWork,
    a: u32,
    b: u32,
    rec: &[u8],
    alt: u32,
) -> u32 {
    let k = env.call(0x0201_C1E8, &mut [Arg::V(u16_at(rec, 0))]);
    let slot = if k != 2 { 8 } else { 10 };
    begin(env, 0x30FE);
    let ia = mon_id(env, a);
    put(env, 3, ia);
    let ib = mon_id(env, b);
    put(env, 4, ib);
    put_a8(env, 0x3B, 0x1F);
    put_a8(env, 0x51, 0);
    send(env, 0x39);
    let r3b = read(env, 0x3B) & 0xFF;
    let mut mon = a;
    if r3b != 0x1F {
        let pokecon = u32_at(&w.mem[..], OFS_POKECON);
        mon = env.call(0x021B_9934, &mut [Arg::V(pokecon), Arg::V(r3b)]);
    }
    let flagged = read(env, 0x51) != 0;
    let v = pick_value(env, mon, slot, flagged, alt != 0);
    put(env, 0x12, u16_at(rec, 0));
    put(env, 0x16, rec[6] as u32);
    put(env, 0x1A, u32_at(rec, 8));
    put_a8(env, 0x33, v & 0xFFFF);
    put_reply_range(env, 0x19A, 0x20000);
    send(env, 0x3B);
    let r33 = read(env, 0x33);
    let r35 = read(env, 0x35);
    let res = scale(env, r33 & 0xFFFF, r35);
    end(env, 0x3123);
    res & 0xFFFF
}

/// Port of `FUN_overlay_d_93__021c7474`: message 0x3133 (type multiplier id 9 / 0xB; reply 0x3D may swap it), then the
/// BattleMon value of `b` scaled by 0x1800 for id 0xB under `c2ab0(work) == 4` and `5814(b, 5)`, then message 0x3157;
/// returns `7b10(reply 0x34, reply 0x35)` as a u16.
pub fn multiplier_message<E: FlowEnv>(env: &mut E, a: u32, b: u32, rec: &[u8], alt: u32) -> u32 {
    let k = env.call(0x0201_C1E8, &mut [Arg::V(u16_at(rec, 0))]);
    let mut id = if k != 2 { 9 } else { 0xB };
    let mut last = u32_at(rec, 8);
    begin(env, 0x3133);
    let ia = mon_id(env, a);
    put(env, 3, ia);
    let ib = mon_id(env, b);
    put(env, 4, ib);
    put(env, 0x3C, id);
    put_a8(env, 0x3D, 0);
    put_reply(env, 0x51, 0);
    send(env, 0x3A);
    if read(env, 0x3D) & 1 != 0 {
        id = if id == 9 { 0xB } else { 9 };
        last = if id == 9 { 1 } else { 2 };
    }
    let flagged = read(env, 0x51) & 0xFF != 0;
    end(env, 0x3142);
    let mut v = pick_value(env, b, id, flagged, alt != 0) & 0xFFFF;
    let c = env.call(0x021C_2AB0, &mut [Arg::P]);
    if c == 4 && env.call(0x021D_5814, &mut [Arg::V(b), Arg::V(5)]) != 0 && id == 0xB {
        v = scale(env, v, 0x1800) & 0xFFFF;
    }
    begin(env, 0x3157);
    let ia = mon_id(env, a);
    put(env, 3, ia);
    let ib = mon_id(env, b);
    put(env, 4, ib);
    put(env, 0x12, u16_at(rec, 0));
    put(env, 0x16, rec[6] as u32);
    put(env, 0x1A, last);
    put_a8(env, 0x34, v);
    put_reply_range(env, 0x19A, 0x20000);
    send(env, 0x3C);
    let r34 = read(env, 0x34);
    let r35 = read(env, 0x35);
    end(env, 0x3157 + 0xB);
    scale(env, r34 & 0xFFFF, r35) & 0xFFFF
}

/// Port of `FUN_overlay_d_93__021c75ec`: `5814(mon, x)` as the default of the reply 0x51 of message 0x3178 (sent 0x40);
/// then message part 0x41 with the value 0x1000 (0x1800 when that reply was non-zero); returns the reply 0x35.
pub fn boost_message<E: FlowEnv>(env: &mut E, mon: u32, x: u32) -> u32 {
    let q = env.call(0x021D_5814, &mut [Arg::V(mon), Arg::V(x)]);
    let mut t = 0x1000;
    begin(env, 0x3178);
    let id = mon_id(env, mon);
    put(env, 2, id);
    put_reply(env, 0x51, q);
    send(env, 0x40);
    let r = read(env, 0x51);
    if r != 0 {
        t = 0x1800;
    }
    put(env, 0x44, r);
    put_a8(env, 0x35, t);
    send(env, 0x41);
    let res = read(env, 0x35);
    end(env, 0x3189);
    res
}

/// Port of `FUN_overlay_d_93__021c7664`: message 0x319D for `mon` with `v = bd28(x, 10)` (replies 0x41 / 0x45 / 0x26):
/// 0 when the reply 0x41 is set and `v` non-zero; else `v` (or the reply 0x26 when `v` is 0), doubled when the
/// reply 0x45 is set.
pub fn message_319d<E: FlowEnv>(env: &mut E, x: u32, mon: u32) -> u32 {
    let v = field(env, x, 10);
    begin(env, 0x319D);
    let id = mon_id(env, mon);
    put(env, 3, id);
    put(env, 0x25, v);
    put_reply(env, 0x45, 0);
    put_reply(env, 0x41, 0);
    put_a8(env, 0x26, 0);
    send(env, 0x6C);
    let r41 = read(env, 0x41);
    let r45 = read(env, 0x45);
    let r26 = read(env, 0x26);
    end(env, 0x319D + 0xB);
    if r41 != 0 && v != 0 {
        return 0;
    }
    let mut res = if v == 0 { r26 } else { v };
    if r45 != 0 {
        res <<= 1;
    }
    res
}

/// Port of `FUN_overlay_d_93__021c76f4`: message 0x31C9 for `mon` with `x` (slot 0x26); false when the reply 0x41 is set;
/// else with a non-zero reply 0x26: true when `82b8(work)` says so, else a percent roll `78ac(100) < reply`.
pub fn message_31c9<E: FlowEnv>(env: &mut E, mon: u32, x: u32) -> bool {
    begin(env, 0x31C9);
    let id = mon_id(env, mon);
    put(env, 4, id);
    put_a8(env, 0x26, x);
    put_reply(env, 0x41, 0);
    send(env, 0x6D);
    let r26 = read(env, 0x26);
    let r41 = read(env, 0x41);
    end(env, 0x31C9 + 8);
    if r41 != 0 {
        return false;
    }
    if r26 != 0 {
        if env.call(0x021C_82B8, &mut [Arg::P]) != 0 {
            return true;
        }
        let roll = env.call(0x021D_78AC, &mut [Arg::V(100)]);
        return roll < (r26 & 0xFF);
    }
    false
}

/// Port of `FUN_overlay_d_93__021c7780`: message 0x31EA (command 0x6E) for `mon`.
pub fn message_31ea<E: FlowEnv>(env: &mut E, mon: u32) {
    begin(env, 0x31EA);
    let id = mon_id(env, mon);
    put_a8(env, 2, id);
    send(env, 0x6E);
    end(env, 0x31EA + 3);
}

/// Port of `FUN_overlay_d_93__021c77b0`: message 0x3201 (command 0x5B) for `mon` with the values `a` (0x1F), `b` (3),
/// `c` (0x20) and `d` (0x19); true when the reply 0x41 is 0.
pub fn message_3201<E: FlowEnv>(env: &mut E, mon: u32, a: u32, b: u32, c: u32, d: u32) -> bool {
    begin(env, 0x3201);
    let id = mon_id(env, mon);
    put(env, 2, id);
    put(env, 3, b);
    put(env, 0x1F, a);
    put(env, 0x20, c);
    put(env, 0x19, d);
    put_reply(env, 0x41, 0);
    send(env, 0x5B);
    let r = read(env, 0x41);
    end(env, 0x3201 + 9);
    r == 0
}

/// Port of `FUN_overlay_d_93__021c7820`: message 0x321A (command 0x5C) for `mon` with `x` (slot 0x19).
pub fn message_321a<E: FlowEnv>(env: &mut E, mon: u32, x: u32) {
    begin(env, 0x321A);
    let id = mon_id(env, mon);
    put(env, 2, id);
    put(env, 0x19, x);
    send(env, 0x5C);
    end(env, 0x321A + 4);
}

/// Port of `FUN_overlay_d_93__021c785c`: message 0x322E (command 0x5D): `raw` as argument 3 (a raw value), the id of `mon`
/// as argument 2, then `a` (0x1F) and `b` (0x20).
pub fn message_322e<E: FlowEnv>(env: &mut E, raw: u32, mon: u32, a: u32, b: u32) {
    begin(env, 0x322E);
    put(env, 3, raw);
    let id = mon_id(env, mon);
    put(env, 2, id);
    put(env, 0x1F, a);
    put(env, 0x20, b);
    send(env, 0x5D);
    end(env, 0x322E + 6);
}

/// Port of `FUN_overlay_d_93__021c78a8`: message 0x3244 (command 0x5E) for `mon` with `x` (0x12), `a` (0x1F), `b` (0x20).
pub fn message_3244<E: FlowEnv>(env: &mut E, mon: u32, x: u32, a: u32, b: u32) {
    begin(env, 0x3244);
    let id = mon_id(env, mon);
    put(env, 3, id);
    put(env, 0x12, x);
    put(env, 0x1F, a);
    put(env, 0x20, b);
    send(env, 0x5E);
    end(env, 0x3244 + 6);
}

/// Port of `FUN_overlay_d_93__021c78f4`: messages 0x8C / 0x8D (0x325E) for `mon` and `other` (id 0x1F when 0) around
/// `7b2c(reply 0x20, reply 0x35)`; returns the final reply 0x20 as a u16.
pub fn message_325e<E: FlowEnv>(env: &mut E, mon: u32, other: u32, x: u32) -> u32 {
    let oid = if other == 0 {
        0x1F
    } else {
        mon_id(env, other) & 0xFF
    };
    begin(env, 0x325E);
    let id = mon_id(env, mon);
    put(env, 3, id);
    put(env, 4, oid);
    put_reply_range(env, 0x19A, 0x20000);
    put_a8(env, 0x20, x);
    send(env, 0x8C);
    let r20 = read(env, 0x20) & 0xFFFF;
    let r35 = read(env, 0x35);
    let mut v = 0;
    if r20 != 0 {
        v = env.call(0x021D_7B2C, &mut [Arg::V(r20), Arg::V(r35)]) & 0xFFFF;
    }
    env.call(0x021D_76BC, &mut [Arg::V(0x20), Arg::V(v)]);
    send(env, 0x8D);
    let r = read(env, 0x20) & 0xFFFF;
    end(env, 0x326F);
    r
}

/// Port of `FUN_overlay_d_93__021c7998`: message 0x3282 with `x` (slot 0x39): sends 0x7D, and 0x7E when the reply 0x41 is 0.
pub fn message_3282<E: FlowEnv>(env: &mut E, x: u32) {
    begin(env, 0x3282);
    put(env, 0x39, x);
    put_reply(env, 0x41, 0);
    send(env, 0x7D);
    if read(env, 0x41) == 0 {
        send(env, 0x7E);
    }
    end(env, 0x328A);
}

/// Port of `FUN_overlay_d_93__021c79e0`: `v = bd28(x, 0x1A)`; message 0x329C for `mon` (reply 0x35); the value `59b4(mon, 0xE)`
/// is scaled by `7b3c(.., v)` or, with a reply, `7b10(.., reply)`; at least 1 and at most the value.
pub fn message_329c<E: FlowEnv>(env: &mut E, x: u32, mon: u32) -> u32 {
    let v = field(env, x, 0x1A);
    begin(env, 0x329C);
    let id = mon_id(env, mon);
    put(env, 2, id);
    put_a8(env, 0x35, 0);
    send(env, 0x8F);
    let r35 = read(env, 0x35);
    end(env, 0x329C + 5);
    let value = mon_value(env, mon, 0xE);
    let mut r = if r35 == 0 {
        env.call(0x021D_7B3C, &mut [Arg::V(value), Arg::V(v)])
    } else {
        scale(env, value, r35)
    };
    if r == 0 {
        return 1;
    }
    if value < r {
        r = value;
    }
    r
}

/// C: `FUN_overlay_d_93__021c7a54`: message 0x32C4 round trip: argument 2 = the id of `mon`, 0x2D = `value`,
/// sent with 0x9A; returns the reply in slot 0x41.
pub fn message_9a<E: FlowEnv>(env: &mut E, mon: u32, value: u32) -> u32 {
    begin(env, 0x32C4);
    let id = mon_id(env, mon);
    put(env, 2, id);
    put(env, 0x2D, value);
    put_reply(env, 0x41, 0);
    send(env, 0x9A);
    let reply = read(env, 0x41);
    end(env, 0x32C4 + 6);
    reply
}

/// C: `FUN_overlay_d_93__021c7aa0`: message 0x32D9 (command 0x9B) for `mon`.
pub fn message_32d9<E: FlowEnv>(env: &mut E, mon: u32) {
    begin(env, 0x32D9);
    let id = mon_id(env, mon);
    put(env, 2, id);
    send(env, 0x9B);
    end(env, 0x32D9 + 3);
}

/// Port of `FUN_overlay_d_93__021c7ad0`: message 0x32E9 (command 0x9C) for `mon` with `x` (slot 0x2D).
pub fn message_32e9<E: FlowEnv>(env: &mut E, mon: u32, x: u32) {
    begin(env, 0x32E9);
    let id = mon_id(env, mon);
    put(env, 2, id);
    put(env, 0x2D, x);
    send(env, 0x9C);
    end(env, 0x32E9 + 4);
}

/// Port of `FUN_overlay_d_93__021c7b0c`: message 0x32FA (command 0x9D) for `mon`.
pub fn message_32fa<E: FlowEnv>(env: &mut E, mon: u32) {
    begin(env, 0x32FA);
    let id = mon_id(env, mon);
    put(env, 2, id);
    send(env, 0x9D);
    end(env, 0x32FA + 3);
}

/// Port of `FUN_overlay_d_93__021c7b3c`: message 0x330C (command 0x89) with the raw values `a` (2), `b` (0xF), `c` (0x10).
pub fn message_330c<E: FlowEnv>(env: &mut E, a: u32, b: u32, c: u32) {
    begin(env, 0x330C);
    put(env, 2, a);
    put(env, 0xF, b);
    put(env, 0x10, c);
    send(env, 0x89);
    end(env, 0x330C + 5);
}

/// Port of `FUN_overlay_d_93__021c7b7c`: message 0x331D (command 0x8A) with the raw value `a` (2).
pub fn message_331d<E: FlowEnv>(env: &mut E, a: u32) {
    begin(env, 0x331D);
    put(env, 2, a);
    send(env, 0x8A);
    end(env, 0x331D + 3);
}

/// C: `FUN_overlay_d_93__021c7ba8`: message 0x332E round trip, sent with 0x9F: ids `a` (slot 2), `c` (0x52) and `b`
/// (0x53) and the word `*word` (slot 0x1E); the reply in slot 0x1E replaces `*word`.
pub fn message_9f<E: FlowEnv>(env: &mut E, a: u32, b: u32, c: u32, word: &mut [u8]) {
    let w = u32::from_le_bytes(word[..4].try_into().unwrap());
    begin(env, 0x332E);
    put(env, 2, a);
    put(env, 0x52, c);
    put(env, 0x53, b);
    put_a8(env, 0x1E, w);
    send(env, 0x9F);
    let reply = read(env, 0x1E);
    word[..4].copy_from_slice(&reply.to_le_bytes());
    end(env, 0x332E + 7);
}

/// C: `FUN_overlay_d_93__021c7bf8`: message 0x3343 (command 0x79), no arguments.
pub fn message_3343<E: FlowEnv>(env: &mut E) {
    begin(env, 0x3343);
    send(env, 0x79);
    end(env, 0x3343 + 2);
}

/// Port of `FUN_overlay_d_93__021c7c18`: `MI_CpuFill8(buf, 0, 0xC)`.
pub fn clear_12(buf: &mut [u8]) {
    buf[..12].fill(0);
}

/// Port of `FUN_overlay_d_93__021c7c24`: bumps one of six u16 counters in `counts` (saturating at 9999): the mons `a`, `b`
/// (ids from `5440`, then `8d08` of each and `9a24(a, b)`) and the signed selector `sel` choose the counter.
pub fn count_statistic<E: FlowEnv>(env: &mut E, counts: &mut [u8], a: u32, b: u32, sel: i32) {
    let ia = mon_id(env, a);
    let ib = mon_id(env, b);
    let i1 = env.call(0x021B_8D08, &mut [Arg::V(ia)]);
    let i2 = env.call(0x021B_8D08, &mut [Arg::V(ib)]);
    let i3 = env.call(0x021B_9A24, &mut [Arg::V(ia), Arg::V(ib)]);
    let mut target = None;
    if i1 == 0 {
        if i3 == 0 {
            target = Some(0);
            if sel != 0 {
                target = if sel < 4 {
                    if sel < 3 {
                        Some(2)
                    } else {
                        None
                    }
                } else {
                    Some(1)
                };
            }
        }
    } else if i2 == 0 && i3 == 0 {
        target = if sel == 0 {
            Some(3)
        } else if sel < 4 {
            if sel < 3 {
                Some(5)
            } else {
                None
            }
        } else {
            Some(4)
        };
    }
    if let Some(i) = target {
        let cur = u16::from_le_bytes([counts[2 * i], counts[2 * i + 1]]);
        if cur < 9999 {
            counts[2 * i..2 * i + 2].copy_from_slice(&(cur + 1).to_le_bytes());
        }
    }
}

#[cfg(test)]
mod tests;
