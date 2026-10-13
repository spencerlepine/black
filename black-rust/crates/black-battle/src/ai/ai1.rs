//! Overlay 96 trainer-AI functions (0x02187F00-0x0218A574, `tr_ai.c`; B14). Specs:
//! `tools/rom-diff/svflow_specs/ai1.spec` (`nowork`, `set battle+96`).
//!
//! Every function works on [`Mem`] with the AI object `ai`, the script VM `vm` and the global AI table at
//! [`AI_TABLE_PTR`] given as addresses. Command handlers take `(vm, ai)`, read their script operands through the VM
//! (`0201134c`, [`op`]), branch through `020113b0` ([`jump`], target = script base `vm[0x14]` + operand) and return the
//! status word `ai[0xC4]`. Callees outside the overlay go through [`FlowEnv`] / [`AiEnv`]; callees inside it are called
//! directly. The meaning of most commands is inferred from the BattleMon accessors they call (UNVERIFIED names).

use super::{AiEnv, Mem, INIT_RECORD};
use crate::server_flow::env::{Arg, FlowEnv};

/// Functions of this area that are ported and checked by `tests::svflow_match_rom`.
pub const PORTED: &[u32] = &[
    0x0218_7F00,
    0x0218_7FB0,
    0x0218_8080,
    0x0218_80BC,
    0x0218_8134,
    0x0218_8140,
    0x0218_814C,
    0x0218_8284,
    0x0218_85BC,
    0x0218_871C,
    0x0218_872C,
    0x0218_873C,
    0x0218_874C,
    0x0218_875C,
    0x0218_878C,
    0x0218_87B4,
    0x0218_87C4,
    0x0218_87D4,
    0x0218_87E4,
    0x0218_87F4,
    0x0218_883C,
    0x0218_884C,
    0x0218_885C,
    0x0218_889C,
    0x0218_88AC,
    0x0218_88BC,
    0x0218_8904,
    0x0218_8914,
    0x0218_8924,
    0x0218_8980,
    0x0218_8990,
    0x0218_89A0,
    0x0218_89E8,
    0x0218_89F8,
    0x0218_8A08,
    0x0218_8A54,
    0x0218_8A64,
    0x0218_8A74,
    0x0218_8A84,
    0x0218_8A94,
    0x0218_8AA4,
    0x0218_8AB4,
    0x0218_8AD8,
    0x0218_8AE8,
    0x0218_8AF8,
    0x0218_8B1C,
    0x0218_8B64,
    0x0218_8BB0,
    0x0218_8C1C,
    0x0218_8C88,
    0x0218_8CA0,
    0x0218_8D48,
    0x0218_8D5C,
    0x0218_8E00,
    0x0218_8E24,
    0x0218_8E90,
    0x0218_8F08,
    0x0218_8F14,
    0x0218_8F2C,
    0x0218_8F50,
    0x0218_8F58,
    0x0218_8FAC,
    0x0218_8FBC,
    0x0218_8FCC,
    0x0218_905C,
    0x0218_906C,
    0x0218_907C,
    0x0218_908C,
    0x0218_90C0,
    0x0218_90D0,
    0x0218_90E0,
    0x0218_90F0,
    0x0218_9100,
    0x0218_9150,
    0x0218_9160,
    0x0218_9170,
    0x0218_91D4,
    0x0218_91E4,
    0x0218_91F4,
    0x0218_92D8,
    0x0218_92E8,
    0x0218_92F8,
    0x0218_93AC,
    0x0218_93B4,
    0x0218_93CC,
    0x0218_93D4,
    0x0218_93DC,
    0x0218_9400,
    0x0218_9434,
    0x0218_945C,
    0x0218_948C,
    0x0218_94B4,
    0x0218_94C4,
    0x0218_94D4,
    0x0218_94F8,
    0x0218_9500,
    0x0218_9524,
    0x0218_9548,
    0x0218_9590,
    0x0218_95AC,
    0x0218_95C4,
    0x0218_9620,
    0x0218_9630,
    0x0218_9640,
    0x0218_9668,
    0x0218_9694,
    0x0218_96E0,
    0x0218_9718,
    0x0218_9758,
    0x0218_979C,
    0x0218_97CC,
    0x0218_9800,
    0x0218_989C,
    0x0218_9978,
    0x0218_99C4,
    0x0218_99DC,
    0x0218_9A2C,
    0x0218_9A44,
    0x0218_9A6C,
    0x0218_9A9C,
    0x0218_9AC0,
    0x0218_9B78,
    0x0218_9BFC,
    0x0218_9C8C,
    0x0218_9CEC,
    0x0218_9D30,
    0x0218_9D38,
    0x0218_9D40,
    0x0218_9D48,
    0x0218_9E70,
    0x0218_9E80,
    0x0218_9E90,
    0x0218_9ED0,
    0x0218_9EF8,
    0x0218_9F34,
    0x0218_9F58,
    0x0218_9F68,
    0x0218_9F78,
    0x0218_9F88,
    0x0218_9F98,
    0x0218_9FC0,
    0x0218_A034,
    0x0218_A074,
    0x0218_A084,
    0x0218_A094,
    0x0218_A0A4,
    0x0218_A0F4,
    0x0218_A160,
    0x0218_A220,
    0x0218_A2FC,
    0x0218_A308,
    0x0218_A310,
    0x0218_A324,
    0x0218_A378,
    0x0218_A384,
    0x0218_A390,
    0x0218_A3F4,
    0x0218_A41C,
    0x0218_A458,
    0x0218_A494,
    0x0218_A528,
];

/// AI object: status word (the handler return value: 0 continue, 1 yield).
const STATUS: u32 = 0xC4;
/// AI object: scratch result word most commands write before a compare command reads it.
const RESULT: u32 = 0x38;
/// AI object: flags (bit 0 running, bit 1 abort, bit 3 single move, bit 4 done).
const FLAGS: u32 = 0x44;
/// AI object: own / iterated battle position.
const OWN_POS: u32 = 0x94;
const ITER_POS: u32 = 0x95;
/// AI object: battle kind (0 single, 1 double, 2 triple, 3 rotation; UNVERIFIED).
const KIND: u32 = 0x98;
/// AI object: main module M / pokecon-flow / setup; self and target BattleMon.
const MAIN: u32 = 0xB0;
const FLOW: u32 = 0xB4;
const SETUP: u32 = 0xB8;
const SELF_MON: u32 = 0xBC;
const TARGET_MON: u32 = 0xC0;
/// VM: script base pointer (jump targets and tables are script-relative).
const VM_BASE: u32 = 0x14;
/// C: `0x0218A7C0`, the BSS word holding the global AI table (0x18 bytes: u8 kind[4], u8 refcount[4], ptr[4]).
pub const AI_TABLE_PTR: u32 = 0x0218_A7C0;

// ---- callees outside the overlay ----------------------------------------------------------------------------------------

/// Env wrapper: callee with plain-value arguments.
fn call<E: FlowEnv>(env: &mut E, addr: u32, args: &[u32]) -> u32 {
    let mut a: Vec<Arg<'_>> = args.iter().map(|&v| Arg::V(v)).collect();
    env.call(addr, &mut a)
}

/// Env wrapper: BattleMon accessor `addr(bpp)` (the stub ignores the pointer value).
fn bp<E: FlowEnv>(env: &mut E, addr: u32) -> u32 {
    env.call(addr, &mut [Arg::P])
}

/// Env wrapper: BattleMon accessor `addr(bpp, x)`.
fn bpv<E: FlowEnv>(env: &mut E, addr: u32, x: u32) -> u32 {
    env.call(addr, &mut [Arg::P, Arg::V(x)])
}

/// Env wrapper: `021d59b4(bpp, id)` (BattleMon value by id; both registers are logged, Z21).
fn mon_value<E: FlowEnv>(env: &mut E, bpp: u32, id: u32) -> u32 {
    call(env, 0x021D_59B4, &[bpp, id])
}

/// Env wrapper: `021d5448(bpp)`, the number of moves (0..4).
fn move_count<E: FlowEnv>(env: &mut E, _bpp: u32) -> u32 {
    bp(env, 0x021D_5448)
}

/// Env wrapper: `021d54c0(bpp, slot)`, the move id in a slot.
fn move_at<E: FlowEnv>(env: &mut E, _bpp: u32, slot: u32) -> u32 {
    bpv(env, 0x021D_54C0, slot & 0xFF)
}

/// Env wrapper: `021d5440(bpp)`, the BattleMon's mon id.
fn mon_id<E: FlowEnv>(env: &mut E, _bpp: u32) -> u32 {
    bp(env, 0x021D_5440)
}

/// Env wrapper: `021d5b68(bpp)`, nonzero when fainted (UNVERIFIED).
fn fainted<E: FlowEnv>(env: &mut E, bpp: u32) -> u32 {
    call(env, 0x021D_5B68, &[bpp])
}

/// Env wrapper: `021c7e14(flow, atk_id, def_id, waza, 1, x)` (damage-like score; UNVERIFIED).
#[allow(clippy::too_many_arguments)]
fn score7e14<E: FlowEnv>(
    env: &mut E,
    _flow: u32,
    a: u32,
    b: u32,
    waza: u32,
    one: u32,
    x: u32,
) -> u32 {
    env.call(
        0x021C_7E14,
        &mut [
            Arg::P,
            Arg::V(a),
            Arg::V(b),
            Arg::V(waza),
            Arg::V(one),
            Arg::V(x),
        ],
    )
}

/// Env wrapper: `021c7dac(flow, atk_id, def_id, waza)` (type-effectiveness class; UNVERIFIED).
fn eff7dac<E: FlowEnv>(env: &mut E, _flow: u32, a: u32, b: u32, waza: u32) -> u32 {
    env.call(
        0x021C_7DAC,
        &mut [Arg::P, Arg::V(a), Arg::V(b), Arg::V(waza)],
    )
}

/// Env wrapper: `0203f0a8()`, the battle RNG (u32).
fn rand<E: FlowEnv>(env: &mut E) -> u32 {
    call(env, 0x0203_F0A8, &[])
}

/// `(rand * n) >> 32` through `0209c054` (64-bit multiply, high word); `rand` itself when `n` is 0.
fn rand_below(r: u32, n: u32) -> u32 {
    if n == 0 {
        r
    } else {
        ((u64::from(r) * u64::from(n)) >> 32) as u32
    }
}

/// Env wrapper: `0201134c(vm)`, the next u32 script operand.
fn op<E: FlowEnv>(env: &mut E) -> u32 {
    env.call(0x0201_134C, &mut [Arg::P])
}

/// `020113b0(vm, vm[0x14] + off)`: set the script PC (a taken branch).
fn jump<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, off: u32) {
    let t = m.r32(vm + VM_BASE).wrapping_add(off);
    env.call(0x0201_13B0, &mut [Arg::P, Arg::V(t)]);
}

/// The party / client helpers of the main module: `021b8de4(M, pos)` client of a position, `021b9af8(setup, cl)` party,
/// `021b99b8(M, cl)` first party index (UNVERIFIED roles).
fn party_of<E: FlowEnv>(env: &mut E, m: &Mem, ai: u32, pos: u32) -> (u32, u32) {
    let cl = env.call(0x021B_8DE4, &mut [Arg::P, Arg::V(pos)]);
    let party = call(env, 0x021B_9AF8, &[m.r32(ai + SETUP), cl]);
    let first = env.call(0x021B_99B8, &mut [Arg::P, Arg::V(cl)]);
    (party, first)
}

/// Env wrapper: `021b9b94(party)`, party size.
fn party_len<E: FlowEnv>(env: &mut E) -> u32 {
    bp(env, 0x021B_9B94)
}

fn status(m: &Mem, ai: u32) -> u32 {
    m.r32(ai + STATUS)
}

// ---- shared helpers -----------------------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_96__0218a0f4`: compare `a` with `b` per `kind` (0 `<`, 1 `>`, 2 `==`, 3 `!=`, 4 `a & b != 0`,
/// 5 `a & b == 0`, 6 `<=`, 7 `>=`; signed; other kinds false) and branch to script offset `target` when true; returns the
/// result (0/1).
pub fn branch_if<E: FlowEnv>(
    env: &mut E,
    m: &Mem,
    vm: u32,
    kind: u32,
    a: u32,
    b: u32,
    target: u32,
) -> u32 {
    let (sa, sb) = (a as i32, b as i32);
    let t = match kind {
        0 => sa < sb,
        1 => sa > sb,
        2 => a == b,
        3 => a != b,
        4 => a & b != 0,
        5 => a & b == 0,
        6 => sa <= sb,
        7 => sa >= sb,
        _ => false,
    };
    if t {
        jump(env, m, vm, target);
    }
    u32::from(t)
}

/// `side_pos` of [`pos_of`] slots 2/3: the position in front of `p` per battle kind (0 / 3 itself, 1 `p ^ 2`, 2 via
/// `021b8c38(p, …)`); any other kind returns the AI object address (the original leaves `r0` = `ai`).
fn front_pos<E: FlowEnv>(env: &mut E, m: &Mem, ai: u32, p: u32) -> u32 {
    match m.r32(ai + KIND) {
        0 | 3 => p,
        1 => (p ^ 2) & 0xFF,
        2 => {
            let h = p >> 1;
            let q = if h == 2 { 1 } else { h + 1 };
            call(env, 0x021B_8C38, &[p, q & 0xFF])
        }
        _ => ai,
    }
}

/// C: `FUN_overlay_d_96__0218a160`: battle position of a script slot: 1 = own (`ai[0x94]`), 2 = the one in front of the
/// iterated position, 3 = the one in front of the own position, anything else = the iterated position (`ai[0x95]`).
pub fn pos_of<E: FlowEnv>(env: &mut E, m: &Mem, ai: u32, slot: u32) -> u32 {
    match slot {
        1 => m.r8(ai + OWN_POS),
        2 => front_pos(env, m, ai, m.r8(ai + ITER_POS)),
        3 => front_pos(env, m, ai, m.r8(ai + OWN_POS)),
        _ => m.r8(ai + ITER_POS),
    }
}

/// C: `FUN_overlay_d_96__0218a2fc`: BattleMon at a position, `021b98ac(ai[0xB8], pos)` (tail call).
pub fn mon_at<E: FlowEnv>(env: &mut E, _m: &Mem, _ai: u32, pos: u32) -> u32 {
    env.call(0x021B_98AC, &mut [Arg::P, Arg::V(pos)])
}

/// C: `FUN_overlay_d_96__0218a308`: party member `i`, `021b9c00(party, i)` (tail call).
pub fn party_mon<E: FlowEnv>(env: &mut E, _party: u32, i: u32) -> u32 {
    env.call(0x021B_9C00, &mut [Arg::P, Arg::V(i)])
}

/// C: `FUN_overlay_d_96__0218a310`: copy the 4 base scores `ai[0x14..]` to the slot scores `ai[0x04..]`; returns `ai`
/// (the original leaves `r0` untouched).
pub fn reset_scores(m: &mut Mem, ai: u32) -> u32 {
    for i in 0..4 {
        let v = m.r32(ai + 0x14 + 4 * i);
        m.w32(ai + 4 + 4 * i, v);
    }
    ai
}

/// C: `FUN_overlay_d_96__0218a324`: remember the iterated mon's last move (`021d67a8`) in its 4-entry table
/// `ai[0x4C + 8 * pos]` (first free entry, unless already there); returns the last `021d67a8` result.
/// (ai1.spec bounds `ai[0x95]` to a position 0..5; Z21.)
pub fn note_last_move<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    mon_at(env, m, ai, m.r8(ai + ITER_POS));
    let mut last = 0;
    for i in 0..4 {
        let e = ai + 0x4C + 8 * m.r8(ai + ITER_POS) + 2 * i;
        let w = m.r16(e);
        last = bp(env, 0x021D_67A8);
        if w == last {
            return last;
        }
        if w == 0 {
            last = bp(env, 0x021D_67A8);
            let e = ai + 0x4C + 8 * m.r8(ai + ITER_POS) + 2 * i;
            m.w16(e, last);
            return last;
        }
    }
    last
}

/// C: `FUN_overlay_d_96__0218a378`: move-data field `0201bd28(waza, id)` (tail call).
pub fn waza_param<E: FlowEnv>(env: &mut E, _ai: u32, waza: u32, id: u32) -> u32 {
    call(env, 0x0201_BD28, &[waza, id])
}

/// C: `FUN_overlay_d_96__0218a384`: move flag `0201beb8(waza, id)` (tail call).
pub fn waza_flag<E: FlowEnv>(env: &mut E, _ai: u32, waza: u32, id: u32) -> u32 {
    call(env, 0x0201_BEB8, &[waza, id])
}

/// C: `FUN_overlay_d_96__0218a390`: best `021c7e14` score of `atk`'s moves against `def` (unsigned max, 0 if none).
pub fn best_score<E: FlowEnv>(env: &mut E, m: &Mem, ai: u32, atk: u32, def: u32, x: u32) -> u32 {
    let mut best = 0;
    let ia = mon_id(env, atk);
    let ib = mon_id(env, def);
    let mut i = 0;
    while (i as i32) < move_count(env, atk) as i32 {
        let w = move_at(env, atk, i);
        let v = score7e14(env, m.r32(ai + FLOW), ia, ib, w, 1, x);
        if v > best {
            best = v;
        }
        i += 1;
    }
    best
}

/// C: `FUN_overlay_d_96__0218a3f4`: look up `(a, b)` in a table opened from `ai[0xA8]` (`02020f20` open,
/// `02021014` read, `020307b0` free); returns the read value.
pub fn table_lookup<E: FlowEnv>(env: &mut E, m: &Mem, ai: u32, a: u32, b: u32) -> u32 {
    let t = call(env, 0x0202_0F20, &[m.r32(ai + 0xA8), a]);
    let x = env.call(0x0202_1014, &mut [Arg::P, Arg::V(b)]);
    env.call(0x0203_07B0, &mut [Arg::P]);
    let _ = t;
    x
}

/// C: `FUN_overlay_d_96__0218a41c`: create the global AI table once (`02030734(heap, 0x18, 1, "tr_ai.c", 0xF9F)`, kinds
/// set to 0xFF); returns `heap` when it already exists (the original leaves `r0`), else the table address.
pub fn table_create<E: FlowEnv>(env: &mut E, m: &mut Mem, heap: u32) -> u32 {
    if m.r32(AI_TABLE_PTR) != 0 {
        return heap;
    }
    let p = call(env, 0x0203_0734, &[heap, 0x18, 1, 0x0218_A7A0, 0xF9F]);
    m.w32(AI_TABLE_PTR, p);
    for i in 0..4 {
        m.w8(m.r32(AI_TABLE_PTR) + i, 0xFF);
    }
    m.r32(AI_TABLE_PTR)
}

/// C: `FUN_overlay_d_96__0218a458`: free the global AI table and its 4 entries; returns 0.
pub fn table_free<E: FlowEnv>(env: &mut E, m: &mut Mem) -> u32 {
    if m.r32(AI_TABLE_PTR) == 0 {
        return 0;
    }
    for i in 0..4 {
        let e = m.r32(AI_TABLE_PTR) + 4 * i + 8;
        if m.r32(e) != 0 {
            env.call(0x0203_07B0, &mut [Arg::P]);
            let e = m.r32(AI_TABLE_PTR) + 4 * i + 8;
            m.w32(e, 0);
        }
    }
    env.call(0x0203_07B0, &mut [Arg::P]);
    m.w32(AI_TABLE_PTR, 0);
    0
}

/// C: `FUN_overlay_d_96__0218a494`: get (and reference) the AI script entry of kind `ai[0x45]` in the global table,
/// loading it with `02049250(ai[0xA4], kind, heap)` into the first free entry when missing (heap | 0x8000 for kind 2).
pub fn script_acquire<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let g = m.r32(AI_TABLE_PTR);
    let k = m.r8(ai + 0x45);
    for i in 0..4 {
        if k == m.r8(g + i) {
            let c = m.r8(g + 4 + i);
            m.w8(g + 4 + i, c + 1);
            return m.r32(m.r32(AI_TABLE_PTR) + 4 * i + 8);
        }
    }
    let mut i = 0;
    while i < 4 && m.r8(g + i) != 0xFF {
        i += 1;
    }
    let h = if k == 2 {
        (m.r16(ai + 0xA0) & 0x7FFF) | 0x8000
    } else {
        m.r16(ai + 0xA0)
    };
    m.w8(g + i, k);
    let g = m.r32(AI_TABLE_PTR);
    let c = m.r8(g + 4 + i);
    m.w8(g + 4 + i, c + 1);
    let x = call(
        env,
        0x0204_9250,
        &[m.r32(ai + 0xA4), m.r8(ai + 0x45), h & 0xFFFF],
    );
    let e = m.r32(AI_TABLE_PTR) + 4 * i + 8;
    m.w32(e, x);
    m.r32(m.r32(AI_TABLE_PTR) + 4 * i + 8)
}

/// C: `FUN_overlay_d_96__0218a528`: drop a reference to the script entry of kind `ai[0x45]`; the last one frees it
/// (kind back to 0xFF). Returns what the original leaves in `r0` (the kind byte / the count / the entry address).
pub fn script_release<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let g = m.r32(AI_TABLE_PTR);
    let k = m.r8(ai + 0x45);
    let mut last = 0;
    for i in 0..4 {
        last = m.r8(g + i);
        if k == last {
            let c = m.r8(g + 4 + i).wrapping_sub(1);
            m.w8(g + 4 + i, c);
            let g = m.r32(AI_TABLE_PTR);
            let c = m.r8(g + i + 4);
            if c != 0 {
                return c;
            }
            m.w8(g + i, 0xFF);
            env.call(0x0203_07B0, &mut [Arg::P]);
            let e = m.r32(AI_TABLE_PTR) + 4 * i;
            m.w32(e + 8, 0);
            return e;
        }
    }
    last
}

/// C: `FUN_overlay_d_96__0218a220`: a "held item / ability"-like value of the mon at `pos` (UNVERIFIED): 0 when
/// `021d62c4(bpp, 0x10)`; for slots 0/2 the cached `021f8afc(pos)` byte `ai[0x7C + pos]`, else, unless `021d59b4(bpp,
/// 0x10)` is 0x17/0x2A/0x47, a random nonzero one of the species' `02019c90(species, form, 0x1A..0x1C)` values; other slots
/// `021d59b4(bpp, 0x10)`. With no nonzero value the original indexes its 3-word stack array with the raw random number
/// (garbage); the port panics there.
pub fn mon_trait<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32, slot: u32, pos: u32) -> u32 {
    let bpp = mon_at(env, m, ai, pos);
    if bpv(env, 0x021D_62C4, 0x10) != 0 {
        return 0;
    }
    if slot != 0 && slot != 2 {
        return mon_value(env, bpp, 0x10);
    }
    let v = call(env, 0x021F_8AFC, &[pos]);
    m.w8(ai + 0x7C + pos, v);
    let c = m.r8(ai + 0x7C + pos);
    if c != 0 {
        return c;
    }
    let x = mon_value(env, bpp, 0x10);
    if x == 0x17 || x == 0x2A || x == 0x47 {
        return x;
    }
    let sp = bp(env, 0x021D_5444);
    let form = mon_value(env, bpp, 0x13);
    let mut list = [0u32; 3];
    let mut n = 0;
    for id in [0x1A, 0x1B, 0x1C] {
        let v = call(env, 0x0201_9C90, &[sp & 0xFFFF, form & 0xFFFF, id]);
        if v != 0 {
            list[n] = v;
            n += 1;
        }
    }
    let r = rand(env);
    list[rand_below(r, n as u32) as usize]
}

// ---- create / destroy / run ---------------------------------------------------------------------------------------------

/// C: `FUN_overlay_d_96__02187f00` (`ai_create`): allocate the 0xD8-byte AI object (`02030734(heap, 0xD8, 1, "tr_ai.c",
/// 473)`), fill its handles (`020490f4(0xAB, heap)` at +0xA4, `02020f14(heap)` at +0xA8, M / flow / setup, kind
/// `021b8588(M)`, aux `021b86b8(M)`, mask), create its VM on the init record ([`INIT_RECORD`]) with the AI object as
/// context, create the waza cache once (`0201c328(0x30, heap)` unless `0201c408()`), create the global table; returns the VM.
#[allow(clippy::too_many_arguments)]
pub fn ai_create<E: FlowEnv>(
    env: &mut E,
    m: &mut Mem,
    main: u32,
    flow: u32,
    setup: u32,
    mask: u32,
    heap: u32,
) -> u32 {
    let ai = call(env, 0x0203_0734, &[heap, 0xD8, 1, 0x0218_A7A0, 473]);
    m.w16(ai + 0xA0, heap);
    m.w32(ai + STATUS, 1);
    m.w32(ai + 0x40, mask);
    let a4 = call(env, 0x0204_90F4, &[0xAB, heap]);
    m.w32(ai + 0xA4, a4);
    let a8 = call(env, 0x0202_0F14, &[heap]);
    m.w32(ai + 0xA8, a8);
    m.w32(ai + MAIN, main);
    m.w32(ai + FLOW, flow);
    m.w32(ai + SETUP, setup);
    let k = env.call(0x021B_8588, &mut [Arg::P]);
    m.w32(ai + KIND, k);
    let x = env.call(0x021B_86B8, &mut [Arg::P]);
    m.w32(ai + 0x9C, x);
    let _ = INIT_RECORD;
    let vm = env.call(0x0201_121C, &mut [Arg::V(heap), Arg::P]);
    env.call(0x0201_126C, &mut [Arg::P, Arg::V(ai)]);
    if call(env, 0x0201_C408, &[]) == 0 {
        call(env, 0x0201_C328, &[0x30, heap]);
    }
    table_create(env, m, m.r16(ai + 0xA0));
    vm
}

/// C: `FUN_overlay_d_96__02188080` (`ai_delete`): free the AI object's handles, the object and its VM, the waza cache
/// (`0201c3c0` when `0201c408()` == 1) and the global table; returns 0.
pub fn ai_delete<E: FlowEnv>(env: &mut E, m: &mut Mem, _vm: u32) -> u32 {
    let ai = env.call(0x0201_1324, &mut [Arg::P]);
    let _ = (m.r32(ai + 0xA4), m.r32(ai + 0xA8));
    env.call(0x0204_9238, &mut [Arg::P]);
    env.call(0x0204_9238, &mut [Arg::P]);
    env.call(0x0203_07B0, &mut [Arg::P]);
    env.call(0x0201_1264, &mut [Arg::P]);
    if call(env, 0x0201_C408, &[]) == 1 {
        call(env, 0x0201_C3C0, &[]);
    }
    table_free(env, m)
}

/// C: `FUN_overlay_d_96__021880bc` (`ai_start`): reset the per-turn state of the AI object (of `vm`) for the own position
/// `pos`: base scores 0 / 100 by `usable[i]`, priorities `100 - rand >> 28`, random word `ai[0xCC] = rand >> 24`; returns
/// that word.
pub fn ai_start<E: FlowEnv>(env: &mut E, m: &mut Mem, _vm: u32, usable: u32, pos: u32) -> u32 {
    let ai = env.call(0x0201_1324, &mut [Arg::P]);
    m.w8(ai, 0);
    m.w16(ai + 2, 0);
    m.w8(ai + 1, 0);
    m.w32(ai + RESULT, 0);
    m.w8(ai + FLAGS, 0);
    m.w8(ai + 0x45, 0);
    m.w8(ai + OWN_POS, pos);
    let mask = m.r32(ai + 0x40);
    m.w32(ai + 0x3C, mask);
    m.w32(ai + 0xC8, 4);
    for i in 0..4 {
        let v = if m.r8(usable + i) == 0 { 0 } else { 100 };
        m.w32(ai + 0x14 + 4 * i, v);
        let r = rand(env);
        m.w8(ai + 0x48 + i, 100 - (r >> 28));
    }
    let r = rand(env) >> 24;
    m.w32(ai + 0xCC, r);
    r
}

/// C: `FUN_overlay_d_96__02188134`: the chosen move slot `ai[0x46]` of the AI object of `vm`.
pub fn chosen_move<E: FlowEnv>(env: &mut E, m: &Mem) -> u32 {
    let ai = env.call(0x0201_1324, &mut [Arg::P]);
    m.r8(ai + 0x46)
}

/// C: `FUN_overlay_d_96__02188140`: the chosen target position `ai[0x47]` of the AI object of `vm`.
pub fn chosen_target<E: FlowEnv>(env: &mut E, m: &Mem) -> u32 {
    let ai = env.call(0x0201_1324, &mut [Arg::P]);
    m.r8(ai + 0x47)
}

/// The script loop of [`think_single`] / [`think_multi`] over the set bits of the mask `ai[0x3C]` (one script per bit,
/// `ai[0x45]` = its kind); returns true when the time budget ran out (`ai[0x44]` bit 4).
fn run_scripts<E: AiEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32) -> bool {
    let mut mask = m.r32(ai + 0x3C);
    while mask != 0 {
        if mask & 1 != 0 {
            if m.r8(ai + FLAGS) & 0x10 == 0 {
                m.w8(ai, 0);
            }
            let f = m.r8(ai + FLAGS) & 0xEF;
            m.w8(ai + FLAGS, f);
            let t = env.call64(0x0208_70D0, &mut []);
            m.w32(ai + 0xD0, t as u32);
            m.w32(ai + 0xD4, (t >> 32) as u32);
            run_moves(env, m, vm, ai);
            if m.r8(ai + FLAGS) & 0x10 != 0 {
                return true;
            }
        }
        let n = m.r8(ai + 0x45) + 1;
        m.w8(ai + 0x45, n);
        mask = m.r32(ai + 0x3C) >> 1;
        m.w32(ai + 0x3C, mask);
        m.w8(ai + 1, 0);
    }
    false
}

/// The best-score pick of [`think_single`] / [`think_multi`]: a random slot among those with the highest score
/// `ai[4 + 4 * i]` (slots `1..4` below `021d5448(self)`); returns `(slot, best score)`.
fn pick_best_slot<E: FlowEnv>(env: &mut E, m: &Mem, ai: u32) -> (u32, u32) {
    let mut best = m.r32(ai + 4);
    let mut slots = [0u32; 4];
    let mut n = 1usize;
    for i in 1..4u32 {
        if (i as i32) < move_count(env, m.r32(ai + SELF_MON)) as i32 {
            let v = m.r32(ai + 4 + 4 * i);
            if best == v {
                slots[n] = i;
                n += 1;
            }
            if (best as i32) < (v as i32) {
                best = v;
                n = 1;
                slots[0] = i;
            }
        }
    }
    let r = rand(env);
    (slots[rand_below(r, n as u32) as usize], best)
}

/// C: `FUN_overlay_d_96__0218814c`: think for a single target (`ai[0x95]`): run the scripts, then pick the best-scoring
/// move slot (4 = abort when `ai[0x44]` bit 1); returns 1 when out of time, else 0.
pub fn think_single<E: AiEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32) -> u32 {
    let s = mon_at(env, m, ai, m.r8(ai + OWN_POS));
    m.w32(ai + SELF_MON, s);
    let t = mon_at(env, m, ai, m.r8(ai + ITER_POS));
    m.w32(ai + TARGET_MON, t);
    if m.r8(ai + FLAGS) & 0x10 == 0 {
        reset_scores(m, ai);
        note_last_move(env, m, ai);
    }
    if run_scripts(env, m, vm, ai) {
        return 1;
    }
    let slot = if m.r8(ai + FLAGS) & 2 != 0 {
        4
    } else {
        pick_best_slot(env, m, ai).0
    };
    m.w8(ai + 0x46, slot);
    let p = m.r8(ai + ITER_POS);
    m.w8(ai + 0x47, p);
    0
}

/// C: `FUN_overlay_d_96__02188284`: think over all positions (4 in a double battle, else 6): per target the best slot and
/// score (`ai[0x30 + pos]`, `ai[0x24 + 2 * pos]`; -1 for none / self / fainted), then a random target among the best
/// scores, its slot, and the target fix-ups for self-targeting (move param 0x1B) and non-adjacent (flag 0xB) moves;
/// returns 1 when out of time, else 0.
pub fn think_multi<E: AiEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32) -> u32 {
    let cnt = if m.r32(ai + KIND) == 1 { 4 } else { 6 };
    if m.r8(ai + FLAGS) & 0x10 == 0 {
        m.w8(ai + ITER_POS, 0);
    }
    while (m.r8(ai + ITER_POS) as i32) < cnt {
        'target: {
            let s = mon_at(env, m, ai, m.r8(ai + OWN_POS));
            m.w32(ai + SELF_MON, s);
            let t = mon_at(env, m, ai, m.r8(ai + ITER_POS));
            m.w32(ai + TARGET_MON, t);
            if m.r8(ai + FLAGS) & 0x10 == 0 {
                let i = m.r8(ai + ITER_POS);
                if t == 0 || i == m.r8(ai + OWN_POS) || fainted(env, t) == 1 {
                    m.w8(ai + 0x30 + i, 0xFF);
                    m.w16(ai + 0x24 + 2 * i, 0xFFFF);
                    break 'target;
                }
                reset_scores(m, ai);
                note_last_move(env, m, ai);
                m.w8(ai + 0x45, 0);
                let mask = m.r32(ai + 0x40);
                m.w8(ai + 1, 0);
                m.w32(ai + 0x3C, mask);
            }
            if run_scripts(env, m, vm, ai) {
                return 1;
            }
            if m.r8(ai + FLAGS) & 2 != 0 {
                m.w8(ai + 0x46, 4);
                break 'target;
            }
            let (slot, best) = pick_best_slot(env, m, ai);
            let i = m.r8(ai + ITER_POS);
            m.w8(ai + 0x30 + i, slot);
            m.w16(ai + 0x24 + 2 * i, best);
            let a = env.call(0x021B_8DE4, &mut [Arg::P, Arg::V(m.r8(ai + OWN_POS))]);
            let b = env.call(0x021B_8DE4, &mut [Arg::P, Arg::V(m.r8(ai + ITER_POS))]);
            if env.call(0x021B_8CE8, &mut [Arg::P, Arg::V(a), Arg::V(b)]) == 0 {
                let e = ai + 0x24 + 2 * m.r8(ai + ITER_POS);
                if (m.r16(e) as u16 as i16) < 100 {
                    m.w16(e, 0xFFFF);
                }
            }
        }
        let i = m.r8(ai + ITER_POS) + 1;
        m.w8(ai + ITER_POS, i);
    }
    let mut best = m.r16(ai + 0x24) as u16 as i16;
    let mut tg = [0u32; 8];
    let mut n = 1usize;
    let mut i = 1u32;
    while (i as i32) < cnt {
        let s = m.r16(ai + 0x24 + 2 * i) as u16 as i16;
        if best == s {
            tg[n] = i;
            n += 1;
        }
        if best < s {
            best = s;
            tg[0] = i;
            n = 1;
        }
        i = (i + 1) & 0xFF;
    }
    let r = rand(env);
    m.w8(ai + 0x47, tg[rand_below(r, n as u32) as usize]);
    let slot = m.r8(ai + 0x30 + m.r8(ai + 0x47));
    m.w8(ai + 0x46, slot);
    let w = move_at(env, m.r32(ai + SELF_MON), m.r8(ai + 0x46));
    if waza_param(env, ai, w, 0x1B) == 1 && m.r8(ai + 0x47) & 1 == 0 {
        let p = m.r8(ai + OWN_POS);
        m.w8(ai + 0x47, p);
    }
    let w = move_at(env, m.r32(ai + SELF_MON), m.r8(ai + 0x46));
    if m.r32(ai + KIND) == 2
        && call(env, 0x021B_9A6C, &[m.r8(ai + OWN_POS), m.r8(ai + 0x47)]) == 0
        && waza_flag(env, ai, w, 0xB) == 0
    {
        let p = (m.r8(ai + 0x47) & 1) + 2;
        m.w8(ai + 0x47, p);
    }
    0
}

/// C: `FUN_overlay_d_96__021885bc`: score every move slot of the current script: phase 0 loads the slot's move and the
/// script (`0218a494`, `0201127c`), phase 1 steps the VM (`02011298`) until the script ends (bit 0 of `ai[0x44]`) or
/// 500 ticks pass (sets bit 4); unusable / non-adjacent moves score 0. Returns the phase `ai[0]`.
/// The OS tick is read through [`AiEnv::call64`] (the test env answers the spec's `ticks:100` clock; Z21).
pub fn run_moves<E: AiEnv>(env: &mut E, m: &mut Mem, _vm: u32, ai: u32) -> u32 {
    while m.r8(ai) != 2 && m.r8(ai + FLAGS) & 0x10 == 0 {
        let ph = m.r8(ai);
        if ph == 0 {
            if bpv(env, 0x021D_5584, m.r8(ai + 1)) == 0 {
                m.w16(ai + 2, 0);
            } else {
                let w = move_at(env, m.r32(ai + SELF_MON), m.r8(ai + 1));
                m.w16(ai + 2, w);
                let x = script_acquire(env, m, ai);
                m.w32(ai + 0xAC, x);
                env.call(0x0201_127C, &mut [Arg::P, Arg::V(x)]);
            }
            let p = m.r8(ai) + 1;
            m.w8(ai, p);
        } else if ph != 1 {
            m.w8(ai, 2);
            continue;
        }
        // phase 1
        if m.r32(ai + KIND) == 2
            && m.r16(ai + 2) != 0
            && call(env, 0x021B_9A6C, &[m.r8(ai + OWN_POS), m.r8(ai + ITER_POS)]) == 0
            && waza_flag(env, ai, m.r16(ai + 2), 0xB) == 0
        {
            m.w16(ai + 2, 0);
            env.call(0x0201_1290, &mut [Arg::P]);
            script_release(env, m, ai);
            m.w32(ai + 0xAC, 0);
        }
        if m.r16(ai + 2) == 0 {
            m.w32(ai + 4 + 4 * m.r8(ai + 1), 0);
            let f = m.r8(ai + FLAGS) | 1;
            m.w8(ai + FLAGS, f);
        } else {
            env.call(0x0201_1298, &mut [Arg::P]);
            if m.r8(ai + FLAGS) & 1 != 0 {
                let f = m.r8(ai + FLAGS) & 0xEF;
                m.w8(ai + FLAGS, f);
                script_release(env, m, ai);
                m.w32(ai + 0xAC, 0);
            } else {
                let t = env.call64(0x0208_70D0, &mut []);
                let start = u64::from(m.r32(ai + 0xD0)) | (u64::from(m.r32(ai + 0xD4)) << 32);
                if t.wrapping_sub(start) > 500 {
                    let f = m.r8(ai + FLAGS) | 0x10;
                    m.w8(ai + FLAGS, f);
                    continue;
                }
            }
        }
        if m.r8(ai + FLAGS) & 1 != 0 {
            let s = m.r8(ai + 1) + 1;
            m.w8(ai + 1, s);
            let n = move_count(env, m.r32(ai + SELF_MON));
            let p = if m.r8(ai + 1) < n && m.r8(ai + FLAGS) & 8 == 0 {
                0
            } else {
                m.r8(ai) + 1
            };
            m.w8(ai, p);
            let f = m.r8(ai + FLAGS) & 0xFE;
            m.w8(ai + FLAGS, f);
        }
    }
    m.r8(ai)
}

/// C: `FUN_overlay_d_96__02187fb0` (`ai_think`): pick the move and target: single battles ([`think_single`] on the
/// opposite position), double / triple ([`think_multi`]), otherwise a random opposing position that is not fainted
/// (target 0 when the think yields nothing).
pub fn ai_think<E: AiEnv>(env: &mut E, m: &mut Mem, vm: u32) -> u32 {
    let ai = env.call(0x0201_1324, &mut [Arg::P]);
    let k = m.r32(ai + KIND);
    if k == 0 {
        let p = m.r8(ai + OWN_POS) ^ 1;
        m.w8(ai + ITER_POS, p);
        return think_single(env, m, vm, ai);
    }
    if k.wrapping_sub(1) <= 1 {
        return think_multi(env, m, vm, ai);
    }
    let mut cand = [0u32; 3];
    let mut n = 0usize;
    m.w8(ai + ITER_POS, 0);
    while m.r8(ai + ITER_POS) < 6 {
        let i = m.r8(ai + ITER_POS);
        let own = m.r8(ai + OWN_POS);
        if i != own && (i & 1) != (own & 1) {
            let mon = mon_at(env, m, ai, i);
            if fainted(env, mon) != 1 {
                cand[n] = m.r8(ai + ITER_POS);
                n += 1;
            }
        }
        let i = m.r8(ai + ITER_POS) + 1;
        m.w8(ai + ITER_POS, i);
    }
    let r = rand(env);
    m.w8(ai + ITER_POS, cand[rand_below(r, n as u32) as usize]);
    let x = think_single(env, m, vm, ai);
    if x == 0 {
        m.w8(ai + 0x47, 0);
    }
    x
}

// ---- command handlers (table 0x0218A5A8) --------------------------------------------------------------------------------

/// C: `FUN_overlay_d_96__0218875c`: `if (rand >> 24) <kind> a: goto b` (random-chance branch).
pub fn cmd_if_rand<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, kind: u32) -> u32 {
    let a = op(env);
    let b = op(env);
    let r = rand(env) >> 24;
    branch_if(env, m, vm, kind, r, a, b)
}

macro_rules! wrapper {
    ($(#[$d:meta])* $name:ident => $helper:ident($k:expr)) => {
        $(#[$d])*
        pub fn $name<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32) -> u32 {
            $helper(env, m, vm, ai, $k);
            status(m, ai)
        }
    };
}

fn if_rand_h<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, _ai: u32, k: u32) -> u32 {
    cmd_if_rand(env, m, vm, k)
}

wrapper!(
    /// C: `FUN_overlay_d_96__0218871c`: command 0, random branch `<`.
    cmd_00 => if_rand_h(0));
wrapper!(
    /// C: `FUN_overlay_d_96__0218872c`: command 1, random branch `>`.
    cmd_01 => if_rand_h(1));
wrapper!(
    /// C: `FUN_overlay_d_96__0218873c`: command 2, random branch `==`.
    cmd_02 => if_rand_h(2));
wrapper!(
    /// C: `FUN_overlay_d_96__0218874c`: command 3, random branch `!=`.
    cmd_03 => if_rand_h(3));

/// C: `FUN_overlay_d_96__0218878c`: command 4, add the operand to the current slot's score (clamped at 0).
pub fn cmd_add_score<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let v = op(env);
    let p = ai + 4 + 4 * m.r8(ai + 1);
    let w = m.r32(p).wrapping_add(v);
    m.w32(p, w);
    let p = ai + 4 + 4 * m.r8(ai + 1);
    if (m.r32(p) as i32) < 0 {
        m.w32(p, 0);
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__021887f4`: `if hp_ratio(slot) >> 12 <kind> a: goto b` (`021d5c44`, fixed point).
pub fn if_hp<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let a = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let bpp = mon_at(env, m, ai, pos);
    let hp = bp(env, 0x021D_5C44);
    let _ = bpp;
    branch_if(env, m, vm, kind, ((hp as i32) >> 12) as u32, a, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__021887b4`: command 5, HP ratio `<`.
    cmd_05 => if_hp(0));
wrapper!(
    /// C: `FUN_overlay_d_96__021887c4`: command 6, HP ratio `>`.
    cmd_06 => if_hp(1));
wrapper!(
    /// C: `FUN_overlay_d_96__021887d4`: command 7, HP ratio `==`.
    cmd_07 => if_hp(2));
wrapper!(
    /// C: `FUN_overlay_d_96__021887e4`: command 8, HP ratio `!=`.
    cmd_08 => if_hp(3));

/// C: `FUN_overlay_d_96__0218885c`: `if status(slot) != 0 <kind>: goto b` (`021d62a8`; kind 2 / 3 = has / has not).
pub fn if_status<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    mon_at(env, m, ai, pos);
    let x = bp(env, 0x021D_62A8);
    branch_if(env, m, vm, kind, x, 0, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__0218883c`: command 9, branch if the mon has a status condition.
    cmd_09 => if_status(3));
wrapper!(
    /// C: `FUN_overlay_d_96__0218884c`: command 10, branch if the mon has no status condition.
    cmd_10 => if_status(2));

/// C: `FUN_overlay_d_96__021888bc`: `if cond(slot, a) <kind> 1: goto b` (`021d62c4`, a condition flag).
pub fn if_cond<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let a = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    mon_at(env, m, ai, pos);
    let x = bpv(env, 0x021D_62C4, a);
    branch_if(env, m, vm, kind, x, 1, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__0218889c`: command 11, branch if the condition is set.
    cmd_11 => if_cond(2));
wrapper!(
    /// C: `FUN_overlay_d_96__021888ac`: command 12, branch if the condition is clear.
    cmd_12 => if_cond(3));

/// C: `FUN_overlay_d_96__02188924`: `if (cond(slot, 5) && 021e8a78(021d6314(bpp, 5))) <kind> 1: goto b`.
pub fn if_cond5<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    mon_at(env, m, ai, pos);
    let mut r = 0;
    if bpv(env, 0x021D_62C4, 5) != 0 {
        let x = bpv(env, 0x021D_6314, 5);
        if call(env, 0x021E_8A78, &[x]) != 0 {
            r = 1;
        }
    }
    branch_if(env, m, vm, kind, r, 1, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__02188904`: command 13.
    cmd_13 => if_cond5(2));
wrapper!(
    /// C: `FUN_overlay_d_96__02188914`: command 14.
    cmd_14 => if_cond5(3));

/// C: `FUN_overlay_d_96__021889a0`: `if 021d5bdc(slot, a) <kind> 1: goto b`.
pub fn if_5bdc<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let a = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    mon_at(env, m, ai, pos);
    let x = bpv(env, 0x021D_5BDC, a);
    branch_if(env, m, vm, kind, x, 1, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__02188980`: command 15.
    cmd_15 => if_5bdc(2));
wrapper!(
    /// C: `FUN_overlay_d_96__02188990`: command 16.
    cmd_16 => if_5bdc(3));

/// C: `FUN_overlay_d_96__02188a08`: `if (021c82d0(flow, pos(slot), a) != 0) <kind> 1: goto b` (side condition).
pub fn if_side<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let a = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let r = u32::from(call(env, 0x021C_82D0, &[m.r32(ai + FLOW), pos, a]) != 0);
    branch_if(env, m, vm, kind, r, 1, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__021889e8`: command 17.
    cmd_17 => if_side(2));
wrapper!(
    /// C: `FUN_overlay_d_96__021889f8`: command 18.
    cmd_18 => if_side(3));

/// C: `FUN_overlay_d_96__02188ab4`: `if ai[0x38] <kind> a: goto b` (compare the last result).
pub fn if_result<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let a = op(env);
    let b = op(env);
    branch_if(env, m, vm, kind, m.r32(ai + RESULT), a, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__02188a54`: command 19, result `<`.
    cmd_19 => if_result(0));
wrapper!(
    /// C: `FUN_overlay_d_96__02188a64`: command 20, result `>`.
    cmd_20 => if_result(1));
wrapper!(
    /// C: `FUN_overlay_d_96__02188a74`: commands 21 / 36, result `==`.
    cmd_21 => if_result(2));
wrapper!(
    /// C: `FUN_overlay_d_96__02188a84`: commands 22 / 37, result `!=`.
    cmd_22 => if_result(3));
wrapper!(
    /// C: `FUN_overlay_d_96__02188a94`: command 23, result `& a != 0`.
    cmd_23 => if_result(4));
wrapper!(
    /// C: `FUN_overlay_d_96__02188aa4`: command 24, result `& a == 0`.
    cmd_24 => if_result(5));

/// C: `FUN_overlay_d_96__02188af8`: `if move(ai[2]) <kind> a: goto b` (compare the current move id).
pub fn if_move<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let a = op(env);
    let b = op(env);
    branch_if(env, m, vm, kind, m.r16(ai + 2), a, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__02188ad8`: command 25, current move `==`.
    cmd_25 => if_move(2));
wrapper!(
    /// C: `FUN_overlay_d_96__02188ae8`: command 26, current move `!=`.
    cmd_26 => if_move(3));

/// Word `i` of the u32 list at script address `tbl` (two halfword reads, as the original).
fn list_word(m: &Mem, tbl: u32, i: u32) -> u32 {
    m.r16(tbl.wrapping_add(4 * i)) | (m.r16(tbl.wrapping_add(4 * i + 2)) << 16)
}

/// C: `FUN_overlay_d_96__02188b1c`: command 27, `if ai[0x38] in list(a): goto b` (script list ended by -1).
pub fn cmd_if_result_in<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32) -> u32 {
    let a = op(env);
    let tbl = m.r32(vm + VM_BASE).wrapping_add(a);
    let b = op(env);
    let x = m.r32(ai + RESULT);
    let mut i = 0;
    loop {
        let w = list_word(m, tbl, i);
        if x == w {
            jump(env, m, vm, b);
            break;
        }
        i += 1;
        if w == u32::MAX {
            break;
        }
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188b64`: command 28, `if ai[0x38] not in list(a): goto b`.
pub fn cmd_if_result_not_in<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32) -> u32 {
    let a = op(env);
    let tbl = m.r32(vm + VM_BASE).wrapping_add(a);
    let b = op(env);
    let x = m.r32(ai + RESULT);
    let mut i = 0;
    loop {
        let w = list_word(m, tbl, i);
        if x == w {
            return status(m, ai);
        }
        i += 1;
        if w == u32::MAX {
            break;
        }
    }
    jump(env, m, vm, b);
    status(m, ai)
}

/// Index of the first own move with nonzero move param 3 (`0218a378(.., 3)`, power?), or the move count.
fn first_with_param3<E: FlowEnv>(env: &mut E, m: &Mem, ai: u32) -> (u32, u32) {
    let me = m.r32(ai + SELF_MON);
    let mut i = 0u32;
    if move_count(env, me) as i32 > 0 {
        loop {
            let w = move_at(env, me, i);
            if waza_param(env, ai, w, 3) != 0 {
                break;
            }
            i += 1;
            if (i as i32) >= move_count(env, me) as i32 {
                break;
            }
        }
    }
    (i, move_count(env, me))
}

/// C: `FUN_overlay_d_96__02188bb0`: command 29, branch if the mon has a move with nonzero param 3 (a damaging move?).
pub fn cmd_if_has_damaging<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let b = op(env);
    let (i, n) = first_with_param3(env, m, ai);
    if (i as i32) < n as i32 {
        jump(env, m, vm, b);
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188c1c`: command 30, branch if the mon has no move with nonzero param 3.
pub fn cmd_if_no_damaging<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let b = op(env);
    let (i, n) = first_with_param3(env, m, ai);
    if i == n {
        jump(env, m, vm, b);
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188c88`: command 31, `ai[0x38] = 021c8138(flow)` (turn count?).
pub fn cmd_get_8138<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let x = env.call(0x021C_8138, &mut [Arg::P]);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// Env wrapper: `021d57fc(bpp)`, the packed types of a BattleMon (the callee reads only r0).
fn mon_types<E: FlowEnv>(env: &mut E, bpp: u32) -> u32 {
    call(env, 0x021D_57FC, &[bpp])
}

/// C: `FUN_overlay_d_96__02188ca0`: command 32, `ai[0x38]` = a type (`021e8cfc` first / `021e8d04` second of the packed
/// `021d57fc` types) of self (0..3), the current move param 0 (4) or the mons in front (5..8); other selectors keep it.
pub fn cmd_get_type<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let sel = op(env);
    let t1 = mon_types(env, m.r32(ai + SELF_MON));
    let t2 = mon_types(env, m.r32(ai + TARGET_MON));
    let front = |env: &mut E, m: &Mem, slot: u32| -> u32 {
        let pos = pos_of(env, m, ai, slot);
        let bpp = mon_at(env, m, ai, pos);
        mon_types(env, bpp)
    };
    let r = match sel {
        0 => call(env, 0x021E_8CFC, &[t2]),
        1 => call(env, 0x021E_8CFC, &[t1]),
        2 => call(env, 0x021E_8D04, &[t2]),
        3 => call(env, 0x021E_8D04, &[t1]),
        4 => waza_param(env, ai, m.r16(ai + 2), 0),
        5 => {
            let t = front(env, m, 2);
            call(env, 0x021E_8CFC, &[t])
        }
        6 => {
            let t = front(env, m, 3);
            call(env, 0x021E_8CFC, &[t])
        }
        7 => {
            let t = front(env, m, 2);
            call(env, 0x021E_8D04, &[t])
        }
        8 => {
            let t = front(env, m, 3);
            call(env, 0x021E_8D04, &[t])
        }
        _ => return status(m, ai),
    };
    m.w32(ai + RESULT, r);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188d48`: command 33, `ai[0x38] = 0201c1d8(move)` (move power?).
pub fn cmd_move_1d8<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let x = call(env, 0x0201_C1D8, &[m.r16(ai + 2)]);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188d5c`: command 34, compare the current move's score against the mon's other moves:
/// `ai[0x38]` = 0 (scores 0), 1 (another move scores higher), 2 (best).
pub fn cmd_cmp_scores<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let x = op(env);
    let ia = mon_id(env, m.r32(ai + SELF_MON));
    let ib = mon_id(env, m.r32(ai + TARGET_MON));
    let base = score7e14(env, m.r32(ai + FLOW), ia, ib, m.r16(ai + 2), 1, x);
    if base == 0 {
        m.w32(ai + RESULT, 0);
        return status(m, ai);
    }
    m.w32(ai + RESULT, 2);
    let me = m.r32(ai + SELF_MON);
    let mut i = 0u32;
    while (i as i32) < move_count(env, me) as i32 {
        let w = move_at(env, me, i);
        if i != m.r8(ai + 1) && score7e14(env, m.r32(ai + FLOW), ia, ib, w, 1, x) > base {
            m.w32(ai + RESULT, 1);
            break;
        }
        i += 1;
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188e00`: command 35, `ai[0x38]` = last move of the mon at a slot (`021d67a8`).
pub fn cmd_last_move<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let s = op(env);
    let pos = pos_of(env, m, ai, s);
    mon_at(env, m, ai, pos);
    let x = bp(env, 0x021D_67A8);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188e24`: command 38, compare a `021c81c8(flow, self, 1)` value with the target's under kind
/// `k` (0 -> `>`, 1 -> `<`, 2 -> `==`, others as given): speed order? (`021c81c8` is a thunk: its target gets r0..r2.)
pub fn cmd_cmp_speed<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let k = op(env);
    let b = op(env);
    let a = env.call(
        0x021C_81C8,
        &mut [Arg::P, Arg::V(m.r32(ai + SELF_MON)), Arg::V(1)],
    );
    let c = env.call(
        0x021C_81C8,
        &mut [Arg::P, Arg::V(m.r32(ai + TARGET_MON)), Arg::V(1)],
    );
    let k = match k {
        0 => 1,
        1 => 0,
        2 => 2,
        k => k,
    };
    branch_if(env, m, vm, k, a, c, b);
    status(m, ai)
}

/// Env wrapper: `021d5b7c(bpp)` (UNVERIFIED: "can act"; the callee reads only r0).
fn mon_5b7c<E: FlowEnv>(env: &mut E, bpp: u32) -> u32 {
    call(env, 0x021D_5B7C, &[bpp])
}

/// C: `FUN_overlay_d_96__02188e90`: command 39, `ai[0x38]` = number of party members (from the client's first index) with
/// `021d5b7c` set.
pub fn cmd_count_party_5b7c<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let s = op(env);
    let pos = pos_of(env, m, ai, s);
    let (party, mut i) = party_of(env, m, ai, pos);
    m.w32(ai + RESULT, 0);
    while (i as i32) < party_len(env) as i32 {
        let mon = party_mon(env, party, i & 0xFF);
        if mon_5b7c(env, mon) != 0 {
            let r = m.r32(ai + RESULT) + 1;
            m.w32(ai + RESULT, r);
        }
        i += 1;
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188f08`: command 40 "load the slot score"; `ai[0x38] = ai[2]` (u16), return `ai[0xC4]`.
pub fn cmd_result_from_score(m: &mut Mem, ai: u32) -> u32 {
    let v = m.r16(ai + 2);
    m.w32(ai + RESULT, v);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188f14`: command 41, `ai[0x38]` = param 0x1C of the current move (effect id?).
pub fn cmd_move_effect<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let x = waza_param(env, ai, m.r16(ai + 2), 0x1C);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188f2c`: command 42, `ai[0x38]` = [`mon_trait`] of a slot.
pub fn cmd_trait<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let s = op(env);
    let pos = pos_of(env, m, ai, s);
    let x = mon_trait(env, m, ai, s, pos);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188f50`: command 43, no-op; return `ai[0xC4]`.
pub fn cmd_status(m: &Mem, ai: u32) -> u32 {
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188f58`: command 44, `if eff(current move) == a: goto b` (`021c7dac`).
pub fn cmd_if_effectiveness<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let a = op(env);
    let b = op(env);
    let ia = mon_id(env, m.r32(ai + SELF_MON));
    let ib = mon_id(env, m.r32(ai + TARGET_MON));
    let x = eff7dac(env, m.r32(ai + FLOW), ia, ib, m.r16(ai + 2));
    branch_if(env, m, vm, 2, x, a, b);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02188fcc`: `if any living party member has status <kind>: goto b` (stops at the first branch);
/// returns the last branch result or party size (what the original leaves in `r0`).
pub fn if_party_status<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let (party, mut i) = party_of(env, m, ai, pos);
    loop {
        let n = party_len(env);
        if (i as i32) >= n as i32 {
            return n;
        }
        let mon = party_mon(env, party, i & 0xFF);
        if fainted(env, mon) == 0 {
            let x = bp(env, 0x021D_62A8);
            let r = branch_if(env, m, vm, kind, x, 0, b);
            if r != 0 {
                return r;
            }
        }
        i += 1;
    }
}

wrapper!(
    /// C: `FUN_overlay_d_96__02188fac`: command 45.
    cmd_45 => if_party_status(2));
wrapper!(
    /// C: `FUN_overlay_d_96__02188fbc`: command 46.
    cmd_46 => if_party_status(3));

/// C: `FUN_overlay_d_96__0218905c`: command 47, `ai[0x38] = 021ef884()`.
pub fn cmd_get_f884<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let x = call(env, 0x021E_F884, &[]);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__0218908c`: `if effect(current move) <kind> a: goto b` (move param 0x1C).
pub fn if_move_effect<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let a = op(env);
    let b = op(env);
    let x = waza_param(env, ai, m.r16(ai + 2), 0x1C);
    branch_if(env, m, vm, kind, x, a, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__0218906c`: command 48.
    cmd_48 => if_move_effect(2));
wrapper!(
    /// C: `FUN_overlay_d_96__0218907c`: command 49.
    cmd_49 => if_move_effect(3));

/// C: `FUN_overlay_d_96__02189100`: `if value(slot, id) <kind> a: goto b` (`021d59b4`).
pub fn if_value<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let id = op(env);
    let a = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let bpp = mon_at(env, m, ai, pos);
    let x = mon_value(env, bpp, id);
    branch_if(env, m, vm, kind, x, a, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__021890c0`: command 50, value `<`.
    cmd_50 => if_value(0));
wrapper!(
    /// C: `FUN_overlay_d_96__021890d0`: command 51, value `>`.
    cmd_51 => if_value(1));
wrapper!(
    /// C: `FUN_overlay_d_96__021890e0`: command 52, value `==`.
    cmd_52 => if_value(2));
wrapper!(
    /// C: `FUN_overlay_d_96__021890f0`: command 53, value `!=`.
    cmd_53 => if_value(3));

/// C: `FUN_overlay_d_96__02189170`: `if value(target, 0xD) <kind> score(current move): goto b` (first operand unused).
pub fn if_hp_vs_damage<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    op(env);
    let b = op(env);
    let x = mon_value(env, m.r32(ai + TARGET_MON), 0xD);
    let ia = mon_id(env, m.r32(ai + SELF_MON));
    let ib = mon_id(env, m.r32(ai + TARGET_MON));
    let y = score7e14(env, m.r32(ai + FLOW), ia, ib, m.r16(ai + 2), 1, 0);
    branch_if(env, m, vm, kind, x, y, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__02189150`: command 54 (kind 6 `<=`).
    cmd_54 => if_hp_vs_damage(6));
wrapper!(
    /// C: `FUN_overlay_d_96__02189160`: command 55 (kind 1 `>`).
    cmd_55 => if_hp_vs_damage(1));

/// C: `FUN_overlay_d_96__021891f4`: `if knows_move(slot, w) <kind> 1: goto b`: slot 0 looks in the remembered moves of the
/// iterated mon, 1 in the own moves, 3 in the moves of the living mon in front.
pub fn if_knows_move<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let w = op(env) & 0xFFFF;
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let mut found = 0;
    match s {
        0 => {
            for i in 0..4 {
                if w == m.r16(ai + 0x4C + 8 * pos + 2 * i) {
                    found = 1;
                    break;
                }
            }
        }
        1 => {
            let me = m.r32(ai + SELF_MON);
            let mut i = 0u32;
            while (i as i32) < move_count(env, me) as i32 {
                if move_at(env, me, i) == w {
                    found = 1;
                    break;
                }
                i += 1;
            }
        }
        3 => {
            let p = pos_of(env, m, ai, 3);
            let mon = mon_at(env, m, ai, p);
            if fainted(env, mon) == 0 {
                let mut i = 0u32;
                while (i as i32) < move_count(env, mon) as i32 {
                    if move_at(env, mon, i) == w {
                        found = 1;
                        break;
                    }
                    i += 1;
                }
            }
        }
        _ => {}
    }
    branch_if(env, m, vm, kind, found, 1, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__021891d4`: command 56.
    cmd_56 => if_knows_move(2));
wrapper!(
    /// C: `FUN_overlay_d_96__021891e4`: command 57.
    cmd_57 => if_knows_move(3));

/// C: `FUN_overlay_d_96__021892f8`: `if knows_effect(slot, e) <kind> 1: goto b` (move param 0x1C == e): slot 0 in the
/// remembered moves (nonzero entries), 1 in the own moves.
pub fn if_knows_effect<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let e = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let mut found = 0;
    match s {
        0 => {
            for i in 0..4 {
                let w = m.r16(ai + 0x4C + 8 * pos + 2 * i);
                if w != 0 && waza_param(env, ai, w, 0x1C) == e {
                    found = 1;
                    break;
                }
            }
        }
        1 => {
            let me = m.r32(ai + SELF_MON);
            let mut i = 0u32;
            while (i as i32) < move_count(env, me) as i32 {
                let w = move_at(env, me, i);
                if waza_param(env, ai, w, 0x1C) == e {
                    found = 1;
                    break;
                }
                i += 1;
            }
        }
        _ => {}
    }
    branch_if(env, m, vm, kind, found, 1, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__021892d8`: command 58.
    cmd_58 => if_knows_effect(2));
wrapper!(
    /// C: `FUN_overlay_d_96__021892e8`: command 59.
    cmd_59 => if_knows_effect(3));

/// C: `FUN_overlay_d_96__021893ac` (command 60), `021893cc` (62), `021893d4` (63), `021894f8` (72), `02189d30` (102),
/// `02189d38` (103), `02189d40` (104): no-op commands; return `ai[0xC4]`.
pub fn cmd_nop(m: &Mem, ai: u32) -> u32 {
    status(m, ai)
}

/// C: `FUN_overlay_d_96__021893b4`: command 61 "set the abort flag"; `ai[0x44] |= 2`, return `ai[0xC4]`.
pub fn cmd_set_abort(m: &mut Mem, ai: u32) -> u32 {
    let f = m.r8(ai + FLAGS) | 2;
    m.w8(ai + FLAGS, f);
    status(m, ai)
}

/// `bpp` of the slot read from the script (`0218a2fc(ai, 0218a160(ai, op))`).
fn slot_mon<E: FlowEnv>(env: &mut E, m: &Mem, ai: u32) -> u32 {
    let s = op(env);
    let pos = pos_of(env, m, ai, s);
    mon_at(env, m, ai, pos)
}

/// C: `FUN_overlay_d_96__021893dc`: command 64, `ai[0x38]` = ability of a slot (`021d5b40`).
pub fn cmd_ability<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    slot_mon(env, m, ai);
    let x = bp(env, 0x021D_5B40);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189400`: command 65, `ai[0x38]` = [`table_lookup`]`(ability, 1)`.
pub fn cmd_ability_lookup<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    slot_mon(env, m, ai);
    let x = bp(env, 0x021D_5B40);
    let r = table_lookup(env, m, ai, x & 0xFFFF, 1);
    m.w32(ai + RESULT, r);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189434`: command 66, `ai[0x38]` = value 0x12 of a slot.
pub fn cmd_value12<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let bpp = slot_mon(env, m, ai);
    let x = mon_value(env, bpp, 0x12);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__0218945c`: command 67, `ai[0x38] = 021d5bdc(slot, 0) == 0`.
pub fn cmd_5bdc0_zero<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    slot_mon(env, m, ai);
    let r = u32::from(bpv(env, 0x021D_5BDC, 0) == 0);
    m.w32(ai + RESULT, r);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__0218948c`: command 68, `ai[0x38] = 021d69ac(slot, 0)`.
pub fn cmd_69ac<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    slot_mon(env, m, ai);
    let x = bpv(env, 0x021D_69AC, 0);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__021894b4`: command 69 "copy field +0x98 (battle kind) to the result".
pub fn cmd_result_from_98(m: &mut Mem, ai: u32) -> u32 {
    let v = m.r32(ai + KIND);
    m.w32(ai + RESULT, v);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__021894c4`: command 70 "copy field +0x9C to the result".
pub fn cmd_result_from_9c(m: &mut Mem, ai: u32) -> u32 {
    let v = m.r32(ai + 0x9C);
    m.w32(ai + RESULT, v);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__021894d4`: command 71, `ai[0x38] = 021d6740(slot)` (held item?).
pub fn cmd_6740<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    slot_mon(env, m, ai);
    let x = bp(env, 0x021D_6740);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189500`: command 73, `ai[0x38]` = param 3 of move `ai[0x38]` (0 for move 0).
pub fn cmd_result_param3<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let x = m.r32(ai + RESULT);
    let r = if x != 0 {
        waza_param(env, ai, x & 0xFFFF, 3)
    } else {
        0
    };
    m.w32(ai + RESULT, r);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189524`: command 74, `ai[0x38]` = param 0x1C of move `ai[0x38]` (-1 for move 0).
pub fn cmd_result_effect<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let x = m.r32(ai + RESULT);
    let r = if x != 0 {
        waza_param(env, ai, x & 0xFFFF, 0x1C)
    } else {
        u32::MAX
    };
    m.w32(ai + RESULT, r);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189548`: command 75, `ai[0x38] = 021d679c(slot)` when its last move is 0xB6 / 0xC5 / 0xCB,
/// else 0.
pub fn cmd_protect_count<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    slot_mon(env, m, ai);
    let r = if bp(env, 0x021D_67A8) == 0xB6
        || bp(env, 0x021D_67A8) == 0xC5
        || bp(env, 0x021D_67A8) == 0xCB
    {
        bp(env, 0x021D_679C)
    } else {
        0
    };
    m.w32(ai + RESULT, r);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189590`: command 76, `goto a`.
pub fn cmd_goto<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let a = op(env);
    jump(env, m, vm, a);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__021895ac`: command 77, end the script: `ai[0x44] |= 1`, stop the VM (`02011290`); returns 1.
pub fn cmd_end<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let f = m.r8(ai + FLAGS) | 1;
    m.w8(ai + FLAGS, f);
    env.call(0x0201_1290, &mut [Arg::P]);
    1
}

/// C: `FUN_overlay_d_96__021895c4`: command 78, `if speed(self) <K[a]> speed(target): goto b` with `K = {1, 0, 2}`
/// ([`INIT_RECORD`] words 3..5, copied to the stack), value 0xF of both mons.
/// For `a >= 3` the original reads past its stack copy (the port panics); ai1.spec bounds the operand to 0..2 (Z21).
pub fn cmd_cmp_value_f<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let a = op(env);
    let b = op(env);
    let tbl = [INIT_RECORD[3], INIT_RECORD[4], INIT_RECORD[5]];
    let x = mon_value(env, m.r32(ai + SELF_MON), 0xF);
    let y = mon_value(env, m.r32(ai + TARGET_MON), 0xF);
    branch_if(env, m, vm, tbl[a as usize], x, y, b);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189640`: `if cond(target, 0xB) <kind> 1: goto b`.
pub fn if_target_cond_b<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let b = op(env);
    let x = bpv(env, 0x021D_62C4, 0xB);
    let _ = m.r32(ai + TARGET_MON);
    branch_if(env, m, vm, kind, x, 1, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__02189620`: command 79.
    cmd_79 => if_target_cond_b(2));
wrapper!(
    /// C: `FUN_overlay_d_96__02189630`: command 80.
    cmd_80 => if_target_cond_b(3));

/// C: `FUN_overlay_d_96__02189668`: command 81, `if own and iterated positions are on the same side: goto b`.
pub fn cmd_if_same_side<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let b = op(env);
    branch_if(
        env,
        m,
        vm,
        2,
        m.r8(ai + OWN_POS) & 1,
        m.r8(ai + ITER_POS) & 1,
        b,
    );
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189694`: command 82, `ai[0x38]` = the mon at a slot has type `t`.
pub fn cmd_has_type<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let s = op(env);
    let t = op(env) & 0xFF;
    let pos = pos_of(env, m, ai, s);
    let bpp = mon_at(env, m, ai, pos);
    let ty = mon_types(env, bpp);
    let r = u32::from(call(env, 0x021E_8CFC, &[ty]) == t || call(env, 0x021E_8D04, &[ty]) == t);
    m.w32(ai + RESULT, r);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__021896e0`: command 83, `ai[0x38] = mon_trait(slot) == t`.
pub fn cmd_trait_is<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let s = op(env);
    let t = op(env);
    let pos = pos_of(env, m, ai, s);
    let r = u32::from(mon_trait(env, m, ai, s, pos) == t);
    m.w32(ai + RESULT, r);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189718`: command 84, `if 021d5bdc(slot, 0xD) != 0: goto b`.
pub fn cmd_if_5bdc_d<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    mon_at(env, m, ai, pos);
    if bpv(env, 0x021D_5BDC, 0xD) != 0 {
        jump(env, m, vm, b);
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189758`: command 85, `if ability(slot) == a: goto b`.
pub fn cmd_if_ability<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let s = op(env);
    let a = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    mon_at(env, m, ai, pos);
    if a == bp(env, 0x021D_5B40) {
        jump(env, m, vm, b);
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__0218979c`: command 86, `if 021ef948(a) == 1: goto b`.
pub fn cmd_if_f948<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let a = op(env);
    let b = op(env);
    let x = call(env, 0x021E_F948, &[a]);
    branch_if(env, m, vm, 2, x, 1, b);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__021897cc`: command 87, `ai[0x38] = 021c82d0(flow, pos(slot), a)`.
pub fn cmd_side<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let s = op(env);
    let a = op(env);
    let pos = pos_of(env, m, ai, s);
    let x = call(env, 0x021C_82D0, &[m.r32(ai + FLOW), pos, a]);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189800`: command 88, for each living party member: `if hp(mon at slot) < 100: goto b`
/// (stops at the first branch; the HP read is always of the slot's mon).
pub fn cmd_if_party_hp<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let (party, mut i) = party_of(env, m, ai, pos);
    while (i as i32) < party_len(env) as i32 {
        let mon = party_mon(env, party, i & 0xFF);
        if fainted(env, mon) == 0 {
            mon_at(env, m, ai, pos);
            let hp = bp(env, 0x021D_5C44);
            if branch_if(env, m, vm, 0, hp, 100, b) != 0 {
                break;
            }
        }
        i += 1;
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__0218989c`: command 89, for each living party member: `if pp(mon at slot, j) == 0` for some `j`:
/// goto b (`021d554c`; stops at the first branch).
pub fn cmd_if_party_no_pp<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let (party, mut i) = party_of(env, m, ai, pos);
    while (i as i32) < party_len(env) as i32 {
        let mon = party_mon(env, party, i & 0xFF);
        if fainted(env, mon) == 0 {
            let mut j = 0u32;
            let me = mon_at(env, m, ai, pos);
            if move_count(env, me) as i32 > 0 {
                loop {
                    mon_at(env, m, ai, pos);
                    let x = bpv(env, 0x021D_554C, j & 0xFF);
                    if branch_if(env, m, vm, 3, x, 0, b) != 0 {
                        break;
                    }
                    j += 1;
                    let me = mon_at(env, m, ai, pos);
                    if (j as i32) >= move_count(env, me) as i32 {
                        break;
                    }
                }
            }
            let me = mon_at(env, m, ai, pos);
            if j != move_count(env, me) {
                break;
            }
        }
        i += 1;
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189978`: command 90, `ai[0x38]` = [`table_lookup`]`(ability, 0xA)` unless `cond(slot, 0x13)`
/// (else 0).
pub fn cmd_ability_lookup_a<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let s = op(env);
    let pos = pos_of(env, m, ai, s);
    m.w32(ai + RESULT, 0);
    mon_at(env, m, ai, pos);
    if bpv(env, 0x021D_62C4, 0x13) == 0 {
        mon_at(env, m, ai, pos);
        let x = bp(env, 0x021D_5B40);
        let r = table_lookup(env, m, ai, x & 0xFFFF, 0xA);
        m.w32(ai + RESULT, r);
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__021899c4`: command 91, `ai[0x38] = 021d5584(self, slot)` (the current slot is usable?).
pub fn cmd_slot_usable<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let x = bpv(env, 0x021D_5584, m.r8(ai + 1));
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__021899dc`: command 92, `if 021d5478(slot) >= move count && move count > 1: goto b`.
pub fn cmd_if_5478<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let bpp = mon_at(env, m, ai, pos);
    let a = bp(env, 0x021D_5478);
    if a >= move_count(env, bpp) && move_count(env, bpp) > 1 {
        jump(env, m, vm, b);
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189a2c`: command 93, `ai[0x38]` = param 2 of the current move.
pub fn cmd_move_param2<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let x = waza_param(env, ai, m.r16(ai + 2), 2);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189a44`: command 94, `ai[0x38]` = param 2 of the target's last move (0 for none).
pub fn cmd_target_last_param2<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let w = bp(env, 0x021D_67A8);
    let r = if w != 0 { waza_param(env, ai, w, 2) } else { 0 };
    m.w32(ai + RESULT, r);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189a6c`: command 95, `ai[0x38] = 021c81d0(flow, mon at slot, 1)`.
pub fn cmd_81d0<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let bpp = slot_mon(env, m, ai);
    let x = env.call(0x021C_81D0, &mut [Arg::P, Arg::V(bpp), Arg::V(1)]);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189a9c`: command 96, `ai[0x38] = 021d5ba4(slot)`.
pub fn cmd_5ba4<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    slot_mon(env, m, ai);
    let x = bp(env, 0x021D_5BA4);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189ac0`: command 97, `if some living party member outscores self against the target: goto b`
/// ([`best_score`]).
pub fn cmd_if_party_better<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let x = op(env);
    let b = op(env);
    let (party, mut i) = party_of(env, m, ai, m.r8(ai + OWN_POS));
    let base = best_score(env, m, ai, m.r32(ai + SELF_MON), m.r32(ai + TARGET_MON), x);
    while (i as i32) < party_len(env) as i32 {
        let mon = party_mon(env, party, i & 0xFF);
        mon_id(env, mon);
        if fainted(env, mon) == 0 && best_score(env, m, ai, mon, m.r32(ai + TARGET_MON), x) > base {
            jump(env, m, vm, b);
            break;
        }
        i += 1;
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189b78`: command 98, `if some own move is super effective (021c7dac class 4 or 5): goto b`.
pub fn cmd_if_super_effective<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let b = op(env);
    let me = m.r32(ai + SELF_MON);
    let mut i = 0u32;
    while (i as i32) < move_count(env, me) as i32 {
        let w = move_at(env, me, i);
        let ia = mon_id(env, me);
        let ib = mon_id(env, m.r32(ai + TARGET_MON));
        let x = eff7dac(env, m.r32(ai + FLOW), ia, ib, w);
        if x.wrapping_sub(4) <= 1 {
            jump(env, m, vm, b);
            break;
        }
        i += 1;
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189bfc`: command 99, `if score(last move of the mon at slot vs target) > best own score: goto b`.
pub fn cmd_if_last_move_beats<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let s = op(env);
    let x = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let bpp = mon_at(env, m, ai, pos);
    let base = best_score(env, m, ai, m.r32(ai + SELF_MON), m.r32(ai + TARGET_MON), x);
    let w = bp(env, 0x021D_67A8);
    let ia = mon_id(env, bpp);
    let ib = mon_id(env, m.r32(ai + TARGET_MON));
    let y = score7e14(env, m.r32(ai + FLOW), ia, ib, w, 1, x);
    if base < y {
        jump(env, m, vm, b);
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189c8c`: command 100, `ai[0x38]` = sum over the stat-stage values 1..7 (ids from
/// [`INIT_RECORD`] words 6..12) of a slot of `max(v - 6, 0)` (boosts above neutral).
pub fn cmd_sum_boosts<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let s = op(env);
    let pos = pos_of(env, m, ai, s);
    m.w32(ai + RESULT, 0);
    for k in 0..7 {
        let bpp = mon_at(env, m, ai, pos);
        let v = mon_value(env, bpp, INIT_RECORD[6 + k]);
        if v as i32 > 6 {
            let r = m.r32(ai + RESULT).wrapping_add(v - 6);
            m.w32(ai + RESULT, r);
        }
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189cec`: command 101, `ai[0x38] = value(slot, id) - value(self, id)`.
pub fn cmd_value_diff<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let s = op(env);
    let id = op(env);
    let pos = pos_of(env, m, ai, s);
    let bpp = mon_at(env, m, ai, pos);
    let a = mon_value(env, bpp, id);
    let b = mon_value(env, m.r32(ai + SELF_MON), id);
    m.w32(ai + RESULT, a.wrapping_sub(b));
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189d48`: command 105, `ai[0x38]` = 0 (self not in the party / current move scores 0), 2 (no
/// other party member does better), 1 (another `021d5b7c` party member has a move that outscores the current one).
pub fn cmd_party_outscores<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let x = op(env);
    let (party, cnt) = party_of(env, m, ai, m.r8(ai + OWN_POS));
    let me = env.call(0x021B_9C98, &mut [Arg::P, Arg::V(m.r32(ai + SELF_MON))]);
    let ia = mon_id(env, m.r32(ai + SELF_MON));
    let ib = mon_id(env, m.r32(ai + TARGET_MON));
    let base = score7e14(env, m.r32(ai + FLOW), ia, ib, m.r16(ai + 2), 1, x);
    if (me as i32) < 0 || base == 0 {
        m.w32(ai + RESULT, 0);
        return status(m, ai);
    }
    let mut j = 0u32;
    while (j as i32) < cnt as i32 {
        let mon = party_mon(env, party, j & 0xFF);
        let im = mon_id(env, mon);
        if mon_5b7c(env, mon) != 0 && j != me {
            m.w32(ai + RESULT, 2);
            let mut i = 0u32;
            while (i as i32) < move_count(env, mon) as i32 {
                let w = move_at(env, mon, i);
                if i != m.r8(ai + 1) && score7e14(env, m.r32(ai + FLOW), im, ib, w, 1, x) > base {
                    m.w32(ai + RESULT, 1);
                    break;
                }
                i += 1;
            }
        }
        j += 1;
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189e90`: `if fainted(slot) <kind> 1: goto b`.
pub fn if_fainted<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let bpp = mon_at(env, m, ai, pos);
    let x = fainted(env, bpp);
    branch_if(env, m, vm, kind, x, 1, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__02189e70`: command 106.
    cmd_106 => if_fainted(2));
wrapper!(
    /// C: `FUN_overlay_d_96__02189e80`: command 107.
    cmd_107 => if_fainted(3));

/// C: `FUN_overlay_d_96__02189ed0`: command 108, `ai[0x38]` = value 0x11 of a slot.
pub fn cmd_value11<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    let bpp = slot_mon(env, m, ai);
    let x = mon_value(env, bpp, 0x11);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189ef8`: command 109, `if 021d6d7c(slot) != 0: goto b`.
pub fn cmd_if_6d7c<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    mon_at(env, m, ai, pos);
    if bp(env, 0x021D_6D7C) != 0 {
        jump(env, m, vm, b);
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189f34`: command 110, `ai[0x38]` = species of a slot (`021d5444`).
pub fn cmd_species<E: FlowEnv>(env: &mut E, m: &mut Mem, ai: u32) -> u32 {
    slot_mon(env, m, ai);
    let x = bp(env, 0x021D_5444);
    m.w32(ai + RESULT, x);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__02189f98`: `if ai[0xCC] <kind> a: goto b` (the per-turn random word).
pub fn if_turn_rand<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let a = op(env);
    let b = op(env);
    branch_if(env, m, vm, kind, m.r32(ai + 0xCC), a, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__02189f58`: command 111.
    cmd_111 => if_turn_rand(0));
wrapper!(
    /// C: `FUN_overlay_d_96__02189f68`: command 112.
    cmd_112 => if_turn_rand(1));
wrapper!(
    /// C: `FUN_overlay_d_96__02189f78`: command 113.
    cmd_113 => if_turn_rand(2));
wrapper!(
    /// C: `FUN_overlay_d_96__02189f88`: command 114.
    cmd_114 => if_turn_rand(3));

/// C: `FUN_overlay_d_96__02189fc0`: command 115, a jump table on the current move's effect: mode != 0 or effect > `thr`
/// ends the script ([`cmd_end`], returns 1); else jumps to `tbl + tbl[effect]` (script u32 table at `vm[0x14] + t`).
pub fn cmd_switch_effect<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32) -> u32 {
    let mode = op(env);
    let thr = op(env);
    let t = op(env);
    let tbl = m.r32(vm + VM_BASE).wrapping_add(t);
    let mut idx = 0u32;
    let stop = if mode != 0 {
        true
    } else {
        let x = waza_param(env, ai, m.r16(ai + 2), 0x1C);
        idx = x << 1;
        let v = ((idx as i32).wrapping_add((idx >> 31) as i32)) >> 1;
        v > thr as i32
    };
    if stop {
        return cmd_end(env, m, ai);
    }
    let off = m.r16(tbl.wrapping_add(idx << 1)) | (m.r16(tbl.wrapping_add((idx + 1) << 1)) << 16);
    env.call(0x0201_13B0, &mut [Arg::P, Arg::V(tbl.wrapping_add(off))]);
    status(m, ai)
}

/// C: `FUN_overlay_d_96__0218a034`: command 116, `if 021c82f4(flow, pos(slot), 3) != 0: goto b`.
pub fn cmd_if_82f4<E: FlowEnv>(env: &mut E, m: &Mem, vm: u32, ai: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    if call(env, 0x021C_82F4, &[m.r32(ai + FLOW), pos, 3]) != 0 {
        jump(env, m, vm, b);
    }
    status(m, ai)
}

/// C: `FUN_overlay_d_96__0218a0a4`: `if pos(slot) <kind> value(slot, 0xA): goto b` (value 8 read and dropped first).
pub fn if_pos_vs_value<E: FlowEnv>(env: &mut E, m: &mut Mem, vm: u32, ai: u32, kind: u32) -> u32 {
    let s = op(env);
    let b = op(env);
    let pos = pos_of(env, m, ai, s);
    let bpp = mon_at(env, m, ai, pos);
    mon_value(env, bpp, 8);
    let bpp = mon_at(env, m, ai, pos);
    let x = mon_value(env, bpp, 0xA);
    branch_if(env, m, vm, kind, pos, x, b)
}

wrapper!(
    /// C: `FUN_overlay_d_96__0218a074`: command 117.
    cmd_117 => if_pos_vs_value(0));
wrapper!(
    /// C: `FUN_overlay_d_96__0218a084`: command 118.
    cmd_118 => if_pos_vs_value(1));
wrapper!(
    /// C: `FUN_overlay_d_96__0218a094`: command 119.
    cmd_119 => if_pos_vs_value(2));

#[cfg(test)]
mod tests;
